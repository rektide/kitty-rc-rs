//! Typed kitty remote-control commands and output models.
//!
//! This crate implements the finite command catalog on the accepted redesign's
//! terms: a command binds its typed payload construction to a
//! [`PlainRequest`](kitty_rc_schema::PlainRequest) and its typed output
//! decoding runs only after the remote outcome has been validated. It
//! implements audited domain families rather than drifted legacy builders.
//! Upload execution is deliberately deferred. New commands follow this
//! pattern against upstream-derived fixtures, never by treating an old
//! serializer as an oracle.
//!
//! Decoding law used throughout: **missing required data is an error, never
//! an empty success.** Execution classifies native errors before invoking
//! [`Command::decode`]. The optional pure [`ls_from_response`] helper likewise
//! reports [`LsError::RemoteFailure`] instead of fabricating an empty listing.
//!
//! ```
//! use kitty_rc_commands::{Command, Submission};
//! use kitty_rc_commands::actions::{Action, KnownAction};
//! use kitty_rc_commands::configuration::LoadConfig;
//! use kitty_rc_commands::targets::{WindowId, WindowTarget};
//! let action = Action::builder()
//!     .action(KnownAction::GotoTab(2))
//!     .target(WindowTarget::Id(WindowId(7)))
//!     .build();
//! assert_eq!(action.request().cmd, "action");
//! // Deliberate suppression acknowledges only local submission.
//! let config = Submission(LoadConfig::builder()
//!     .overrides(vec!["font_size=16".into()]).build());
//! assert!(!config.reusable());
//! ```

pub mod actions;
pub mod appearance;
pub mod catalog;
pub mod configuration;
pub mod outputs;
pub mod process;
pub mod tab;
pub mod targets;
pub mod window;

use kitty_rc_schema::{PlainRequest, ResponseEnvelope, ResponseKind};
use outputs::OsWindow;

/// Scheduling class, not a claim about parallel server CPU execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class {
    Short,
    Observation,
    LongWait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Reply,
    Deferred,
    SubmissionOnly,
}
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("required command data missing or invalid")]
    MissingData,
    #[error("typed JSON data decode failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("custom command decoder failed: {0}")]
    Custom(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Commands bind their output decoder and finite wire conversation.
/// Decoders receive only a classified success's data, never a remote error.
pub trait Command: Send + 'static {
    type Output: Send + 'static;
    fn request(&self) -> PlainRequest;
    fn class(&self) -> Class;
    fn mode(&self) -> Mode {
        Mode::Reply
    }
    /// Only audited single-terminal-reply profiles opt into sequential reuse.
    fn reusable(&self) -> bool {
        false
    }
    fn decode(data: Option<serde_json::Value>) -> Result<Self::Output, DecodeError>;
}

impl Command for Ls {
    fn reusable(&self) -> bool {
        true
    }
    type Output = Vec<OsWindow>;
    fn request(&self) -> PlainRequest {
        self.to_request()
    }
    fn class(&self) -> Class {
        Class::Observation
    }
    fn decode(data: Option<serde_json::Value>) -> Result<Self::Output, DecodeError> {
        Ok(outputs::parse_ls_data(
            &data.ok_or(DecodeError::MissingData)?,
        )?)
    }
}

/// A finite screen snapshot. Only baseline screen/all/selection extents are
/// exposed; HEAD-only alternates are not silently asserted as supported.
#[derive(Debug, Clone, Copy, Default)]
pub enum TextExtent {
    #[default]
    Screen,
    All,
    Selection,
    FirstCommandOutputOnScreen,
    LastCommandOutput,
    LastVisitedCommandOutput,
    LastNonEmptyOutput,
    /// Observed since kitty 0.48; not a global support-floor claim.
    Alternate,
    AlternateScrollback,
}

impl TextExtent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::All => "all",
            Self::Selection => "selection",
            Self::FirstCommandOutputOnScreen => "first_cmd_output_on_screen",
            Self::LastCommandOutput => "last_cmd_output",
            Self::LastVisitedCommandOutput => "last_visited_cmd_output",
            Self::LastNonEmptyOutput => "last_non_empty_output",
            Self::Alternate => "alternate",
            Self::AlternateScrollback => "alternate_scrollback",
        }
    }
}

#[derive(Debug, Clone, Default, bon::Builder)]
#[builder(on(String, into))]
pub struct GetText {
    pub match_spec: Option<String>,
    #[builder(default)]
    pub extent: TextExtent,
    #[builder(default)]
    pub ansi: bool,
    #[builder(default)]
    pub cursor: bool,
    #[builder(default)]
    pub wrap_markers: bool,
    #[builder(default)]
    pub clear_selection: bool,
    #[builder(default)]
    pub only_self: bool,
}
impl Command for GetText {
    fn reusable(&self) -> bool {
        true
    }
    type Output = String;
    fn request(&self) -> PlainRequest {
        let mut r = PlainRequest::new("get-text");
        r.payload = Some(
            serde_json::json!({"match": self.match_spec, "extent": self.extent.as_str(), "ansi": self.ansi, "cursor": self.cursor, "wrap_markers": self.wrap_markers, "clear_selection": self.clear_selection, "self": self.only_self}),
        );
        r
    }
    fn class(&self) -> Class {
        Class::Observation
    }
    fn decode(data: Option<serde_json::Value>) -> Result<String, DecodeError> {
        decode_string(data)
    }
}

#[derive(Debug, Clone, bon::Builder)]
#[builder(on(String, into))]
pub struct FocusWindow {
    pub match_spec: String,
}
impl Command for FocusWindow {
    fn reusable(&self) -> bool {
        true
    }
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut r = PlainRequest::new("focus-window");
        r.payload = Some(serde_json::json!({"match": self.match_spec}));
        r
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn decode(_: Option<serde_json::Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}

/// Wait for a launched child's exit. The output is upstream's exit-code or
/// signal-name string, not a window ID. Cancel is local, not child rollback.
#[derive(Debug, Clone, bon::Builder)]
pub struct LaunchWait {
    pub args: Vec<String>,
}
impl Command for LaunchWait {
    fn reusable(&self) -> bool {
        true
    }
    type Output = String;
    fn request(&self) -> PlainRequest {
        let mut r = PlainRequest::new("launch");
        r.payload = Some(
            serde_json::json!({"args": self.args, "wait_for_child_to_exit": true, "type": "window"}),
        );
        r
    }
    fn class(&self) -> Class {
        Class::LongWait
    }
    fn mode(&self) -> Mode {
        Mode::Deferred
    }
    fn decode(data: Option<serde_json::Value>) -> Result<String, DecodeError> {
        decode_string(data)
    }
}

fn decode_string(data: Option<serde_json::Value>) -> Result<String, DecodeError> {
    match data {
        Some(serde_json::Value::String(s)) => Ok(s),
        _ => Err(DecodeError::MissingData),
    }
}

pub(crate) fn request(name: &str, payload: serde_json::Value) -> PlainRequest {
    let mut request = PlainRequest::new(name);
    request.payload = Some(payload);
    request
}

/// Only commands whose audited native contract permits reply suppression.
/// This does not establish remote execution or replay safety.
pub trait CanSubmitWithoutReply: Command {}

/// Deliberately suppress the native reply of an eligible command. A completed
/// submission proves only local outbound completion; its transport retires.
#[derive(Debug, Clone)]
pub struct Submission<C: CanSubmitWithoutReply>(pub C);
impl<C: CanSubmitWithoutReply> Command for Submission<C> {
    type Output = ();
    fn request(&self) -> PlainRequest {
        self.0.request()
    }
    fn class(&self) -> Class {
        self.0.class()
    }
    fn mode(&self) -> Mode {
        Mode::SubmissionOnly
    }
    fn decode(_: Option<serde_json::Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}
impl CanSubmitWithoutReply for FocusWindow {}

/// Session serialization is a separate unfiltered listing result profile;
/// upstream ignores output_format=session when match/self filters are supplied.
#[derive(Debug, Clone, Copy, Default)]
pub struct SessionText;
impl Command for SessionText {
    type Output = String;
    fn request(&self) -> PlainRequest {
        request(
            "ls",
            serde_json::json!({"output_format": "session", "all_env_vars": false}),
        )
    }
    fn class(&self) -> Class {
        Class::Observation
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<serde_json::Value>) -> Result<String, DecodeError> {
        decode_string(data)
    }
}

/// Explicit finite escape hatch. Stream/async/no_response envelope flags are
/// controlled by execution, not arbitrary raw input. No replay is inferred.
#[derive(Debug, Clone)]
pub struct RawFinite {
    pub name: String,
    pub payload: serde_json::Value,
    pub class: Class,
    pub submission_only: bool,
}
impl Command for RawFinite {
    type Output = Option<serde_json::Value>;
    fn request(&self) -> PlainRequest {
        let mut r = PlainRequest::new(&self.name);
        r.payload = Some(self.payload.clone());
        r
    }
    fn class(&self) -> Class {
        self.class
    }
    fn mode(&self) -> Mode {
        if self.submission_only {
            Mode::SubmissionOnly
        } else {
            Mode::Reply
        }
    }
    fn decode(data: Option<serde_json::Value>) -> Result<Self::Output, DecodeError> {
        Ok(data)
    }
}

/// Errors from decoding an `ls` outcome.
#[derive(Debug, thiserror::Error)]
pub enum LsError {
    /// The server answered `ok: false`; `error` is its message when present.
    /// This is the command's own negative answer, not a transport failure and
    /// not proof of zero side effects.
    #[error("remote ls failure: {error}")]
    RemoteFailure { error: String },

    /// The reply carried no `data` field; that is missing evidence, not an
    /// empty listing.
    #[error("ls reply carried no data")]
    MissingData,

    /// The reply did not match any shape the inspected server produces.
    #[error("ls reply envelope was structurally invalid: {0}")]
    InvalidEnvelope(#[source] kitty_rc_schema::SchemaError),

    #[error("ls data decode failed: {0}")]
    Decode(#[from] serde_json::Error),
}

/// `ls`: list operating-system windows, their tabs, and windows.
///
/// Wire payload per `kitty/rc/ls.py`: `all_env_vars`, `match`, `match_tab`
/// (and `self`). The response's `data` is the JSON listing encoded as a
/// **string** (unless `output_format: session` was requested, which is a
/// separate result interpretation this type does not claim).
#[derive(Debug, Clone, Default, bon::Builder)]
#[builder(on(String, into))]
pub struct Ls {
    /// Send every window's full environment instead of only differing
    /// variables.
    #[builder(default)]
    pub all_env_vars: bool,
    /// Restrict the listing to windows matching a kitty window expression.
    pub match_spec: Option<String>,
    /// Restrict the listing to tabs matching a kitty tab expression.
    pub match_tab: Option<String>,
    /// List only the window the command runs in.
    #[builder(default)]
    pub only_self: bool,
}

impl Ls {
    /// The outbound request for this command at the declared supported
    /// version.
    pub fn to_request(&self) -> PlainRequest {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "all_env_vars".to_string(),
            serde_json::Value::Bool(self.all_env_vars),
        );
        if let Some(value) = &self.match_spec {
            payload.insert(
                "match".to_string(),
                serde_json::Value::String(value.clone()),
            );
        }
        if let Some(value) = &self.match_tab {
            payload.insert(
                "match_tab".to_string(),
                serde_json::Value::String(value.clone()),
            );
        }
        if self.only_self {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        let mut request = PlainRequest::new("ls");
        request.payload = Some(serde_json::Value::Object(payload));
        request
    }
}

/// Decode the typed listing from a classified terminal response.
pub fn ls_from_response(envelope: &ResponseEnvelope) -> Result<Vec<OsWindow>, LsError> {
    match envelope.classify() {
        Ok(ResponseKind::Failure { error }) => Err(LsError::RemoteFailure {
            error: error.unwrap_or_default(),
        }),
        Ok(ResponseKind::StreamAcknowledged) | Ok(ResponseKind::Success { data: None }) => {
            Err(LsError::MissingData)
        }
        Ok(ResponseKind::Success { data: Some(data) }) => Ok(outputs::parse_ls_data(&data)?),
        Err(error) => Err(LsError::InvalidEnvelope(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn first_slice_profiles_match_inspected_handlers() {
        let text = GetText::default().request();
        assert_eq!(text.cmd, "get-text");
        assert_eq!(text.payload.unwrap()["extent"], "screen");
        let focus = FocusWindow {
            match_spec: "id:7".into(),
        }
        .request();
        assert_eq!(focus.payload, Some(json!({"match": "id:7"})));
        let wait = LaunchWait {
            args: vec!["sh".into(), "-c".into(), "exit 7".into()],
        };
        assert_eq!(wait.mode(), Mode::Deferred);
        assert_eq!(
            wait.request().payload,
            Some(
                json!({"args": ["sh", "-c", "exit 7"], "wait_for_child_to_exit": true, "type": "window"})
            )
        );
        assert_eq!(
            LaunchWait::decode(Some(json!("SIGTERM"))).unwrap(),
            "SIGTERM"
        );
        assert!(GetText::decode(None).is_err());
    }

    /// Payload field names follow the upstream ls protocol spec.
    #[test]
    fn request_payload_uses_upstream_field_names() {
        let request = Ls::builder()
            .all_env_vars(true)
            .match_spec("id:1")
            .match_tab("title:work")
            .only_self(true)
            .build()
            .to_request();

        assert_eq!(request.cmd, "ls");
        assert_eq!(request.version, kitty_rc_schema::SUPPORTED);
        assert_eq!(
            request.payload,
            Some(json!({
                "all_env_vars": true,
                "match": "id:1",
                "match_tab": "title:work",
                "self": true,
            }))
        );
    }

    /// Defaults omit optional filters; all_env_vars stays explicit like the
    /// reference client.
    #[test]
    fn default_request_is_minimal() {
        let request = Ls::builder().build().to_request();
        assert_eq!(request.payload, Some(json!({"all_env_vars": false})));
    }

    /// Upstream-derived fixture: the response `data` is a JSON *string*
    /// containing the listing; window fields include list-valued `cmdline`
    /// and nested `foreground_processes` with their own `cmdline` lists.
    #[test]
    fn decodes_real_shaped_ls_data() {
        let listing = json!([{
            "id": 1,
            "is_active": true,
            "is_focused": true,
            "platform_window_id": 440402,
            "tabs": [{
                "id": 2,
                "is_active": true,
                "is_focused": true,
                "title": "zsh",
                "layout": "fat",
                "enabled_layouts": ["fat", "stack"],
                "active_window_history": [3],
                "windows": [{
                    "id": 3,
                    "is_active": true,
                    "is_focused": true,
                    "is_self": true,
                    "title": "ls — kitty",
                    "title_overridden": false,
                    "pid": 4242,
                    "cwd": "/home/user/src",
                    "cmdline": ["/usr/bin/zsh"],
                    "last_reported_cmdline": null,
                    "last_cmd_exit_status": 0,
                    "env": {"TERM": "xterm-kitty"},
                    "user_vars": {},
                    "foreground_processes": [{
                        "pid": 4243,
                        "cwd": "/home/user/src",
                        "cmdline": ["ls", "--color"],
                    }],
                    "at_prompt": false,
                    "in_alternate_screen": false,
                    "lines": 40,
                    "columns": 120,
                    "created_at": 1790000000.123,
                    "neighbors": {"left": null},
                    "session_name": null,
                }],
            }],
        }])
        .to_string();

        let response: ResponseEnvelope =
            serde_json::from_value(json!({"ok": true, "data": listing})).unwrap();
        let instances = ls_from_response(&response).unwrap();

        assert_eq!(instances.len(), 1);
        let instance = &instances[0];
        assert_eq!(instance.id, Some(1));
        assert_eq!(instance.is_active, Some(true));
        let tab = &instance.tabs[0];
        assert_eq!(tab.id, Some(2));
        assert_eq!(tab.layout.as_deref(), Some("fat"));
        let window = &tab.windows[0];
        assert_eq!(window.id, Some(3));
        assert_eq!(window.pid, Some(4242));
        assert_eq!(window.cmdline, vec!["/usr/bin/zsh".to_string()]);
        assert_eq!(
            window.env.get("TERM").map(String::as_str),
            Some("xterm-kitty")
        );
        let process = &window.foreground_processes[0];
        assert_eq!(process.pid, Some(4243));
        assert_eq!(
            process.cmdline,
            vec!["ls".to_string(), "--color".to_string()]
        );
        // Additive fields present in newer servers (neighbors,
        // title_overridden, session_name) are tolerated without modeling.
    }

    /// A remote failure surfaces as RemoteFailure, not an empty listing.
    #[test]
    fn remote_failure_is_not_empty_success() {
        let response: ResponseEnvelope =
            serde_json::from_value(json!({"ok": false, "error": "Permission denied"})).unwrap();
        match ls_from_response(&response) {
            Err(LsError::RemoteFailure { error }) => assert_eq!(error, "Permission denied"),
            other => panic!("expected remote failure, got {other:?}"),
        }
    }

    /// A success without data is missing evidence, never an empty listing.
    #[test]
    fn missing_data_is_not_empty_success() {
        let response: ResponseEnvelope = serde_json::from_value(json!({"ok": true})).unwrap();
        assert!(matches!(
            ls_from_response(&response),
            Err(LsError::MissingData)
        ));
    }
}

//! Typed kitty remote-control commands and output models.
//!
//! This crate is the seed of the command catalog on the accepted redesign's
//! terms: a command binds its typed payload construction to a
//! [`PlainRequest`](kitty_rc_schema::PlainRequest) and its typed output
//! decoding runs only after the remote outcome has been validated. It
//! implements the finite first slice rather than a migrated pile of drifted
//! legacy builders. New commands follow this
//! pattern against upstream-derived fixtures, never by treating an old
//! serializer as an oracle.
//!
//! Decoding law used throughout: **missing required data is an error, never
//! an empty success.** A remote `ok: false` reaches the caller as
//! [`LsError::RemoteFailure`] instead of becoming an empty listing.

pub mod outputs;

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

/// Commands bind their output decoder and finite wire conversation.
/// Decoders receive only a classified success's data, never a remote error.
pub trait Command: Send + 'static {
    type Output: Send + 'static;
    fn request(&self) -> PlainRequest;
    fn class(&self) -> Class;
    fn mode(&self) -> Mode {
        Mode::Reply
    }
    fn decode(data: Option<serde_json::Value>) -> Result<Self::Output, String>;
}

impl Command for Ls {
    type Output = Vec<OsWindow>;
    fn request(&self) -> PlainRequest {
        self.to_request()
    }
    fn class(&self) -> Class {
        Class::Observation
    }
    fn decode(data: Option<serde_json::Value>) -> Result<Self::Output, String> {
        outputs::parse_ls_data(&data.ok_or("ls reply carried no data")?).map_err(|e| e.to_string())
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
}

#[derive(Debug, Clone, Default)]
pub struct GetText {
    pub match_spec: Option<String>,
    pub extent: TextExtent,
}
impl Command for GetText {
    type Output = String;
    fn request(&self) -> PlainRequest {
        let mut r = PlainRequest::new("get-text");
        r.payload = Some(
            serde_json::json!({"match": self.match_spec, "extent": match self.extent { TextExtent::Screen => "screen", TextExtent::All => "all", TextExtent::Selection => "selection" }, "ansi": false, "cursor": false, "wrap_markers": false, "clear_selection": false, "self": false}),
        );
        r
    }
    fn class(&self) -> Class {
        Class::Observation
    }
    fn decode(data: Option<serde_json::Value>) -> Result<String, String> {
        decode_string(data)
    }
}

#[derive(Debug, Clone)]
pub struct FocusWindow {
    pub match_spec: String,
}
impl Command for FocusWindow {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut r = PlainRequest::new("focus-window");
        r.payload = Some(serde_json::json!({"match": self.match_spec}));
        r
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn decode(_: Option<serde_json::Value>) -> Result<(), String> {
        Ok(())
    }
}

/// Wait for a launched child's exit. The output is upstream's exit-code or
/// signal-name string, not a window ID. Cancel is local, not child rollback.
#[derive(Debug, Clone)]
pub struct LaunchWait {
    pub args: Vec<String>,
}
impl Command for LaunchWait {
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
    fn decode(data: Option<serde_json::Value>) -> Result<String, String> {
        decode_string(data)
    }
}

fn decode_string(data: Option<serde_json::Value>) -> Result<String, String> {
    match data {
        Some(serde_json::Value::String(s)) => Ok(s),
        _ => Err("required string data missing or invalid".into()),
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
    fn decode(data: Option<serde_json::Value>) -> Result<Self::Output, String> {
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
    InvalidEnvelope(String),

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
        Err(error) => Err(LsError::InvalidEnvelope(error.to_string())),
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

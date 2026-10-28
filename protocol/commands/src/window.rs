//! Finite window operations. Acknowledgment does not observe eventual removal,
//! layout redraw, child delivery or window-manager compliance.

use crate::targets::{
    Assignment, Finite, KeyChord, Signal, TabId, TabMatch, TabTarget, WindowId, WindowSet,
    WindowTarget,
};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, Mode, request};
use base64::{Engine, engine::general_purpose::STANDARD};
use kitty_rc_schema::PlainRequest;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkerKind {
    Text,
    Itext,
    Regex,
    Iregex,
}
#[derive(Debug, Clone, Copy)]
pub enum MarkerGroup {
    One,
    Two,
    Three,
}
impl MarkerGroup {
    pub fn number(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MarkerSpec {
    Patterns {
        kind: MarkerKind,
        first: (MarkerGroup, String),
        additional: Vec<(MarkerGroup, String)>,
    },
    /// Server-loaded marker function; availability and code are caller-owned.
    Function(String),
}
impl MarkerSpec {
    pub fn arguments(&self) -> Vec<String> {
        match self {
            Self::Function(s) => vec!["function".into(), s.clone()],
            Self::Patterns {
                kind,
                first,
                additional,
            } => {
                let mut ans = vec![
                    match kind {
                        MarkerKind::Text => "text",
                        MarkerKind::Itext => "itext",
                        MarkerKind::Regex => "regex",
                        MarkerKind::Iregex => "iregex",
                    }
                    .into(),
                ];
                for (group, pattern) in std::iter::once(first).chain(additional.iter()) {
                    ans.push(group.number().to_string());
                    ans.push(pattern.clone());
                }
                ans
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum DetachDestination {
    NewOsWindow,
    NewTab,
    Tab(TabId),
    Match(String),
}
impl DetachDestination {
    pub fn tab(tab: impl Into<TabMatch>) -> Self {
        Self::Match(tab.into().to_string())
    }
    fn wire(&self) -> Option<String> {
        match self {
            Self::NewOsWindow => None,
            Self::NewTab => Some("new".into()),
            Self::Tab(id) => Some(format!("id:{}", id.0)),
            Self::Match(s) => Some(s.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OsWindowAction {
    Resize,
    ToggleFullscreen,
    ToggleMaximized,
    ToggleVisibility,
    Hide,
    Show,
    OsPanel,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SizeUnit {
    Cells,
    Pixels,
}

#[derive(Debug, Clone)]
pub enum ScrollAmount {
    Start,
    End,
    Lines(Finite),
    Pages(Finite),
    Unscroll(u32),
    Prompts(i32),
}
impl ScrollAmount {
    fn wire(&self) -> Value {
        match self {
            Self::Start => json!(["start", null]),
            Self::End => json!(["end", null]),
            Self::Lines(n) => json!([n, "l"]),
            Self::Pages(n) => json!([n, "p"]),
            Self::Unscroll(n) => json!([n, "u"]),
            Self::Prompts(n) => json!([n, "r"]),
        }
    }
}

/// One grouped finite acknowledgment family, not one marker wrapper per method.
#[derive(Debug, Clone, bon::Builder)]
pub struct WindowCommand {
    #[builder(default, into)]
    pub target: WindowTarget,
    pub operation: WindowOperation,
}
impl WindowCommand {
    pub fn new(target: impl Into<WindowTarget>, operation: WindowOperation) -> Self {
        Self {
            target: target.into(),
            operation,
        }
    }
    pub fn close(target: impl Into<WindowTarget>) -> Self {
        Self::new(
            target,
            WindowOperation::Close {
                ignore_no_match: false,
            },
        )
    }
    pub fn title(target: impl Into<WindowTarget>, title: impl Into<String>) -> Self {
        Self::new(
            target,
            WindowOperation::Title {
                title: Some(title.into()),
                temporary: false,
            },
        )
    }
    pub fn reset_title(target: impl Into<WindowTarget>) -> Self {
        Self::new(
            target,
            WindowOperation::Title {
                title: None,
                temporary: false,
            },
        )
    }
    pub fn os_title(target: impl Into<WindowTarget>, title: impl Into<String>) -> Self {
        Self::new(
            target,
            WindowOperation::OsTitle {
                title: Some(title.into()),
            },
        )
    }
    pub fn scroll(target: impl Into<WindowTarget>, amount: ScrollAmount) -> Self {
        Self::new(target, WindowOperation::Scroll(amount))
    }
    pub fn signal(
        target: impl Into<WindowTarget>,
        signals: impl IntoIterator<Item = Signal>,
    ) -> Self {
        Self::new(
            target,
            WindowOperation::Signal(signals.into_iter().collect()),
        )
    }
    pub fn detach(target: impl Into<WindowTarget>, destination: DetachDestination) -> Self {
        Self::new(
            target,
            WindowOperation::Detach {
                destination,
                stay_in_tab: false,
            },
        )
    }
    pub fn create_marker(target: impl Into<WindowTarget>, spec: MarkerSpec) -> Self {
        Self::new(target, WindowOperation::CreateMarker(spec))
    }
    pub fn remove_marker(target: impl Into<WindowTarget>) -> Self {
        Self::new(target, WindowOperation::RemoveMarker)
    }
}
#[derive(Debug, Clone)]
pub enum WindowOperation {
    Close {
        ignore_no_match: bool,
    },
    Title {
        title: Option<String>,
        temporary: bool,
    },
    OsTitle {
        title: Option<String>,
    },
    CreateMarker(MarkerSpec),
    RemoveMarker,
    Detach {
        destination: DetachDestination,
        stay_in_tab: bool,
    },
    Scroll(ScrollAmount),
    Signal(Vec<Signal>),
    RemoveLogo,
    OsWindow {
        action: OsWindowAction,
        unit: SizeUnit,
        width: i32,
        height: i32,
        incremental: bool,
        panel_settings: Vec<String>,
    },
}
impl Command for WindowCommand {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut p = self.target.payload();
        let name = match &self.operation {
            WindowOperation::Close { ignore_no_match } => {
                p["ignore_no_match"] = json!(ignore_no_match);
                "close-window"
            }
            WindowOperation::Title { title, temporary } => {
                if let Some(title) = title {
                    p["title"] = json!(title);
                }
                p["temporary"] = json!(temporary);
                "set-window-title"
            }
            WindowOperation::OsTitle { title } => {
                if let Some(title) = title {
                    p["title"] = json!(title);
                }
                "set-os-window-title"
            }
            WindowOperation::CreateMarker(spec) => {
                p["marker_spec"] = json!(spec.arguments());
                "create-marker"
            }
            WindowOperation::RemoveMarker => "remove-marker",
            WindowOperation::Detach {
                destination,
                stay_in_tab,
            } => {
                p["target_tab"] = json!(destination.wire());
                p["stay_in_tab"] = json!(stay_in_tab);
                "detach-window"
            }
            WindowOperation::Scroll(amount) => {
                p["amount"] = amount.wire();
                "scroll-window"
            }
            WindowOperation::Signal(signals) => {
                p["signals"] = if signals.is_empty() {
                    json!([Signal::Int])
                } else {
                    json!(signals)
                };
                "signal-child"
            }
            WindowOperation::RemoveLogo => {
                p["data"] = json!("-");
                p["position"] = json!("");
                p["alpha"] = json!(-1);
                "set-window-logo"
            }
            WindowOperation::OsWindow {
                action,
                unit,
                width,
                height,
                incremental,
                panel_settings,
            } => {
                p["action"] = json!(action);
                p["unit"] = json!(unit);
                p["width"] = json!(width);
                p["height"] = json!(height);
                p["incremental"] = json!(incremental);
                p["os_panel"] = json!(panel_settings);
                "resize-os-window"
            }
        };
        request(name, p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn reusable(&self) -> bool {
        !matches!(
            self.operation,
            WindowOperation::CreateMarker(MarkerSpec::Function(_))
        )
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}
impl CanSubmitWithoutReply for WindowCommand {}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResizeAxis {
    Horizontal,
    Vertical,
    Reset,
}

#[derive(Debug, Clone, bon::Builder)]
pub struct ResizeWindow {
    #[builder(default, into)]
    pub target: WindowTarget,
    pub axis: ResizeAxis,
    pub increment: i32,
}
impl ResizeWindow {
    pub fn new(axis: ResizeAxis, increment: i32) -> Self {
        Self {
            target: WindowTarget::Active,
            axis,
            increment,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResizeResult {
    Boolean(bool),
    NoData,
    /// A native `ok:true` reply can carry a layout refusal string. Upstream's
    /// reference client treats that string as an error (`string_return_is_error`);
    /// the server dispatcher does NOT rewrite it into `ok:false`.
    Rejected(String),
}
impl Command for ResizeWindow {
    /// Native handler can return bool or None. Unsupported layout's string is
    /// surfaced distinctly instead of fabricating success or RC failure.
    type Output = ResizeResult;
    fn request(&self) -> PlainRequest {
        let mut p = self.target.payload();
        p["axis"] = json!(self.axis);
        p["increment"] = json!(self.increment);
        request("resize-window", p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<Value>) -> Result<Self::Output, DecodeError> {
        match data {
            None => Ok(ResizeResult::NoData),
            Some(Value::Bool(b)) => Ok(ResizeResult::Boolean(b)),
            Some(Value::String(s)) => Ok(ResizeResult::Rejected(s)),
            _ => Err(DecodeError::MissingData),
        }
    }
}

#[derive(Debug, Clone, Default, bon::Builder)]
#[builder(on(String, into))]
pub struct SelectWindow {
    #[builder(default, into)]
    pub target: TabTarget,
    pub title: Option<String>,
    #[builder(default)]
    pub exclude_active: bool,
    #[builder(default)]
    pub reactivate_previous_tab: bool,
}
impl SelectWindow {
    pub fn new(target: impl Into<TabTarget>) -> Self {
        Self {
            target: target.into(),
            ..Self::default()
        }
    }
}
impl Command for SelectWindow {
    type Output = WindowId;
    fn request(&self) -> PlainRequest {
        let mut p = self.target.payload();
        p["title"] = json!(self.title);
        p["exclude_active"] = json!(self.exclude_active);
        p["reactivate_prev_tab"] = json!(self.reactivate_previous_tab);
        request("select-window", p)
    }
    fn class(&self) -> Class {
        Class::LongWait
    }
    fn mode(&self) -> Mode {
        Mode::Deferred
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<Value>) -> Result<WindowId, DecodeError> {
        data.and_then(|v| v.as_u64())
            .filter(|id| *id != 0)
            .map(WindowId)
            .ok_or(DecodeError::MissingData)
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BracketedPaste {
    #[default]
    Disable,
    Auto,
    Enable,
}

/// Caller-chosen identity for kitty's server-global broadcast bookkeeping, not
/// a transport session, delivery receipt or running conversation owner.
///
/// Only empty strings are rejected, to avoid the server's empty-name slot.
/// No trimming, normalization, random generation or other grammar is imposed.
/// Reusing an ID on the same server shares its accumulated membership.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct BroadcastId(String);
impl BroadcastId {
    pub fn new(value: impl Into<String>) -> Result<Self, EmptyBroadcastId> {
        let value = value.into();
        if value.is_empty() {
            Err(EmptyBroadcastId)
        } else {
            Ok(Self(value))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("broadcast identifier must not be empty")]
pub struct EmptyBroadcastId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastOperation {
    Start,
    End,
}

/// One explicit finite `send-text` broadcast control message, not an owned
/// broadcast conversation. Its required identity is shared with data messages.
///
/// Start marks selected recipients and, only when kitty receives an origin
/// window, installs that origin's removal/focus callbacks. Ordinary socket
/// closure is not origin-window removal and provides no callback cleanup.
/// End attempts to clear both selected recipients and ALL accumulated members
/// of this ID, even members outside the current match/exclusion scope.
///
/// Both operations are submission-only and retire their transport. Completion
/// cannot establish remote receipt/effect. Awaiting successive submissions on
/// separate sockets does NOT establish remote effect ordering. The caller owns
/// lifecycle and explicit end attempts; there is no Drop cleanup or follow-on.
///
/// ```
/// use kitty_rc_commands::window::{BroadcastControl, BroadcastId, BroadcastOperation, SendText};
/// let id = BroadcastId::new("caller-chosen").unwrap();
/// let start = BroadcastControl::start(id.clone());
/// let data = SendText::builder().data(vec![0, 0xff]).broadcast(id.clone()).build();
/// let end = BroadcastControl::builder()
///     .broadcast(id)
///     .operation(BroadcastOperation::End)
///     .build();
/// // These values construct messages only; no command has been submitted.
/// ```
///
/// An identity is mandatory, including through the builder:
/// ```compile_fail
/// use kitty_rc_commands::window::{BroadcastControl, BroadcastOperation};
/// let unnamed = BroadcastControl::builder().operation(BroadcastOperation::Start).build();
/// ```
#[derive(Debug, Clone, bon::Builder)]
pub struct BroadcastControl {
    pub broadcast: BroadcastId,
    pub operation: BroadcastOperation,
    #[builder(default)]
    pub windows: WindowSet,
    #[builder(default)]
    pub exclude_active: bool,
}
impl BroadcastControl {
    pub fn new(broadcast: BroadcastId, operation: BroadcastOperation) -> Self {
        Self {
            broadcast,
            operation,
            windows: WindowSet::default(),
            exclude_active: false,
        }
    }
    pub fn start(broadcast: BroadcastId) -> Self {
        Self::new(broadcast, BroadcastOperation::Start)
    }
    pub fn end(broadcast: BroadcastId) -> Self {
        Self::new(broadcast, BroadcastOperation::End)
    }
}
impl Command for BroadcastControl {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut p = self.windows.input_payload(self.exclude_active);
        p["session_id"] = json!(self.broadcast);
        p["data"] = json!(match self.operation {
            BroadcastOperation::Start => "session:start",
            BroadcastOperation::End => "session:end",
        });
        request("send-text", p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn mode(&self) -> Mode {
        Mode::SubmissionOnly
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}

/// Finite send-text content, with an explicit distinction between Unicode text
/// and arbitrary bytes. Neither variant interprets backslash escapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendTextData {
    /// Literal text, sent as `text:` and UTF-8-encoded by kitty.
    Text(String),
    /// Raw bytes, sent as standard `base64:` without a UTF-8 requirement.
    Bytes(Vec<u8>),
}
impl Default for SendTextData {
    fn default() -> Self {
        Self::Text(String::new())
    }
}
impl From<String> for SendTextData {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}
impl From<&str> for SendTextData {
    fn from(value: &str) -> Self {
        Self::Text(value.into())
    }
}
impl From<Vec<u8>> for SendTextData {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}
impl From<&[u8]> for SendTextData {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(value.into())
    }
}

/// One finite text or byte submission, not a file/stdin/broadcast conversation.
/// Construction performs no I/O or chunking. Completion proves local submission
/// only, not delivery to a child; execution suppresses replies and retires the
/// transport. Normal execution request-size limits still apply.
///
/// `new` remains literal: Python/ANSI-C escapes are NOT interpreted. Bracketed
/// paste can wrap and sanitize the data on the server when selected.
///
/// # Migration
///
/// The former public `text: String` field is now `data: SendTextData`;
/// builder `.text(value)` becomes `.data(value)`. Direct `new(text)` calls retain
/// their literal behavior. Strings select text; byte vectors/slices select bytes.
///
/// ```
/// use kitty_rc_commands::window::{SendText, SendTextData};
/// let literal = SendText::new(r"literal \n");
/// let binary = SendText::bytes(vec![0, 0xff]);
/// let built = SendText::builder().data("literal text").build();
/// let bytes = SendText::builder().data(vec![0, 0xff]).build();
/// assert_eq!(literal.data, SendTextData::Text(r"literal \n".into()));
/// assert_eq!(binary.data, bytes.data);
/// assert_eq!(built.data, SendTextData::Text("literal text".into()));
/// ```
#[derive(Debug, Clone, Default, bon::Builder)]
pub struct SendText {
    #[builder(into)]
    pub data: SendTextData,
    /// Optional server-global broadcast bookkeeping. ID-bearing data creates
    /// or updates membership even without start; it installs no origin hooks.
    /// No lifecycle or remote ordering/cleanup guarantee is implied. Use
    /// [`BroadcastControl`] for explicit start/end attempts.
    pub broadcast: Option<BroadcastId>,
    #[builder(default)]
    pub windows: WindowSet,
    #[builder(default)]
    pub exclude_active: bool,
    #[builder(default)]
    pub bracketed_paste: BracketedPaste,
}
impl SendText {
    /// Literal Unicode text; backslash escapes are left untouched.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            data: SendTextData::Text(text.into()),
            ..Self::default()
        }
    }
    /// Arbitrary bytes, including NUL and invalid UTF-8, in one request.
    pub fn bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            data: SendTextData::Bytes(bytes.into()),
            ..Self::default()
        }
    }
}
impl Command for SendText {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut p = self.windows.input_payload(self.exclude_active);
        p["data"] = json!(match &self.data {
            SendTextData::Text(text) => format!("text:{text}"),
            SendTextData::Bytes(bytes) => format!("base64:{}", STANDARD.encode(bytes)),
        });
        p["bracketed_paste"] = json!(self.bracketed_paste);
        if let Some(broadcast) = &self.broadcast {
            p["session_id"] = json!(broadcast);
        }
        request("send-text", p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn mode(&self) -> Mode {
        Mode::SubmissionOnly
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, bon::Builder)]
pub struct SendKey {
    pub keys: Vec<KeyChord>,
    #[builder(default)]
    pub windows: WindowSet,
    #[builder(default)]
    pub exclude_active: bool,
}
impl SendKey {
    pub fn new(keys: impl IntoIterator<Item = impl Into<KeyChord>>) -> Self {
        Self {
            keys: keys.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
}
impl Command for SendKey {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut p = self.windows.input_payload(self.exclude_active);
        p["keys"] = json!(self.keys);
        request("send-key", p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn mode(&self) -> Mode {
        Mode::SubmissionOnly
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, bon::Builder)]
pub struct UserVars {
    #[builder(default, into)]
    pub target: WindowTarget,
    /// Empty operations query variables; assignments/removals otherwise.
    #[builder(default)]
    pub operations: Vec<Assignment>,
}
impl UserVars {
    pub fn new(operations: impl IntoIterator<Item = Assignment>) -> Self {
        Self {
            operations: operations.into_iter().collect(),
            ..Self::default()
        }
    }
    pub fn query(target: impl Into<WindowTarget>) -> Self {
        Self {
            target: target.into(),
            ..Self::default()
        }
    }
}
impl Command for UserVars {
    type Output = String;
    fn request(&self) -> PlainRequest {
        let mut p = self.target.payload();
        p["var"] = json!(
            self.operations
                .iter()
                .map(Assignment::as_argument)
                .collect::<Vec<_>>()
        );
        request("set-user-vars", p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<Value>) -> Result<String, DecodeError> {
        crate::decode_string(data)
    }
}

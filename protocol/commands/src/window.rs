//! Finite window operations. Acknowledgment does not observe eventual removal,
//! layout redraw, child delivery or window-manager compliance.

use crate::targets::{Assignment, Finite, TabId, TabTarget, WindowId, WindowSet, WindowTarget};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, Mode, request};
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
#[derive(Debug, Clone)]
pub struct WindowCommand {
    pub target: WindowTarget,
    pub operation: WindowOperation,
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
    Signal(Vec<String>),
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
                p["signals"] = json!(if signals.is_empty() {
                    vec!["SIGINT".into()]
                } else {
                    signals.iter().map(|s| s.to_uppercase()).collect()
                });
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
    #[builder(default)]
    pub target: WindowTarget,
    pub axis: ResizeAxis,
    pub increment: i32,
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
    #[builder(default)]
    pub target: TabTarget,
    pub title: Option<String>,
    #[builder(default)]
    pub exclude_active: bool,
    #[builder(default)]
    pub reactivate_previous_tab: bool,
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

/// One finite text submission, not a file/stdin/broadcast conversation. Text
/// is literal UTF-8; Python escapes are NOT interpreted by this constructor.
#[derive(Debug, Clone, Default, bon::Builder)]
#[builder(on(String, into))]
pub struct SendText {
    pub text: String,
    #[builder(default)]
    pub windows: WindowSet,
    #[builder(default)]
    pub exclude_active: bool,
    #[builder(default)]
    pub bracketed_paste: BracketedPaste,
}
impl Command for SendText {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut p = self.windows.input_payload(self.exclude_active);
        p["data"] = json!(format!("text:{}", self.text));
        p["bracketed_paste"] = json!(self.bracketed_paste);
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
    pub keys: Vec<String>,
    #[builder(default)]
    pub windows: WindowSet,
    #[builder(default)]
    pub exclude_active: bool,
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
    #[builder(default)]
    pub target: WindowTarget,
    /// Empty operations query variables; assignments/removals otherwise.
    #[builder(default)]
    pub operations: Vec<Assignment>,
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

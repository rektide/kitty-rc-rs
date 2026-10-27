//! Launch and finite deferred screenshot profiles. Run output decoding is
//! available independently; [`crate::upload::Run`] binds it to a bounded upload
//! conversation, including the empty-stdin shared-accumulator case.
use crate::appearance::LogoPosition;
use crate::targets::{Assignment, Finite, ServerPath, TabId, TabMatch, TabTarget, WindowId, WindowMatch};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, Mode, request};
use base64::{Engine, engine::general_purpose::STANDARD};
use kitty_rc_schema::PlainRequest;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchType {
    #[default]
    Window,
    Tab,
    OsWindow,
    OsPanel,
    Overlay,
    OverlayMain,
    Background,
    Clipboard,
    Primary,
}
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    #[default]
    Default,
    First,
    After,
    Before,
    Neighbor,
    Last,
    Vsplit,
    Hsplit,
    Split,
}
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OsWindowState {
    #[default]
    Normal,
    Fullscreen,
    Maximized,
    Minimized,
}
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub enum StdinSource {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "@selection")]
    Selection,
    #[serde(rename = "@screen")]
    Screen,
    #[serde(rename = "@screen_scrollback")]
    ScreenScrollback,
    #[serde(rename = "@alternate")]
    Alternate,
    #[serde(rename = "@alternate_scrollback")]
    AlternateScrollback,
    #[serde(rename = "@first_cmd_output_on_screen")]
    FirstCommandOutputOnScreen,
    #[serde(rename = "@last_cmd_output")]
    LastCommandOutput,
    #[serde(rename = "@last_visited_cmd_output")]
    LastVisitedCommandOutput,
}

/// Native cwd selectors refer to the launch source window. A path is resolved
/// by the server; no client-side working directory or filesystem is consulted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchCwd {
    Current,
    LastReported,
    Oldest,
    Root,
    /// Kitty reserves the four selector strings above. Use an unambiguous path
    /// such as `./current` for a directory whose name is a reserved selector.
    Path(ServerPath),
    Raw(String),
}
impl LaunchCwd {
    /// Explicit escape hatch for future native cwd grammar.
    pub fn raw(value: impl Into<String>) -> Self {
        Self::Raw(value.into())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Current => "current",
            Self::LastReported => "last_reported",
            Self::Oldest => "oldest",
            Self::Root => "root",
            Self::Path(path) => path.as_str(),
            Self::Raw(value) => value,
        }
    }
}
impl From<ServerPath> for LaunchCwd {
    fn from(path: ServerPath) -> Self {
        Self::Path(path)
    }
}
impl Serialize for LaunchCwd {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Default, bon::Builder)]
#[builder(on(String, into))]
pub struct Launch {
    #[builder(default)]
    pub args: Vec<String>,
    #[builder(default)]
    pub kind: LaunchType,
    #[builder(default)]
    pub target: TabTarget,
    pub window_title: Option<String>,
    pub tab_title: Option<String>,
    pub cwd: Option<LaunchCwd>,
    pub next_to: Option<WindowMatch>,
    pub source_window: Option<WindowMatch>,
    #[builder(default)]
    pub env: Vec<Assignment>,
    #[builder(default)]
    pub user_vars: Vec<Assignment>,
    /// Local environment snapshot as NAME=VALUE strings, not a boolean flag.
    #[builder(default)]
    pub copy_env: Vec<String>,
    #[builder(default)]
    pub keep_focus: bool,
    #[builder(default)]
    pub copy_colors: bool,
    #[builder(default)]
    pub copy_cmdline: bool,
    #[builder(default)]
    pub hold: bool,
    #[builder(default)]
    pub location: Location,
    #[builder(default)]
    pub stdin_source: StdinSource,
    #[builder(default)]
    pub stdin_add_formatting: bool,
    #[builder(default)]
    pub stdin_add_line_wrap_markers: bool,
    #[builder(default)]
    pub allow_remote_control: bool,
    #[builder(default)]
    pub remote_control_passwords: Vec<String>,
    #[builder(default)]
    pub spacing: Vec<String>,
    #[builder(default)]
    pub colors: Vec<String>,
    #[builder(default)]
    pub watchers: Vec<String>,
    pub marker: Option<String>,
    pub logo: Option<String>,
    pub logo_position: Option<LogoPosition>,
    pub logo_alpha: Option<Finite>,
    pub add_to_session: Option<String>,
    pub os_window_title: Option<String>,
    pub os_window_name: Option<String>,
    pub os_window_class: Option<String>,
    pub os_window_position: Option<String>,
    #[builder(default)]
    pub os_window_state: OsWindowState,
    #[builder(default)]
    pub panel_settings: Vec<String>,
    pub bias: Option<Finite>,
    #[builder(default)]
    pub hold_after_ssh: bool,
}
impl Launch {
    /// Empty argv launches the server's configured shell. Arguments remain
    /// separate native strings, without shell quoting or local expansion.
    pub fn new(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            args: args.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
}

/// The native launch reply is a decimal string; zero is not a real window ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchResult {
    Window(WindowId),
    NoWindow,
}
impl Command for Launch {
    type Output = LaunchResult;
    fn request(&self) -> PlainRequest {
        let mut p = self.target.payload();
        p["args"] = json!(self.args);
        p["type"] = json!(self.kind);
        p["wait_for_child_to_exit"] = json!(false);
        p["window_title"] = json!(self.window_title);
        p["tab_title"] = json!(self.tab_title);
        p["cwd"] = json!(self.cwd);
        p["next_to"] = json!(self.next_to);
        p["source_window"] = json!(self.source_window);
        p["env"] = json!(self.env.iter().map(Assignment::as_argument).collect::<Vec<_>>());
        p["var"] = json!(self.user_vars.iter().map(Assignment::as_argument).collect::<Vec<_>>());
        p["copy_env"] = json!(self.copy_env);
        p["keep_focus"] = json!(self.keep_focus);
        p["copy_colors"] = json!(self.copy_colors);
        p["copy_cmdline"] = json!(self.copy_cmdline);
        p["hold"] = json!(self.hold);
        p["location"] = json!(self.location);
        p["stdin_source"] = json!(self.stdin_source);
        p["stdin_add_formatting"] = json!(self.stdin_add_formatting);
        p["stdin_add_line_wrap_markers"] = json!(self.stdin_add_line_wrap_markers);
        p["allow_remote_control"] = json!(self.allow_remote_control);
        p["remote_control_password"] = json!(self.remote_control_passwords);
        p["spacing"] = json!(self.spacing);
        p["color"] = json!(self.colors);
        p["watcher"] = json!(self.watchers);
        p["marker"] = json!(self.marker);
        p["logo"] = json!(self.logo);
        p["logo_position"] = json!(self.logo_position);
        p["logo_alpha"] = json!(self.logo_alpha);
        p["add_to_session"] = json!(self.add_to_session);
        p["os_window_title"] = json!(self.os_window_title);
        p["os_window_name"] = json!(self.os_window_name);
        p["os_window_class"] = json!(self.os_window_class);
        p["os_window_position"] = json!(self.os_window_position);
        p["os_window_state"] = json!(self.os_window_state);
        p["os_panel"] = json!(self.panel_settings);
        p["bias"] = json!(self.bias);
        p["hold_after_ssh"] = json!(self.hold_after_ssh);
        request("launch", p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<Value>) -> Result<LaunchResult, DecodeError> {
        let text = crate::decode_string(data)?;
        if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
            return Err(DecodeError::MissingData);
        }
        match text.parse::<u64>() {
            Ok(0) => Ok(LaunchResult::NoWindow),
            Ok(n) => Ok(LaunchResult::Window(WindowId(n))),
            Err(_) => Err(DecodeError::MissingData),
        }
    }
}
impl CanSubmitWithoutReply for Launch {}

/// Buffered run data. Nonzero exit_code is a process result, NOT a native
/// remote-control `ok:false` error. Bytes need not be UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
    pub exit_status: i32,
}
impl RunOutput {
    pub fn decode(data: Option<Value>) -> Result<Self, DecodeError> {
        #[derive(Deserialize)]
        struct Wire {
            stdout: String,
            stderr: String,
            exit_code: i32,
            exit_status: i32,
        }
        let w: Wire = serde_json::from_value(data.ok_or(DecodeError::MissingData)?)?;
        Ok(Self {
            stdout: decode_base64(&w.stdout)?,
            stderr: decode_base64(&w.stderr)?,
            exit_code: w.exit_code,
            exit_status: w.exit_status,
        })
    }
}
fn decode_base64(value: &str) -> Result<Vec<u8>, DecodeError> {
    STANDARD.decode(value).map_err(|e| DecodeError::Custom(Box::new(e)))
}

#[derive(Debug, Clone, Default)]
pub enum ScreenshotTarget {
    #[default]
    ActiveOsWindow,
    Window(WindowMatch),
    Tab(TabMatch),
}
impl ScreenshotTarget {
    pub fn window_id(id: WindowId) -> Self {
        Self::Window(WindowMatch::id(id))
    }

    pub fn tab_id(id: TabId) -> Self {
        Self::Tab(TabMatch::id(id))
    }

    fn payload(&self, path: &str) -> Value {
        let (window, tab) = match self {
            Self::ActiveOsWindow => (None, None),
            Self::Window(s) => (Some(s), None),
            Self::Tab(s) => (None, Some(s)),
        };
        json!({"match": window, "match_tab": tab, "output_path": path})
    }
}

/// Stable since 0.49. Binary PNG data, not arbitrary base64/JSON or a path.
#[derive(Debug, Clone, Default, bon::Builder)]
pub struct Screenshot {
    #[builder(default)]
    pub target: ScreenshotTarget,
}
impl Screenshot {
    pub fn new(target: ScreenshotTarget) -> Self {
        Self { target }
    }
}
impl Command for Screenshot {
    type Output = Vec<u8>;
    fn request(&self) -> PlainRequest {
        request("screenshot", self.target.payload(""))
    }
    fn class(&self) -> Class {
        Class::Observation
    }
    fn mode(&self) -> Mode {
        Mode::Deferred
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<Value>) -> Result<Vec<u8>, DecodeError> {
        let bytes = decode_base64(&crate::decode_string(data)?)?;
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err(DecodeError::MissingData);
        }
        Ok(bytes)
    }
}

/// Server-side file save; the reply is a path confirmation, not image bytes.
#[derive(Debug, Clone, bon::Builder)]
#[builder(on(String, into))]
pub struct SaveScreenshot {
    #[builder(default)]
    pub target: ScreenshotTarget,
    pub server_path: ServerPath,
}
impl SaveScreenshot {
    pub fn new(server_path: ServerPath) -> Self {
        Self {
            target: ScreenshotTarget::default(),
            server_path,
        }
    }
}
impl Command for SaveScreenshot {
    type Output = String;
    fn request(&self) -> PlainRequest {
        request("screenshot", self.target.payload(self.server_path.as_str()))
    }
    fn class(&self) -> Class {
        Class::Observation
    }
    fn mode(&self) -> Mode {
        Mode::Deferred
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<Value>) -> Result<String, DecodeError> {
        let text = crate::decode_string(data)?;
        text.strip_prefix("Screenshot saved to: ")
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .ok_or(DecodeError::MissingData)
    }
}

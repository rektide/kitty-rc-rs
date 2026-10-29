//! Mappable actions are a nested grammar, not `{action,args}` RC payloads.
//!
//! Every action retires its transport by default: dispatch acknowledgment is
//! not proof of downstream effects or replay safety, even for typed grammar.
//! Aliases and configuration may change actual dispatch behavior.
use crate::appearance::FontChange;
use crate::targets::{Direction, Layout, Opacity, Signal, WindowTarget};
use crate::window::{MarkerGroup, MarkerSpec};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, request};
use kitty_rc_schema::PlainRequest;
use serde_json::{Value, json};
use std::fmt::Write as _;

/// Known zero-argument actions grouped in one vocabulary. UI/clipboard/child
/// outcomes can differ from successful consumption of the action.
#[derive(Debug, Clone, Copy)]
pub enum SimpleAction {
    Quit,
    CloseTab,
    CloseWindow,
    /// Closes every window in the dispatch window's tab except that tab's
    /// active window (`tabs.py` method; dispatch scope, not a chosen tab ID).
    CloseOtherWindowsInTab,
    /// Closes other tabs in the OS window holding the dispatch window (the
    /// active one without a dispatch target), but the preserved tab is the
    /// **globally active** tab of the focused OS window — not the dispatch
    /// window's tab and not the selected OS window's active tab. If that
    /// globally active tab lives outside the selected OS window, for example
    /// when targeting a nonfocused one, kitty attempts `close_tab` on every
    /// tab there, subject to its close-confirmation policies. A manager with
    /// a single tab is left untouched.
    CloseOtherTabsInOsWindow,
    NextTab,
    PreviousTab,
    MoveTabForward,
    MoveTabBackward,
    NextWindow,
    PreviousWindow,
    MoveWindowForward,
    MoveWindowBackward,
    /// Moves the active window of its tab to the first position; a window
    /// already first stays put.
    MoveWindowToTop,
    FirstWindow,
    ResetWindowSizes,
    ToggleFullscreen,
    ToggleMaximized,
    CopyToClipboard,
    /// Copies the dispatch window's selection to the system clipboard with
    /// ANSI formatting codes; an empty selection copies nothing.
    CopyAnsiToClipboard,
    /// Copies the last non-empty command output to the system clipboard.
    /// Shell integration must have reported the command output; otherwise
    /// there is nothing to copy. Registered upstream by kitty 0.49.2 but not
    /// 0.43.1, where dispatch of this name would fail.
    CopyLastCommandOutput,
    PasteFromClipboard,
    PasteSelection,
    ClearSelection,
    CopyOrInterrupt,
    LastUsedLayout,
    ScrollLineUp,
    ScrollLineDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollHome,
    ScrollEnd,
    ShowScrollback,
    /// Shows the last-run command output in a pager like less. Requires
    /// shell integration to have reported command output.
    ShowLastCommandOutput,
    RemoveMarker,
    CreateMarker,
    DebugConfig,
    EditConfigFile,
}
impl SimpleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Quit => "quit",
            Self::CloseTab => "close_tab",
            Self::CloseWindow => "close_window",
            Self::CloseOtherWindowsInTab => "close_other_windows_in_tab",
            Self::CloseOtherTabsInOsWindow => "close_other_tabs_in_os_window",
            Self::NextTab => "next_tab",
            Self::PreviousTab => "previous_tab",
            Self::MoveTabForward => "move_tab_forward",
            Self::MoveTabBackward => "move_tab_backward",
            Self::NextWindow => "next_window",
            Self::PreviousWindow => "previous_window",
            Self::MoveWindowForward => "move_window_forward",
            Self::MoveWindowBackward => "move_window_backward",
            Self::MoveWindowToTop => "move_window_to_top",
            Self::FirstWindow => "first_window",
            Self::ResetWindowSizes => "reset_window_sizes",
            Self::ToggleFullscreen => "toggle_fullscreen",
            Self::ToggleMaximized => "toggle_maximized",
            Self::CopyToClipboard => "copy_to_clipboard",
            Self::CopyAnsiToClipboard => "copy_ansi_to_clipboard",
            Self::CopyLastCommandOutput => "copy_last_command_output",
            Self::PasteFromClipboard => "paste_from_clipboard",
            Self::PasteSelection => "paste_selection",
            Self::ClearSelection => "clear_selection",
            Self::CopyOrInterrupt => "copy_or_interrupt",
            Self::LastUsedLayout => "last_used_layout",
            Self::ScrollLineUp => "scroll_line_up",
            Self::ScrollLineDown => "scroll_line_down",
            Self::ScrollPageUp => "scroll_page_up",
            Self::ScrollPageDown => "scroll_page_down",
            Self::ScrollHome => "scroll_home",
            Self::ScrollEnd => "scroll_end",
            Self::ShowScrollback => "show_scrollback",
            Self::ShowLastCommandOutput => "show_last_command_output",
            Self::RemoveMarker => "remove_marker",
            Self::CreateMarker => "create_marker",
            Self::DebugConfig => "debug_config",
            Self::EditConfigFile => "edit_config_file",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClearTerminal {
    Reset,
    Scroll,
    Scrollback,
    Clear,
    ToCursor,
    ToCursorScroll,
    LastCommand,
}
impl ClearTerminal {
    fn as_str(self) -> &'static str {
        match self {
            Self::Reset => "reset",
            Self::Scroll => "scroll",
            Self::Scrollback => "scrollback",
            Self::Clear => "clear",
            Self::ToCursor => "to_cursor",
            Self::ToCursorScroll => "to_cursor_scroll",
            Self::LastCommand => "last_command",
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ResizeQuality {
    Reset,
    Taller,
    Shorter,
    Wider,
    Narrower,
}
impl ResizeQuality {
    fn as_str(self) -> &'static str {
        match self {
            Self::Reset => "reset",
            Self::Taller => "taller",
            Self::Shorter => "shorter",
            Self::Wider => "wider",
            Self::Narrower => "narrower",
        }
    }
}
/// Server-side process creation kinds sharing one shlex-split argv grammar.
/// The `*_with_cwd` kinds source the child working directory from the
/// dispatch/fallback window via OSC 7 shell-integration reporting; without it
/// kitty falls back to its process-cwd heuristics.
#[derive(Debug, Clone, Copy)]
pub enum NewProcess {
    Window,
    WindowWithCwd,
    Tab,
    TabWithCwd,
    OsWindow,
    OsWindowWithCwd,
    Launch,
}
impl NewProcess {
    fn as_str(self) -> &'static str {
        match self {
            Self::Window => "new_window",
            Self::WindowWithCwd => "new_window_with_cwd",
            Self::Tab => "new_tab",
            Self::TabWithCwd => "new_tab_with_cwd",
            Self::OsWindow => "new_os_window",
            Self::OsWindowWithCwd => "new_os_window_with_cwd",
            Self::Launch => "launch",
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum DetachWindow {
    NewOsWindow,
    NewTab,
    NewTabLeft,
    NewTabRight,
    Ask,
    PreviousTab,
    LeftTab,
    RightTab,
}
impl DetachWindow {
    fn as_str(self) -> &'static str {
        match self {
            Self::NewOsWindow => "new",
            Self::NewTab => "new-tab",
            Self::NewTabLeft => "new-tab-left",
            Self::NewTabRight => "new-tab-right",
            Self::Ask => "ask",
            Self::PreviousTab => "tab-prev",
            Self::LeftTab => "tab-left",
            Self::RightTab => "tab-right",
        }
    }
}

/// Nonempty literal Unicode for nested [`KnownAction::Paste`] and
/// [`KnownAction::SendText`], not caller-authored escape source.
///
/// Only the empty string is rejected: whitespace, NUL, backslashes and every
/// Rust Unicode scalar are preserved. Private storage and the absence of
/// `Default`/`Deserialize` keep the invariant intact even in public variants.
///
/// ```compile_fail
/// use kitty_rc_commands::actions::ActionText;
/// let invalid = ActionText(String::new()); // storage is private
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionText(String);
impl ActionText {
    pub fn new(literal: impl Into<String>) -> Result<Self, EmptyActionText> {
        let literal = literal.into();
        if literal.is_empty() {
            Err(EmptyActionText)
        } else {
            Ok(Self(literal))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("nested action text must not be empty")]
pub struct EmptyActionText;

/// Validated buffer name for nested [`KnownAction::CopyToBuffer`] and
/// [`KnownAction::PasteFromBuffer`], stored verbatim in the native raw
/// remainder grammar — never shell-quoted and never escape-encoded.
///
/// The native parser keeps everything after the action name as a single
/// argument, so internal spaces, tabs, newlines and backslashes stay literal.
/// Its outer `strip`/`split` still alters edge characters: a leading
/// whitespace run joins the separator, and trailing whitespace (including the
/// `\u{1c}`–`\u{1f}` separator controls and NEL) is stripped. Names that
/// would not parse back identically are rejected rather than silently
/// addressing a different buffer.
///
/// The special names `clipboard` and `primary` reach the system clipboard
/// and primary selection instead of kitty's named in-memory buffers. That
/// distinction belongs to the native handlers and stays a documented string
/// choice rather than a typed split: the wire grammar is identical, and typed
/// recognition would duplicate the existing `CopyToClipboard`/`PasteSelection`
/// vocabulary for what upstream models as plain argument data.
///
/// ```compile_fail
/// use kitty_rc_commands::actions::BufferName;
/// let invalid = BufferName(String::new()); // storage is private
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferName(String);
impl BufferName {
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidBufferName> {
        let name = name.into();
        let alters_edge = |c: Option<char>| c.is_some_and(is_native_edge_whitespace);
        match (name.chars().next(), name.chars().last()) {
            (None, _) => Err(InvalidBufferName::Empty),
            (first, last) if alters_edge(first) || alters_edge(last) => {
                Err(InvalidBufferName::EdgeWhitespace)
            }
            _ => Ok(Self(name)),
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBufferName {
    #[error("clipboard buffer name must not be empty")]
    Empty,
    #[error("clipboard buffer name must not start or end with whitespace")]
    EdgeWhitespace,
}

// Python's str.strip() removes its isspace() set at the edges: Unicode
// White_Space plus the \u{1c}–\u{1f} separator controls Rust's is_whitespace
// does not cover. Either alters the raw remainder's first/last characters.
fn is_native_edge_whitespace(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}

/// Keyboard-mode gate for nested `send_text`, not RC `send-text` targeting.
/// Every choice is nonempty; combinations have canonical normal/application/
/// kitty order. Normal/application refer to DECCKM; kitty is the extended
/// keyboard protocol and takes precedence when enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendTextModes {
    All,
    Normal,
    Application,
    Kitty,
    NormalApplication,
    NormalKitty,
    ApplicationKitty,
    NormalApplicationKitty,
}
impl SendTextModes {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Normal => "normal",
            Self::Application => "application",
            Self::Kitty => "kitty",
            Self::NormalApplication => "normal,application",
            Self::NormalKitty => "normal,kitty",
            Self::ApplicationKitty => "application,kitty",
            Self::NormalApplicationKitty => "normal,application,kitty",
        }
    }
}

#[derive(Debug, Clone)]
pub enum KnownAction {
    Simple(SimpleAction),
    /// Literal text decoded once by kitty, then passed to configured paste
    /// actions. Filters, confirmations, newline/bracketed-paste handling and
    /// aliases can change or delay effects; this is not byte-exact child input.
    Paste(ActionText),
    /// Literal text decoded once to UTF-8 and gated by keyboard mode. Mode
    /// mismatch returns passthrough; RC action can then report "Unknown action".
    /// Dispatch acknowledgment is not a child receipt. Unlike RC `send-text`,
    /// this uses the outer action's single dispatch target and accepts no bytes.
    SendText {
        modes: SendTextModes,
        text: ActionText,
    },
    /// Copies the dispatch window's selection to the named buffer. The names
    /// `clipboard` and `primary` reach the system clipboard/selection; every
    /// other name addresses kitty's in-memory named buffers. Acknowledgment
    /// is not proof a nonempty selection existed.
    CopyToBuffer(BufferName),
    /// Pastes from the named buffer through the configured paste actions,
    /// so filtering, confirmations and newline handling still apply. The
    /// names `clipboard` and `primary` read the system clipboard/selection.
    PasteFromBuffer(BufferName),
    /// Native wire argument; negative values have upstream backwards-compatible
    /// parsing (the parser adds one), not a Rust zero-based tab ID.
    GotoTab(i32),
    NthWindow(i32),
    NextLayout(i32),
    NeighboringWindow(Direction),
    MoveWindow(Direction),
    MoveWindowBy(i32),
    SetWindowTitle(String),
    SetTabTitle(String),
    GotoLayout(Layout),
    ToggleLayout(Layout),
    ResizeWindow {
        quality: ResizeQuality,
        increment: i32,
    },
    CloseWindowWithConfirmation {
        ignore_shell: bool,
    },
    DetachWindow(DetachWindow),
    DetachTab {
        ask: bool,
    },
    ScrollToPrompt {
        count: i32,
        offset: i32,
    },
    ScrollToMark {
        previous: bool,
        group: Option<MarkerGroup>,
    },
    ToggleMarker(MarkerSpec),
    ClearTerminal {
        action: ClearTerminal,
        active_only: bool,
    },
    FontSize {
        all: bool,
        change: FontChange,
    },
    BackgroundOpacity(Opacity),
    /// Upstream to_cmdline performs server-side tilde/environment expansion
    /// AFTER shell splitting. For literal argv values use RC Launch instead.
    NewProcess {
        kind: NewProcess,
        args: Vec<String>,
    },
    /// Nested action argv undergoes server-side expansion. RC Kitten preserves
    /// its JSON argument strings without the nested action parser.
    Kitten {
        name: String,
        args: Vec<String>,
    },
    LoadConfigFiles(Vec<String>),
    SetColors(Vec<String>),
    SignalChild(Vec<Signal>),
    ShowKittyDoc(String),
}
impl KnownAction {
    /// Encode literal Unicode for kitty's native paste parser, not escape
    /// source. Empty input is rejected rather than emitting a missing argument.
    ///
    /// ```
    /// use kitty_rc_commands::actions::{Action, KnownAction};
    /// let paste = Action::new(KnownAction::paste(r" \n ")?);
    /// # Ok::<(), kitty_rc_commands::actions::EmptyActionText>(())
    /// ```
    pub fn paste(literal: impl Into<String>) -> Result<Self, EmptyActionText> {
        ActionText::new(literal).map(Self::Paste)
    }

    /// Encode literal Unicode for nested `send_text`. No local escape decoding
    /// occurs: `r"\n"` remains backslash plus n, while `"\n"` is a newline.
    ///
    /// ```
    /// use kitty_rc_commands::actions::{Action, KnownAction, SendTextModes};
    /// let send = Action::new(KnownAction::send_text(
    ///     SendTextModes::NormalApplication, "hello\n",
    /// )?);
    /// # Ok::<(), kitty_rc_commands::actions::EmptyActionText>(())
    /// ```
    pub fn send_text(
        modes: SendTextModes,
        literal: impl Into<String>,
    ) -> Result<Self, EmptyActionText> {
        Ok(Self::SendText {
            modes,
            text: ActionText::new(literal)?,
        })
    }

    /// Name a buffer for nested `copy_to_buffer`. The name travels as the raw
    /// remainder — internal spaces and backslashes stay literal — while empty
    /// names and edges the native parser would alter are rejected.
    ///
    /// ```
    /// use kitty_rc_commands::actions::{Action, KnownAction};
    /// let copy = Action::new(KnownAction::copy_to_buffer("my buffer")?);
    /// # Ok::<(), kitty_rc_commands::actions::InvalidBufferName>(())
    /// ```
    pub fn copy_to_buffer(name: impl Into<String>) -> Result<Self, InvalidBufferName> {
        BufferName::new(name).map(Self::CopyToBuffer)
    }

    /// Name a buffer for nested `paste_from_buffer`, sharing the raw remainder
    /// grammar and validation of [`Self::copy_to_buffer`].
    ///
    /// ```
    /// use kitty_rc_commands::actions::{Action, KnownAction};
    /// let paste = Action::new(KnownAction::paste_from_buffer("scratch")?);
    /// # Ok::<(), kitty_rc_commands::actions::InvalidBufferName>(())
    /// ```
    pub fn paste_from_buffer(name: impl Into<String>) -> Result<Self, InvalidBufferName> {
        BufferName::new(name).map(Self::PasteFromBuffer)
    }

    pub fn wire(&self) -> String {
        match self {
            Self::Simple(action) => action.as_str().into(),
            Self::Paste(text) => format!("paste {}", encode_action_text(text)),
            Self::SendText { modes, text } => {
                format!("send_text {} {}", modes.as_str(), encode_action_text(text))
            }
            Self::CopyToBuffer(name) => format!("copy_to_buffer {}", name.as_str()),
            Self::PasteFromBuffer(name) => format!("paste_from_buffer {}", name.as_str()),
            Self::GotoTab(n) => format!("goto_tab {n}"),
            Self::NthWindow(n) => format!("nth_window {n}"),
            Self::NextLayout(n) => format!("next_layout {n}"),
            Self::NeighboringWindow(d) => format!("neighboring_window {}", d.as_str()),
            Self::MoveWindow(d) => format!("move_window {}", d.as_str()),
            Self::MoveWindowBy(n) => format!("move_window {n}"),
            Self::SetWindowTitle(s) => format!("set_window_title {s}"),
            Self::SetTabTitle(s) => format!("set_tab_title {s}"),
            Self::GotoLayout(l) => format!("goto_layout {}", l.as_str()),
            Self::ToggleLayout(l) => format!("toggle_layout {}", l.as_str()),
            Self::ResizeWindow { quality, increment } => {
                format!("resize_window {} {increment}", quality.as_str())
            }
            Self::CloseWindowWithConfirmation { ignore_shell } => if *ignore_shell {
                "close_window_with_confirmation ignore-shell"
            } else {
                "close_window_with_confirmation"
            }
            .into(),
            Self::DetachWindow(d) => format!("detach_window {}", d.as_str()),
            Self::DetachTab { ask } => format!("detach_tab {}", if *ask { "ask" } else { "new" }),
            Self::ScrollToPrompt { count, offset } => format!("scroll_to_prompt {count} {offset}"),
            Self::ScrollToMark { previous, group } => format!(
                "scroll_to_mark {} {}",
                if *previous { "prev" } else { "next" },
                group.map_or(0, MarkerGroup::number)
            ),
            Self::ToggleMarker(spec) => {
                let args = spec.arguments();
                format!("toggle_marker {} {}", args[0], shell_arguments(&args[1..]))
            }
            Self::ClearTerminal {
                action,
                active_only,
            } => format!(
                "clear_terminal {} {}",
                action.as_str(),
                if *active_only { "active" } else { "all" }
            ),
            Self::FontSize { all, change } => format!(
                "change_font_size {} {}",
                if *all { "all" } else { "current" },
                change.argument()
            ),
            Self::BackgroundOpacity(opacity) => format!("set_background_opacity {}", opacity.get()),
            Self::NewProcess { kind, args } => {
                format!("{} {}", kind.as_str(), shell_arguments(args))
            }
            Self::Kitten { name, args } => {
                format!("kitten {} {}", shell_quote(name), shell_arguments(args))
            }
            Self::LoadConfigFiles(paths) => format!("load_config_file {}", shell_arguments(paths)),
            Self::SetColors(args) => format!("set_colors {}", shell_arguments(args)),
            Self::SignalChild(signals) => format!(
                "signal_child {}",
                signals
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Self::ShowKittyDoc(s) => format!("show_kitty_doc {s}"),
        }
    }
}

/// Exact complete nested grammar escape hatch. No promise of local parser
/// validation, completion, conflict profile or replay/reuse safety.
#[derive(Debug, Clone)]
pub enum ActionSpec {
    Known(KnownAction),
    Raw(String),
}
impl From<KnownAction> for ActionSpec {
    fn from(value: KnownAction) -> Self {
        Self::Known(value)
    }
}
impl From<SimpleAction> for ActionSpec {
    fn from(value: SimpleAction) -> Self {
        Self::Known(KnownAction::Simple(value))
    }
}
impl ActionSpec {
    pub fn wire(&self) -> String {
        match self {
            Self::Known(k) => k.wire(),
            Self::Raw(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, bon::Builder)]
pub struct Action {
    #[builder(into)]
    pub action: ActionSpec,
    #[builder(default, into)]
    pub target: WindowTarget,
}
impl Action {
    pub fn new(action: impl Into<ActionSpec>) -> Self {
        Self {
            action: action.into(),
            target: WindowTarget::Active,
        }
    }
}
impl Command for Action {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let (spec, only_self) = self.target.fields();
        request(
            "action",
            json!({"action": self.action.wire(), "match_window": spec, "self": only_self}),
        )
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}
impl CanSubmitWithoutReply for Action {}

// Fixed-width Unicode escapes protect ALL whitespace from the outer parser's
// strip/split and protect backslashes from a second interpretation. Kitty does
// the only decoding. Do not replace this with local decoding or octal escapes.
fn encode_action_text(text: &ActionText) -> String {
    let mut encoded = String::new();
    for scalar in text.as_str().chars() {
        write!(encoded, "\\U{:08x}", scalar as u32).expect("writing to String cannot fail");
    }
    encoded
}

/// POSIX shell token quoting preserves token boundaries, not freedom from
/// upstream to_cmdline's subsequent tilde/environment expansion. It is not
/// used for remainder-string or whitespace-split action grammars.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
fn shell_arguments(values: &[String]) -> String {
    values
        .iter()
        .map(|s| shell_quote(s))
        .collect::<Vec<_>>()
        .join(" ")
}

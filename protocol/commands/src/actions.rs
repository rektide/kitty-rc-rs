//! Mappable actions are a nested grammar, not `{action,args}` RC payloads.
//!
//! Every action retires its transport by default: dispatch acknowledgment is
//! not proof of downstream effects or replay safety, even for typed grammar.
//! Aliases and configuration may change actual dispatch behavior.
use crate::appearance::FontChange;
use crate::targets::{Direction, Layout, Opacity, WindowTarget};
use crate::window::{MarkerGroup, MarkerSpec};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, request};
use kitty_rc_schema::PlainRequest;
use serde_json::{Value, json};

/// Known zero-argument actions grouped in one vocabulary. UI/clipboard/child
/// outcomes can differ from successful consumption of the action.
#[derive(Debug, Clone, Copy)]
pub enum SimpleAction {
    Quit,
    CloseTab,
    CloseWindow,
    NextTab,
    PreviousTab,
    MoveTabForward,
    MoveTabBackward,
    NextWindow,
    PreviousWindow,
    MoveWindowForward,
    MoveWindowBackward,
    FirstWindow,
    ResetWindowSizes,
    ToggleFullscreen,
    ToggleMaximized,
    CopyToClipboard,
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
            Self::NextTab => "next_tab",
            Self::PreviousTab => "previous_tab",
            Self::MoveTabForward => "move_tab_forward",
            Self::MoveTabBackward => "move_tab_backward",
            Self::NextWindow => "next_window",
            Self::PreviousWindow => "previous_window",
            Self::MoveWindowForward => "move_window_forward",
            Self::MoveWindowBackward => "move_window_backward",
            Self::FirstWindow => "first_window",
            Self::ResetWindowSizes => "reset_window_sizes",
            Self::ToggleFullscreen => "toggle_fullscreen",
            Self::ToggleMaximized => "toggle_maximized",
            Self::CopyToClipboard => "copy_to_clipboard",
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
#[derive(Debug, Clone, Copy)]
pub enum NewProcess {
    Window,
    WindowWithCwd,
    Tab,
    OsWindow,
    Launch,
}
impl NewProcess {
    fn as_str(self) -> &'static str {
        match self {
            Self::Window => "new_window",
            Self::WindowWithCwd => "new_window_with_cwd",
            Self::Tab => "new_tab",
            Self::OsWindow => "new_os_window",
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

#[derive(Debug, Clone)]
pub enum KnownAction {
    Simple(SimpleAction),
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
    SignalChild(Vec<String>),
    ShowKittyDoc(String),
}
impl KnownAction {
    pub fn wire(&self) -> String {
        match self {
            Self::Simple(action) => action.as_str().into(),
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
                    .map(|s| s.to_uppercase())
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
    #[builder(default)]
    pub target: WindowTarget,
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

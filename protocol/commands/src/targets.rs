//! Open match expressions and semantic finite argument types.

use serde::{Serialize, Serializer};
use serde_json::{Value, json};
use std::fmt;
use std::ops::{BitOr, BitOrAssign};

/// A kitty window ID, not an OS/platform window ID or process ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct TabId(pub u64);

/// A window match expression, serialized as kitty's native match string.
/// Field constructors accept regular expressions, not literal-text searches.
/// Compound expressions and future fields require the explicit [`Self::raw`]
/// escape hatch; no local parser claims to validate kitty's open grammar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct WindowMatch(String);
impl WindowMatch {
    pub fn id(id: WindowId) -> Self {
        Self(format!("id:{}", id.0))
    }
    pub fn title(pattern: impl Into<String>) -> Self {
        Self(match_field("title", &pattern.into()))
    }
    pub fn pid(pid: u32) -> Self {
        Self(format!("pid:{pid}"))
    }
    pub fn cwd(pattern: impl Into<String>) -> Self {
        Self(match_field("cwd", &pattern.into()))
    }
    pub fn cmdline(pattern: impl Into<String>) -> Self {
        Self(match_field("cmdline", &pattern.into()))
    }
    /// Zero-based position within the active tab.
    pub fn num(position: u32) -> Self {
        Self(format!("num:{position}"))
    }
    pub fn neighbor(direction: Direction) -> Self {
        Self(format!("neighbor:{}", direction.as_str()))
    }
    pub fn all() -> Self {
        Self("all".into())
    }
    pub fn raw(expression: impl Into<String>) -> Self {
        Self(expression.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<WindowId> for WindowMatch {
    fn from(id: WindowId) -> Self {
        Self::id(id)
    }
}
impl fmt::Display for WindowMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tab match expression. Native `id`/`title` matching can fall back to a
/// matching window's tab; this type preserves that upstream behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct TabMatch(String);
impl TabMatch {
    pub fn id(id: TabId) -> Self {
        Self(format!("id:{}", id.0))
    }
    /// A native regular expression, not a literal title.
    pub fn title(pattern: impl Into<String>) -> Self {
        Self(match_field("title", &pattern.into()))
    }
    pub fn window_id(id: WindowId) -> Self {
        Self(format!("window_id:{}", id.0))
    }
    pub fn window_title(pattern: impl Into<String>) -> Self {
        Self(match_field("window_title", &pattern.into()))
    }
    pub fn pid(pid: u32) -> Self {
        Self(format!("pid:{pid}"))
    }
    pub fn cwd(pattern: impl Into<String>) -> Self {
        Self(match_field("cwd", &pattern.into()))
    }
    pub fn cmdline(pattern: impl Into<String>) -> Self {
        Self(match_field("cmdline", &pattern.into()))
    }
    pub fn all() -> Self {
        Self("all".into())
    }
    pub fn raw(expression: impl Into<String>) -> Self {
        Self(expression.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<TabId> for TabMatch {
    fn from(id: TabId) -> Self {
        Self::id(id)
    }
}
impl fmt::Display for TabMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// kitty/search_query_parser.py unescapes backslashes/quotes before handing a
// field's query to its matcher. Quote grammar delimiters while preserving the
// original regex, including its backslashes; this is not shell quoting.
fn match_field(field: &str, query: &str) -> String {
    let escaped = query.replace('\\', "\\\\").replace('"', "\\\"");
    if query.is_empty()
        || query
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '"' | '(' | ')'))
    {
        format!("{field}:\"{escaped}\"")
    } else {
        format!("{field}:{escaped}")
    }
}

/// Expressions deliberately remain open: kitty supports compound matching and
/// future match fields. No client-side subset parser pretends to validate them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WindowTarget {
    #[default]
    Active,
    CallingWindow,
    Id(WindowId),
    Match(String),
}
impl WindowTarget {
    pub fn window(window: impl Into<WindowMatch>) -> Self {
        Self::Match(window.into().0)
    }
    pub fn title(pattern: impl Into<String>) -> Self {
        Self::window(WindowMatch::title(pattern))
    }
    pub fn raw(expression: impl Into<String>) -> Self {
        Self::Match(expression.into())
    }
    pub(crate) fn fields(&self) -> (Option<String>, bool) {
        match self {
            Self::Active => (None, false),
            Self::CallingWindow => (None, true),
            Self::Id(id) => (Some(format!("id:{}", id.0)), false),
            Self::Match(s) => (Some(s.clone()), false),
        }
    }
    pub(crate) fn payload(&self) -> Value {
        let (spec, only_self) = self.fields();
        json!({"match": spec, "self": only_self})
    }
}
impl From<WindowId> for WindowTarget {
    fn from(id: WindowId) -> Self {
        Self::Id(id)
    }
}
impl From<WindowMatch> for WindowTarget {
    fn from(value: WindowMatch) -> Self {
        Self::window(value)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TabTarget {
    #[default]
    Active,
    CallingTab,
    Id(TabId),
    Match(String),
    All,
}
impl TabTarget {
    pub fn tab(tab: impl Into<TabMatch>) -> Self {
        Self::Match(tab.into().0)
    }
    pub fn title(pattern: impl Into<String>) -> Self {
        Self::tab(TabMatch::title(pattern))
    }
    pub fn raw(expression: impl Into<String>) -> Self {
        Self::Match(expression.into())
    }
    pub(crate) fn fields(&self) -> (Option<String>, bool) {
        match self {
            Self::Active => (None, false),
            Self::CallingTab => (None, true),
            Self::Id(id) => (Some(format!("id:{}", id.0)), false),
            Self::Match(s) => (Some(s.clone()), false),
            Self::All => (Some("all".into()), false),
        }
    }
    pub(crate) fn payload(&self) -> Value {
        let (spec, only_self) = self.fields();
        json!({"match": spec, "self": only_self})
    }
}
impl From<TabId> for TabTarget {
    fn from(id: TabId) -> Self {
        Self::Id(id)
    }
}
impl From<TabMatch> for TabTarget {
    fn from(value: TabMatch) -> Self {
        Self::tab(value)
    }
}

/// Multi-window selection for commands whose native fields are
/// match_window/match_tab/all. It is not the self-capable window-match profile.
#[derive(Debug, Clone, Default, bon::Builder)]
pub struct WindowSet {
    #[builder(into)]
    pub match_window: Option<WindowMatch>,
    #[builder(into)]
    pub match_tab: Option<TabMatch>,
    #[builder(default)]
    pub all: bool,
}
impl WindowSet {
    /// Leave native matching fields unset. Input commands use the active window;
    /// other multi-window handlers may use the calling window when available.
    pub fn active() -> Self {
        Self::default()
    }
    pub fn window(window: impl Into<WindowMatch>) -> Self {
        Self {
            match_window: Some(window.into()),
            ..Self::default()
        }
    }
    pub fn tab(tab: impl Into<TabMatch>) -> Self {
        Self {
            match_tab: Some(tab.into()),
            ..Self::default()
        }
    }
    pub fn all() -> Self {
        Self {
            all: true,
            ..Self::default()
        }
    }
    pub(crate) fn payload(&self) -> Value {
        json!({"match_window": self.match_window, "match_tab": self.match_tab, "all": self.all})
    }
    pub(crate) fn input_payload(&self, exclude_active: bool) -> Value {
        json!({"match": self.match_window, "match_tab": self.match_tab, "all": self.all, "exclude_active": exclude_active})
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}
impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// Symbolic signals accepted by kitty's server-side Python `signal` module.
/// Availability depends on the server platform. Custom names retain the
/// existing uppercasing behavior and must include their native `SIG` prefix.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Signal {
    Hup,
    Int,
    Quit,
    Ill,
    Trap,
    Abrt,
    Bus,
    Fpe,
    Kill,
    Usr1,
    Segv,
    Usr2,
    Pipe,
    Alrm,
    Term,
    Chld,
    Cont,
    Stop,
    Tstp,
    Ttin,
    Ttou,
    Urg,
    Xcpu,
    Xfsz,
    Vtalrm,
    Prof,
    Winch,
    Custom(String),
}
impl Signal {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Hup => "SIGHUP",
            Self::Int => "SIGINT",
            Self::Quit => "SIGQUIT",
            Self::Ill => "SIGILL",
            Self::Trap => "SIGTRAP",
            Self::Abrt => "SIGABRT",
            Self::Bus => "SIGBUS",
            Self::Fpe => "SIGFPE",
            Self::Kill => "SIGKILL",
            Self::Usr1 => "SIGUSR1",
            Self::Segv => "SIGSEGV",
            Self::Usr2 => "SIGUSR2",
            Self::Pipe => "SIGPIPE",
            Self::Alrm => "SIGALRM",
            Self::Term => "SIGTERM",
            Self::Chld => "SIGCHLD",
            Self::Cont => "SIGCONT",
            Self::Stop => "SIGSTOP",
            Self::Tstp => "SIGTSTP",
            Self::Ttin => "SIGTTIN",
            Self::Ttou => "SIGTTOU",
            Self::Urg => "SIGURG",
            Self::Xcpu => "SIGXCPU",
            Self::Xfsz => "SIGXFSZ",
            Self::Vtalrm => "SIGVTALRM",
            Self::Prof => "SIGPROF",
            Self::Winch => "SIGWINCH",
            Self::Custom(name) => name,
        }
    }
}
impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(name) => f.write_str(&name.to_uppercase()),
            _ => f.write_str(self.as_str()),
        }
    }
}
impl Serialize for Signal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Common kitty shortcut key names. Character keys follow kitty's shortcut
/// parser (ASCII letters are case-insensitive; use SHIFT for shifted keys).
/// `Raw` names future/less common keys without pretending to validate them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Character(char),
    Enter,
    Tab,
    Escape,
    Backspace,
    Insert,
    Delete,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Space,
    Plus,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Raw(String),
}
impl Key {
    pub fn raw(name: impl Into<String>) -> Self {
        Self::Raw(name.into())
    }
}
impl From<char> for Key {
    fn from(value: char) -> Self {
        Self::Character(value)
    }
}
impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Character('+') | Self::Plus => "plus",
            Self::Character(' ') | Self::Space => "space",
            Self::Character(c) => return write!(f, "{c}"),
            Self::Enter => "enter",
            Self::Tab => "tab",
            Self::Escape => "escape",
            Self::Backspace => "backspace",
            Self::Insert => "insert",
            Self::Delete => "delete",
            Self::Left => "left",
            Self::Right => "right",
            Self::Up => "up",
            Self::Down => "down",
            Self::PageUp => "page_up",
            Self::PageDown => "page_down",
            Self::Home => "home",
            Self::End => "end",
            Self::CapsLock => "caps_lock",
            Self::ScrollLock => "scroll_lock",
            Self::NumLock => "num_lock",
            Self::PrintScreen => "print_screen",
            Self::Pause => "pause",
            Self::Menu => "menu",
            Self::F1 => "f1",
            Self::F2 => "f2",
            Self::F3 => "f3",
            Self::F4 => "f4",
            Self::F5 => "f5",
            Self::F6 => "f6",
            Self::F7 => "f7",
            Self::F8 => "f8",
            Self::F9 => "f9",
            Self::F10 => "f10",
            Self::F11 => "f11",
            Self::F12 => "f12",
            Self::Raw(name) => name,
        })
    }
}
impl Serialize for Key {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// A set of native shortcut modifiers; compose with `|`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(u16);
impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const CTRL: Self = Self(1);
    pub const SHIFT: Self = Self(2);
    pub const ALT: Self = Self(4);
    pub const SUPER: Self = Self(8);
    pub const HYPER: Self = Self(16);
    pub const META: Self = Self(32);
    pub const CAPS_LOCK: Self = Self(64);
    pub const NUM_LOCK: Self = Self(128);
    /// Resolved by kitty using the server's configured `kitty_mod` value.
    pub const KITTY_MOD: Self = Self(256);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl BitOr for Modifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut separator = "";
        for (modifier, name) in [
            (Self::CTRL, "ctrl"),
            (Self::SHIFT, "shift"),
            (Self::ALT, "alt"),
            (Self::SUPER, "super"),
            (Self::HYPER, "hyper"),
            (Self::META, "meta"),
            (Self::CAPS_LOCK, "caps_lock"),
            (Self::NUM_LOCK, "num_lock"),
            (Self::KITTY_MOD, "kitty_mod"),
        ] {
            if self.contains(modifier) {
                write!(f, "{separator}{name}")?;
                separator = "+";
            }
        }
        Ok(())
    }
}

/// One native send-key shortcut. Raw preserves an entire shortcut expression;
/// native key codes and unrecognized extensions can still be rejected by kitty.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyChord {
    Key { key: Key, modifiers: Modifiers },
    Raw(String),
}
impl KeyChord {
    pub fn new(key: impl Into<Key>) -> Self {
        Self::modified(Modifiers::NONE, key)
    }
    pub fn modified(modifiers: Modifiers, key: impl Into<Key>) -> Self {
        Self::Key {
            key: key.into(),
            modifiers,
        }
    }
    pub fn ctrl(key: impl Into<Key>) -> Self {
        Self::modified(Modifiers::CTRL, key)
    }
    pub fn alt(key: impl Into<Key>) -> Self {
        Self::modified(Modifiers::ALT, key)
    }
    pub fn shift(key: impl Into<Key>) -> Self {
        Self::modified(Modifiers::SHIFT, key)
    }
    pub fn raw(shortcut: impl Into<String>) -> Self {
        Self::Raw(shortcut.into())
    }
}
impl From<Key> for KeyChord {
    fn from(key: Key) -> Self {
        Self::new(key)
    }
}
impl From<char> for KeyChord {
    fn from(key: char) -> Self {
        Self::new(key)
    }
}
impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key { key, modifiers } => {
                if !modifiers.is_empty() {
                    write!(f, "{modifiers}+")?;
                }
                write!(f, "{key}")
            }
            Self::Raw(shortcut) => f.write_str(shortcut),
        }
    }
}
impl Serialize for KeyChord {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Tall,
    Fat,
    Grid,
    Stack,
    Splits,
    Horizontal,
    Vertical,
    #[serde(untagged)]
    Custom(String),
}
impl Layout {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Tall => "tall",
            Self::Fat => "fat",
            Self::Grid => "grid",
            Self::Stack => "stack",
            Self::Splits => "splits",
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
            Self::Custom(s) => s,
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("value outside its finite native domain")]
pub struct InvalidValue;

/// Nonempty server-side path. Empty screenshot save paths switch to PNG data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPath(String);
impl ServerPath {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidValue> {
        let value = value.into();
        if value.is_empty() || value.contains('\0') {
            Err(InvalidValue)
        } else {
            Ok(Self(value))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// No NaN/infinity can silently serialize as JSON null.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Finite(pub(crate) f64);
impl Finite {
    pub fn new(value: f64) -> Result<Self, InvalidValue> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(InvalidValue)
        }
    }
    pub fn get(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Opacity(Finite);
impl Opacity {
    pub fn new(value: f64) -> Result<Self, InvalidValue> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(Finite::new(value)?))
        } else {
            Err(InvalidValue)
        }
    }
    pub fn get(self) -> f64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Rgb(u32);
impl Rgb {
    pub fn new(value: u32) -> Result<Self, InvalidValue> {
        if value <= 0xffffff {
            Ok(Self(value))
        } else {
            Err(InvalidValue)
        }
    }
    pub fn get(self) -> u32 {
        self.0
    }
}

/// Canonical environment/user-variable operation; values can contain '='.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assignment {
    Set { name: String, value: String },
    Remove(String),
}
impl Assignment {
    pub fn as_argument(&self) -> String {
        match self {
            Self::Set { name, value } => format!("{name}={value}"),
            Self::Remove(name) => name.clone(),
        }
    }
}

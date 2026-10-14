//! Open match expressions and semantic finite argument types.

use serde::Serialize;
use serde_json::{Value, json};

/// A kitty window ID, not an OS/platform window ID or process ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct TabId(pub u64);

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

/// Multi-window selection for commands whose native fields are
/// match_window/match_tab/all. It is not the self-capable window-match profile.
#[derive(Debug, Clone, Default)]
pub struct WindowSet {
    pub match_window: Option<String>,
    pub match_tab: Option<String>,
    pub all: bool,
}
impl WindowSet {
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

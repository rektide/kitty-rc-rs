//! Server configuration and installed kitten entrances. No local filesystem
//! reads, deployment, cache invalidation or installation is hidden here.
use crate::targets::{Assignment, Finite, InvalidValue, Rgb, WindowTarget};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, request};
use kitty_rc_schema::PlainRequest;
use serde::{Serialize, Serializer};
use serde_json::{Value, json};

/// A config override is a name/value pair, not an environment assignment.
/// In particular, an empty value does not mean remove a setting. Kitty parses
/// the value using that setting's configuration grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigOverride {
    Setting { name: String, value: String },
    Raw(String),
}
impl ConfigOverride {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Setting {
            name: name.into(),
            value: value.into(),
        }
    }

    /// Explicit escape hatch for a complete native override expression.
    pub fn raw(value: impl Into<String>) -> Self {
        Self::Raw(value.into())
    }

    /// Kitty clamps sizes below its minimum when applying configuration. Keep
    /// that server behavior, but reject non-finite numeric inputs here.
    pub fn font_size(points: f64) -> Result<Self, InvalidValue> {
        let points = Finite::new(points)?;
        Ok(Self::new("font_size", points.get().to_string()))
    }

    pub fn background(color: Rgb) -> Self {
        Self::new("background", format!("#{:06x}", color.get()))
    }

    pub fn foreground(color: Rgb) -> Self {
        Self::new("foreground", format!("#{:06x}", color.get()))
    }

    pub fn as_argument(&self) -> String {
        match self {
            Self::Setting { name, value } => format!("{name}={value}"),
            Self::Raw(value) => value.clone(),
        }
    }
}
impl Serialize for ConfigOverride {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_argument())
    }
}

#[derive(Debug, Clone, Default, bon::Builder)]
pub struct LoadConfig {
    /// Server paths; empty reloads previous files. Relative paths resolve at
    /// the server's kitty config directory, not this client's current cwd.
    #[builder(default)]
    pub paths: Vec<String>,
    /// Serialized as native name=value entries, never a concatenation or map.
    #[builder(default)]
    pub overrides: Vec<ConfigOverride>,
    #[builder(default)]
    pub ignore_previous_overrides: bool,
}
impl LoadConfig {
    /// Empty paths reload the server's previously loaded configuration files.
    pub fn new(paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            paths: paths.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
}
impl Command for LoadConfig {
    type Output = ();
    fn request(&self) -> PlainRequest {
        request(
            "load-config",
            json!({"paths": self.paths, "override": self.overrides, "ignore_overrides": self.ignore_previous_overrides}),
        )
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}
impl CanSubmitWithoutReply for LoadConfig {}

#[derive(Debug, Clone, bon::Builder)]
pub struct Env {
    pub changes: Vec<Assignment>,
}
impl Env {
    pub fn new(changes: impl IntoIterator<Item = Assignment>) -> Self {
        Self {
            changes: changes.into_iter().collect(),
        }
    }
}
impl Command for Env {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut env = serde_json::Map::new();
        for change in &self.changes {
            match change {
                Assignment::Set { name, value } => {
                    env.insert(name.clone(), json!(value));
                }
                Assignment::Remove(name) => {
                    env.insert(format!("{name}="), json!(""));
                }
            }
        }
        request("env", json!({"env": env}))
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KittenResult {
    /// UI dispatch, no matching window, or a handler whose return type is not
    /// str/bool: the upstream outer contract distinguishes none of these.
    NoData,
    Text(String),
    Boolean(bool),
}

#[derive(Debug, Clone, bon::Builder)]
#[builder(on(String, into))]
pub struct Kitten {
    /// Builtin name or server-side Python file. Relative paths resolve in the
    /// server config directory. No availability/installation is inferred.
    pub name: String,
    #[builder(default)]
    pub args: Vec<String>,
    #[builder(default)]
    pub target: WindowTarget,
}
impl Kitten {
    pub fn new(name: impl Into<String>, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            args: args.into_iter().map(Into::into).collect(),
            target: WindowTarget::default(),
        }
    }
}
impl Command for Kitten {
    type Output = KittenResult;
    fn request(&self) -> PlainRequest {
        let mut p = self.target.payload();
        p["kitten"] = json!(self.name);
        p["args"] = json!(self.args);
        request("kitten", p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    // Custom/installed code and UI lifetimes are not audited reuse profiles.
    fn decode(data: Option<Value>) -> Result<KittenResult, DecodeError> {
        match data {
            None => Ok(KittenResult::NoData),
            Some(Value::String(s)) => Ok(KittenResult::Text(s)),
            Some(Value::Bool(b)) => Ok(KittenResult::Boolean(b)),
            _ => Err(DecodeError::MissingData),
        }
    }
}

//! Server configuration and installed kitten entrances. No local filesystem
//! reads, deployment, cache invalidation or installation is hidden here.
use crate::targets::{Assignment, WindowTarget};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, request};
use kitty_rc_schema::PlainRequest;
use serde_json::{Value, json};

#[derive(Debug, Clone, Default, bon::Builder)]
pub struct LoadConfig {
    /// Server paths; empty reloads previous files. Relative paths resolve at
    /// the server's kitty config directory, not this client's current cwd.
    #[builder(default)]
    pub paths: Vec<String>,
    /// Native name=value entries. Never a scalar concatenation or map.
    #[builder(default)]
    pub overrides: Vec<String>,
    #[builder(default)]
    pub ignore_previous_overrides: bool,
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

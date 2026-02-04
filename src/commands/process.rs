use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Map;

fn is_false(v: &bool) -> bool {
    !v
}

#[derive(Debug, Deserialize)]
pub struct ProcessInfo {
    pub pid: Option<u64>,
    #[serde(default)]
    pub cmdline: Vec<String>,
    pub cwd: Option<String>,
}

#[derive(Builder, Serialize)]
pub struct RunCommand {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    data: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    cmdline: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    env: Option<Map<String, serde_json::Value>>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    allow_remote_control: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    remote_control_password: Option<String>,
}

impl RunCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("run").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct KittenCommand {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    args: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,
}

impl KittenCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("kitten").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct LaunchCommand {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    args: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    window_title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    cwd: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    env: Option<Map<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    var: Option<Map<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    tab_title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    window_type: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    keep_focus: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    copy_colors: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    copy_cmdline: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    copy_env: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    hold: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    location: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    allow_remote_control: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    remote_control_password: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    stdin_source: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    stdin_add_formatting: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    stdin_add_line_wrap_markers: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    spacing: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    marker: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    logo: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    logo_position: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    logo_alpha: Option<f32>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    os_window_title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    os_window_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    os_window_class: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    os_window_state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    watcher: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    bias: Option<i32>,
}

impl LaunchCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("launch").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct EnvCommand {
    env: Map<String, serde_json::Value>,
}

impl EnvCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.env.is_empty() {
            return Err(CommandError::MissingParameter(
                "env".to_string(),
                "env".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("env").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetUserVarsCommand {
    var: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,
}

impl SetUserVarsCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.var.is_empty() {
            return Err(CommandError::MissingParameter(
                "var".to_string(),
                "set-user-vars".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-user-vars")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct LoadConfigCommand {
    paths: Vec<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "override")]
    override_config: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    ignore_overrides: bool,
}

impl LoadConfigCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.paths.is_empty() {
            return Err(CommandError::MissingParameter(
                "paths".to_string(),
                "load-config".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("load-config").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct ResizeOSWindowCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    incremental: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    action: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    unit: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    width: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    height: Option<i32>,
}

impl ResizeOSWindowCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("resize-os-window")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct DisableLigaturesCommand {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    strategy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_window: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,
}

impl DisableLigaturesCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("disable-ligatures")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct SignalChildCommand {
    signals: Vec<i32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,
}

impl SignalChildCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.signals.is_empty() {
            return Err(CommandError::MissingParameter(
                "signals".to_string(),
                "signal-child".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("signal-child").payload(payload).build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_basic() {
        let cmd = RunCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "run");
    }

    #[test]
    fn test_run_with_options() {
        let cmd = RunCommand::builder()
            .data("test data".to_string())
            .cmdline("bash".to_string())
            .allow_remote_control(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "run");
    }

    #[test]
    fn test_kitten_basic() {
        let cmd = KittenCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "kitten");
    }

    #[test]
    fn test_kitten_with_args() {
        let cmd = KittenCommand::builder()
            .args("diff".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "kitten");
    }

    #[test]
    fn test_launch_basic() {
        let cmd = LaunchCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "launch");
    }

    #[test]
    fn test_launch_with_options() {
        let cmd = LaunchCommand::builder()
            .args("bash".to_string())
            .window_title("Test".to_string())
            .cwd("/home".to_string())
            .keep_focus(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "launch");
    }

    #[test]
    fn test_env_basic() {
        let mut env_map = Map::new();
        env_map.insert(
            "PATH".to_string(),
            serde_json::Value::String("/usr/bin".to_string()),
        );
        let cmd = EnvCommand::builder().env(env_map).build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "env");
    }

    #[test]
    fn test_env_empty() {
        let cmd = EnvCommand::builder().env(Map::new()).build().to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "env");
            assert_eq!(cmd_name, "env");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_set_user_vars_basic() {
        let cmd = SetUserVarsCommand::builder()
            .var(vec!["var1".to_string(), "var2".to_string()])
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-user-vars");
    }

    #[test]
    fn test_set_user_vars_empty() {
        let cmd = SetUserVarsCommand::builder()
            .var(vec![])
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "var");
            assert_eq!(cmd_name, "set-user-vars");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_load_config_basic() {
        let cmd = LoadConfigCommand::builder()
            .paths(vec!["kitty.conf".to_string()])
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "load-config");
    }

    #[test]
    fn test_load_config_empty() {
        let cmd = LoadConfigCommand::builder()
            .paths(vec![])
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "paths");
            assert_eq!(cmd_name, "load-config");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_resize_os_window_basic() {
        let cmd = ResizeOSWindowCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "resize-os-window");
    }

    #[test]
    fn test_resize_os_window_with_options() {
        let cmd = ResizeOSWindowCommand::builder()
            .width(800)
            .height(600)
            .unit("px".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "resize-os-window");
    }

    #[test]
    fn test_disable_ligatures_basic() {
        let cmd = DisableLigaturesCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "disable-ligatures");
    }

    #[test]
    fn test_disable_ligatures_with_options() {
        let cmd = DisableLigaturesCommand::builder()
            .strategy("never".to_string())
            .all(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "disable-ligatures");
    }

    #[test]
    fn test_signal_child_basic() {
        let cmd = SignalChildCommand::builder()
            .signals(vec![9, 15])
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "signal-child");
    }

    #[test]
    fn test_signal_child_empty() {
        let cmd = SignalChildCommand::builder()
            .signals(vec![])
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "signals");
            assert_eq!(cmd_name, "signal-child");
        } else {
            panic!("Expected MissingParameter error");
        }
    }
}

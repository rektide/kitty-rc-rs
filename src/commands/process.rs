use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;
use serde::Deserialize;
use serde_json::Map;

#[derive(Debug, Deserialize)]
pub struct ProcessInfo {
    pub pid: Option<u64>,
    #[serde(default)]
    pub cmdline: Vec<String>,
    pub cwd: Option<String>,
}

pub struct RunCommand {
    data: Option<String>,
    cmdline: Option<String>,
    env: Option<Map<String, serde_json::Value>>,
    allow_remote_control: bool,
    remote_control_password: Option<String>,
}

impl RunCommand {
    pub fn new() -> Self {
        Self {
            data: None,
            cmdline: None,
            env: None,
            allow_remote_control: false,
            remote_control_password: None,
        }
    }

    pub fn data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }

    pub fn cmdline(mut self, value: impl Into<String>) -> Self {
        self.cmdline = Some(value.into());
        self
    }

    pub fn env(mut self, value: Map<String, serde_json::Value>) -> Self {
        self.env = Some(value);
        self
    }

    pub fn allow_remote_control(mut self, value: bool) -> Self {
        self.allow_remote_control = value;
        self
    }

    pub fn remote_control_password(mut self, value: impl Into<String>) -> Self {
        self.remote_control_password = Some(value.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if let Some(data) = self.data {
            payload.insert("data".to_string(), serde_json::Value::String(data));
        }

        if let Some(cmdline) = self.cmdline {
            payload.insert("cmdline".to_string(), serde_json::Value::String(cmdline));
        }

        if let Some(env) = self.env {
            payload.insert("env".to_string(), serde_json::Value::Object(env));
        }

        if self.allow_remote_control {
            payload.insert(
                "allow_remote_control".to_string(),
                serde_json::Value::Bool(true),
            );
        }

        if let Some(remote_control_password) = self.remote_control_password {
            payload.insert(
                "remote_control_password".to_string(),
                serde_json::Value::String(remote_control_password),
            );
        }

        Ok(CommandBuilder::new("run")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct KittenCommand {
    args: Option<String>,
    match_spec: Option<String>,
}

impl KittenCommand {
    pub fn new() -> Self {
        Self {
            args: None,
            match_spec: None,
        }
    }

    pub fn args(mut self, value: impl Into<String>) -> Self {
        self.args = Some(value.into());
        self
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if let Some(args) = self.args {
            payload.insert("args".to_string(), serde_json::Value::String(args));
        }

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        Ok(CommandBuilder::new("kitten")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct LaunchCommand {
    args: Option<String>,
    window_title: Option<String>,
    cwd: Option<String>,
    env: Option<Map<String, serde_json::Value>>,
    var: Option<Map<String, serde_json::Value>>,
    tab_title: Option<String>,
    window_type: Option<String>,
    keep_focus: bool,
    copy_colors: bool,
    copy_cmdline: bool,
    copy_env: bool,
    hold: bool,
    location: Option<String>,
    allow_remote_control: bool,
    remote_control_password: Option<String>,
    stdin_source: Option<String>,
    stdin_add_formatting: bool,
    stdin_add_line_wrap_markers: bool,
    spacing: Option<String>,
    marker: Option<String>,
    logo: Option<String>,
    logo_position: Option<String>,
    logo_alpha: Option<f32>,
    self_window: bool,
    os_window_title: Option<String>,
    os_window_name: Option<String>,
    os_window_class: Option<String>,
    os_window_state: Option<String>,
    color: Option<String>,
    watcher: Option<String>,
    bias: Option<i32>,
}

impl LaunchCommand {
    pub fn new() -> Self {
        Self {
            args: None,
            window_title: None,
            cwd: None,
            env: None,
            var: None,
            tab_title: None,
            window_type: None,
            keep_focus: false,
            copy_colors: false,
            copy_cmdline: false,
            copy_env: false,
            hold: false,
            location: None,
            allow_remote_control: false,
            remote_control_password: None,
            stdin_source: None,
            stdin_add_formatting: false,
            stdin_add_line_wrap_markers: false,
            spacing: None,
            marker: None,
            logo: None,
            logo_position: None,
            logo_alpha: None,
            self_window: false,
            os_window_title: None,
            os_window_name: None,
            os_window_class: None,
            os_window_state: None,
            color: None,
            watcher: None,
            bias: None,
        }
    }

    pub fn args(mut self, value: impl Into<String>) -> Self {
        self.args = Some(value.into());
        self
    }

    pub fn window_title(mut self, value: impl Into<String>) -> Self {
        self.window_title = Some(value.into());
        self
    }

    pub fn cwd(mut self, value: impl Into<String>) -> Self {
        self.cwd = Some(value.into());
        self
    }

    pub fn env(mut self, value: Map<String, serde_json::Value>) -> Self {
        self.env = Some(value);
        self
    }

    pub fn var(mut self, value: Map<String, serde_json::Value>) -> Self {
        self.var = Some(value);
        self
    }

    pub fn tab_title(mut self, value: impl Into<String>) -> Self {
        self.tab_title = Some(value.into());
        self
    }

    pub fn window_type(mut self, value: impl Into<String>) -> Self {
        self.window_type = Some(value.into());
        self
    }

    pub fn keep_focus(mut self, value: bool) -> Self {
        self.keep_focus = value;
        self
    }

    pub fn copy_colors(mut self, value: bool) -> Self {
        self.copy_colors = value;
        self
    }

    pub fn copy_cmdline(mut self, value: bool) -> Self {
        self.copy_cmdline = value;
        self
    }

    pub fn copy_env(mut self, value: bool) -> Self {
        self.copy_env = value;
        self
    }

    pub fn hold(mut self, value: bool) -> Self {
        self.hold = value;
        self
    }

    pub fn location(mut self, value: impl Into<String>) -> Self {
        self.location = Some(value.into());
        self
    }

    pub fn allow_remote_control(mut self, value: bool) -> Self {
        self.allow_remote_control = value;
        self
    }

    pub fn remote_control_password(mut self, value: impl Into<String>) -> Self {
        self.remote_control_password = Some(value.into());
        self
    }

    pub fn stdin_source(mut self, value: impl Into<String>) -> Self {
        self.stdin_source = Some(value.into());
        self
    }

    pub fn stdin_add_formatting(mut self, value: bool) -> Self {
        self.stdin_add_formatting = value;
        self
    }

    pub fn stdin_add_line_wrap_markers(mut self, value: bool) -> Self {
        self.stdin_add_line_wrap_markers = value;
        self
    }

    pub fn spacing(mut self, value: impl Into<String>) -> Self {
        self.spacing = Some(value.into());
        self
    }

    pub fn marker(mut self, value: impl Into<String>) -> Self {
        self.marker = Some(value.into());
        self
    }

    pub fn logo(mut self, value: impl Into<String>) -> Self {
        self.logo = Some(value.into());
        self
    }

    pub fn logo_position(mut self, value: impl Into<String>) -> Self {
        self.logo_position = Some(value.into());
        self
    }

    pub fn logo_alpha(mut self, value: f32) -> Self {
        self.logo_alpha = Some(value);
        self
    }

    pub fn self_window(mut self, value: bool) -> Self {
        self.self_window = value;
        self
    }

    pub fn os_window_title(mut self, value: impl Into<String>) -> Self {
        self.os_window_title = Some(value.into());
        self
    }

    pub fn os_window_name(mut self, value: impl Into<String>) -> Self {
        self.os_window_name = Some(value.into());
        self
    }

    pub fn os_window_class(mut self, value: impl Into<String>) -> Self {
        self.os_window_class = Some(value.into());
        self
    }

    pub fn os_window_state(mut self, value: impl Into<String>) -> Self {
        self.os_window_state = Some(value.into());
        self
    }

    pub fn color(mut self, value: impl Into<String>) -> Self {
        self.color = Some(value.into());
        self
    }

    pub fn watcher(mut self, value: impl Into<String>) -> Self {
        self.watcher = Some(value.into());
        self
    }

    pub fn bias(mut self, value: i32) -> Self {
        self.bias = Some(value);
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if let Some(args) = self.args {
            payload.insert("args".to_string(), serde_json::Value::String(args));
        }

        if let Some(window_title) = self.window_title {
            payload.insert(
                "window_title".to_string(),
                serde_json::Value::String(window_title),
            );
        }

        if let Some(cwd) = self.cwd {
            payload.insert("cwd".to_string(), serde_json::Value::String(cwd));
        }

        if let Some(env) = self.env {
            payload.insert("env".to_string(), serde_json::Value::Object(env));
        }

        if let Some(var) = self.var {
            payload.insert("var".to_string(), serde_json::Value::Object(var));
        }

        if let Some(tab_title) = self.tab_title {
            payload.insert(
                "tab_title".to_string(),
                serde_json::Value::String(tab_title),
            );
        }

        if let Some(window_type) = self.window_type {
            payload.insert(
                "window_type".to_string(),
                serde_json::Value::String(window_type),
            );
        }

        if self.keep_focus {
            payload.insert("keep_focus".to_string(), serde_json::Value::Bool(true));
        }

        if self.copy_colors {
            payload.insert("copy_colors".to_string(), serde_json::Value::Bool(true));
        }

        if self.copy_cmdline {
            payload.insert("copy_cmdline".to_string(), serde_json::Value::Bool(true));
        }

        if self.copy_env {
            payload.insert("copy_env".to_string(), serde_json::Value::Bool(true));
        }

        if self.hold {
            payload.insert("hold".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(location) = self.location {
            payload.insert("location".to_string(), serde_json::Value::String(location));
        }

        if self.allow_remote_control {
            payload.insert(
                "allow_remote_control".to_string(),
                serde_json::Value::Bool(true),
            );
        }

        if let Some(remote_control_password) = self.remote_control_password {
            payload.insert(
                "remote_control_password".to_string(),
                serde_json::Value::String(remote_control_password),
            );
        }

        if let Some(stdin_source) = self.stdin_source {
            payload.insert(
                "stdin_source".to_string(),
                serde_json::Value::String(stdin_source),
            );
        }

        if self.stdin_add_formatting {
            payload.insert(
                "stdin_add_formatting".to_string(),
                serde_json::Value::Bool(true),
            );
        }

        if self.stdin_add_line_wrap_markers {
            payload.insert(
                "stdin_add_line_wrap_markers".to_string(),
                serde_json::Value::Bool(true),
            );
        }

        if let Some(spacing) = self.spacing {
            payload.insert("spacing".to_string(), serde_json::Value::String(spacing));
        }

        if let Some(marker) = self.marker {
            payload.insert("marker".to_string(), serde_json::Value::String(marker));
        }

        if let Some(logo) = self.logo {
            payload.insert("logo".to_string(), serde_json::Value::String(logo));
        }

        if let Some(logo_position) = self.logo_position {
            payload.insert(
                "logo_position".to_string(),
                serde_json::Value::String(logo_position),
            );
        }

        if let Some(logo_alpha) = self.logo_alpha {
            payload.insert("logo_alpha".to_string(), serde_json::json!(logo_alpha));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(os_window_title) = self.os_window_title {
            payload.insert(
                "os_window_title".to_string(),
                serde_json::Value::String(os_window_title),
            );
        }

        if let Some(os_window_name) = self.os_window_name {
            payload.insert(
                "os_window_name".to_string(),
                serde_json::Value::String(os_window_name),
            );
        }

        if let Some(os_window_class) = self.os_window_class {
            payload.insert(
                "os_window_class".to_string(),
                serde_json::Value::String(os_window_class),
            );
        }

        if let Some(os_window_state) = self.os_window_state {
            payload.insert(
                "os_window_state".to_string(),
                serde_json::Value::String(os_window_state),
            );
        }

        if let Some(color) = self.color {
            payload.insert("color".to_string(), serde_json::Value::String(color));
        }

        if let Some(watcher) = self.watcher {
            payload.insert("watcher".to_string(), serde_json::Value::String(watcher));
        }

        if let Some(bias) = self.bias {
            payload.insert("bias".to_string(), serde_json::json!(bias));
        }

        Ok(CommandBuilder::new("launch")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct EnvCommand {
    env: Map<String, serde_json::Value>,
}

impl EnvCommand {
    pub fn new(env: Map<String, serde_json::Value>) -> Self {
        Self { env }
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.env.is_empty() {
            return Err(CommandError::MissingParameter(
                "env".to_string(),
                "env".to_string(),
            ));
        }

        payload.insert("env".to_string(), serde_json::Value::Object(self.env));

        Ok(CommandBuilder::new("env")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetUserVarsCommand {
    var: Vec<String>,
    match_spec: Option<String>,
}

impl SetUserVarsCommand {
    pub fn new(var: Vec<String>) -> Self {
        Self {
            var,
            match_spec: None,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.var.is_empty() {
            return Err(CommandError::MissingParameter(
                "var".to_string(),
                "set-user-vars".to_string(),
            ));
        }

        payload.insert("var".to_string(), serde_json::json!(self.var));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        Ok(CommandBuilder::new("set-user-vars")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct LoadConfigCommand {
    paths: Vec<String>,
    override_config: bool,
    ignore_overrides: bool,
}

impl LoadConfigCommand {
    pub fn new(paths: Vec<String>) -> Self {
        Self {
            paths,
            override_config: false,
            ignore_overrides: false,
        }
    }

    pub fn override_config(mut self, value: bool) -> Self {
        self.override_config = value;
        self
    }

    pub fn ignore_overrides(mut self, value: bool) -> Self {
        self.ignore_overrides = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.paths.is_empty() {
            return Err(CommandError::MissingParameter(
                "paths".to_string(),
                "load-config".to_string(),
            ));
        }

        payload.insert("paths".to_string(), serde_json::json!(self.paths));

        if self.override_config {
            payload.insert("override".to_string(), serde_json::Value::Bool(true));
        }

        if self.ignore_overrides {
            payload.insert(
                "ignore_overrides".to_string(),
                serde_json::Value::Bool(true),
            );
        }

        Ok(CommandBuilder::new("load-config")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct ResizeOSWindowCommand {
    match_spec: Option<String>,
    self_window: bool,
    incremental: bool,
    action: Option<String>,
    unit: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
}

impl ResizeOSWindowCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            self_window: false,
            incremental: false,
            action: None,
            unit: None,
            width: None,
            height: None,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn self_window(mut self, value: bool) -> Self {
        self.self_window = value;
        self
    }

    pub fn incremental(mut self, value: bool) -> Self {
        self.incremental = value;
        self
    }

    pub fn action(mut self, value: impl Into<String>) -> Self {
        self.action = Some(value.into());
        self
    }

    pub fn unit(mut self, value: impl Into<String>) -> Self {
        self.unit = Some(value.into());
        self
    }

    pub fn width(mut self, value: i32) -> Self {
        self.width = Some(value);
        self
    }

    pub fn height(mut self, value: i32) -> Self {
        self.height = Some(value);
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        if self.incremental {
            payload.insert("incremental".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(action) = self.action {
            payload.insert("action".to_string(), serde_json::Value::String(action));
        }

        if let Some(unit) = self.unit {
            payload.insert("unit".to_string(), serde_json::Value::String(unit));
        }

        if let Some(width) = self.width {
            payload.insert("width".to_string(), serde_json::json!(width));
        }

        if let Some(height) = self.height {
            payload.insert("height".to_string(), serde_json::json!(height));
        }

        Ok(CommandBuilder::new("resize-os-window")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct DisableLigaturesCommand {
    strategy: Option<String>,
    match_window: Option<String>,
    match_tab: Option<String>,
    all: bool,
}

impl DisableLigaturesCommand {
    pub fn new() -> Self {
        Self {
            strategy: None,
            match_window: None,
            match_tab: None,
            all: false,
        }
    }

    pub fn strategy(mut self, value: impl Into<String>) -> Self {
        self.strategy = Some(value.into());
        self
    }

    pub fn match_window(mut self, spec: impl Into<String>) -> Self {
        self.match_window = Some(spec.into());
        self
    }

    pub fn match_tab(mut self, spec: impl Into<String>) -> Self {
        self.match_tab = Some(spec.into());
        self
    }

    pub fn all(mut self, value: bool) -> Self {
        self.all = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if let Some(strategy) = self.strategy {
            payload.insert("strategy".to_string(), serde_json::Value::String(strategy));
        }

        if let Some(match_window) = self.match_window {
            payload.insert(
                "match_window".to_string(),
                serde_json::Value::String(match_window),
            );
        }

        if let Some(match_tab) = self.match_tab {
            payload.insert(
                "match_tab".to_string(),
                serde_json::Value::String(match_tab),
            );
        }

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("disable-ligatures")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SignalChildCommand {
    signals: Vec<i32>,
    match_spec: Option<String>,
}

impl SignalChildCommand {
    pub fn new(signals: Vec<i32>) -> Self {
        Self {
            signals,
            match_spec: None,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.signals.is_empty() {
            return Err(CommandError::MissingParameter(
                "signals".to_string(),
                "signal-child".to_string(),
            ));
        }

        payload.insert("signals".to_string(), serde_json::json!(self.signals));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        Ok(CommandBuilder::new("signal-child")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_basic() {
        let cmd = RunCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "run");
    }

    #[test]
    fn test_run_with_options() {
        let cmd = RunCommand::new()
            .data("test data")
            .cmdline("bash")
            .allow_remote_control(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "run");
    }

    #[test]
    fn test_kitten_basic() {
        let cmd = KittenCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "kitten");
    }

    #[test]
    fn test_kitten_with_args() {
        let cmd = KittenCommand::new().args("diff").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "kitten");
    }

    #[test]
    fn test_launch_basic() {
        let cmd = LaunchCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "launch");
    }

    #[test]
    fn test_launch_with_options() {
        let cmd = LaunchCommand::new()
            .args("bash")
            .window_title("Test")
            .cwd("/home")
            .keep_focus(true)
            .build();
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
        let cmd = EnvCommand::new(env_map).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "env");
    }

    #[test]
    fn test_env_empty() {
        let cmd = EnvCommand::new(Map::new()).build();
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
        let cmd = SetUserVarsCommand::new(vec!["var1".to_string(), "var2".to_string()]).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-user-vars");
    }

    #[test]
    fn test_set_user_vars_empty() {
        let cmd = SetUserVarsCommand::new(vec![]).build();
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
        let cmd = LoadConfigCommand::new(vec!["kitty.conf".to_string()]).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "load-config");
    }

    #[test]
    fn test_load_config_empty() {
        let cmd = LoadConfigCommand::new(vec![]).build();
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
        let cmd = ResizeOSWindowCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "resize-os-window");
    }

    #[test]
    fn test_resize_os_window_with_options() {
        let cmd = ResizeOSWindowCommand::new()
            .width(800)
            .height(600)
            .unit("px")
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "resize-os-window");
    }

    #[test]
    fn test_disable_ligatures_basic() {
        let cmd = DisableLigaturesCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "disable-ligatures");
    }

    #[test]
    fn test_disable_ligatures_with_options() {
        let cmd = DisableLigaturesCommand::new()
            .strategy("never")
            .all(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "disable-ligatures");
    }

    #[test]
    fn test_signal_child_basic() {
        let cmd = SignalChildCommand::new(vec![9, 15]).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "signal-child");
    }

    #[test]
    fn test_signal_child_empty() {
        let cmd = SignalChildCommand::new(vec![]).build();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "signals");
            assert_eq!(cmd_name, "signal-child");
        } else {
            panic!("Expected MissingParameter error");
        }
    }
}

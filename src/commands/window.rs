use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;
use crate::commands::process::ProcessInfo;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct WindowInfo {
    pub id: Option<u64>,
    pub title: Option<String>,
    pub pid: Option<u64>,
    pub cwd: Option<String>,
    #[serde(default)]
    pub cmdline: Vec<String>,
    #[serde(default)]
    pub foreground_processes: Vec<ProcessInfo>,
}

#[derive(Debug, Deserialize)]
pub struct TabInfo {
    #[serde(default)]
    pub windows: Vec<WindowInfo>,
}

#[derive(Debug, Deserialize)]
pub struct OsInstance {
    #[serde(default)]
    pub tabs: Vec<TabInfo>,
}

pub fn parse_response_data(data: &Value) -> Result<Vec<OsInstance>, serde_json::Error> {
    let parsed_data = if let Some(s) = data.as_str() {
        serde_json::from_str(s)?
    } else {
        data.clone()
    };
    serde_json::from_value(parsed_data)
}

use crate::protocol::KittyResponse;

pub struct LsCommand {
    all_env_vars: bool,
    match_spec: Option<String>,
    match_tab: Option<String>,
    self_window: bool,
}

impl LsCommand {
    pub fn new() -> Self {
        Self {
            all_env_vars: false,
            match_spec: None,
            match_tab: None,
            self_window: false,
        }
    }

    pub fn all_env_vars(mut self, value: bool) -> Self {
        self.all_env_vars = value;
        self
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn match_tab(mut self, spec: impl Into<String>) -> Self {
        self.match_tab = Some(spec.into());
        self
    }

    pub fn self_window(mut self, value: bool) -> Self {
        self.self_window = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if self.all_env_vars {
            payload.insert("all_env_vars".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(match_tab) = self.match_tab {
            payload.insert("match_tab".to_string(), serde_json::Value::String(match_tab));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("ls")
            .payload(serde_json::Value::Object(payload))
            .build())
    }

    pub fn parse_response(response: &KittyResponse) -> Result<Vec<OsInstance>, serde_json::Error> {
        if let Some(data) = &response.data {
            parse_response_data(data)
        } else {
            Ok(vec![])
        }
    }
}

pub struct SendTextCommand {
    data: String,
    match_spec: Option<String>,
    match_tab: Option<String>,
    all: bool,
    exclude_active: bool,
    bracketed_paste: String,
}

impl SendTextCommand {
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            match_spec: None,
            match_tab: None,
            all: false,
            exclude_active: false,
            bracketed_paste: "disable".to_string(),
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
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

    pub fn exclude_active(mut self, value: bool) -> Self {
        self.exclude_active = value;
        self
    }

    pub fn bracketed_paste(mut self, value: impl Into<String>) -> Self {
        self.bracketed_paste = value.into();
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if self.data.is_empty() {
            return Err(CommandError::MissingParameter(
                "data".to_string(),
                "send-text".to_string(),
            ));
        }

        payload.insert("data".to_string(), serde_json::Value::String(self.data));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(match_tab) = self.match_tab {
            payload.insert("match_tab".to_string(), serde_json::Value::String(match_tab));
        }

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        if self.exclude_active {
            payload.insert("exclude_active".to_string(), serde_json::Value::Bool(true));
        }

        if self.bracketed_paste != "disable" {
            payload.insert("bracketed_paste".to_string(), serde_json::Value::String(self.bracketed_paste));
        }

        Ok(CommandBuilder::new("send-text")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SendKeyCommand {
    keys: String,
    match_spec: Option<String>,
    match_tab: Option<String>,
    all: bool,
    exclude_active: bool,
}

impl SendKeyCommand {
    pub fn new(keys: impl Into<String>) -> Self {
        Self {
            keys: keys.into(),
            match_spec: None,
            match_tab: None,
            all: false,
            exclude_active: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
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

    pub fn exclude_active(mut self, value: bool) -> Self {
        self.exclude_active = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if self.keys.is_empty() {
            return Err(CommandError::MissingParameter(
                "keys".to_string(),
                "send-key".to_string(),
            ));
        }

        payload.insert("keys".to_string(), serde_json::Value::String(self.keys));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(match_tab) = self.match_tab {
            payload.insert("match_tab".to_string(), serde_json::Value::String(match_tab));
        }

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        if self.exclude_active {
            payload.insert("exclude_active".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("send-key")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct CloseWindowCommand {
    match_spec: Option<String>,
    self_window: bool,
    ignore_no_match: bool,
}

impl CloseWindowCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            self_window: false,
            ignore_no_match: false,
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

    pub fn ignore_no_match(mut self, value: bool) -> Self {
        self.ignore_no_match = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        if self.ignore_no_match {
            payload.insert("ignore_no_match".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("close-window")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct ResizeWindowCommand {
    match_spec: Option<String>,
    self_window: bool,
    increment: i32,
    axis: String,
}

impl ResizeWindowCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            self_window: false,
            increment: 2,
            axis: "horizontal".to_string(),
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

    pub fn increment(mut self, value: i32) -> Self {
        self.increment = value;
        self
    }

    pub fn axis(mut self, value: impl Into<String>) -> Self {
        self.axis = value.into();
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        payload.insert("increment".to_string(), serde_json::Value::Number(self.increment.into()));

        if self.axis != "horizontal" {
            payload.insert("axis".to_string(), serde_json::Value::String(self.axis));
        }

        Ok(CommandBuilder::new("resize-window")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct FocusWindowCommand {
    match_spec: Option<String>,
}

impl FocusWindowCommand {
    pub fn new() -> Self {
        Self { match_spec: None }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        Ok(CommandBuilder::new("focus-window")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SelectWindowCommand {
    match_spec: Option<String>,
    title: Option<String>,
    exclude_active: bool,
    reactivate_prev_tab: bool,
}

impl SelectWindowCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            title: None,
            exclude_active: false,
            reactivate_prev_tab: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = Some(value.into());
        self
    }

    pub fn exclude_active(mut self, value: bool) -> Self {
        self.exclude_active = value;
        self
    }

    pub fn reactivate_prev_tab(mut self, value: bool) -> Self {
        self.reactivate_prev_tab = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(title) = self.title {
            payload.insert("title".to_string(), serde_json::Value::String(title));
        }

        if self.exclude_active {
            payload.insert("exclude_active".to_string(), serde_json::Value::Bool(true));
        }

        if self.reactivate_prev_tab {
            payload.insert("reactivate_prev_tab".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("select-window")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct NewWindowCommand {
    args: Option<String>,
    title: Option<String>,
    cwd: Option<String>,
    keep_focus: bool,
    window_type: Option<String>,
    new_tab: bool,
    tab_title: Option<String>,
}

impl NewWindowCommand {
    pub fn new() -> Self {
        Self {
            args: None,
            title: None,
            cwd: None,
            keep_focus: false,
            window_type: None,
            new_tab: false,
            tab_title: None,
        }
    }

    pub fn args(mut self, value: impl Into<String>) -> Self {
        self.args = Some(value.into());
        self
    }

    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = Some(value.into());
        self
    }

    pub fn cwd(mut self, value: impl Into<String>) -> Self {
        self.cwd = Some(value.into());
        self
    }

    pub fn keep_focus(mut self, value: bool) -> Self {
        self.keep_focus = value;
        self
    }

    pub fn window_type(mut self, value: impl Into<String>) -> Self {
        self.window_type = Some(value.into());
        self
    }

    pub fn new_tab(mut self, value: bool) -> Self {
        self.new_tab = value;
        self
    }

    pub fn tab_title(mut self, value: impl Into<String>) -> Self {
        self.tab_title = Some(value.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(args) = self.args {
            payload.insert("args".to_string(), serde_json::Value::String(args));
        }

        if let Some(title) = self.title {
            payload.insert("title".to_string(), serde_json::Value::String(title));
        }

        if let Some(cwd) = self.cwd {
            payload.insert("cwd".to_string(), serde_json::Value::String(cwd));
        }

        if self.keep_focus {
            payload.insert("keep_focus".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(window_type) = self.window_type {
            payload.insert("window_type".to_string(), serde_json::Value::String(window_type));
        }

        if self.new_tab {
            payload.insert("new_tab".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(tab_title) = self.tab_title {
            payload.insert("tab_title".to_string(), serde_json::Value::String(tab_title));
        }

        Ok(CommandBuilder::new("new-window")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct DetachWindowCommand {
    match_spec: Option<String>,
    target_tab: Option<String>,
    self_window: bool,
    stay_in_tab: bool,
}

impl DetachWindowCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            target_tab: None,
            self_window: false,
            stay_in_tab: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn target_tab(mut self, spec: impl Into<String>) -> Self {
        self.target_tab = Some(spec.into());
        self
    }

    pub fn self_window(mut self, value: bool) -> Self {
        self.self_window = value;
        self
    }

    pub fn stay_in_tab(mut self, value: bool) -> Self {
        self.stay_in_tab = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(target_tab) = self.target_tab {
            payload.insert("target_tab".to_string(), serde_json::Value::String(target_tab));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        if self.stay_in_tab {
            payload.insert("stay_in_tab".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("detach-window")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetWindowTitleCommand {
    match_spec: Option<String>,
    title: String,
    temporary: bool,
}

impl SetWindowTitleCommand {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            match_spec: None,
            title: title.into(),
            temporary: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn temporary(mut self, value: bool) -> Self {
        self.temporary = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if self.title.is_empty() {
            return Err(CommandError::MissingParameter("title".to_string(), "set-window-title".to_string()));
        }

        payload.insert("title".to_string(), serde_json::Value::String(self.title));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.temporary {
            payload.insert("temporary".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("set-window-title")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetWindowLogoCommand {
    match_spec: Option<String>,
    data: Option<String>,
    position: Option<String>,
    alpha: Option<f32>,
    self_window: bool,
}

impl SetWindowLogoCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            data: None,
            position: None,
            alpha: None,
            self_window: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }

    pub fn position(mut self, value: impl Into<String>) -> Self {
        self.position = Some(value.into());
        self
    }

    pub fn alpha(mut self, value: f32) -> Self {
        self.alpha = Some(value);
        self
    }

    pub fn self_window(mut self, value: bool) -> Self {
        self.self_window = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(data) = self.data {
            payload.insert("data".to_string(), serde_json::Value::String(data));
        }

        if let Some(position) = self.position {
            payload.insert("position".to_string(), serde_json::Value::String(position));
        }

        if let Some(alpha) = self.alpha {
            payload.insert("alpha".to_string(), serde_json::json!(alpha));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("set-window-logo")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct GetTextCommand {
    match_spec: Option<String>,
    extent: Option<String>,
    ansi: bool,
    cursor: bool,
    wrap_markers: bool,
    clear_selection: bool,
    self_window: bool,
}

impl GetTextCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            extent: None,
            ansi: false,
            cursor: false,
            wrap_markers: false,
            clear_selection: false,
            self_window: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn extent(mut self, value: impl Into<String>) -> Self {
        self.extent = Some(value.into());
        self
    }

    pub fn ansi(mut self, value: bool) -> Self {
        self.ansi = value;
        self
    }

    pub fn cursor(mut self, value: bool) -> Self {
        self.cursor = value;
        self
    }

    pub fn wrap_markers(mut self, value: bool) -> Self {
        self.wrap_markers = value;
        self
    }

    pub fn clear_selection(mut self, value: bool) -> Self {
        self.clear_selection = value;
        self
    }

    pub fn self_window(mut self, value: bool) -> Self {
        self.self_window = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(extent) = self.extent {
            payload.insert("extent".to_string(), serde_json::Value::String(extent));
        }

        if self.ansi {
            payload.insert("ansi".to_string(), serde_json::Value::Bool(true));
        }

        if self.cursor {
            payload.insert("cursor".to_string(), serde_json::Value::Bool(true));
        }

        if self.wrap_markers {
            payload.insert("wrap_markers".to_string(), serde_json::Value::Bool(true));
        }

        if self.clear_selection {
            payload.insert("clear_selection".to_string(), serde_json::Value::Bool(true));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("get-text")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct ScrollWindowCommand {
    amount: i32,
    match_spec: Option<String>,
}

impl ScrollWindowCommand {
    pub fn new(amount: i32) -> Self {
        Self {
            amount,
            match_spec: None,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        payload.insert("amount".to_string(), serde_json::json!(self.amount));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        Ok(CommandBuilder::new("scroll-window")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct CreateMarkerCommand {
    match_spec: Option<String>,
    self_window: bool,
    marker_spec: Option<String>,
}

impl CreateMarkerCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            self_window: false,
            marker_spec: None,
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

    pub fn marker_spec(mut self, value: impl Into<String>) -> Self {
        self.marker_spec = Some(value.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(marker_spec) = self.marker_spec {
            payload.insert("marker_spec".to_string(), serde_json::Value::String(marker_spec));
        }

        Ok(CommandBuilder::new("create-marker")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct RemoveMarkerCommand {
    match_spec: Option<String>,
    self_window: bool,
}

impl RemoveMarkerCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            self_window: false,
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

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.self_window {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("remove-marker")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls_basic() {
        let cmd = LsCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "ls");
    }

    #[test]
    fn test_ls_with_options() {
        let cmd = LsCommand::new()
            .all_env_vars(true)
            .self_window(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "ls");
    }

    #[test]
    fn test_ls_with_match() {
        let cmd = LsCommand::new().match_spec("id:1").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "ls");
    }

    #[test]
    fn test_send_text_basic() {
        let cmd = SendTextCommand::new("text:hello").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send-text");
    }

    #[test]
    fn test_send_text_empty() {
        let cmd = SendTextCommand::new("").build();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "data");
            assert_eq!(cmd_name, "send-text");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_send_text_with_options() {
        let cmd = SendTextCommand::new("text:test")
            .match_spec("id:1")
            .all(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send-text");
    }

    #[test]
    fn test_send_key_basic() {
        let cmd = SendKeyCommand::new("ctrl+c").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send-key");
    }

    #[test]
    fn test_send_key_empty() {
        let cmd = SendKeyCommand::new("").build();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "keys");
            assert_eq!(cmd_name, "send-key");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_send_key_with_options() {
        let cmd = SendKeyCommand::new("alt+f4")
            .match_spec("id:1")
            .all(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send-key");
    }

    #[test]
    fn test_close_window_basic() {
        let cmd = CloseWindowCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-window");
    }

    #[test]
    fn test_close_window_with_options() {
        let cmd = CloseWindowCommand::new()
            .match_spec("id:1")
            .self_window(true)
            .ignore_no_match(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-window");
    }

    #[test]
    fn test_resize_window_basic() {
        let cmd = ResizeWindowCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "resize-window");
    }

    #[test]
    fn test_resize_window_with_options() {
        let cmd = ResizeWindowCommand::new()
            .match_spec("id:1")
            .increment(5)
            .axis("vertical")
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "resize-window");
    }

    #[test]
    fn test_focus_window_basic() {
        let cmd = FocusWindowCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "focus-window");
    }

    #[test]
    fn test_focus_window_with_match() {
        let cmd = FocusWindowCommand::new().match_spec("id:1").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "focus-window");
    }

    #[test]
    fn test_select_window_basic() {
        let cmd = SelectWindowCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "select-window");
    }

    #[test]
    fn test_select_window_with_options() {
        let cmd = SelectWindowCommand::new()
            .match_spec("id:1")
            .title("Select Me")
            .exclude_active(true)
            .reactivate_prev_tab(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "select-window");
    }

    #[test]
    fn test_new_window_basic() {
        let cmd = NewWindowCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "new-window");
    }

    #[test]
    fn test_new_window_with_options() {
        let cmd = NewWindowCommand::new()
            .args("bash")
            .title("My Window")
            .cwd("/home/user")
            .keep_focus(true)
            .window_type("overlay")
            .new_tab(true)
            .tab_title("New Tab")
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "new-window");
    }

    #[test]
    fn test_detach_window_basic() {
        let cmd = DetachWindowCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-window");
    }

    #[test]
    fn test_detach_window_with_options() {
        let cmd = DetachWindowCommand::new()
            .match_spec("id:1")
            .target_tab("id:2")
            .self_window(true)
            .stay_in_tab(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-window");
    }

    #[test]
    fn test_set_window_title_basic() {
        let cmd = SetWindowTitleCommand::new("My Title").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-window-title");
    }

    #[test]
    fn test_set_window_title_empty() {
        let cmd = SetWindowTitleCommand::new("").build();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "title");
            assert_eq!(cmd_name, "set-window-title");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_set_window_title_with_options() {
        let cmd = SetWindowTitleCommand::new("New Title")
            .match_spec("id:1")
            .temporary(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-window-title");
    }

    #[test]
    fn test_set_window_logo_basic() {
        let cmd = SetWindowLogoCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-window-logo");
    }

    #[test]
    fn test_set_window_logo_with_options() {
        let cmd = SetWindowLogoCommand::new()
            .match_spec("id:1")
            .data("base64data")
            .position("top-left")
            .alpha(0.5)
            .self_window(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-window-logo");
    }

    #[test]
    fn test_get_text_basic() {
        let cmd = GetTextCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "get-text");
    }

    #[test]
    fn test_get_text_with_options() {
        let cmd = GetTextCommand::new()
            .match_spec("id:1")
            .extent("all")
            .ansi(true)
            .cursor(true)
            .wrap_markers(true)
            .clear_selection(true)
            .self_window(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "get-text");
    }

    #[test]
    fn test_scroll_window_basic() {
        let cmd = ScrollWindowCommand::new(5).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "scroll-window");
    }

    #[test]
    fn test_scroll_window_with_match() {
        let cmd = ScrollWindowCommand::new(-5).match_spec("id:1").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "scroll-window");
    }

    #[test]
    fn test_create_marker_basic() {
        let cmd = CreateMarkerCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "create-marker");
    }

    #[test]
    fn test_create_marker_with_options() {
        let cmd = CreateMarkerCommand::new()
            .match_spec("id:1")
            .self_window(true)
            .marker_spec("marker1")
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "create-marker");
    }

    #[test]
    fn test_remove_marker_basic() {
        let cmd = RemoveMarkerCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "remove-marker");
    }

    #[test]
    fn test_remove_marker_with_options() {
        let cmd = RemoveMarkerCommand::new()
            .match_spec("id:1")
            .self_window(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "remove-marker");
    }

    #[test]
    fn test_parse_ls_response() {
        let json_data = serde_json::json!([
            {
                "tabs": [
                    {
                        "windows": [
                            {
                                "id": 1,
                                "title": "Test Window",
                                "pid": 12345,
                                "cwd": "/home/user",
                                "cmdline": ["/bin/bash"],
                                "foreground_processes": []
                            }
                        ]
                    }
                ]
            }
        ]);

        let response = KittyResponse {
            ok: true,
            data: Some(json_data),
            error: None,
        };

        let instances = LsCommand::parse_response(&response).unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].tabs.len(), 1);
        assert_eq!(instances[0].tabs[0].windows.len(), 1);
        assert_eq!(instances[0].tabs[0].windows[0].id, Some(1));
        assert_eq!(instances[0].tabs[0].windows[0].title, Some("Test Window".to_string()));
    }

    #[test]
    fn test_parse_ls_response_empty() {
        let response = KittyResponse {
            ok: true,
            data: None,
            error: None,
        };

        let instances = LsCommand::parse_response(&response).unwrap();
        assert!(instances.is_empty());
    }
}

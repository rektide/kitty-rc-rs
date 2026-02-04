use crate::command::CommandBuilder;
use crate::commands::process::ProcessInfo;
use crate::error::CommandError;
use crate::protocol::KittyMessage;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

fn is_false(v: &bool) -> bool {
    !v
}

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
    pub at_prompt: Option<bool>,
    pub columns: Option<u64>,
    pub created_at: Option<u64>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub in_alternate_screen: Option<bool>,
    pub is_active: Option<bool>,
    pub is_focused: Option<bool>,
    pub is_self: Option<bool>,
    pub last_cmd_exit_status: Option<i32>,
    pub last_reported_cmdline: Option<String>,
    pub lines: Option<u64>,
    #[serde(default)]
    pub user_vars: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct LayoutOpts {
    #[serde(default)]
    pub bias: i32,
    #[serde(default)]
    pub full_size: i32,
    #[serde(default)]
    pub mirrored: String,
}

#[derive(Debug, Deserialize)]
pub struct WindowGroup {
    pub id: u64,
    #[serde(default)]
    pub window_ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct AllWindows {
    #[serde(default)]
    pub active_group_history: Vec<u64>,
    pub active_group_idx: Option<u64>,
    #[serde(default)]
    pub window_groups: Vec<WindowGroup>,
}

#[derive(Debug, Deserialize)]
pub struct LayoutState {
    pub all_windows: Option<AllWindows>,
    #[serde(default)]
    pub biased_map: HashMap<String, serde_json::Value>,
    pub class: Option<String>,
    #[serde(default)]
    pub main_bias: Vec<f32>,
    pub opts: Option<LayoutOpts>,
}

#[derive(Debug, Deserialize)]
pub struct TabGroup {
    pub id: u64,
    #[serde(default)]
    pub windows: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TabInfo {
    #[serde(default)]
    pub windows: Vec<WindowInfo>,
    #[serde(default)]
    pub active_window_history: Vec<u64>,
    #[serde(default)]
    pub enabled_layouts: Vec<String>,
    #[serde(default)]
    pub groups: Vec<TabGroup>,
    pub id: Option<u64>,
    pub is_active: Option<bool>,
    pub is_focused: Option<bool>,
    pub layout: Option<String>,
    pub layout_opts: Option<LayoutOpts>,
    pub layout_state: Option<LayoutState>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OsInstance {
    #[serde(default)]
    pub tabs: Vec<TabInfo>,
    pub background_opacity: Option<f32>,
    pub id: Option<u64>,
    pub is_active: Option<bool>,
    pub is_focused: Option<bool>,
    pub last_focused: Option<bool>,
    pub platform_window_id: Option<u64>,
    pub wm_class: Option<String>,
    pub wm_name: Option<String>,
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

#[derive(Builder, Serialize)]
pub struct LsCommand {
    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all_env_vars: bool,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,
}

impl LsCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("ls").payload(payload).build())
    }

    pub fn parse_response(response: &KittyResponse) -> Result<Vec<OsInstance>, serde_json::Error> {
        if let Some(data) = &response.data {
            parse_response_data(data)
        } else {
            Ok(vec![])
        }
    }
}

#[derive(Builder, Serialize)]
pub struct SendTextCommand {
    data: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    exclude_active: bool,

    #[builder(default = "disable".to_string())]
    #[serde(skip_serializing_if = "is_skip_bracketed_paste")]
    bracketed_paste: String,
}

fn is_skip_bracketed_paste(v: &String) -> bool {
    v == "disable"
}

impl SendTextCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.data.is_empty() {
            return Err(CommandError::MissingParameter(
                "data".to_string(),
                "send-text".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("send-text").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct SendKeyCommand {
    keys: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    exclude_active: bool,
}

impl SendKeyCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.keys.is_empty() {
            return Err(CommandError::MissingParameter(
                "keys".to_string(),
                "send-key".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("send-key").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct CloseWindowCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    ignore_no_match: bool,
}

impl CloseWindowCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("close-window").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct ResizeWindowCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,

    #[builder(default = 2)]
    increment: i32,

    #[builder(default = "horizontal".to_string())]
    #[serde(skip_serializing_if = "is_skip_horizontal")]
    axis: String,
}

fn is_skip_horizontal(v: &String) -> bool {
    v == "horizontal"
}

impl ResizeWindowCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("resize-window")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct FocusWindowCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,
}

impl FocusWindowCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("focus-window").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct SelectWindowCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    title: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    exclude_active: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    reactivate_prev_tab: bool,
}

impl SelectWindowCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("select-window")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct NewWindowCommand {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    args: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    cwd: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    keep_focus: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    window_type: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    new_tab: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    tab_title: Option<String>,
}

impl NewWindowCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("new-window").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct DetachWindowCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    target_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    stay_in_tab: bool,
}

impl DetachWindowCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("detach-window")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetWindowTitleCommand {
    title: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    temporary: bool,
}

impl SetWindowTitleCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.title.is_empty() {
            return Err(CommandError::MissingParameter(
                "title".to_string(),
                "set-window-title".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-window-title")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetWindowLogoCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    data: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    position: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    alpha: Option<f32>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,
}

impl SetWindowLogoCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-window-logo")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct GetTextCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    extent: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    ansi: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    cursor: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    wrap_markers: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    clear_selection: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,
}

impl GetTextCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("get-text").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct ScrollWindowCommand {
    amount: i32,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,
}

impl ScrollWindowCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("scroll-window")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct CreateMarkerCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    marker_spec: Option<String>,
}

impl CreateMarkerCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("create-marker")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct RemoveMarkerCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_window: bool,
}

impl RemoveMarkerCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("remove-marker")
            .payload(payload)
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls_basic() {
        let cmd = LsCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "ls");
    }

    #[test]
    fn test_ls_with_options() {
        let cmd = LsCommand::builder()
            .all_env_vars(true)
            .self_window(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "ls");
    }

    #[test]
    fn test_ls_with_match() {
        let cmd = LsCommand::builder()
            .match_spec("id:1".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "ls");
    }

    #[test]
    fn test_send_text_basic() {
        let cmd = SendTextCommand::builder()
            .data("text:hello".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send-text");
    }

    #[test]
    fn test_send_text_empty() {
        let cmd = SendTextCommand::builder()
            .data("".to_string())
            .build()
            .to_message();
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
        let cmd = SendTextCommand::builder()
            .data("text:test".to_string())
            .match_spec("id:1".to_string())
            .all(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send-text");
    }

    #[test]
    fn test_send_key_basic() {
        let cmd = SendKeyCommand::builder()
            .keys("ctrl+c".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send-key");
    }

    #[test]
    fn test_send_key_empty() {
        let cmd = SendKeyCommand::builder()
            .keys("".to_string())
            .build()
            .to_message();
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
        let cmd = SendKeyCommand::builder()
            .keys("alt+f4".to_string())
            .match_spec("id:1".to_string())
            .all(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send-key");
    }

    #[test]
    fn test_close_window_basic() {
        let cmd = CloseWindowCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-window");
    }

    #[test]
    fn test_close_window_with_options() {
        let cmd = CloseWindowCommand::builder()
            .match_spec("id:1".to_string())
            .self_window(true)
            .ignore_no_match(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-window");
    }

    #[test]
    fn test_resize_window_basic() {
        let cmd = ResizeWindowCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "resize-window");
    }

    #[test]
    fn test_resize_window_with_options() {
        let cmd = ResizeWindowCommand::builder()
            .match_spec("id:1".to_string())
            .increment(5)
            .axis("vertical".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "resize-window");
    }

    #[test]
    fn test_focus_window_basic() {
        let cmd = FocusWindowCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "focus-window");
    }

    #[test]
    fn test_focus_window_with_match() {
        let cmd = FocusWindowCommand::builder()
            .match_spec("id:1".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "focus-window");
    }

    #[test]
    fn test_select_window_basic() {
        let cmd = SelectWindowCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "select-window");
    }

    #[test]
    fn test_select_window_with_options() {
        let cmd = SelectWindowCommand::builder()
            .match_spec("id:1".to_string())
            .title("Select Me".to_string())
            .exclude_active(true)
            .reactivate_prev_tab(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "select-window");
    }

    #[test]
    fn test_new_window_basic() {
        let cmd = NewWindowCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "new-window");
    }

    #[test]
    fn test_new_window_with_options() {
        let cmd = NewWindowCommand::builder()
            .args("bash".to_string())
            .title("My Window".to_string())
            .cwd("/home/user".to_string())
            .keep_focus(true)
            .window_type("overlay".to_string())
            .new_tab(true)
            .tab_title("New Tab".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "new-window");
    }

    #[test]
    fn test_detach_window_basic() {
        let cmd = DetachWindowCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-window");
    }

    #[test]
    fn test_detach_window_with_options() {
        let cmd = DetachWindowCommand::builder()
            .match_spec("id:1".to_string())
            .target_tab("id:2".to_string())
            .self_window(true)
            .stay_in_tab(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-window");
    }

    #[test]
    fn test_set_window_title_basic() {
        let cmd = SetWindowTitleCommand::builder()
            .title("My Title".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-window-title");
    }

    #[test]
    fn test_set_window_title_empty() {
        let cmd = SetWindowTitleCommand::builder()
            .title("".to_string())
            .build()
            .to_message();
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
        let cmd = SetWindowTitleCommand::builder()
            .title("New Title".to_string())
            .match_spec("id:1".to_string())
            .temporary(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-window-title");
    }

    #[test]
    fn test_set_window_logo_basic() {
        let cmd = SetWindowLogoCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-window-logo");
    }

    #[test]
    fn test_set_window_logo_with_options() {
        let cmd = SetWindowLogoCommand::builder()
            .match_spec("id:1".to_string())
            .data("base64data".to_string())
            .position("top-left".to_string())
            .alpha(0.5)
            .self_window(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-window-logo");
    }

    #[test]
    fn test_get_text_basic() {
        let cmd = GetTextCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "get-text");
    }

    #[test]
    fn test_get_text_with_options() {
        let cmd = GetTextCommand::builder()
            .match_spec("id:1".to_string())
            .extent("all".to_string())
            .ansi(true)
            .cursor(true)
            .wrap_markers(true)
            .clear_selection(true)
            .self_window(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "get-text");
    }

    #[test]
    fn test_scroll_window_basic() {
        let cmd = ScrollWindowCommand::builder()
            .amount(5)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "scroll-window");
    }

    #[test]
    fn test_scroll_window_with_match() {
        let cmd = ScrollWindowCommand::builder()
            .amount(-5)
            .match_spec("id:1".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "scroll-window");
    }

    #[test]
    fn test_create_marker_basic() {
        let cmd = CreateMarkerCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "create-marker");
    }

    #[test]
    fn test_create_marker_with_options() {
        let cmd = CreateMarkerCommand::builder()
            .match_spec("id:1".to_string())
            .self_window(true)
            .marker_spec("marker1".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "create-marker");
    }

    #[test]
    fn test_remove_marker_basic() {
        let cmd = RemoveMarkerCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "remove-marker");
    }

    #[test]
    fn test_remove_marker_with_options() {
        let cmd = RemoveMarkerCommand::builder()
            .match_spec("id:1".to_string())
            .self_window(true)
            .build()
            .to_message();
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
        assert_eq!(
            instances[0].tabs[0].windows[0].title,
            Some("Test Window".to_string())
        );
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

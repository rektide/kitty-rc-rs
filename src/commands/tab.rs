use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;
use bon::Builder;
use serde::Serialize;

fn is_false(v: &bool) -> bool {
    !v
}

#[derive(Builder, Serialize)]
pub struct FocusTabCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,
}

impl FocusTabCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("focus-tab").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetTabTitleCommand {
    title: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,
}

impl SetTabTitleCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.title.is_empty() {
            return Err(CommandError::MissingParameter(
                "title".to_string(),
                "set-tab-title".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-tab-title")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct CloseTabCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_tab: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    ignore_no_match: bool,
}

impl CloseTabCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("close-tab").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct DetachTabCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    target_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_tab: bool,
}

impl DetachTabCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("detach-tab").payload(payload).build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_tab_basic() {
        let cmd = FocusTabCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "focus-tab");
    }

    #[test]
    fn test_focus_tab_with_match() {
        let cmd = FocusTabCommand::builder()
            .match_spec("id:0".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "focus-tab");
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_set_tab_title() {
        let cmd = SetTabTitleCommand::builder()
            .title("My Tab".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-tab-title");
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_set_tab_title_empty() {
        let cmd = SetTabTitleCommand::builder()
            .title("".to_string())
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "title");
            assert_eq!(cmd_name, "set-tab-title");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_set_tab_title_with_match() {
        let cmd = SetTabTitleCommand::builder()
            .title("New Title".to_string())
            .match_spec("id:1".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-tab-title");
    }

    #[test]
    fn test_close_tab_basic() {
        let cmd = CloseTabCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-tab");
    }

    #[test]
    fn test_close_tab_with_match() {
        let cmd = CloseTabCommand::builder()
            .match_spec("id:2".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-tab");
    }

    #[test]
    fn test_close_tab_self() {
        let cmd = CloseTabCommand::builder()
            .self_tab(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-tab");
    }

    #[test]
    fn test_close_tab_ignore_no_match() {
        let cmd = CloseTabCommand::builder()
            .ignore_no_match(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-tab");
    }

    #[test]
    fn test_detach_tab_basic() {
        let cmd = DetachTabCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }

    #[test]
    fn test_detach_tab_with_match() {
        let cmd = DetachTabCommand::builder()
            .match_spec("id:0".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }

    #[test]
    fn test_detach_tab_with_target_tab() {
        let cmd = DetachTabCommand::builder()
            .target_tab("id:1".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }

    #[test]
    fn test_detach_tab_self() {
        let cmd = DetachTabCommand::builder()
            .self_tab(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }

    #[test]
    fn test_detach_tab_all_options() {
        let cmd = DetachTabCommand::builder()
            .match_spec("id:0".to_string())
            .target_tab("id:1".to_string())
            .self_tab(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }
}

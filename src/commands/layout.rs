use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;
use bon::Builder;
use serde::Serialize;

fn is_false(v: &bool) -> bool {
    !v
}

#[derive(Builder, Serialize)]
pub struct GotoLayoutCommand {
    layout: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,
}

impl GotoLayoutCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.layout.is_empty() {
            return Err(CommandError::MissingParameter(
                "layout".to_string(),
                "goto-layout".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("goto-layout").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetEnabledLayoutsCommand {
    layouts: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    configured: bool,
}

impl SetEnabledLayoutsCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.layouts.is_empty() {
            return Err(CommandError::MissingParameter(
                "layouts".to_string(),
                "set-enabled-layouts".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-enabled-layouts")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct LastUsedLayoutCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,
}

impl LastUsedLayoutCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("last-used-layout")
            .payload(payload)
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto_layout() {
        let cmd = GotoLayoutCommand::builder()
            .layout("tall".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "goto-layout");
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_goto_layout_empty() {
        let cmd = GotoLayoutCommand::builder()
            .layout("".to_string())
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "layout");
            assert_eq!(cmd_name, "goto-layout");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_goto_layout_with_match() {
        let cmd = GotoLayoutCommand::builder()
            .layout("grid".to_string())
            .match_spec("id:0".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "goto-layout");
    }

    #[test]
    fn test_set_enabled_layouts() {
        let layouts = vec!["tall".to_string(), "grid".to_string()];
        let cmd = SetEnabledLayoutsCommand::builder()
            .layouts(layouts)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-enabled-layouts");
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_set_enabled_layouts_empty() {
        let cmd = SetEnabledLayoutsCommand::builder()
            .layouts(vec![])
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "layouts");
            assert_eq!(cmd_name, "set-enabled-layouts");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_set_enabled_layouts_with_match() {
        let layouts = vec!["stack".to_string()];
        let cmd = SetEnabledLayoutsCommand::builder()
            .layouts(layouts)
            .match_spec("id:1".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-enabled-layouts");
    }

    #[test]
    fn test_set_enabled_layouts_configured() {
        let layouts = vec!["tall".to_string()];
        let cmd = SetEnabledLayoutsCommand::builder()
            .layouts(layouts)
            .configured(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-enabled-layouts");
    }

    #[test]
    fn test_last_used_layout() {
        let cmd = LastUsedLayoutCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "last-used-layout");
    }

    #[test]
    fn test_last_used_layout_with_match() {
        let cmd = LastUsedLayoutCommand::builder()
            .match_spec("id:0".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "last-used-layout");
    }

    #[test]
    fn test_last_used_layout_all() {
        let cmd = LastUsedLayoutCommand::builder()
            .all(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "last-used-layout");
    }
}

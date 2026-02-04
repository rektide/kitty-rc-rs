use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;
use bon::{bon, Builder};
use serde::Serialize;
use serde_json::Map;

fn is_false(v: &bool) -> bool {
    !v
}

#[derive(Builder, Serialize)]
pub struct SetBackgroundOpacityCommand {
    opacity: f32,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_window: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    toggle: bool,
}

impl SetBackgroundOpacityCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err(CommandError::ValidationError(
                "opacity must be between 0.0 and 1.0".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-background-opacity")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetBackgroundImageCommand {
    data: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    layout: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    configured: bool,
}

impl SetBackgroundImageCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.data.is_empty() {
            return Err(CommandError::MissingParameter(
                "data".to_string(),
                "set-background-image".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-background-image")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetColorsCommand {
    colors: Map<String, serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_window: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    configured: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    reset: bool,
}

impl SetColorsCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.colors.is_empty() {
            return Err(CommandError::MissingParameter(
                "colors".to_string(),
                "set-colors".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-colors").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetFontSizeCommand {
    #[serde(rename = "size")]
    size: f64,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    increment_op: Option<String>,
}

impl SetFontSizeCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-font-size")
            .payload(payload)
            .build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetSpacingCommand {
    settings: Map<String, serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_window: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    match_tab: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    all: bool,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false")]
    configured: bool,
}

impl SetSpacingCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.settings.is_empty() {
            return Err(CommandError::MissingParameter(
                "settings".to_string(),
                "set-spacing".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-spacing").payload(payload).build())
    }
}

#[derive(Builder, Serialize)]
pub struct SetTabColorCommand {
    colors: Map<String, serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[builder(default = false)]
    #[serde(skip_serializing_if = "is_false", rename = "self")]
    self_tab: bool,
}

impl SetTabColorCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        if self.colors.is_empty() {
            return Err(CommandError::MissingParameter(
                "colors".to_string(),
                "set-tab-color".to_string(),
            ));
        }

        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("set-tab-color")
            .payload(payload)
            .build())
    }
}

#[derive(Serialize)]
pub struct GetColorsCommand {
    #[serde(skip_serializing_if = "Option::is_none", rename = "match", default)]
    match_spec: Option<String>,

    #[serde(skip_serializing_if = "is_false", default)]
    configured: bool,
}

#[bon]
impl GetColorsCommand {
    #[builder]
    pub fn new(match_spec: Option<String>, #[builder(default = false)] configured: bool) -> Self {
        Self {
            match_spec,
            configured,
        }
    }
}

impl GetColorsCommand {
    pub fn to_message(self) -> Result<KittyMessage, CommandError> {
        let payload =
            serde_json::to_value(self).map_err(|e| CommandError::ValidationError(e.to_string()))?;

        Ok(CommandBuilder::new("get-colors").payload(payload).build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_background_opacity_basic() {
        let cmd = SetBackgroundOpacityCommand::builder()
            .opacity(0.5)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-background-opacity");
    }

    #[test]
    fn test_set_background_opacity_invalid() {
        let cmd = SetBackgroundOpacityCommand::builder()
            .opacity(1.5)
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::ValidationError(msg)) = cmd {
            assert!(msg.contains("opacity"));
        } else {
            panic!("Expected ValidationError error");
        }
    }

    #[test]
    fn test_set_background_opacity_with_options() {
        let cmd = SetBackgroundOpacityCommand::builder()
            .opacity(0.8)
            .all(true)
            .toggle(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-background-opacity");
    }

    #[test]
    fn test_set_background_image_basic() {
        let cmd = SetBackgroundImageCommand::builder()
            .data("base64data".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-background-image");
    }

    #[test]
    fn test_set_background_image_empty() {
        let cmd = SetBackgroundImageCommand::builder()
            .data("".to_string())
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "data");
            assert_eq!(cmd_name, "set-background-image");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_set_background_image_with_options() {
        let cmd = SetBackgroundImageCommand::builder()
            .data("base64data".to_string())
            .layout("tiled".to_string())
            .all(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-background-image");
    }

    #[test]
    fn test_set_colors_basic() {
        let mut colors = Map::new();
        colors.insert(
            "foreground".to_string(),
            serde_json::Value::String("#ffffff".to_string()),
        );
        let cmd = SetColorsCommand::builder()
            .colors(colors)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-colors");
    }

    #[test]
    fn test_set_colors_empty() {
        let cmd = SetColorsCommand::builder()
            .colors(Map::new())
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "colors");
            assert_eq!(cmd_name, "set-colors");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_set_colors_with_options() {
        let mut colors = Map::new();
        colors.insert(
            "background".to_string(),
            serde_json::Value::String("#000000".to_string()),
        );
        let cmd = SetColorsCommand::builder()
            .colors(colors)
            .all(true)
            .reset(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-colors");
    }

    #[test]
    fn test_set_font_size_basic() {
        let cmd = SetFontSizeCommand::builder()
            .size(14.0)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-font-size");
    }

    #[test]
    fn test_set_font_size_with_options() {
        let cmd = SetFontSizeCommand::builder()
            .size(16.0)
            .all(true)
            .increment_op("set".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-font-size");
    }

    #[test]
    fn test_set_spacing_basic() {
        let mut settings = Map::new();
        settings.insert("padding".to_string(), serde_json::json!(10));
        let cmd = SetSpacingCommand::builder()
            .settings(settings)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-spacing");
    }

    #[test]
    fn test_set_spacing_empty() {
        let cmd = SetSpacingCommand::builder()
            .settings(Map::new())
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "settings");
            assert_eq!(cmd_name, "set-spacing");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_set_spacing_with_options() {
        let mut settings = Map::new();
        settings.insert("margin".to_string(), serde_json::json!(5));
        let cmd = SetSpacingCommand::builder()
            .settings(settings)
            .all(true)
            .configured(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-spacing");
    }

    #[test]
    fn test_set_tab_color_basic() {
        let mut colors = Map::new();
        colors.insert(
            "active_tab_foreground".to_string(),
            serde_json::Value::String("#ffffff".to_string()),
        );
        let cmd = SetTabColorCommand::builder()
            .colors(colors)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-tab-color");
    }

    #[test]
    fn test_set_tab_color_empty() {
        let cmd = SetTabColorCommand::builder()
            .colors(Map::new())
            .build()
            .to_message();
        assert!(cmd.is_err());
        if let Err(CommandError::MissingParameter(field, cmd_name)) = cmd {
            assert_eq!(field, "colors");
            assert_eq!(cmd_name, "set-tab-color");
        } else {
            panic!("Expected MissingParameter error");
        }
    }

    #[test]
    fn test_set_tab_color_with_options() {
        let mut colors = Map::new();
        colors.insert(
            "active_tab_background".to_string(),
            serde_json::Value::String("#000000".to_string()),
        );
        let cmd = SetTabColorCommand::builder()
            .colors(colors)
            .self_tab(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-tab-color");
    }

    #[test]
    fn test_get_colors_basic() {
        let cmd = GetColorsCommand::builder().build().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "get-colors");
    }

    #[test]
    fn test_get_colors_with_options() {
        let cmd = GetColorsCommand::builder()
            .match_spec("id:1".to_string())
            .configured(true)
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "get-colors");
    }
}

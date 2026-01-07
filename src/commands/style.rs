use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;
use serde_json::Map;

pub struct SetBackgroundOpacityCommand {
    opacity: f32,
    match_window: Option<String>,
    match_tab: Option<String>,
    all: bool,
    toggle: bool,
}

impl SetBackgroundOpacityCommand {
    pub fn new(opacity: f32) -> Self {
        Self {
            opacity,
            match_window: None,
            match_tab: None,
            all: false,
            toggle: false,
        }
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

    pub fn toggle(mut self, value: bool) -> Self {
        self.toggle = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err(CommandError::ValidationError("opacity must be between 0.0 and 1.0".to_string()));
        }

        payload.insert("opacity".to_string(), serde_json::json!(self.opacity));

        if let Some(match_window) = self.match_window {
            payload.insert("match_window".to_string(), serde_json::Value::String(match_window));
        }

        if let Some(match_tab) = self.match_tab {
            payload.insert("match_tab".to_string(), serde_json::Value::String(match_tab));
        }

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        if self.toggle {
            payload.insert("toggle".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("set-background-opacity")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetBackgroundImageCommand {
    data: String,
    match_spec: Option<String>,
    layout: Option<String>,
    all: bool,
    configured: bool,
}

impl SetBackgroundImageCommand {
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            match_spec: None,
            layout: None,
            all: false,
            configured: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn layout(mut self, value: impl Into<String>) -> Self {
        self.layout = Some(value.into());
        self
    }

    pub fn all(mut self, value: bool) -> Self {
        self.all = value;
        self
    }

    pub fn configured(mut self, value: bool) -> Self {
        self.configured = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.data.is_empty() {
            return Err(CommandError::MissingParameter("data".to_string(), "set-background-image".to_string()));
        }

        payload.insert("data".to_string(), serde_json::Value::String(self.data));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(layout) = self.layout {
            payload.insert("layout".to_string(), serde_json::Value::String(layout));
        }

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        if self.configured {
            payload.insert("configured".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("set-background-image")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetColorsCommand {
    colors: Map<String, serde_json::Value>,
    match_window: Option<String>,
    match_tab: Option<String>,
    all: bool,
    configured: bool,
    reset: bool,
}

impl SetColorsCommand {
    pub fn new(colors: Map<String, serde_json::Value>) -> Self {
        Self {
            colors,
            match_window: None,
            match_tab: None,
            all: false,
            configured: false,
            reset: false,
        }
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

    pub fn configured(mut self, value: bool) -> Self {
        self.configured = value;
        self
    }

    pub fn reset(mut self, value: bool) -> Self {
        self.reset = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.colors.is_empty() {
            return Err(CommandError::MissingParameter("colors".to_string(), "set-colors".to_string()));
        }

        payload.insert("colors".to_string(), serde_json::Value::Object(self.colors));

        if let Some(match_window) = self.match_window {
            payload.insert("match_window".to_string(), serde_json::Value::String(match_window));
        }

        if let Some(match_tab) = self.match_tab {
            payload.insert("match_tab".to_string(), serde_json::Value::String(match_tab));
        }

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        if self.configured {
            payload.insert("configured".to_string(), serde_json::Value::Bool(true));
        }

        if self.reset {
            payload.insert("reset".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("set-colors")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetFontSizeCommand {
    size: i32,
    all: bool,
    increment_op: Option<String>,
}

impl SetFontSizeCommand {
    pub fn new(size: i32) -> Self {
        Self {
            size,
            all: false,
            increment_op: None,
        }
    }

    pub fn all(mut self, value: bool) -> Self {
        self.all = value;
        self
    }

    pub fn increment_op(mut self, value: impl Into<String>) -> Self {
        self.increment_op = Some(value.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        payload.insert("size".to_string(), serde_json::json!(self.size));

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(increment_op) = self.increment_op {
            payload.insert("increment_op".to_string(), serde_json::Value::String(increment_op));
        }

        Ok(CommandBuilder::new("set-font-size")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetSpacingCommand {
    settings: Map<String, serde_json::Value>,
    match_window: Option<String>,
    match_tab: Option<String>,
    all: bool,
    configured: bool,
}

impl SetSpacingCommand {
    pub fn new(settings: Map<String, serde_json::Value>) -> Self {
        Self {
            settings,
            match_window: None,
            match_tab: None,
            all: false,
            configured: false,
        }
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

    pub fn configured(mut self, value: bool) -> Self {
        self.configured = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.settings.is_empty() {
            return Err(CommandError::MissingParameter("settings".to_string(), "set-spacing".to_string()));
        }

        payload.insert("settings".to_string(), serde_json::Value::Object(self.settings));

        if let Some(match_window) = self.match_window {
            payload.insert("match_window".to_string(), serde_json::Value::String(match_window));
        }

        if let Some(match_tab) = self.match_tab {
            payload.insert("match_tab".to_string(), serde_json::Value::String(match_tab));
        }

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        if self.configured {
            payload.insert("configured".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("set-spacing")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetTabColorCommand {
    colors: Map<String, serde_json::Value>,
    match_spec: Option<String>,
    self_tab: bool,
}

impl SetTabColorCommand {
    pub fn new(colors: Map<String, serde_json::Value>) -> Self {
        Self {
            colors,
            match_spec: None,
            self_tab: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn self_tab(mut self, value: bool) -> Self {
        self.self_tab = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if self.colors.is_empty() {
            return Err(CommandError::MissingParameter("colors".to_string(), "set-tab-color".to_string()));
        }

        payload.insert("colors".to_string(), serde_json::Value::Object(self.colors));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.self_tab {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("set-tab-color")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct GetColorsCommand {
    match_spec: Option<String>,
    configured: bool,
}

impl GetColorsCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            configured: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn configured(mut self, value: bool) -> Self {
        self.configured = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.configured {
            payload.insert("configured".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("get-colors")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_background_opacity_basic() {
        let cmd = SetBackgroundOpacityCommand::new(0.5).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-background-opacity");
    }

    #[test]
    fn test_set_background_opacity_invalid() {
        let cmd = SetBackgroundOpacityCommand::new(1.5).build();
        assert!(cmd.is_err());
        if let Err(CommandError::ValidationError(msg)) = cmd {
            assert!(msg.contains("opacity"));
        } else {
            panic!("Expected ValidationError error");
        }
    }

    #[test]
    fn test_set_background_opacity_with_options() {
        let cmd = SetBackgroundOpacityCommand::new(0.8)
            .all(true)
            .toggle(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-background-opacity");
    }

    #[test]
    fn test_set_background_image_basic() {
        let cmd = SetBackgroundImageCommand::new("base64data").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-background-image");
    }

    #[test]
    fn test_set_background_image_empty() {
        let cmd = SetBackgroundImageCommand::new("").build();
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
        let cmd = SetBackgroundImageCommand::new("base64data")
            .layout("tiled")
            .all(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-background-image");
    }

    #[test]
    fn test_set_colors_basic() {
        let mut colors = Map::new();
        colors.insert("foreground".to_string(), serde_json::Value::String("#ffffff".to_string()));
        let cmd = SetColorsCommand::new(colors).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-colors");
    }

    #[test]
    fn test_set_colors_empty() {
        let cmd = SetColorsCommand::new(Map::new()).build();
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
        colors.insert("background".to_string(), serde_json::Value::String("#000000".to_string()));
        let cmd = SetColorsCommand::new(colors)
            .all(true)
            .reset(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-colors");
    }

    #[test]
    fn test_set_font_size_basic() {
        let cmd = SetFontSizeCommand::new(14).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-font-size");
    }

    #[test]
    fn test_set_font_size_with_options() {
        let cmd = SetFontSizeCommand::new(16)
            .all(true)
            .increment_op("set")
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-font-size");
    }

    #[test]
    fn test_set_spacing_basic() {
        let mut settings = Map::new();
        settings.insert("padding".to_string(), serde_json::json!(10));
        let cmd = SetSpacingCommand::new(settings).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-spacing");
    }

    #[test]
    fn test_set_spacing_empty() {
        let cmd = SetSpacingCommand::new(Map::new()).build();
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
        let cmd = SetSpacingCommand::new(settings)
            .all(true)
            .configured(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-spacing");
    }

    #[test]
    fn test_set_tab_color_basic() {
        let mut colors = Map::new();
        colors.insert("active_tab_foreground".to_string(), serde_json::Value::String("#ffffff".to_string()));
        let cmd = SetTabColorCommand::new(colors).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-tab-color");
    }

    #[test]
    fn test_set_tab_color_empty() {
        let cmd = SetTabColorCommand::new(Map::new()).build();
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
        colors.insert("active_tab_background".to_string(), serde_json::Value::String("#000000".to_string()));
        let cmd = SetTabColorCommand::new(colors)
            .self_tab(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-tab-color");
    }

    #[test]
    fn test_get_colors_basic() {
        let cmd = GetColorsCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "get-colors");
    }

    #[test]
    fn test_get_colors_with_options() {
        let cmd = GetColorsCommand::new()
            .match_spec("id:1")
            .configured(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "get-colors");
    }
}

use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;

pub struct GotoLayoutCommand {
    layout: String,
    match_spec: Option<String>,
}

impl GotoLayoutCommand {
    pub fn new(layout: impl Into<String>) -> Self {
        Self {
            layout: layout.into(),
            match_spec: None,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if self.layout.is_empty() {
            return Err(CommandError::MissingParameter("layout".to_string(), "goto-layout".to_string()));
        }

        payload.insert("layout".to_string(), serde_json::Value::String(self.layout));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        Ok(CommandBuilder::new("goto-layout")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetEnabledLayoutsCommand {
    layouts: Vec<String>,
    match_spec: Option<String>,
    configured: bool,
}

impl SetEnabledLayoutsCommand {
    pub fn new(layouts: Vec<String>) -> Self {
        Self {
            layouts,
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
        let mut payload = serde_json::Map::new();

        if self.layouts.is_empty() {
            return Err(CommandError::MissingParameter(
                "layouts".to_string(),
                "set-enabled-layouts".to_string(),
            ));
        }

        let layouts_value: Vec<serde_json::Value> = self.layouts
            .into_iter()
            .map(serde_json::Value::String)
            .collect();

        payload.insert("layouts".to_string(), serde_json::Value::Array(layouts_value));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.configured {
            payload.insert("configured".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("set-enabled-layouts")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct LastUsedLayoutCommand {
    match_spec: Option<String>,
    all: bool,
}

impl LastUsedLayoutCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            all: false,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn all(mut self, value: bool) -> Self {
        self.all = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.all {
            payload.insert("all".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("last-used-layout")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto_layout() {
        let cmd = GotoLayoutCommand::new("tall").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "goto-layout");
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_goto_layout_empty() {
        let cmd = GotoLayoutCommand::new("").build();
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
        let cmd = GotoLayoutCommand::new("grid").match_spec("id:0").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "goto-layout");
    }

    #[test]
    fn test_set_enabled_layouts() {
        let layouts = vec!["tall".to_string(), "grid".to_string()];
        let cmd = SetEnabledLayoutsCommand::new(layouts).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-enabled-layouts");
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_set_enabled_layouts_empty() {
        let cmd = SetEnabledLayoutsCommand::new(vec![]).build();
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
        let cmd = SetEnabledLayoutsCommand::new(layouts).match_spec("id:1").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-enabled-layouts");
    }

    #[test]
    fn test_set_enabled_layouts_configured() {
        let layouts = vec!["tall".to_string()];
        let cmd = SetEnabledLayoutsCommand::new(layouts).configured(true).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-enabled-layouts");
    }

    #[test]
    fn test_last_used_layout() {
        let cmd = LastUsedLayoutCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "last-used-layout");
    }

    #[test]
    fn test_last_used_layout_with_match() {
        let cmd = LastUsedLayoutCommand::new().match_spec("id:0").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "last-used-layout");
    }

    #[test]
    fn test_last_used_layout_all() {
        let cmd = LastUsedLayoutCommand::new().all(true).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "last-used-layout");
    }
}

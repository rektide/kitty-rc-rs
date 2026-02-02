use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;

pub struct FocusTabCommand {
    match_spec: Option<String>,
}

impl FocusTabCommand {
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

        Ok(CommandBuilder::new("focus-tab")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct SetTabTitleCommand {
    title: String,
    match_spec: Option<String>,
}

impl SetTabTitleCommand {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            match_spec: None,
        }
    }

    pub fn match_spec(mut self, spec: impl Into<String>) -> Self {
        self.match_spec = Some(spec.into());
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if self.title.is_empty() {
            return Err(CommandError::MissingParameter(
                "title".to_string(),
                "set-tab-title".to_string(),
            ));
        }

        payload.insert("title".to_string(), serde_json::Value::String(self.title));

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        Ok(CommandBuilder::new("set-tab-title")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct CloseTabCommand {
    match_spec: Option<String>,
    self_tab: bool,
    ignore_no_match: bool,
}

impl CloseTabCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            self_tab: false,
            ignore_no_match: false,
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

    pub fn ignore_no_match(mut self, value: bool) -> Self {
        self.ignore_no_match = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if self.self_tab {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        if self.ignore_no_match {
            payload.insert("ignore_no_match".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("close-tab")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

pub struct DetachTabCommand {
    match_spec: Option<String>,
    target_tab: Option<String>,
    self_tab: bool,
}

impl DetachTabCommand {
    pub fn new() -> Self {
        Self {
            match_spec: None,
            target_tab: None,
            self_tab: false,
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

    pub fn self_tab(mut self, value: bool) -> Self {
        self.self_tab = value;
        self
    }

    pub fn build(self) -> Result<KittyMessage, CommandError> {
        let mut payload = serde_json::Map::new();

        if let Some(match_spec) = self.match_spec {
            payload.insert("match".to_string(), serde_json::Value::String(match_spec));
        }

        if let Some(target_tab) = self.target_tab {
            payload.insert(
                "target_tab".to_string(),
                serde_json::Value::String(target_tab),
            );
        }

        if self.self_tab {
            payload.insert("self".to_string(), serde_json::Value::Bool(true));
        }

        Ok(CommandBuilder::new("detach-tab")
            .payload(serde_json::Value::Object(payload))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_tab_basic() {
        let cmd = FocusTabCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "focus-tab");
    }

    #[test]
    fn test_focus_tab_with_match() {
        let cmd = FocusTabCommand::new().match_spec("id:0").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "focus-tab");
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_set_tab_title() {
        let cmd = SetTabTitleCommand::new("My Tab").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-tab-title");
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_set_tab_title_empty() {
        let cmd = SetTabTitleCommand::new("").build();
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
        let cmd = SetTabTitleCommand::new("New Title")
            .match_spec("id:1")
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "set-tab-title");
    }

    #[test]
    fn test_close_tab_basic() {
        let cmd = CloseTabCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-tab");
    }

    #[test]
    fn test_close_tab_with_match() {
        let cmd = CloseTabCommand::new().match_spec("id:2").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-tab");
    }

    #[test]
    fn test_close_tab_self() {
        let cmd = CloseTabCommand::new().self_tab(true).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-tab");
    }

    #[test]
    fn test_close_tab_ignore_no_match() {
        let cmd = CloseTabCommand::new().ignore_no_match(true).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "close-tab");
    }

    #[test]
    fn test_detach_tab_basic() {
        let cmd = DetachTabCommand::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }

    #[test]
    fn test_detach_tab_with_match() {
        let cmd = DetachTabCommand::new().match_spec("id:0").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }

    #[test]
    fn test_detach_tab_with_target_tab() {
        let cmd = DetachTabCommand::new().target_tab("id:1").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }

    #[test]
    fn test_detach_tab_self() {
        let cmd = DetachTabCommand::new().self_tab(true).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }

    #[test]
    fn test_detach_tab_all_options() {
        let cmd = DetachTabCommand::new()
            .match_spec("id:0")
            .target_tab("id:1")
            .self_tab(true)
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "detach-tab");
    }
}

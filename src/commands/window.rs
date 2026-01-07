use crate::command::CommandBuilder;
use crate::error::CommandError;
use crate::protocol::KittyMessage;

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
}

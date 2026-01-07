use crate::protocol::KittyMessage;

pub struct CommandBuilder {
    cmd: String,
    version: Vec<u32>,
    no_response: Option<bool>,
    kitty_window_id: Option<String>,
    payload: Option<serde_json::Value>,
}

impl CommandBuilder {
    pub fn new(cmd: impl Into<String>) -> Self {
        Self {
            cmd: cmd.into(),
            version: vec![0, 43, 1],
            no_response: None,
            kitty_window_id: None,
            payload: None,
        }
    }

    pub fn version(mut self, version: Vec<u32>) -> Self {
        self.version = version;
        self
    }

    pub fn no_response(mut self, value: bool) -> Self {
        self.no_response = Some(value);
        self
    }

    pub fn kitty_window_id(mut self, id: impl Into<String>) -> Self {
        self.kitty_window_id = Some(id.into());
        self
    }

    pub fn payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn build(self) -> KittyMessage {
        let mut msg = KittyMessage::new(self.cmd, self.version);

        if let Some(no_response) = self.no_response {
            msg = msg.no_response(no_response);
        }

        if let Some(window_id) = self.kitty_window_id {
            msg = msg.kitty_window_id(window_id);
        }

        if let Some(payload) = self.payload {
            msg = msg.payload(payload);
        }

        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_builder_basic() {
        let cmd = CommandBuilder::new("ls").build();
        assert_eq!(cmd.cmd, "ls");
        assert_eq!(cmd.version, vec![0, 43, 1]);
    }

    #[test]
    fn test_builder_with_options() {
        let cmd = CommandBuilder::new("send-text")
            .version(vec![0, 15, 0])
            .no_response(true)
            .payload(json!({"data": "text:hello"}))
            .build();

        assert_eq!(cmd.cmd, "send-text");
        assert_eq!(cmd.version, vec![0, 15, 0]);
        assert_eq!(cmd.no_response, Some(true));
        assert!(cmd.payload.is_some());
    }

    #[test]
    fn test_builder_to_encoded() {
        let cmd = CommandBuilder::new("ls").build();
        let encoded = cmd.encode().unwrap();
        assert!(encoded.starts_with(b"\x1bP@kitty-cmd"));
        assert!(encoded.ends_with(b"\x1b\\"));
    }
}

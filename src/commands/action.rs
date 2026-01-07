use crate::protocol::KittyMessage;

pub struct ActionCommand {
    action: String,
    args: Vec<String>,
}

impl ActionCommand {
    pub fn new(action: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for arg in args {
            self.args.push(arg.into());
        }
        self
    }

    pub fn build(self) -> Result<KittyMessage, crate::error::CommandError> {
        let mut payload = serde_json::Map::new();
        payload.insert("action".to_string(), serde_json::Value::String(self.action));

        if !self.args.is_empty() {
            payload.insert("args".to_string(), serde_json::Value::Array(
                self.args.into_iter().map(|a| serde_json::Value::String(a)).collect()
            ));
        }

        Ok(KittyMessage::new("send_key", vec![0, 14, 2])
            .payload(serde_json::Value::Object(payload)))
    }
}

// Session actions

pub struct QuitAction;

impl QuitAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("quit")
    }
}

// Tab actions

pub struct NewTabAction;

impl NewTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("new_tab")
    }
}

pub struct CloseTabAction;

impl CloseTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("close_tab")
    }
}

pub struct NextTabAction;

impl NextTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("next_tab")
    }
}

pub struct PreviousTabAction;

impl PreviousTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("previous_tab")
    }
}

pub struct GotoTabAction;

impl GotoTabAction {
    pub fn new(tab_num: i32) -> ActionCommand {
        ActionCommand::new("goto_tab").arg(tab_num.to_string())
    }
}

pub struct SetTabTitleAction;

impl SetTabTitleAction {
    pub fn new(title: impl Into<String>) -> ActionCommand {
        ActionCommand::new("set_tab_title").arg(title.into())
    }
}

pub struct DetachTabAction;

impl DetachTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("detach_tab")
    }
}

pub struct MoveTabForwardAction;

impl MoveTabForwardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("move_tab_forward")
    }
}

pub struct MoveTabBackwardAction;

impl MoveTabBackwardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("move_tab_backward")
    }
}

// Window actions

pub struct NewWindowAction;

impl NewWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("new_window")
    }
}

pub struct NewWindowWithCwdAction;

impl NewWindowWithCwdAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("new_window_with_cwd")
    }
}

pub struct CloseWindowAction;

impl CloseWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("close_window")
    }
}

pub struct CloseWindowWithConfirmationAction;

impl CloseWindowWithConfirmationAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("close_window_with_confirmation")
    }
}

pub struct NextWindowAction;

impl NextWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("next_window")
    }
}

pub struct PreviousWindowAction;

impl PreviousWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("previous_window")
    }
}

pub struct MoveWindowForwardAction;

impl MoveWindowForwardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("move_window_forward")
    }
}

pub struct MoveWindowBackwardAction;

impl MoveWindowBackwardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("move_window_backward")
    }
}

pub struct NthWindowAction;

impl NthWindowAction {
    pub fn new(n: i32) -> ActionCommand {
        ActionCommand::new("nth_window").arg(n.to_string())
    }
}

pub struct FirstWindowAction;

impl FirstWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("first_window")
    }
}

pub struct SetWindowTitleAction;

impl SetWindowTitleAction {
    pub fn new(title: impl Into<String>) -> ActionCommand {
        ActionCommand::new("set_window_title").arg(title.into())
    }
}

pub struct ResizeWindowAction;

impl ResizeWindowAction {
    pub fn new(shrink: bool) -> ActionCommand {
        if shrink {
            ActionCommand::new("resize_window").arg("shrink")
        } else {
            ActionCommand::new("resize_window").arg("taller")
        }
    }
}

pub struct ResetWindowSizesAction;

impl ResetWindowSizesAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("reset_window_sizes")
    }
}

pub struct MoveWindowAction;

impl MoveWindowAction {
    pub fn new(direction: impl Into<String>) -> ActionCommand {
        ActionCommand::new("move_window").arg(direction.into())
    }
}

pub struct NeighboringWindowAction;

impl NeighboringWindowAction {
    pub fn new(direction: impl Into<String>) -> ActionCommand {
        ActionCommand::new("neighboring_window").arg(direction.into())
    }
}

pub struct ToggleFullscreenAction;

impl ToggleFullscreenAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("toggle_fullscreen")
    }
}

pub struct ToggleMaximizedAction;

impl ToggleMaximizedAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("toggle_maximized")
    }
}

// Clipboard actions

pub struct CopyToClipboardAction;

impl CopyToClipboardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("copy_to_clipboard")
    }
}

pub struct PasteAction;

impl PasteAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("paste")
    }
}

pub struct PasteFromClipboardAction;

impl PasteFromClipboardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("paste_from_clipboard")
    }
}

pub struct PasteSelectionAction;

impl PasteSelectionAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("paste_selection")
    }
}

pub struct ClearSelectionAction;

impl ClearSelectionAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("clear_selection")
    }
}

pub struct CopyOrInterruptAction;

impl CopyOrInterruptAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("copy_or_interrupt")
    }
}

// Layout actions

pub struct GotoLayoutAction;

impl GotoLayoutAction {
    pub fn new(layout: impl Into<String>) -> ActionCommand {
        ActionCommand::new("goto_layout").arg(layout.into())
    }
}

pub struct NextLayoutAction;

impl NextLayoutAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("next_layout")
    }
}

pub struct LastUsedLayoutAction;

impl LastUsedLayoutAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("last_used_layout")
    }
}

pub struct ToggleLayoutAction;

impl ToggleLayoutAction {
    pub fn new(layout: impl Into<String>) -> ActionCommand {
        ActionCommand::new("toggle_layout").arg(layout.into())
    }
}

// Scroll actions

pub struct ScrollLineUpAction;

impl ScrollLineUpAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("scroll_line_up")
    }
}

pub struct ScrollLineDownAction;

impl ScrollLineDownAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("scroll_line_down")
    }
}

pub struct ScrollPageUpAction;

impl ScrollPageUpAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("scroll_page_up")
    }
}

pub struct ScrollPageDownAction;

impl ScrollPageDownAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("scroll_page_down")
    }
}

pub struct ScrollHomeAction;

impl ScrollHomeAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("scroll_home")
    }
}

pub struct ScrollEndAction;

impl ScrollEndAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("scroll_end")
    }
}

pub struct ScrollToPromptAction;

impl ScrollToPromptAction {
    pub fn new(direction: i32) -> ActionCommand {
        ActionCommand::new("scroll_to_prompt").arg(direction.to_string())
    }
}

pub struct ShowScrollbackAction;

impl ShowScrollbackAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("show_scrollback")
    }
}

// Mark actions

pub struct CreateMarkerAction;

impl CreateMarkerAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("create_marker")
    }
}

pub struct RemoveMarkerAction;

impl RemoveMarkerAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("remove_marker")
    }
}

pub struct ScrollToMarkAction;

impl ScrollToMarkAction {
    pub fn new(marker_type: impl Into<String>) -> ActionCommand {
        ActionCommand::new("scroll_to_mark").arg(marker_type.into())
    }
}

pub struct ToggleMarkerAction;

impl ToggleMarkerAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("toggle_marker")
    }
}

// Mouse actions

pub struct MouseClickUrlAction;

impl MouseClickUrlAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("mouse_click_url")
    }
}

pub struct MouseSelectionAction;

impl MouseSelectionAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("mouse_selection")
    }
}

// Debug actions

pub struct DebugConfigAction;

impl DebugConfigAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("debug_config")
    }
}

pub struct DumpLinesWithAttrsAction;

impl DumpLinesWithAttrsAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("dump_lines_with_attrs")
    }
}

// Misc actions

pub struct SendKeyAction;

impl SendKeyAction {
    pub fn new(keys: impl Into<String>) -> ActionCommand {
        ActionCommand::new("send_key").arg(keys.into())
    }
}

pub struct SendTextAction;

impl SendTextAction {
    pub fn new(text: impl Into<String>) -> ActionCommand {
        ActionCommand::new("send_text").arg(text.into())
    }
}

pub struct KittenAction;

impl KittenAction {
    pub fn new(kitten_name: impl Into<String>) -> ActionCommand {
        ActionCommand::new("kitten").arg(kitten_name.into())
    }
}

pub struct LaunchAction;

impl LaunchAction {
    pub fn new(args: impl Into<String>) -> ActionCommand {
        ActionCommand::new("launch").arg(args.into())
    }
}

pub struct SignalChildAction;

impl SignalChildAction {
    pub fn new(signal: impl Into<String>) -> ActionCommand {
        ActionCommand::new("signal_child").arg(signal.into())
    }
}

pub struct ClearTerminalAction;

impl ClearTerminalAction {
    pub fn new(mode: impl Into<String>) -> ActionCommand {
        ActionCommand::new("clear_terminal").arg(mode.into())
    }
}

pub struct ShowKittyDocAction;

impl ShowKittyDocAction {
    pub fn new(topic: impl Into<String>) -> ActionCommand {
        ActionCommand::new("show_kitty_doc").arg(topic.into())
    }
}

pub struct EditConfigFileAction;

impl EditConfigFileAction {
    pub fn new() -> ActionCommand {
        ActionCommand::new("edit_config_file")
    }
}

pub struct SetBackgroundOpacityAction;

impl SetBackgroundOpacityAction {
    pub fn new(opacity: f32) -> ActionCommand {
        ActionCommand::new("set_background_opacity").arg(opacity.to_string())
    }
}

pub struct ChangeFontSizeAction;

impl ChangeFontSizeAction {
    pub fn new(delta: impl Into<String>) -> ActionCommand {
        ActionCommand::new("change_font_size").arg(delta.into())
    }
}

pub struct LoadConfigFileAction;

impl LoadConfigFileAction {
    pub fn new(path: impl Into<String>) -> ActionCommand {
        ActionCommand::new("load_config_file").arg(path.into())
    }
}

pub struct SetColorsAction;

impl SetColorsAction {
    pub fn new(path: impl Into<String>) -> ActionCommand {
        ActionCommand::new("set_colors").arg(path.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_command_basic() {
        let cmd = ActionCommand::new("quit").build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send_key");
    }

    #[test]
    fn test_action_command_with_args() {
        let cmd = ActionCommand::new("goto_tab")
            .arg("1")
            .build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_quit_action() {
        let cmd = QuitAction::new().build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send_key");
    }

    #[test]
    fn test_new_tab_action() {
        let cmd = NewTabAction::new().build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_goto_tab_action() {
        let cmd = GotoTabAction::new(5).build();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_new_window_action() {
        let cmd = NewWindowAction::new().build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_nth_window_action() {
        let cmd = NthWindowAction::new(3).build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_copy_to_clipboard_action() {
        let cmd = CopyToClipboardAction::new().build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_paste_action() {
        let cmd = PasteAction::new().build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_goto_layout_action() {
        let cmd = GotoLayoutAction::new("tall").build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_scroll_line_up_action() {
        let cmd = ScrollLineUpAction::new().build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_create_marker_action() {
        let cmd = CreateMarkerAction::new().build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_send_key_action() {
        let cmd = SendKeyAction::new("ctrl+c").build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_send_text_action() {
        let cmd = SendTextAction::new("hello").build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_kitten_action() {
        let cmd = KittenAction::new("hints").build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_signal_child_action() {
        let cmd = SignalChildAction::new("SIGTERM").build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_clear_terminal_action() {
        let cmd = ClearTerminalAction::new("reset").build();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_set_background_opacity_action() {
        let cmd = SetBackgroundOpacityAction::new(0.8).build();
        assert!(cmd.is_ok());
    }
}

use crate::protocol::KittyMessage;
use bon::Builder;
use serde::Serialize;

#[derive(Builder, Serialize)]
pub struct ActionCommand {
    action: String,

    #[builder(default = vec![])]
    #[serde(skip_serializing_if = "is_empty_args")]
    args: Vec<String>,
}

fn is_empty_args(v: &Vec<String>) -> bool {
    v.is_empty()
}

impl ActionCommand {
    pub fn to_message(self) -> Result<KittyMessage, crate::error::CommandError> {
        let payload = serde_json::to_value(self)
            .map_err(|e| crate::error::CommandError::ValidationError(e.to_string()))?;

        Ok(KittyMessage::new("send_key", vec![0, 14, 2]).payload(payload))
    }
}

// Session actions

pub struct QuitAction;

impl QuitAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder().action("quit".to_string()).build()
    }
}

// Tab actions

pub struct NewTabAction;

impl NewTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("new_tab".to_string())
            .build()
    }
}

pub struct CloseTabAction;

impl CloseTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("close_tab".to_string())
            .build()
    }
}

pub struct NextTabAction;

impl NextTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("next_tab".to_string())
            .build()
    }
}

pub struct PreviousTabAction;

impl PreviousTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("previous_tab".to_string())
            .build()
    }
}

pub struct GotoTabAction;

impl GotoTabAction {
    pub fn new(tab_num: i32) -> ActionCommand {
        ActionCommand::builder()
            .action("goto_tab".to_string())
            .args(vec![tab_num.to_string()])
            .build()
    }
}

pub struct SetTabTitleAction;

impl SetTabTitleAction {
    pub fn new(title: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("set_tab_title".to_string())
            .args(vec![title.into()])
            .build()
    }
}

pub struct DetachTabAction;

impl DetachTabAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("detach_tab".to_string())
            .build()
    }
}

pub struct MoveTabForwardAction;

impl MoveTabForwardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("move_tab_forward".to_string())
            .build()
    }
}

pub struct MoveTabBackwardAction;

impl MoveTabBackwardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("move_tab_backward".to_string())
            .build()
    }
}

// Window actions

pub struct NewWindowAction;

impl NewWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("new_window".to_string())
            .build()
    }
}

pub struct NewWindowWithCwdAction;

impl NewWindowWithCwdAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("new_window_with_cwd".to_string())
            .build()
    }
}

pub struct CloseWindowAction;

impl CloseWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("close_window".to_string())
            .build()
    }
}

pub struct CloseWindowWithConfirmationAction;

impl CloseWindowWithConfirmationAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("close_window_with_confirmation".to_string())
            .build()
    }
}

pub struct NextWindowAction;

impl NextWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("next_window".to_string())
            .build()
    }
}

pub struct PreviousWindowAction;

impl PreviousWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("previous_window".to_string())
            .build()
    }
}

pub struct MoveWindowForwardAction;

impl MoveWindowForwardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("move_window_forward".to_string())
            .build()
    }
}

pub struct MoveWindowBackwardAction;

impl MoveWindowBackwardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("move_window_backward".to_string())
            .build()
    }
}

pub struct NthWindowAction;

impl NthWindowAction {
    pub fn new(n: i32) -> ActionCommand {
        ActionCommand::builder()
            .action("nth_window".to_string())
            .args(vec![n.to_string()])
            .build()
    }
}

pub struct FirstWindowAction;

impl FirstWindowAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("first_window".to_string())
            .build()
    }
}

pub struct SetWindowTitleAction;

impl SetWindowTitleAction {
    pub fn new(title: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("set_window_title".to_string())
            .args(vec![title.into()])
            .build()
    }
}

pub struct ResizeWindowAction;

impl ResizeWindowAction {
    pub fn new(shrink: bool) -> ActionCommand {
        if shrink {
            ActionCommand::builder()
                .action("resize_window".to_string())
                .args(vec!["shrink".to_string()])
                .build()
        } else {
            ActionCommand::builder()
                .action("resize_window".to_string())
                .args(vec!["taller".to_string()])
                .build()
        }
    }
}

pub struct ResetWindowSizesAction;

impl ResetWindowSizesAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("reset_window_sizes".to_string())
            .build()
    }
}

pub struct MoveWindowAction;

impl MoveWindowAction {
    pub fn new(direction: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("move_window".to_string())
            .args(vec![direction.into()])
            .build()
    }
}

pub struct NeighboringWindowAction;

impl NeighboringWindowAction {
    pub fn new(direction: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("neighboring_window".to_string())
            .args(vec![direction.into()])
            .build()
    }
}

pub struct ToggleFullscreenAction;

impl ToggleFullscreenAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("toggle_fullscreen".to_string())
            .build()
    }
}

pub struct ToggleMaximizedAction;

impl ToggleMaximizedAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("toggle_maximized".to_string())
            .build()
    }
}

// Clipboard actions

pub struct CopyToClipboardAction;

impl CopyToClipboardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("copy_to_clipboard".to_string())
            .build()
    }
}

pub struct PasteAction;

impl PasteAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder().action("paste".to_string()).build()
    }
}

pub struct PasteFromClipboardAction;

impl PasteFromClipboardAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("paste_from_clipboard".to_string())
            .build()
    }
}

pub struct PasteSelectionAction;

impl PasteSelectionAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("paste_selection".to_string())
            .build()
    }
}

pub struct ClearSelectionAction;

impl ClearSelectionAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("clear_selection".to_string())
            .build()
    }
}

pub struct CopyOrInterruptAction;

impl CopyOrInterruptAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("copy_or_interrupt".to_string())
            .build()
    }
}

// Layout actions

pub struct GotoLayoutAction;

impl GotoLayoutAction {
    pub fn new(layout: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("goto_layout".to_string())
            .args(vec![layout.into()])
            .build()
    }
}

pub struct NextLayoutAction;

impl NextLayoutAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("next_layout".to_string())
            .build()
    }
}

pub struct LastUsedLayoutAction;

impl LastUsedLayoutAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("last_used_layout".to_string())
            .build()
    }
}

pub struct ToggleLayoutAction;

impl ToggleLayoutAction {
    pub fn new(layout: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("toggle_layout".to_string())
            .args(vec![layout.into()])
            .build()
    }
}

// Scroll actions

pub struct ScrollLineUpAction;

impl ScrollLineUpAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("scroll_line_up".to_string())
            .build()
    }
}

pub struct ScrollLineDownAction;

impl ScrollLineDownAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("scroll_line_down".to_string())
            .build()
    }
}

pub struct ScrollPageUpAction;

impl ScrollPageUpAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("scroll_page_up".to_string())
            .build()
    }
}

pub struct ScrollPageDownAction;

impl ScrollPageDownAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("scroll_page_down".to_string())
            .build()
    }
}

pub struct ScrollHomeAction;

impl ScrollHomeAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("scroll_home".to_string())
            .build()
    }
}

pub struct ScrollEndAction;

impl ScrollEndAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("scroll_end".to_string())
            .build()
    }
}

pub struct ScrollToPromptAction;

impl ScrollToPromptAction {
    pub fn new(direction: i32) -> ActionCommand {
        ActionCommand::builder()
            .action("scroll_to_prompt".to_string())
            .args(vec![direction.to_string()])
            .build()
    }
}

pub struct ShowScrollbackAction;

impl ShowScrollbackAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("show_scrollback".to_string())
            .build()
    }
}

// Mark actions

pub struct CreateMarkerAction;

impl CreateMarkerAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("create_marker".to_string())
            .build()
    }
}

pub struct RemoveMarkerAction;

impl RemoveMarkerAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("remove_marker".to_string())
            .build()
    }
}

pub struct ScrollToMarkAction;

impl ScrollToMarkAction {
    pub fn new(marker_type: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("scroll_to_mark".to_string())
            .args(vec![marker_type.into()])
            .build()
    }
}

pub struct ToggleMarkerAction;

impl ToggleMarkerAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("toggle_marker".to_string())
            .build()
    }
}

// Mouse actions

pub struct MouseClickUrlAction;

impl MouseClickUrlAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("mouse_click_url".to_string())
            .build()
    }
}

pub struct MouseSelectionAction;

impl MouseSelectionAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("mouse_selection".to_string())
            .build()
    }
}

// Debug actions

pub struct DebugConfigAction;

impl DebugConfigAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("debug_config".to_string())
            .build()
    }
}

pub struct DumpLinesWithAttrsAction;

impl DumpLinesWithAttrsAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("dump_lines_with_attrs".to_string())
            .build()
    }
}

// Misc actions

pub struct SendKeyAction;

impl SendKeyAction {
    pub fn new(keys: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("send_key".to_string())
            .args(vec![keys.into()])
            .build()
    }
}

pub struct SendTextAction;

impl SendTextAction {
    pub fn new(text: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("send_text".to_string())
            .args(vec![text.into()])
            .build()
    }
}

pub struct KittenAction;

impl KittenAction {
    pub fn new(kitten_name: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("kitten".to_string())
            .args(vec![kitten_name.into()])
            .build()
    }
}

pub struct LaunchAction;

impl LaunchAction {
    pub fn new(args: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("launch".to_string())
            .args(vec![args.into()])
            .build()
    }
}

pub struct SignalChildAction;

impl SignalChildAction {
    pub fn new(signal: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("signal_child".to_string())
            .args(vec![signal.into()])
            .build()
    }
}

pub struct ClearTerminalAction;

impl ClearTerminalAction {
    pub fn new(mode: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("clear_terminal".to_string())
            .args(vec![mode.into()])
            .build()
    }
}

pub struct ShowKittyDocAction;

impl ShowKittyDocAction {
    pub fn new(topic: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("show_kitty_doc".to_string())
            .args(vec![topic.into()])
            .build()
    }
}

pub struct EditConfigFileAction;

impl EditConfigFileAction {
    pub fn new() -> ActionCommand {
        ActionCommand::builder()
            .action("edit_config_file".to_string())
            .build()
    }
}

pub struct SetBackgroundOpacityAction;

impl SetBackgroundOpacityAction {
    pub fn new(opacity: f32) -> ActionCommand {
        ActionCommand::builder()
            .action("set_background_opacity".to_string())
            .args(vec![opacity.to_string()])
            .build()
    }
}

pub struct ChangeFontSizeAction;

impl ChangeFontSizeAction {
    pub fn new(delta: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("change_font_size".to_string())
            .args(vec![delta.into()])
            .build()
    }
}

pub struct LoadConfigFileAction;

impl LoadConfigFileAction {
    pub fn new(path: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("load_config_file".to_string())
            .args(vec![path.into()])
            .build()
    }
}

pub struct SetColorsAction;

impl SetColorsAction {
    pub fn new(path: impl Into<String>) -> ActionCommand {
        ActionCommand::builder()
            .action("set_colors".to_string())
            .args(vec![path.into()])
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_command_basic() {
        let cmd = ActionCommand::builder()
            .action("quit".to_string())
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send_key");
    }

    #[test]
    fn test_action_command_with_args() {
        let cmd = ActionCommand::builder()
            .action("goto_tab".to_string())
            .args(vec!["1".to_string()])
            .build()
            .to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_quit_action() {
        let cmd = QuitAction::new().to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert_eq!(msg.cmd, "send_key");
    }

    #[test]
    fn test_new_tab_action() {
        let cmd = NewTabAction::new().to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_goto_tab_action() {
        let cmd = GotoTabAction::new(5).to_message();
        assert!(cmd.is_ok());
        let msg = cmd.unwrap();
        assert!(msg.payload.is_some());
    }

    #[test]
    fn test_new_window_action() {
        let cmd = NewWindowAction::new().to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_nth_window_action() {
        let cmd = NthWindowAction::new(3).to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_copy_to_clipboard_action() {
        let cmd = CopyToClipboardAction::new().to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_paste_action() {
        let cmd = PasteAction::new().to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_goto_layout_action() {
        let cmd = GotoLayoutAction::new("tall".to_string()).to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_scroll_line_up_action() {
        let cmd = ScrollLineUpAction::new().to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_create_marker_action() {
        let cmd = CreateMarkerAction::new().to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_send_key_action() {
        let cmd = SendKeyAction::new("ctrl+c".to_string()).to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_send_text_action() {
        let cmd = SendTextAction::new("hello".to_string()).to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_kitten_action() {
        let cmd = KittenAction::new("hints".to_string()).to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_signal_child_action() {
        let cmd = SignalChildAction::new("SIGTERM".to_string()).to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_clear_terminal_action() {
        let cmd = ClearTerminalAction::new("reset".to_string()).to_message();
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_set_background_opacity_action() {
        let cmd = SetBackgroundOpacityAction::new(0.8).to_message();
        assert!(cmd.is_ok());
    }
}

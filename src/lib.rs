pub mod client;
pub mod command;
pub mod commands;
pub mod error;
pub mod protocol;

pub use client::{Kitty, KittyBuilder};
pub use commands::{
    // Tab commands
    CloseTabCommand, DetachTabCommand, FocusTabCommand, SetTabTitleCommand,
    // Window commands
    CloseWindowCommand, CreateMarkerCommand, DetachWindowCommand, FocusWindowCommand,
    GetTextCommand, NewWindowCommand, RemoveMarkerCommand, ResizeWindowCommand,
    ScrollWindowCommand, SelectWindowCommand, SendKeyCommand, SendTextCommand,
    SetWindowLogoCommand, SetWindowTitleCommand,
    // Layout commands
    GotoLayoutCommand, LastUsedLayoutCommand, SetEnabledLayoutsCommand,
    // Style commands
    GetColorsCommand, SetBackgroundImageCommand, SetBackgroundOpacityCommand,
    SetColorsCommand, SetFontSizeCommand, SetSpacingCommand, SetTabColorCommand,
    // Process commands
    DisableLigaturesCommand, EnvCommand, KittenCommand, LaunchCommand,
    LoadConfigCommand, ResizeOSWindowCommand, RunCommand, SetUserVarsCommand,
    SignalChildCommand,
    // Special commands
    LsCommand,
    action::*,
    process::ProcessInfo,
    window::{OsInstance, TabInfo, WindowInfo, parse_response_data},
};
pub use error::{CommandError, ConnectionError, EncryptionError, KittyError, ProtocolError};
pub use protocol::{KittyMessage, KittyResponse};

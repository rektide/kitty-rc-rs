pub mod client;
pub mod command;
pub mod commands;
pub mod encryption;
pub mod error;
pub mod protocol;

pub use client::{Kitty, KittyBuilder};
pub use commands::{
    // Tab commands
    CloseTabCommand,
    // Window commands
    CloseWindowCommand,
    CreateMarkerCommand,
    DetachTabCommand,
    DetachWindowCommand,
    // Process commands
    DisableLigaturesCommand,
    EnvCommand,
    FocusTabCommand,
    FocusWindowCommand,
    // Style commands
    GetColorsCommand,
    GetTextCommand,
    // Layout commands
    GotoLayoutCommand,
    KittenCommand,
    LastUsedLayoutCommand,
    LaunchCommand,
    LoadConfigCommand,
    // Special commands
    LsCommand,
    NewWindowCommand,
    RemoveMarkerCommand,
    ResizeOSWindowCommand,
    ResizeWindowCommand,
    RunCommand,
    ScrollWindowCommand,
    SelectWindowCommand,
    SendKeyCommand,
    SendTextCommand,
    SetBackgroundImageCommand,
    SetBackgroundOpacityCommand,
    SetColorsCommand,
    SetEnabledLayoutsCommand,
    SetFontSizeCommand,
    SetSpacingCommand,
    SetTabColorCommand,
    SetTabTitleCommand,
    SetUserVarsCommand,
    SetWindowLogoCommand,
    SetWindowTitleCommand,
    SignalChildCommand,
    action::*,
    process::ProcessInfo,
    window::{OsInstance, TabInfo, WindowInfo, parse_response_data},
};
pub use error::{CommandError, ConnectionError, EncryptionError, KittyError, ProtocolError};
pub use protocol::{KittyMessage, KittyResponse};

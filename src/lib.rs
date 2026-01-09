pub mod command;
pub mod commands;
pub mod error;
pub mod protocol;
pub mod transport;

pub use command::CommandBuilder;
pub use commands::{
    CloseTabCommand, CloseWindowCommand, FocusTabCommand, GotoLayoutCommand,
    LastUsedLayoutCommand, LsCommand, ResizeWindowCommand, SendKeyCommand,
    SetEnabledLayoutsCommand, SetTabTitleCommand, SendTextCommand,
    action::*,
    process::ProcessInfo,
    window::{OsInstance, TabInfo, WindowInfo},
};
pub use error::{CommandError, ConnectionError, EncryptionError, KittyError, ProtocolError};
pub use protocol::{KittyMessage, KittyResponse};
pub use transport::{ConnectionPool, KittyClient};

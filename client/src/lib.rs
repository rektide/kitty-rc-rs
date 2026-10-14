//! Independently usable direct and managed entrances.
pub use kitty_rc_commands as commands;
pub use kitty_rc_credentials as credentials;
pub use kitty_rc_crypto::ServerPublicKey;
pub use kitty_rc_discovery as discovery;
pub use kitty_rc_endpoint::{AccessBinding, Address, Endpoint, Protection};
pub use kitty_rc_exchange::{Completion, Failure, Progress};
pub use kitty_rc_exchange::{ExecutionError, Phase, PrepareError};
pub use kitty_rc_registry::{
    AcquisitionError, BindingGeneration, CredentialGeneration, SetupError, SetupRejection,
    SetupTicket, SetupVersion,
};
pub use kitty_rc_registry::{
    CoordinationGroupId, LaneClaim, LaneError, LanePolicy, RefusedUpload, RejectedUpload,
    UploadOptions as RegistryUploadOptions,
};
pub use kitty_rc_registry::{
    CustodyError, Driver, EndpointId, Limits, Operation, Options, PoolStats, RefusalReason,
    Registry, RegistryError, Reservation, Shutdown, ShutdownError, Status, Token,
};
pub use kitty_rc_tokio::{ConnectionToken, Session, SessionOptions};
pub use kitty_rc_tokio::{UploadOptions as SessionUploadOptions, UploadSource};
pub fn parse_server_public_key(
    published: &str,
) -> Result<ServerPublicKey, kitty_rc_crypto::CryptoError> {
    ServerPublicKey::parse(published)
}

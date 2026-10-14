//! Explicit setup. Addresses are routes, not proven server identities.
use kitty_rc_crypto::ServerPublicKey;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Unix(PathBuf),
    Tcp(SocketAddr),
}

#[derive(Clone)]
pub enum Protection {
    Plain,
    Encrypted {
        key: ServerPublicKey,
        password: String,
        key_origin: String,
    },
}
impl std::fmt::Debug for Protection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain => f.write_str("Plain"),
            Self::Encrypted { key_origin, .. } => f
                .debug_struct("Encrypted")
                .field("key_origin", key_origin)
                .finish_non_exhaustive(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub address: Address,
    pub protection: Protection,
}

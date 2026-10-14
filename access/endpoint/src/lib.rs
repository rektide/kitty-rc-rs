//! Explicit setup. Addresses are routes, not proven server identities.
use kitty_rc_crypto::ServerPublicKey;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Unix(PathBuf),
    Tcp(SocketAddr),
}
impl std::str::FromStr for Address {
    type Err = String;
    /// Explicit `unix:/path` or `tcp:IP:PORT` (IPv6 uses brackets). No DNS,
    /// environment fallback, abstract-socket inference or identity claim.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(path) = value.strip_prefix("unix:") {
            if path.is_empty() {
                return Err("empty Unix socket path".into());
            }
            return Ok(Self::Unix(path.into()));
        }
        if let Some(address) = value.strip_prefix("tcp:") {
            return address
                .parse()
                .map(Self::Tcp)
                .map_err(|e| format!("invalid numeric TCP address: {e}"));
        }
        Err("explicit unix: or tcp: scheme required".into())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_address_parsing_never_falls_back() {
        assert_eq!(
            "unix:/chosen/socket".parse::<Address>().unwrap(),
            Address::Unix("/chosen/socket".into())
        );
        assert_eq!(
            "tcp:[::1]:1234".parse::<Address>().unwrap(),
            Address::Tcp("[::1]:1234".parse().unwrap())
        );
        for bad in ["", "unix:", "/ambient/socket", "tcp:hostname:1"] {
            assert!(bad.parse::<Address>().is_err());
        }
    }
}

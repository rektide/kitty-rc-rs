//! Explicit setup. Addresses are routes, not proven server identities.
use kitty_rc_crypto::ServerPublicKey;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Unix(PathBuf),
    Tcp(SocketAddr),
}
impl std::str::FromStr for Address {
    type Err = AddressError;
    /// Explicit `unix:/path` or `tcp:IP:PORT` (IPv6 uses brackets). No DNS,
    /// environment fallback, abstract-socket inference or identity claim.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_native(std::ffi::OsStr::new(value))
    }
}
impl Address {
    /// Parse selected native environment/config input without losing Unix path
    /// bytes. No expansion, DNS, PID suffix inference, or terminal fallback.
    pub fn parse_native(value: &std::ffi::OsStr) -> Result<Self, AddressError> {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        if let Some(path) = value.as_bytes().strip_prefix(b"unix:") {
            if path.is_empty() {
                return Err(AddressError::EmptyUnixPath);
            }
            if path.starts_with(b"@") {
                return Err(AddressError::Unsupported("abstract Unix socket"));
            }
            if path.contains(&0) {
                return Err(AddressError::NulPath);
            }
            return Ok(Self::Unix(
                std::ffi::OsString::from_vec(path.to_vec()).into(),
            ));
        }
        let text = value.to_str().ok_or(AddressError::NonUnicodeAddress)?;
        if let Some(address) = text
            .strip_prefix("tcp:")
            .or_else(|| text.strip_prefix("tcp6:"))
        {
            return address.parse().map(Self::Tcp).map_err(AddressError::Tcp);
        }
        if text.starts_with("fd:") {
            return Err(AddressError::Unsupported("inherited remote-control fd"));
        }
        Err(AddressError::Scheme)
    }
}
#[derive(Debug, thiserror::Error)]
pub enum AddressError {
    #[error("empty Unix socket path")]
    EmptyUnixPath,
    #[error("explicit unix: or tcp: scheme required")]
    Scheme,
    #[error("invalid numeric TCP address: {0}")]
    Tcp(#[source] std::net::AddrParseError),
    #[error("non-Unicode address outside native unix: path")]
    NonUnicodeAddress,
    #[error("Unix socket path contains a NUL byte")]
    NulPath,
    #[error("unsupported address route: {0}")]
    Unsupported(&'static str),
}

/// Caller-selected route and optional PID lookup hint. Neither is proof of a
/// server incarnation; aliases and coordination groups belong to caller policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessBinding {
    pub address: Address,
    pub pid_hint: Option<u32>,
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

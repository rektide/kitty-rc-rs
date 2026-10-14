use serde::{Deserialize, Serialize};
use std::fmt;

/// Kitty protocol version as sent on the wire: a three-component array such as
/// `[0, 43, 1]`.
///
/// The RC dispatcher compares only the first two components when rejecting
/// newer clients; this type keeps all three so exact versions can also be
/// cited in evidence and fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion(#[serde(deserialize_with = "deserialize_array3")] [u32; 3]);

/// The kitty version this workspace declares it develops against. Kept at the
/// original project target; raising it is a support-policy decision that
/// requires conformance evidence, not an automatic bump.
pub const SUPPORTED: ProtocolVersion = ProtocolVersion([0, 43, 1]);

impl ProtocolVersion {
    pub const fn new(components: [u32; 3]) -> Self {
        Self(components)
    }

    pub const fn components(&self) -> [u32; 3] {
        self.0
    }

    /// True when this version's major/minor pair is strictly newer than the
    /// server's, which is the condition under which kitty rejects a client.
    pub fn major_minor_newer_than(&self, server: &Self) -> bool {
        self.0[..2] > server.0[..2]
    }
}

impl From<[u32; 3]> for ProtocolVersion {
    fn from(components: [u32; 3]) -> Self {
        Self(components)
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0[0], self.0[1], self.0[2])
    }
}

fn deserialize_array3<'de, D>(deserializer: D) -> Result<[u32; 3], D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = [u32; 3];

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("an array of three version components")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut components = [0u32; 3];
            for (index, component) in components.iter_mut().enumerate() {
                let value = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(index, &self))?;
                *component = value;
            }
            let extra = seq.next_element::<u32>()?;
            if extra.is_some() {
                return Err(serde::de::Error::invalid_length(4, &self));
            }
            Ok(components)
        }
    }

    deserializer.deserialize_seq(Visitor)
}

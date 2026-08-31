use std::fmt;

use sha2::{Digest as _, Sha256};

use crate::IdentityError;

const MAX_SEMANTIC_ID_BYTES: usize = 512;

macro_rules! semantic_id {
    ($name:ident, $field:literal) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdentityError> {
                let value = value.into();
                validate_semantic_id($field, &value)?;
                Ok(Self(value.into_boxed_str()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

semantic_id!(RepositoryId, "repository identity");
semantic_id!(ProfileId, "profile identity");
semantic_id!(ProducerId, "producer identity");
semantic_id!(SchemaId, "schema identity");
semantic_id!(ToolId, "tool identity");

fn validate_semantic_id(field: &'static str, value: &str) -> Result<(), IdentityError> {
    if value.is_empty() {
        return Err(IdentityError::new(field, "must not be empty"));
    }
    if value.len() > MAX_SEMANTIC_ID_BYTES {
        return Err(IdentityError::new(
            field,
            format!("must not exceed {MAX_SEMANTIC_ID_BYTES} bytes"),
        ));
    }
    if !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(IdentityError::new(
            field,
            "must contain only printable non-whitespace ASCII",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ObjectAlgorithm {
    Sha1,
    Sha256,
}

impl ObjectAlgorithm {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
        }
    }

    pub const fn digest_bytes(self) -> usize {
        match self {
            Self::Sha1 => 20,
            Self::Sha256 => 32,
        }
    }

    pub const fn hex_digits(self) -> usize {
        self.digest_bytes() * 2
    }
}

impl fmt::Display for ObjectAlgorithm {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum ObjectId {
    Sha1([u8; 20]),
    Sha256([u8; 32]),
}

impl ObjectId {
    pub fn from_hex(value: &str) -> Result<Self, IdentityError> {
        match value.len() {
            40 => decode_hex::<20>(value).map(Self::Sha1),
            64 => decode_hex::<32>(value).map(Self::Sha256),
            length => Err(IdentityError::new(
                "object identity",
                format!("expected 40 or 64 hexadecimal digits, got {length}"),
            )),
        }
    }

    pub const fn algorithm(self) -> ObjectAlgorithm {
        match self {
            Self::Sha1(_) => ObjectAlgorithm::Sha1,
            Self::Sha256(_) => ObjectAlgorithm::Sha256,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Sha1(bytes) => bytes,
            Self::Sha256(bytes) => bytes,
        }
    }

    pub fn to_hex(self) -> String {
        hex_lower(self.as_bytes())
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.algorithm(), self.to_hex())
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

macro_rules! object_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(ObjectId);

        impl $name {
            pub fn from_hex(value: &str) -> Result<Self, IdentityError> {
                ObjectId::from_hex(value).map(Self)
            }

            pub const fn from_object_id(value: ObjectId) -> Self {
                Self(value)
            }

            pub const fn object_id(self) -> ObjectId {
                self.0
            }

            pub const fn algorithm(self) -> ObjectAlgorithm {
                self.0.algorithm()
            }

            pub fn to_hex(self) -> String {
                self.0.to_hex()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

object_id!(RevisionId);
object_id!(TreeId);
object_id!(ContentId);

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct EvidenceDigest([u8; 32]);

impl EvidenceDigest {
    pub fn of_bytes(domain: &[u8], bytes: &[u8]) -> Self {
        let mut builder = DigestBuilder::new(domain);
        builder.push_bytes(bytes);
        builder.finish()
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        hex_lower(&self.0)
    }
}

impl fmt::Debug for EvidenceDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "sha256:{}", self.to_hex())
    }
}

impl fmt::Display for EvidenceDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "sha256:{}", self.to_hex())
    }
}

pub struct DigestBuilder(Sha256);

impl DigestBuilder {
    pub fn new(domain: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"oyatie\0");
        hasher.update((domain.len() as u64).to_be_bytes());
        hasher.update(domain);
        Self(hasher)
    }

    pub fn push_bytes(&mut self, value: &[u8]) {
        self.0.update((value.len() as u64).to_be_bytes());
        self.0.update(value);
    }

    pub fn push_u64(&mut self, value: u64) {
        self.0.update(value.to_be_bytes());
    }

    pub fn finish(self) -> EvidenceDigest {
        EvidenceDigest(self.0.finalize().into())
    }
}

fn decode_hex<const N: usize>(value: &str) -> Result<[u8; N], IdentityError> {
    let mut output = [0; N];
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    debug_assert!(remainder.is_empty());
    for (index, pair) in pairs.iter().enumerate() {
        let high = hex_nibble(pair[0]).ok_or_else(|| {
            IdentityError::new("object identity", "contains a non-hexadecimal digit")
        })?;
        let low = hex_nibble(pair[1]).ok_or_else(|| {
            IdentityError::new("object identity", "contains a non-hexadecimal digit")
        })?;
        output[index] = high << 4 | low;
    }
    Ok(output)
}

const fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

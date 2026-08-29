use sha2::Digest as _;

/// A SHA-256 identity with an unambiguous display form.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DigestV1([u8; 32]);

impl DigestV1 {
    /// Hashes exact bytes.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        Self(sha2::Sha256::digest(bytes).into())
    }

    /// Creates an identity from already verified digest bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw digest bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl std::fmt::Display for DigestV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("sha256:")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

pub(crate) struct CanonicalHasherV1(sha2::Sha256);

impl CanonicalHasherV1 {
    pub(crate) fn new(domain: &[u8]) -> Self {
        let mut hash = sha2::Sha256::new();
        hash.update(domain);
        Self(hash)
    }

    pub(crate) fn tag(&mut self, value: u8) {
        self.0.update([value]);
    }

    pub(crate) fn boolean(&mut self, value: bool) {
        self.tag(u8::from(value));
    }

    pub(crate) fn u64(&mut self, value: u64) {
        self.0.update(value.to_be_bytes());
    }

    pub(crate) fn u128(&mut self, value: u128) {
        self.0.update(value.to_be_bytes());
    }

    pub(crate) fn i128(&mut self, value: i128) {
        self.0.update(value.to_be_bytes());
    }

    pub(crate) fn digest(&mut self, value: DigestV1) {
        self.0.update(value.0);
    }

    pub(crate) fn bytes(&mut self, value: &[u8]) -> Result<(), FailureV1> {
        self.u64(u64::try_from(value.len()).map_err(|_| invalid_request())?);
        self.0.update(value);
        Ok(())
    }

    pub(crate) fn string(&mut self, value: &str) -> Result<(), FailureV1> {
        self.bytes(value.as_bytes())
    }

    pub(crate) fn finish(self) -> DigestV1 {
        DigestV1(self.0.finalize().into())
    }
}

pub(crate) fn checked_u64(value: usize, failure: FailureV1) -> Result<u64, FailureV1> {
    u64::try_from(value).map_err(|_| failure)
}

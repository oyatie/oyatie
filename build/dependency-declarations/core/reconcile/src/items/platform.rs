/// Frozen validation limits for the v1 transaction.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ValidationBoundsV1;

impl ValidationBoundsV1 {
    pub const MAX_DECLARED_FILE_BYTES: usize = 32 * 1024 * 1024;
    pub const MAX_REPOSITORY_READ_FILES: u64 = 1_000_000;
    pub const MAX_REPOSITORY_READ_BYTES: u64 = 16 * 1024 * 1024 * 1024;
    pub const MAX_FIXUP_FILES: u64 = 16_384;
    pub const MAX_FIXUP_BYTES: u64 = 64 * 1024 * 1024;
    pub const MAX_CARGO_HOME_READ_FILES: u64 = 1_000_000;
    pub const MAX_CARGO_HOME_READ_BYTES: u64 = 16 * 1024 * 1024 * 1024;
    pub const MAX_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
    pub const MAX_STDERR_BYTES: usize = 1024 * 1024;
    pub const MAX_DIAGNOSTIC_BYTES: usize = 8 * 1024;
    pub const GENERATION_TIMEOUT_SECONDS: u64 = 120;
    pub const MAX_PATH_BYTES: usize = 4_096;
    pub const MAX_RULES: usize = 100_000;
    pub const MAX_ATTRIBUTES_PER_RULE: usize = 512;
    pub const MAX_LIST_ENTRIES: usize = 131_072;
    pub const MAX_SEMANTIC_NODES: usize = 1_000_000;
    pub const MAX_STRING_BYTES: usize = 1024 * 1024;
    pub const MAX_VALUE_DEPTH: usize = 128;
    pub const MAX_GRAPH_BYTES: usize = 64 * 1024 * 1024;
    pub const MAX_IDENTITY_BYTES: usize = 4_096;

    pub(crate) const fn tag(self) -> u8 {
        0
    }
}

/// One exact generation platform mapping.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PlatformIdentityV1 {
    pub(crate) name: Box<str>,
    pub(crate) target_triple: Box<str>,
    pub(crate) select_label: Box<str>,
    pub(crate) platform_label: Box<str>,
    pub(crate) execution_platform: bool,
}

impl PlatformIdentityV1 {
    /// Creates a validated platform mapping.
    pub fn try_new(
        name: impl Into<String>,
        target_triple: impl Into<String>,
        select_label: impl Into<String>,
        platform_label: impl Into<String>,
        execution_platform: bool,
    ) -> Result<Self, FailureV1> {
        let name = validated_identity(name.into())?;
        let target_triple = validated_identity(target_triple.into())?;
        let select_label = validated_identity(select_label.into())?;
        let platform_label = validated_identity(platform_label.into())?;
        Ok(Self {
            name,
            target_triple,
            select_label,
            platform_label,
            execution_platform,
        })
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.string(&self.name)?;
        hash.string(&self.target_triple)?;
        hash.string(&self.select_label)?;
        hash.string(&self.platform_label)?;
        hash.boolean(self.execution_platform);
        Ok(())
    }
}

/// A canonically ordered, duplicate-free platform set.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PlatformSetV1 {
    pub(crate) entries: Box<[PlatformIdentityV1]>,
}

impl PlatformSetV1 {
    /// Canonicalizes mappings by name and refuses colliding identities.
    pub fn try_new(mut entries: Vec<PlatformIdentityV1>) -> Result<Self, FailureV1> {
        if entries.is_empty() || entries.len() > 64 {
            return Err(invalid_request());
        }
        entries.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
        let mut names = std::collections::BTreeSet::new();
        let mut triples = std::collections::BTreeSet::new();
        let mut selects = std::collections::BTreeSet::new();
        let mut platforms = std::collections::BTreeSet::new();
        for entry in &entries {
            if !names.insert(entry.name.as_ref())
                || !triples.insert(entry.target_triple.as_ref())
                || !selects.insert(entry.select_label.as_ref())
                || !platforms.insert(entry.platform_label.as_ref())
            {
                return Err(invalid_request());
            }
        }
        Ok(Self {
            entries: entries.into_boxed_slice(),
        })
    }

    pub(crate) fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), FailureV1> {
        hash.u64(checked_u64(self.entries.len(), invalid_request())?);
        for entry in &self.entries {
            entry.encode(hash)?;
        }
        Ok(())
    }
}

/// Closed generation environment profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnvironmentProfileV1 {
    ReindeerHermeticV1,
}

/// Closed sandbox capability profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SandboxProfileV1 {
    DeclaredReadStageWriteNoNetworkV1,
}

/// Closed semantic validator profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValidatorProfileV1 {
    ReindeerBuckV1,
}

pub(crate) fn validated_identity(value: String) -> Result<Box<str>, FailureV1> {
    if value.is_empty()
        || value.len() > ValidationBoundsV1::MAX_IDENTITY_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(invalid_request());
    }
    Ok(value.into_boxed_str())
}

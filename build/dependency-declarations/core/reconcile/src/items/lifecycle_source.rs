/// Frozen limits for lifecycle facts admitted by one transaction.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct LifecycleBoundsV1;

impl LifecycleBoundsV1 {
    pub const MAX_SOURCE_OBJECT_BYTES: u64 = 128 * 1024 * 1024;
    pub const MAX_TOTAL_SOURCE_BYTES: u64 = 512 * 1024 * 1024;
    pub const MAX_SOURCE_OBJECTS: usize = 64;
    pub const MAX_RELEASE_ITEMS: usize = 100_000;
    pub const MAX_DISPOSITIONS: usize = 100_000;
    pub const MAX_TOOLCHAIN_TARGETS: usize = 64;
    pub const MAX_AFFECTED_UNITS: u64 = 1_000_000;
    pub const MAX_AFFECTED_UNIT_BYTES: u64 = 256 * 1024 * 1024;
}

/// Upstream component represented by an immutable source object.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum LifecycleComponentV1 {
    Rust = 0,
    Cargo = 1,
    Rustfmt = 2,
    Clippy = 3,
    RustDistribution = 4,
    DependencyRegistry = 5,
    RustSec = 6,
    Osv = 7,
    Cve = 8,
    Cna = 9,
    GitHubAdvisory = 10,
    Reindeer = 11,
    Buck2 = 12,
    Buck2ChangeDetector = 13,
}

/// Channel semantics are independent of a display version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum LifecycleChannelV1 {
    Stable = 0,
    Beta = 1,
    Nightly = 2,
    ReleasedTool = 3,
    Dependency = 4,
    Advisory = 5,
}

/// Whether upstream has published a final completeness boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SourceMaturityV1 {
    Released = 0,
    Provisional = 1,
}

/// Validated target identity for a target-specific upstream object.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LifecycleTargetTripleV1(Box<str>);

impl LifecycleTargetTripleV1 {
    pub fn try_new(value: impl Into<String>) -> Result<Self, LifecycleFailureV1> {
        Ok(Self(lifecycle_identity(value.into())?))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Explicit target scope of one source object.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LifecycleSourceScopeV1 {
    Global,
    Target(LifecycleTargetTripleV1),
}

impl LifecycleSourceScopeV1 {
    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        match self {
            Self::Global => hash.tag(0),
            Self::Target(target) => {
                hash.tag(1);
                lifecycle_hash_string(hash, target.as_str())?;
            }
        }
        Ok(())
    }
}

/// Semantic identity of one upstream source object.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LifecycleSourceDescriptorV1 {
    project: Box<str>,
    component: LifecycleComponentV1,
    channel: LifecycleChannelV1,
    revision: Box<str>,
    object_name: Box<str>,
    scope: LifecycleSourceScopeV1,
    maturity: SourceMaturityV1,
}

impl LifecycleSourceDescriptorV1 {
    pub fn try_new(
        project: impl Into<String>,
        component: LifecycleComponentV1,
        channel: LifecycleChannelV1,
        revision: impl Into<String>,
        object_name: impl Into<String>,
        scope: LifecycleSourceScopeV1,
        maturity: SourceMaturityV1,
    ) -> Result<Self, LifecycleFailureV1> {
        Ok(Self {
            project: lifecycle_identity(project.into())?,
            component,
            channel,
            revision: lifecycle_identity(revision.into())?,
            object_name: lifecycle_identity(object_name.into())?,
            scope,
            maturity,
        })
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        lifecycle_hash_string(hash, &self.project)?;
        hash.tag(self.component as u8);
        hash.tag(self.channel as u8);
        lifecycle_hash_string(hash, &self.revision)?;
        lifecycle_hash_string(hash, &self.object_name)?;
        self.scope.encode(hash)?;
        hash.tag(self.maturity as u8);
        Ok(())
    }

    #[must_use]
    pub fn project(&self) -> &str {
        &self.project
    }

    #[must_use]
    pub const fn component(&self) -> LifecycleComponentV1 {
        self.component
    }

    #[must_use]
    pub const fn channel(&self) -> LifecycleChannelV1 {
        self.channel
    }

    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    #[must_use]
    pub fn object_name(&self) -> &str {
        &self.object_name
    }

    #[must_use]
    pub const fn scope(&self) -> &LifecycleSourceScopeV1 {
        &self.scope
    }

    #[must_use]
    pub const fn maturity(&self) -> SourceMaturityV1 {
        self.maturity
    }
}

/// One exact, immutable upstream object.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LifecycleSourceV1 {
    descriptor: LifecycleSourceDescriptorV1,
    length_bytes: u64,
    object_sha256: DigestV1,
    schema_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl LifecycleSourceV1 {
    pub fn try_new(
        descriptor: LifecycleSourceDescriptorV1,
        length_bytes: u64,
        object_sha256: DigestV1,
        schema_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if length_bytes == 0 || length_bytes > LifecycleBoundsV1::MAX_SOURCE_OBJECT_BYTES {
            return Err(lifecycle_bounds());
        }
        let mut value = Self {
            descriptor,
            length_bytes,
            object_sha256,
            schema_sha256,
            identity_sha256: DigestV1::from_bytes([0; 32]),
        };
        let mut hash = CanonicalHasherV1::new(b"build.lifecycle-source.v1\0");
        value.encode_fields(&mut hash)?;
        value.identity_sha256 = hash.finish();
        Ok(value)
    }

    pub(crate) fn encode_fields(
        &self,
        hash: &mut CanonicalHasherV1,
    ) -> Result<(), LifecycleFailureV1> {
        self.descriptor.encode(hash)?;
        hash.u64(self.length_bytes);
        hash.digest(self.object_sha256);
        hash.digest(self.schema_sha256);
        Ok(())
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }

    #[must_use]
    pub const fn descriptor(&self) -> &LifecycleSourceDescriptorV1 {
        &self.descriptor
    }

    #[must_use]
    pub const fn component(&self) -> LifecycleComponentV1 {
        self.descriptor.component
    }

    #[must_use]
    pub const fn channel(&self) -> LifecycleChannelV1 {
        self.descriptor.channel
    }

    #[must_use]
    pub const fn maturity(&self) -> SourceMaturityV1 {
        self.descriptor.maturity
    }

    #[must_use]
    pub const fn length_bytes(&self) -> u64 {
        self.length_bytes
    }

    #[must_use]
    pub fn revision(&self) -> &str {
        &self.descriptor.revision
    }

    #[must_use]
    pub const fn object_sha256(&self) -> DigestV1 {
        self.object_sha256
    }

    #[must_use]
    pub const fn schema_sha256(&self) -> DigestV1 {
        self.schema_sha256
    }
}

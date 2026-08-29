/// Stable advisory identifier namespace.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AdvisoryNamespaceV1 {
    Cve = 0,
    Ghsa = 1,
    RustSec = 2,
    Osv = 3,
    Upstream = 4,
}

/// One canonical advisory identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdvisoryIdentifierV1 {
    namespace: AdvisoryNamespaceV1,
    value: Box<str>,
    identity_sha256: DigestV1,
}

impl AdvisoryIdentifierV1 {
    pub fn try_new(
        namespace: AdvisoryNamespaceV1,
        value: impl Into<String>,
    ) -> Result<Self, LifecycleFailureV1> {
        let value = lifecycle_identity(value.into())?;
        if !valid_advisory_identifier(namespace, &value) {
            return Err(lifecycle_invalid());
        }
        let mut hash = CanonicalHasherV1::new(b"build.advisory-identifier.v1\0");
        hash.tag(namespace as u8);
        lifecycle_hash_string(&mut hash, &value)?;
        Ok(Self {
            namespace,
            value,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn namespace(&self) -> AdvisoryNamespaceV1 {
        self.namespace
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Validated organization or upstream project identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdvisoryAuthorityNameV1(Box<str>);

impl AdvisoryAuthorityNameV1 {
    pub fn try_new(value: impl Into<String>) -> Result<Self, LifecycleFailureV1> {
        Ok(Self(lifecycle_identity(value.into())?))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Publisher role of one exact advisory source.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdvisoryAuthorityV1 {
    Cna(AdvisoryAuthorityNameV1),
    CveProgram,
    RustSec,
    Osv,
    GitHubAdvisory,
    Upstream(AdvisoryAuthorityNameV1),
}

impl AdvisoryAuthorityV1 {
    fn encode(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        match self {
            Self::Cna(name) => {
                hash.tag(0);
                lifecycle_hash_string(hash, name.as_str())?;
            }
            Self::CveProgram => hash.tag(1),
            Self::RustSec => hash.tag(2),
            Self::Osv => hash.tag(3),
            Self::GitHubAdvisory => hash.tag(4),
            Self::Upstream(name) => {
                hash.tag(5);
                lifecycle_hash_string(hash, name.as_str())?;
            }
        }
        Ok(())
    }

    fn matches_component(&self, component: LifecycleComponentV1) -> bool {
        matches!(
            (self, component),
            (Self::Cna(_), LifecycleComponentV1::Cna)
                | (Self::CveProgram, LifecycleComponentV1::Cve)
                | (Self::RustSec, LifecycleComponentV1::RustSec)
                | (Self::Osv, LifecycleComponentV1::Osv)
                | (
                    Self::GitHubAdvisory,
                    LifecycleComponentV1::GitHubAdvisory
                )
                | (Self::Upstream(_), LifecycleComponentV1::UpstreamAdvisory)
        )
    }

    fn matches_primary_namespace(&self, namespace: AdvisoryNamespaceV1) -> bool {
        matches!(
            (self, namespace),
            (
                Self::Cna(_) | Self::CveProgram,
                AdvisoryNamespaceV1::Cve
            ) | (Self::RustSec, AdvisoryNamespaceV1::RustSec)
                | (Self::Osv, AdvisoryNamespaceV1::Osv)
                | (Self::GitHubAdvisory, AdvisoryNamespaceV1::Ghsa)
                | (Self::Upstream(_), AdvisoryNamespaceV1::Upstream)
        )
    }
}

/// Qualification state of an advisory record producer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AdvisoryRecordQualificationV1 {
    Candidate {
        observation_receipt_sha256: DigestV1,
    },
    Qualified {
        qualification_receipt_sha256: DigestV1,
    },
}

impl AdvisoryRecordQualificationV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::Candidate {
                observation_receipt_sha256,
            } => {
                hash.tag(0);
                hash.digest(observation_receipt_sha256);
            }
            Self::Qualified {
                qualification_receipt_sha256,
            } => {
                hash.tag(1);
                hash.digest(qualification_receipt_sha256);
            }
        }
    }

    #[must_use]
    pub const fn is_qualified(self) -> bool {
        matches!(self, Self::Qualified { .. })
    }
}

/// Exact source, publisher role, and producer qualification for one record.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AdvisoryRecordSourceV1 {
    source: LifecycleSourceV1,
    authority: AdvisoryAuthorityV1,
    qualification: AdvisoryRecordQualificationV1,
    identity_sha256: DigestV1,
}

impl AdvisoryRecordSourceV1 {
    pub fn try_new(
        source: LifecycleSourceV1,
        authority: AdvisoryAuthorityV1,
        qualification: AdvisoryRecordQualificationV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if source.channel() != LifecycleChannelV1::Advisory
            || !authority.matches_component(source.component())
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::AdvisorySourceMismatch,
            ));
        }
        if qualification.is_qualified() && source.maturity() != SourceMaturityV1::Released {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ProvisionalSource,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.advisory-record-source.v1\0");
        hash.digest(source.identity_sha256());
        authority.encode(&mut hash)?;
        qualification.encode(&mut hash);
        Ok(Self {
            source,
            authority,
            qualification,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn source(&self) -> &LifecycleSourceV1 {
        &self.source
    }

    #[must_use]
    pub const fn authority(&self) -> &AdvisoryAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub const fn qualification(&self) -> AdvisoryRecordQualificationV1 {
        self.qualification
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn valid_advisory_identifier(namespace: AdvisoryNamespaceV1, value: &str) -> bool {
    match namespace {
        AdvisoryNamespaceV1::Cve => valid_numeric_advisory(value, "CVE-"),
        AdvisoryNamespaceV1::RustSec => valid_numeric_advisory(value, "RUSTSEC-"),
        AdvisoryNamespaceV1::Ghsa => valid_ghsa(value),
        AdvisoryNamespaceV1::Osv | AdvisoryNamespaceV1::Upstream => {
            !value.chars().any(char::is_whitespace)
        }
    }
}

fn valid_numeric_advisory(value: &str, prefix: &str) -> bool {
    let Some((year, sequence)) = value.strip_prefix(prefix).and_then(|rest| rest.split_once('-'))
    else {
        return false;
    };
    year.len() == 4
        && year.bytes().all(|byte| byte.is_ascii_digit())
        && sequence.len() >= 4
        && sequence.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_ghsa(value: &str) -> bool {
    let Some(suffix) = value.strip_prefix("GHSA-") else {
        return false;
    };
    let mut groups = suffix.split('-');
    let (Some(first), Some(second), Some(third), None) = (
        groups.next(),
        groups.next(),
        groups.next(),
        groups.next(),
    ) else {
        return false;
    };
    let valid_group = |group: &str| {
        group.len() == 4
            && group
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    };
    valid_group(first) && valid_group(second) && valid_group(third)
}

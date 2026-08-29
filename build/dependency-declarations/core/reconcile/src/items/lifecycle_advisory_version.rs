/// Exact Cargo package namespace and name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CargoPackageIdentityV1 {
    registry: Box<str>,
    name: Box<str>,
    identity_sha256: DigestV1,
}

impl CargoPackageIdentityV1 {
    pub fn try_new(
        registry: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, LifecycleFailureV1> {
        let registry = lifecycle_identity(registry.into())?;
        let name = lifecycle_identity(name.into())?;
        let mut hash = CanonicalHasherV1::new(b"build.cargo-package-identity.v1\0");
        lifecycle_hash_string(&mut hash, &registry)?;
        lifecycle_hash_string(&mut hash, &name)?;
        Ok(Self {
            registry,
            name,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn registry(&self) -> &str {
        &self.registry
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Canonical Cargo package version with SemVer precedence available internally.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CargoVersionV1 {
    canonical: Box<str>,
    parsed: semver::Version,
    identity_sha256: DigestV1,
}

impl CargoVersionV1 {
    pub fn try_new(value: impl Into<String>) -> Result<Self, LifecycleFailureV1> {
        let canonical = lifecycle_identity(value.into())?;
        let parsed = semver::Version::parse(&canonical).map_err(|_| {
            LifecycleFailureV1::new(LifecycleFailureClassV1::InvalidPackageVersion)
        })?;
        if parsed.to_string() != canonical.as_ref() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::InvalidPackageVersion,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.cargo-version.v1\0");
        lifecycle_hash_string(&mut hash, &canonical)?;
        Ok(Self {
            canonical,
            parsed,
            identity_sha256: hash.finish(),
        })
    }

    fn precedence_cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.parsed.cmp_precedence(&other.parsed)
    }

    fn has_build_metadata(&self) -> bool {
        !self.parsed.build.is_empty()
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Inclusive beginning of one affected Cargo version interval.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AdvisoryRangeStartV1 {
    Beginning,
    Introduced(CargoVersionV1),
}

/// End semantics of one affected Cargo version interval.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AdvisoryRangeEndV1 {
    Unbounded,
    Fixed(CargoVersionV1),
    LastAffected(CargoVersionV1),
}

/// One normalized affected Cargo interval.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CargoAffectedRangeV1 {
    start: AdvisoryRangeStartV1,
    end: AdvisoryRangeEndV1,
    identity_sha256: DigestV1,
}

impl CargoAffectedRangeV1 {
    pub fn try_new(
        start: AdvisoryRangeStartV1,
        end: AdvisoryRangeEndV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if endpoint_has_build_metadata(&start, &end) || !valid_interval(&start, &end) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::InvalidPackageVersion,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.cargo-affected-range.v1\0");
        encode_range_start(&mut hash, &start);
        encode_range_end(&mut hash, &end);
        Ok(Self {
            start,
            end,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn contains(&self, version: &CargoVersionV1) -> bool {
        let after_start = match &self.start {
            AdvisoryRangeStartV1::Beginning => true,
            AdvisoryRangeStartV1::Introduced(start) => {
                start.precedence_cmp(version) != std::cmp::Ordering::Greater
            }
        };
        let before_end = match &self.end {
            AdvisoryRangeEndV1::Unbounded => true,
            AdvisoryRangeEndV1::Fixed(end) => {
                version.precedence_cmp(end) == std::cmp::Ordering::Less
            }
            AdvisoryRangeEndV1::LastAffected(end) => {
                version.precedence_cmp(end) != std::cmp::Ordering::Greater
            }
        };
        after_start && before_end
    }

    #[must_use]
    pub const fn start(&self) -> &AdvisoryRangeStartV1 {
        &self.start
    }

    #[must_use]
    pub const fn end(&self) -> &AdvisoryRangeEndV1 {
        &self.end
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Canonical affected intervals for one Cargo package.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CargoAdvisoryClaimV1 {
    package: CargoPackageIdentityV1,
    ranges: Box<[CargoAffectedRangeV1]>,
    identity_sha256: DigestV1,
}

impl CargoAdvisoryClaimV1 {
    pub fn try_new(
        package: CargoPackageIdentityV1,
        mut ranges: Vec<CargoAffectedRangeV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        if ranges.is_empty() || ranges.len() > LifecycleBoundsV1::MAX_ADVISORY_RANGES_PER_PACKAGE {
            return Err(lifecycle_bounds());
        }
        ranges.sort_by(compare_ranges);
        if ranges
            .windows(2)
            .any(|pair| !ends_before_start(&pair[0].end, &pair[1].start))
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ConflictingAdvisoryRange,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.cargo-advisory-claim.v1\0");
        hash.digest(package.identity_sha256());
        hash.u64(lifecycle_len(ranges.len())?);
        for range in &ranges {
            hash.digest(range.identity_sha256());
        }
        Ok(Self {
            package,
            ranges: ranges.into_boxed_slice(),
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn contains(&self, version: &CargoVersionV1) -> bool {
        self.ranges.iter().any(|range| range.contains(version))
    }

    #[must_use]
    pub const fn package(&self) -> &CargoPackageIdentityV1 {
        &self.package
    }

    #[must_use]
    pub fn ranges(&self) -> &[CargoAffectedRangeV1] {
        &self.ranges
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

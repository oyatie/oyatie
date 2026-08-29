/// Current lifecycle after retaining every source revision.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NormalizedAdvisoryLifecycleV1 {
    Active,
    Withdrawn,
}

/// Qualification level of the normalized affected-package set.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NormalizedAdvisoryAffectedSetQualificationV1 {
    ReferenceOnly,
    Candidate,
    Qualified,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum NormalizedAffectedStateV1 {
    ReferenceOnly,
    Candidate(AdvisoryAffectedSetV1),
    Qualified(AdvisoryAffectedSetV1),
}

impl NormalizedAffectedStateV1 {
    fn qualification(&self) -> NormalizedAdvisoryAffectedSetQualificationV1 {
        match self {
            Self::ReferenceOnly => NormalizedAdvisoryAffectedSetQualificationV1::ReferenceOnly,
            Self::Candidate(_) => NormalizedAdvisoryAffectedSetQualificationV1::Candidate,
            Self::Qualified(_) => NormalizedAdvisoryAffectedSetQualificationV1::Qualified,
        }
    }

    fn encode(&self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::ReferenceOnly => hash.tag(0),
            Self::Candidate(affected) => {
                hash.tag(1);
                hash.digest(affected.identity_sha256());
            }
            Self::Qualified(affected) => {
                hash.tag(2);
                hash.digest(affected.identity_sha256());
            }
        }
    }
}

/// One alias-connected vulnerability with complete source history.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NormalizedAdvisoryFactV1 {
    canonical: AdvisoryIdentifierV1,
    identifiers: Box<[AdvisoryIdentifierV1]>,
    records: Box<[AdvisoryRecordV1]>,
    lifecycle: NormalizedAdvisoryLifecycleV1,
    affected: NormalizedAffectedStateV1,
    identity_sha256: DigestV1,
}

impl NormalizedAdvisoryFactV1 {
    fn try_from_records(
        mut records: Vec<AdvisoryRecordV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        let mut identifiers: Vec<AdvisoryIdentifierV1> = records
            .iter()
            .flat_map(AdvisoryRecordV1::identifiers)
            .cloned()
            .collect();
        identifiers.sort();
        identifiers.dedup();
        let canonical = identifiers.first().cloned().ok_or_else(lifecycle_internal)?;

        let latest = latest_advisory_records(&records)?;
        let lifecycle = if latest.iter().all(|record| record.lifecycle().is_withdrawn()) {
            NormalizedAdvisoryLifecycleV1::Withdrawn
        } else {
            NormalizedAdvisoryLifecycleV1::Active
        };
        let active: Vec<&AdvisoryRecordV1> = latest
            .iter()
            .copied()
            .filter(|record| !record.lifecycle().is_withdrawn())
            .collect();
        let affected = if active.is_empty() {
            normalized_affected_state(&latest)?
        } else {
            normalized_affected_state(&active)?
        };

        records.sort_by(|left, right| {
            (
                left.lifecycle().modified_at(),
                left.source().identity_sha256(),
                left.identity_sha256(),
            )
                .cmp(&(
                    right.lifecycle().modified_at(),
                    right.source().identity_sha256(),
                    right.identity_sha256(),
                ))
        });
        let mut hash = CanonicalHasherV1::new(b"build.normalized-advisory-fact.v1\0");
        hash.digest(canonical.identity_sha256());
        hash.u64(lifecycle_len(identifiers.len())?);
        for identifier in &identifiers {
            hash.digest(identifier.identity_sha256());
        }
        hash.u64(lifecycle_len(records.len())?);
        for record in &records {
            hash.digest(record.identity_sha256());
        }
        hash.tag(match lifecycle {
            NormalizedAdvisoryLifecycleV1::Active => 0,
            NormalizedAdvisoryLifecycleV1::Withdrawn => 1,
        });
        affected.encode(&mut hash);
        Ok(Self {
            canonical,
            identifiers: identifiers.into_boxed_slice(),
            records: records.into_boxed_slice(),
            lifecycle,
            affected,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn canonical(&self) -> &AdvisoryIdentifierV1 {
        &self.canonical
    }

    #[must_use]
    pub fn identifiers(&self) -> &[AdvisoryIdentifierV1] {
        &self.identifiers
    }

    #[must_use]
    pub fn records(&self) -> &[AdvisoryRecordV1] {
        &self.records
    }

    #[must_use]
    pub const fn lifecycle(&self) -> NormalizedAdvisoryLifecycleV1 {
        self.lifecycle
    }

    #[must_use]
    pub fn affected_set_qualification(&self) -> NormalizedAdvisoryAffectedSetQualificationV1 {
        self.affected.qualification()
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Bounded result of one in-memory advisory alias normalization pass.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AdvisoryLedgerV1 {
    facts: Box<[NormalizedAdvisoryFactV1]>,
    record_count: u64,
    identifier_occurrence_count: u64,
    identity_sha256: DigestV1,
}

impl AdvisoryLedgerV1 {
    pub fn try_normalize(
        mut records: Vec<AdvisoryRecordV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        if records.is_empty() || records.len() > LifecycleBoundsV1::MAX_ADVISORY_RECORDS {
            return Err(lifecycle_bounds());
        }
        records.sort_by_key(AdvisoryRecordV1::identity_sha256);
        if records.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }

        let identifier_occurrence_count = records.iter().try_fold(0_usize, |total, record| {
            total
                .checked_add(record.identifiers().count())
                .filter(|count| *count <= LifecycleBoundsV1::MAX_TOTAL_ADVISORY_IDENTIFIERS)
                .ok_or_else(lifecycle_bounds)
        })?;
        validate_advisory_input_bounds(&records)?;
        validate_advisory_source_keys(&records)?;
        validate_advisory_qualification_lane(&records)?;

        let mut sets = AdvisoryUnionFindV1::new(records.len());
        let mut first_by_identifier: std::collections::HashMap<AdvisoryIdentifierV1, usize> =
            std::collections::HashMap::with_capacity(identifier_occurrence_count);
        for (index, record) in records.iter().enumerate() {
            for identifier in record.identifiers() {
                if let Some(first) = first_by_identifier.get(identifier) {
                    sets.union(index, *first);
                } else {
                    first_by_identifier.insert(identifier.clone(), index);
                }
            }
        }

        let roots: Vec<usize> = (0..records.len()).map(|index| sets.find(index)).collect();
        let mut groups: std::collections::BTreeMap<usize, Vec<AdvisoryRecordV1>> =
            std::collections::BTreeMap::new();
        for (root, record) in roots.into_iter().zip(records) {
            groups.entry(root).or_default().push(record);
        }
        let mut facts: Vec<NormalizedAdvisoryFactV1> = groups
            .into_values()
            .map(NormalizedAdvisoryFactV1::try_from_records)
            .collect::<Result<_, _>>()?;
        facts.sort_by(|left, right| left.canonical.cmp(&right.canonical));

        let record_count = facts.iter().try_fold(0_usize, |total, fact| {
            total
                .checked_add(fact.records.len())
                .ok_or_else(lifecycle_bounds)
        })?;
        let mut hash = CanonicalHasherV1::new(b"build.advisory-ledger.v1\0");
        hash.u64(lifecycle_len(facts.len())?);
        for fact in &facts {
            hash.digest(fact.identity_sha256());
        }
        hash.u64(lifecycle_len(record_count)?);
        hash.u64(lifecycle_len(identifier_occurrence_count)?);
        Ok(Self {
            facts: facts.into_boxed_slice(),
            record_count: lifecycle_len(record_count)?,
            identifier_occurrence_count: lifecycle_len(identifier_occurrence_count)?,
            identity_sha256: hash.finish(),
        })
    }

    /// Normalized observations; absence is not a corpus-coverage proof.
    #[must_use]
    pub fn facts(&self) -> &[NormalizedAdvisoryFactV1] {
        &self.facts
    }

    #[must_use]
    pub const fn record_count(&self) -> u64 {
        self.record_count
    }

    #[must_use]
    pub const fn identifier_occurrence_count(&self) -> u64 {
        self.identifier_occurrence_count
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

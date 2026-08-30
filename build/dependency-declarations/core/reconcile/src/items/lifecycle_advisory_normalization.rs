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
    fn try_from_records<C>(
        mut records: Vec<AdvisoryRecordV1>,
        control: &mut AdvisoryNormalizationControlV1<C>,
    ) -> Result<Self, LifecycleFailureV1>
    where
        C: FnMut(AdvisoryNormalizationProgressV1) -> LifecycleControlDecisionV1,
    {
        let mut identifiers = Vec::new();
        for record in &records {
            for identifier in record.identifiers() {
                identifiers.push(identifier.clone());
                control.record_work()?;
            }
        }
        control.checkpoint_and_reset()?;
        identifiers.sort();
        control.checkpoint_and_reset()?;
        let mut unique_identifiers = Vec::with_capacity(identifiers.len());
        for identifier in identifiers {
            if unique_identifiers.last() != Some(&identifier) {
                unique_identifiers.push(identifier);
            }
            control.record_work()?;
        }
        let identifiers = unique_identifiers;
        control.checkpoint_and_reset()?;
        let canonical = identifiers.first().cloned().ok_or_else(lifecycle_internal)?;

        let latest = latest_advisory_records(&records, control)?;
        let mut all_withdrawn = true;
        let mut active = Vec::with_capacity(latest.len());
        for record in &latest {
            let withdrawn = record.lifecycle().is_withdrawn();
            all_withdrawn &= withdrawn;
            if !withdrawn {
                active.push(*record);
            }
            control.record_work()?;
        }
        let lifecycle = if all_withdrawn {
            NormalizedAdvisoryLifecycleV1::Withdrawn
        } else {
            NormalizedAdvisoryLifecycleV1::Active
        };
        let affected = if active.is_empty() {
            normalized_affected_state(&latest, control)?
        } else {
            normalized_affected_state(&active, control)?
        };

        control.checkpoint_and_reset()?;
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
        control.checkpoint_and_reset()?;
        let mut hash = CanonicalHasherV1::new(b"build.normalized-advisory-fact.v1\0");
        hash.digest(canonical.identity_sha256());
        hash.u64(lifecycle_len(identifiers.len())?);
        for identifier in &identifiers {
            hash.digest(identifier.identity_sha256());
            control.record_work()?;
        }
        hash.u64(lifecycle_len(records.len())?);
        for record in &records {
            hash.digest(record.identity_sha256());
            control.record_work()?;
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
    pub fn try_normalize<C>(
        mut records: Vec<AdvisoryRecordV1>,
        control: C,
    ) -> Result<Self, LifecycleFailureV1>
    where
        C: FnMut(AdvisoryNormalizationProgressV1) -> LifecycleControlDecisionV1,
    {
        if records.is_empty() || records.len() > LifecycleBoundsV1::MAX_ADVISORY_RECORDS {
            return Err(lifecycle_bounds());
        }
        let mut control = AdvisoryNormalizationControlV1::try_new(control)?;
        records.sort_by_key(AdvisoryRecordV1::identity_sha256);
        control.checkpoint_and_reset()?;
        for pair in records.windows(2) {
            if pair[0] == pair[1] {
                return Err(LifecycleFailureV1::new(
                    LifecycleFailureClassV1::DuplicateIdentity,
                ));
            }
            control.record_work()?;
        }
        control.checkpoint_and_reset()?;

        let identifier_occurrence_count = validate_advisory_input_bounds(&records, &mut control)?;
        validate_advisory_source_keys(&records, &mut control)?;
        control.checkpoint_and_reset()?;
        validate_advisory_qualification_lane(&records, &mut control)?;
        control.checkpoint_and_reset()?;

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
                control.record_work()?;
            }
        }
        control.checkpoint_and_reset()?;

        let mut roots = Vec::with_capacity(records.len());
        for index in 0..records.len() {
            roots.push(sets.find(index));
            control.record_work()?;
        }
        control.checkpoint_and_reset()?;
        let mut groups: std::collections::BTreeMap<usize, Vec<AdvisoryRecordV1>> =
            std::collections::BTreeMap::new();
        for (root, record) in roots.into_iter().zip(records) {
            groups.entry(root).or_default().push(record);
            control.record_work()?;
        }
        control.checkpoint_and_reset()?;
        let mut facts = Vec::with_capacity(groups.len());
        for group in groups.into_values() {
            facts.push(NormalizedAdvisoryFactV1::try_from_records(
                group,
                &mut control,
            )?);
        }
        control.checkpoint_and_reset()?;
        facts.sort_by(|left, right| left.canonical.cmp(&right.canonical));
        control.checkpoint_and_reset()?;

        let mut record_count = 0_usize;
        for fact in &facts {
            record_count = record_count
                .checked_add(fact.records.len())
                .ok_or_else(lifecycle_bounds)?;
            control.record_work()?;
        }
        let mut hash = CanonicalHasherV1::new(b"build.advisory-ledger.v1\0");
        hash.u64(lifecycle_len(facts.len())?);
        for fact in &facts {
            hash.digest(fact.identity_sha256());
            control.record_work()?;
        }
        hash.u64(lifecycle_len(record_count)?);
        hash.u64(lifecycle_len(identifier_occurrence_count)?);
        control.checkpoint_and_reset()?;
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

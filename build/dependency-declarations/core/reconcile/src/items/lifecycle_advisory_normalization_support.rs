fn latest_advisory_records<'a, C>(
    records: &'a [AdvisoryRecordV1],
    control: &mut AdvisoryNormalizationControlV1<C>,
) -> Result<Vec<&'a AdvisoryRecordV1>, LifecycleFailureV1>
where
    C: FnMut(AdvisoryNormalizationProgressV1) -> LifecycleControlDecisionV1,
{
    let mut latest: std::collections::BTreeMap<
        (AdvisoryAuthorityV1, AdvisoryIdentifierV1),
        &AdvisoryRecordV1,
    > = std::collections::BTreeMap::new();
    for record in records {
        let key = (
            record.source().authority().clone(),
            record.primary().clone(),
        );
        match latest.get(&key).copied() {
            None => {
                latest.insert(key, record);
            }
            Some(current) => {
                let ordering = record
                    .lifecycle()
                    .modified_at()
                    .cmp(&current.lifecycle().modified_at());
                if ordering == std::cmp::Ordering::Equal && !record.same_payload(current) {
                    return Err(LifecycleFailureV1::new(
                        LifecycleFailureClassV1::ConflictingAdvisoryHistory,
                    ));
                }
                if ordering == std::cmp::Ordering::Greater
                    || (ordering == std::cmp::Ordering::Equal
                        && prefer_advisory_record(record, current))
                {
                    latest.insert(key, record);
                }
            }
        }
        control.record_work()?;
    }
    Ok(latest.into_values().collect())
}

fn prefer_advisory_record(candidate: &AdvisoryRecordV1, current: &AdvisoryRecordV1) -> bool {
    let candidate_qualified = candidate.source().qualification().is_qualified();
    let current_qualified = current.source().qualification().is_qualified();
    (candidate_qualified && !current_qualified)
        || (candidate_qualified == current_qualified
            && candidate.source().identity_sha256() > current.source().identity_sha256())
}

fn normalized_affected_state<C>(
    records: &[&AdvisoryRecordV1],
    control: &mut AdvisoryNormalizationControlV1<C>,
) -> Result<NormalizedAffectedStateV1, LifecycleFailureV1>
where
    C: FnMut(AdvisoryNormalizationProgressV1) -> LifecycleControlDecisionV1,
{
    let mut affected: Option<AdvisoryAffectedSetV1> = None;
    let mut qualified = false;
    for record in records {
        if record.affected().completeness() == AdvisoryAffectedSetCompletenessV1::ReferenceOnly {
            control.record_work()?;
            continue;
        }
        match &affected {
            None => affected = Some(record.affected().clone()),
            Some(current) if current != record.affected() => {
                return Err(LifecycleFailureV1::new(
                    LifecycleFailureClassV1::ConflictingAdvisoryRange,
                ));
            }
            Some(_) => {}
        }
        qualified |= record.source().qualification().is_qualified();
        control.record_work()?;
    }
    Ok(match affected {
        None => NormalizedAffectedStateV1::ReferenceOnly,
        Some(affected) if qualified => NormalizedAffectedStateV1::Qualified(affected),
        Some(affected) => NormalizedAffectedStateV1::Candidate(affected),
    })
}

fn validate_advisory_source_keys<C>(
    records: &[AdvisoryRecordV1],
    control: &mut AdvisoryNormalizationControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(AdvisoryNormalizationProgressV1) -> LifecycleControlDecisionV1,
{
    let mut keys = std::collections::BTreeSet::new();
    for record in records {
        let key = (
            record.source().source().identity_sha256(),
            record.primary().clone(),
        );
        if !keys.insert(key) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        control.record_work()?;
    }
    Ok(())
}

fn validate_advisory_qualification_lane<C>(
    records: &[AdvisoryRecordV1],
    control: &mut AdvisoryNormalizationControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(AdvisoryNormalizationProgressV1) -> LifecycleControlDecisionV1,
{
    let qualified = records
        .first()
        .ok_or_else(lifecycle_internal)?
        .source()
        .qualification()
        .is_qualified();
    for record in records {
        if record.source().qualification().is_qualified() != qualified {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::MixedAdvisoryQualification,
            ));
        }
        control.record_work()?;
    }
    Ok(())
}

fn validate_advisory_input_bounds<C>(
    records: &[AdvisoryRecordV1],
    control: &mut AdvisoryNormalizationControlV1<C>,
) -> Result<usize, LifecycleFailureV1>
where
    C: FnMut(AdvisoryNormalizationProgressV1) -> LifecycleControlDecisionV1,
{
    let mut identifier_occurrences = 0_usize;
    let mut package_claims = 0_usize;
    let mut ranges = 0_usize;
    for record in records {
        for _ in record.identifiers() {
            identifier_occurrences = identifier_occurrences
                .checked_add(1)
                .filter(|count| *count <= LifecycleBoundsV1::MAX_TOTAL_ADVISORY_IDENTIFIERS)
                .ok_or_else(lifecycle_bounds)?;
            control.complete_identifier_occurrence()?;
        }
        let Some(claims) = record.affected().claims() else {
            control.complete_record()?;
            continue;
        };
        for claim in claims {
            package_claims = package_claims
                .checked_add(1)
                .filter(|count| *count <= LifecycleBoundsV1::MAX_TOTAL_ADVISORY_PACKAGE_CLAIMS)
                .ok_or_else(lifecycle_bounds)?;
            control.complete_package_claim()?;
            for _ in claim.ranges() {
                ranges = ranges
                    .checked_add(1)
                    .filter(|count| *count <= LifecycleBoundsV1::MAX_TOTAL_ADVISORY_RANGES)
                    .ok_or_else(lifecycle_bounds)?;
                control.complete_range()?;
            }
        }
        control.complete_record()?;
    }
    control.checkpoint_and_reset()?;
    Ok(identifier_occurrences)
}

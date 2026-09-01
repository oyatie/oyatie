use std::collections::{BTreeMap, BTreeSet};

use super::validation::{semantic_claim_id, valid_projection_target, valid_work_reference};
use super::{
    OwnerProseClassification, OwnerProseRefusal, OwnerProseRefusalKind, OwnerProseRevision,
    OwnerProseSource, QualifiedOwnerProseClaim, canonical_digest, owner_prose_sha256,
};

pub(super) type ProjectionIntervals = BTreeMap<String, Vec<(usize, usize, String)>>;

pub(super) struct ClaimState<'a> {
    pub source_paths: &'a BTreeSet<String>,
    pub claim_ids: &'a mut BTreeSet<String>,
    pub candidate_blobs: &'a mut BTreeMap<String, Vec<u8>>,
    pub intervals: &'a mut ProjectionIntervals,
    pub refusals: &'a mut Vec<OwnerProseRefusal>,
    pub qualified: &'a mut Vec<QualifiedOwnerProseClaim>,
}

pub(super) fn validate_claims<F>(
    source: &OwnerProseSource,
    bytes: &[u8],
    read_blob: &mut F,
    state: &mut ClaimState<'_>,
) where
    F: FnMut(OwnerProseRevision, &str) -> Result<Option<Vec<u8>>, String>,
{
    let mut claims = source.claims.clone();
    claims.sort_by(|left, right| {
        (left.start, left.end, &left.id).cmp(&(right.start, right.end, &right.id))
    });
    let mut cursor = 0;
    if claims.is_empty() {
        refuse(
            state,
            OwnerProseRefusalKind::ClaimCoverageMismatch,
            &source.path,
            "source bytes have no classified claims",
        );
    }
    for claim in claims {
        if !semantic_claim_id(&claim.id) {
            refuse(
                state,
                OwnerProseRefusalKind::ClaimIdentityInvalid,
                &source.path,
                format!("claim id {:?} is not a semantic identifier", claim.id),
            );
        } else if !state.claim_ids.insert(claim.id.clone()) {
            refuse(
                state,
                OwnerProseRefusalKind::DuplicateClassification,
                &source.path,
                format!("claim id {:?} is classified more than once", claim.id),
            );
        }
        let range_in_bounds = claim.start < claim.end && claim.end <= bytes.len();
        if !range_in_bounds || claim.start != cursor {
            refuse(
                state,
                OwnerProseRefusalKind::ClaimCoverageMismatch,
                format!("{}#{}", source.path, claim.id),
                format!(
                    "expected next byte range to start at {cursor}, got {}..{} for {} bytes",
                    claim.start,
                    claim.end,
                    bytes.len()
                ),
            );
        }
        if range_in_bounds {
            let actual = owner_prose_sha256(&bytes[claim.start..claim.end]);
            if !canonical_digest(&claim.sha256) || claim.sha256 != actual {
                refuse(
                    state,
                    OwnerProseRefusalKind::ClaimDigestMismatch,
                    format!("{}#{}", source.path, claim.id),
                    "claim digest does not match its exact source byte range",
                );
            }
            cursor = claim.end;
        }

        match claim.classification {
            OwnerProseClassification::Unknown => refuse(
                state,
                OwnerProseRefusalKind::UnknownClassification,
                format!("{}#{}", source.path, claim.id),
                "classification is Unknown",
            ),
            OwnerProseClassification::AcceptedCurrent if claim.projections.len() != 1 => refuse(
                state,
                OwnerProseRefusalKind::ProjectionCountMismatch,
                format!("{}#{}", source.path, claim.id),
                "accepted-current requires exactly one native projection",
            ),
            OwnerProseClassification::ProposalWork
            | OwnerProseClassification::HistoricalRejected
                if !claim.projections.is_empty() =>
            {
                refuse(
                    state,
                    OwnerProseRefusalKind::ProjectionCountMismatch,
                    format!("{}#{}", source.path, claim.id),
                    "non-current classifications cannot project into current authority",
                );
            }
            _ => {}
        }
        match claim.classification {
            OwnerProseClassification::ProposalWork
                if !claim
                    .work_reference
                    .as_ref()
                    .is_some_and(valid_work_reference) =>
            {
                refuse(
                    state,
                    OwnerProseRefusalKind::WorkReferenceInvalid,
                    format!("{}#{}", source.path, claim.id),
                    "proposal/work requires one semantic HTTPS PR or external-work locator",
                );
            }
            OwnerProseClassification::AcceptedCurrent
            | OwnerProseClassification::HistoricalRejected
            | OwnerProseClassification::Unknown
                if claim.work_reference.is_some() =>
            {
                refuse(
                    state,
                    OwnerProseRefusalKind::WorkReferenceInvalid,
                    format!("{}#{}", source.path, claim.id),
                    "only proposal/work may carry an external work reference",
                );
            }
            _ => {}
        }
        if matches!(
            claim.classification,
            OwnerProseClassification::AcceptedCurrent
        ) {
            for projection in &claim.projections {
                validate_projection(source, &claim.id, projection, read_blob, state);
            }
        }
        state.qualified.push(QualifiedOwnerProseClaim {
            source_path: source.path.clone(),
            id: claim.id,
            start: claim.start,
            end: claim.end,
            sha256: claim.sha256,
            classification: claim.classification,
            work_reference: claim.work_reference,
            projections: claim.projections,
        });
    }
    if cursor != bytes.len() {
        refuse(
            state,
            OwnerProseRefusalKind::ClaimCoverageMismatch,
            &source.path,
            format!(
                "classified bytes end at {cursor}, source ends at {}",
                bytes.len()
            ),
        );
    }
}

fn validate_projection<F>(
    source: &OwnerProseSource,
    claim_id: &str,
    projection: &super::OwnerProseProjection,
    read_blob: &mut F,
    state: &mut ClaimState<'_>,
) where
    F: FnMut(OwnerProseRevision, &str) -> Result<Option<Vec<u8>>, String>,
{
    let subject = format!("{}#{claim_id} -> {}", source.path, projection.path);
    if !valid_projection_target(projection, state.source_paths) {
        refuse(
            state,
            OwnerProseRefusalKind::ProjectionTargetInvalid,
            subject,
            "projection target is not matching semantic native authority",
        );
        return;
    }
    if !state.candidate_blobs.contains_key(&projection.path) {
        match read_blob(OwnerProseRevision::Candidate, &projection.path) {
            Ok(Some(bytes)) => {
                state.candidate_blobs.insert(projection.path.clone(), bytes);
            }
            Ok(None) => {
                refuse(
                    state,
                    OwnerProseRefusalKind::ProjectionUnavailable,
                    subject,
                    "candidate projection target is absent",
                );
                return;
            }
            Err(error) => {
                refuse(
                    state,
                    OwnerProseRefusalKind::RepositoryReadFailed,
                    subject,
                    error,
                );
                return;
            }
        }
    }
    let bytes = &state.candidate_blobs[&projection.path];
    if projection.start >= projection.end || projection.end > bytes.len() {
        refuse(
            state,
            OwnerProseRefusalKind::ProjectionDigestMismatch,
            subject,
            format!(
                "projection byte range {}..{} is outside {} candidate bytes",
                projection.start,
                projection.end,
                bytes.len()
            ),
        );
        return;
    }
    let actual = owner_prose_sha256(&bytes[projection.start..projection.end]);
    if !canonical_digest(&projection.sha256) || projection.sha256 != actual {
        refuse(
            state,
            OwnerProseRefusalKind::ProjectionDigestMismatch,
            subject,
            "projection digest does not match its exact candidate byte range",
        );
    }
    state
        .intervals
        .entry(projection.path.clone())
        .or_default()
        .push((projection.start, projection.end, claim_id.to_owned()));
}

fn refuse(
    state: &mut ClaimState<'_>,
    kind: OwnerProseRefusalKind,
    subject: impl Into<String>,
    detail: impl Into<String>,
) {
    state
        .refusals
        .push(OwnerProseRefusal::new(kind, subject, detail));
}

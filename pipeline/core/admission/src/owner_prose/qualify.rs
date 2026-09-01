use std::collections::{BTreeMap, BTreeSet};

use super::claims::{ClaimState, ProjectionIntervals, validate_claims};
use super::{
    OWNER_PROSE_CLASSIFICATION_SCHEMA, OWNER_PROSE_CLASSIFIER_IDENTITY, OWNER_PROSE_NAMES,
    OWNER_PROSE_PRODUCER_SCHEMA, OWNER_PROSE_QUALIFIED_VIEW_SCHEMA, OWNER_PROSE_QUALIFIER_IDENTITY,
    OWNER_PROSE_QUALIFIER_SCHEMA, OwnerProseManifest, OwnerProsePathDigest, OwnerProseProducer,
    OwnerProseQualification, OwnerProseRefusal, OwnerProseRefusalKind, OwnerProseRepositoryBinding,
    OwnerProseRevision, QualifiedOwnerProseClaim, QualifiedOwnerProseView, canonical_digest,
    owner_prose_sha256,
};

pub fn qualify_owner_prose<F>(
    manifest_bytes: &[u8],
    observed: &OwnerProseRepositoryBinding,
    mut read_blob: F,
) -> OwnerProseQualification
where
    F: FnMut(OwnerProseRevision, &str) -> Result<Option<Vec<u8>>, String>,
{
    let manifest: OwnerProseManifest = match serde_json::from_slice(manifest_bytes) {
        Ok(manifest) => manifest,
        Err(error) => {
            return OwnerProseQualification::unknown(vec![OwnerProseRefusal::new(
                OwnerProseRefusalKind::ManifestInvalid,
                "owner-prose view",
                format!("strict JSON decoding failed: {error}"),
            )]);
        }
    };
    let mut refusals = validate_envelope(&manifest, observed);
    if !refusals.is_empty() {
        return OwnerProseQualification::unknown(refusals);
    }

    let expected_paths: BTreeSet<String> = OWNER_PROSE_NAMES
        .iter()
        .map(|name| format!("{}/{name}", manifest.owner))
        .collect();
    let source_paths: BTreeSet<String> = manifest
        .sources
        .iter()
        .map(|source| source.path.clone())
        .collect();
    if source_paths.len() != manifest.sources.len() || source_paths != expected_paths {
        refusals.push(OwnerProseRefusal::new(
            OwnerProseRefusalKind::SourceSetMismatch,
            &manifest.owner,
            format!(
                "source records must equal the four fixed owner-law paths; expected {expected_paths:?}, got {source_paths:?}"
            ),
        ));
        return OwnerProseQualification::unknown(refusals);
    }

    let mut sources = manifest.sources.clone();
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    let mut source_digests = Vec::new();
    let mut candidate_blobs = BTreeMap::new();
    let mut claim_ids = BTreeSet::new();
    let mut intervals = ProjectionIntervals::new();
    let mut qualified = Vec::<QualifiedOwnerProseClaim>::new();
    for source in &sources {
        verify_deleted_in_candidate(source, &mut read_blob, &mut refusals);
        let bytes = match read_blob(OwnerProseRevision::Source, &source.path) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                refusals.push(OwnerProseRefusal::new(
                    OwnerProseRefusalKind::SourceUnavailable,
                    &source.path,
                    "fixed owner-law source is absent at the bound source revision",
                ));
                continue;
            }
            Err(error) => {
                refusals.push(OwnerProseRefusal::new(
                    OwnerProseRefusalKind::RepositoryReadFailed,
                    &source.path,
                    error,
                ));
                continue;
            }
        };
        let actual = owner_prose_sha256(&bytes);
        source_digests.push(OwnerProsePathDigest {
            path: source.path.clone(),
            sha256: actual.clone(),
        });
        if !canonical_digest(&source.sha256) || source.sha256 != actual {
            refusals.push(OwnerProseRefusal::new(
                OwnerProseRefusalKind::SourceDigestMismatch,
                &source.path,
                "source digest does not match the complete source blob",
            ));
        }
        let mut state = ClaimState {
            source_paths: &expected_paths,
            claim_ids: &mut claim_ids,
            candidate_blobs: &mut candidate_blobs,
            intervals: &mut intervals,
            refusals: &mut refusals,
            qualified: &mut qualified,
        };
        validate_claims(source, &bytes, &mut read_blob, &mut state);
    }
    reject_duplicate_projections(&mut intervals, &mut refusals);
    if !refusals.is_empty() {
        return OwnerProseQualification::unknown(refusals);
    }

    let candidate_digests = candidate_blobs
        .into_iter()
        .map(|(path, bytes)| OwnerProsePathDigest {
            path,
            sha256: owner_prose_sha256(&bytes),
        })
        .collect();
    qualified.sort_by(|left, right| {
        (&left.source_path, left.start, left.end, &left.id).cmp(&(
            &right.source_path,
            right.start,
            right.end,
            &right.id,
        ))
    });
    OwnerProseQualification::Ready(Box::new(QualifiedOwnerProseView {
        schema: OWNER_PROSE_QUALIFIED_VIEW_SCHEMA.to_owned(),
        repository: observed.clone(),
        producer: manifest.producer,
        qualifier: OwnerProseProducer {
            identity: OWNER_PROSE_QUALIFIER_IDENTITY.to_owned(),
            schema: OWNER_PROSE_QUALIFIER_SCHEMA.to_owned(),
        },
        owner: manifest.owner,
        input_manifest_sha256: owner_prose_sha256(manifest_bytes),
        source_digests,
        candidate_digests,
        claims: qualified,
    }))
}

fn validate_envelope(
    manifest: &OwnerProseManifest,
    observed: &OwnerProseRepositoryBinding,
) -> Vec<OwnerProseRefusal> {
    let mut refusals = Vec::new();
    if manifest.schema != OWNER_PROSE_CLASSIFICATION_SCHEMA {
        refusals.push(OwnerProseRefusal::new(
            OwnerProseRefusalKind::SchemaMismatch,
            "owner-prose view",
            format!("unsupported schema {:?}", manifest.schema),
        ));
    }
    if manifest.repository != *observed || !valid_binding(observed) {
        refusals.push(OwnerProseRefusal::new(
            OwnerProseRefusalKind::RepositoryBindingMismatch,
            "owner-prose view",
            "repository identity or exact source/candidate commit/tree differs from observation",
        ));
    }
    if manifest.producer.identity != OWNER_PROSE_CLASSIFIER_IDENTITY
        || manifest.producer.schema != OWNER_PROSE_PRODUCER_SCHEMA
    {
        refusals.push(OwnerProseRefusal::new(
            OwnerProseRefusalKind::ProducerInvalid,
            "owner-prose view",
            "producer identity and schema must equal the supported classifier contract",
        ));
    }
    if !valid_owner(&manifest.owner) {
        refusals.push(OwnerProseRefusal::new(
            OwnerProseRefusalKind::OwnerInvalid,
            &manifest.owner,
            "owner must be one canonical capability or app/<product> path",
        ));
    }
    refusals
}

fn valid_binding(binding: &OwnerProseRepositoryBinding) -> bool {
    !binding.identity.trim().is_empty()
        && [
            &binding.source.commit,
            &binding.source.tree,
            &binding.candidate.commit,
            &binding.candidate.tree,
        ]
        .into_iter()
        .all(|value| valid_object_id(value))
}

fn valid_owner(owner: &str) -> bool {
    owner == "base"
        || crate::layout::is_capability_root(owner)
        || owner
            .strip_prefix("app/")
            .is_some_and(|product| crate::layout::APP_PRODUCT_DIRS.contains(&product))
}

fn valid_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn verify_deleted_in_candidate<F>(
    source: &super::OwnerProseSource,
    read_blob: &mut F,
    refusals: &mut Vec<OwnerProseRefusal>,
) where
    F: FnMut(OwnerProseRevision, &str) -> Result<Option<Vec<u8>>, String>,
{
    match read_blob(OwnerProseRevision::Candidate, &source.path) {
        Ok(None) => {}
        Ok(Some(_)) => refusals.push(OwnerProseRefusal::new(
            OwnerProseRefusalKind::AtomicDeletionIncomplete,
            &source.path,
            "fixed owner-law source remains in the candidate tree",
        )),
        Err(error) => refusals.push(OwnerProseRefusal::new(
            OwnerProseRefusalKind::RepositoryReadFailed,
            &source.path,
            error,
        )),
    }
}

fn reject_duplicate_projections(
    intervals: &mut ProjectionIntervals,
    refusals: &mut Vec<OwnerProseRefusal>,
) {
    for (path, ranges) in intervals {
        ranges.sort();
        for pair in ranges.windows(2) {
            if pair[1].0 < pair[0].1 {
                refusals.push(OwnerProseRefusal::new(
                    OwnerProseRefusalKind::DuplicateProjection,
                    path,
                    format!(
                        "claims {:?} and {:?} project overlapping byte ranges",
                        pair[0].2, pair[1].2
                    ),
                ));
            }
        }
    }
}

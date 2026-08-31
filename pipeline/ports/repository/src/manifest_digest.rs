use crate::{DigestBuilder, Entry, EvidenceDigest, ResolvedRevision};

pub(crate) fn entries_digest(entries: &[Entry]) -> EvidenceDigest {
    let mut digest = DigestBuilder::new(b"pipeline-repository-manifest-entries-v1");
    digest.push_u64(entries.len() as u64);
    for entry in entries {
        entry.path().digest_into(&mut digest);
        entry.state().digest_into(&mut digest);
    }
    digest.finish()
}

pub(crate) fn manifest_digest(
    revision: ResolvedRevision,
    entries: EvidenceDigest,
) -> EvidenceDigest {
    let mut digest = DigestBuilder::new(b"pipeline-repository-manifest-v2");
    revision.digest_into(&mut digest);
    digest.push_bytes(entries.as_bytes());
    digest.finish()
}

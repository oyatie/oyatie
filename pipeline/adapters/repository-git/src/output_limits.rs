use pipeline_repository::{ObjectAlgorithm, SnapshotLimits};

const AMBIGUOUS_MERGE_BASE_RECORDS: u64 = 2;
const RESOLVED_REVISION_OBJECTS: u64 = 6;
const MANIFEST_ENTRY_FRAMING_BYTES: u64 = 8;
const MAX_U64_DECIMAL_DIGITS: u64 = 20;

pub(crate) fn merge_base_stdout_limit(algorithm: ObjectAlgorithm, limits: SnapshotLimits) -> u64 {
    let record_bytes = algorithm.hex_digits() as u64 + b"\n".len() as u64;
    limits
        .max_stdout_bytes()
        .min(record_bytes.saturating_mul(AMBIGUOUS_MERGE_BASE_RECORDS))
}

pub(crate) fn resolved_objects_stdout_limit(
    algorithm: ObjectAlgorithm,
    limits: SnapshotLimits,
) -> u64 {
    let record_bytes = algorithm.hex_digits() as u64 + b" commit\n".len() as u64;
    limits
        .max_stdout_bytes()
        .min(record_bytes.saturating_mul(RESOLVED_REVISION_OBJECTS))
}

pub(crate) fn tree_stdout_limit(algorithm: ObjectAlgorithm, limits: SnapshotLimits) -> u64 {
    let git_framing_bytes = b"160000 commit \t\0".len() as u64;
    let expansion = algorithm.hex_digits() as u64 - algorithm.digest_bytes() as u64
        + git_framing_bytes
        - MANIFEST_ENTRY_FRAMING_BYTES;
    let operation_limit = limits
        .max_manifest_bytes()
        .saturating_add(limits.max_entries().saturating_mul(expansion));
    limits.max_stdout_bytes().min(operation_limit)
}

pub(crate) fn content_stdout_limit(
    algorithm: ObjectAlgorithm,
    selected: usize,
    limits: SnapshotLimits,
) -> u64 {
    let records = u64::try_from(selected).unwrap_or(u64::MAX);
    let record_overhead = algorithm.hex_digits() as u64
        + b" blob ".len() as u64
        + MAX_U64_DECIMAL_DIGITS
        + b"\n\n".len() as u64;
    let operation_limit = limits
        .max_total_content_bytes()
        .saturating_add(records.saturating_mul(record_overhead));
    limits.max_stdout_bytes().min(operation_limit)
}

#[cfg(test)]
mod tests {
    use pipeline_repository::SnapshotLimitSpec;

    use super::*;

    fn limits() -> SnapshotLimits {
        SnapshotLimits::new(limit_spec()).unwrap()
    }

    fn limit_spec() -> SnapshotLimitSpec {
        SnapshotLimitSpec {
            max_entries: 10,
            max_path_bytes: 20,
            max_manifest_bytes: 300,
            max_selected_contents: 4,
            max_content_bytes: 50,
            max_total_content_bytes: 60,
            max_stdout_bytes: 1_000_000,
            max_stderr_bytes: 8,
        }
    }

    #[test]
    fn each_git_operation_has_its_own_stdout_bound() {
        let limits = limits();

        assert_eq!(merge_base_stdout_limit(ObjectAlgorithm::Sha1, limits), 82);
        assert_eq!(
            resolved_objects_stdout_limit(ObjectAlgorithm::Sha1, limits),
            288
        );
        assert_eq!(tree_stdout_limit(ObjectAlgorithm::Sha1, limits), 580);
        assert_eq!(content_stdout_limit(ObjectAlgorithm::Sha1, 4, limits), 332);
        assert_eq!(
            merge_base_stdout_limit(ObjectAlgorithm::Sha256, limits),
            130
        );
        assert_eq!(
            resolved_objects_stdout_limit(ObjectAlgorithm::Sha256, limits),
            432
        );
        assert_eq!(tree_stdout_limit(ObjectAlgorithm::Sha256, limits), 700);
        assert_eq!(
            content_stdout_limit(ObjectAlgorithm::Sha256, 4, limits),
            428
        );
    }

    #[test]
    fn profile_stdout_limit_remains_the_outer_ceiling() {
        let limits = SnapshotLimits::new(SnapshotLimitSpec {
            max_stdout_bytes: 64,
            ..limit_spec()
        })
        .unwrap();

        assert_eq!(merge_base_stdout_limit(ObjectAlgorithm::Sha1, limits), 64);
        assert_eq!(
            resolved_objects_stdout_limit(ObjectAlgorithm::Sha1, limits),
            64
        );
        assert_eq!(tree_stdout_limit(ObjectAlgorithm::Sha1, limits), 64);
        assert_eq!(content_stdout_limit(ObjectAlgorithm::Sha1, 4, limits), 64);
    }
}

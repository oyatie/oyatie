const MAX_PROVIDER_SOURCE_FILES_V1: usize = 4_096;
const MAX_PROVIDER_SOURCE_FILE_BYTES_V1: usize = 8 * 1024 * 1024;
const MAX_PROVIDER_SOURCE_TREE_BYTES_V1: usize = 64 * 1024 * 1024;
const MAX_PROVIDER_SOURCE_PATH_BYTES_V1: usize = 1_024;
const MAX_PROVIDER_SOURCE_REVISION_BYTES_V1: usize = 128;

const REINDEER_SOURCE_TREE_SHA256_V1: ReindeerProviderDigestV1 =
    ReindeerProviderDigestV1([
        0xd4, 0x64, 0x4d, 0xb6, 0xbe, 0xe4, 0xfc, 0xe0, 0x64, 0x25, 0xc6, 0x80, 0x2d, 0xfc,
        0x5b, 0x3c, 0x2d, 0x2a, 0x12, 0xba, 0x93, 0xea, 0x3d, 0x63, 0x5e, 0x07, 0x67, 0x00,
        0xbc, 0x34, 0xd6, 0x14,
    ]);

fn canonical_reindeer_provider_source_snapshot_v1(
    source_revision: String,
    mut files: Vec<ReindeerProviderSourceFileV1>,
) -> Result<ReindeerProviderSourceSnapshotV1, ReindeerProviderAdaptationErrorV1> {
    if source_revision.is_empty()
        || source_revision.len() > MAX_PROVIDER_SOURCE_REVISION_BYTES_V1
        || source_revision.chars().any(char::is_control)
        || files.is_empty()
        || files.len() > MAX_PROVIDER_SOURCE_FILES_V1
    {
        return Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch);
    }
    files.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    if files
        .windows(2)
        .any(|pair| pair[0].path == pair[1].path)
    {
        return Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch);
    }

    let mut total_bytes = 0_usize;
    for file in &files {
        if !canonical_reindeer_source_path_v1(&file.path) {
            return Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch);
        }
        if file.bytes.len() > MAX_PROVIDER_SOURCE_FILE_BYTES_V1 {
            return Err(ReindeerProviderAdaptationErrorV1::SourceTooLarge);
        }
        total_bytes = total_bytes
            .checked_add(file.bytes.len())
            .filter(|total| *total <= MAX_PROVIDER_SOURCE_TREE_BYTES_V1)
            .ok_or(ReindeerProviderAdaptationErrorV1::SourceTooLarge)?;
    }

    let source_tree_sha256 = reindeer_provider_source_tree_digest_v1(&files)?;
    let total_bytes = u64::try_from(total_bytes)
        .map_err(|_| ReindeerProviderAdaptationErrorV1::SourceTooLarge)?;
    Ok(ReindeerProviderSourceSnapshotV1 {
        source_revision,
        files: files.into_boxed_slice(),
        source_tree_sha256,
        total_bytes,
    })
}

fn canonical_reindeer_source_path_v1(path: &str) -> bool {
    if path.is_empty()
        || path.len() > MAX_PROVIDER_SOURCE_PATH_BYTES_V1
        || path.starts_with('/')
        || path.ends_with('/')
        || path.bytes().any(|byte| byte.is_ascii_control() || byte == b'\\')
    {
        return false;
    }
    path.split('/')
        .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn reindeer_provider_source_tree_digest_v1(
    files: &[ReindeerProviderSourceFileV1],
) -> Result<ReindeerProviderDigestV1, ReindeerProviderAdaptationErrorV1> {
    let mut hash = Sha256::new();
    hash.update(b"build.reindeer-provider-source-tree.v1\0");
    hash.update(
        u64::try_from(files.len())
            .map_err(|_| ReindeerProviderAdaptationErrorV1::SourceTooLarge)?
            .to_be_bytes(),
    );
    for file in files {
        hash_reindeer_source_bytes_v1(&mut hash, file.path.as_bytes())?;
        hash.update([file.mode as u8]);
        hash_reindeer_source_bytes_v1(&mut hash, &file.bytes)?;
    }
    Ok(ReindeerProviderDigestV1(hash.finalize().into()))
}

fn hash_reindeer_source_bytes_v1(
    hash: &mut Sha256,
    bytes: &[u8],
) -> Result<(), ReindeerProviderAdaptationErrorV1> {
    hash.update(
        u64::try_from(bytes.len())
            .map_err(|_| ReindeerProviderAdaptationErrorV1::SourceTooLarge)?
            .to_be_bytes(),
    );
    hash.update(bytes);
    Ok(())
}

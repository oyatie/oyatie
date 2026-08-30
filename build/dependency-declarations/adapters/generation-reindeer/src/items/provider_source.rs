const REINDEER_PROVIDER_SOURCE_PATHS_V1: [&str; 9] = [
    "src/artifact.rs",
    "src/artifact/serializer.rs",
    "src/artifact/serializer/builders.rs",
    "src/artifact/value.rs",
    "src/buck.rs",
    "src/buckify.rs",
    "src/index.rs",
    "src/main.rs",
    "src/version_naming.rs",
];
const REINDEER_PROVIDER_GENERATED_PATHS_V1: [&str; 4] = [
    "src/artifact.rs",
    "src/artifact/serializer.rs",
    "src/artifact/serializer/builders.rs",
    "src/artifact/value.rs",
];
const MAX_PROVIDER_OUTPUT_BYTES_V1: usize = 4 * 1024 * 1024;
const REINDEER_SOURCE_REPOSITORY_V1: &str = "https://github.com/facebookincubator/reindeer";
const REINDEER_SOURCE_TAG_V1: &str = "v2026.08.10.00";
const REINDEER_ADAPTATION_RECIPE_ID_V1: &str = concat!(
    "build.reindeer-provider-source-recipe.v1;",
    "syn=2.0.119@872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297;",
    "prettyplease=0.2.37@479ca8adacdd7ce8f1fb39ce9ecccbfe93a3f1344b3d0d97f20bc0196208f62b;",
    "proc-macro2=1.0.107@985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9;",
    "quote=1.0.47@1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001;",
    "sha2=0.10.9@a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283;",
    "public-naming=resolved-compatibility-slot.v1;",
    "reserved-targets=source-distinct.v1;",
    "workspace-roots=non-workspace-public-targets.v1",
);

struct ReindeerParsedProviderSourceV1<'a> {
    bytes: &'a [u8],
    text: &'a str,
    syntax: syn::File,
}

/// Adapts one exact, immutable Reindeer source snapshot as a whole batch.
pub fn adapt_reindeer_provider_source_v1(
    snapshot: &ReindeerProviderSourceSnapshotV1,
) -> Result<ReindeerProviderSourceAdaptationV1, ReindeerProviderAdaptationErrorV1> {
    if snapshot.source_revision() != PINNED_SOURCE_REVISION {
        return Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceRevision);
    }
    let files = canonical_provider_source_batch_v1(&snapshot.files)?;
    let buck = parse_exact_provider_source_v1(&files, "src/buck.rs", REINDEER_BUCK_SHA256_V1)?;
    let buckify =
        parse_exact_provider_source_v1(&files, "src/buckify.rs", REINDEER_BUCKIFY_SHA256_V1)?;
    let index = parse_exact_provider_source_v1(&files, "src/index.rs", REINDEER_INDEX_SHA256_V1)?;
    let main = parse_exact_provider_source_v1(&files, "src/main.rs", REINDEER_MAIN_SHA256_V1)?;
    let version_naming = parse_exact_provider_source_v1(
        &files,
        "src/version_naming.rs",
        REINDEER_VERSION_NAMING_SHA256_V1,
    )?;
    let schema = inspect_reindeer_provider_schema_syntax_v1(buck.bytes, &buck.syntax)?;
    if snapshot.source_tree_sha256() != REINDEER_SOURCE_TREE_SHA256_V1 {
        return Err(ReindeerProviderAdaptationErrorV1::SourceTreeMismatch);
    }

    let generated = BTreeMap::from([
        (
            "src/artifact.rs",
            render_reindeer_artifact_root_v1(&schema)?,
        ),
        (
            "src/artifact/serializer.rs",
            render_reindeer_artifact_serializer_v1()?,
        ),
        (
            "src/artifact/serializer/builders.rs",
            render_reindeer_artifact_builders_v1()?,
        ),
        (
            "src/artifact/value.rs",
            render_reindeer_artifact_value_v1()?,
        ),
    ]);
    let transformed = BTreeMap::from([
        (
            "src/buck.rs",
            adapt_reindeer_buck_v1(buck.text, &buck.syntax).map_err(|error| {
                provider_shape_context_v1(
                    error,
                    ReindeerProviderAdaptationErrorV1::UnsupportedBuckSourceShape,
                )
            })?,
        ),
        (
            "src/buckify.rs",
            adapt_reindeer_buckify_v1(buckify.text, &buckify.syntax).map_err(|error| {
                provider_shape_context_v1(
                    error,
                    ReindeerProviderAdaptationErrorV1::UnsupportedBuckifySourceShape,
                )
            })?,
        ),
        (
            "src/index.rs",
            adapt_reindeer_index_v1(index.text, &index.syntax).map_err(|error| {
                provider_shape_context_v1(
                    error,
                    ReindeerProviderAdaptationErrorV1::UnsupportedIndexSourceShape,
                )
            })?,
        ),
        (
            "src/main.rs",
            adapt_reindeer_main_v1(main.text, &main.syntax).map_err(|error| {
                provider_shape_context_v1(
                    error,
                    ReindeerProviderAdaptationErrorV1::UnsupportedMainSourceShape,
                )
            })?,
        ),
        (
            "src/version_naming.rs",
            adapt_reindeer_version_naming_v1(version_naming.text, &version_naming.syntax)
                .map_err(|error| {
                    provider_shape_context_v1(
                        error,
                        ReindeerProviderAdaptationErrorV1::UnsupportedVersionNamingSourceShape,
                    )
                })?,
        ),
    ]);

    let mut adapted = Vec::with_capacity(REINDEER_PROVIDER_SOURCE_PATHS_V1.len());
    for path in REINDEER_PROVIDER_SOURCE_PATHS_V1 {
        let postimage = generated
            .get(path)
            .or_else(|| transformed.get(path))
            .ok_or(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch)?;
        if postimage.len() > MAX_PROVIDER_OUTPUT_BYTES_V1 {
            return Err(ReindeerProviderAdaptationErrorV1::OutputTooLarge);
        }
        let preimage = files
            .get(path)
            .map(|file| file.bytes.to_vec())
            .map(Vec::into_boxed_slice);
        let preimage_sha256 = preimage.as_deref().map(ReindeerProviderDigestV1::of);
        adapted.push(ReindeerProviderAdaptedFileV1 {
            path: path.to_owned(),
            preimage,
            preimage_sha256,
            postimage: postimage.clone().into_boxed_slice(),
            postimage_sha256: ReindeerProviderDigestV1::of(postimage),
        });
    }

    let adapted_batch_sha256 = provider_adapted_batch_digest_v1(&adapted)?;
    let source_tree_sha256 = snapshot.source_tree_sha256();
    let receipt_sha256 =
        provider_adaptation_receipt_v1(source_tree_sha256, &schema, adapted_batch_sha256)?;
    Ok(ReindeerProviderSourceAdaptationV1 {
        source_tree_sha256,
        adapted_batch_sha256,
        schema,
        files: adapted.into_boxed_slice(),
        receipt_sha256,
    })
}

fn provider_shape_context_v1(
    error: ReindeerProviderAdaptationErrorV1,
    shape_error: ReindeerProviderAdaptationErrorV1,
) -> ReindeerProviderAdaptationErrorV1 {
    if error == ReindeerProviderAdaptationErrorV1::UnsupportedSourceShape {
        shape_error
    } else {
        error
    }
}

fn canonical_provider_source_batch_v1(
    files: &[ReindeerProviderSourceFileV1],
) -> Result<BTreeMap<&str, &ReindeerProviderSourceFileV1>, ReindeerProviderAdaptationErrorV1> {
    let mut by_path = BTreeMap::new();
    for file in files {
        if by_path.insert(file.path.as_str(), file).is_some() {
            return Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch);
        }
    }
    for path in REINDEER_PROVIDER_GENERATED_PATHS_V1 {
        if by_path.contains_key(path) {
            return Err(ReindeerProviderAdaptationErrorV1::SourcePresenceMismatch);
        }
    }
    for path in [
        "src/buck.rs",
        "src/buckify.rs",
        "src/index.rs",
        "src/main.rs",
        "src/version_naming.rs",
    ] {
        if !by_path.contains_key(path) {
            return Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch);
        }
    }
    Ok(by_path)
}

fn parse_exact_provider_source_v1<'a>(
    files: &BTreeMap<&str, &'a ReindeerProviderSourceFileV1>,
    path: &str,
    expected_sha256: &str,
) -> Result<ReindeerParsedProviderSourceV1<'a>, ReindeerProviderAdaptationErrorV1> {
    let bytes = files
        .get(path)
        .ok_or(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch)?
        .bytes();
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err(ReindeerProviderAdaptationErrorV1::SourceTooLarge);
    }
    if format!("{:x}", Sha256::digest(bytes)) != expected_sha256 {
        return Err(ReindeerProviderAdaptationErrorV1::SourceFileDigestMismatch);
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| ReindeerProviderAdaptationErrorV1::InvalidUtf8)?;
    let syntax =
        syn::parse_file(text).map_err(|_| ReindeerProviderAdaptationErrorV1::InvalidRust)?;
    Ok(ReindeerParsedProviderSourceV1 {
        bytes,
        text,
        syntax,
    })
}

fn provider_adapted_batch_digest_v1(
    files: &[ReindeerProviderAdaptedFileV1],
) -> Result<ReindeerProviderDigestV1, ReindeerProviderAdaptationErrorV1> {
    let mut hash = Sha256::new();
    hash.update(b"build.reindeer-provider-adapted-batch.v1\0");
    for file in files {
        hash_string(&mut hash, &file.path)?;
        match file.preimage_sha256 {
            Some(digest) => {
                hash.update([1]);
                hash.update(digest.0);
            }
            None => hash.update([0]),
        }
        hash.update(file.postimage_sha256.0);
        let length = u64::try_from(file.postimage.len())
            .map_err(|_| ReindeerProviderAdaptationErrorV1::OutputTooLarge)?;
        hash.update(length.to_be_bytes());
    }
    Ok(ReindeerProviderDigestV1(hash.finalize().into()))
}

fn provider_adaptation_receipt_v1(
    source_tree_sha256: ReindeerProviderDigestV1,
    schema: &ReindeerProviderSchemaV1,
    adapted_batch_sha256: ReindeerProviderDigestV1,
) -> Result<ReindeerProviderDigestV1, ReindeerProviderAdaptationErrorV1> {
    let mut hash = Sha256::new();
    hash.update(b"build.reindeer-provider-adaptation.v1\0");
    for identity in [
        REINDEER_SOURCE_REPOSITORY_V1,
        REINDEER_SOURCE_TAG_V1,
        PINNED_SOURCE_REVISION,
        REINDEER_ADAPTATION_RECIPE_ID_V1,
    ] {
        hash_string(&mut hash, identity)?;
    }
    hash.update(source_tree_sha256.0);
    hash.update(schema.semantic_schema_sha256.0);
    hash.update(adapted_batch_sha256.0);
    Ok(ReindeerProviderDigestV1(hash.finalize().into()))
}

const REINDEER_BUCK_SHA256_V1: &str =
    "49d79a30a880c042f3c383b6b5d17d3152caacbf82e402ab7d1875087e56237b";
const REINDEER_BUCKIFY_SHA256_V1: &str =
    "6d09d2b7a51b7fca101d2fbd356d96e626467a8b8b02090747eb3979d4f61ecf";
const REINDEER_INDEX_SHA256_V1: &str =
    "23546695e322a9d86298f6aeb38abbeff4e10503674fca251eb153917beb6689";
const REINDEER_MAIN_SHA256_V1: &str =
    "2b53f3680985fec0974441ad37b80397ca3cc85e52c259917af15277ec874a27";
const REINDEER_VERSION_NAMING_SHA256_V1: &str =
    "547603f2df2e163a12d719c290d94e14f56bdaa2451208f093f06d469aab0415";

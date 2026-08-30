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
    let fixups =
        parse_exact_provider_source_v1(&files, "src/fixups.rs", REINDEER_FIXUPS_SHA256_V1)?;
    let fixup_buildscript = parse_exact_provider_source_v1(
        &files,
        "src/fixups/buildscript.rs",
        REINDEER_FIXUP_BUILDSCRIPT_SHA256_V1,
    )?;
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
    let adapted_buck = adapt_reindeer_buck_v1(buck.text, &buck.syntax).map_err(|error| {
        provider_shape_context_v1(
            error,
            ReindeerProviderAdaptationErrorV1::UnsupportedBuckSourceShape,
        )
    })?;
    let adapted_buck = adapt_reindeer_cxx_rule_v1(&adapted_buck).map_err(|error| {
        provider_shape_context_v1(
            error,
            ReindeerProviderAdaptationErrorV1::UnsupportedBuckSourceShape,
        )
    })?;
    let transformed = BTreeMap::from([
        (
            "src/buck.rs",
            adapted_buck,
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
            "src/fixups.rs",
            adapt_reindeer_cxx_fixup_application_v1(fixups.text).map_err(|error| {
                provider_shape_context_v1(
                    error,
                    ReindeerProviderAdaptationErrorV1::UnsupportedFixupsSourceShape,
                )
            })?,
        ),
        (
            "src/fixups/buildscript.rs",
            adapt_reindeer_cxx_fixup_schema_v1(fixup_buildscript.text).map_err(|error| {
                provider_shape_context_v1(
                    error,
                    ReindeerProviderAdaptationErrorV1::UnsupportedFixupSchemaSourceShape,
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
        "src/fixups.rs",
        "src/fixups/buildscript.rs",
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

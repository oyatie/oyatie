use super::*;

pub(super) fn release_source_for_module(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<String, CloudIacReleaseIndexError> {
    if let Some(location) = &module.archive_source_location {
        return Ok(location.clone());
    }
    Ok(format!(
        "{CLOUD_IAC_APP_ARTIFACTS_BASE_PATH}{}",
        archive_file_name(&module.archive_file)?
    ))
}

pub(super) fn archive_file_name(path: &str) -> Result<String, CloudIacReleaseIndexError> {
    Path::new(path)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(str::to_string)
        .ok_or_else(|| CloudIacReleaseIndexError::InvalidField {
            field: "archive_file",
            reason: "archive path must include a UTF-8 filename".to_string(),
        })
}

pub(super) fn is_valid_archive_file_name(file_name: &str, version: &str) -> bool {
    if file_name.is_empty()
        || file_name == "."
        || file_name == ".."
        || !file_name.ends_with(".zip")
        || !file_name.ends_with(&format!("-{version}.zip"))
    {
        return false;
    }
    file_name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '.'))
}

pub(super) fn archive_artifacts_from_seed(
    seed: &CloudIacReleaseIndexSeed,
) -> Result<BTreeMap<String, CloudIacAppArchiveArtifact>, CloudIacReleaseIndexError> {
    let mut artifacts = BTreeMap::new();
    for module in seed.modules() {
        if module.archive_source_location.is_some() {
            continue;
        }
        let file_name = archive_file_name(&module.archive_file)?;
        artifacts.insert(
            file_name,
            CloudIacAppArchiveArtifact {
                archive_file: PathBuf::from(&module.archive_file),
                archive_sha256: module.archive_sha256.clone(),
                media_type: module.archive_media_type.clone(),
            },
        );
    }
    Ok(artifacts)
}

pub(super) fn contains_secret_like_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("token=")
        || lower.contains("password=")
        || lower.contains("secret=")
        || lower.contains("kubeconfig")
        || lower.contains("-----begin")
        || lower.contains("sk-live")
        || lower.contains("sk-")
}

mod repository_manifest_paths {
    use crate::repository_manifests::ManifestError;

    pub(super) fn relative(base: &str, path: &str) -> Result<String, ManifestError> {
        if path.starts_with('/') || path.contains(['\\', ':']) || path.chars().any(char::is_control)
        {
            return Err(ManifestError::InvalidPath(path.into()));
        }
        let mut parts: Vec<_> = base.split('/').filter(|part| !part.is_empty()).collect();
        for part in path.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    if parts.pop().is_none() {
                        return Err(ManifestError::InvalidPath(path.into()));
                    }
                }
                _ => parts.push(part),
            }
        }
        Ok(parts.join("/"))
    }
}

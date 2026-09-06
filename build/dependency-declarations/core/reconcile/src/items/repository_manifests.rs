pub mod repository_manifests {
    use crate::repository_manifest_paths::relative;
    use sha2::{Digest, Sha256};
    use std::collections::{BTreeMap, BTreeSet};
    use toml::{Table, Value};

    #[derive(Clone, Copy, Debug)]
    pub struct ManifestInput<'a> {
        pub directory: &'a str,
        pub contents: &'a str,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Limits {
        pub manifests: usize,
        pub bytes: usize,
        pub edges: usize,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum ManifestError {
        LimitExceeded(&'static str),
        InvalidPath(String),
        DuplicateManifest(String),
        MissingManifest(String),
        InvalidManifest(String),
        MissingWorkspaceDependency(String),
        UnsupportedSurface(String),
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct ManifestPlan {
        pub dependencies: BTreeMap<String, BTreeSet<String>>,
        pub input_digest: [u8; 32],
    }

    /// Plans declared local path dependencies, without filesystem or Cargo-config resolution.
    pub fn plan(
        workspace: &str,
        inputs: &[ManifestInput<'_>],
        seeds: &[&str],
        limits: Limits,
    ) -> Result<ManifestPlan, ManifestError> {
        if inputs.len() > limits.manifests || seeds.len() > limits.manifests {
            return Err(ManifestError::LimitExceeded("manifests"));
        }
        let bytes = inputs
            .iter()
            .try_fold(workspace.len(), |total, input| {
                total
                    .checked_add(input.directory.len())?
                    .checked_add(input.contents.len())
            })
            .and_then(|total| {
                seeds
                    .iter()
                    .try_fold(total, |total, seed| total.checked_add(seed.len()))
            });
        if !bytes.is_some_and(|bytes| bytes <= limits.bytes) {
            return Err(ManifestError::LimitExceeded("bytes"));
        }
        let mut manifests = BTreeMap::new();
        for input in inputs {
            if relative("", input.directory)? != input.directory {
                return Err(ManifestError::InvalidPath(input.directory.into()));
            }
            if input.directory.is_empty() && input.contents != workspace {
                return Err(ManifestError::InvalidManifest(
                    "conflicting root manifest".into(),
                ));
            }
            if manifests.insert(input.directory, input.contents).is_some() {
                return Err(ManifestError::DuplicateManifest(input.directory.into()));
            }
        }
        let workspace_bytes = workspace;
        let workspace = parse(workspace, "Cargo.toml")?;
        let workspace_dependencies = workspace
            .get("workspace")
            .map(|value| table(value, "workspace"))
            .transpose()?
            .and_then(|workspace| workspace.get("dependencies"))
            .map(|value| table(value, "workspace.dependencies"))
            .transpose()?
            .cloned()
            .unwrap_or_default();
        let mut pending = BTreeSet::new();
        for seed in seeds {
            if relative("", seed)? != *seed {
                return Err(ManifestError::InvalidPath((*seed).into()));
            }
            pending.insert((*seed).to_owned());
        }
        if pending.is_empty() {
            return Err(ManifestError::InvalidManifest("empty seed set".into()));
        }
        let mut dependencies = BTreeMap::new();
        let mut edge_count = 0usize;
        while let Some(directory) = pending.pop_first() {
            if dependencies.contains_key(&directory) {
                continue;
            }
            let contents = manifests
                .get(directory.as_str())
                .ok_or_else(|| ManifestError::MissingManifest(directory.clone()))?;
            let document = parse(contents, &directory)?;
            if !directory.is_empty() && document.contains_key("workspace") {
                return Err(ManifestError::UnsupportedSurface("nested workspace".into()));
            }
            if let Some(package) = document.get("package")
                && let Some(workspace) = table(package, &directory)?.get("workspace")
            {
                let path = workspace
                    .as_str()
                    .ok_or_else(|| ManifestError::InvalidManifest(directory.clone()))?;
                if !relative(&directory, path)?.is_empty() {
                    return Err(ManifestError::UnsupportedSurface(
                        "workspace redirect".into(),
                    ));
                }
            }
            let edges = local_dependencies(&document, &workspace_dependencies, &directory)?;
            edge_count = edge_count
                .checked_add(edges.len())
                .filter(|count| *count <= limits.edges)
                .ok_or(ManifestError::LimitExceeded("edges"))?;
            pending.extend(edges.iter().cloned());
            dependencies.insert(directory, edges);
        }
        let mut digest = Sha256::new();
        digest.update(b"oyatie-repository-manifest-inputs-v1\0");
        hash_field(&mut digest, workspace_bytes);
        digest.update((manifests.len() as u64).to_le_bytes());
        for (directory, contents) in manifests {
            hash_field(&mut digest, directory);
            hash_field(&mut digest, contents);
        }
        let seeds: BTreeSet<_> = seeds.iter().collect();
        digest.update((seeds.len() as u64).to_le_bytes());
        for seed in seeds {
            hash_field(&mut digest, seed);
        }
        Ok(ManifestPlan {
            dependencies,
            input_digest: digest.finalize().into(),
        })
    }

    fn hash_field(digest: &mut Sha256, value: &str) {
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }

    fn parse(contents: &str, location: &str) -> Result<Table, ManifestError> {
        let document: Table = contents
            .parse()
            .map_err(|_| ManifestError::InvalidManifest(location.into()))?;
        for surface in ["patch", "replace"] {
            if document.contains_key(surface) {
                return Err(ManifestError::UnsupportedSurface(surface.into()));
            }
        }
        Ok(document)
    }

    fn table<'a>(value: &'a Value, location: &str) -> Result<&'a Table, ManifestError> {
        value
            .as_table()
            .ok_or_else(|| ManifestError::InvalidManifest(location.into()))
    }

    fn local_dependencies(
        document: &Table,
        workspace: &Table,
        directory: &str,
    ) -> Result<BTreeSet<String>, ManifestError> {
        let mut sections = vec![document];
        if let Some(targets) = document.get("target") {
            for target in table(targets, directory)?.values() {
                sections.push(table(target, directory)?);
            }
        }
        let mut edges = BTreeSet::new();
        for section in sections {
            for unsupported in ["dev_dependencies", "build_dependencies"] {
                if section.contains_key(unsupported) {
                    return Err(ManifestError::UnsupportedSurface(unsupported.into()));
                }
            }
            for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
                let Some(dependencies) = section.get(kind) else {
                    continue;
                };
                for (name, dependency) in table(dependencies, directory)? {
                    if dependency.is_str() {
                        continue;
                    }
                    let specification = table(dependency, directory)?;
                    let inherited = match specification.get("workspace") {
                        None | Some(Value::Boolean(false)) => false,
                        Some(Value::Boolean(true)) => true,
                        _ => return Err(ManifestError::InvalidManifest(directory.into())),
                    };
                    let (base, specification) = if inherited {
                        if specification.contains_key("path") {
                            return Err(ManifestError::InvalidManifest(directory.into()));
                        }
                        let value = workspace.get(name).ok_or_else(|| {
                            ManifestError::MissingWorkspaceDependency(name.clone())
                        })?;
                        if value.is_str() {
                            continue;
                        }
                        ("", table(value, "workspace.dependencies")?)
                    } else {
                        (directory, specification)
                    };
                    if let Some(path) = specification.get("path") {
                        let path = path
                            .as_str()
                            .ok_or_else(|| ManifestError::InvalidManifest(directory.into()))?;
                        edges.insert(relative(base, path)?);
                    }
                }
            }
        }
        Ok(edges)
    }
}

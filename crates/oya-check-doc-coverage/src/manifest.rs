//! Localization-pack manifest parsing (`docs/localization-packs/<pack>/pack.yaml`).
//!
//! Per ADR-0064 §4 and ADR-0063 §2, `pack.yaml` is the source of truth for
//! (pack × µservice × material_scope) scope.

use anyhow::Result;
use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Deserialize)]
pub struct PackManifest {
    pub pack: PackMeta,
    #[serde(default)]
    pub microservices_in_scope: Vec<MicroserviceScope>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackMeta {
    pub code: String,
    pub name: String,
    pub status: String, // planned | active | maintained | retired
    #[serde(default)]
    pub foundational: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MicroserviceScope {
    pub microservice: String,
    #[serde(default)]
    pub material_scope: bool,
    #[serde(default)]
    pub lead_milestone: Option<String>,
}

/// Discover every pack manifest under `docs/localization-packs/<pack>/pack.yaml`.
pub fn read_pack_catalog(repo_root: &Path) -> Result<Vec<PackManifest>> {
    let dir = repo_root.join("docs/localization-packs");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut packs = Vec::new();
    for entry in WalkDir::new(&dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|r| r.ok())
    {
        if entry.file_name() == "pack.yaml" {
            let content = std::fs::read_to_string(entry.path())?;
            let manifest: PackManifest = serde_yaml::from_str(&content)?;
            packs.push(manifest);
        }
    }
    Ok(packs)
}

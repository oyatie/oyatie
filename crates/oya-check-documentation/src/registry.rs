//! Workspace-metadata and MASTERPLAN catalog parsing.
//!
//! - Registered set: `[workspace.metadata.oya.microservices]` in root `Cargo.toml`
//! - Planned set: §2.1 catalog enumeration in `docs/MASTERPLAN.md`

use anyhow::Result;
use std::path::Path;

/// Read registered µservices from the workspace `Cargo.toml`'s
/// `[workspace.metadata.oya.microservices]` table.
pub fn read_workspace_microservices(repo_root: &Path) -> Result<Vec<String>> {
    let cargo_toml = repo_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)?;
    let parsed: toml::Value = toml::from_str(&content)?;
    let names = parsed
        .get("workspace")
        .and_then(|w| w.get("metadata"))
        .and_then(|m| m.get("oya"))
        .and_then(|o| o.get("microservices"))
        .and_then(|s| s.as_table())
        .map(|t| t.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    Ok(names)
}

/// Read the planned-µservice catalog from `docs/MASTERPLAN.md` §2.1.
///
/// Parses the fenced code block under `### 2.1 Flat µservice catalog`.
/// Returns kebab-case µservice tokens. Best-effort: the masterplan format
/// is markdown so this is a heuristic extractor, not a strict parser.
pub fn read_masterplan_catalog(repo_root: &Path) -> Result<Vec<String>> {
    let path = repo_root.join("docs/MASTERPLAN.md");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    let mut names = Vec::new();
    let mut in_catalog = false;
    for line in content.lines() {
        if line.contains("Flat catalog") || line.contains("Substrate µservices") {
            in_catalog = true;
            continue;
        }
        if in_catalog && (line.starts_with("```") || line.contains("Connect Personal")) {
            // closing fence or end of catalog block
            if line.starts_with("```") {
                in_catalog = false;
            }
            continue;
        }
        if in_catalog {
            for tok in line.split(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
                let t = tok.trim();
                if t.len() >= 2
                    && t.chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                {
                    names.push(t.to_string());
                }
            }
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

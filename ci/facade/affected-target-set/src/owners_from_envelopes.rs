//! # owners-from-envelopes
//!
//! Generate Google-style directory `OWNERS` and GitHub `CODEOWNERS` text from
//! `specs/integ-branch-envelopes.json` (`#roots` / `#planes` + `#path_ownership`).
//!
//! ## Forever shape
//! - **Team ≡** one `integ/<root>` envelope (`#path_ownership.team_equals_integ_envelope_root`).
//! - **Principal** = `integ/<name>` branch tail (self-explanatory; OWNERS schema: `[a-z0-9-]+`).
//! - **Emit faces** under this package (BAN live `.github/CODEOWNERS` flag-day replace here —
//!   quiet-window cutover). Freshness via `--check`.
//! - Cite envelopes JSON pointers; do not re-list roots in prose.
//!
//! Authority: living SSOT `domain_stack_integ_model` § Merge admission + domain green;
//! machine law `specs/integ-branch-envelopes.json#path_ownership`.
//!
//! Colocated in this crate to avoid Cargo.lock hub churn from a new package.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::Value;

/// Envelopes SSOT (cite; do not fork).
pub const ENVELOPES_RELPATH: &str = "specs/integ-branch-envelopes.json";

/// Generated GitHub CODEOWNERS face (not the live `.github/CODEOWNERS` until cutover).
pub const EMIT_CODEOWNERS_RELPATH: &str =
    "ci/facade/affected-target-set/owners-from-envelopes/CODEOWNERS";

/// Generated directory-OWNERS map (prefix → principal line).
pub const EMIT_OWNERS_MAP_RELPATH: &str =
    "ci/facade/affected-target-set/owners-from-envelopes/OWNERS-by-prefix.json";

/// Default catch-all GitHub team (matches live CODEOWNERS hub owner until org remap).
pub const DEFAULT_CODEOWNERS_TEAM: &str = "@teams/council-architecture";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipEntry {
    /// Envelope key under `#roots` or `#planes`.
    pub envelope_id: String,
    /// Durable integ branch (`integ/messaging`, …).
    pub branch: String,
    /// OWNERS principal (branch tail).
    pub principal: String,
    /// One envelope glob (`compute/**`, `AGENTS.md`, …).
    pub glob: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedOwners {
    /// GitHub CODEOWNERS document body.
    pub codeowners: String,
    /// Directory prefix (`compute/`) → OWNERS file body (`messaging\n`).
    pub owners_by_prefix: BTreeMap<String, String>,
    pub entries: Vec<OwnershipEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnersFromEnvelopesError {
    MissingObject {
        field: String,
    },
    InvalidGlob {
        envelope_id: String,
        glob: String,
    },
    InvalidPrincipal {
        envelope_id: String,
        principal: String,
    },
    EmptyEntries,
}

impl std::fmt::Display for OwnersFromEnvelopesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingObject { field } => write!(f, "envelopes missing object {field}"),
            Self::InvalidGlob { envelope_id, glob } => write!(
                f,
                "envelope {envelope_id}: unsupported ownership glob {glob:?}"
            ),
            Self::InvalidPrincipal {
                envelope_id,
                principal,
            } => write!(
                f,
                "envelope {envelope_id}: principal {principal:?} fails OWNERS schema"
            ),
            Self::EmptyEntries => write!(f, "no ownership entries derived from envelopes"),
        }
    }
}

/// OWNERS schema: lowercase alphanumeric + interior hyphens, 1..=63 chars.
pub fn is_valid_owner_principal(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes.len() > 63 {
        return false;
    }
    let alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    alnum(bytes[0]) && alnum(bytes[bytes.len() - 1]) && bytes.iter().all(|&b| alnum(b) || b == b'-')
}

/// Principal for an envelope entry: `integ/<name>` tail (self-explanatory).
pub fn principal_for_branch(envelope_id: &str, branch: &str) -> String {
    if let Some(tail) = branch.strip_prefix("integ/")
        && is_valid_owner_principal(tail)
    {
        return tail.to_owned();
    }
    envelope_id.replace('_', "-")
}

/// Map an envelope glob to a CODEOWNERS path pattern.
///
/// - `dir/**` → `dir/`
/// - exact file / path → unchanged (no leading `/`)
pub fn glob_to_codeowners_pattern(glob: &str) -> Option<String> {
    let glob = glob.trim();
    if glob.is_empty() || glob.starts_with('/') || glob.contains("..") {
        return None;
    }
    if let Some(stem) = glob.strip_suffix("/**") {
        if stem.is_empty() || stem.contains('*') {
            return None;
        }
        return Some(format!("{stem}/"));
    }
    if glob.contains('*') {
        // Only the measured `dir/**` star form is accepted for generation.
        return None;
    }
    Some(glob.to_owned())
}

/// Directory OWNERS apply only to `dir/**` envelope globs → prefix `dir/`.
pub fn glob_to_owners_prefix(glob: &str) -> Option<String> {
    let pattern = glob_to_codeowners_pattern(glob)?;
    if pattern.ends_with('/') {
        Some(pattern)
    } else {
        None
    }
}

fn collect_section(
    section: &Value,
    section_name: &str,
    out: &mut Vec<OwnershipEntry>,
) -> Result<(), OwnersFromEnvelopesError> {
    let object = section
        .as_object()
        .ok_or_else(|| OwnersFromEnvelopesError::MissingObject {
            field: section_name.to_owned(),
        })?;
    for (envelope_id, entry) in object {
        let branch = entry
            .get("branch")
            .and_then(Value::as_str)
            .unwrap_or(envelope_id);
        let principal = principal_for_branch(envelope_id, branch);
        if !is_valid_owner_principal(&principal) {
            return Err(OwnersFromEnvelopesError::InvalidPrincipal {
                envelope_id: envelope_id.clone(),
                principal,
            });
        }
        let Some(globs) = entry.get("envelope_globs").and_then(Value::as_array) else {
            continue;
        };
        for glob_value in globs {
            let Some(glob) = glob_value.as_str() else {
                return Err(OwnersFromEnvelopesError::InvalidGlob {
                    envelope_id: envelope_id.clone(),
                    glob: glob_value.to_string(),
                });
            };
            if glob_to_codeowners_pattern(glob).is_none() {
                return Err(OwnersFromEnvelopesError::InvalidGlob {
                    envelope_id: envelope_id.clone(),
                    glob: glob.to_owned(),
                });
            }
            out.push(OwnershipEntry {
                envelope_id: envelope_id.clone(),
                branch: branch.to_owned(),
                principal: principal.clone(),
                glob: glob.to_owned(),
            });
        }
    }
    Ok(())
}

/// Derive ownership entries from envelopes `#roots` + `#planes`.
pub fn collect_ownership_entries(
    envelopes: &Value,
) -> Result<Vec<OwnershipEntry>, OwnersFromEnvelopesError> {
    let mut entries = Vec::new();
    let roots = envelopes
        .get("roots")
        .ok_or_else(|| OwnersFromEnvelopesError::MissingObject {
            field: "roots".to_owned(),
        })?;
    collect_section(roots, "roots", &mut entries)?;
    if let Some(planes) = envelopes.get("planes") {
        collect_section(planes, "planes", &mut entries)?;
    }
    if entries.is_empty() {
        return Err(OwnersFromEnvelopesError::EmptyEntries);
    }
    // Stable: longer/more-specific globs last so CODEOWNERS last-match-wins favors specificity.
    entries.sort_by(|a, b| {
        let pa = glob_to_codeowners_pattern(&a.glob).unwrap_or_default();
        let pb = glob_to_codeowners_pattern(&b.glob).unwrap_or_default();
        pa.len()
            .cmp(&pb.len())
            .then_with(|| pa.cmp(&pb))
            .then_with(|| a.envelope_id.cmp(&b.envelope_id))
    });
    Ok(entries)
}

/// Render GitHub CODEOWNERS + directory OWNERS map.
pub fn generate_owners(envelopes: &Value) -> Result<GeneratedOwners, OwnersFromEnvelopesError> {
    let entries = collect_ownership_entries(envelopes)?;
    let mut codeowners = String::new();
    let _ = writeln!(
        codeowners,
        "# GENERATED by owners-from-envelopes — DO NOT HAND-EDIT."
    );
    let _ = writeln!(
        codeowners,
        "# Source: {ENVELOPES_RELPATH}#roots|#planes (+ #path_ownership)."
    );
    let _ = writeln!(
        codeowners,
        "# Living SSOT: domain_stack_integ_model § Merge admission + domain green."
    );
    let _ = writeln!(
        codeowners,
        "# Live GitHub path `.github/CODEOWNERS` cutover is a quiet-window land — this face is the emit SSOT."
    );
    let _ = writeln!(codeowners);
    let _ = writeln!(
        codeowners,
        "# Catch-all hub owner (logical @teams/ handle until org namespace remap)."
    );
    let _ = writeln!(codeowners, "* {DEFAULT_CODEOWNERS_TEAM}");
    let _ = writeln!(codeowners);

    let mut owners_by_prefix = BTreeMap::new();
    // Preserve length-ascending order from `entries` so CODEOWNERS last-match-wins
    // prefers more specific paths. Collapse duplicate patterns to last writer.
    let mut ordered_patterns: Vec<(String, String)> = Vec::new();
    let mut pattern_index: BTreeMap<String, usize> = BTreeMap::new();

    for entry in &entries {
        let pattern = glob_to_codeowners_pattern(&entry.glob).ok_or_else(|| {
            OwnersFromEnvelopesError::InvalidGlob {
                envelope_id: entry.envelope_id.clone(),
                glob: entry.glob.clone(),
            }
        })?;
        let team = format!("@teams/{}", entry.principal);
        if let Some(&idx) = pattern_index.get(&pattern) {
            ordered_patterns[idx] = (pattern.clone(), team);
        } else {
            pattern_index.insert(pattern.clone(), ordered_patterns.len());
            ordered_patterns.push((pattern, team));
        }
        if let Some(prefix) = glob_to_owners_prefix(&entry.glob) {
            owners_by_prefix.insert(prefix, format!("{}\n", entry.principal));
        }
    }

    for (pattern, team) in &ordered_patterns {
        let _ = writeln!(codeowners, "{pattern} {team}");
    }

    Ok(GeneratedOwners {
        codeowners,
        owners_by_prefix,
        entries,
    })
}

/// Serialize OWNERS-by-prefix map as canonical JSON object.
pub fn owners_map_json(
    owners_by_prefix: &BTreeMap<String, String>,
) -> Result<String, OwnersFromEnvelopesError> {
    let mut map = serde_json::Map::new();
    for (prefix, body) in owners_by_prefix {
        map.insert(
            prefix.clone(),
            Value::String(body.trim_end_matches('\n').to_owned()),
        );
    }
    let mut out = serde_json::to_string_pretty(&Value::Object(map)).map_err(|_| {
        // serde_json::Map/String values always serialize; treat failure as empty-input class.
        OwnersFromEnvelopesError::EmptyEntries
    })?;
    out.push('\n');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mini_envelopes() -> Value {
        json!({
            "path_ownership": { "law": "envelope_prefix_ownership" },
            "roots": {
                "messaging": {
                    "branch": "integ/messaging",
                    "envelope_globs": ["messaging/**"]
                },
                "compute": {
                    "branch": "integ/compute",
                    "envelope_globs": ["compute/**"]
                },
                "app-docs": {
                    "branch": "integ/app-docs",
                    "envelope_globs": ["app/docs/**"]
                }
            },
            "planes": {
                "docs": {
                    "branch": "integ/docs",
                    "envelope_globs": ["docs/**", "templates/**"]
                },
                "process_meta": {
                    "branch": "integ/ci",
                    "envelope_globs": [".github/**", "AGENTS.md"]
                },
                "root_manifests": {
                    "branch": "integ/build",
                    "envelope_globs": ["OWNERS", "Cargo.lock"]
                }
            }
        })
    }

    #[test]
    fn principal_uses_integ_branch_tail() {
        assert_eq!(principal_for_branch("process_meta", "integ/ci"), "ci");
        assert_eq!(
            principal_for_branch("root_manifests", "integ/build"),
            "build"
        );
        assert_eq!(
            principal_for_branch("messaging", "integ/messaging"),
            "messaging"
        );
    }

    #[test]
    fn generate_codeowners_and_owners_map() {
        let generated = generate_owners(&mini_envelopes()).expect("generate");
        assert!(
            generated
                .codeowners
                .contains("* @teams/council-architecture")
        );
        assert!(generated.codeowners.contains("messaging/ @teams/messaging"));
        assert!(generated.codeowners.contains("compute/ @teams/compute"));
        assert!(generated.codeowners.contains("app/docs/ @teams/app-docs"));
        assert!(generated.codeowners.contains("docs/ @teams/docs"));
        assert!(generated.codeowners.contains(".github/ @teams/ci"));
        assert!(generated.codeowners.contains("AGENTS.md @teams/ci"));
        assert!(generated.codeowners.contains("OWNERS @teams/build"));
        assert!(generated.codeowners.contains("Cargo.lock @teams/build"));
        // Specificity: app/docs/ must appear (last-match can override app/ if present).
        assert_eq!(
            generated
                .owners_by_prefix
                .get("messaging/")
                .map(String::as_str),
            Some("messaging\n")
        );
        assert_eq!(
            generated
                .owners_by_prefix
                .get("app/docs/")
                .map(String::as_str),
            Some("app-docs\n")
        );
        // Exact files do not get directory OWNERS entries.
        assert!(!generated.owners_by_prefix.contains_key("AGENTS.md"));
    }

    #[test]
    fn rejects_star_globs_that_are_not_dir_star_star() {
        let bad = json!({
            "roots": {
                "evil": {
                    "branch": "integ/evil",
                    "envelope_globs": ["foo/*"]
                }
            }
        });
        assert!(matches!(
            generate_owners(&bad),
            Err(OwnersFromEnvelopesError::InvalidGlob { .. })
        ));
    }
}

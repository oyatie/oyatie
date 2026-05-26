//! M02-P05-IP-001 — Capability registry use-cases.
//!
//! In-memory `CapabilityRegistry` + three use-cases:
//!   - `RegisterCapability` (validates, then inserts)
//!   - `ListCapabilities` (sorted by id)
//!   - `GetCapability` (by id)
//!
//! No I/O beyond optional file load for the seed JSON.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::fmt;

use oya_intelligence_capability_registry_domain::{PublishValidationError, validate_publish};
use oya_intelligence_capability_registry_kernel::{AutonomyTier, Capability, CapabilityId};

#[derive(Default)]
pub struct CapabilityRegistry {
    entries: BTreeMap<CapabilityId, Capability>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryError {
    Validation(PublishValidationError),
    Duplicate { id: String },
    NotFound { id: String },
    SeedParse { detail: String },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "publish validation failed: {e}"),
            Self::Duplicate { id } => write!(f, "capability already registered: {id}"),
            Self::NotFound { id } => write!(f, "capability not found: {id}"),
            Self::SeedParse { detail } => write!(f, "seed parse error: {detail}"),
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<PublishValidationError> for RegistryError {
    fn from(e: PublishValidationError) -> Self {
        Self::Validation(e)
    }
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use-case: RegisterCapability.
    pub fn register(&mut self, cap: Capability) -> Result<(), RegistryError> {
        validate_publish(&cap)?;
        if self.entries.contains_key(&cap.id) {
            return Err(RegistryError::Duplicate {
                id: cap.id.0.clone(),
            });
        }
        self.entries.insert(cap.id.clone(), cap);
        Ok(())
    }

    /// Use-case: ListCapabilities (sorted by id, deterministic).
    pub fn list(&self) -> Vec<Capability> {
        self.entries.values().cloned().collect()
    }

    /// Use-case: GetCapability.
    pub fn get(&self, id: &CapabilityId) -> Result<Capability, RegistryError> {
        self.entries
            .get(id)
            .cloned()
            .ok_or_else(|| RegistryError::NotFound { id: id.0.clone() })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Parse a capability seed JSON file. Accepts ONLY the schema produced by
/// `registry/capabilities/foundry-internal.json`:
///
/// ```json
/// [
///   {"id": "foundry.x.y", "name": "...", "autonomy_tier": "T1Read", "evidence_emit_required": true}
/// ]
/// ```
///
/// Hand-rolled minimal parser — no external deps (HARD CONSTRAINT).
pub fn parse_seed_json(src: &str) -> Result<Vec<Capability>, RegistryError> {
    let trimmed = src.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| RegistryError::SeedParse {
            detail: "expected outer array `[...]`".to_owned(),
        })?;

    let mut caps = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let bytes = inner.as_bytes();
    let mut in_string = false;
    let mut escape = false;

    for (i, b) in bytes.iter().enumerate() {
        let c = *b as char;
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match c {
            '{' => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let obj = &inner[start..=i];
                    caps.push(parse_object(obj)?);
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(RegistryError::SeedParse {
            detail: "unbalanced braces".to_owned(),
        });
    }
    Ok(caps)
}

fn parse_object(obj: &str) -> Result<Capability, RegistryError> {
    let id = extract_str(obj, "id")?;
    let name = extract_str(obj, "name")?;
    let tier_s = extract_str(obj, "autonomy_tier")?;
    let emit = extract_bool(obj, "evidence_emit_required")?;
    let tier = AutonomyTier::try_from(tier_s.as_str()).map_err(|e| RegistryError::SeedParse {
        detail: format!("unknown tier {}: {}", tier_s, e),
    })?;
    Ok(Capability::new(CapabilityId::new(id), name, tier, emit))
}

fn extract_str(obj: &str, key: &str) -> Result<String, RegistryError> {
    let needle = format!("\"{key}\"");
    let idx = obj.find(&needle).ok_or_else(|| RegistryError::SeedParse {
        detail: format!("missing key: {key}"),
    })?;
    let after = &obj[idx + needle.len()..];
    let colon = after.find(':').ok_or_else(|| RegistryError::SeedParse {
        detail: format!("missing `:` after key {key}"),
    })?;
    let rest = after[colon + 1..].trim_start();
    let rest = rest
        .strip_prefix('"')
        .ok_or_else(|| RegistryError::SeedParse {
            detail: format!("value for {key} is not a string"),
        })?;
    let end = rest.find('"').ok_or_else(|| RegistryError::SeedParse {
        detail: format!("unterminated string for {key}"),
    })?;
    Ok(rest[..end].to_owned())
}

fn extract_bool(obj: &str, key: &str) -> Result<bool, RegistryError> {
    let needle = format!("\"{key}\"");
    let idx = obj.find(&needle).ok_or_else(|| RegistryError::SeedParse {
        detail: format!("missing key: {key}"),
    })?;
    let after = &obj[idx + needle.len()..];
    let colon = after.find(':').ok_or_else(|| RegistryError::SeedParse {
        detail: format!("missing `:` after key {key}"),
    })?;
    let rest = after[colon + 1..].trim_start();
    if rest.starts_with("true") {
        Ok(true)
    } else if rest.starts_with("false") {
        Ok(false)
    } else {
        Err(RegistryError::SeedParse {
            detail: format!("value for {key} is not a bool"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cap(id: &str) -> Capability {
        Capability::new(CapabilityId::new(id), "n", AutonomyTier::T1Read, true)
    }

    #[test]
    fn register_then_list() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.account.list")).unwrap();
        r.register(cap("foundry.session.read")).unwrap();
        let listed = r.list();
        assert_eq!(listed.len(), 2);
    }

    #[test]
    fn register_rejects_duplicate() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.account.list")).unwrap();
        let err = r.register(cap("foundry.account.list")).unwrap_err();
        assert!(matches!(err, RegistryError::Duplicate { .. }));
    }

    #[test]
    fn get_returns_capability() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.audit.tail")).unwrap();
        let got = r.get(&CapabilityId::new("foundry.audit.tail")).unwrap();
        assert_eq!(got.id.0, "foundry.audit.tail");
    }

    #[test]
    fn get_not_found() {
        let r = CapabilityRegistry::new();
        let err = r.get(&CapabilityId::new("foundry.x.y")).unwrap_err();
        assert!(matches!(err, RegistryError::NotFound { .. }));
    }

    #[test]
    fn list_is_sorted() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.z.last")).unwrap();
        r.register(cap("foundry.a.first")).unwrap();
        let listed = r.list();
        assert_eq!(listed[0].id.0, "foundry.a.first");
        assert_eq!(listed[1].id.0, "foundry.z.last");
    }

    #[test]
    fn parse_seed_minimal() {
        let src = r#"[
          {"id":"foundry.account.list","name":"List accounts","autonomy_tier":"T1Read","evidence_emit_required":true}
        ]"#;
        let caps = parse_seed_json(src).unwrap();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].id.0, "foundry.account.list");
    }

    #[test]
    fn parse_seed_two() {
        let src = r#"[{"id":"foundry.a.b","name":"X","autonomy_tier":"T1Read","evidence_emit_required":true},
                      {"id":"foundry.c.d","name":"Y","autonomy_tier":"T2Suggest","evidence_emit_required":false}]"#;
        let caps = parse_seed_json(src).unwrap();
        assert_eq!(caps.len(), 2);
        assert_eq!(caps[1].autonomy_tier, AutonomyTier::T2Suggest);
    }

    #[test]
    fn seed_file_publishes_at_least_50_capabilities() {
        // Integration: load registry/capabilities/foundry-internal.json from
        // the workspace root (CARGO_MANIFEST_DIR points at this crate).
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest_dir)
            .join("..")
            .join("..")
            .join("registry")
            .join("capabilities")
            .join("foundry-internal.json");
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        let caps = parse_seed_json(&src).expect("parse seed json");
        assert!(
            caps.len() >= 50,
            "expected ≥ 50 capabilities, got {}",
            caps.len()
        );
        let mut reg = CapabilityRegistry::new();
        for c in caps {
            reg.register(c).expect("seed cap publishes cleanly");
        }
        assert!(reg.len() >= 50);
        // T4Actuate disabled by default for seed.
        let t4_count = reg
            .list()
            .into_iter()
            .filter(|c| c.autonomy_tier == AutonomyTier::T4Actuate)
            .count();
        assert_eq!(t4_count, 0, "seed must publish no T4Actuate caps");
    }
}

//! M02-P05-IP-001 — Capability registry use-cases.
//!
//! In-memory `CapabilityRegistry` + four use-cases:
//!   - `RegisterCapability` (validates, then inserts)
//!   - `ListCapabilities` (sorted by id)
//!   - `GetCapability` (by id)
//!   - `AffectedSet` (transitive impacted set via dependency edges)
//!
//! No I/O beyond optional file load for the seed JSON.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use intelligence_capability_registry_domain::{PublishValidationError, validate_publish};
use intelligence_capability_registry_kernel::{AutonomyTier, Capability, CapabilityId};

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

    /// Use-case: AffectedSet.
    ///
    /// Given a set of changed capability IDs, returns the transitive set of
    /// all impacted capability IDs by following `owner_capability_id` edges
    /// in reverse (dependents propagate impact forward).
    ///
    /// IDs in `changed` that are not present in the registry are included
    /// unchanged — they changed, so they are trivially affected.
    ///
    /// Output is a [`BTreeSet`] for deterministic, sorted iteration.
    pub fn affected_set(&self, changed: &BTreeSet<CapabilityId>) -> BTreeSet<CapabilityId> {
        if changed.is_empty() {
            return BTreeSet::new();
        }

        // Build reverse index: owner_id -> Vec<dependent_id>.
        // O(n) over the registry.
        let mut reverse: BTreeMap<&CapabilityId, Vec<&CapabilityId>> = BTreeMap::new();
        for (dep_id, cap) in &self.entries {
            if let Some(owner_id) = &cap.owner_capability_id {
                reverse.entry(owner_id).or_default().push(dep_id);
            }
        }

        // BFS from all seeds in `changed`.
        let mut visited: BTreeSet<CapabilityId> = BTreeSet::new();
        let mut queue: VecDeque<&CapabilityId> = VecDeque::new();

        for id in changed {
            if visited.insert(id.clone()) {
                queue.push_back(id);
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(dependents) = reverse.get(current) {
                for dep_id in dependents {
                    if visited.insert((*dep_id).clone()) {
                        queue.push_back(dep_id);
                    }
                }
            }
        }

        visited
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

    // ── AffectedSet use-case tests ───────────────────────────────────────

    fn cap_owned(id: &str, owner: &str) -> Capability {
        Capability::new(CapabilityId::new(id), "n", AutonomyTier::T1Read, true)
            .owned_by(CapabilityId::new(owner))
    }

    fn id_set(ids: &[&str]) -> BTreeSet<CapabilityId> {
        ids.iter().map(|s| CapabilityId::new(*s)).collect()
    }

    #[test]
    fn affected_set_empty_changed() {
        let r = CapabilityRegistry::new();
        assert!(r.affected_set(&BTreeSet::new()).is_empty());
    }

    #[test]
    fn affected_set_no_dependents() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.account.list")).unwrap();
        let result = r.affected_set(&id_set(&["foundry.account.list"]));
        assert_eq!(result, id_set(&["foundry.account.list"]));
    }

    #[test]
    fn affected_set_direct_dependent() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.account.list")).unwrap();
        r.register(cap_owned("foundry.account.summary", "foundry.account.list")).unwrap();
        let result = r.affected_set(&id_set(&["foundry.account.list"]));
        assert_eq!(
            result,
            id_set(&["foundry.account.list", "foundry.account.summary"])
        );
    }

    #[test]
    fn affected_set_transitive() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.account.list")).unwrap();
        r.register(cap_owned("foundry.account.summary", "foundry.account.list")).unwrap();
        r.register(cap_owned("foundry.account.report", "foundry.account.summary")).unwrap();
        let result = r.affected_set(&id_set(&["foundry.account.list"]));
        assert_eq!(
            result,
            id_set(&[
                "foundry.account.list",
                "foundry.account.summary",
                "foundry.account.report"
            ])
        );
    }

    #[test]
    fn affected_set_leaf_change_no_upstream() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.account.list")).unwrap();
        r.register(cap_owned("foundry.account.summary", "foundry.account.list")).unwrap();
        // Changing the child (summary) does not pull in the parent (list).
        let result = r.affected_set(&id_set(&["foundry.account.summary"]));
        assert_eq!(result, id_set(&["foundry.account.summary"]));
    }

    #[test]
    fn affected_set_diamond() {
        // B and C both depend on A; changing A affects A, B, C.
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.core.base")).unwrap();
        r.register(cap_owned("foundry.core.alpha", "foundry.core.base")).unwrap();
        r.register(cap_owned("foundry.core.beta", "foundry.core.base")).unwrap();
        let result = r.affected_set(&id_set(&["foundry.core.base"]));
        assert_eq!(
            result,
            id_set(&["foundry.core.base", "foundry.core.alpha", "foundry.core.beta"])
        );
    }

    #[test]
    fn affected_set_unknown_id_passes_through() {
        let r = CapabilityRegistry::new();
        let result = r.affected_set(&id_set(&["foundry.unknown.capability"]));
        assert_eq!(result, id_set(&["foundry.unknown.capability"]));
    }

    #[test]
    fn affected_set_multiple_roots() {
        let mut r = CapabilityRegistry::new();
        r.register(cap("foundry.account.list")).unwrap();
        r.register(cap_owned("foundry.account.summary", "foundry.account.list")).unwrap();
        r.register(cap("foundry.session.read")).unwrap();
        r.register(cap_owned("foundry.session.token", "foundry.session.read")).unwrap();
        let result = r.affected_set(&id_set(&["foundry.account.list", "foundry.session.read"]));
        assert_eq!(
            result,
            id_set(&[
                "foundry.account.list",
                "foundry.account.summary",
                "foundry.session.read",
                "foundry.session.token"
            ])
        );
    }

    #[test]
    fn seed_file_publishes_at_least_50_capabilities() {
        // Integration: load registry/capabilities/foundry-internal.json from
        // the workspace root (CARGO_MANIFEST_DIR points at this crate).
        // Ascend from this crate's manifest dir to the workspace root and locate
        // the seed by presence — nesting depth varies (ADR-0357 moved this crate
        // under microservices/intelligence/crates/), so don't hard-code `../..`.
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir
            .ancestors()
            .map(|dir| dir.join("registry/capabilities/foundry-internal.json"))
            .find(|candidate| candidate.exists())
            .unwrap_or_else(|| {
                panic!(
                    "foundry-internal.json not found ascending from {}",
                    manifest_dir.display()
                )
            });
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

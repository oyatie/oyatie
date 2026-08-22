---
doc_status: published
---

# Fitness Lane: capability-publish

- status: Accepted
- date: 2026-05-12
- purpose: Verify every kernel capability declared in a `*-kernel` crate is published to `docs/CAPABILITY-MAP.md`.
- enforces: STANDARD/capability-map; AGENTS.md fitness-lane `governance-capability-publish`.
- kernel_crate: `intelligence-capability-kernel` (EXISTING; extend with verdict) — `CapabilityDecl { crate_id, capability_id }`, `PublishedRow { capability_id, crate_id }`, verdict `CapabilityPublishFitnessReport { capabilities_checked }`.
- runner_path: `tools/governance-capability-publish`
- inputs: kernel sources `// capability:` markers, `docs/CAPABILITY-MAP.md`.
- failure_modes:
  - declared capability not in map
  - map row references missing crate
  - duplicate capability id
- ci_invocation: `cargo run -p governance-capability-publish`
- runtime_budget: 400 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct CapabilityDecl {
    pub crate_id: String,       // data_class: INTERNAL_ONLY
    pub capability_id: String,  // data_class: INTERNAL_ONLY
}
pub struct PublishedRow {
    pub capability_id: String,  // data_class: INTERNAL_ONLY
    pub crate_id: String,       // data_class: INTERNAL_ONLY
}

pub struct CapabilityPublishFitnessReport { pub capabilities_checked: usize }

pub enum CapabilityPublishFitnessError {
    Unpublished { capability_id: String, crate_id: String },
    OrphanMapRow { capability_id: String, crate_id: String },
    DuplicateCapability { capability_id: String },
}

pub fn validate_capability_publish_fitness(
    decls: &[CapabilityDecl],
    rows: &[PublishedRow],
    known_crates: &[String],
) -> Result<CapabilityPublishFitnessReport, CapabilityPublishFitnessError> {
    let known: std::collections::BTreeSet<&str> = known_crates.iter().map(|s| s.as_str()).collect();
    let by_cap: std::collections::BTreeMap<&str, &PublishedRow> =
        rows.iter().map(|r| (r.capability_id.as_str(), r)).collect();
    let mut seen = std::collections::BTreeSet::new();
    for d in decls {
        if !seen.insert(d.capability_id.clone()) {
            return Err(CapabilityPublishFitnessError::DuplicateCapability { capability_id: d.capability_id.clone() });
        }
        if !by_cap.contains_key(d.capability_id.as_str()) {
            return Err(CapabilityPublishFitnessError::Unpublished {
                capability_id: d.capability_id.clone(), crate_id: d.crate_id.clone(),
            });
        }
    }
    for r in rows {
        if !known.contains(r.crate_id.as_str()) {
            return Err(CapabilityPublishFitnessError::OrphanMapRow {
                capability_id: r.capability_id.clone(), crate_id: r.crate_id.clone(),
            });
        }
    }
    Ok(CapabilityPublishFitnessReport { capabilities_checked: decls.len() })
}
```

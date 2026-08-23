---
doc_status: published
---

# Fitness Lane: architecture-map-freshness

- status: Accepted
- date: 2026-05-12
- purpose: Verify `docs/architecture-map.md` (visualization) lists every active crate/axis and is regenerated when the crate list changes.
- enforces: Directive 11 (MASTERPLAN) — visualization is up-to-date.
- kernel_crate: `governance-architecture-map-freshness-kernel` — `MapNode { crate_id }`, verdict `ArchitectureMapFreshnessFitnessReport { crates_checked }`.
- runner_path: `tools/governance-architecture-map-freshness`
- inputs: `docs/architecture-map.md` parse, workspace crate list, last regen timestamp.
- failure_modes:
  - crate exists but absent from map
  - map node references missing crate
  - map last regen older than workspace `Cargo.toml`
- ci_invocation: `cargo run -p governance-architecture-map-freshness`
- runtime_budget: 350 ms
- severity: MED
- kernel_sketch:
```rust
pub struct MapNode { pub crate_id: String } // data_class: INTERNAL_ONLY
pub struct ArchitectureMapFreshnessFitnessReport { pub crates_checked: usize }

pub enum ArchitectureMapFreshnessFitnessError {
    MissingNode { crate_id: String },
    OrphanNode { crate_id: String },
    MapStale { map_age_seconds: u64, manifest_age_seconds: u64 },
}

pub fn validate_architecture_map_freshness_fitness(
    nodes: &[MapNode],
    workspace_crates: &[String],
    map_age_seconds: u64,
    manifest_age_seconds: u64,
) -> Result<ArchitectureMapFreshnessFitnessReport, ArchitectureMapFreshnessFitnessError> {
    let workspace: std::collections::BTreeSet<&str> = workspace_crates.iter().map(|s| s.as_str()).collect();
    let mapped: std::collections::BTreeSet<&str> = nodes.iter().map(|n| n.crate_id.as_str()).collect();
    for c in workspace_crates {
        if !mapped.contains(c.as_str()) {
            return Err(ArchitectureMapFreshnessFitnessError::MissingNode { crate_id: c.clone() });
        }
    }
    for n in nodes {
        if !workspace.contains(n.crate_id.as_str()) {
            return Err(ArchitectureMapFreshnessFitnessError::OrphanNode { crate_id: n.crate_id.clone() });
        }
    }
    if map_age_seconds > manifest_age_seconds {
        return Err(ArchitectureMapFreshnessFitnessError::MapStale { map_age_seconds, manifest_age_seconds });
    }
    Ok(ArchitectureMapFreshnessFitnessReport { crates_checked: workspace_crates.len() })
}
```

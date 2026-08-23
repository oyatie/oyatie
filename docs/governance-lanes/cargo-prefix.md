---
doc_status: published
---

# Fitness Lane: cargo-prefix

- status: Accepted
- date: 2026-05-12
- purpose: Verify every crate id follows the canonical `oyatie-{layer}-{axis}[-{role}]-{shape}` prefix grammar.
- enforces: STANDARD/cargo-prefix; existing crate `governance-cargo-prefix-kernel` (EXISTING; extend with verdict).
- kernel_crate: `governance-cargo-prefix-kernel` (EXISTING) — `CrateId { crate_id }`, verdict `CargoPrefixFitnessReport { crates_checked }`.
- runner_path: `tools/governance-cargo-prefix`
- inputs: workspace `Cargo.toml` member list, layer/axis/shape registries.
- failure_modes:
  - crate id `frobnicate` (no axis)
  - shape token not in {kernel, api, app, sdk, fitness}
  - crate id contains uppercase
- ci_invocation: `cargo run -p governance-cargo-prefix`
- runtime_budget: 150 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct CrateId { pub crate_id: String } // data_class: INTERNAL_ONLY
pub struct CargoPrefixFitnessReport { pub crates_checked: usize }

pub enum CargoPrefixFitnessError {
    UnknownLayer { crate_id: String, layer: String },
    UnknownAxis { crate_id: String, axis: String },
    UnknownShape { crate_id: String, shape: String },
    Uppercase { crate_id: String },
    Malformed { crate_id: String },
}

pub fn validate_cargo_prefix_fitness(
    crates: &[CrateId],
    layers: &[String],
    axes: &[String],
    shapes: &[String],
) -> Result<CargoPrefixFitnessReport, CargoPrefixFitnessError> {
    let layers_s: std::collections::BTreeSet<&str> = layers.iter().map(|s| s.as_str()).collect();
    let axes_s: std::collections::BTreeSet<&str> = axes.iter().map(|s| s.as_str()).collect();
    let shapes_s: std::collections::BTreeSet<&str> = shapes.iter().map(|s| s.as_str()).collect();
    for c in crates {
        if c.crate_id.chars().any(|ch| ch.is_uppercase()) {
            return Err(CargoPrefixFitnessError::Uppercase { crate_id: c.crate_id.clone() });
        }
        let parts: Vec<&str> = c.crate_id.split('-').collect();
        if parts.len() < 4 || parts[0] != "oya" {
            return Err(CargoPrefixFitnessError::Malformed { crate_id: c.crate_id.clone() });
        }
        if !layers_s.contains(parts[1]) { return Err(CargoPrefixFitnessError::UnknownLayer { crate_id: c.crate_id.clone(), layer: parts[1].into() }); }
        if !axes_s.contains(parts[2]) { return Err(CargoPrefixFitnessError::UnknownAxis { crate_id: c.crate_id.clone(), axis: parts[2].into() }); }
        let shape = parts.last().copied().unwrap_or("");
        if !shapes_s.contains(shape) { return Err(CargoPrefixFitnessError::UnknownShape { crate_id: c.crate_id.clone(), shape: shape.into() }); }
    }
    Ok(CargoPrefixFitnessReport { crates_checked: crates.len() })
}
```

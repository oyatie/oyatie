---
doc_status: published
---

# Fitness Lane: brand-residue

- status: Accepted
- date: 2026-05-12
- purpose: Verify retired brand/product names (e.g., legacy "Foundry-Classic", "OYABrand-v0") do not appear in active docs/code outside archived paths.
- enforces: STANDARD/brand-hygiene; AGENTS.md fitness-lane `governance-brand-residue`.
- kernel_crate: `governance-brand-residue-kernel` (EXISTING) — `BrandOccurrence { token, path, line, archived }`, verdict `BrandResidueFitnessReport { tokens_checked, occurrences }`.
- runner_path: `tools/governance-brand-residue`
- inputs: `docs/**/*`, `crates/**/*.rs`, retired-brand list `docs/standards/retired-brands.md`.
- failure_modes:
  - retired token in non-archive doc
  - retired token in active source file
  - case-variant of retired token
- ci_invocation: `cargo run -p governance-brand-residue`
- runtime_budget: 900 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct BrandOccurrence {
    pub token: String,   // data_class: INTERNAL_ONLY
    pub path: String,    // data_class: INTERNAL_ONLY
    pub line: u32,       // data_class: INTERNAL_ONLY
    pub archived: bool,  // data_class: INTERNAL_ONLY
}

pub struct BrandResidueFitnessReport {
    pub tokens_checked: usize,
    pub occurrences: usize,
}

pub enum BrandResidueFitnessError {
    LiveResidue { token: String, path: String, line: u32 },
}

pub fn validate_brand_residue_fitness(
    occurrences: &[BrandOccurrence],
    retired: &[String],
) -> Result<BrandResidueFitnessReport, BrandResidueFitnessError> {
    let retired: std::collections::BTreeSet<&str> = retired.iter().map(|s| s.as_str()).collect();
    for o in occurrences {
        if !o.archived && retired.contains(o.token.as_str()) {
            return Err(BrandResidueFitnessError::LiveResidue {
                token: o.token.clone(),
                path: o.path.clone(),
                line: o.line,
            });
        }
    }
    Ok(BrandResidueFitnessReport { tokens_checked: retired.len(), occurrences: occurrences.len() })
}
```

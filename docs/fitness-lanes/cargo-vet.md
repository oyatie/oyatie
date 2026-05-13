# Fitness Lane: cargo-vet

- status: Accepted
- date: 2026-05-12
- purpose: Verify every transitive dep is vetted (cargo-vet audit trail present and current).
- enforces: hyperscaler-best-practices spec — supply-chain review trail via cargo-vet.
- kernel_crate: `oya-foundry-fitness-cargo-vet-kernel` — `VetRecord { crate_id, version, audited_by, exemption }`, verdict `CargoVetFitnessReport { deps_checked }`.
- runner_path: `tools/oya-foundry-fitness-cargo-vet`
- inputs: `supply-chain/audits.toml`, `supply-chain/imports.lock`, `cargo vet check` JSON.
- failure_modes:
  - new dep with no audit row
  - exemption used without expiry date
  - cargo-vet reports `unaudited`
- ci_invocation: `cargo run -p oya-foundry-fitness-cargo-vet`
- runtime_budget: 1500 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct VetRecord {
    pub crate_id: String,            // data_class: INTERNAL_ONLY
    pub version: String,             // data_class: INTERNAL_ONLY
    pub audited_by: Option<String>,  // data_class: INTERNAL_ONLY
    pub exemption_expires: Option<String>, // data_class: INTERNAL_ONLY
}
pub struct CargoVetFitnessReport { pub deps_checked: usize }

pub enum CargoVetFitnessError {
    Unaudited { crate_id: String, version: String },
    ExpiredExemption { crate_id: String, expired_at: String },
    OpenExemption { crate_id: String },
}

pub fn validate_cargo_vet_fitness(
    records: &[VetRecord],
    today: &str,
) -> Result<CargoVetFitnessReport, CargoVetFitnessError> {
    for r in records {
        match (&r.audited_by, &r.exemption_expires) {
            (None, None) => return Err(CargoVetFitnessError::Unaudited {
                crate_id: r.crate_id.clone(), version: r.version.clone(),
            }),
            (None, Some(d)) if d.as_str() < today => return Err(CargoVetFitnessError::ExpiredExemption {
                crate_id: r.crate_id.clone(), expired_at: d.clone(),
            }),
            (None, Some(_)) => return Err(CargoVetFitnessError::OpenExemption { crate_id: r.crate_id.clone() }),
            _ => {}
        }
    }
    Ok(CargoVetFitnessReport { deps_checked: records.len() })
}
```

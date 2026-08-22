---
doc_status: published
---

# Fitness Lane: cargo-vet

- status: Retired from live admission until maintained inputs exist
- date: 2026-06-29
- purpose: Historical dependency-review lane. Do not count raw `cargo vet` output as Product-ready or Hyperscaler-ready evidence while `supply-chain/audits.toml` and `supply-chain/imports.lock` are absent.
- current_authority: `cloud-ci-supply-chain-audit` — owned RustSec advisory scan over vendored mirror, wired into `presubmit`.
- retired_inputs: `supply-chain/audits.toml`, `supply-chain/imports.lock`, `cargo vet check` JSON.
- failure_modes_when_reintroduced:
  - new dep with no audit row
  - exemption used without expiry date
  - cargo-vet reports `unaudited`
- ci_invocation: none while retired; reintroduction requires maintained input files plus a cloud-ci gate wired into `presubmit`.
- runtime_budget: 1500 ms
- severity: HIGH
- reintroduction_kernel_sketch:
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

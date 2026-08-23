---
doc_status: published
---

# Fitness Lane: slsa-provenance

- status: Accepted
- date: 2026-05-12
- purpose: Verify every release artifact has a SLSA Level 2+ provenance attestation tied to the build workflow.
- enforces: hyperscaler-best-practices spec — SLSA L2+ provenance.
- kernel_crate: `governance-slsa-provenance-kernel` — `ProvenanceRecord { artifact, builder_id, slsa_level, attestation_path }`, verdict `SlsaProvenanceFitnessReport { records_checked }`.
- runner_path: `tools/governance-slsa-provenance`
- inputs: in-toto attestation JSONs, registered builder list.
- failure_modes:
  - missing attestation file
  - SLSA level < 2
  - builder id not in registered list
- ci_invocation: `cargo run -p governance-slsa-provenance`
- runtime_budget: 1200 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct ProvenanceRecord {
    pub artifact: String,                // data_class: INTERNAL_ONLY
    pub builder_id: String,              // data_class: INTERNAL_ONLY
    pub slsa_level: u8,                  // data_class: INTERNAL_ONLY
    pub attestation_path: Option<String>,// data_class: INTERNAL_ONLY
}
pub struct SlsaProvenanceFitnessReport { pub records_checked: usize }

pub enum SlsaProvenanceFitnessError {
    MissingAttestation { artifact: String },
    InsufficientLevel { artifact: String, level: u8 },
    UnknownBuilder { artifact: String, builder_id: String },
}

pub fn validate_slsa_provenance_fitness(
    records: &[ProvenanceRecord],
    known_builders: &[String],
    minimum_level: u8,
) -> Result<SlsaProvenanceFitnessReport, SlsaProvenanceFitnessError> {
    let builders: std::collections::BTreeSet<&str> = known_builders.iter().map(|s| s.as_str()).collect();
    for r in records {
        if r.attestation_path.is_none() {
            return Err(SlsaProvenanceFitnessError::MissingAttestation { artifact: r.artifact.clone() });
        }
        if r.slsa_level < minimum_level {
            return Err(SlsaProvenanceFitnessError::InsufficientLevel { artifact: r.artifact.clone(), level: r.slsa_level });
        }
        if !builders.contains(r.builder_id.as_str()) {
            return Err(SlsaProvenanceFitnessError::UnknownBuilder { artifact: r.artifact.clone(), builder_id: r.builder_id.clone() });
        }
    }
    Ok(SlsaProvenanceFitnessReport { records_checked: records.len() })
}
```

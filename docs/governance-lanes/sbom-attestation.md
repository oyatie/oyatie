---
doc_status: published
---

# Fitness Lane: sbom-attestation

- status: Accepted
- date: 2026-05-12
- purpose: Verify every shipped binary has a Syft-generated SBOM attached as a release artifact.
- enforces: hyperscaler-best-practices spec — every binary ships SBOM via Syft.
- kernel_crate: `governance-sbom-attestation-kernel` — `BinaryArtifact { binary, sbom_path, sbom_format }`, verdict `SbomAttestationFitnessReport { binaries_checked }`.
- runner_path: `tools/governance-sbom-attestation`
- inputs: release artifact manifest, SBOM file index.
- failure_modes:
  - binary published with no SBOM
  - SBOM not in SPDX or CycloneDX format
  - SBOM path unresolved
- ci_invocation: `cargo run -p governance-sbom-attestation`
- runtime_budget: 1500 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct BinaryArtifact {
    pub binary: String,       // data_class: INTERNAL_ONLY
    pub sbom_path: Option<String>, // data_class: INTERNAL_ONLY
    pub sbom_format: Option<String>, // data_class: INTERNAL_ONLY
}
pub struct SbomAttestationFitnessReport { pub binaries_checked: usize }

pub enum SbomAttestationFitnessError {
    MissingSbom { binary: String },
    UnknownFormat { binary: String, format: String },
    SbomFileMissing { binary: String, path: String },
}

pub fn validate_sbom_attestation_fitness(
    artifacts: &[BinaryArtifact],
    allowed_formats: &[String],
    existing_files: &[String],
) -> Result<SbomAttestationFitnessReport, SbomAttestationFitnessError> {
    let formats: std::collections::BTreeSet<&str> = allowed_formats.iter().map(|s| s.as_str()).collect();
    let files: std::collections::BTreeSet<&str> = existing_files.iter().map(|s| s.as_str()).collect();
    for a in artifacts {
        let p = a.sbom_path.as_ref().ok_or_else(|| SbomAttestationFitnessError::MissingSbom { binary: a.binary.clone() })?;
        let f = a.sbom_format.as_ref().ok_or_else(|| SbomAttestationFitnessError::MissingSbom { binary: a.binary.clone() })?;
        if !formats.contains(f.as_str()) {
            return Err(SbomAttestationFitnessError::UnknownFormat { binary: a.binary.clone(), format: f.clone() });
        }
        if !files.contains(p.as_str()) {
            return Err(SbomAttestationFitnessError::SbomFileMissing { binary: a.binary.clone(), path: p.clone() });
        }
    }
    Ok(SbomAttestationFitnessReport { binaries_checked: artifacts.len() })
}
```

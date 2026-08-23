---
doc_status: published
---

# Fitness Lane: cosign-signature

- status: Accepted
- date: 2026-05-12
- purpose: Verify every release artifact carries a valid Cosign signature using the platform root key.
- enforces: hyperscaler-best-practices spec — every artifact Cosign-signed.
- kernel_crate: `governance-cosign-signature-kernel` — `SignedArtifact { artifact, signature_path, issuer, verified }`, verdict `CosignSignatureFitnessReport { artifacts_checked }`.
- runner_path: `tools/governance-cosign-signature`
- inputs: cosign verify report JSON, root-key fingerprint.
- failure_modes:
  - artifact without `.sig`
  - signature issuer mismatch
  - cosign verify returned false
- ci_invocation: `cargo run -p governance-cosign-signature`
- runtime_budget: 1800 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct SignedArtifact {
    pub artifact: String,           // data_class: INTERNAL_ONLY
    pub signature_path: Option<String>, // data_class: INTERNAL_ONLY
    pub issuer: Option<String>,     // data_class: INTERNAL_ONLY
    pub verified: bool,             // data_class: INTERNAL_ONLY
}
pub struct CosignSignatureFitnessReport { pub artifacts_checked: usize }

pub enum CosignSignatureFitnessError {
    MissingSignature { artifact: String },
    IssuerMismatch { artifact: String, issuer: String },
    Unverified { artifact: String },
}

pub fn validate_cosign_signature_fitness(
    artifacts: &[SignedArtifact],
    expected_issuer: &str,
) -> Result<CosignSignatureFitnessReport, CosignSignatureFitnessError> {
    for a in artifacts {
        if a.signature_path.is_none() {
            return Err(CosignSignatureFitnessError::MissingSignature { artifact: a.artifact.clone() });
        }
        let i = a.issuer.as_deref().unwrap_or("");
        if i != expected_issuer {
            return Err(CosignSignatureFitnessError::IssuerMismatch { artifact: a.artifact.clone(), issuer: i.to_string() });
        }
        if !a.verified {
            return Err(CosignSignatureFitnessError::Unverified { artifact: a.artifact.clone() });
        }
    }
    Ok(CosignSignatureFitnessReport { artifacts_checked: artifacts.len() })
}
```

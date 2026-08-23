---
doc_status: published
---

# Fitness Lane: evidence-bundle-shape

- status: Accepted
- date: 2026-05-12
- purpose: Verify every PR's Phase 00 evidence bundle conforms to the template (commands run, output hashes, exit codes, timestamps).
- enforces: TEMPLATE/phase-00-evidence-template.
- kernel_crate: `governance-evidence-bundle-shape-kernel` — `EvidenceBundle { pr_id, commands, hashes_present, timestamps_present }`, verdict `EvidenceBundleShapeFitnessReport { bundles_checked }`.
- runner_path: `tools/governance-evidence-bundle-shape`
- inputs: PR `/evidence/<pr>.md`, evidence template.
- failure_modes:
  - missing `cargo nextest run` row
  - row without output hash
  - timestamp not ISO-8601
- ci_invocation: `cargo run -p governance-evidence-bundle-shape`
- runtime_budget: 350 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct EvidenceRow {
    pub command: String,       // data_class: INTERNAL_ONLY
    pub exit_code: i32,        // data_class: INTERNAL_ONLY
    pub output_hash: Option<String>, // data_class: INTERNAL_ONLY
    pub timestamp: Option<String>,   // data_class: INTERNAL_ONLY
}
pub struct EvidenceBundle {
    pub pr_id: String,         // data_class: INTERNAL_ONLY
    pub rows: Vec<EvidenceRow>,// data_class: INTERNAL_ONLY
}
pub struct EvidenceBundleShapeFitnessReport { pub bundles_checked: usize }

pub enum EvidenceBundleShapeFitnessError {
    MissingRequiredCommand { pr_id: String, command: String },
    MissingHash { pr_id: String, command: String },
    BadTimestamp { pr_id: String, command: String },
}

pub fn validate_evidence_bundle_shape_fitness(
    bundles: &[EvidenceBundle],
    required_commands: &[String],
) -> Result<EvidenceBundleShapeFitnessReport, EvidenceBundleShapeFitnessError> {
    for b in bundles {
        let cmds: std::collections::BTreeSet<&str> = b.rows.iter().map(|r| r.command.as_str()).collect();
        for req in required_commands {
            if !cmds.contains(req.as_str()) {
                return Err(EvidenceBundleShapeFitnessError::MissingRequiredCommand { pr_id: b.pr_id.clone(), command: req.clone() });
            }
        }
        for r in &b.rows {
            if r.output_hash.is_none() {
                return Err(EvidenceBundleShapeFitnessError::MissingHash { pr_id: b.pr_id.clone(), command: r.command.clone() });
            }
            let ts = r.timestamp.as_deref().unwrap_or("");
            if !ts.contains('T') || !ts.ends_with('Z') {
                return Err(EvidenceBundleShapeFitnessError::BadTimestamp { pr_id: b.pr_id.clone(), command: r.command.clone() });
            }
        }
    }
    Ok(EvidenceBundleShapeFitnessReport { bundles_checked: bundles.len() })
}
```

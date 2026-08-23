---
doc_status: published
---

# Fitness Lane: perf-evidence

- status: Accepted
- date: 2026-05-12
- purpose: Verify every PR touching a perf-sensitive crate ships a numeric evidence block (p50/p95/p99 in ms) in CHANGELOG.
- enforces: STANDARD/perf-evidence; AGENTS.md fitness-lane `governance-perf-evidence`.
- kernel_crate: `governance-perf-evidence-kernel` — `PerfEvidence { crate_id, p50_ms, p95_ms, p99_ms, source }`, verdict `PerfEvidenceFitnessReport { evidences_checked }`.
- runner_path: `tools/governance-perf-evidence`
- inputs: PR body, `docs/CHANGELOG.md`, registry of perf-sensitive crate ids.
- failure_modes:
  - perf-sensitive crate touched but no evidence block
  - evidence has placeholder `TBD` value
  - p99 < p50 (impossible)
- ci_invocation: `cargo run -p governance-perf-evidence`
- runtime_budget: 300 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct PerfEvidence {
    pub crate_id: String,   // data_class: INTERNAL_ONLY
    pub p50_ms: f64,        // data_class: INTERNAL_ONLY
    pub p95_ms: f64,        // data_class: INTERNAL_ONLY
    pub p99_ms: f64,        // data_class: INTERNAL_ONLY
    pub source: String,     // data_class: INTERNAL_ONLY (bench id)
}

pub struct PerfEvidenceFitnessReport { pub evidences_checked: usize }

pub enum PerfEvidenceFitnessError {
    Missing { crate_id: String },
    InvalidOrdering { crate_id: String },
    PlaceholderValue { crate_id: String },
}

pub fn validate_perf_evidence_fitness(
    evidences: &[PerfEvidence],
    touched_perf_sensitive: &[String],
) -> Result<PerfEvidenceFitnessReport, PerfEvidenceFitnessError> {
    let by_crate: std::collections::BTreeMap<&str, &PerfEvidence> =
        evidences.iter().map(|e| (e.crate_id.as_str(), e)).collect();
    for c in touched_perf_sensitive {
        let e = by_crate.get(c.as_str()).ok_or_else(|| PerfEvidenceFitnessError::Missing { crate_id: c.clone() })?;
        if e.source.is_empty() { return Err(PerfEvidenceFitnessError::PlaceholderValue { crate_id: c.clone() }); }
        if e.p99_ms < e.p50_ms || e.p95_ms < e.p50_ms || e.p99_ms < e.p95_ms {
            return Err(PerfEvidenceFitnessError::InvalidOrdering { crate_id: c.clone() });
        }
    }
    Ok(PerfEvidenceFitnessReport { evidences_checked: evidences.len() })
}
```

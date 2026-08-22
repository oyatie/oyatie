---
doc_status: published
---

# Fitness Lane: forward-reference-resolved

- status: Accepted
- date: 2026-05-12
- purpose: Verify no `<!-- forward-reference: wave-N -->` sentinels remain at the wave gate.
- enforces: STANDARD/wave-gates.
- kernel_crate: `governance-forward-reference-resolved-kernel` — `Sentinel { path, line, wave_id }`, verdict `ForwardReferenceResolvedFitnessReport { sentinels_checked }`.
- runner_path: `tools/governance-forward-reference-resolved`
- inputs: doc/source tree, current wave id.
- failure_modes:
  - sentinel for current wave still present
  - sentinel for past wave present (always unresolved)
  - sentinel has no wave id
- ci_invocation: `cargo run -p governance-forward-reference-resolved`
- runtime_budget: 500 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct Sentinel {
    pub path: String,           // data_class: INTERNAL_ONLY
    pub line: u32,              // data_class: INTERNAL_ONLY
    pub wave_id: Option<String>,// data_class: INTERNAL_ONLY
}
pub struct ForwardReferenceResolvedFitnessReport { pub sentinels_checked: usize }

pub enum ForwardReferenceResolvedFitnessError {
    UnresolvedAtGate { path: String, line: u32, wave_id: String },
    UnknownWave { path: String, line: u32 },
}

pub fn validate_forward_reference_resolved_fitness(
    sentinels: &[Sentinel],
    current_wave: &str,
    past_waves: &[String],
) -> Result<ForwardReferenceResolvedFitnessReport, ForwardReferenceResolvedFitnessError> {
    let past: std::collections::BTreeSet<&str> = past_waves.iter().map(|s| s.as_str()).collect();
    for s in sentinels {
        let w = s.wave_id.as_ref().ok_or_else(|| ForwardReferenceResolvedFitnessError::UnknownWave {
            path: s.path.clone(), line: s.line,
        })?;
        if w == current_wave || past.contains(w.as_str()) {
            return Err(ForwardReferenceResolvedFitnessError::UnresolvedAtGate {
                path: s.path.clone(), line: s.line, wave_id: w.clone(),
            });
        }
    }
    Ok(ForwardReferenceResolvedFitnessReport { sentinels_checked: sentinels.len() })
}
```

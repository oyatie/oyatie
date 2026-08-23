---
doc_status: published
---

# Fitness Lane: cutover-bootstrap-window

- status: Accepted
- date: 2026-05-12
- purpose: Verify every cutover bootstrap window has audit rows for begin/end and is bounded in time.
- enforces: ADR-0053 — cutover bootstrap window.
- adr_citations: ADR-0053 (sanctioned primitives — defines the bootstrap window constraints during which the transition from legacy to sanctioned primitives occurs)
- kernel_crate: `governance-cutover-bootstrap-window-kernel` — `BootstrapWindow { window_id, begin_row, end_row, duration_minutes }`, verdict `CutoverBootstrapWindowFitnessReport { windows_checked }`.
- runner_path: `tools/governance-cutover-bootstrap-window`
- inputs: audit ledger, declared cutover-window registry.
- failure_modes:
  - window begin row exists but no end row
  - window duration > policy budget (e.g., 60 min)
  - end row precedes begin row
- ci_invocation: `cargo run -p governance-cutover-bootstrap-window`
- runtime_budget: 500 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct BootstrapWindow {
    pub window_id: String,            // data_class: INTERNAL_ONLY
    pub begin_row: Option<String>,    // data_class: INTERNAL_ONLY (audit id)
    pub end_row: Option<String>,      // data_class: INTERNAL_ONLY
    pub duration_minutes: Option<u32>,// data_class: INTERNAL_ONLY
}
pub struct CutoverBootstrapWindowFitnessReport { pub windows_checked: usize }

pub enum CutoverBootstrapWindowFitnessError {
    MissingBegin { window_id: String },
    MissingEnd { window_id: String },
    DurationExceeded { window_id: String, duration_minutes: u32, budget: u32 },
    InvertedTimes { window_id: String },
}

pub fn validate_cutover_bootstrap_window_fitness(
    windows: &[BootstrapWindow],
    budget_minutes: u32,
) -> Result<CutoverBootstrapWindowFitnessReport, CutoverBootstrapWindowFitnessError> {
    for w in windows {
        if w.begin_row.is_none() { return Err(CutoverBootstrapWindowFitnessError::MissingBegin { window_id: w.window_id.clone() }); }
        if w.end_row.is_none() { return Err(CutoverBootstrapWindowFitnessError::MissingEnd { window_id: w.window_id.clone() }); }
        let d = w.duration_minutes.unwrap_or(u32::MAX);
        if d > budget_minutes {
            return Err(CutoverBootstrapWindowFitnessError::DurationExceeded {
                window_id: w.window_id.clone(), duration_minutes: d, budget: budget_minutes,
            });
        }
        if d == 0 {
            return Err(CutoverBootstrapWindowFitnessError::InvertedTimes { window_id: w.window_id.clone() });
        }
    }
    Ok(CutoverBootstrapWindowFitnessReport { windows_checked: windows.len() })
}
```

---
doc_status: published
---

# Fitness Lane: redirect-thinness

- status: Accepted
- date: 2026-05-12
- purpose: Verify "redirect-only" doc files stay thin (single canonical pointer + redirect header only) and never accumulate content.
- enforces: STANDARD/redirect-shape; AGENTS.md fitness-lane `governance-redirect-thinness`.
- kernel_crate: `governance-redirect-thinness-kernel` — `RedirectDoc { path, byte_size, body_line_count }`, verdict `RedirectThinnessFitnessReport { docs_checked }`.
- runner_path: `tools/governance-redirect-thinness`
- inputs: docs with `redirect_to:` front-matter; size budget registry.
- failure_modes:
  - redirect file > 30 body lines
  - redirect file has H1 other than `# Moved`
  - redirect target unresolved
- ci_invocation: `cargo run -p governance-redirect-thinness`
- runtime_budget: 200 ms
- severity: MED
- kernel_sketch:
```rust
pub struct RedirectDoc {
    pub path: String,           // data_class: INTERNAL_ONLY
    pub redirect_to: String,    // data_class: INTERNAL_ONLY
    pub body_line_count: u32,   // data_class: INTERNAL_ONLY
    pub h1: String,             // data_class: INTERNAL_ONLY
    pub target_exists: bool,    // data_class: INTERNAL_ONLY
}

pub struct RedirectThinnessFitnessReport { pub docs_checked: usize }

pub enum RedirectThinnessFitnessError {
    BodyTooLarge { path: String, lines: u32, budget: u32 },
    WrongH1 { path: String, h1: String },
    TargetMissing { path: String, target: String },
}

pub fn validate_redirect_thinness_fitness(
    docs: &[RedirectDoc],
    budget_lines: u32,
) -> Result<RedirectThinnessFitnessReport, RedirectThinnessFitnessError> {
    for d in docs {
        if !d.target_exists {
            return Err(RedirectThinnessFitnessError::TargetMissing { path: d.path.clone(), target: d.redirect_to.clone() });
        }
        if d.h1 != "Moved" {
            return Err(RedirectThinnessFitnessError::WrongH1 { path: d.path.clone(), h1: d.h1.clone() });
        }
        if d.body_line_count > budget_lines {
            return Err(RedirectThinnessFitnessError::BodyTooLarge { path: d.path.clone(), lines: d.body_line_count, budget: budget_lines });
        }
    }
    Ok(RedirectThinnessFitnessReport { docs_checked: docs.len() })
}
```

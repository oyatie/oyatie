---
doc_status: published
---

# Fitness Lane: clippy-pedantic

- status: Accepted
- date: 2026-05-12
- purpose: Verify the workspace passes `cargo clippy --all-targets -- -D warnings` with pedantic + workspace lint table.
- enforces: hyperscaler-best-practices spec — Rust clippy pedantic + workspace lints.
- kernel_crate: `governance-clippy-pedantic-kernel` — `ClippyFinding { crate_id, file, lint }`, verdict `ClippyPedanticFitnessReport { findings_checked }`.
- runner_path: `tools/governance-clippy-pedantic`
- inputs: `cargo clippy --message-format=json` output.
- failure_modes:
  - any warning at error level
  - lint in `denied-without-bypass` set fires
  - missing `[workspace.lints]` table (handed off to `workspace-lints` lane)
- ci_invocation: `cargo run -p governance-clippy-pedantic`
- runtime_budget: 3500 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct ClippyFinding {
    pub crate_id: String, // data_class: INTERNAL_ONLY
    pub file: String,     // data_class: INTERNAL_ONLY
    pub lint: String,     // data_class: INTERNAL_ONLY
    pub level: String,    // data_class: INTERNAL_ONLY
}
pub struct ClippyPedanticFitnessReport { pub findings_checked: usize }

pub enum ClippyPedanticFitnessError {
    DenyLintFired { crate_id: String, file: String, lint: String },
    UnknownLevel { lint: String, level: String },
}

pub fn validate_clippy_pedantic_fitness(
    findings: &[ClippyFinding],
    deny_lints: &[String],
) -> Result<ClippyPedanticFitnessReport, ClippyPedanticFitnessError> {
    let deny: std::collections::BTreeSet<&str> = deny_lints.iter().map(|s| s.as_str()).collect();
    for f in findings {
        match f.level.as_str() {
            "error" | "warning" => {
                if deny.contains(f.lint.as_str()) || f.level == "error" {
                    return Err(ClippyPedanticFitnessError::DenyLintFired {
                        crate_id: f.crate_id.clone(), file: f.file.clone(), lint: f.lint.clone(),
                    });
                }
            }
            "note" | "help" => {}
            other => return Err(ClippyPedanticFitnessError::UnknownLevel {
                lint: f.lint.clone(), level: other.to_string(),
            }),
        }
    }
    Ok(ClippyPedanticFitnessReport { findings_checked: findings.len() })
}
```

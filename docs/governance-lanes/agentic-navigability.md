---
doc_status: published
---

# Fitness Lane: agentic-navigability

- status: Accepted
- date: 2026-05-12
- enforces: Directive 10 (MASTERPLAN) — agentic navigability.
- kernel_crate: `governance-agentic-navigability-kernel` — `DirEntry { path, file_count, has_index }`, verdict `AgenticNavigabilityFitnessReport { dirs_checked }`.
- runner_path: `tools/governance-agentic-navigability`
- failure_modes:
  - dir with > 5 files but no INDEX.md
  - filename violates naming pattern (e.g., `Final-v2-NEW.md`)
  - INDEX.md row points at missing sibling
- ci_invocation: `cargo run -p governance-agentic-navigability`
- runtime_budget: 600 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct DirEntry {
    pub path: String,         // data_class: INTERNAL_ONLY
    pub file_count: u32,      // data_class: INTERNAL_ONLY
    pub has_index: bool,      // data_class: INTERNAL_ONLY
    pub files: Vec<String>,   // data_class: INTERNAL_ONLY
}

pub struct AgenticNavigabilityFitnessReport { pub dirs_checked: usize }

pub enum AgenticNavigabilityFitnessError {
    MissingIndex { path: String, file_count: u32 },
    BadNaming { path: String, file: String },
    IndexRowUnresolved { path: String, target: String },
}

pub fn validate_agentic_navigability_fitness(
    dirs: &[DirEntry],
    threshold: u32,
    naming_pattern: &str,
) -> Result<AgenticNavigabilityFitnessReport, AgenticNavigabilityFitnessError> {
    let re = regex::Regex::new(naming_pattern).expect("valid pattern");
    for d in dirs {
        if d.file_count > threshold && !d.has_index {
            return Err(AgenticNavigabilityFitnessError::MissingIndex {
                path: d.path.clone(), file_count: d.file_count,
            });
        }
        for f in &d.files {
            if !re.is_match(f) {
                return Err(AgenticNavigabilityFitnessError::BadNaming {
                    path: d.path.clone(), file: f.clone(),
                });
            }
        }
    }
    Ok(AgenticNavigabilityFitnessReport { dirs_checked: dirs.len() })
}
```

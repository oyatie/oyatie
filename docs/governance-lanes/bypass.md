---
doc_status: published
---

# Fitness Lane: bypass

- status: Accepted
- date: 2026-05-12
- enforces: STANDARD/no-silent-bypass; AGENTS.md fitness-lane `governance-bypass`.
- adr_citations: ADR-0053 (sanctioned primitives — bypass directives must carry an accepted ADR justification, consistent with the primitive-restriction rationale model)
- kernel_crate: `intelligence-bypass-kernel` (EXISTING) — `BypassDirective { path, line, directive, justification_ref }`, verdict `BypassFitnessReport { directives_checked, justified }`.
- runner_path: `tools/governance-bypass`
- inputs: every source/doc file, registry of allowed bypass directives + justification ADR ids.
- failure_modes:
  - `#[allow(clippy::all)]` at module level
  - `<!-- fitness-skip:* -->` whose ADR is `proposed`
- ci_invocation: `cargo run -p governance-bypass`
- runtime_budget: 700 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct BypassDirective {
    pub path: String,                       // data_class: INTERNAL_ONLY
    pub line: u32,                          // data_class: INTERNAL_ONLY
    pub directive: String,                  // data_class: INTERNAL_ONLY
    pub justification_ref: Option<String>,  // data_class: INTERNAL_ONLY
}

pub struct BypassFitnessReport {
    pub directives_checked: usize,
    pub justified: usize,
}

pub enum BypassFitnessError {
    UnjustifiedBypass { path: String, line: u32, directive: String },
    UnknownJustification { path: String, line: u32, ref_id: String },
}

pub fn validate_bypass_fitness(
    directives: &[BypassDirective],
    accepted_refs: &[String],
) -> Result<BypassFitnessReport, BypassFitnessError> {
    let known: std::collections::BTreeSet<&str> = accepted_refs.iter().map(|s| s.as_str()).collect();
    let mut justified = 0;
    for d in directives {
        let r = d.justification_ref.as_deref().ok_or_else(|| BypassFitnessError::UnjustifiedBypass {
            path: d.path.clone(), line: d.line, directive: d.directive.clone(),
        })?;
        if !known.contains(r) {
            return Err(BypassFitnessError::UnknownJustification {
                path: d.path.clone(), line: d.line, ref_id: r.to_string(),
            });
        }
        justified += 1;
    }
    Ok(BypassFitnessReport { directives_checked: directives.len(), justified })
}
```

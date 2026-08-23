---
doc_status: published
---

# Fitness Lane: adr-shape

- status: Accepted
- date: 2026-05-12
- purpose: Verify every ADR has required sections (Context, Decision, Consequences, Status, Drivers, Alternatives) and a valid status.
- enforces: TEMPLATE/adr-template; AGENTS.md fitness-lane `governance-adr-shape`.
- kernel_crate: `governance-adr-shape-kernel` — `AdrDocument { adr_id, sections, status }`, verdict `AdrShapeFitnessReport { adrs_checked }`.
- runner_path: `tools/governance-adr-shape`
- inputs: `docs/decisions/ADR-*.md`, ADR template `docs/templates/ADR-TEMPLATE.md`.
- failure_modes:
  - ADR missing Consequences section
  - status not in {proposed, accepted, superseded, retracted}
  - sections in wrong order
- ci_invocation: `cargo run -p governance-adr-shape`
- runtime_budget: 250 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct AdrDocument {
    pub adr_id: String,               // data_class: INTERNAL_ONLY
    pub sections: Vec<String>,        // data_class: INTERNAL_ONLY
    pub status: String,               // data_class: INTERNAL_ONLY
}

pub struct AdrShapeFitnessReport { pub adrs_checked: usize }

pub enum AdrShapeFitnessError {
    MissingSection { adr_id: String, section: String },
    InvalidStatus { adr_id: String, status: String },
    SectionsOutOfOrder { adr_id: String, expected: String, actual: String },
}

pub fn validate_adr_shape_fitness(
    adrs: &[AdrDocument],
    required: &[String],
    valid_statuses: &[String],
) -> Result<AdrShapeFitnessReport, AdrShapeFitnessError> {
    let valid: std::collections::BTreeSet<&str> = valid_statuses.iter().map(|s| s.as_str()).collect();
    for a in adrs {
        if !valid.contains(a.status.as_str()) {
            return Err(AdrShapeFitnessError::InvalidStatus {
                adr_id: a.adr_id.clone(), status: a.status.clone(),
            });
        }
        for (i, req) in required.iter().enumerate() {
            if i >= a.sections.len() {
                return Err(AdrShapeFitnessError::MissingSection {
                    adr_id: a.adr_id.clone(), section: req.clone(),
                });
            }
            if &a.sections[i] != req {
                return Err(AdrShapeFitnessError::SectionsOutOfOrder {
                    adr_id: a.adr_id.clone(), expected: req.clone(), actual: a.sections[i].clone(),
                });
            }
        }
    }
    Ok(AdrShapeFitnessReport { adrs_checked: adrs.len() })
}
```

## Diagnostic audit API

The retirement-marked app remains only the blocking validation path:
`buck2 run //tools/governance-adr-shape-app:governance-adr-shape-app`.
Its explicit-path form is blocking as well. Migration-inventory consumers call
the public Rust `audit_adr_shape_fitness(&[AdrDocument])` API and own their input
loading and output presentation.

The sorted API findings are not admission authority. A clean report does not
accept an ADR, normalize lifecycle state, authorize planning, dispatch work, or
close a Stage-1 gate. Noncanonical or legacy status spelling is inventory only.
Unit parser tests and filesystem fixtures are intentionally separate.

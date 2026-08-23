---
doc_status: published
---

# Fitness Lane: codeowners-mirror

- status: Accepted
- date: 2026-05-12
- purpose: Verify `.github/CODEOWNERS` mirrors the catalog + RACI owner_axis values for every canonical path.
- enforces: STANDARD/codeowners-mirror.
- kernel_contract: `OwnerRule { path, owner_axis }`, verdict `CodeownersMirrorFitnessReport { rules_checked }`.
- runner_path: `tools/governance-codeowners-mirror`
- inputs: `.github/CODEOWNERS`, catalog rows, RACI rows.
- failure_modes:
  - canonical path has no CODEOWNERS rule
  - CODEOWNERS owner differs from catalog owner_axis
  - rule references unknown owner team
- ci_invocation: `cargo run -p governance-codeowners-mirror`
- runtime_budget: 300 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct OwnerRule { pub path: String, pub owner_axis: String } // data_class: INTERNAL_ONLY
pub struct ExpectedOwner { pub path: String, pub owner_axis: String } // data_class: INTERNAL_ONLY
pub struct CodeownersMirrorFitnessReport { pub rules_checked: usize }

pub enum CodeownersMirrorFitnessError {
    MissingRule { path: String, expected_owner: String },
    OwnerMismatch { path: String, expected: String, actual: String },
    UnknownOwner { path: String, owner: String },
}

pub fn validate_codeowners_mirror_fitness(
    rules: &[OwnerRule],
    expected: &[ExpectedOwner],
    known_owners: &[String],
) -> Result<CodeownersMirrorFitnessReport, CodeownersMirrorFitnessError> {
    let owners: std::collections::BTreeSet<&str> = known_owners.iter().map(|s| s.as_str()).collect();
    let by_path: std::collections::BTreeMap<&str, &OwnerRule> = rules.iter().map(|r| (r.path.as_str(), r)).collect();
    for e in expected {
        let r = by_path.get(e.path.as_str()).ok_or_else(|| CodeownersMirrorFitnessError::MissingRule {
            path: e.path.clone(), expected_owner: e.owner_axis.clone(),
        })?;
        if r.owner_axis != e.owner_axis {
            return Err(CodeownersMirrorFitnessError::OwnerMismatch {
                path: e.path.clone(), expected: e.owner_axis.clone(), actual: r.owner_axis.clone(),
            });
        }
        if !owners.contains(r.owner_axis.as_str()) {
            return Err(CodeownersMirrorFitnessError::UnknownOwner { path: r.path.clone(), owner: r.owner_axis.clone() });
        }
    }
    Ok(CodeownersMirrorFitnessReport { rules_checked: rules.len() })
}
```

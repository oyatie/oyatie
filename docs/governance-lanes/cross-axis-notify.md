---
doc_status: published
---

# Fitness Lane: cross-axis-notify

- status: Accepted
- date: 2026-05-12
- purpose: Verify every cross-axis contract change is accompanied by a notification entry naming each consumer axis owner.
- enforces: STANDARD/cross-axis-notify; AGENTS.md fitness-lane `governance-cross-axis-notify`.
- kernel_crate: `governance-cross-axis-notify-kernel` — `NotifyRecord { contract_id, notified_axes }`, verdict `CrossAxisNotifyFitnessReport { contracts_checked }`.
- runner_path: `tools/governance-cross-axis-notify`
- inputs: PR body `Cross-Axis-Notify:` section, contracts registry consumer axes.
- failure_modes:
  - contract changed but not all consumer_axes listed
  - notify row references unknown axis
  - empty notify row
- ci_invocation: `cargo run -p governance-cross-axis-notify`
- runtime_budget: 250 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct NotifyRecord {
    pub contract_id: String,        // data_class: INTERNAL_ONLY
    pub notified_axes: Vec<String>, // data_class: INTERNAL_ONLY
}
pub struct ContractRequirement {
    pub contract_id: String,         // data_class: INTERNAL_ONLY
    pub consumer_axes: Vec<String>,  // data_class: INTERNAL_ONLY
}

pub struct CrossAxisNotifyFitnessReport { pub contracts_checked: usize }

pub enum CrossAxisNotifyFitnessError {
    MissingNotify { contract_id: String },
    IncompleteAxes { contract_id: String, missing: Vec<String> },
    UnknownAxis { contract_id: String, axis: String },
}

pub fn validate_cross_axis_notify_fitness(
    notifies: &[NotifyRecord],
    requirements: &[ContractRequirement],
    known_axes: &[String],
) -> Result<CrossAxisNotifyFitnessReport, CrossAxisNotifyFitnessError> {
    let known: std::collections::BTreeSet<&str> = known_axes.iter().map(|s| s.as_str()).collect();
    let by_contract: std::collections::BTreeMap<&str, &NotifyRecord> =
        notifies.iter().map(|n| (n.contract_id.as_str(), n)).collect();
    for r in requirements {
        let n = by_contract.get(r.contract_id.as_str())
            .ok_or_else(|| CrossAxisNotifyFitnessError::MissingNotify { contract_id: r.contract_id.clone() })?;
        let missing: Vec<String> = r.consumer_axes.iter()
            .filter(|a| !n.notified_axes.contains(a)).cloned().collect();
        if !missing.is_empty() {
            return Err(CrossAxisNotifyFitnessError::IncompleteAxes {
                contract_id: r.contract_id.clone(), missing,
            });
        }
        for a in &n.notified_axes {
            if !known.contains(a.as_str()) {
                return Err(CrossAxisNotifyFitnessError::UnknownAxis {
                    contract_id: r.contract_id.clone(), axis: a.clone(),
                });
            }
        }
    }
    Ok(CrossAxisNotifyFitnessReport { contracts_checked: requirements.len() })
}
```

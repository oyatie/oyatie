---
doc_status: published
---

# Fitness Lane: cloud-mutation

- status: Accepted
- date: 2026-05-12
- purpose: Verify every cloud-mutation (create/update/delete cloud resource) goes through the sanctioned mutation kernel and carries an idempotency key.
- enforces: STANDARD/cloud-mutation; existing crate `intelligence-cloud-mutation-kernel` (EXISTING).
- adr_citations: ADR-0053 (sanctioned primitives — cloud mutations must route through the sanctioned kernel, not call provider SDKs directly)
- kernel_crate: `intelligence-cloud-mutation-kernel` (EXISTING) — `CloudMutation { mutation_id, kernel_path, idempotency_key }`, verdict `CloudMutationFitnessReport { mutations_checked }`.
- runner_path: `tools/governance-cloud-mutation`
- inputs: mutation registry, source AST showing cloud SDK calls.
- failure_modes:
  - cloud SDK called outside `intelligence-cloud-mutation-kernel`
  - mutation has no idempotency key
  - duplicate mutation id
- ci_invocation: `cargo run -p governance-cloud-mutation`
- runtime_budget: 800 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct CloudMutation {
    pub mutation_id: String,     // data_class: INTERNAL_ONLY
    pub kernel_path: String,     // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>, // data_class: INTERNAL_ONLY
}
pub struct CloudMutationFitnessReport { pub mutations_checked: usize }

pub enum CloudMutationFitnessError {
    OutsideSanctionedKernel { mutation_id: String, kernel_path: String },
    MissingIdempotencyKey { mutation_id: String },
    DuplicateMutationId { mutation_id: String },
}

pub fn validate_cloud_mutation_fitness(
    mutations: &[CloudMutation],
    sanctioned_kernel: &str,
) -> Result<CloudMutationFitnessReport, CloudMutationFitnessError> {
    let mut seen = std::collections::BTreeSet::new();
    for m in mutations {
        if !m.kernel_path.starts_with(sanctioned_kernel) {
            return Err(CloudMutationFitnessError::OutsideSanctionedKernel {
                mutation_id: m.mutation_id.clone(), kernel_path: m.kernel_path.clone(),
            });
        }
        if m.idempotency_key.is_none() {
            return Err(CloudMutationFitnessError::MissingIdempotencyKey { mutation_id: m.mutation_id.clone() });
        }
        if !seen.insert(m.mutation_id.clone()) {
            return Err(CloudMutationFitnessError::DuplicateMutationId { mutation_id: m.mutation_id.clone() });
        }
    }
    Ok(CloudMutationFitnessReport { mutations_checked: mutations.len() })
}
```

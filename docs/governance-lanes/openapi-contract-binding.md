---
doc_status: published
---

# Fitness Lane: openapi-contract-binding

- status: Accepted
- date: 2026-05-12
- purpose: Verify every public API has an OpenAPI spec + schema-bindings crate + runtime-bindings crate that compile against the same revision.
- enforces: STANDARD/api-contract-binding.
- kernel_crate: `governance-openapi-contract-binding-kernel` — `ApiContract { api_id, openapi_path, schema_crate, runtime_crate, schema_revision, runtime_revision }`, verdict `OpenApiContractBindingFitnessReport { apis_checked }`.
- runner_path: `tools/governance-openapi-contract-binding`
- inputs: API registry, OpenAPI specs, crate dependency tree.
- failure_modes:
  - openapi spec missing
  - schema-bindings crate missing
  - schema/runtime revisions out of sync
- ci_invocation: `cargo run -p governance-openapi-contract-binding`
- runtime_budget: 1200 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct ApiContract {
    pub api_id: String,            // data_class: INTERNAL_ONLY
    pub openapi_path: Option<String>, // data_class: INTERNAL_ONLY
    pub schema_crate: Option<String>, // data_class: INTERNAL_ONLY
    pub runtime_crate: Option<String>,// data_class: INTERNAL_ONLY
    pub schema_revision: Option<String>,// data_class: INTERNAL_ONLY
    pub runtime_revision: Option<String>,// data_class: INTERNAL_ONLY
}
pub struct OpenApiContractBindingFitnessReport { pub apis_checked: usize }

pub enum OpenApiContractBindingFitnessError {
    MissingSpec { api_id: String },
    MissingBindings { api_id: String, role: String },
    RevisionMismatch { api_id: String, schema: String, runtime: String },
}

pub fn validate_openapi_contract_binding_fitness(
    contracts: &[ApiContract],
) -> Result<OpenApiContractBindingFitnessReport, OpenApiContractBindingFitnessError> {
    for c in contracts {
        if c.openapi_path.is_none() { return Err(OpenApiContractBindingFitnessError::MissingSpec { api_id: c.api_id.clone() }); }
        if c.schema_crate.is_none() { return Err(OpenApiContractBindingFitnessError::MissingBindings { api_id: c.api_id.clone(), role: "schema".into() }); }
        if c.runtime_crate.is_none() { return Err(OpenApiContractBindingFitnessError::MissingBindings { api_id: c.api_id.clone(), role: "runtime".into() }); }
        let s = c.schema_revision.as_deref().unwrap_or("");
        let r = c.runtime_revision.as_deref().unwrap_or("");
        if s != r {
            return Err(OpenApiContractBindingFitnessError::RevisionMismatch {
                api_id: c.api_id.clone(), schema: s.to_string(), runtime: r.to_string(),
            });
        }
    }
    Ok(OpenApiContractBindingFitnessReport { apis_checked: contracts.len() })
}
```

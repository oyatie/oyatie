---
doc_status: published
---

# Fitness Lane: provider-agnostic

- status: Accepted
- date: 2026-05-12
- purpose: Verify no provider-specific code (AWS, GCP, Azure SDK calls / brand strings) leaks outside designated adapter crates.
- enforces: Directive 4 (MASTERPLAN) — provider-agnostic kernel.
- adr_citations: ADR-0053 (sanctioned primitives — provider SDK calls are a form of unsanctioned direct invocation; only adapter crates may call provider SDKs)
- kernel_crate: `governance-provider-agnostic-kernel` — `ProviderToken { path, line, token, in_adapter_crate }`, verdict `ProviderAgnosticFitnessReport { tokens_checked }`.
- runner_path: `tools/governance-provider-agnostic`
- inputs: source tree, adapter-crate allowlist, provider-token registry.
- failure_modes:
  - `aws_sdk_s3::Client` used in `*-kernel` crate
  - GCP project id literal in kernel
  - Azure SDK use in api crate
- ci_invocation: `cargo run -p governance-provider-agnostic`
- runtime_budget: 900 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct ProviderToken {
    pub path: String,             // data_class: INTERNAL_ONLY
    pub line: u32,                // data_class: INTERNAL_ONLY
    pub token: String,            // data_class: INTERNAL_ONLY
    pub in_adapter_crate: bool,   // data_class: INTERNAL_ONLY
}
pub struct ProviderAgnosticFitnessReport { pub tokens_checked: usize }

pub enum ProviderAgnosticFitnessError {
    LeakedProviderToken { path: String, line: u32, token: String },
}

pub fn validate_provider_agnostic_fitness(
    tokens: &[ProviderToken],
) -> Result<ProviderAgnosticFitnessReport, ProviderAgnosticFitnessError> {
    for t in tokens {
        if !t.in_adapter_crate {
            return Err(ProviderAgnosticFitnessError::LeakedProviderToken {
                path: t.path.clone(), line: t.line, token: t.token.clone(),
            });
        }
    }
    Ok(ProviderAgnosticFitnessReport { tokens_checked: tokens.len() })
}
```

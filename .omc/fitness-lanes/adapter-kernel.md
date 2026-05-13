# Fitness Lane: adapter-kernel

- purpose: Verify every adapter crate depends only on the sanctioned `oya-foundry-adapter-kernel` for shaping outbound provider IO.
- enforces: STANDARD/adapter-shape; existing crate `oya-foundry-adapter-kernel` (EXISTING).
- kernel_crate: `oya-foundry-adapter-kernel` (EXISTING) — extend with `AdapterCrate { crate_id, depends_on_adapter_kernel, calls_provider_sdk }`, verdict `AdapterKernelFitnessReport { adapters_checked }`.
- runner_path: `tools/oya-foundry-fitness-adapter-kernel`
- inputs: workspace dep graph, provider-SDK token list.
- failure_modes:
  - adapter crate calls SDK but does not depend on `oya-foundry-adapter-kernel`
  - non-adapter crate depends on `oya-foundry-adapter-kernel`
  - adapter crate exposes provider type publicly
- ci_invocation: `cargo run -p oya-foundry-fitness-adapter-kernel`
- runtime_budget: 800 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct AdapterCrate {
    pub crate_id: String,                 // data_class: INTERNAL_ONLY
    pub is_adapter: bool,                 // data_class: INTERNAL_ONLY
    pub depends_on_adapter_kernel: bool,  // data_class: INTERNAL_ONLY
    pub calls_provider_sdk: bool,         // data_class: INTERNAL_ONLY
    pub leaks_provider_type: bool,        // data_class: INTERNAL_ONLY
}
pub struct AdapterKernelFitnessReport { pub adapters_checked: usize }

pub enum AdapterKernelFitnessError {
    SdkWithoutAdapterKernel { crate_id: String },
    NonAdapterUsesKernel { crate_id: String },
    LeakedProviderType { crate_id: String },
}

pub fn validate_adapter_kernel_fitness(
    crates: &[AdapterCrate],
) -> Result<AdapterKernelFitnessReport, AdapterKernelFitnessError> {
    for c in crates {
        if c.calls_provider_sdk && !c.depends_on_adapter_kernel {
            return Err(AdapterKernelFitnessError::SdkWithoutAdapterKernel { crate_id: c.crate_id.clone() });
        }
        if !c.is_adapter && c.depends_on_adapter_kernel {
            return Err(AdapterKernelFitnessError::NonAdapterUsesKernel { crate_id: c.crate_id.clone() });
        }
        if c.leaks_provider_type {
            return Err(AdapterKernelFitnessError::LeakedProviderType { crate_id: c.crate_id.clone() });
        }
    }
    Ok(AdapterKernelFitnessReport { adapters_checked: crates.len() })
}
```

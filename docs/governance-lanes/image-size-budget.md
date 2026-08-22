---
doc_status: published
---

# Fitness Lane: image-size-budget

- status: Accepted
- date: 2026-05-12
- purpose: Verify every binary's distroless image stays within the declared size budget.
- enforces: Directive 5 (MASTERPLAN) — image size budget.
- kernel_crate: `governance-image-size-budget-kernel` — `ImageMeasurement { binary, image_mb, budget_mb }`, verdict `ImageSizeBudgetFitnessReport { images_checked }`.
- runner_path: `tools/governance-image-size-budget`
- inputs: image inspection (`docker inspect` JSON dump), `images-budget.toml`.
- failure_modes:
  - image grew > budget
  - image not in budget registry
  - budget not declared
- ci_invocation: `cargo run -p governance-image-size-budget`
- runtime_budget: 2000 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct ImageMeasurement {
    pub binary: String,    // data_class: INTERNAL_ONLY
    pub image_mb: f64,     // data_class: INTERNAL_ONLY
    pub budget_mb: f64,    // data_class: INTERNAL_ONLY
}
pub struct ImageSizeBudgetFitnessReport { pub images_checked: usize }
pub enum ImageSizeBudgetFitnessError {
    OverBudget { binary: String, image_mb: f64, budget_mb: f64 },
    MissingBudget { binary: String },
}

pub fn validate_image_size_budget_fitness(
    images: &[ImageMeasurement],
) -> Result<ImageSizeBudgetFitnessReport, ImageSizeBudgetFitnessError> {
    for i in images {
        if i.budget_mb <= 0.0 {
            return Err(ImageSizeBudgetFitnessError::MissingBudget { binary: i.binary.clone() });
        }
        if i.image_mb > i.budget_mb {
            return Err(ImageSizeBudgetFitnessError::OverBudget {
                binary: i.binary.clone(), image_mb: i.image_mb, budget_mb: i.budget_mb,
            });
        }
    }
    Ok(ImageSizeBudgetFitnessReport { images_checked: images.len() })
}
```

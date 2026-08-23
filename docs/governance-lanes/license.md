---
doc_status: published
---

# Fitness Lane: license

- status: Accepted
- date: 2026-05-12
- purpose: Verify every workspace crate declares an SPDX-valid `Apache-2.0` license.
- repository_default_boundary: Root [`/LICENSE`](../../LICENSE) is the repository-default proprietary IP posture; this lane verifies crate/package metadata and dependency-policy compatibility only, and does not grant a repository-wide open-source license.
- enforces: STANDARD/repo-license-policy; AGENTS.md fitness-lane `governance-license`.
- kernel_crate: `governance-license-kernel` — `CrateLicense { crate_id, license_expression, manifest_path }`, verdict `LicenseFitnessReport { crates_checked, missing, invalid }`.
- runner_path: `tools/governance-license`
- inputs: workspace `Cargo.toml`, every crate `Cargo.toml`, SPDX allowlist.
- failure_modes:
  - crate omits `license` key
  - crate uses `MIT OR Apache-2.0` dual-license without allowance
  - non-SPDX string (e.g., "Proprietary")
- ci_invocation: `cargo run -p governance-license`
- runtime_budget: 200 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct CrateLicense {
    pub crate_id: String,            // data_class: INTERNAL_ONLY
    pub license_expression: String,  // data_class: INTERNAL_ONLY
    pub manifest_path: String,       // data_class: INTERNAL_ONLY
}

pub struct LicenseFitnessReport {
    pub crates_checked: usize,
    pub missing: usize,
}

pub enum LicenseFitnessError {
    MissingLicense { crate_id: String },
    DisallowedExpression { crate_id: String, expression: String },
    NonSpdxString { crate_id: String },
}

pub fn validate_license_fitness<A>(
    crates: &[CrateLicense],
    allowed: A,
) -> Result<LicenseFitnessReport, LicenseFitnessError>
where
    A: IntoIterator,
    A::Item: AsRef<str>,
{
    let allowed = allowed.into_iter().map(|s| s.as_ref().to_string()).collect::<std::collections::BTreeSet<_>>();
    for c in crates {
        if c.license_expression.trim().is_empty() {
            return Err(LicenseFitnessError::MissingLicense { crate_id: c.crate_id.clone() });
        }
        if !allowed.contains(&c.license_expression) {
            return Err(LicenseFitnessError::DisallowedExpression {
                crate_id: c.crate_id.clone(),
                expression: c.license_expression.clone(),
            });
        }
    }
    Ok(LicenseFitnessReport { crates_checked: crates.len(), missing: 0 })
}
```

## Repository-default license boundary

The root [`/LICENSE`](../../LICENSE) is the repository-default IP posture: proprietary, all rights reserved, and not an open-source license. This governance lane remains intentionally narrower: it validates crate/package metadata and dependency/license-policy compatibility for Rust workspace members. An `Apache-2.0` package-manifest value or third-party notice applies only where an explicit file-level or component-level notice grants it; it does not relicense the repository as a whole.

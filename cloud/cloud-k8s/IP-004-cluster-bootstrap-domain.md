---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-004-cluster-bootstrap-domain
status: pending
execution_unit: ChangeSet
owner: axis-cloud
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-cloud-k8s-cluster-bootstrap-domain

## Intent

Pure domain crate: kubeadm-version compatibility arithmetic, etcd-snapshot integrity computation, upgrade-window math, BootstrapEvidence SHA computation, K8s-deprecation-API analysis. Zero I/O. Imports kernel only.

## ChangeSet boundary

One new Rust crate `microservices/cloud-k8s/src/crates/oya-cloud-k8s-cluster-bootstrap-domain/`. Catalog row.

## Concrete File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/version_compat.rs` | create — kubeadm 1.X → 1.X+1 compatibility matrix |
| `.../src/snapshot_integrity.rs` | create — Ed25519 verification logic (pure) |
| `.../src/upgrade_window.rs` | create — N-2 support window math |
| `.../src/evidence_sha.rs` | create — SHA-256 of (kubeadm output ⊕ component versions) |
| `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-domain.yaml` | create |

## Crate Naming

```
NAME: oya-cloud-k8s-cluster-bootstrap-domain
JUSTIFICATION:
- microservice = cloud-k8s; bc-tokens = cluster-bootstrap; layer = domain
- exemptions: none
```

## Code Shape

```rust
// src/version_compat.rs
use oya_cloud_k8s_cluster_bootstrap_kernel::entities::KubeadmConfig;

pub fn is_compatible_upgrade(from: &str, to: &str) -> Result<bool, VersionError> {
    let (from_maj, from_min) = parse(from)?;
    let (to_maj, to_min) = parse(to)?;
    Ok(from_maj == to_maj && (to_min == from_min || to_min == from_min + 1))
}

pub fn is_supported(version: &str, supported_window: &[&str]) -> bool { /* N-2 logic */ }
```

## Acceptance Gates

```bash
cargo check -p oya-cloud-k8s-cluster-bootstrap-domain
cargo build -p oya-cloud-k8s-cluster-bootstrap-domain
cargo clippy -p oya-cloud-k8s-cluster-bootstrap-domain -- -D warnings
cargo nextest run -p oya-cloud-k8s-cluster-bootstrap-domain
cargo deny check
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-k8s-cluster-bootstrap-domain
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-cloud-k8s-cluster-bootstrap-domain
```

## Test Plan

Per kernel + domain class: 1 test per public function + property tests for compatibility matrix. Coverage 95% line / 90% branch.

| Test | Verifies |
|---|---|
| `test_is_compatible_upgrade_n_to_n_plus_1` | only +1 minor allowed |
| `test_is_compatible_upgrade_n_to_n_plus_2_rejected` | skip-minor refused |
| `test_supported_window_n_minus_2` | N-2 support arithmetic |
| `test_evidence_sha_deterministic` | same input → same SHA |
| `test_evidence_sha_collision_rare` | property test 1k random inputs |

## Halt Conditions

- Any I/O reachable
- Any adapter / kernel-port impl present here — refactor to adapter

## Next IP

[`IP-005-cluster-bootstrap-usecase.md`](IP-005-cluster-bootstrap-usecase.md)

## References

- ADR-0121 §"Version pins"; ADR-0105 layer enum; PRD.

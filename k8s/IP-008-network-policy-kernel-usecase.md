---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-008-network-policy-kernel-usecase
status: pending
execution_unit: ChangeSet
owner: axis-cloud + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, check-cedar-derived-policy-paired]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: cloud-k8s-network-policy-{kernel,domain,usecase,adapter}

## Intent

Scaffold `network-policy` BC: Cedar → NetworkPolicy + Istio AuthorizationPolicy paired emission. Kernel (NetworkPolicyEmitter + AuthorizationPolicyEmitter ports + NetworkPolicy / AuthorizationPolicy / PeerSelector / TenantNamespace entities), domain (Cedar→CR translation logic; pair-symmetric validation), usecase (atomic apply orchestrator), adapter (kube-apiserver + Istio CR client).

## ChangeSet boundary

Four new Rust crates. Catalog rows.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-k8s/src/crates/cloud-k8s-network-policy-kernel/{Cargo.toml,src/*}` | create |
| `.../cloud-k8s-network-policy-domain/{Cargo.toml,src/{lib.rs,cedar_to_crs.rs,pair_validation.rs}}` | create |
| `.../cloud-k8s-network-policy-usecase/{Cargo.toml,src/{lib.rs,apply.rs,reconcile.rs}}` | create |
| `.../cloud-k8s-network-policy-adapter/{Cargo.toml,src/{lib.rs,k8s_networkpolicy_client.rs,istio_authpolicy_client.rs}}` | create |
| `microservices/cloud-k8s/catalog/cloud-k8s-network-policy-{kernel,domain,usecase,adapter}.yaml` | create |

## Crate Naming

```
NAMES: cloud-k8s-network-policy-{kernel,domain,usecase,adapter}
JUSTIFICATION: microservice + bc + layer per ADR-0105; exemptions: none
```

## Code Shape

```rust
// domain/src/cedar_to_crs.rs
use cedar_policy::Policy;
use cloud_k8s_network_policy_kernel::entities::*;

pub fn derive_pair(
    cedar_fragment: &Policy,
    tenant_namespace: &str,
) -> Result<(NetworkPolicy, AuthorizationPolicy), TranslationError> {
    let np_spec = derive_network_policy(cedar_fragment, tenant_namespace)?;
    let ap_spec = derive_authorization_policy(cedar_fragment, tenant_namespace)?;
    validate_pair_symmetric(&np_spec, &ap_spec)?;  // ALWAYS-PAIRED invariant CI-06
    Ok((np_spec, ap_spec))
}
```

```rust
// usecase/src/apply.rs
pub struct ApplyUseCase<NPE, APE>
where NPE: NetworkPolicyEmitter, APE: AuthorizationPolicyEmitter {
    np_emitter: NPE,
    ap_emitter: APE,
}

impl<NPE, APE> ApplyUseCase<NPE, APE> where NPE: NetworkPolicyEmitter, APE: AuthorizationPolicyEmitter {
    pub async fn apply_pair(&self, cedar_sha: String, namespace: String,
                            np: NetworkPolicy, ap: AuthorizationPolicy) -> Result<ApplyResult, UseCaseError> {
        // Atomic: emit both or rollback. The LEAN check `check-cedar-derived-policy-paired`
        // verifies the pair invariant at CI time.
    }
}
```

## Acceptance Gates

```bash
for crate in network-policy-{kernel,domain,usecase,adapter}; do
  cargo check -p cloud-k8s-$crate
  cargo nextest run -p cloud-k8s-$crate
done
cargo run -p dev-cli -- gate validate cedar-derived-policy-paired --microservice cloud-k8s
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_cedar_to_pair_simple_allow` | Cedar `permit (...)` → NetworkPolicy + AuthorizationPolicy |
| `test_cedar_to_pair_cross_namespace_allow` | cross-ns paired |
| `test_pair_validation_rejects_np_without_ap` | always-paired invariant |
| `test_apply_atomic_rollback_on_failure` | atomicity |
| `test_idempotent_re_apply` | re-apply same Cedar = no-op |

## Halt Conditions

- NetworkPolicy emitted without paired AuthorizationPolicy — refuse
- Any direct k8s API write bypassing usecase — refactor

## Next IP

[`IP-009-service-mesh-control-plane-istio.md`](IP-009-service-mesh-control-plane-istio.md)

## References

- ADR-0140 (retired per ADR-0145) (Cedar); ADR-0121; `policy/cluster-isolation.md` CI-06.
- Cedar — `docs.cedarpolicy.com`.

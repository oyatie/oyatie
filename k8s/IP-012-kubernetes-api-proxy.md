---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-012-kubernetes-api-proxy
status: pending
execution_unit: ChangeSet
owner: axis-cloud + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, oya-check-kubernetes-api-proxy-only-path]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: oya-cloud-k8s-kubernetes-api-proxy-{kernel,usecase,adapter,rest,worker,sdk,app}

## Intent

Scaffold `kubernetes-api-proxy` BC — the HTTP reverse-proxy that mediates every kube-apiserver call with Cedar policy decisions + audit-chain emission. Implements invariant CI-07 (no direct 6443 access) and CI-08 (Cedar on every call). 7 crates: kernel + usecase + adapter + rest + worker + sdk + app.

## ChangeSet boundary

7 new crates. Catalog rows. Helm deployment added to IP-001's kustomize base.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-k8s/src/crates/oya-cloud-k8s-kubernetes-api-proxy-kernel/{Cargo.toml,src/*}` | create — ApiCall, CallerPrincipal, PolicyDecision, AuditRecord entities + ApiCallMediator port |
| `.../oya-cloud-k8s-kubernetes-api-proxy-usecase/{Cargo.toml,src/{lib.rs,mediate.rs,cedar_eval.rs,audit_emit.rs}}` | create |
| `.../oya-cloud-k8s-kubernetes-api-proxy-adapter/{Cargo.toml,src/{lib.rs,reverse_proxy.rs,kube_upstream.rs,audit_chain_client.rs}}` | create |
| `.../oya-cloud-k8s-kubernetes-api-proxy-rest/{Cargo.toml,src/{lib.rs,routes.rs}}` | create — receives kubectl-shape traffic |
| `.../oya-cloud-k8s-kubernetes-api-proxy-worker/{Cargo.toml,src/{lib.rs,policy_cache_refresher.rs}}` | create — refresh Cedar fragments |
| `.../oya-cloud-k8s-kubernetes-api-proxy-sdk/{Cargo.toml,src/{lib.rs,client.rs}}` | create — operator + Foundry SDK |
| `.../oya-cloud-k8s-kubernetes-api-proxy-app/{Cargo.toml,src/main.rs}` | create — composition root binary |
| `microservices/cloud-k8s/catalog/oya-cloud-k8s-kubernetes-api-proxy-{kernel,usecase,adapter,rest,worker,sdk,app}.yaml` | create |

## Crate Naming

```
NAMES: oya-cloud-k8s-kubernetes-api-proxy-{kernel,usecase,adapter,rest,worker,sdk,app}
JUSTIFICATION: 7 layers per ADR-0105; no exemptions.
```

## Code Shape

```rust
// usecase/src/mediate.rs
pub struct MediateUseCase<M> where M: ApiCallMediator {
    mediator: M,
}

impl<M> MediateUseCase<M> where M: ApiCallMediator {
    pub async fn mediate(&self, call: ApiCall, principal: CallerPrincipal) -> Result<UpstreamResponse, MediateError> {
        // 1. Validate JWT / SPIFFE SVID
        // 2. Evaluate Cedar policy fragments (tenant-scope / ci-scope / auditor-scope / public-read)
        // 3. If DENY: audit-emit + return 403
        // 4. If ALLOW: forward to kube-apiserver upstream
        // 5. Capture upstream response; audit-emit
        // 6. Return response + audit-emission status
    }
}
```

## Acceptance Gates

```bash
for crate in kubernetes-api-proxy-{kernel,usecase,adapter,rest,worker,sdk,app}; do
  cargo check -p oya-cloud-k8s-$crate
  cargo nextest run -p oya-cloud-k8s-$crate
done
cargo run -p oya-dev-cli -- gate validate kubernetes-api-proxy-only-path --microservice cloud-k8s
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_mediate_authorized_call_forwards` | happy path |
| `test_mediate_unauthorized_denies_with_403` | Cedar enforcement |
| `test_mediate_invalid_spiffe_returns_401` | auth validation |
| `test_mediate_audit_chain_emit_per_call` | invariant CI-09 |
| `test_direct_6443_access_refused` | NetworkPolicy probe (e2e) |
| `test_decision_latency_p99_under_50ms` | perf SLO |
| `test_cedar_fragment_hot_reload` | worker refreshes from git |
| `test_reserved_namespace_writes_refused_for_tenants` | invariant CI-08 |

## Halt Conditions

- Any path that bypasses Cedar evaluation — refuse
- Any audit-chain emission failure that doesn't fail-closed — refactor
- Decision latency p99 > 50ms — perf-budget breach

## Next IP

[`IP-013-cluster-bootstrap-rest-worker-sdk-app.md`](IP-013-cluster-bootstrap-rest-worker-sdk-app.md)

## References

- ADR-0121; ADR-0140 (retired per ADR-0145) (Cedar); ADR-0028 (audit-chain).
- `k8s/policy/cluster-isolation.md` CI-07, CI-08, CI-09.
- Cedar — `docs.cedarpolicy.com`.

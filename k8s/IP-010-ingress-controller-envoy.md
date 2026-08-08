---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-010-ingress-controller-envoy
status: pending
execution_unit: ChangeSet
owner: axis-cloud
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, oya-check-envoy-tls-13-only]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: oya-cloud-k8s-ingress-controller-{kernel,usecase,adapter-envoy}

## Intent

Scaffold `ingress-controller` BC: kernel (EnvoyConfigurer port + Gateway / VirtualService / DestinationRule / TlsCertificate / SniRoute entities), usecase (apply/update/delete orchestrators with SNI validation), adapter-envoy (Gateway / VirtualService / DestinationRule CR client).

## ChangeSet boundary

Three new crates: kernel, usecase, adapter-envoy. Catalog rows.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-k8s/src/crates/oya-cloud-k8s-ingress-controller-kernel/{Cargo.toml,src/*}` | create |
| `.../oya-cloud-k8s-ingress-controller-usecase/{Cargo.toml,src/{lib.rs,apply_gateway.rs,apply_virtualservice.rs}}` | create |
| `.../oya-cloud-k8s-ingress-controller-adapter-envoy/{Cargo.toml,src/{lib.rs,gateway_cr.rs,vs_cr.rs,dr_cr.rs}}` | create |
| `microservices/cloud-k8s/catalog/oya-cloud-k8s-ingress-controller-{kernel,usecase,adapter-envoy}.yaml` | create |

## Crate Naming

```
NAMES: oya-cloud-k8s-ingress-controller-{kernel,usecase,adapter-envoy}
JUSTIFICATION: layers per ADR-0105; adapter-envoy backend-qualified; no exemptions.
```

## Code Shape

```rust
// usecase/src/apply_gateway.rs
pub struct ApplyGatewayUseCase<E> where E: EnvoyConfigurer {
    envoy: E,
}

impl<E> ApplyGatewayUseCase<E> where E: EnvoyConfigurer {
    pub async fn apply(&self, gw: Gateway) -> Result<ApplyResult, UseCaseError> {
        // 1. Validate TLS 1.3 only (refuse if TLS 1.2 listener)
        // 2. Validate hosts: list (refuse wildcards unless authorised)
        // 3. Verify cert SAN matches host(s)
        // 4. Emit Gateway CR
        // 5. Audit-chain emit
    }
}
```

## Acceptance Gates

```bash
for crate in ingress-controller-{kernel,usecase,adapter-envoy}; do
  cargo check -p oya-cloud-k8s-$crate
  cargo nextest run -p oya-cloud-k8s-$crate
done
cargo run -p oya-dev-cli -- gate validate envoy-tls-13-only --microservice cloud-k8s
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_apply_gateway_tls_13_only` | TLS 1.2 listener refused |
| `test_apply_gateway_sni_san_matches_hosts` | cert SAN check |
| `test_wildcard_host_refused_unless_authorized` | wildcard guard |
| `test_apply_virtualservice_routes_validated` | route correctness |
| `test_apply_destinationrule_subsets_validated` | subset shape |

## Halt Conditions

- Any path that permits TLS < 1.3 — refuse
- Any SNI host not validated against SAN — refuse

## Next IP

[`IP-011-csi-storage-driver-per-backend.md`](IP-011-csi-storage-driver-per-backend.md)

## References

- ADR-0121 §"Envoy"; threat-model T-S-03; runbook envoy-sni-debug.
- Envoy TLS — `envoyproxy.io/docs/envoy/latest/intro/arch_overview/security/tls`.

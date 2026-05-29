---
doc_class: ImplementationPlan
id: IP-015
title: "oya-payments-charge-rest + grpc + app — composition root, OpenAPI surface, gRPC surface"
microservice: payments
bounded_context: charge
layer: app
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤800 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0145
  - ADR-0248
  - ADR-0253
  - ADR-0254
diataxis_quadrant: how-to
doc_status: published
---

# IP-015 — oya-payments-charge-rest + grpc + app

## Purpose

Wire the composition root for the charge-BC: `oya-payments-charge-rest` (OpenAPI 3.2.0 surface), `oya-payments-charge-grpc` (proto3 surface), and `oya-payments-charge-app` (K8s binary entry point). Includes health, readiness, and Cedar-gate middleware.

## Acceptance criteria

- [ ] `oya-payments-charge-rest` exposes routes per `contracts/openapi-v1.yaml`: `POST /v1/charges`, `GET /v1/charges/{id}`, `POST /v1/charges/{id}/capture`, `POST /v1/charges/{id}/void`, `GET /v1/charges` (paginated).
- [ ] `oya-payments-charge-grpc` implements `PaymentsService` from `contracts/payments-v1.proto`.
- [ ] HTTP/3 server via `hyper` + `h3` crate; fallback to HTTP/2 via `hyper` TLS (Alt-Svc header set per ADR-0253).
- [ ] Middleware stack (in order): SVID verification → Cedar gate → fraud-score enrichment → usecase dispatch → audit emit.
- [ ] `oya-payments-charge-app` binary: Tokio multi-thread runtime; Cloud Hypervisor + Kata pod shape per ADR-0254; graceful shutdown (SIGTERM → 30s drain).
- [ ] `/healthz` (liveness) + `/readyz` (readiness) + `/metrics` (Prometheus scrape) endpoints on port 9090.
- [ ] K8s Deployment manifest at `iac/helm/payments-app/templates/deployment.yaml`.
- [ ] Integration smoke-test: `POST /v1/charges` → 201 with idempotency-key; replay → 200 same body.
- [ ] `cargo clippy` zero warnings.

## Dependencies

- IP-001 through IP-004 (kernel, domain, usecase, Stripe adapter) must be merged first.

## Composition root wiring

```rust
// oya-payments-charge-app/src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secrets = OpenBaoSecrets::from_env()?;
    let policy_eval = LibraryFirstPolicyEval::new(CEDAR_FRAGMENT_PATH)?;
    let intelligence = LibraryFirstIntelligence::new()?;
    let ontology = LibraryFirstOntology::new()?;
    let db = CrdbPool::from_env()?;
    let psp = StripeAdapter::new(secrets.clone());
    let charge_repo = CrdbChargeRepository::new(db.clone());
    let usecase = CreateChargeUseCase::new(policy_eval, intelligence, ontology, psp, charge_repo);
    let router = charge_rest::router(usecase);
    serve_h3(router, "[::]:8443").await
}
```

## Hyperscaler precedent

Stripe's own gateway uses Envoy + gRPC internally; REST surface for external callers. Same two-surface pattern here.

## Cross-references

- `contracts/openapi-v1.yaml` — REST surface spec.
- `contracts/payments-v1.proto` — gRPC surface spec.
- `iac/helm/payments-app/values.yaml` — K8s deployment config.
- `slos/charge-api-availability.openslo.yaml` — SLO gating this surface.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-015-payments-rest-grpc-app.md` matched `SLO, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.

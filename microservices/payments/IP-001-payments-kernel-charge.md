---
doc_class: ImplementationPlan
id: IP-001
title: "oya-payments-charge-kernel — port traits, entity types, value objects"
microservice: payments
bounded_context: charge
layer: kernel
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤400 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0145
  - ADR-0243
  - ADR-0244
diataxis_quadrant: how-to
doc_status: published
---

# IP-001 — oya-payments-charge-kernel

## Purpose

Scaffold the `oya-payments-charge-kernel` crate: sealed port traits, value objects, error types, and domain-event envelopes for the `charge` bounded context. Zero I/O. This is the root dependency for all charge-BC layers.

## Acceptance criteria

- [ ] Crate `oya-payments-charge-kernel` compiles with `cargo build -p oya-payments-charge-kernel`.
- [ ] `PspAdapter` trait defined with all method signatures per `ARCHITECTURE.md §D`.
- [ ] `ChargeId`, `PaymentMethodId`, `IdempotencyKey`, `Currency`, `AmountMinor` value objects defined; `Currency` validated against ISO 4217 at construction.
- [ ] `ChargeState` enum: `Authorized | Captured | Voided | Declined | Errored`.
- [ ] `AudienceType` enum: `B2bTenant | B2cConsumer | PartnerAgency`.
- [ ] `ChargeError` / `PspError` error types using `thiserror`; no `anyhow` in kernel.
- [ ] `ChargeCreatedEvent`, `ChargeCapturedEvent`, `ChargeDeclinedEvent`, `ChargeErroredEvent` domain events with `tenant_id: TenantId`, `charge_id: ChargeId`, `audit_chain_seq: u64`.
- [ ] `AuthorizeRequest`, `AuthorizeResponse`, `CaptureRequest`, `CaptureResponse`, `RefundRequest`, `RefundResponse`, `PayoutRequest`, `PayoutResponse`, `WebhookPayload` request/response types.
- [ ] No `tokio`, no DB, no HTTP in this crate.
- [ ] `cargo clippy` zero warnings (deny(warnings)).
- [ ] Unit tests ≥ 5 (value object validation, state machine guards).

## Dependencies

None (kernel is the leaf node). This IP MUST land before IP-002 through IP-005.

## Implementation notes

```toml
# Cargo.toml
[package]
name = "oya-payments-charge-kernel"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "2"
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v7", "serde"] }
rust_decimal = "1"

[dev-dependencies]
rstest = "0.23"
```

Crate layout:
```
src/
  lib.rs
  ports.rs        ← PspAdapter + PolicyEvalPort + AuditEmitPort
  entities.rs     ← Charge, ChargeAttempt, PaymentMethod, CardFingerprint
  value_objects.rs
  events.rs
  errors.rs
```

`PspAdapter` is `#[async_trait]`; `Send + Sync + 'static` bounds required for composition root in `app` layer.

## Hyperscaler precedent

Stripe's charge object model (id, amount, currency, status, payment_method, idempotency_key) is the reference shape. Adyen's `PaymentRequest` adds `merchantReference` (idempotency-key equivalent).

## Cross-references

- `ARCHITECTURE.md §D` — PspAdapter trait definition.
- `IP-002-payments-domain-charge.md` — consumes this kernel.
- `contracts/payments-v1.proto` — gRPC surface that maps these types.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-001-payments-kernel-charge.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.

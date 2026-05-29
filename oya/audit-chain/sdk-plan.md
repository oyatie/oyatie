---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-audit-chain + gtm-customer-success
deciders: axis-audit-chain, council-architecture
related_adrs: [ADR-0028, ADR-0003, ADR-0131]
related_artifacts:
  - microservices/audit-chain/contracts/openapi/audit-chain.yaml
  - microservices/audit-chain/contracts/proto/audit-chain.proto
  - microservices/audit-chain/PRD.md (FR-08)
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (audit-chain µservice)

## Purpose

Every oyatie µservice must emit to audit-chain — the SDK is **the load-bearing primitive** for SOC 2 / ISO 27001 / GDPR / HIPAA / KR PIPA evidence emission. Tenants who run side-cars or workloads in their own cluster also consume the SDK for tenant-internal events. This document specifies SDK strategy: which languages, generation, guarantees, and sunset policy.

## Languages

| Language | Priority | Generation | Authority |
|---|---|---|---|
| **Rust** | M01 (primary; oyatie's own language) | First-party `oya-audit-chain-{emission,verification,query}-sdk` crates | axis-audit-chain |
| **TypeScript** | M01 (every workload pipeline / tenant-side Node) | OpenAPI-generated baseline + first-party ergonomic wrappers | axis-audit-chain + gtm |
| **Python** | M01+1 (data + ML workloads) | OpenAPI-generated + ergonomic wrappers | axis-audit-chain + gtm |
| **Go** | M02 | gRPC-generated + ergonomic wrappers | axis-audit-chain + gtm |
| **JVM (Kotlin / Java)** | M02 | gRPC + ergonomic wrappers; Maven Central | axis-audit-chain + gtm |
| **C# / .NET** | M03 | OpenAPI; NuGet | axis-audit-chain + gtm |
| **Ruby** | M04 (only if tenant demand) | OpenAPI | axis-audit-chain |

Prioritisation: oyatie µservice languages first; TypeScript prioritised over typical SDK order because tenant-side workloads + frontend agents commonly emit audit events.

## Generation Strategy

### Rust SDK (first-party)

Three crates per BC:
- `oya-audit-chain-emission-sdk` — `AuditEmitter::emit(event) -> EmitReceipt` with built-in retry, idempotency-key generation, payload-class validation.
- `oya-audit-chain-verification-sdk` — `verify(event_envelope, proof, root, signature) -> Verdict` with embedded KeyResolver (reads published roots from S3 OR GitHub-pinned manifest).
- `oya-audit-chain-query-sdk` — `query(tenant_id, time_range, filter) -> QueryResult; export_request(...) -> ExportBundle`.

Properties:
- `Client::new(opts)` — bound to a tenant + pack at construction; SPIFFE identity loaded from environment.
- Built-in exponential backoff for transient 5xx + 429.
- Idempotency-key auto-generated if caller doesn't supply; ULID-shaped.
- Payload-class declared by caller; SDK refuses unannotated payloads at construction time.
- No `unsafe`; `#![deny(unsafe_code)]`.
- Verification SDK is **standalone** — does not require audit-chain network access; reads public manifests + verifies offline. Reference implementation for tenant + auditor + public verifiers.

### Generated SDKs

Generation pipeline (lives in `microservices/audit-chain/sdk-generation/`):

1. Source: `contracts/openapi/audit-chain.yaml` + `contracts/proto/audit-chain.proto`.
2. OpenAPI: `openapi-generator-cli` 7.x.
3. Proto: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer.
5. Embedded verification SDK: each language ships an offline verifier matching the Rust reference (same RFC 6962 + RFC 8032 logic).
6. Per-language CI lane: build + lint + integration-test against staging audit-chain cluster.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Emit audit event | `emit(event)` | `EmitReceipt { event_id, period_id, sealed: bool }` |
| Verify event proof | `verify(envelope, proof, root, signature)` | `Verdict { verified, reason }` |
| Query own-tenant audit | `query(filter)` | `QueryResult[]` |
| Request export bundle | `requestExport(time_range, scoped_event_classes)` | `ExportBundleHandle` |
| Read inclusion proof for event | `getProof(event_id)` | `MerkleProof` |
| Read latest signed root | `getRoot(pack, period_id)` | `SignedRoot` |

Verification SDK additionally exposes:
| Capability | Method | Returns |
|---|---|---|
| Resolve key for period | `keyForPeriod(pack, period_id)` | `PublicKey` |
| Independent full chain verify | `verifyChain(pack, partition, from, to)` | `ChainVerdict` |

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue per-tenant SPIFFE-bound certificate via OpenBao | ops-security |
| Provide quick-start (per language) | gtm-customer-success |
| Sample integration: how to emit from tenant-side workflows | axis-audit-chain |
| Quarterly SDK update notification (breaking changes 6mo advance) | axis-audit-chain |

## Open-Source Decision

**The verification SDK SHOULD be open-source.** Reason: external auditor + tenant + public ability to independently verify the chain is part of the trust posture (Bominal ADR-0028 §"External transparency"). Decision: open-source the verification SDK with the next quarter's release; emission + query SDKs remain proprietary until stable + tenant-driven request.

This is a stronger open-source posture than observability — driven by the audit-chain's trust model.

## Sunset Policy

Same shape as `observability/sdk-plan.md` §"Sunset Policy". Notably:
- Verification SDK MUST maintain backward compatibility with ALL historical key epochs (∞ retention of public keys → ∞ verification capability).

## Versioning

Match audit-chain major.minor; SDK patch independent. Compatibility matrix verified by CI lane.

## Verification

- Per-SDK CI lane.
- Cross-SDK signature-equivalence: `cargo run -p oya-dev-cli -- gate validate audit-chain-sdk-equivalence` — exit 0; every generated SDK's emit + verify produces identical wire outputs for the same input set.
- Annual SDK telemetry review.

## References

- `microservices/audit-chain/contracts/openapi/audit-chain.yaml`.
- `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- `microservices/audit-chain/PRD.md` BC layer mapping.
- ADR-0105 (13-layer enum; `sdk` is canonical layer).
- Bominal ADR-0028 §"External transparency" + ADR-0003 §"SDK contract".
- OpenAPI Generator; gRPC tooling.
- Stripe + Twilio SDK precedents.

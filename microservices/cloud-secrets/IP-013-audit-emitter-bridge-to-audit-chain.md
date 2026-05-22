---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-013-audit-emitter-bridge-to-audit-chain
status: pending
owner: axis-cloud-secrets + axis-governance
acceptance_lanes: [audit-seal-e2e]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: audit-emitter bridge to audit-chain

## Intent

Bridge the local OpenBao audit-device file → `audit-chain` µservice with Ed25519 signing per Bominal ADR-0028.

## ChangeSet boundary

Five new crates: kernel, usecase, api, adapter-audit-chain-bridge, app.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-audit-emitter-kernel/` | `SecretAuditEvent`, `AuditChainBridgeMessage` |
| `…/oya-cloud-secrets-audit-emitter-usecase/` | orchestrate file-tail → sign → bridge |
| `…/oya-cloud-secrets-audit-emitter-api/` | typed contracts |
| `…/oya-cloud-secrets-audit-emitter-adapter-audit-chain-bridge/` | bridge HTTP client (audit-chain) |
| `…/oya-cloud-secrets-audit-emitter-app/` | bridge worker binary |
| 5× catalog yamls | create |

## Acceptance Gates

```bash
cargo nextest run -p 'oya-cloud-secrets-audit-emitter-*'
# E2E with mock audit-chain
cargo nextest run --features audit-seal-e2e
```

## Test Plan

- Every SecretAccessed event reaches audit-chain within p99 ≤1s.
- audit-chain outage: local file durable; bridge resumes on recovery.
- Replay idempotent: dedup via `(event_id, signature)`.

## Halt Conditions

- Audit events sent unsigned — BLOCKER.
- Local file capped without rotation — BLOCKER.

## Next IP

`IP-014-observability-slo-branch-protection-hg-cloud-secrets.md`

## Wave 15-IP-substance A-G

### A. Problem
Secret access is only defensible if every create, read, rotate, revoke, namespace, and KEK event reaches audit-chain with tamper-evident signatures. OpenBao audit devices alone do not satisfy Oyatie's bilateral audit-chain posture.

### B. Approach
Bridge OpenBao audit-device output into the `audit-chain` microservice through a typed adapter. The bridge canonicalizes events, signs with Ed25519, buffers durably under backpressure, and replays idempotently after outages.

### C. Deliverables
- `oya-cloud-secrets-audit-emitter-{kernel,usecase,api,adapter-audit-chain-bridge,app}`.
- Event schema alignment with `contracts/asyncapi/cloud-secrets-events.yaml`.
- Audit completeness SLO in `slos/audit-log-completeness.openslo.yaml`.
- Dashboard `dashboards/audit-emission-completeness.json`.
- Runbook `runbooks/audit-emission-backlog.md`.

### D. Ordered Implementation Steps
1. Define `SecretAuditEvent` and canonical serialization in the kernel crate.
2. Map OpenBao audit-device fields into typed event classes.
3. Sign events and include tenant, pack, path hash, principal, action, and outcome.
4. Implement durable local buffering with bounded backpressure behavior.
5. Implement audit-chain bridge adapter and idempotent replay by event id/signature.
6. Emit metrics for sealed, pending, failed, and replayed events.
7. Add e2e tests with mock audit-chain outage and recovery.

### E. Acceptance
- `cargo nextest run -p 'oya-cloud-secrets-audit-emitter-*'`.
- `cargo nextest run --features audit-seal-e2e`.
- Every `SecretAccessed` event reaches audit-chain within the p99 SLO or enters durable backlog.
- Unsigned events and unbounded local files are blockers.

### F. Evidence
Evidence anchors are `PRD.md` FR-06/FR-10, `manifest.json`, `catalog/oya-cloud-secrets-audit-emitter-adapter-audit-chain-bridge.yaml`, `contracts/asyncapi/cloud-secrets-events.yaml`, `slos/audit-log-completeness.openslo.yaml`, and `runbooks/audit-emission-backlog.md`.

### G. Counterpart Comparison
Vault audit devices, AWS CloudTrail, Google Cloud Audit Logs, Azure Monitor, OCI Audit, and Akeyless activity logs provide access records. The parity matrix says Oyatie's differentiator is Merkle + Ed25519 non-repudiation and per-pack audit residency; this bridge is the concrete implementation point.

Grep-recognized counterpart anchor: GitHub Actions Secrets is relevant when CI jobs emit secret-access test events that must be sealed rather than merely masked in logs. The main comparator remains vendor audit logs and Vault audit devices.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/IP-013-audit-emitter-bridge-to-audit-chain.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-013-audit-emitter-bridge-to-audit-chain.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/cloud-secrets/manifest.json#paid_billing_components_emitted` is absent; this section is triggered by IP text and must be reconciled with the manifest billing model.
- Surface evidence: `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-013-audit-emitter-bridge-to-audit-chain.md`.

---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-006-resolver-adapter-openbao
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [cargo-test, integration-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-cloud-secrets-secret-reference-resolver-adapter-openbao

## Intent

Implement `OpenBaoClient` port against OpenBao 2.x HTTP API. mTLS via SPIFFE SVID. Per-tenant Kubernetes auth method backed authentication.

## ChangeSet boundary

One new crate at `…/oya-cloud-secrets-secret-reference-resolver-adapter-openbao/`. Adapter-* per ADR-0105 Amendment 3.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/Cargo.toml` | create — deps: `reqwest`, `rustls`, `spiffe`, async-trait |
| `…/src/lib.rs` | create |
| `…/src/client.rs` | create — `OpenBaoHttpClient { http: reqwest::Client, endpoint: Url, svid_provider: SpiffeProvider }` |
| `…/src/auth.rs` | create — Kubernetes auth method exchange (SVID → OpenBao token) |
| `…/src/kv_v2.rs` | create — KV v2 read/write/list |
| `…/src/revoke.rs` | create — revoke endpoint |
| `…/src/sse.rs` | create — server-sent events client for revocation push |
| `microservices/cloud-secrets/catalog/oya-cloud-secrets-secret-reference-resolver-adapter-openbao.yaml` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-adapter-openbao
# Integration test against sandbox OpenBao:
docker-compose -f microservices/cloud-secrets/tests/integration/docker-compose.yml up -d
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-adapter-openbao --features integration
```

## Test Plan

- Integration: read/write/list/revoke against real OpenBao 2.x instance via testcontainers.
- SSE: simulate disconnect + reconnect; verify catch-up via `since=<last-event-id>`.

## Halt Conditions

- mTLS handshake without SPIFFE SVID validation — BLOCKER.

## Next IP

`IP-007-resolver-rest-and-sdk-rust.md`

## Wave 15-IP-substance A-G

### A. Problem
OpenBao is the backing secret engine, but direct HTTP calls from product code would bypass SPIFFE identity checks, tenant namespace binding, retry discipline, and audit-aware error mapping.

### B. Approach
Implement one adapter for the kernel `OpenBaoClient` port. It owns mTLS, Kubernetes auth exchange, KV v2 path translation, lease/revoke calls, and server-sent revocation events while presenting typed domain errors to the usecase layer.

### C. Deliverables
- `oya-cloud-secrets-secret-reference-resolver-adapter-openbao` crate from `manifest.json`.
- `src/client.rs`, `src/auth.rs`, `src/kv_v2.rs`, `src/revoke.rs`, and `src/sse.rs`.
- Catalog file `catalog/oya-cloud-secrets-secret-reference-resolver-adapter-openbao.yaml`.
- Integration fixtures for sandbox OpenBao and testcontainers.
- Policy and residency alignment with `policy/data-residency.md` and `multi-region.md`.

### D. Ordered Implementation Steps
1. Build a reqwest client requiring SPIFFE SVID-backed mTLS.
2. Exchange workload identity for a tenant-scoped OpenBao token.
3. Translate normalized SecretReference paths into KV v2 API calls under the tenant namespace.
4. Map OpenBao errors into typed retry/deny/backend-unavailable categories.
5. Implement lease revoke and prefix revoke operations for cascade rotation.
6. Implement SSE revocation stream with reconnect and last-event-id catch-up.
7. Add integration tests against sandbox OpenBao and SoftHSM-backed unseal fixtures.

### E. Acceptance
- `cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-adapter-openbao`.
- Integration run with `--features integration` against sandbox OpenBao.
- mTLS without SPIFFE validation is impossible.
- All read/list/revoke calls include tenant namespace headers and never log response bodies.

### F. Evidence
Evidence anchors are `PRD.md` FR-02/FR-08, `ARCHITECTURE.md` adapter-openbao mapping, `manifest.json`, `catalog/oya-cloud-secrets-secret-reference-resolver-adapter-openbao.yaml`, `policy/tenant-scope.cedar`, `policy/data-residency.md`, and `runbooks/openbao-restart.md`.

### G. Counterpart Comparison
Vault clients, AWS IAM, Google IAM, and Azure managed identity all solve workload access differently. The cloud-secrets matrices show Oyatie's target is SPIFFE+mTLS plus per-tenant namespace isolation and revocation push, so this adapter must be stricter than a generic vendor client wrapper.

Grep-recognized counterpart anchor: GitHub Actions Secrets is mentioned only for CI secret distribution into adapter integration tests, where workflow credentials must never bypass SPIFFE, mTLS, or namespace checks. The adapter's primary counterpart remains OpenBao/Vault client behavior.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-006-resolver-adapter-openbao.md`.

## Pod runtime tier (per ADR-0338)

- `pod_runtime_tier: 0`.
- Justification: tenant-customer code is present in this IP's execution path; Tier 0 requires Kata plus Cloud Hypervisor isolation.
- Surface evidence: `microservices/cloud-secrets/IP-006-resolver-adapter-openbao.md`; matched trigger term(s): `sandbox`.
- Admission expectation: spawned workloads for this path use `kata-cloud-hypervisor`; first-party helpers may only run outside Tier 0 when split into a separate non-tenant-customer IP.

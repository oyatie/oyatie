---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-005-resolver-usecase
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [cargo-test, lean-a1, lean-a2]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-cloud-secrets-secret-reference-resolver-usecase

## Intent

Orchestrate the resolve flow: parse URI → check cache → on miss query OpenBao → policy-eval → emit audit → cache result → return.

## ChangeSet boundary

One new crate; depends on `-kernel` + `-domain`.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-secret-reference-resolver-usecase/Cargo.toml` | create |
| `…/src/lib.rs` | create |
| `…/src/resolve.rs` | create — orchestrator: `pub struct ResolveUseCase<O, C, A> { openbao: O, cache: C, audit: A }` |
| `…/src/list.rs` | create — list orchestrator |
| `…/src/policy_eval.rs` | create — Cedar policy hook |
| `microservices/cloud-secrets/catalog/oya-cloud-secrets-secret-reference-resolver-usecase.yaml` | create |

## Code Shape

```rust
pub async fn resolve<O, C, A>(
    deps: &Deps<O, C, A>,
    reference: &SecretReference,
    principal: &Principal,
) -> Result<ResolvedSecret, ResolveError>
where
    O: OpenBaoClient,
    C: SecretCache,
    A: AuditEmitter,
{
    deps.policy.evaluate(principal, "resolve_secret_reference", reference)?;

    if let Some(cached) = deps.cache.get(reference).await {
        deps.audit.emit_accessed(reference, principal, AccessOutcome::CacheHit).await?;
        return Ok(cached);
    }

    let resolved = deps.openbao.read(reference).await?;
    let ttl = clamp_ttl(resolved.suggested_ttl());
    deps.cache.put(reference.clone(), resolved.clone(), ttl).await;
    deps.audit.emit_accessed(reference, principal, AccessOutcome::CacheMiss).await?;
    Ok(resolved)
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-usecase
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-secrets-secret-reference-resolver-usecase
cargo run -p oya-dev-cli -- gate validate lean-a2 --crate oya-cloud-secrets-secret-reference-resolver-usecase
```

## Test Plan

- Mock ports; cover paths: cache hit, cache miss, policy deny, OpenBao error.
- Property: every resolve → exactly one audit emission.
- Property: every cache write uses clamped TTL.

## Halt Conditions

- Audit emission skippable — BLOCKER (security invariant).

## Next IP

`IP-006-resolver-adapter-openbao.md`

## Wave 15-IP-substance A-G

### A. Problem
Secret resolution touches cache, OpenBao, policy, audit, and revocation state. If orchestration is left to individual callers, some paths can skip audit emission, over-cache raw values, or retry through degraded policy state.

### B. Approach
Centralize the resolve flow in `oya-cloud-secrets-secret-reference-resolver-usecase`: parse, check cache, query OpenBao on miss, evaluate tenant policy, emit `SecretAccessed`, write bounded cache, and return a redaction-safe `ResolvedSecret`. Ports from the kernel are injected so the usecase remains testable.

### C. Deliverables
- `oya-cloud-secrets-secret-reference-resolver-usecase` crate and catalog entry.
- `ResolveSecret` orchestration over `OpenBaoClient`, `SecretCache`, `RevocationConsumer`, and `AuditChainBridgeClient`.
- Audit event mapping to `contracts/asyncapi/cloud-secrets-events.yaml`.
- SLO linkage to `slos/secret-resolve-latency.openslo.yaml` and `slos/audit-log-completeness.openslo.yaml`.
- Policy linkage to `policy/tenant-scope.cedar` and `policy/secret-isolation.md`.

### D. Ordered Implementation Steps
1. Define `ResolveSecretCommand` with tenant, principal, microservice, reference, and purpose.
2. Parse and normalize the reference via the domain crate.
3. Check the in-process cache with TTL and revocation epoch validation.
4. On miss, call `OpenBaoClient` through the kernel port and never log the returned value.
5. Evaluate tenant and microservice scope policy before returning.
6. Emit a signed audit event for every allow, deny, and backend failure.
7. Cache only allowed results with domain-clamped TTL and zeroize on drop.

### E. Acceptance
- `cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-usecase`.
- `cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-secrets-secret-reference-resolver-usecase`.
- `cargo run -p oya-dev-cli -- gate validate lean-a2 --crate oya-cloud-secrets-secret-reference-resolver-usecase`.
- Tests cover cache hit, cache miss, policy deny, OpenBao error, revocation invalidation, and audit failure.
- Every resolve attempt produces exactly one audit outcome.

### F. Evidence
Evidence anchors are `PRD.md` FR-02/FR-06/FR-08, `manifest.json`, `catalog/oya-cloud-secrets-secret-reference-resolver-usecase.yaml`, `contracts/asyncapi/cloud-secrets-events.yaml`, `runbooks/secret-leak-detected.md`, and `dashboards/secret-resolution-rate.json`.

### G. Counterpart Comparison
AWS, Azure, GCP, and Vault SDKs retrieve values, but the parity matrix calls out Oyatie's stronger SDK and audit contract: `Secret<T>`, no-log behavior, cache TTL ceilings, revocation push, and audit-chain sealing. This usecase is where those counterpart advantages become mandatory flow control.

Grep-recognized counterpart anchor: GitHub Actions Secrets is relevant when CI jobs exercise resolver flows with distributed credentials; the usecase must still audit, redact, and revoke through Oyatie controls. That anchor is secondary to the Vault/OpenBao and managed-secret runtime comparators.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/IP-005-resolver-usecase.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-005-resolver-usecase.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/cloud-secrets/manifest.json#paid_billing_components_emitted` is absent; this section is triggered by IP text and must be reconciled with the manifest billing model.
- Surface evidence: `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-005-resolver-usecase.md`.

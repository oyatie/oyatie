---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: ontology
ip_id: IP-010
impl_plan_id: IP-010-audit-chain-merkle-ed25519
title: ontology audit-chain BC (Merkle tree + Ed25519 sealing via OpenBao)
milestone: M01-foundation
phase: P01-typed-entity-substrate
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner_team: axis-ontology
co_owners: [axis-audit-chain, axis-security]
date: 2026-05-18
related_adrs: [ADR-0028, ADR-0117, ADR-0064]
depends_on: [IP-007]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-governance-audit-chain-emission
  - audit-chain-tamper-detect
  - oya-governance-promotion-readiness
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-audit-chain-{kernel,domain,usecase,adapter,worker}/
doc_status: published
---


# IP-010 — Ontology audit-chain BC (Merkle + Ed25519)

## Goal

Author the ontology audit-chain bounded context: per-tenant Merkle tree, per-period segment (60s rolling OR 10^4 events, whichever first), Ed25519 sealing via OpenBao Transit, and chain-of-chains mirror to the platform `audit-chain` µservice per ADR-0028. Per-pack (US-healthcare, EU) HSM-backed Ed25519 keys for residency compliance (ADR-0117).

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `microservices/ontology/src/crates/oya-ontology-audit-chain-kernel/Cargo.toml` + `src/lib.rs` | create | ~140 LoC; pure trait surface (Merkle types, signing port) |
| `microservices/ontology/src/crates/oya-ontology-audit-chain-domain/Cargo.toml` + `src/lib.rs` | create | ~220 LoC; Merkle builder (deterministic; sha256 leaves; sha256 internal); segment-rollover logic |
| `microservices/ontology/src/crates/oya-ontology-audit-chain-usecase/Cargo.toml` + `src/lib.rs` | create | ~180 LoC; orchestration: consume outbox → append → seal cadence |
| `microservices/ontology/src/crates/oya-ontology-audit-chain-adapter/Cargo.toml` + `src/lib.rs` | create | ~220 LoC; OpenBao Transit signing adapter + Postgres append-only adapter + Kafka outbox consumer |
| `microservices/ontology/src/crates/oya-ontology-audit-chain-worker/Cargo.toml` + `src/main.rs` | create | ~180 LoC; daemon entrypoint |
| `microservices/ontology/src/crates/oya-ontology-audit-chain-domain/tests/merkle_test.rs` | create | ~240 LoC; 6 deterministic + tamper-detect tests |
| `microservices/ontology/src/crates/oya-ontology-audit-chain-adapter/tests/transit_signing_test.rs` | create | ~140 LoC; 3 OpenBao-signing tests |
| `microservices/ontology/src/crates/oya-ontology-audit-chain-worker/tests/replay_test.rs` | create | ~160 LoC; replay-from-outbox reproduces same root |
| `microservices/ontology/iac/postgres-audit-chain.sql` | create | ~80 LoC; append-only table + trigger forbidding UPDATE/DELETE |
| `microservices/ontology/runbooks/audit-chain-tamper-response.md` | create | ~120 LoC operator playbook |
| `microservices/ontology/decisions/ADR-0028.md` | append §"Ontology audit-chain BC landed" | +6 LoC |

## Code shape

`audit-chain-domain/src/lib.rs` (excerpt — Merkle builder):

```rust
pub struct Segment {
    pub tenant_id: TenantId,
    pub period: PeriodWindow,
    pub events: SmallVec<[EventHash; 1024]>,
    pub root: MerkleRoot,
    pub signature: Option<Ed25519Signature>,
}

impl Segment {
    pub fn append(&mut self, event: EventHash) -> Result<(), AuditChainError> {
        if self.is_full() { return Err(SegmentFull); }
        self.events.push(event);
        self.root = recompute_root_incremental(&self.root, event);
        Ok(())
    }

    pub fn seal(&mut self, signer: &dyn Ed25519Signer) -> Result<Ed25519Signature, AuditChainError> {
        let sig = signer.sign(self.root.as_bytes())?;
        self.signature = Some(sig);
        Ok(sig)
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `merkle_root_is_deterministic` | merkle_test.rs | Same event sequence → same root across runs |
| `merkle_tampered_node_detected_during_verify` | merkle_test.rs | Mutate one event hash → verify fails |
| `segment_rolls_over_at_period_boundary` | merkle_test.rs | 60s elapsed → new segment opens |
| `segment_rolls_over_at_10000_events` | merkle_test.rs | 10001th event → new segment |
| `incremental_root_matches_full_recompute` | merkle_test.rs | Streaming append matches batch root |
| `concurrent_appends_no_interleave_corruption` | merkle_test.rs | Concurrent appends preserve order under lock |
| `openbao_transit_signs_with_per_tenant_key` | transit_signing_test.rs | Sign uses tenant-bound key (verified by key URN) |
| `pack_us_healthcare_uses_hsm_backed_key` | transit_signing_test.rs | US-healthcare pack signs with HSM key (verified via Transit metadata) |
| `transit_signing_fails_closed_on_outage` | transit_signing_test.rs | OpenBao down → signing returns error (no fallback to plaintext) |
| `replay_from_outbox_reproduces_same_root` | replay_test.rs | Replay Kafka outbox → identical Merkle root |
| `chain_of_chains_mirror_appended_at_platform` | replay_test.rs | After seal, platform audit-chain µservice has matching root |

Minimum 6 required; 11 specified.

## Evidence to emit

- `evidence/microservices/ontology/audit-chain-merkle-correctness-{date}.json`
- `evidence/microservices/ontology/audit-chain-tamper-detect-{date}.json`
- Audit-chain seal: `oya audit-chain seal --kind ontology-audit-chain --window 30d`
- Metrics: `oya_ontology_audit_chain_segment_seal_latency_ms_bucket`, `oya_ontology_audit_chain_events_appended_total`, `oya_ontology_audit_chain_tamper_detect_total`, `oya_ontology_audit_chain_chain_of_chains_lag_seconds`

## Rollback procedure

1. Revert ChangeSet for the 5 audit-chain BC crates + IaC + runbook.
2. Worker daemon stopped via `systemctl stop oya-ontology-audit-chain-worker` (or k8s equivalent).
3. Outbox topic drains naturally; no audit-chain seals until restored.
4. Existing sealed segments remain valid (no destructive operation).
5. Emit rollback evidence JSON; alert audit-chain owner that ontology emissions are paused.

## Blocking dependencies

- IP-007 — typed-entity substrate (event hashes must be addressable + ordered).
- Platform audit-chain µservice — chain-of-chains mirror endpoint must accept appends.
- OpenBao Transit deployed with per-tenant keys; HSM-backed keys for US-healthcare + EU packs.
- ADR-0028 — audit-chain Merkle/Ed25519 canonical.
- ADR-0117 — residency + per-pack key custody.

## Acceptance gates

```bash
cargo nextest run -p oya-ontology-audit-chain-worker --test merkle_verify
buck2 build //:quality-lane-registry-authority-check # lane=audit-chain-emission --microservice ontology
buck2 build //:quality-lane-registry-authority-check # lane=audit-chain-tamper-detect --microservice ontology
buck2 build //:quality-lane-registry-authority-check # lane=oya-governance-promotion-readiness --microservice ontology
```

## Halt conditions

- Tamper-detect test fails (verification did not catch mutation): STOP, security-critical.
- Replay-reproduces-root test fails: STOP, determinism violated.
- OpenBao Transit signing succeeds with non-bound key: STOP, security-critical.

## Exit criteria

1. All 11 tests green on CI.
2. `audit-chain-emission`, `audit-chain-tamper-detect`, `oya-governance-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. Postgres append-only trigger live in dev cluster.
5. Worker daemon deployed; emission rate matches expected event volume (verified via metric).
6. Runbook published.
7. ADR-0028 status updated for ontology audit-chain BC.

## Next IP

[`IP-011-ontology-rls-row-level-security.md`](IP-011-ontology-rls-row-level-security.md)

## References

- ADR-0028 — audit-chain Merkle/Ed25519 (canonical heritage from Bominal).
- ADR-0117 — residency + per-pack key custody.
- ADR-0064 — canonical base + localization overlay.
- OpenBao Transit API — `https://openbao.org/docs/secrets/transit/`.
- RFC 6962 Certificate Transparency (Merkle log design heritage).
- Trillian (Google) — `https://github.com/google/trillian`.


## A. Problem
`IP-010 — Ontology audit-chain BC (Merkle + Ed25519)` is not a generic implementation packet; it closes the `010 audit chain merkle ed25519` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
Typed registry evolution with monotonic data-class/pillar rules, versioned object/link/action/function schemas, and migration receipts for caller-side read libraries. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/ontology/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/openapi/ontology.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/proto/ontology.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/runbooks/type-registry-migration.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/catalog/oya-ontology-object-type-registry-domain.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/capabilities/type-register.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/ontology/PRD.md` and `microservices/ontology/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `ontology`.
2. Diff the declared contract in `microservices/ontology/contracts/openapi/ontology.yaml` and `microservices/ontology/contracts/proto/ontology.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/ontology/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/ontology/PRD.md`, `microservices/ontology/ARCHITECTURE.md`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/policy/tenant-scope.cedar`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and `microservices/ontology/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/ontology/PRD.md`
- `microservices/ontology/ARCHITECTURE.md`
- `microservices/ontology/contracts/openapi/ontology.yaml`
- `microservices/ontology/contracts/proto/ontology.proto`
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml`
- `microservices/ontology/policy/tenant-scope.cedar`
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`
- `microservices/ontology/runbooks/type-registry-migration.md`
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml`
- `microservices/ontology/competitor-parity-matrix.md`
- `microservices/ontology/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `010 audit chain merkle ed25519` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |

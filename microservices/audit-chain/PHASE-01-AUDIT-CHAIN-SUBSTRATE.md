---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-audit-chain-substrate
status: Active
entry_gate: |
  Bominal ADR-0028 + ADR-0003 inheritance ratified; ADR-0131 per-microservice flat layout accepted; existing crates oya-audit-chain-{domain,file-adapter,usecase} referenced (not physically moved this phase per task brief); /specs/audit-chain-merkle-ed25519.json published; observability µservice's SLO substrate available for self-SLO authoring.
exit_gate: |
  All 15 IPs merged; oya-audit-chain-emission + sealing + verification + query + retention-cascade crate families landed and Cargo workspace builds clean; HSM integration smoke-passes against OCI Cloud-HSM test partition; Merkle-root publication to Mimir + GitHub-pinned manifest live in pack-kr; HG-AUDIT registered in /specs/hyperscaler-gates.json; oya gate validate per-microservice-layout --microservice audit-chain exits 0; cargo nextest run --workspace exits 0; cross-pack-replication-forbidden lane green; end-to-end emission → seal → verify drill exits 0.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion (observability)
    reason: self-SLO authoring requires the SLO engine; rollback safety net for audit-chain itself depends on observability gate.
owner_team: axis-audit-chain
related_adrs: [ADR-0028, ADR-0003, ADR-0056, ADR-0105, ADR-0117, ADR-0123, ADR-0131]
related_specs: [/specs/audit-chain-merkle-ed25519.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-audit-chain-substrate: Land the cryptographic audit-chain end-to-end

## Purpose

This phase ships the full Bominal ADR-0028 + ADR-0003 design — Merkle-tree + Ed25519 audit chain with HSM-backed signing, Postgres index, S3-WORM raw-event storage, per-pack chain locality, per-tenant partitioning, retention cascade honouring per-pack legal minima, and tenant + auditor + CI verification surfaces.

This phase advances master-plan principles:
- Hyperscaler-grade non-repudiation (HSM + Ed25519 + eIDAS AdES + KR 전자문서법 compliance).
- Nothing scheduled-for-distinct-tracked-work (no "later we'll add signatures" stubs).
- Bominal-inheritance precedence (ADR-0028 + ADR-0003 inherited 1:1; oyatie overlays only where the master-plan or local jurisdiction diverges).
- Per-microservice flat layout (this phase is authored natively under ADR-0131).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected |
|---|---|---|
| `audit-chain` | `emission`, `sealing`, `verification`, `query`, `retention-cascade` | All under `microservices/audit-chain/` per ADR-0131; ~38 new Rust crates across the 5 BCs |

Plus repo-wide artifacts:
- `Cargo.toml` (workspace) — register the 38 new crates.
- `/specs/audit-chain-merkle-ed25519.json` — formalised spec (new).
- `/specs/hyperscaler-gates.json` — register HG-AUDIT gate per ADR-0123.
- `.github/branch-protection.yaml` — add `oya-audit-chain-self-verification` lane to required_status_checks on `dev`.

### Out-of-scope

- Migration of existing crates `oya-audit-chain-{domain,file-adapter,usecase}` into the new layout — they are **referenced** by this phase via thin re-export shims; the physical move is owned by the IP-M01-MIGR-audit-chain phase running in parallel under ADR-0131's migration spec.
- HSM hardware procurement — owned by `cloud-secrets` µservice's procurement plan; this phase assumes OCI Cloud-HSM is available per `iac/helm/hsm-operator/`.
- Cross-pack export tenant-controlled receiving-bucket attestation tooling — this phase ships the emission/refusal at the audit-chain side; receiving-bucket attestation is a separate `cloud-secrets` µservice deliverable.

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-storage-backend-iac.md`](IP-001-storage-backend-iac.md) | Helm/Kustomize for Postgres (HA) + audit-storage S3 bucket + HSM operator under `microservices/audit-chain/iac/` | pending | cloud-secrets + axis-audit-chain | — |
| [`IP-002-self-slo-manifest.md`](IP-002-self-slo-manifest.md) | OpenSLO manifests at `microservices/audit-chain/slos/` for emit_latency / seal_latency / verify_latency / hsm_avail SLIs | pending | axis-audit-chain | observability P01 |
| [`IP-003-emission-kernel.md`](IP-003-emission-kernel.md) | `oya-audit-chain-emission-kernel` crate: port traits + entity types + errors | pending | axis-audit-chain | — |
| [`IP-004-emission-domain.md`](IP-004-emission-domain.md) | `oya-audit-chain-emission-domain` crate: event-classification + envelope construction | pending | axis-audit-chain | IP-003 |
| [`IP-005-emission-usecase-and-adapter.md`](IP-005-emission-usecase-and-adapter.md) | `oya-audit-chain-emission-{usecase,api,adapter,rest,sdk,app}`; WAL writer + event-id minter + REST surface + SDK | pending | axis-audit-chain | IP-004, IP-001 |
| [`IP-006-sealing-kernel.md`](IP-006-sealing-kernel.md) | `oya-audit-chain-sealing-kernel` crate: port traits + entity types | pending | axis-audit-chain | IP-003 |
| [`IP-007-sealing-domain-merkle.md`](IP-007-sealing-domain-merkle.md) | `oya-audit-chain-sealing-domain` crate: RFC-6962-shaped Merkle math + root-chaining; property-tested | pending | axis-audit-chain | IP-006 |
| [`IP-008-sealing-adapter-hsm.md`](IP-008-sealing-adapter-hsm.md) | `oya-audit-chain-sealing-adapter-hsm` crate: PKCS#11 / KMIP to OCI Cloud-HSM partition; per-pack key handle | pending | cloud-secrets + axis-audit-chain | IP-007, IP-001 |
| [`IP-009-sealing-adapter-postgres-s3.md`](IP-009-sealing-adapter-postgres-s3.md) | `oya-audit-chain-sealing-{adapter-postgres,adapter-s3}` crates: SealRecord index + WORM-locked raw blob writer | pending | axis-audit-chain | IP-007, IP-001 |
| [`IP-010-sealing-worker-app.md`](IP-010-sealing-worker-app.md) | `oya-audit-chain-sealing-{worker,app,usecase,api,adapter}`: leader-elected sealing-cycle daemon | pending | axis-audit-chain | IP-008, IP-009 |
| [`IP-011-verification-stack.md`](IP-011-verification-stack.md) | `oya-audit-chain-verification-{kernel,domain,usecase,api,adapter,rest,sdk}`: full verifier + KeyResolver respecting key-rotation epochs | pending | axis-audit-chain | IP-010 |
| [`IP-012-query-stack.md`](IP-012-query-stack.md) | `oya-audit-chain-query-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk}`: tenant-scoped + Cedar-gated query + auditor export | pending | axis-audit-chain | IP-009, IP-011 |
| [`IP-013-retention-cascade.md`](IP-013-retention-cascade.md) | `oya-audit-chain-retention-cascade-{kernel,domain,usecase,api,adapter,worker}`: per-pack retention sweep + DSR cascade | pending | council-privacy + axis-audit-chain | IP-009, IP-012 |
| [`IP-014-cross-microservice-emission-adapter.md`](IP-014-cross-microservice-emission-adapter.md) | Standard `AuditEmitter` SDK consumed by every other oyatie µservice; reference integration in `tenancy` + `observability` | pending | axis-audit-chain + axis-tenancy + axis-observability | IP-005 |
| [`IP-015-self-observability-slo-wiring.md`](IP-015-self-observability-slo-wiring.md) | Wire audit-chain's own SLI emission into the observability substrate; HG-AUDIT gate registration; cross-pack-replication-forbidden LEAN lane | pending | axis-audit-chain + axis-observability | IP-002, IP-010, IP-011, IP-012, IP-013 |

Coverage check vs Bominal ADR-0028 + ADR-0003: every section (emission contract; Merkle construction; HSM signing; root publication; verification; retention; key rotation; cross-region locality) has a corresponding IP.

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps

oya gate validate lean-a1 --microservice audit-chain
oya gate validate lean-a2 --microservice audit-chain
oya gate validate port-location --microservice audit-chain
oya gate validate layer-correctness --microservice audit-chain
oya gate validate per-microservice-layout --microservice audit-chain
oya gate validate statelessness --microservice audit-chain
oya gate validate shardability --microservice audit-chain
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
oya gate validate cross-pack-replication-forbidden --microservice audit-chain
oya gate validate hsm-key-rotation-overlap --microservice audit-chain
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Emission happy path | `cargo nextest run -p oya-audit-chain-emission-usecase --test emission_happy_path` | event_id returned within ≤100ms; receipt durable |
| Seal latency | `cargo nextest run -p oya-audit-chain-sealing-worker --test seal_latency_drill` | seal completed ≤1s after period end |
| Verify tamper detection | property-based across 10k mutations | every mutation classified `verified=false` |
| HSM key rotation overlap | `cargo nextest run -p oya-audit-chain-sealing-worker --test key_rotation_overlap` | both pre- and post-rotation events verifiable across 24h window |
| DSR cascade redaction | scripted e2e | target events redacted within 30d; Merkle proof of redaction emitted |
| Cross-pack-replication refusal | `cargo nextest run -p oya-audit-chain-emission-rest --test cross_pack_refusal` | emission with foreign-pack header rejected with structured error |

## Clean Architecture Compliance

Layer assignments per the standard:

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-audit-chain-*-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-audit-chain-*-domain` | `domain` | own-BC `kernel` | adapter, rest, worker, sdk, app |
| `oya-audit-chain-*-usecase` | `usecase` | own-BC `domain`, `kernel` | adapter, rest, worker, sdk, app |
| `oya-audit-chain-*-api` | `api` | own-BC `kernel` (request/response types only) | impl crates |
| `oya-audit-chain-*-adapter` | `adapter` | own-BC `usecase`, `domain`, `kernel` | rest, worker, app directly |
| `oya-audit-chain-sealing-adapter-{postgres,s3,hsm}` | `adapter` | own-BC `usecase`, `domain`, `kernel` | other backend-qualified adapters |
| `oya-audit-chain-*-rest` | `rest` | own-BC `usecase`, `api`, `domain`, `kernel` | adapter directly (uses ports) |
| `oya-audit-chain-sealing-worker` | `worker` | own-BC `usecase`, `api`, `domain`, `kernel` | adapter directly (uses ports) |
| `oya-audit-chain-*-sdk` | `sdk` | own-BC `api`, `kernel` | usecase, adapter |
| `oya-audit-chain-*-app` | `app` | composition-root wiring only | (none — wiring only) |

Cross-BC: only the `sealing` worker consumes `emission`'s WAL writes via the WAL port; no other cross-BC import.

Cross-product check: this phase introduces NO direct imports between `audit-chain` and any other product µservice's crates. All cross-product data flow uses:
- **Inbound**: `AuditEmitter` port (consumed via `oya-audit-chain-emission-sdk`) — every other µservice imports the SDK.
- **Outbound**: events (`AuditEmitted`, `SealMinted`, `VerificationFailed`, `RetentionApplied`, `KeyRotated`).
- **Ontology**: writes to `AuditEvent`, `SealRecord`, `RedactionToken` types; reads from `Tenant`, `Microservice`.

## ChangeSet Contract per IP

Same shape as observability P01 — every IP emits per-changeset multispectrum evidence at `microservices/audit-chain/evidence/multispectrum/<change_id>-<unix_ts>.json`.

## Per-IP Test Coverage Threshold

Per PHASE-01 (observability) §"Per-IP Test Coverage Threshold" — same matrix applies. Notable adjustments:
- Merkle-tree math (IP-007) requires **property tests** (`proptest`-based) hitting ≥ 10k random trees, ≥ 10k random tamper mutations; coverage ≥ 95% line / 90% branch.
- HSM adapter (IP-008) requires `signing_correctness` end-to-end test against an OCI Cloud-HSM test partition; in CI, the test runs against a SoftHSM stub with the equivalent PKCS#11 interface; signature equivalence verified via offline-public-key reproduction.

## branch-protection.yaml diff preview

```yaml
branches:
  dev:
    required_status_checks:
      # ADDED by this phase:
      - oya-audit-chain-self-verification          # NEW; verifies the µservice's own audit chain
      - oya-governance-cross-pack-replication-forbidden  # NEW; refuses cross-pack export without SCC
      - oya-governance-hsm-key-rotation-overlap    # NEW; checks no period straddles a rotation without overlap
```

## Oya VCS Symbol Locks

Per ADR-0116; same primitives as observability P01.

## References

- Bominal ADR-0028 (Audit chain — Merkle + Ed25519); inherited 1:1.
- Bominal ADR-0003 (Audit emission contract); inherited 1:1.
- ADR-0056 BNF v4.1; ADR-0105 13-layer enum; ADR-0110 ChangeSet; ADR-0117 cloud-native infra; ADR-0123 hyperscaler-maturity claim gate; ADR-0131 per-microservice flat layout.
- `/specs/audit-chain-merkle-ed25519.json`.
- `microservices/audit-chain/PRD.md`.
- Memory: `feedback_bominal_inheritance_precedence.md`, `feedback_milestone_phase_hierarchy.md`, `feedback_oya_vcs_canonical_2026_05_16.md`, `feedback_clean_architecture_requirements.md`, `feedback_no_silent_regression.md`, `feedback_workflow_objectgraph_adapter_layer.md`.

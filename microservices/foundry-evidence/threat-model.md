---
doc_class: ThreatModel
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: ops-security + axis-foundry-evidence
related_adrs: [ADR-0028, ADR-0024, ADR-0056, ADR-0117, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/foundry-evidence/PRD.md
  - microservices/foundry-evidence/policy/tenant-scope.cedar
  - microservices/foundry-evidence/policy/regulator-export-scope.cedar
  - microservices/foundry-evidence/policy/evidence-pack-integrity.md
  - microservices/foundry-evidence/policy/data-residency.md
  - microservices/audit-chain/threat-model.md  (substrate model; this doc references it for delegated controls)
doc_status: published
---

# Threat model: foundry-evidence

## Scope

This threat model covers the foundry-evidence **frontend** µservice only: per-invocation pack assembly, the audit-chain bridge, per-invocation Postgres index, evidence-query API, and regulator-export bundle assembly. Cryptographic seal/verify/HSM/WORM controls are owned by the `audit-chain` substrate and are modelled in `microservices/audit-chain/threat-model.md`; this document references those controls where the foundry-evidence frontend depends on them.

## Trust boundaries

1. **Workload-emitter ↔ recorder** — foundry-runtime / foundry-guardrails / foundry-supervisor / foundry-eval worker pods → `oya-foundry-evidence-capability-invocation-recorder-rest`. SPIFFE-authenticated; Cedar-gated.
2. **Pack-builder ↔ audit-chain** — `oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge` → `audit-chain` substrate. SPIFFE-authenticated; substrate-side Cedar gate also applies.
3. **Tenant-operator ↔ evidence-query** — tenant portal / SDK → `oya-foundry-evidence-evidence-query-rest`. SPIFFE + tenant-token; Cedar-gated.
4. **Auditor / regulator ↔ regulator-export** — auditor portal / regulator engagement workflow → `oya-foundry-evidence-regulator-export-rest`. SPIFFE + DPA-recorded entitlement; Cedar-gated; two-person rule for export issuance.
5. **Operator ↔ archive cascade** — internal SRE / privacy operator → cold-archive lifecycle. Cedar-gated; audit-emitted.
6. **CI-lane ↔ self-verification** — CI runner → public-read endpoints (root-hash transparency, schema introspection). Read-only; per `policy/public-read.cedar`.

## STRIDE catalogue

### Spoofing (S)

| ID | Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|---|
| T-S-01 | Workload pod spoofs a different µservice's `source_microservice` field | Compromised pod injects `source_microservice=foundry-supervisor` while running as foundry-runtime | M | H | SPIFFE→source binding enforced server-side; `policy/tenant-scope.cedar` forbids `claimed_source != spiffe.microservice` | L |
| T-S-02 | Workload pod spoofs a different tenant's `tenant_id` | Compromised pod claims foreign tenant_id | M | C | SPIFFE→tenant binding; `bound_tenant_id == resource.tenant_id` Cedar invariant; counter `oya_foundry_evidence_tenant_spoofing_attempt_total` paged Sev-1 | L |
| T-S-03 | Regulator portal session-cookie replay | Stolen browser session | L | H | Short-lived SPIFFE-bound session tokens; second-factor; per-export 2-person rule; export-issuance audit-emitted | L |
| T-S-04 | Spoofed `invocation_id` collision | Adversary submits a fabricated `invocation_id` to associate pack data with a real but unrelated invocation | L | H | `invocation_id` validated against foundry-runtime's signed envelope; envelopes carry SPIFFE-signed `claimed_invocation_id` | L |
| T-S-05 | Spoofed eval verdict from compromised foundry-eval worker | Compromised foundry-eval pod publishes false verdict, pack-builder joins it | L | H | Subscription source verified via SPIFFE; eval verdict carries foundry-eval signature; aggregator rejects unsigned | L |

### Tampering (T)

| ID | Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|---|
| T-T-01 | Tamper of evidence-pack content between assembly and audit-chain seal | Pack-builder process compromise | L | C | Pack hash computed at assembly + carried into audit-chain `payload_sha`; substrate-side reseal would detect mismatch; `evidence-pack-integrity.md` §EPI-03 | L |
| T-T-02 | Tamper of Postgres evidence-index row | Operator runs ad-hoc UPDATE | L | C | Postgres role `foundry_evidence_writer` is INSERT-only; UPDATE/DELETE only via retention-cascade RPC (Cedar-gated; 2-person rule); LEAN lane `evidence-index-append-only` blocks SQL grants drift | L |
| T-T-03 | Tamper of evidence-blob payload after pack-builder hand-off | Adversary modifies blob between local stage and audit-chain WORM | L | C | Blob hash carried end-to-end; audit-chain WORM Object Lock + hash-verify-on-read; mismatch raises Sev-1 via `runbooks/pack-assembly-fail.md` | L |
| T-T-04 | Schema drift (silent field rename in pack-schema) | Code change ships without ADR | L | H | `oya-foundry-evidence-evidence-pack-builder-kernel` schema is contract-tested vs `/specs/foundry-evidence.json`; LEAN lane `no-silent-regression` blocks; ADR + version bump + sunset required | L |
| T-T-05 | Tamper of regulator-export bundle in transit to regulator | MITM | L | C | Bundle signed by pack-resident audit-chain key (substrate-delegated); regulator independently verifies; receiving channel TLS 1.3 + mTLS where regulator endpoint supports | L |
| T-T-06 | Backfill of historical packs alters chain | Adversary issues backfill-replay to fabricate evidence of past invocation | L | C | Backfill writes always go to current period (never historical); historical period roots immutable per audit-chain substrate; `runbooks/evidence-pack-rebuild.md` requires 2-person rule + on-chain reason | L |

### Repudiation (R)

| ID | Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|---|
| T-R-01 | Tenant disputes evidence of an agent invocation | Tenant claims "we never ran that capability" | M | H | Every pack carries audit-chain Merkle proof + Ed25519 signature; tenant can independently verify via audit-chain verifier; per-invocation `subject_hash` ties to tenant-side attestation if used | L |
| T-R-02 | Internal operator denies issuing a regulator-export | Operator denies running export workflow | L | M | 2-person rule + audit-of-audits emission; operator SPIFFE + approver SPIFFE on the export receipt; export-issuance audit-emitted | L |
| T-R-03 | foundry-runtime denies submitting an invocation | "We didn't actually call that" | L | H | record_invocation requires SPIFFE; envelope signature; audit-chain emit captures `source_microservice + principal_spiffe_id` per Bominal ADR-0003 | L |
| T-R-04 | Regulator disputes bundle authenticity | Regulator claims bundle was modified | L | H | Bundle Ed25519-signed by pack-resident key; regulator verifies via published transparency root; signature + Merkle proof carried in bundle envelope | L |

### Information Disclosure (I)

| ID | Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|---|
| T-I-01 | Cross-tenant read via API parameter manipulation | Tenant operator queries another tenant's pack | M | C | Cedar `tenant-scope.cedar` enforces `principal.tenant_id == resource.tenant_id`; tenant_id derived from SPIFFE, never from request body; LEAN lane `cross-tenant-leak-prevention` blocks | L |
| T-I-02 | Prompt / output text leaked via evidence-pack read | Tenant operator without sensitive-data entitlement reads pack containing PHI or PIPA Art. 23 data | M | C | Cedar permits gate on `payload_data_class ∈ principal.sensitive_data_entitlements`; SECRET-class never exposed to TenantOperator; data-class set at invocation time by source µservice | L |
| T-I-03 | Cross-pack read | Tenant on pack-eu reads pack-us-healthcare evidence | L | C | `policy/data-residency.md` + Cedar `bound_pack == resource.pack` invariant; cross-pack queries refused; LEAN lane `cross-pack-replication-forbidden` blocks at substrate boundary | L |
| T-I-04 | Public-read endpoint reveals tenant_id distribution | Adversary scrapes transparency log root + correlates timing | L | M | Public-read endpoints expose only `(pack, period_id, root_hash)`; tenant_id never present; aggregate roots per `tenant:oya-aggregate` Mimir series | L |
| T-I-05 | Side-channel via evidence-query response timing | Adversary infers existence of invocations via timing | L | L | Query layer adds constant-time padding for tenant-not-found vs tenant-empty (per `policy/tenant-scope.cedar` §TS-09) | L |
| T-I-06 | Regulator-export bundle leak via misdirected delivery | Bundle written to wrong S3 bucket | L | C | Receiving-bucket bound to receiving-tenant SCC + DPA-recorded export plan; 2-person rule validates target bucket; audit-emitted | L |
| T-I-07 | Evidence-blob URL exposure | Pre-signed URL leaked from operator screen | L | H | Pre-signed URLs short-lived (≤ 5 min) + IP-bound where supported + audit-emitted on issuance | L |
| T-I-08 | Inadvertent disclosure of foundry-supervisor autonomy-tier rationale | Pack exposes T3 escalation reasoning that is itself sensitive | L | M | Autonomy-tier rationale is a separate `data_class=INTERNAL_ONLY` payload field; gated by `policy/tenant-scope.cedar` §TS-04 (TenantOperator never reads INTERNAL_ONLY without entitlement) | L |

### Denial of Service (D)

| ID | Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|---|
| T-D-01 | Workload pod floods record_invocation | Misconfigured retry loop | H | H | Per-tenant + per-source-µservice rate limits; 429 + back-off; `oya_foundry_evidence_rate_limit_429_total` paged Sev-2 at sustained rate | L |
| T-D-02 | Pack-builder backlog explodes | foundry-runtime burst exceeds builder capacity | M | H | Horizontal sharding per tenant_partition; back-pressure to record_invocation as 429; `runbooks/audit-chain-backlog.md` Sev-2 | L |
| T-D-03 | audit-chain substrate down | substrate outage | L | H | Degraded-mode: record_invocation still durable in WAL + dead-letter; substrate-bridge worker retries; bridge availability SLO + `runbooks/audit-chain-backlog.md` | L |
| T-D-04 | Postgres exhaustion (per-pack) | Index growth overruns capacity model | L | H | Capacity model with 18-month headroom (`capacity-model.md`); per-pack shard split runbook; archival cascade clears warm tier | L |
| T-D-05 | Regulator-export workload exhausts builder | Large export request blocks pack-assembly path | L | H | regulator-export worker pool isolated from pack-builder pool; rate-limited; queue priority class | L |
| T-D-06 | Cedar policy compile DOS | Adversary submits pathological policy reload | L | M | Policy reload restricted to ops-security principals (Cedar `ci-scope.cedar` §CI-04); reload runs in side-process; rollback on compile error | L |

### Elevation of Privilege (E)

| ID | Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|---|
| T-E-01 | Tenant escalates to RegulatorExporter | Forged entitlement | L | C | Entitlement issued by `tenancy` µservice at onboarding; signed by tenancy substrate; foundry-evidence verifies signature; entitlement-revocation propagated via Workflow event | L |
| T-E-02 | Compromised ci-runner reads tenant pack | CI principal abuses inspection rights | L | H | `policy/ci-scope.cedar` permits CI principals only on test-tenant fixtures; production tenant scope refused | L |
| T-E-03 | Builder process escalates to retention-cascade authority | Pack-builder pod hijacked | L | H | Retention-cascade is a separate process (worker app, distinct SPIFFE); pack-builder SPIFFE cannot invoke cascade RPC per Cedar | L |
| T-E-04 | Regulator-exporter principal bypasses 2-person rule | Single-operator export | L | H | 2-person rule enforced in usecase layer + Cedar `regulator-export-scope.cedar` `approver_principal != requester_principal`; CI lane checks export-receipt schema | L |
| T-E-05 | Cross-microservice import bypass | foundry-evidence-adapter directly imports audit-chain-domain types | L | M | LEAN lane `cross-microservice-import-forbidden` blocks; only `oya-audit-chain-emission-sdk` re-exports are permitted | L |
| T-E-06 | Workflow event-bus replay attack | Adversary replays old `foundry.runtime.invocation.completed.v1` event | L | M | Events carry monotonic `(invocation_id, attempt_no)`; aggregator deduplicates; bus carries SPIFFE-signed publisher | L |

## Threat-to-control matrix

| Control | Owner | Verifies T-* |
|---|---|---|
| Cedar `policy/tenant-scope.cedar` | ops-security | T-S-01, T-S-02, T-I-01, T-I-02, T-I-05 |
| Cedar `policy/regulator-export-scope.cedar` | council-privacy | T-S-03, T-E-04, T-I-06 |
| Cedar `policy/ci-scope.cedar` | ops-security | T-E-02 |
| Cedar `policy/public-read.cedar` | ops-security | T-I-04 |
| Cedar `policy/auditor-scope.cedar` | council-privacy | T-R-04 |
| `policy/evidence-pack-integrity.md` | axis-foundry-evidence | T-T-01, T-T-02, T-T-03, T-T-04, T-T-05, T-T-06 |
| `policy/data-residency.md` | council-privacy | T-I-03 |
| `runbooks/audit-chain-backlog.md` | ops-sre-reliability | T-D-02, T-D-03 |
| `runbooks/pack-assembly-fail.md` | ops-sre-reliability | T-T-03, T-D-02 |
| `runbooks/regulator-export-reissue.md` | council-privacy | T-T-05, T-I-06 |
| `runbooks/evidence-pack-rebuild.md` | axis-foundry-evidence | T-T-06 |
| `runbooks/blob-storage-restore.md` | axis-foundry-evidence | T-T-03 |
| `runbooks/evidence-archive-migration.md` | axis-foundry-evidence + council-privacy | T-I-07, T-T-03 |
| `failure-modes.md` | axis-foundry-evidence | all D-* |
| LEAN lane `cross-tenant-leak-prevention` | axis-governance | T-I-01 |
| LEAN lane `cross-pack-replication-forbidden` | axis-governance | T-I-03 |
| LEAN lane `evidence-index-append-only` | axis-governance | T-T-02 |
| LEAN lane `no-silent-regression` | axis-governance | T-T-04 |
| LEAN lane `cross-microservice-import-forbidden` | axis-governance | T-E-05 |
| LEAN lane `hyperscaler-maturity-claims` | axis-governance | claim honesty per ADR-0133 |

## Inherited substrate threats

The following threats are owned by the `audit-chain` substrate model and are referenced here without restatement:

- Merkle-root cross-channel divergence — see `microservices/audit-chain/threat-model.md` T-T-09.
- HSM key compromise — see `microservices/audit-chain/threat-model.md` T-T-13.
- WORM Object Lock bypass — see `microservices/audit-chain/threat-model.md` T-T-15.
- Genesis-record mismatch — see `microservices/audit-chain/threat-model.md` T-T-17.

foundry-evidence's posture inherits these unchanged; the Sev-1 escalation path defers to the substrate runbooks (`merkle-seal-recovery.md`, `hsm-key-rotation.md`, `audit-chain-restart.md`).

## Non-fabrication posture (ADR-0133)

Per ADR-0133, claims about residual likelihood/impact are CI-asserted via the `hyperscaler-maturity-claims` gate. Where a residual is rated "L" with a control that is not yet fully deployed, the claim is annotated `aspirational=true` and the gate refuses commit. As of this document's date all listed mitigations are scheduled within phase P01; none are aspirational at exit-gate time.

## Review cadence

- Cadence: per release + on every change to `policy/`, `contracts/`, `iac/`, or `Cargo.toml` deps.
- Owner: ops-security + axis-foundry-evidence.
- Sign-off: ops-security director + council-privacy chair for releases that change residual ratings.

---
id: ADR-NET-0004
status: Accepted
date: 2026-05-17
microservice: network
deciders: council-architecture, ops-architecture, axis-network, axis-ats, ops-sre-reliability
owner: axis-network + axis-ats
supersedes: []
superseded_by: []
related:
  - ADR-0126
  - ADR-0131
  - ADR-0132
  - ADR-NET-0002
related_artifacts:
  - microservices/network/PRD.md (FR-34)
  - microservices/network/contracts/asyncapi/network-events.yaml
  - microservices/network/contracts/proto/network.proto
  - microservices/network/runbooks/jobs-handoff-ats-failure.md
  - microservices/network/backfill-replay.md (§"Jobs-Handoff Replay")
purpose: Establish the clean boundary between the `network` µservice (job-posting surface) and the Tier-G ATS µservice (applicant-tracking-system pipeline management). The handoff is contract-versioned event-based; dual-version window discipline applies; the `network` µservice never owns ATS pipeline state.
---

# ADR-NET-0004: Jobs-handoff to ATS µservice — clean µservice boundary; contract-versioned event handoff; dual-version window for contract evolution

## Status

Accepted — 2026-05-17.

## Context

LinkedIn and competitors mix two distinct concerns under "jobs":

1. **Job posting + discovery surface**: a tenant Page publishes job postings; candidates browse + apply via the social-network surface.
2. **ATS pipeline management**: applicant tracking, interview scheduling, offer-letter workflow, hiring-team coordination, candidate-experience surveys, hire/no-hire decision recording.

These are very different domains. ATS is a deep workflow product (Greenhouse, Lever, Ashby, Workday Recruiting all show the depth); coupling it into `network` would (a) bloat the µservice, (b) create suite-creep against ADR-0132, (c) make ATS feature-velocity tied to social-network release cadence.

Per ADR-0131 + ADR-0132, the boundary is enforced by µservice-decomposition. `network` is the **posting surface**; a separate Tier-G ATS µservice (under `microservices/ats/`, to be authored after M03) is the **pipeline management**. The handoff between them is the focus of this ADR.

Constraints:

1. **Idempotent + replay-safe**: event handoff must be idempotent on `event_id`; ATS µservice can be replayed (per `backfill-replay.md` §"Jobs-Handoff Replay").
2. **Contract-versioned**: the event schema is versioned; future evolution must not break older ATS deployments.
3. **Dual-version window**: when network upgrades from v1 to v2, both versions must be emit-compatible for ≥ 6mo before v1 is retired; symmetric for ATS upgrades.
4. **Per-pack residency**: events stay in-pack; ATS µservice in the same pack receives the handoff.
5. **Authoritative state**: ATS µservice owns the applicant-pipeline state; network µservice owns only the posting + the public-applicant-referral. There is no coupling of state between them; queries are one-directional (network → ATS via event).
6. **Sense-and-respond**: tenants want to see "Application received" feedback in their network UI; ATS must emit a confirmation event back; network does not block on it.
7. **Cedar contract-handoff**: ATS µservice principal must be authorised via `policy/tenant-scope.cedar` PERMIT 8 + must carry valid contract version.

## Decision

oyatie network's `jobs-handoff` BC implements:

1. **Bridge crate**: `oya-network-jobs-handoff-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-ats-bridge,worker,sdk}`.
2. **Event-based handoff** via Workflow event bus (Kafka / Redpanda):
   - `oya.network.jobposting.v1.published` — emitted on `POST /job-postings`.
   - `oya.network.jobapplication.v1.filed` — emitted on candidate apply-action.
   - `oya.ats.v1.application-accepted` — emitted by ATS µservice; network receives + updates `ats_handoff_status` on the JobPosting.
   - `oya.ats.v1.pipeline-state-changed` — fire-and-forget audit-only events.
3. **Contract version field** mandatory: every event carries `contract_version: "v1" | "v2"`.
4. **Dual-version window** (per ADR-0131): when network or ATS upgrades a major version of the contract, both versions remain emit + receive compatible for ≥ 6 months. After 6mo, the older version is retired with 30d notice to tenants. The LEAN lane `oya-gate validate jobs-handoff-contract` enforces dual-version emission during a transition window.
5. **Idempotency**: event_id is ULID; ATS µservice maintains a 30d deduplication index; replay (per `backfill-replay.md`) is safe.
6. **Per-pack residency**: events stay in-pack; cross-pack ATS handoff is forbidden per `policy/data-residency.md`.
7. **Authoritative state separation**:
   - network owns: JobPosting record, public-visibility status, candidate-referral metadata.
   - ATS owns: pipeline-stage (applied, screening, interview-1, ..., hired/rejected), interview schedule, offer letter, hiring-team interactions, candidate-experience survey responses, hire/no-hire decision recording (for EEOC + AB-331 + LL144 record-keeping 2y).
8. **Tenant-feedback latency**: network displays "Application received" UI on optimistic-emit (before ATS confirms); on ATS-bridge degraded (FM-19), the UI shows "Application queued; ATS pipeline reflecting" pending bridge recovery.
9. **Recruiter-stub link**: recruiter-stub (per ADR-NET-0002) is OFF by default; when activated, recruiter-search-result hits are NOT auto-handed-off to ATS — recruiter must take explicit action to invite a candidate to apply, which then emits `JobApplicationFiled` to ATS.
10. **Backfill-replay**: per `backfill-replay.md` §"Jobs-Handoff Replay", ATS may request resume from `from_event_id` after extended outage; network worker scans + re-emits; ATS deduplicates.

## Alternatives Considered

### A. Build ATS inside `network` (no handoff)

- Pros: tight UX integration; lower latency for state transitions; single µservice to operate.
- Cons: ATS is a deep workflow product; bloats `network`; violates ADR-0132 suite-and-bundle dissolution; ATS feature velocity tied to social-network release cadence; conflicts with µservice-decomposition.
- Rejected.

### B. ATS as a separate µservice but call synchronously (gRPC)

- Pros: simpler tenant feedback (synchronous "application received").
- Cons: couples `network` availability to ATS availability; FM-19-class outage cascades; harder to scale independently; tenants on ATS-tier-2 (slower ATS) would degrade `network` UX.
- Rejected.

### C. Event-based but no contract version field

- Pros: simpler events.
- Cons: contract evolution becomes painful; breaking-change deployments require lockstep; ATS deployments would block on network deployments and vice versa.
- Rejected.

### D. Event-based with hard cutover (no dual-version window)

- Pros: less operational complexity during steady state.
- Cons: contract upgrade requires lockstep across all tenant + ATS deployments; tenants cannot upgrade ATS independently; high-friction.
- Rejected: dual-version window is the correct discipline.

### E. Federated ATS (multiple ATS providers per tenant)

- Pros: tenant choice of ATS vendor.
- Cons: out-of-scope for P01; multiple ATS adapters complicates `network` codebase; defer to M04+ via additional adapter crates that map oyatie's event schema to vendor-specific (Greenhouse, Lever, Ashby, Workday) APIs.
- Partial accept: P01 ships single Tier-G ATS µservice; future ADR-NET may add adapter crates for Greenhouse / Lever / Ashby / Workday Recruiting bridge.

## Consequences

### Positive

- Clean µservice boundary preserved per ADR-0131 + ADR-0132.
- ATS µservice can scale + release independently of `network`.
- Contract-versioned dual-version-window discipline supports continuous evolution.
- Idempotent + replay-safe handoff supports BCDR per `multi-region.md` + `backfill-replay.md`.
- Per-pack residency aligned.
- Tenant has ATS-vendor flexibility (P01 = single Tier-G ATS; future ADR-NET may add multi-vendor adapter pattern).

### Negative

- Tenant-feedback latency is optimistic-emit + delayed-confirm; UX edge cases when ATS-bridge degraded.
- Contract evolution requires dual-version-window discipline; operational discipline.
- Recruiter-stub workflow has explicit recruiter-action gate (not auto-handoff); slightly more friction for recruiters.
- Cross-µservice retrospective required when handoff failures occur (FM-19); coordination overhead.

### Operational

- Cargo workspace: `oya-network-jobs-handoff-*` per BNF v4.1.
- gRPC + AsyncAPI contracts per `contracts/proto/network.proto` + `contracts/asyncapi/network-events.yaml`.
- LEAN lane: `oya-gate validate jobs-handoff-contract` validates dual-version-emit + contract-version field.
- Runbook: `jobs-handoff-ats-failure.md` (FM-19).
- Backfill-replay: `backfill-replay.md` §"Jobs-Handoff Replay".
- Helm: `jobsHandoffWorker.replicas: 3`.

### Regulatory

- **EEOC UGESP 29 CFR §1607**: ATS owns 2y candidate-record retention for record-keeping; network's emit-only path is not the record-of-truth for ATS data.
- **NYC LL144 §20-872**: ATS bias-audit obligations operate per ATS µservice; network's recruiter-stub bias audit (ADR-NET-0002) is upstream of the ATS handoff.
- **GDPR Art. 17 (erasure)**: DSR cascade per `policy/data-residency.md` must reach ATS via `JobApplicationErasureRequested` event; ATS confirms erasure back via `oya.ats.v1.candidate-erased` event.
- **DPDPA 2023 (India)**: data-fiduciary obligation extends to ATS µservice via contract-versioned event.

## References

- ADR-0126 (Connect dissolution, parallel).
- ADR-0131 (per-microservice flat layout).
- ADR-0132 (suite-and-bundle dissolution).
- ADR-NET-0001 (storage layer).
- ADR-NET-0002 (recruiter-stub bounds; upstream of jobs-handoff).
- `microservices/network/contracts/asyncapi/network-events.yaml`.
- `microservices/network/contracts/proto/network.proto`.
- `microservices/network/runbooks/jobs-handoff-ats-failure.md`.
- `microservices/network/backfill-replay.md`.
- Tier-G ATS µservice (future; under `microservices/ats/`).
- Greenhouse API docs `developers.greenhouse.io`; Lever API `hire.lever.co/developer/documentation`; Ashby API `developers.ashbyhq.com`; Workday Recruiting docs `community.workday.com`.
- EEOC UGESP 29 CFR §1607.4 record-keeping.
- NYC LL144 §20-872; CA AB-331; CO SB 24-205.

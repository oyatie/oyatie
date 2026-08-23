---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Workspace surface rollouts (Mail / Docs / Drive / Calendar / Meet).
planned_enforcement_ref:
  - governance-canary-required
  - governance-rollback-evidence
related_adrs: [ADR-0029, ADR-0040, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Playbook: Workspace Surface Rollout


## 1. Surface

Workspace productivity platform ([ADR-0029](../../decisions/ADR-0029-workspace-productivity-suite-architecture.md)) — Mail, Docs, Drive, Calendar, Meet, plus integrative surfaces.

## 2. Default rail per surface

| Surface | Rail | Notes |
|---|---|---|
| **Mail (SMTP / IMAP / inbox)** | Blue/green per spool | Mail spool is stateful; rolling-update kills threads |
| **Docs (collaborative editor)** | Canary | Per-document CRDT is replayable; canary-safe |
| **Drive (object storage)** | Canary; BG for replica change | Per-region replica topology = BG |
| **Calendar** | Canary | Event store is replayable |
| **Meet (real-time A/V)** | Canary per signalling node | Stateful at session level; per-node canary |
| **Search-within-Workspace** | Canary + dark-launch | Read-side dark-launch on ranking changes |

Per [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md), Workspace cadence: weekly per-surface; mail-spool / Drive-replica changes carry `requires_human_signoff: true`.

## 3. Mail-specific (spool stateful)

Mail rollout must:

1. Drain in-flight deliveries from blue spool before traffic-shift.
2. Replicate spool to green during preparation (read-only on green).
3. Atomic traffic-shift via mesh.
4. Soak ≥ 24 h; rollback re-shifts traffic to blue with replay of any green-only deliveries (idempotency required).

`governance-rollback-evidence` requires per-spool replay-proof.

## 4. CRDT compatibility (Docs)

Docs canary deployments must publish a CRDT schema version. Mixed-version sessions (some clients on blue, some on green) MUST converge correctly. Lane `governance-schema-migration` (existing; extended) verifies CRDT schema is forward + backward compatible across one minor version.

## 5. Per-tenant smoke

Workspace post-canary smoke covers, per tenant:
- Send + receive a probe email.
- Create + share a probe document with a probe peer.
- Schedule + cancel a probe calendar event.
- Drive upload + download.

Failure = per-tenant rollback. Surfaced in tenant trust portal.

## 6. Connect-no-ads cohort honour

Workspace surfaces respect the `connect-no-ads` cohort overlay ([`stable-cohort-spec.md`](stable-cohort-spec.md) §8). Any rollout that introduces an ad-supported feature MUST exclude this cohort. Planned advisory lane: `governance-cohort-honor`.

## 7. SLO targets (Workspace-specific)

| Service | SLO target | Window |
|---|---|---|
| Mail send | 99.95% | 30 d |
| Mail deliver-to-inbox | 99.9% | 30 d |
| Docs open | 99.95% | 30 d |
| Docs co-edit convergence | 99.99% | 30 d |
| Drive upload | 99.95% | 30 d |
| Calendar event-create | 99.95% | 30 d |
| Meet session-join | 99.9% | 30 d |

## 8. Per-region phasing

Workspace is region-heavy (latency-sensitive). Per-region phasing per [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md): primary cell in primary region → secondary cells → secondary region → other regions.

## 9. Hyperscaler equivalent

Google Workspace release-tracks (rapid / scheduled); Microsoft 365 deployment-rings (Targeted / Standard); Apple iCloud per-region rollout. We adopt the Google rapid-vs-scheduled mapping onto our cohort taxonomy.

## 10. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — Workspace weekly cadence per-surface; mail-spool changes carry `requires_human_signoff: true`; typescript-reviewer + rust-reviewer in dispatch.

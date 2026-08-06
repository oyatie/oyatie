---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Workspace surface rollouts (Mail / Docs / Drive / Calendar / Meet).
planned_enforcement_ref:
  - oya-governance-canary-required
  - oya-governance-rollback-evidence
related_adrs: [ADR-0029, ADR-0040]
doc_status: published
---

# Playbook: Workspace Surface Rollout

> **Status:** pending approval. **Owner:** `axis-workspace`. **Date:** 2026-05-12.

## 1. Surface

Workspace productivity platform ([ADR-0029](../../../docs/decisions/ADR-0029-workspace-productivity-suite-architecture.md)) — Mail, Docs, Drive, Calendar, Meet, plus integrative surfaces.

## 2. Default rail per surface

| Surface | Rail | Notes |
|---|---|---|
| **Mail (SMTP / IMAP / inbox)** | Blue/green per spool | Mail spool is stateful; rolling-update kills threads |
| **Docs (collaborative editor)** | Canary | Per-document CRDT is replayable; canary-safe |
| **Drive (object storage)** | Canary; BG for replica change | Per-region replica topology = BG |
| **Calendar** | Canary | Event store is replayable |
| **Meet (real-time A/V)** | Canary per signalling node | Stateful at session level; per-node canary |
| **Search-within-Workspace** | Canary + dark-launch | Read-side dark-launch on ranking changes |

## 3. Mail-specific (spool stateful)

Mail rollout must:

1. Drain in-flight deliveries from blue spool before traffic-shift.
2. Replicate spool to green during preparation (read-only on green).
3. Atomic traffic-shift via mesh.
4. Soak ≥ 24 h; rollback re-shifts traffic to blue with replay of any green-only deliveries (idempotency required).

`oya-governance-rollback-evidence` requires per-spool replay-proof.

## 4. CRDT compatibility (Docs)

Docs canary deployments must publish a CRDT schema version. Mixed-version sessions (some clients on blue, some on green) MUST converge correctly. Lane `oya-governance-schema-migration` (existing; extended) verifies CRDT schema is forward + backward compatible across one minor version.

## 5. Per-tenant smoke

Workspace post-canary smoke covers, per tenant:
- Send + receive a probe email.
- Create + share a probe document with a probe peer.
- Schedule + cancel a probe calendar event.
- Drive upload + download.

Failure = per-tenant rollback. Surfaced in tenant trust portal.

## 6. Connect-no-ads cohort honour

Workspace surfaces respect the `connect-no-ads` cohort overlay ([`stable-cohort-spec.md`](stable-cohort-spec.md) §8). Any rollout that introduces an ad-supported feature MUST exclude this cohort. Planned advisory lane: `oya-governance-cohort-honor`.

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

Workspace is region-heavy (latency-sensitive). Per-region phasing per [ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md): primary cell in primary region → secondary cells → secondary region → other regions.

## 9. Hyperscaler equivalent

Google Workspace release-tracks (rapid / scheduled); Microsoft 365 deployment-rings (Targeted / Standard); Apple iCloud per-region rollout. We adopt the Google rapid-vs-scheduled mapping onto our cohort taxonomy.

## 10. Lift target

`oyatie/docs/playbooks/playbook-workspace.md` on approval.

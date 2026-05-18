---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: docs
runbook_id: RB-share-acl-drift
status: Accepted
date: 2026-05-17
owner_team: ops-security + axis-docs
severity_applicable: [Sev-1, Sev-2]
related_failure_modes: [FM-06, FM-09]
doc_status: published
---

# Runbook — Share ACL drift (private doc accidentally public; or per-block ACL bypass)

## When this runbook fires

- LEAN scan `oya-check-public-collection-drift` detects a row with `visibility=public` lacking tenant + tenant-admin co-signed opt-in.
- `oya_docs_per_block_acl_violation_total > 0` (AC-04 breach).
- Tenant reports "my private doc is showing up in search / external access logs."
- `oya_docs_per_block_acl_check_p99 > 50ms` (latency-spike variant; Section C below).

## Severity

- Confirmed cross-tenant disclosure: Sev-1 (data breach).
- Confirmed within-tenant unauthorised access: Sev-2.
- Latency-only spike with no disclosure: Sev-3.

## Symptoms

- Anonymous redemption of share-link succeeds when it shouldn't.
- Per-block ACL violation count > 0 in audit-chain.
- Doc shows up in search by principal that should not see it.

## Probable causes

1. Tenant misconfiguration (operator set visibility to public without tenant-admin co-sign).
2. Race condition between share-grant write + cache invalidation.
3. Cedar policy reload introduced a regression in per-block ACL eval.
4. Adapter-bug: per-block ACL not consulted on some code path.
5. Postgres RLS misconfiguration on `blocks` table.

## Triage (within 15 min)

1. Acknowledge page; classify severity per scope.
2. Identify affected doc(s):
   ```bash
   oya docs acl-drift list --pack <pack>
   ```
3. Check audit-chain for any cross-principal accesses on the affected doc since drift began:
   ```bash
   oya docs audit-chain query --document <d> --action document_read --since "<iso>"
   ```
4. If cross-tenant disclosure confirmed: page council-privacy + ops-security.

## Section A — Confirmed public drift (private doc became public)

Cause: tenant misconfig OR race condition.

| Step | Action |
|---|---|
| 1 | Revert visibility to `private`: `oya docs acl set --document <d> --visibility private --reason "RB-share-acl-drift-<id>"`. |
| 2 | Emit `ShareGrantDriftDetected` audit event. |
| 3 | Identify which principals accessed the doc during drift window. |
| 4 | Notify affected tenant + (if disclosed beyond tenant) per pack regulator. |
| 5 | Forensic: was tenant-admin co-sign verified? Strengthen LEAN check if not. |

## Section B — Confirmed per-block ACL bypass

Cause: Cedar policy regression or adapter-bug.

| Step | Action |
|---|---|
| 1 | Halt affected code path: `cargo run -p oya-dev-cli -- vcs override-paths --microservice docs --halt-path <bc>`. |
| 2 | Audit affected blocks + principals via audit-chain replay. |
| 3 | Patch Cedar policy or adapter; add regression test. |
| 4 | Re-validate with `oya gate validate per-block-acl --microservice docs`. |
| 5 | Re-deploy with hotfix. |
| 6 | Tenant + regulator notification. |

## Section C — Per-block ACL latency spike (no disclosure; just slow)

Cause: Cedar policy reload across many tenants concurrently; per-block ACL projection cache cold.

| Step | Action |
|---|---|
| 1 | Check cache hit ratio: `oya_docs_per_block_acl_cache_hit_ratio`. |
| 2 | Enable single-flight per (doc_id, principal_id) at ACL evaluator. |
| 3 | Pre-warm cache: `oya docs acl-cache prewarm --pack <pack>`. |
| 4 | If recurring: optimise Cedar evaluation; add per-tenant policy cache. |
| 5 | Verify p99 returns to < 50ms within 15 min. |

## Section D — Cross-pack ACL drift (cross-tenant share grant misrouted)

Cause: tenant in pack-A shared with pack-B principal but ingress did not honour pack tag.

| Step | Action |
|---|---|
| 1 | Identify the misrouted grant. |
| 2 | Cedar policy ENFORCEs cross-pack only via embed-resolver snapshot per `policy/data-residency.md` Invariant DR-04; raw cross-pack share is FORBIDDEN. |
| 3 | If a cross-pack raw share existed: revoke immediately + audit-emit `CrossPackRawShareDetected`. |
| 4 | Forensic: investigate ingress routing; ensure pack-tag enforcement in OIDC claim. |

## Recovery validation

| Check | Target |
|---|---|
| Affected docs reverted to private (Section A) | yes |
| Per-block ACL bypass closed (Section B) | yes |
| ACL eval p99 < 50ms (Section C) | yes |
| Cross-pack raw share revoked (Section D) | yes |
| `oya-check-public-collection-drift` lane green | yes |
| Audit-chain seal continuity unbroken | yes |

## Post-incident review

- Was tenant-admin co-sign UX clear?
- Did Cedar policy reload propagate consistently?
- Update threat-model.md T-I-01 mitigation if needed.
- Update Cedar `tenant-scope.cedar` per-block ACL clauses if a new bypass vector discovered.

## Drills

- Annual red-team: per-block ACL bypass attempt against staging.
- Quarterly: simulated public-drift recovery drill.

## References

- `failure-modes.md` FM-06, FM-09.
- `threat-model.md` T-I-01, T-I-08.
- ADR-DOCS-0004 (per-block ACL).
- `policy/tenant-scope.cedar` (per-block ACL enforcement).
- `policy/public-read.cedar` (anonymous access rules).
- `policy/data-residency.md` Invariant DR-04 (cross-pack rules).

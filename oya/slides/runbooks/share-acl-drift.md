---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: slides
runbook_id: share-acl-drift
status: Accepted
severity: Sev-2 (Sev-1 if cross-tenant access enabled by drift)
date: 2026-05-17
owner_team: axis-workspace + ops-security + ops-sre-reliability
related_artifacts:
  - microservices/slides/decisions/ADR-SLIDES-0007-per-slide-acl-granularity.md
  - microservices/slides/failure-modes.md FM-19
  - microservices/slides/policy/tenant-scope.cedar
doc_status: published
---

# Runbook — Share / ACL drift

## When to use

- Per-decision ACL audit reveals deck-level vs cache mismatch.
- Tenant reports a viewer can access a slide they previously revoked.
- `oya_slides_acl_cache_miss_rate` > 5% or `oya_slides_acl_drift_detected_count` > 0.

## Symptom triage

| Symptom | Likely cause | Run |
|---|---|---|
| Single deck ACL drift | Cache eviction + lazy-refresh race | step 1 |
| Per-slide ACL drift | Per-slide ACL cache + deck-level grant mismatch | step 2 |
| Public-share-link still working after revoke | CRL / cache propagation delay | step 3 |
| Cross-tenant ACL drift | Sev-1 (security incident) | step 4 |

## Step 1 — Single-deck drift

```bash
DECK_ID=<deck_id>
TENANT_ID=<tenant_id>

# Invalidate ACL cache
oya vcs --service slides --action acl-cache-invalidate --deck-id $DECK_ID

# Reverify Postgres source-of-truth
oya vcs --service slides --action acl-verify --deck-id $DECK_ID

# Audit drift
oya vcs --service slides --action audit-tail --kind acl_drift --since 1h
```

## Step 2 — Per-slide drift

Per ADR-SLIDES-0007 — per-slide ACL refines deck-level. Drift between layers indicates a cache invalidation gap.

```bash
SLIDE_ID=<slide_id>

# Invalidate per-slide ACL cache
oya vcs --service slides --action per-slide-acl-cache-invalidate --slide-id $SLIDE_ID

# Reverify against Cedar
oya vcs --service slides --action cedar-preview --slide-id $SLIDE_ID --action slide:read --principal <test-principal>
```

## Step 3 — Public-share-link revoke not propagating

```bash
LINK_ID=<share_link_id>

# Force CRL propagation
oya vcs --service slides --action share-link-crl-propagate --link-id $LINK_ID

# Expire link immediately (forces re-issue or refusal)
oya vcs --service slides --action share-link-expire --link-id $LINK_ID --immediate

# Audit
oya vcs --service slides --action audit-tail --kind share_link_revoke --since 1h
```

CDN cache may carry signed share-link bundle; purge:

```bash
oya vcs --service slides --action cdn-purge-share-link --link-id $LINK_ID
```

## Step 4 — Cross-tenant ACL drift (Sev-1)

Per `threat-model.md` T-I-01.

```bash
# Freeze affected tenant write path immediately
oya vcs --service slides --action tenant-freeze --tenant-id <tenant_id> --reason "acl-drift-investigation"

# Capture forensic snapshot
oya vcs --service slides --action snapshot --tenant-id <tenant_id> --kind forensic

# Escalate: ops-security + DPO + legal
```

Determine if cross-tenant access was actually used (audit-chain). If yes: breach notification SLA starts (GDPR Art. 33: 72h; HIPAA: 60d).

## Re-enable

After fix:

```bash
oya vcs --service slides --action tenant-unfreeze --tenant-id <tenant_id>
oya vcs --service slides --action acl-health
```

## Verification

- ACL drift counter at 0 over 30min.
- Cedar preview matches Postgres source-of-truth.
- Audit-chain seal of drift + remediation emitted.

## References

- ADR-SLIDES-0007 (per-slide ACL).
- threat-model.md T-I-01.
- failure-modes.md FM-19.
- GDPR Art. 33.

---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: community
runbook_id: post-mass-deletion
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-security
related_artifacts:
  - microservices/community/failure-modes.md (FM-07, FM-14)
  - microservices/community/threat-model.md (T6.1, T6.2)
doc_status: published
---

# Runbook: post-mass-deletion

## When to use

- FM-07 (post mass-deletion by compromised tenant_admin)
- FM-14 (KB article impersonation via stolen credential)
- Audit-chain delete-rate anomaly alert

## Symptoms

- `delete_post` action rate > 100 / day for a tenant_admin.
- `purge_kb_article` action rate > 10 / day for a single admin.
- Author velocity anomaly for KB articles.

## Detection

- Grafana alert `community-tenant-admin-delete-velocity`.
- Grafana alert `community-kb-author-velocity-anomaly`.
- foundry-guardrails impersonation classifier signal.

## Triage

1. Identify tenant + admin / author.
2. Verify with tenant_admin (out-of-band: phone / verified channel).
3. Check session origin: unusual IP / geolocation?

## Mitigation

### Compromised tenant_admin (mass-deletion)

1. Engage tenancy: revoke all sessions for the admin.
2. Suspend admin role (Cedar fragment `admin_role == false`).
3. Stop further deletions immediately.
4. Restore deleted posts from Postgres WAL replay:
   `cargo run -p oya-community-post-store-cli -- restore --tenant <T> --since <ts> --actor <admin>`
5. Audit-chain emits `PostRestored` per restored post.
6. Engage tenant security + legal.

### Compromised author (KB impersonation)

1. Engage tenancy: revoke session.
2. Identify articles published in suspect window.
3. Revert articles to prior revision (KB revisions preserved).
4. Notify tenant_admin + readers (if attachment includes malicious content, scan flag).

### Legitimate event (tenant data spring-cleaning)

1. Tenant_admin confirms intent (two-eyes).
2. No reversal.
3. Audit-chain seal documents intent.

## Verification

- Deletion / publication rate returns to baseline.
- Restored posts visible.
- Audit-chain consistency check passes.

## Post-Incident

- Tenant_admin password rotation + 2FA verification.
- Two-eyes enforcement on destructive actions > 100/day audited.
- Per-tenant transparency note.

## Owner

axis-community (primary) + ops-security + tenancy.

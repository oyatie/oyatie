---
runbook: oauth-token-revocation-cascade
microservice: connector
owner_team: axis-integration + ops-security
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0263, ADR-0273, ADR-0296]
doc_status: published
---

# Runbook — OAuth Token Revocation Cascade

## A. Trigger conditions

- Vendor reports breach impacting OAuth infrastructure
- Internal detection: suspected refresh-token theft
- Tenant requests mass revocation
- Compliance event: PIPC / DPA / SCC violation requires revocation

## B. Pre-checks

1. Identify scope: vendor-wide, tenant-wide, or grant-specific?
2. Estimate impact: count of active grants affected via PG query.
3. Confirm authority: step-up auth from TAB or ops-security with WebAuthn.

## C. Procedure

1. **Quantify scope** (≤5min)
   ```sql
   SELECT count(*), tenant_id, status FROM connector.oauth_grants
   WHERE connector_name = '<vendor>' AND status = 'ACTIVE'
   GROUP BY tenant_id, status;
   ```

2. **Per-tenant mass revoke** (≤10min)
   - Via OAuth broker API: `POST /v1/oauth/grants/mass-revoke {connector, tenant_id}`.
   - Broker emits `OAuthGrantRevoked` × N audit events.
   - Broker calls vendor's revoke endpoint per grant (best-effort; failures still mark local state REVOKED).

3. **Downstream propagation** (≤60s)
   - workflow-engine subscribes to `OAuthGrantRevoked` events.
   - Active wirings using revoked grants auto-pause.
   - workflow-engine emits notification to wiring owner.

4. **Per ADR-0273 tenant notification** (≤24h SLA; ≤72h GDPR Art. 33)
   - DKIM-signed email to tenant admin via per-tenant DMARC.
   - In-app banner via workflow-studio.
   - For pack-eu: notify lead DPA within 72h.
   - For pack-kr: notify PIPC within 24h per PIPA Art. 34.

5. **Force re-OAuth** (when ready)
   - Broker schedules `OAuthGrantRotationForced` for each affected wiring.
   - TIE prompted via in-app banner to re-authorize.
   - Wirings remain paused until re-auth completes.

## D. Verification

```sql
SELECT count(*) FROM connector.oauth_grants
WHERE connector_name = '<vendor>' AND status = 'REVOKED'
  AND revoked_at > NOW() - INTERVAL '1 hour';
```

Expected: count matches expected scope. Plus:

```bash
# Audit chain emitted N events
curl http://connector-oauth-broker.connector:9090/metrics | grep connector_oauth_grant_revoke_total
```

## E. Rollback

Revocation is one-way; cannot rollback. If revoked in error:
- Apologize to tenants; communicate via in-app + email.
- Force re-OAuth flow for legitimate restoration.
- Audit chain captures the error; post-incident review mandatory.

## F. Post-incident

- Mandatory retro within 48h for any revocation cascade.
- Evidence pack: full chain of audit events sealed per ADR-0263.
- If vendor breach: track vendor SLA + future onboarding policy.

## G. References

- ADR-0263 audit-event emission
- ADR-0273 per-tenant DKIM/SPF/DMARC (notification path)
- ADR-0296 library-first credential sidecar
- PIPA Art. 34 (KR notification)
- GDPR Art. 33 (EU 72h notification)
- `microservices/connector/runbooks/admin-mfa-cascade.md` (referenced from ops-dashboard)

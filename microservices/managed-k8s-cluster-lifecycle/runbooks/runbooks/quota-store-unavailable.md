# Runbook: Quota Store Unavailable

## Symptom
`POST /tenants/{id}/quota/check` returns HTTP 500 with `quota store error`.

## Impact
Cluster-lifecycle cannot provision new clusters (fail-closed by design).

## Steps
1. Check the in-memory store (dev) or Postgres adapter (production) health.
2. Review application logs for `QuotaPortError::Persistence` entries.
3. Restart the quota service pod if the store is recovered.
4. Verify with `GET /healthz` → 200 OK.

## Prevention
Wire a persistent Postgres-backed store with retry (follow-on ADR-0376 wave).

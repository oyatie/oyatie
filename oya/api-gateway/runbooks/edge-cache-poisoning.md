# Runbook — Edge cache poisoning

**Authority:** ADR-0297 (in flight) + `microservices/api-gateway/threat-model.md` §C-T.
**Owner:** axis-network + ops-security.
**Trigger SEV:** SEV-1.
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- Per-tenant cache-key collision detection: `oya_api_gateway_cache_key_collision_total` > 0.
- Anomalous cache-hit response with wrong tenant-context.
- Customer report: "I'm seeing another tenant's data."
- WAF rule `cache-poisoning-attempt` triggered.

## B — Pre-checks

1. Confirm scope:
   - Single tenant affected vs multiple?
   - Single route vs all routes?
   - Per-cell vs global?
2. Identify attack vector:
   - Vary-header injection?
   - Cache-key-mutating header?
   - Path-parameter injection?
3. Snapshot affected cache entries:
   - `oyatie-cache-snapshot --route <route> --cell <cell>` → audit trail.

## C — Procedure

1. **Purge affected cache** immediately:
   - `oyatie-cache-purge --tenant <id> --route <route> --recursive`.
   - Audit: `oya.api_gateway.cache.purged.emergency`.
2. **Disable cache for affected route** temporarily:
   - Edit `iac/envoy-config.yaml`: set `cache_policy: none` for the route.
   - Push config; Envoy hot-reload.
3. **Add WAF rule** to reject cache-poisoning attempt:
   - `iac/edge-waf.yaml`: new rule for the specific attack vector.
4. **Notify affected tenants** (data-exposure audit):
   - Per `dpia.md §G GDPR breach notification`: 72h timer to lead DPA if confirmed PII exposure.
5. **Forensic analysis:**
   - Pull access logs from affected window; identify all tenants who received poisoned response.
   - Cross-reference Cedar audit events.
6. **Apply fix** (root-cause):
   - Vary-header normalisation if header-injection.
   - Cache-key cleansing if path-parameter-injection.
7. **Re-enable cache** once fix verified.
8. **Status page** if cross-tenant exposure confirmed.

## D — Verification

- Cache-key collision metric returns to 0.
- WAF rule firing on synthetic poisoning attempts.
- Customer-affected tenants notified per breach-notification SLA.
- Postmortem published.

## E — Rollback

If cache disable caused performance degradation:

1. Re-enable cache with conservative Vary headers.
2. Monitor `oya_api_gateway_cache_hit_ratio` recovery.

## F — Post-incident

1. **Breach notification** if PII exposure: 72h per GDPR Art. 33.
2. **Postmortem** within 5 business days.
3. **CI lane addition:** synthetic cache-poisoning test in `oya-governance-edge-cache-coverage`.
4. **WAF rule promotion** to default ruleset.

## G — References

- `iac/edge-waf.yaml`
- `iac/envoy-config.yaml`
- `microservices/api-gateway/dpia.md`
- ADR-0297 (in flight)
- OWASP API Security Top 10 (2023) — A03 Cache Poisoning

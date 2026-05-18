---
doc_class: Runbook
title: CDN Purge — global asset invalidation
microservice: application
severity: "Sev-1 (cache poisoning) / Sev-2 (operational purge) / Sev-3 (purge backlog)"
status: Accepted
owner_team: axis-application + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/application/failure-modes.md (FM-01, FM-11)
  - microservices/application/incident-response.md
  - microservices/application/policy/data-residency.md
doc_status: published
---

# Runbook: CDN Purge

## Trigger

ONE of:

1. **FM-01 CDN cache poisoning** — automated: `oya_application_bundle_sri_mismatch_total > 0` for ≥ 1 min raises Sev-1 page; worker auto-invokes global purge.
2. **FM-04 / FM-10 bundle revert** — operational: `oya gate validate hyperscaler-maturity-claims` lane red on new bundle; purge stale edge.
3. **FM-11 purge queue backlog** — `oya_application_cdn_purge_queue_depth > 100` for ≥ 3 min; Sev-3 successor-IP.
4. **Manual** — IC declares purge after security event.

## Severity

- Cache poisoning (automated): **Sev-1**.
- Operational revert: **Sev-2**.
- Queue backlog: **Sev-3**.

## Pre-checks

1. Confirm the offending asset URL: read `oya_application_bundle_sri_mismatch_total{url=...}` from Mimir.
2. Confirm the rollback bundle version: query `oya_application_bundle_version_active{environment="production"}` and the prior in `oya_application_bundle_version_prior`.
3. Verify operator credential (CDN admin API token; OpenBao path `secret/application/cdn/<pack>/admin-token`).
4. If Sev-1 + breach suspected: PrivacyLead joins immediately.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC; declare severity | ≤ 5 min |
| 2 | Confirm pre-checks | ≤ 2 min |
| 3 | Invoke purge: `cargo run -p oya-dev-cli -- application cdn purge --pack <pack> --pattern "/assets/*" --reason "<rfc>"`. The CLI: <br>a. fetches CDN admin token from OpenBao;<br>b. issues per-POP purge requests in parallel;<br>c. polls purge-job status until all POPs report complete (≤ 60 s p99);<br>d. emits `CdnPurgeRequested` event into audit-chain;<br>e. sets `oya_application_bundle_version_active` to the rollback SHA. | ≤ 60 s |
| 4 | Verify purge: probe random POPs (`curl -H 'Cache-Control: no-cache' https://<pop>.cdn.../assets/main.<hash>.wasm`) returns the rollback hash | ≤ 5 min |
| 5 | Verify SRI mismatch alarm cleared in Mimir | ≤ 5 min |
| 6 | CommsLead: status-page update | ≤ 30 min |
| 7 | If Sev-1: PrivacyLead initiates regulatory-notification per `incident-response.md` §"Regulatory-Notification Timelines" | per timeline |
| 8 | Postmortem within 5 BDs | – |

## Manual override (if CLI fails)

```bash
# Per-pack OCI CDN admin API direct call (last resort)
oci ce cdn purge \
  --distribution-id <pack-cdn-distribution-id> \
  --paths '["/assets/*"]' \
  --auth instance_principal
```

After manual purge: backfill `oya_application_bundle_purge_manual_total{reason="..."}` counter and file an Issue for "why did CLI fail".

## Verification

After completion:

- All POPs return rollback bundle hash on probe.
- `oya_application_bundle_sri_mismatch_total == 0` for ≥ 5 min.
- `oya_application_cdn_purge_queue_depth == 0`.
- `CdnPurgeRequested` event present in audit-chain seal log.
- Status page reflects "Resolved".

## Rollback (of the purge — if purge itself causes issues)

If purge purged a legitimate asset by mistake (over-broad pattern):
1. Re-promote the legitimate bundle: `cargo run -p oya-dev-cli -- application bundle promote --sha <prior-sha>`.
2. CDN re-caches on next request; warm via synthetic probe.
3. File postmortem on the pattern-match defect.

## Post-incident updates

- Postmortem published.
- Action items: typically "tighten purge pattern matcher", "add canary purge before global".
- Update this runbook if procedure missed a step.

## References

- `microservices/application/failure-modes.md` FM-01, FM-11.
- `microservices/application/incident-response.md` §"Sev-1 response".
- ADR-0123 (cross-product auth + hyperscaler maturity).

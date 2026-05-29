---
doc_class: Runbook
title: SMTP queue backup (outbound + inbound spool growth)
microservice: mail
severity: "Sev-2 (queue depth > 30s SLO) / Sev-1 (multi-tenant > 5min)"
status: Accepted
owner_team: axis-mail + ops-deliverability + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/mail/failure-modes.md (FM-OB-01 outbound spool growth, FM-IB-02 inbound queue stall)
  - microservices/mail/capacity-model.md (§"Outbound queue + spool sizing")
  - microservices/mail/contracts/asyncapi.yaml §"DeliveryAttempt"
  - /specs/microservices/mail.json
  - ADR-0133 cross-tenant mail-server pattern
doc_status: published
---

# Runbook: SMTP queue backup

## Trigger

ANY of:

1. `oya_mail_outbound_queue_depth_seconds{tenant_id=<t>}` > 60s sustained ≥ 5 min (page).
2. `oya_mail_inbound_smtp_queue_depth_seconds` > 30s sustained ≥ 5 min (page).
3. Spool volume (`/var/spool/postfix` PV) ≥ 75% (warn) or ≥ 90% (page).
4. Per-tenant reputation score crossed `< 70` AND queue depth growing — bounces are creating retry backlog.
5. Manual: operator observes Postfix `mailq` length > 50 000 entries OR Stalwart Queue API reports queue size > 50 000 per node.

Refers to FM-OB-01 (outbound spool growth) + FM-IB-02 (inbound queue stall) in `failure-modes.md`.

## Severity

| Condition | Severity |
|---|---|
| Single tenant, < 30 min, no SLO breach yet | Sev-3 |
| Single tenant, SLO at risk (queue > 60s × 5 min) | Sev-2 |
| Multi-tenant OR > 5 min sustained OR PV ≥ 90% | Sev-1 |
| Inbound impact (incoming mail rejected w/ 4xx) | Sev-1 (deliverability + sender-reputation risk) |

## Pre-checks

| # | Check | Command |
|---|---|---|
| 1 | Identify scope: per-tenant or global? | `topk(10, oya_mail_outbound_queue_depth_seconds) by (tenant_id)` in Mimir |
| 2 | Sample queued envelopes for symptom: hard-bounce, defer, or accept-pending? | `kubectl exec -n mail <postfix-pod> -- postqueue -j \| jq -s '. \| group_by(.delay_reason)'` (Postfix) or `kubectl exec <stalwart-pod> -- stalwart queue list --status scheduled-for-distinct-tracked-work` (Stalwart) |
| 3 | Recipient MX health (DNS + connectivity probe) for top 5 destination domains | `kubectl exec <pod> -- /opt/scripts/mx-probe.sh top5.txt` |
| 4 | DKIM signer accessible (cloud-secrets OpenBao reachable)? | `kubectl exec <pod> -- /opt/scripts/dkim-key-fetch.sh --dry-run --tenant=<t>` |
| 5 | Per-tenant reputation score + recent SPF/DMARC alignment failures | `oya_mail_deliverability_reputation_score{tenant_id=<t>}` + `oya_mail_outbound_dmarc_alignment_failure_total[10m]` |
| 6 | RBL / blocklist hit on tenant's outbound IP pool | `kubectl exec <pod> -- rbl-check.sh <ip>` against Spamhaus + SORBS + Barracuda |
| 7 | Cell capacity: is per-cell limit (1k msg/s baseline; 10k max) being hit? | `oya_mail_outbound_submission_rate_per_cell` vs `capacity-model.md` |
| 8 | Underlying Postgres + S3-blob health (DEK fetch latency, blob-write tail latency) | `oya_mail_mailbox_dek_fetch_p99_seconds`, `oya_mail_mimeblob_write_p99_seconds` |

## Recovery Path A — Single-tenant defer storm

Cause: One tenant's recipient population has high tempfail rate (recipient infrastructure issue; rate-limited by remote; greylisted).

| Step | Action | Time |
|---|---|---|
| 1 | Verify defer-class concentration: `kubectl exec <pod> -- postqueue -j \| jq '[.[] \| select(.delay_reason \| test("4\\."))] \| length'` | ≤ 5 min |
| 2 | If single recipient domain dominates: throttle per-recipient-domain submission rate via `oya-mail-outbound-smtp-app` config (`per_domain_max_concurrent: 4`); rolling-update | ≤ 10 min |
| 3 | Decision: bypass current retry schedule for the affected destination? — only if the destination explicitly requests; otherwise let Postfix/Stalwart exponential backoff run (5min → 1h → 4h → 24h envelope-by-envelope) | ≤ 5 min |
| 4 | Notify tenant: status-page note "outbound to `<domain>` scheduled-for-distinct-tracked-work due to recipient infrastructure; will auto-retry" | ≤ 30 min |
| 5 | Monitor: `oya_mail_outbound_queue_depth_seconds{tenant_id=<t>}` should trend down within 30 min once destination MX recovers | ≤ 1 h |

## Recovery Path B — Tenant reputation collapse (RBL hit)

Cause: Tenant's outbound IP appears on Spamhaus / SORBS; remote MXs returning `5.7.1` rejections; queue accumulating bounces.

| Step | Action | Time |
|---|---|---|
| 1 | Confirm RBL listing: `dig +short <ip>.zen.spamhaus.org` (positive listing returns `127.0.0.x`) | ≤ 5 min |
| 2 | Engage ops-deliverability; declare Sev-1 if tenant business-critical | immediate |
| 3 | Quarantine tenant outbound: pause submission for the affected mailbox via `oya-mail-outbound-smtp` admin API (`POST /tenant/{id}/pause`); audit-emit `MailDeliverabilityPaused` | ≤ 5 min |
| 4 | Identify abuse vector: spike in spam complaints? compromised account? misconfigured marketing tool? — query `oya_mail_outbound_complaint_total[24h]` + `oya_mail_outbound_message_volume_total[24h]` per-mailbox | ≤ 30 min |
| 5 | If compromised: reset mailbox credentials; rotate DKIM key (see `dkim-key-rotation.md`); page user | ≤ 15 min |
| 6 | Delist request: submit to Spamhaus per `https://check.spamhaus.org/` — provide remediation evidence | ≤ 1 h preparation |
| 7 | Move tenant to warm-pool IPs while listing clears (per ADR-0133 cross-tenant per-tenant SMTP IP allocation) | ≤ 30 min |
| 8 | Resume submission once delist confirmed (typically 24-72h) | per-listing |

## Recovery Path C — Spool volume exhaustion (PV full)

Cause: Postfix spool or Stalwart queue volume approaching 100%; queue cannot accept new submissions.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-sre-reliability for storage scale-out | immediate |
| 2 | Confirm volume usage: `kubectl exec <pod> -- df -h /var/spool` | ≤ 2 min |
| 3 | Identify large envelopes: `kubectl exec <pod> -- du -sh /var/spool/postfix/scheduled-for-distinct-tracked-work/* \| sort -rh \| head` | ≤ 5 min |
| 4 | Apply per-envelope size limit cap (RFC 5321 §4.5.3.1.7 max 64 MB; oyatie default 25 MB) if any envelopes ≥ limit are abusive | ≤ 5 min |
| 5 | Scale the underlying PV (StorageClass supports online expansion via CSI): patch PVC `spec.resources.requests.storage` to next tier (e.g., 100Gi → 250Gi) | ≤ 10 min |
| 6 | Verify pod observes new size: `kubectl exec <pod> -- df -h /var/spool` | ≤ 5 min |
| 7 | If single tenant is generating storm: see Path A or Path B | – |
| 8 | If multi-tenant systemic load: HPA scale `outbound-smtp` replicas (default min 4; can scale to 50 per HPA) | ≤ 10 min |

## Recovery Path D — DKIM signer outage (cloud-secrets unreachable)

Cause: OpenBao mTLS path broken; `oya-mail-outbound-smtp-adapter-smtp` cannot fetch tenant DKIM private key; submissions defer with `temporary signing failure`.

| Step | Action | Time |
|---|---|---|
| 1 | Confirm signer error rate: `oya_mail_outbound_dkim_sign_error_total[5m]` > 0 | ≤ 2 min |
| 2 | Engage `cloud-secrets` µservice oncall (their µservice owns OpenBao); see their runbook | immediate |
| 3 | If outage > 5 min: enable degraded mode — DKIM signing optional (NOT recommended for production tenants); only for emergency unblocking with explicit ops-deliverability + council-privacy approval (audit-emit) | per-incident |
| 4 | Cached DKIM keys (in-memory, 1h TTL) typically continue to serve; verify cache hit rate | ≤ 5 min |
| 5 | Once OpenBao recovers, queue drains automatically; verify `oya_mail_outbound_dkim_sign_error_total[5m]` returns to 0 | ≤ 30 min |

## Recovery Path E — Inbound queue stall (receiver can't persist)

Cause: `mailbox-store` write path saturated (Postgres CPU/IO; S3 blob write tail latency); inbound SMTP sessions accept DATA but post-DATA persistence blocks; sessions time out client-side.

| Step | Action | Time |
|---|---|---|
| 1 | Verify symptom: `oya_mail_inbound_smtp_post_data_persist_p99_seconds` > 2s | ≤ 2 min |
| 2 | Verify root: Postgres CPU + S3 write p99 | ≤ 5 min |
| 3 | If Postgres: see `tenancy` runbook for per-tenant Postgres scale | per-tenancy |
| 4 | If S3: see `cloud-secrets` runbook for object-storage backend health | per-cloud-secrets |
| 5 | Mitigation: shed inbound load with SMTP `421 4.7.0 try later` — RFC 5321 graceful temp-fail; senders retry per their own queue policy (typically 30-60 min) | ≤ 5 min |
| 6 | Verify shed: `oya_mail_inbound_smtp_421_temp_fail_total[5m]` rising while session count drops | ≤ 5 min |

## Verification

After completion:
- `oya_mail_outbound_queue_depth_seconds{tenant_id=<t>}` < 30s for ≥ 30 min.
- `oya_mail_inbound_smtp_queue_depth_seconds` < 30s for ≥ 30 min.
- Spool volume usage < 75%.
- Reputation score recovering (where applicable) — track 7-day trend.
- No active alerts on outbound delivery SLI (per `slos/outbound-delivery-latency.openslo.yaml`).
- `MailDeliverabilityReputationChanged` events emitted if score crossed threshold during incident.

## Post-incident updates

- If Path A/B (defer-storm or RBL hit): file Issue for tenant outbound configuration review; consider per-tenant warm-pool sizing per `capacity-model.md`.
- If Path C (spool exhaustion) recurs ≥ 2× in 6mo: tune autoscaler thresholds; revisit per-cell spool baseline.
- If Path D (signer outage): joint-postmortem with cloud-secrets µservice.
- Postmortem within 5 business days per `incident-response.md`.

## References

- RFC 5321 §4.5.4 (Retry Strategies); §4.5.3.1.7 (Size limits)
- Postfix queue management — `postfix.org/QSHAPE_README.html`, `postfix.org/postqueue.1.html`
- Stalwart Mail Server queue API — `stalw.art/docs/management/queue/`
- ADR-0133 (cross-tenant mail-server pattern, per-tenant IP pool)
- ADR-0117 (data residency, jurisdiction_code)
- Spamhaus SBL/PBL — `spamhaus.org/sbl/`
- `microservices/mail/failure-modes.md` FM-OB-01, FM-IB-02
- `microservices/mail/capacity-model.md`

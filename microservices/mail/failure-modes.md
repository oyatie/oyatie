---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-mail + ops-deliverability
deciders: ops-sre-reliability, axis-mail, ops-security, ops-deliverability, council-privacy
related_adrs: [ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/mail/threat-model.md
  - microservices/mail/dpia.md
  - microservices/mail/policy/dual-context-isolation.md
  - microservices/mail/incident-response.md
  - microservices/mail/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting mail
doc_status: published
---

# Failure-Mode Catalog (mail µservice)

## Purpose

Enumerate failure scenarios on-call must handle. Cross-referenced from `incident-response.md`.

## FM-01: SMTP relay outage (inbound-smtp pods down)

| Field | Value |
|---|---|
| Trigger | Cluster autoscaler eviction, hardware failure, kernel panic, OOM kill of inbound-smtp pods in an AZ |
| Detection | `mail_inbound_smtp_request_duration_seconds{quantile="0.99"} > 3s` for ≥ 5min OR `mail_inbound_smtp_active_pods` < 4 |
| Tenant impact | External senders queue + retry per RFC 5321; legitimate mail delayed up to recipient MX retry window (typically 4h) |
| Severity | Sev-2 (degraded; queue absorbs short outages); Sev-1 if > 30 min in production |
| Immediate mitigation | Verify HPA scaling; cordon affected AZ; allow cross-AZ rebalance |
| RTO | ≤ 15 min for pod recovery; ≤ 30 min for full AZ |
| Recovery runbook | `runbooks/smtp-relay-outage.md` |
| Postmortem owner | axis-mail + ops-sre-reliability |

## FM-02: Outbound SMTP delivery failure to major ISP

| Field | Value |
|---|---|
| Trigger | ISP (Gmail/Outlook/Yahoo) refuses tenant pool delivery; reputation drop; greylisting cascade |
| Detection | `mail_outbound_delivery_reject_total{recipient_isp="gmail"}` rate climbs > 5% |
| Tenant impact | Tenant outbound to that ISP fails; recipients don't receive |
| Severity | Sev-2 (single-ISP); Sev-1 if multi-ISP cascading |
| Immediate mitigation | Pause outbound on affected IP pool; investigate reputation; engage ISP postmaster |
| RTO | ≤ 1h pool quarantine; ISP reputation recovery may take days |
| Recovery runbook | `runbooks/deliverability-reputation-recovery.md` (referenced from runbooks set) |
| Postmortem owner | ops-deliverability + axis-mail |

## FM-03: Mailbox quota exhaustion (tenant or user-level)

| Field | Value |
|---|---|
| Trigger | Mailbox reaches 100% quota; inbound writes refused |
| Detection | `mail_mailbox_quota_exceeded_total > 0`; per-user dashboard at 80% warning |
| Tenant impact | New inbound mail to mailbox bounced (DSN-552); user notified |
| Severity | Sev-3 (single user); Sev-2 if tenant-pool exhaustion |
| Immediate mitigation | Notify user/tenant; offer quota increase per tenant_scope policy |
| RTO | ≤ 1h quota increase; user-side cleanup may take longer |
| Recovery runbook | `runbooks/mailbox-restore.md` (also covers quota mgmt) |
| Postmortem owner | axis-mail |

## FM-04: Search index corruption

| Field | Value |
|---|---|
| Trigger | Concurrent write conflict; hardware bit-rot; Tantivy version-upgrade bug |
| Detection | `mail_search_index_corruption_detected_total > 0`; search returning incomplete results |
| Tenant impact | Per-tenant search degraded or wrong; rebuild required |
| Severity | Sev-2 (limited to affected tenant) |
| Immediate mitigation | Quarantine affected index; switch reads to backup replica; trigger rebuild from mailbox-store |
| RTO | ≤ 1h rebuild for 10k-message mailbox |
| Recovery runbook | `runbooks/search-index-rebuild.md` |
| Postmortem owner | axis-mail |

## FM-05: Legal-hold drift (hold engaged but retention sweep ignored)

| Field | Value |
|---|---|
| Trigger | Bug in retention worker: legal-hold check returns false-negative; or hold state stale in worker cache |
| Detection | `mail_legal_hold_drift_total > 0` (set by audit-chain reconciliation worker); OR retention sweep deletes message that had hold engaged |
| Tenant impact | DPIA R-03: held material deleted; potential regulatory breach |
| Severity | Sev-1 (regulatory) |
| Immediate mitigation | Pause retention worker; reconcile hold state; restore deleted from soft-delete grace window |
| RTO | ≤ 30 min pause; ≤ 24h restore from grace |
| Recovery runbook | `runbooks/legal-hold-engage.md` (drift recovery section) |
| Postmortem owner | council-privacy + axis-mail + ops-legal |

## FM-06: KMS rotation gap (DEK rotation event causes mailbox unreadability window)

| Field | Value |
|---|---|
| Trigger | KMS rotation event; some blobs encrypted under old DEK; re-wrap worker lagged |
| Detection | `mail_kms_dek_unreachable_total > 0` |
| Tenant impact | Affected tenant's mailbox temporarily unreadable for affected blobs |
| Severity | Sev-2 |
| Immediate mitigation | Pause rotation; verify old + new DEK both accessible during re-wrap window; accelerate re-wrap worker |
| RTO | ≤ 1h re-wrap completion |
| Recovery runbook | `runbooks/kms-rotation-recovery.md` (cross-referenced) |
| Postmortem owner | cloud-secrets + axis-mail |

## FM-07: IMAP brute-force storm (credential stuffing wave)

| Field | Value |
|---|---|
| Trigger | Botnet wave targets tenant IMAP endpoint; thousands of credential-stuff attempts |
| Detection | `mail_imap_auth_fail_total` rate > 100/s sustained |
| Tenant impact | Per-IP and per-mailbox lockouts engaged; legitimate users with shared NAT may be blocked |
| Severity | Sev-2 |
| Immediate mitigation | Throttle source IPs; enable CAPTCHA cliff; engage WAF rate limit |
| RTO | ≤ 15min throttle; sustained event may require ISP coordination |
| Recovery runbook | `runbooks/imap-storm-throttle.md` |
| Postmortem owner | ops-security + axis-mail |

## FM-08: DKIM key rotation failure

| Field | Value |
|---|---|
| Trigger | OpenBao rotation event fails; DNS propagation lag; mismatched key in OpenBao vs DNS |
| Detection | `mail_dkim_sign_dns_mismatch_total > 0`; outbound DKIM verification failing at recipients |
| Tenant impact | Outbound mail goes to spam folder at recipient (DKIM fail); reputation declines |
| Severity | Sev-2 |
| Immediate mitigation | Pause rotation; revert to previous DKIM key (dual-publish); re-attempt rotation with DNS update first |
| RTO | ≤ 1h |
| Recovery runbook | `runbooks/dkim-rotation-recovery.md` (cross-referenced) |
| Postmortem owner | ops-deliverability + ops-security |

## FM-09: SMTP IP pool blocklisting (cascading reputation loss)

| Field | Value |
|---|---|
| Trigger | Spammer compromised tenant credentials sent abuse; major ISP blocklisted IP pool |
| Detection | `mail_outbound_blocklisted_total` rising; tenant deliverability dashboard reputation < 50 |
| Tenant impact | Outbound delivery to that ISP fails entirely until reputation recovered |
| Severity | Sev-1 |
| Immediate mitigation | Quarantine affected pool member; switch tenant to standby IPs; engage ISP postmaster relationship |
| RTO | ≤ 1h IP swap; ISP delisting may take days |
| Recovery runbook | `runbooks/deliverability-reputation-recovery.md` |
| Postmortem owner | ops-deliverability + axis-mail + ops-security |

## FM-10: Postgres mailbox-store outage

| Field | Value |
|---|---|
| Trigger | Postgres primary failure; replica promotion needed; or Citus shard balance event |
| Detection | `mail_postgres_request_duration_seconds{quantile="0.99"} > 2s` for ≥ 3min |
| Tenant impact | Mailbox metadata reads/writes blocked; SMTP receive queued; IMAP fetch fails |
| Severity | Sev-1 (broad impact) |
| Immediate mitigation | Auto-promote standby; verify connection; engage Citus rebalance if shard-related |
| RTO | ≤ 5 min failover (HA standby); ≤ 30 min for full Citus rebalance |
| Recovery runbook | `runbooks/mailbox-restore.md` (Postgres failover section) |
| Postmortem owner | axis-mail + cloud-secrets |

## FM-11: Cross-context (cross-pillar) routing leak detected

| Field | Value |
|---|---|
| Trigger | LEAN check or runtime audit detects a Professional API surface that read a Personal mailbox |
| Detection | `mail_cross_context_routing_refused_total > 0` (refused; OK) OR `mail_cross_context_routing_succeeded_total > 0` (BREACH) |
| Tenant impact | DPIA R-02; potential pillar breach; trust + legal disaster if confirmed |
| Severity | Sev-1 (privacy breach) |
| Immediate mitigation | Freeze affected API endpoint; engage ops-security + council-privacy; begin forensic trace |
| RTO | ≤ 5min freeze; investigation + breach-notification chain may take 72h+ |
| Recovery runbook | `runbooks/security-incident.md` (pillar-breach section) |
| Postmortem owner | ops-security + council-privacy + axis-mail |

## FM-12: eDiscovery export corruption (chain-of-custody seal mismatch)

| Field | Value |
|---|---|
| Trigger | Export bundle digest does not re-derive from source blocks; potential tampering or storage corruption |
| Detection | `mail_ediscovery_seal_verify_fail_total > 0` |
| Tenant impact | Export legally unusable; tenant must re-request; investigation triggered |
| Severity | Sev-1 (potential tampering) |
| Immediate mitigation | Quarantine bundle; pause exports for affected tenant; engage ops-legal + ops-security; re-export from source |
| RTO | ≤ 24h re-export |
| Recovery runbook | `runbooks/ediscovery-export.md` (seal-mismatch section) |
| Postmortem owner | ops-legal + ops-security + axis-mail |

## FM-13: Spam wave (inbound flood from botnet)

| Field | Value |
|---|---|
| Trigger | Botnet wave targets mail receivers; thousands of inbound msg/sec |
| Detection | `mail_inbound_smtp_rate` > tenant baseline 10× |
| Tenant impact | Receiver queue depth grows; legitimate mail delayed |
| Severity | Sev-2 |
| Immediate mitigation | Engage rate-limiting; greylisting threshold lowered; per-IP blocklist for top offenders |
| RTO | ≤ 15min throttle; wave duration typically hours |
| Recovery runbook | `runbooks/smtp-relay-outage.md` (spam wave section) |
| Postmortem owner | axis-mail + ops-deliverability |

## FM-14: Retention sweep stalled (worker crashloop)

| Field | Value |
|---|---|
| Trigger | Worker pod crashloop on parsing bug or KMS rate-limit |
| Detection | `mail_retention_sweep_last_run_age_seconds > 86400` (nightly sweep missed) |
| Tenant impact | Retention not enforced overnight; messages past expiry remain (no privacy breach but compliance drift) |
| Severity | Sev-3 (operational drift); Sev-2 if > 3 days |
| Immediate mitigation | Restart worker; identify crashloop cause; manual sweep for affected tenants |
| RTO | ≤ 30 min restart; full sweep ≤ 24h |
| Recovery runbook | `runbooks/retention-sweep-recovery.md` (cross-referenced) |
| Postmortem owner | axis-mail + council-privacy |

## FM-15: Mail-to-Workflow handoff failure

| Field | Value |
|---|---|
| Trigger | Workflow-engine unreachable; handoff event emits but consumer never picks up; user blocked at handoff UX |
| Detection | `mail_workflow_handoff_pending_seconds{quantile="0.99"} > 60s` |
| Tenant impact | User unable to convert mail to workflow item; UX degraded |
| Severity | Sev-3 |
| Immediate mitigation | Buffer handoff events; verify workflow-engine status; manual retry |
| RTO | ≤ 30 min |
| Recovery runbook | `runbooks/handoff-recovery.md` (cross-referenced) |
| Postmortem owner | axis-mail + axis-workflow |

## FM-16: TLS certificate expiry (SMTP edge / IMAP / REST)

| Field | Value |
|---|---|
| Trigger | cert-manager renewal failure; ACME challenge timeout |
| Detection | `mail_tls_cert_expiry_days` < 7 |
| Tenant impact | Inbound STARTTLS fails (downgrade or refused); IMAP TLS fails; REST refused |
| Severity | Sev-1 (broad availability impact) |
| Immediate mitigation | Manual cert renewal; engage cloud-secrets µservice; investigate ACME failure cause |
| RTO | ≤ 30 min manual renewal |
| Recovery runbook | `runbooks/tls-cert-renewal.md` (cross-referenced) |
| Postmortem owner | ops-security + cloud-secrets + axis-mail |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| SMTP relay outage (single AZ) | 15min | 0 (sender queues per RFC) |
| Outbound delivery rejection | 1h pool quarantine | N/A (queued) |
| Mailbox quota exhaustion | 1h | N/A |
| Search index corruption | 1h rebuild | 0 (rebuilt from mailbox-store) |
| Legal-hold drift | 30min pause + 24h restore | varies; soft-delete grace dependent |
| KMS rotation gap | 1h re-wrap | 0 |
| IMAP brute-force storm | 15min throttle | N/A |
| DKIM rotation failure | 1h revert | 0 |
| SMTP IP pool blocklisting | 1h IP swap | N/A (queued during) |
| Postgres outage | 5min failover | 5min (sync WAL replication) |
| Cross-context leak | 5min freeze | N/A (breach occurred) |
| eDiscovery seal mismatch | 24h re-export | N/A |
| Spam wave | 15min throttle | N/A |
| Retention sweep stalled | 30min + 24h sweep | varies |
| Workflow handoff failure | 30min | 0 (events buffered) |
| TLS cert expiry | 30min renewal | 0 |

## SLO on Failure-Detection Pipeline

Meta-SLO: mail substrate's failure detection has its own SLO.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Inbound mail receive p99 | ≤ 1s | 14.4× burn over 1h |
| Outbound delivery success rate (per major ISP) | ≥ 99% | 6× burn over 6h |
| Legal-hold engage latency | ≤ 2s | 14.4× burn over 1h |
| eDiscovery export completion within 24h SLA | ≥ 99% | informational |
| Cross-context routing refusal | ≥ 100% of attempts refused | 0 tolerance — Sev-1 page on any miss |
| Mailbox restore RTO | ≤ 15min | informational |

## References

- `microservices/mail/threat-model.md` (each FM maps to ≥ 1 STRIDE/LINDDUN threat ID).
- `microservices/mail/dpia.md` (FM-11, FM-12, FM-05 map to R-02, R-04, R-03).
- `microservices/mail/incident-response.md` §"Severity Definitions".
- `microservices/mail/runbooks/*` (recovery procedures).
- `microservices/mail/capacity-model.md`.
- Postfix operations docs — `postfix.org/`.
- Google SRE Workbook ch. 12 (Postmortem culture).

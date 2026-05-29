---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-audit-chain
deciders: ops-sre-reliability, axis-audit-chain, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0028, ADR-0131]
related_artifacts:
  - microservices/audit-chain/threat-model.md
  - microservices/audit-chain/dpia.md
  - microservices/audit-chain/policy/seal-integrity.md
  - microservices/audit-chain/policy/data-residency.md
  - microservices/audit-chain/incident-response.md
  - microservices/audit-chain/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Failure-Mode Catalog (audit-chain µservice)

## Purpose

Per-failure-mode runbook coverage for the audit-chain µservice. Cross-referenced from `incident-response.md` for severity classification. Note: because audit-chain is the evidence backbone, several FMs that would be Sev-2 in other µservices are **Sev-1 here** — chain-integrity failures cascade to every other µservice's compliance posture.

## Failure-Mode Index

| FM-ID | Title | Severity (default) |
|---|---|---|
| FM-01 | HSM partition outage | Sev-2 (eventual-seal model means emission survives) |
| FM-02 | HSM signing returns mismatched signature | **Sev-1** (integrity threat) |
| FM-03 | Cross-channel root divergence detected | **Sev-1** (tamper-suspect) |
| FM-04 | Genesis record mismatch on sealing-worker boot | **Sev-1** (fundamental tampering) |
| FM-05 | Postgres outage | Sev-2 |
| FM-06 | S3 WORM bucket unavailable | Sev-2 |
| FM-07 | sealing-worker crash-loop | Sev-2 (emission OK; sealing degraded) |
| FM-08 | emission-rest overload (DoS or burst) | Sev-2 |
| FM-09 | Cross-pack emission attempt detected | **Sev-1** (residency violation) |
| FM-10 | Verification-failed alert spikes (potential tamper) | **Sev-1** |
| FM-11 | DSR cascade backlog > 30d SLA | Sev-2 (compliance breach risk) |
| FM-12 | Retention-cascade applies unexpected mass-delete | **Sev-1** (insider-attack-suspect) |
| FM-13 | Key-rotation overlap expired without retire | Sev-2 |
| FM-14 | PII / PHI detected in audit payload (caller-redaction failure) | Sev-2 (DPIA R-01 realised) |
| FM-15 | Source µservice impersonation (T-S-01) detected | **Sev-1** |

## FM-01: HSM partition outage

| Field | Value |
|---|---|
| Trigger | OCI Cloud-HSM partition unreachable (vendor outage, network partition, planned maintenance not properly notified) |
| Detection | `oya_audit_chain_hsm_avail{partition=<id>} == 0` for ≥ 1 min; sealing-worker logs `pkcs11_session_failed` |
| Tenant impact | emit() continues; events accumulate in unsealed buffer; tenant dashboards show `sealed: false` for recent events |
| Severity | Sev-2 (degraded; no data loss within emission durability) — Sev-1 if persistent > 4h or affects regulator-bound SLAs |
| Immediate mitigation | Verify OCI status; engage Oracle support; if DR pair available (pack-eu / pack-us etc.), initiate DR failover per `multi-region.md` |
| RTO | ≤ 35 min for DR-failover-eligible packs; ≤ 4h for single-region packs (Oracle SLA) |
| Recovery runbook | `runbooks/hsm-key-rotation.md` §"Partition outage" + `runbooks/audit-chain-restart.md` |
| Postmortem owner | ops-sre-reliability + cloud-secrets + axis-audit-chain |

## FM-02: HSM signing returns mismatched signature

| Field | Value |
|---|---|
| Trigger | sealing-worker submits root to HSM; HSM returns signature; sealing-worker's local-verify against the same root + public key produces `verified: false` |
| Detection | `oya_audit_chain_hsm_signing_mismatch_total > 0`; emitted at first occurrence |
| Tenant impact | Sealing halts for the affected partition; events accumulate unsealed; tenant dashboards show prolonged `sealed: false` |
| Severity | **Sev-1** (potential HSM compromise; integrity of the chain is the question) |
| Immediate mitigation | Halt sealing for affected partition; engage ops-security + ExecSponsor + Oracle Cloud-HSM support; preserve forensic state; prepare key revocation contingency |
| RTO | depends on diagnosis; if confirmed compromise: emergency key revocation + new partition cutover within 4h; if Oracle-side fault: per Oracle SLA |
| Recovery runbook | `runbooks/signature-verification-failure.md` |
| Postmortem owner | ops-security + axis-audit-chain (Tier-1 review with ExecSponsor) |

## FM-03: Cross-channel root divergence detected

| Field | Value |
|---|---|
| Trigger | The continuous-validator detects that the published root in Mimir, S3 WORM, or the GitHub-pinned manifest differs from the other channels for the same `(pack, partition, period_id)` |
| Detection | `oya:audit_chain_root_cross_channel_match:rate < 1.0` |
| Tenant impact | Verification reads may return inconsistent results; tenant trust impact |
| Severity | **Sev-1** (tamper-suspect on at least one channel) |
| Immediate mitigation | Halt sealing; engage ops-security; investigate each channel: who/what wrote the divergent value; freeze the divergent channel for forensic preservation |
| RTO | Diagnosis ≤ 1h; restoration depends on root cause (config drift vs intentional tamper) |
| Recovery runbook | `runbooks/merkle-seal-recovery.md` §"Cross-channel divergence" |
| Postmortem owner | ops-security + axis-audit-chain |

## FM-04: Genesis record mismatch on sealing-worker boot

| Field | Value |
|---|---|
| Trigger | sealing-worker boots; reads genesis record from all three channels; mismatch detected |
| Detection | sealing-worker logs `genesis_record_mismatch`; emits the same as a metric |
| Tenant impact | sealing-worker refuses to start; no new seals; emission continues |
| Severity | **Sev-1** (fundamental chain integrity question) |
| Immediate mitigation | Engage ExecSponsor + ops-security; declare incident; investigate which channel diverges |
| RTO | depends on root cause; engagement window ≤ 24h |
| Recovery runbook | `runbooks/merkle-seal-recovery.md` §"Genesis mismatch" |
| Postmortem owner | ExecSponsor + ops-security + axis-audit-chain |

## FM-05: Postgres outage

| Field | Value |
|---|---|
| Trigger | Per-pack Postgres primary down |
| Detection | `pg_replication_lag_seconds` alarm; pod health-probe fail |
| Tenant impact | emission-rest falls back to local-WAL-on-disk; sealing-worker degraded (cannot read WAL); events accumulate locally on emission-rest pods |
| Severity | Sev-2 |
| Immediate mitigation | Promote replica to primary (Postgres HA); ensure local-WAL drainage to new primary; verify SealRecord catch-up |
| RTO | ≤ 5 min replica promotion; ≤ 30 min catch-up |
| Recovery runbook | `runbooks/audit-chain-restart.md` §"Postgres failover" |
| Postmortem owner | ops-sre-reliability |

## FM-06: S3 WORM bucket unavailable

| Field | Value |
|---|---|
| Trigger | OCI Object Storage outage in pack region |
| Detection | `s3_request_failures_total > 0` |
| Tenant impact | Raw blob writes degraded; sealing-worker queues; events durable in Postgres WAL until S3 returns |
| Severity | Sev-2 (degraded — emission durability ensures no data loss within Postgres window) |
| Immediate mitigation | Verify OCI status; DR-pair failover for eligible packs |
| RTO | ≤ 1h DR-pair failover; ≤ 4h single-region (per OCI SLA) |
| Recovery runbook | `runbooks/audit-chain-restart.md` §"S3 outage" |
| Postmortem owner | ops-sre-reliability + cloud-secrets |

## FM-07: sealing-worker crash-loop

| Field | Value |
|---|---|
| Trigger | sealing-worker pod crashloops; typical causes: Merkle-build bug on a malformed event; OpenBao token expiry; HSM PKCS#11 session failure |
| Detection | Pod restart count > 3 in 5min; `oya_audit_chain_sealing_alive == 0` |
| Tenant impact | Emission continues; sealing degraded — events sealed once worker recovers |
| Severity | Sev-2 (Sev-1 if persistent > 1h) |
| Immediate mitigation | Worker HA leader-election failover to standby; if both replicas crashloop, diagnose root cause from logs |
| RTO | ≤ 5 min failover; ≤ 30 min diagnosis + hotfix |
| Recovery runbook | `runbooks/audit-chain-restart.md` §"Worker recovery" |
| Postmortem owner | axis-audit-chain |

## FM-08: emission-rest overload (DoS or burst)

| Field | Value |
|---|---|
| Trigger | One workload µservice emits 10× expected rate; per-source rate limit kicks in |
| Detection | `oya_audit_chain_rate_limit_429_total > 0` rate climbs; emission p99 spikes |
| Tenant impact | Affected source µservice receives 429; non-affected sources unaffected |
| Severity | Sev-2 (per-source; not platform-wide unless multiple sources affected) |
| Immediate mitigation | HPA on emission-rest absorbs short bursts; per-source rate-limit shields cross-tenant impact; engage source µservice owner |
| RTO | ≤ 15 min |
| Recovery runbook | `runbooks/audit-chain-restart.md` §"Emission overload" |
| Postmortem owner | ops-sre-reliability + offending source µservice owner |

## FM-09: Cross-pack emission attempt detected

| Field | Value |
|---|---|
| Trigger | emission-rest receives an event whose `pack` field does not match the receiving cluster's pack; rejected per `policy/tenant-scope.cedar` |
| Detection | `oya_audit_chain_cross_pack_emission_rejected_total > 0` |
| Tenant impact | The originating µservice's emission failed; legitimate operations may be blocked |
| Severity | **Sev-1** (residency violation; potential breach of pack-pinning invariant) |
| Immediate mitigation | Engage offending workload µservice owner; verify OTel collector + workload config; engage ops-security if pattern suggests intentional bypass |
| RTO | ≤ 1h for routing correction; ≤ 72h for breach notification if EU/KR-pinned data was rerouted |
| Recovery runbook | `runbooks/audit-export.md` §"Cross-pack rejection investigation" |
| Postmortem owner | council-privacy + ops-security + offending workload owner |

## FM-10: Verification-failed alert spikes (potential tamper)

| Field | Value |
|---|---|
| Trigger | `oya_audit_chain_verification_failed_total` rate > threshold OR pattern of failures concentrated on a specific period range |
| Detection | Sustained increase over 5min |
| Tenant impact | Tenant + auditor verification queries returning false; trust signal degraded |
| Severity | **Sev-1** (potential tamper attempt) |
| Immediate mitigation | Engage ops-security; investigate the failure pattern; correlate with recent operations; check three-channel root match |
| RTO | Diagnosis ≤ 4h; preservation of forensic state immediate |
| Recovery runbook | `runbooks/signature-verification-failure.md` |
| Postmortem owner | ops-security + axis-audit-chain |

## FM-11: DSR cascade backlog > 30d SLA

| Field | Value |
|---|---|
| Trigger | retention-cascade backlog exceeds 30d for pending DSRs; risk of GDPR Art. 17 / KR PIPA Art. 36 SLA breach |
| Detection | `oya_audit_chain_dsr_backlog_seconds > 30d` for ≥ 1h |
| Tenant impact | Tenant DSR receipts delayed; tenant may face their own regulator inquiry |
| Severity | Sev-2 (compliance breach risk) — Sev-1 if multiple tenants affected |
| Immediate mitigation | Scale retention-cascade-worker replicas; verify tenant identifier mappings; engage council-privacy |
| RTO | ≤ 24h for backlog drain |
| Recovery runbook | `runbooks/retention-cascade.md` §"Backlog drain" |
| Postmortem owner | council-privacy + axis-audit-chain |

## FM-12: Retention-cascade applies unexpected mass-delete

| Field | Value |
|---|---|
| Trigger | retention-cascade-worker reports it has marked >> baseline events for redaction in a single run |
| Detection | `oya_audit_chain_retention_apply_rate{}` exceeds expected baseline + 3σ |
| Tenant impact | Tenant audit history visibly shrinks; tenant trust impact |
| Severity | **Sev-1** (insider-attack-suspect or policy misconfiguration with cascading effect) |
| Immediate mitigation | Halt retention-cascade-worker; engage ops-security; verify policy-matrix.yaml git history for unintended changes; ALL retention applications emit RetentionApplied events into the chain (so per Bominal ADR-0028 the deletion is itself recorded; recovery is by inspecting RetentionApplied logs and, if illegitimate, restoring from S3 WORM raw blobs which are *not* deleted by retention soft-delete) |
| RTO | Halt ≤ 5 min; investigation depends; restoration ≤ 24h |
| Recovery runbook | `runbooks/retention-cascade.md` §"Mass-delete anomaly" |
| Postmortem owner | ops-security + council-privacy + axis-audit-chain |

## FM-13: Key-rotation overlap expired without retire

| Field | Value |
|---|---|
| Trigger | Scheduled key rotation began; 24h overlap window elapsed; old key not retired due to operational oversight or pending operations against old key |
| Detection | `oya_audit_chain_key_overlap_expired_total > 0` cron-alarm |
| Tenant impact | None immediate; ops-debt accruing |
| Severity | Sev-2 |
| Immediate mitigation | Verify no pending sealing operations against old key; execute retirement via OpenBao 2-person JIT |
| RTO | ≤ 1h |
| Recovery runbook | `runbooks/hsm-key-rotation.md` §"Overdue retirement" |
| Postmortem owner | ops-security + axis-audit-chain |

## FM-14: PII / PHI detected in audit payload (caller-redaction failure)

| Field | Value |
|---|---|
| Trigger | Synthetic-PII detector lane scans audit-chain payloads in staging; flags unredacted PII pattern in events from a specific source µservice |
| Detection | `oya_audit_chain_pii_detector_finding_total > 0` |
| Tenant impact | DPIA R-01 risk realised; possible regulator notification if PII reached production |
| Severity | Sev-2 (DPIA-bounded) — Sev-1 if affects pack-us-healthcare (PHI in production) |
| Immediate mitigation | Engage source µservice owner; patch OTel + emission SDK redactor; trigger DSR cascade for affected records; engage council-privacy for notification chain |
| RTO | ≤ 1h source patch; ≤ 24h DSR cascade; ≤ 72h breach notification (if required) |
| Recovery runbook | `runbooks/audit-export.md` §"PII remediation" + `runbooks/retention-cascade.md` |
| Postmortem owner | council-privacy + offending source µservice owner |

## FM-15: Source µservice impersonation (T-S-01) detected

| Field | Value |
|---|---|
| Trigger | emission-rest rejects event due to SPIFFE-vs-tenant mismatch |
| Detection | `oya_audit_chain_tenant_spoofing_attempt_total > 0` |
| Tenant impact | Legitimate-attribution preserved (event rejected); attacker's emissions never enter chain |
| Severity | **Sev-1** (spoofing attempt suggests compromised credential or insider threat) |
| Immediate mitigation | Identify the spoofing SPIFFE identity; engage ops-security; investigate the source pod/credential; revoke if confirmed compromise |
| RTO | Investigation ≤ 4h; remediation depends |
| Recovery runbook | `runbooks/audit-export.md` §"Source impersonation investigation" |
| Postmortem owner | ops-security + offending source µservice owner |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| HSM partition outage | 35min (DR pair) / 4h (single) | 0 (Postgres + S3 durable) |
| HSM mismatched signature | depends on diagnosis | 0 (preserved at WAL) |
| Cross-channel divergence | 1h diagnosis | 0 |
| Genesis mismatch | 24h | 0 |
| Postgres outage | 5 min replica promote | 0 (HA) |
| S3 WORM outage | 1h DR-pair / 4h single | 5min (WAL → S3 catch-up window) |
| sealing-worker crashloop | 5 min HA failover | 0 |
| Emission overload | 15 min | N/A |
| Cross-pack rejection | 1h + 72h breach-notif | N/A |
| Verification-failed spike | 4h diagnosis | N/A |
| DSR cascade backlog | 24h drain | N/A |
| Retention mass-delete | 24h | 0 (S3 WORM preserved) |
| Key rotation overdue retire | 1h | 0 |
| PII detected | 24h DSR | N/A |
| Source impersonation | 4h | N/A |

## SLO on Failure-Detection Pipeline

Meta-SLO: audit-chain's tamper-detection MUST detect within window.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Cross-channel root match | ≥ 99.99% within 60s of seal | any failure pages immediately |
| Verification correctness against synthetic-tamper drill | 100% | any failure pages immediately |
| Genesis-mismatch detection at boot | 100% | binary |
| Verification-failed-spike detection latency p99 | ≤ 60s | 14.4× burn over 1h |
| DSR cascade SLA compliance | ≥ 99.5% within 30d | 6× burn over 6h |

## References

- `microservices/audit-chain/threat-model.md`.
- `microservices/audit-chain/dpia.md`.
- `microservices/audit-chain/incident-response.md`.
- `microservices/audit-chain/runbooks/*`.
- `microservices/audit-chain/capacity-model.md`.
- Bominal ADR-0028 + ADR-0003.
- Google SRE Workbook ch. 12.

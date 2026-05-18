---
doc_class: Runbook
title: Cross-tenant leak recovery (RLS bypass / Link cross-tenant / pillar misuse / pack misroute / audit tamper)
microservice: ontology
severity: "Sev-1 (security breach)"
status: Accepted
owner_team: ops-security + axis-ontology + council-privacy
date: 2026-05-17
related_artifacts:
  - microservices/ontology/failure-modes.md (FM-07, FM-08, FM-13, FM-14, FM-16)
  - microservices/ontology/threat-model.md (T-I-01, T-I-02, T-I-03, T-T-05, T-I-07)
  - microservices/ontology/incident-response.md
doc_status: published
---

# Runbook: Cross-tenant leak recovery

## Trigger

Any of:
- LEAN runtime probe detects cross-tenant Function read (FM-07).
- Tier-escape detected by property-tier-enforcement lane (FM-08).
- Audit chain Merkle root mismatch (FM-13).
- Cross-pillar grant misuse alert (FM-14).
- Pack-misroute detector emits non-zero (FM-16).

## Severity

**Sev-1 always.** Cross-tenant boundary breach + cross-border misroute + audit tampering all engage regulatory notification chains.

## Immediate response (first 10 minutes)

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-<id>` Slack; declare Sev-1; assign IC (ops-security); engage PrivacyLead + ExecSponsor | ≤ 5 min |
| 2 | Two-channel corroboration: confirm both metric + OnCall page received | ≤ 1 min |
| 3 | Identify scope: which tenants involved? which Object Types? which time window? | ≤ 5 min |
| 4 | Freeze affected endpoint: gateway middleware rejects all reads/writes for the affected Function / Action / Link | ≤ 2 min |
| 5 | Revoke implicated API keys: `openbao revoke <key-id>` for any keys associated with the breach | ≤ 5 min |
| 6 | Begin forensic capture: snapshot Postgres + Mimir audit-chain + Cedar evaluation log over the breach window | ≤ 30 min |

## Cross-tenant Function leak (FM-07)

Possible vectors:
- RLS policy `WITH CHECK` clause missing or `tenant_id` not bound to session var.
- Function evaluator bypassed RLS via `SECURITY DEFINER` PL/pgSQL with insufficient guards.
- Postgres superuser session active during the breach window.

| Step | Action |
|---|---|
| 1 | Verify RLS state: `SELECT relname, relrowsecurity, relforcerowsecurity FROM pg_class WHERE relname IN (...)` |
| 2 | Verify session-var binding code path: `git grep "app.tenant_id"` in adapter code; check for any code path that sets it from non-JWT source. |
| 3 | Audit `pg_stat_activity` for the breach window: any superuser session active? |
| 4 | Quarantine affected Function: ArgoCD apply a temporary Cedar `forbid` clause for the Function Type. |
| 5 | Patch the bug: PR + 2-person sign-off + emergency merge via OpenBao JIT bypass of branch-protection. |
| 6 | Audit-chain emit `CrossTenantLeakRecovered{scope, fix_sha, executed_at}`. |

## Cross-tenant Link leak (FM-07 variant)

| Step | Action |
|---|---|
| 1 | Identify offending Link rows: `SELECT id FROM <link_table> WHERE src_tenant_id != dst_tenant_id AND id NOT IN (SELECT link_id FROM cross_tenant_link_grants WHERE expires_at > now())` |
| 2 | Tombstone affected Link rows. |
| 3 | Engage tenants whose Object Types were referenced. |

## Tier escape (FM-08)

| Step | Action |
|---|---|
| 1 | Identify the Function projection that leaked: log + audit chain. |
| 2 | Patch the projection: add explicit tier-filter; PR + 2-person + emergency merge. |
| 3 | Purge cached Function results in Valkey: `oya-ontology-sdk valkey-flush --keyspace ontology:function-cache` |
| 4 | Purge ClickHouse history-mirror rows that include the leaked tier: `ALTER TABLE ... DROP PARTITION <range>` for affected window. |
| 5 | If properties were exfiltrated: engage DSR (request-of-impact); offer subject-of-record notifications. |

## Audit-chain tampering (FM-13)

| Step | Action |
|---|---|
| 1 | Verify Merkle root: `oya-ontology-sdk audit-chain-verify --tenant <id> --period <window>` |
| 2 | If verification fails: quarantine affected period in audit chain; trust-state set to `unverifiable`. |
| 3 | Engage ops-security: was Ed25519 signing key compromised? `openbao audit list --key <id>` |
| 4 | If key compromised: rotate immediately via OpenBao Transit; old key shredded after grace. |
| 5 | Re-seal from raw events if outbox still has the originals; new Merkle root issued with timestamped re-seal note. |
| 6 | Tenant + regulator notification: provenance claim for the affected window flagged `unverifiable` until trust re-established. |

## Cross-pillar grant misuse (FM-14)

| Step | Action |
|---|---|
| 1 | Identify the grant: `SELECT id, principal, allowed_pillars, expires_at, signed_by FROM cross_pillar_grants WHERE id = <id>` |
| 2 | Revoke grant: `UPDATE cross_pillar_grants SET revoked_at = now() WHERE id = <id>` |
| 3 | Verify Cedar evaluator picked up the revoke (hot-reload via cedar-fragment-coverage worker). |
| 4 | Audit reads that exercised the grant during the misuse window; engage council-privacy. |
| 5 | If forged grant: tighten 2-person rule enforcement; ADR successor-IP. |

## Pack misroute (FM-16)

| Step | Action |
|---|---|
| 1 | Identify misrouted rows: `SELECT id, tenant_id, jurisdiction, pack FROM <table> WHERE tenant_pack != pack` |
| 2 | Quarantine misrouted rows: move to a `_misrouted` table in the correct pack; mark for tenant review. |
| 3 | Correct SDK config in the workload µservice: update pack endpoint pinning. |
| 4 | Audit-chain emit `PackMisrouteRecovered{tenant, src_pack, dst_pack, row_count, executed_at}` |
| 5 | Engage council-privacy: cross-border-transfer violation; GDPR Art. 33 72-hour clock; KR PIPA Art. 34. |

## Regulatory notification (any Sev-1 breach affecting personal data)

Per `incident-response.md` §"Regulatory Notifications":

| Jurisdiction | Authority | Timeline | Trigger |
|---|---|---|---|
| EU | Lead DPA | 72h | GDPR Art. 33 |
| KR | PIPC | 72h | KR PIPA Art. 34 |
| US Healthcare | HHS OCR | 60 days | HIPAA §164.404 / .408 |
| JP | PPC | reasonable (~72h) | APPI Art. 26-2 |
| BR | ANPD | 2 business days | LGPD Art. 48 |
| IN | DPB | 72h | DPDPA 2023 §13 |
| Pack-EU NIS2 (when applicable) | National CSIRT | 24h initial + 72h detailed + 1mo final | NIS2 |
| Pack-KR-FSS | FSS | 24h | KR-FSS guidance |

CommsLead drafts; PrivacyLead reviews; ExecSponsor approves; transmission via official channels.

## Verification

After incident closure:
- `oya gate validate ontology-tenancy-isolation` — exit 0 (LEAN runtime probe passes).
- `oya gate validate cedar-coverage --microservice ontology` — exit 0.
- `oya gate validate audit-chain-emission --microservice ontology` — exit 0.
- All affected tenants notified per their DPA.
- Regulatory notifications transmitted within respective timelines.
- Postmortem published at `evidence/postmortems/<year>/<incident-id>.md`.
- ADR successor-IP filed for systemic remediation.

## Post-incident updates

- Postmortem within 5 business days (Sev-1 cadence).
- Action items tracked: tighten LEAN probes; expand Cedar coverage; tighten 2-person rule; audit-chain validation cadence.
- Tabletop exercise within 90 days simulating the same incident class.

## References

- `microservices/ontology/failure-modes.md` FM-07, FM-08, FM-13, FM-14, FM-16.
- `microservices/ontology/threat-model.md` T-I-01, T-I-02, T-I-03, T-T-05, T-I-07.
- `microservices/ontology/incident-response.md` §"Severity 1 response" + §"Regulatory Notifications".
- `microservices/ontology/policy/{tenant-scope, pillar, ci-scope}.cedar`.
- ADR-0028 (audit-chain).
- ADR-0140 (Cedar policy enforcement).

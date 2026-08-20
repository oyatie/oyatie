---
doc_class: Runbook
title: Sovereign-tenant cross-region inference leak — P0 data-residency invariant violation
microservice: translate
severity: "Sev-1 (P0) — HARD; data-residency invariant violation"
status: Accepted
owner_team: axis-translate + council-privacy + ops-security + legal-counsel
date: 2026-05-18
related_artifacts:
  - microservices/translate/failure-modes.md (FM-50)
  - microservices/translate/decisions/ADR-TRANSLATE-0004-data-residency-bound-inference.md
  - microservices/translate/policy/data-residency.md
  - microservices/translate/slos/data-residency-correctness.openslo.yaml
  - microservices/translate/threat-model.md (T-RESIDENCY-*)
  - microservices/translate/incident-response.md
  - microservices/translate/compliance.md
doc_status: published
---

# Runbook: Sovereign-tenant cross-region inference leak — P0

## Stop and read first

This is a **P0 data-residency invariant violation**. It is a HARD failure per
ADR-TRANSLATE-0004. Treat every step below as forensically observable. Do
NOT delete, mutate, or modify any evidence chain. Engage council-privacy +
legal-counsel + ops-security in the first 15 minutes regardless of perceived
blast radius. The clock starts now.

## Trigger

Any of:

- FM-50: `oya_translate_data_residency_violation_total > 0` (HARD invariant — must always be zero).
- Evidence-emitter detects `(tenant_pack, engine_region)` mismatch in `EngineRouted` event.
- External finding: tenant DPO escalation that sovereign content reached non-resident endpoint.
- Regulator finding: KR PIPC / EU DPA / CN CAC / IN DPB notification.
- Cell µservice cross-region traffic anomaly on `translate` namespace egress.

## Severity

- **Sev-1 P0** unconditionally. No demotion possible.

## Pack-Specific Regulatory Escalation Map

| Pack | Regulator | Notification clock | Statute |
|---|---|---|---|
| pack-kr | KR PIPC + KISA | within 72 h (DPA) + within 24 h (KISA messenger) | KR PIPA Arts. 28 + 29-2 + 34 + 28-2 cross-border |
| pack-eu | National DPA (e.g., BfDI / CNIL / DPC) + EDPB | within 72 h | GDPR Arts. 33-34, Arts. 44-50 cross-border |
| pack-us-healthcare | HHS OCR | within 60 days (or per state law) | HIPAA Breach Notification Rule 45 CFR §164.404 |
| pack-jp | PPC (Personal Information Protection Commission) | promptly | APPI Art. 24 (cross-border restriction) + Art. 26 (breach) |
| pack-sg | PDPC | within 72 h | PDPA Singapore §26B + cross-border transfer guidance |
| pack-au | OAIC | within 72 h | Privacy Act 1988 APP 8 + Notifiable Data Breaches |
| pack-in | DPB (Data Protection Board) | within 72 h | DPDPA 2023 §16 + §17 |
| pack-br | ANPD | within 2 business days | LGPD Arts. 33 + 48 |
| pack-ae | UAE Data Office | promptly | UAE PDPL Art. 9 cross-border + Art. 23 breach |
| pack-ksa | SDAIA / NDMO | within 72 h | KSA PDPL Art. 27 + cross-border restrictions |
| pack-cn-stub | CAC + MIIT | promptly | CN Cybersecurity Law Art. 37 + DSL Arts. 31-37 + PIPL Arts. 38-43 |

Notes:

- The clock starts at **detection** (not classification). Defer-classification escalation is itself reportable as a breach of internal controls.
- "Promptly" interpreted operationally as within 72 h to align with the strictest matching clock unless legal-counsel directs otherwise.

## Immediate Mitigation (≤ 15 min — HALT FIRST)

| Step | Action | Time |
|---|---|---|
| 1 | **HALT** translate-router for the affected pack: `cargo run -p oya-dev-cli -- translate halt-pack --pack <p>` | ≤ 2 min |
| 2 | Engage council-privacy + legal-counsel + ops-security via `#inc-translate-p0` (PagerDuty escalation chain) | ≤ 5 min |
| 3 | Identify scope: query `EngineRouted` events for window where `tenant_pack != engine_region` | ≤ 10 min |
| 4 | Snapshot evidence (read-only): `cargo run -p oya-dev-cli -- audit-chain snapshot --pack <p> --window <ts>..<ts>` → preserve to `evidence/p0-translate-residency-leak/<inc-id>/` | ≤ 10 min |
| 5 | Identify affected tenant(s); engage tenant DPO contacts per `compliance.md` §"DPO chain" | ≤ 15 min |

**Do NOT** restart router on affected pack until forensic snapshot is sealed.

## Forensic Snapshot Procedure (chain-of-custody seal)

Per ADR-TRANSLATE-0004 §"chain-of-custody":

1. Snapshot `audit-chain` events for the affected pack + window — read-only export.
2. Snapshot `EngineRouted` events with `(tenant_id, engine_region, source_region, content_hash)` triples.
3. Compute `SHA-256` over snapshot bundle.
4. Sign with `Ed25519` audit-chain root key: `cargo run -p oya-dev-cli -- audit-chain seal --snapshot <path> --inc-id <id>`.
5. Persist seal + signature to `evidence/p0-translate-residency-leak/<inc-id>/seal.json`.
6. Council-privacy verifies chain.

## Audit-Chain Procedure

Per ADR-0028 + ADR-TRANSLATE-0004:

- Every `EngineRouted` event MUST include `tenant_pack` + `engine_region` + `cross_border_consent_ref` (null when no consent).
- The leak event is detectable by:
  ```promql
  oya_translate_data_residency_violation_total
    = count(EngineRouted{tenant_pack != engine_region AND cross_border_consent_ref == null})
  ```
- Audit-chain seals each event with Ed25519; the tampering bar is the
  cryptographic chain (no auditor can selectively delete; insertion is detected
  by chain-length mismatch).

## Tenant Notification (≤ 72 h — coordinate with legal-counsel)

Per `compliance.md` §"Tenant notification template":

1. Identify each affected tenant.
2. Draft per-tenant notification including:
   - What happened (data-residency invariant violation).
   - Which content classes affected (segment count, data class, date range).
   - Which engine/region received the content.
   - Mitigation taken (halt + rollback + adapter version pinned).
   - Tenant-side action required (none for invariant; tenant DPO may file own DPA notice).
3. Council-privacy + legal-counsel countersign before send.
4. Audit-chain emit `TenantNotificationSent{inc_id, tenant_id, ts, signature}`.

## Regulator Notification (per pack — see map above)

1. Engage legal-counsel.
2. Within the per-pack clock (see escalation map), file notification with the applicable regulator.
3. Use pre-drafted regulator templates from `compliance.md` §"Regulator notification templates" (KR/EU/US-HC/JP/SG/AU/IN/BR/AE/KSA).
4. Audit-chain emit `RegulatorNotificationFiled{inc_id, regulator, filing_ref, ts, signature}`.

## Root-Cause Investigation

Common root causes:

| RCA | Description | Detection |
|---|---|---|
| RCA-1 | Router decision-tree bug: per-pack engine whitelist missed a candidate | unit test gap in `oya-translate-router-domain` |
| RCA-2 | Cell µservice routing: cross-region pod selected by mistake | mesh routing config drift |
| RCA-3 | Adapter misconfiguration: vendor region inferred from default rather than pack | adapter init review |
| RCA-4 | Cross-border consent expired but request still routed | consent-expiry-monitor failure |
| RCA-5 | Per-tenant override: tenant operator set per-engine override; ADR-TRANSLATE-0004 §"override-policy" violated | per-tenant entitlement review |
| RCA-6 | Network partition fallback: cross-region failover engaged outside policy | failover-policy review |

## Resolution Path

1. Identify RCA.
2. Patch + ship hotfix.
3. Run `tests/integration/data_residency_invariant.rs` per pack — must be 100 % green.
4. Re-enable translate-router on affected pack only after council-privacy + ops-security sign-off.
5. Audit-chain emit `PackResumed{inc_id, pack, ts, signature}`.

## Verification Commands

```bash
# Invariant — must always be zero
cargo run -p oya-dev-cli -- translate verify-residency-invariant \
  --pack <p> --window 24h
# expects: zero violations

# Per-pack engine routing
cargo run -p oya-dev-cli -- translate audit-engine-routing \
  --pack <p> --window 7d

# Cross-border consent matrix
cargo run -p oya-dev-cli -- translate audit-cross-border-consent \
  --pack <p>

# Audit-chain seal verification
cargo run -p oya-dev-cli -- audit-chain verify-seal \
  --inc-id <id>
```

## Verification After Recovery

- `oya_translate_data_residency_violation_total == 0` for 30 days sustained.
- `tests/integration/data_residency_invariant.rs` re-run green per pack.
- Per-tenant chain-of-custody seal verified.
- Audit-chain seal intact (cryptographic verification).
- Regulator notification filings sealed.
- ADR-TRANSLATE-0004 amendment shipped if policy changed.

## Postmortem

- Within 5 business days (HARD; per masterplan SLO-gated promotion).
- Council-privacy + legal-counsel + ops-security + axis-translate sign blameless postmortem.
- Outputs:
  - ADR-TRANSLATE-0004 amendment if needed.
  - Per-RCA preventive controls (CI gates, validators, alerting).
  - Public statement IF required by regulator filing.
  - `evidence/postmortems/<inc-id>.md` sealed.

## Named Industry Sources

- GDPR Arts. 33-34 (breach notification) + Arts. 44-50 (cross-border).
- KR PIPA Art. 28 (cross-border) + Art. 29-2 (automated) + Art. 34 (breach).
- EU AI Act (Reg. (EU) 2024/1689) Art. 9 (risk management) + Art. 15 (accuracy).
- HIPAA Breach Notification Rule 45 CFR §164.404 + 45 CFR §164.314 (BAA).
- APPI Art. 24 + Art. 26.
- PDPA SG cross-border transfer guidance + §26B (data-breach notification).
- AU Privacy Act 1988 APP 8 + Notifiable Data Breaches.
- DPDPA 2023 §16 (cross-border) + §17 (breach).
- LGPD Arts. 33 + 48.
- UAE PDPL Art. 9 + Art. 23.
- KSA PDPL Art. 27.
- CN Cybersecurity Law Art. 37 + DSL Arts. 31-37 + PIPL Arts. 38-43.
- NIS2 Directive (EU) 2022/2555 — significant-incident notification.
- ISO 27001:2022 A.5.24 / A.5.25 / A.5.26 (incident management).
- SOC 2 Trust Services Criteria CC7.4.
- NIST SP 800-61r2 (incident response).

## References

- ADR-TRANSLATE-0004 (data-residency-bound inference).
- `microservices/translate/policy/data-residency.md`.
- `microservices/translate/slos/data-residency-correctness.openslo.yaml`.
- `microservices/translate/threat-model.md` T-RESIDENCY-*.
- `microservices/translate/incident-response.md`.
- `microservices/translate/compliance.md`.

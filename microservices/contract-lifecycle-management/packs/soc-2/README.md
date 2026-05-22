---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: soc-2
authoritative_source: AICPA Trust Services Criteria 2017 (TSP Section 100), 2022 revision
related_adrs: [ADR-0251, ADR-0244, ADR-0263]
date: 2026-05-21
---

# SOC-2 Pack Overlay — CLM

SOC-2 (Service Organization Control 2) is the AICPA assurance framework for service organizations. The Trust Services Criteria (TSC) span five categories: Security (CC), Availability (A), Processing Integrity (PI), Confidentiality (C), Privacy (P). CLM's default scope is Security + Confidentiality; Availability and Processing Integrity are layered when the tenant requires them.

## Active triggers

The `soc-2` pack is **mandatory** for any paid tenant (`tenant_class=paid`) and **optional but recommended** for `tenant_class=demo_trial`.

## Trust Services Criteria mapping

### Security (Common Criteria — CC1 through CC9)

- **CC1 — Control environment**: tenant + cell + pack activation registered; governance documents in `compliance.md`.
- **CC2 — Communication and information**: every state transition emits audit-chain event per ADR-0263.
- **CC3 — Risk assessment**: threat model in `threat-model.md`; failure modes in `failure-modes.md`.
- **CC4 — Monitoring activities**: SLOs in `slos/*.openslo.yaml`; dashboards in `dashboards/*.json`.
- **CC5 — Control activities**: Cedar default-deny per ADR-0243; segregation of duties.
- **CC6 — Logical and physical access controls**: identity µservice for principal authentication; SPIFFE/SVID workload identity; cell isolation per ADR-0248.
- **CC7 — System operations**: incident response in `incident-response.md`; runbooks in `runbooks/`.
- **CC8 — Change management**: every code change passes the Foundry pipeline per ADR-0112.
- **CC9 — Risk mitigation**: business continuity in `multi-region.md`; capacity planning in `capacity-model.md`.

### Confidentiality (C1)

- Document-level confidentiality classification per `legal-dimensions/confidentiality-classification-overlay.md`.
- Tenant-scoped projection prevents cross-tenant leakage by construction.
- Encryption-at-rest with tenant-scoped keys.
- Egress controls via Cedar policy.

### Availability (A1)

When the tenant requires SOC-2 Type 2 with Availability:

- Per-tenant SLO sub-objectives carved out of the canonical SLOs.
- Cross-region failover per `multi-region.md`.
- Capacity headroom maintained per `capacity-model.md`.

### Processing Integrity (PI1)

When the tenant requires SOC-2 Type 2 with Processing Integrity:

- Contract state transitions are atomic; partial states never reach storage.
- Replay tests in `tests/` validate state-machine invariants.
- Backfill replay capability per `backfill-replay.md`.

### Privacy (P1)

When the tenant requires SOC-2 Type 2 with Privacy:

- Composes with `gdpr` and/or `kr-pipa` packs.
- Notice-and-consent flows per `legal-dimensions/esign-consumer-disclosure-flow.md` + `legal-dimensions/gdpr-article-7-consent-records.md`.

## Audit evidence package

CLM produces a SOC-2 audit evidence bundle on demand:

- Tenant + cell + pack activation history.
- All Cedar policy evaluations for the audit period (with PII suppression).
- All audit-chain events for the audit period.
- SLO compliance report.
- Incident log + post-incident reviews.
- Change-management ledger from the Foundry pipeline.
- Backup + restore drill evidence.

## Retention overlay

SOC-2 audit evidence retained for the audit period + 7 years.

## Cedar gate fragment

```cedar
permit (
  principal,
  action == Action::"AuditEvidenceExport",
  resource is Tenant
) when {
  principal.role == "auditor" &&
  principal.engagement_letter_signed == true &&
  resource.active_packs.contains("soc-2")
};
```

## Composition with other packs

- `soc-2` + `gdpr`: SOC-2 Privacy criterion satisfied by GDPR overlay.
- `soc-2` + `iso-27001`: significant control overlap; bridged via the AICPA / ISO crosswalk.
- `soc-2` + `hipaa-baa`: SOC-2 Security + HIPAA Security Rule §164.308 satisfied jointly.

## Evidence on activation

- `oya.contract.lifecycle.management.pack.soc-2.activated` audit event with the tenant's declared TSC categories.
- Cedar policy compilation.
- Quarterly evidence-export task in workflow-engine.

## Standards references

- AICPA Trust Services Criteria 2017 (TSP Section 100, 2022 revision).
- AICPA Description Criteria for a Description of a Service Organization's System.
- ISAE 3000 / ISAE 3402 for international equivalence.

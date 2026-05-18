---
microservice: compliance
doc: ComplianceMapping
status: Drafting
authority_tier: 2
owner: axis-compliance
date: 2026-05-18
related_adrs: [ADR-0209]
---

# Compliance — Regulatory Framework Mapping

## SOC 2 Type II — AICPA Trust Services Criteria

| Criterion | Required artifact kinds | Cadence | Collector |
|---|---|---|---|
| CC1 Control Environment | access-review-snapshot | weekly | accessReviewSnapshot |
| CC2 Communication & Information | (informational; tracked via Runbooks index per ADR-0170) | continuous | — |
| CC3 Risk Assessment | vuln-scan-report + pen-test-report | per image + yearly | vulnScanReport + penTestReport |
| CC4 Monitoring Activities | minimum-necessary-access-log + audit-chain seal coverage | continuous | minimumNecessaryAccessLog |
| CC5 Control Activities | deploy-receipt + ci-artifact-hash | per deploy + per build | deployReceipt + ciArtifactHash |
| CC6 Logical & Physical Access | access-review-snapshot | weekly | accessReviewSnapshot |
| CC7 System Operations | backup-restore-drill-receipt | quarterly | backupRestoreDrillReceipt |
| CC8 Change Management | deploy-receipt + ci-artifact-hash | per deploy + per build | deployReceipt + ciArtifactHash |
| CC9 Risk Mitigation | vuln-scan-report + pen-test-report | per image + yearly | vulnScanReport + penTestReport |
| A1 Availability | backup-restore-drill-receipt + SLO burn-down evidence | quarterly + continuous | backupRestoreDrillReceipt + attestationAggregator |
| C1 Confidentiality | access-review-snapshot + audit-chain seal coverage | weekly + continuous | accessReviewSnapshot |
| PI1 Processing Integrity | audit-chain seal coverage (every operation sealed) | continuous | auditChainSealCoverage |
| P1-P8 Privacy | DSAR completion record + Cedar policy snapshots | per DSAR + weekly | dsarCompletionRecord |

## GDPR — General Data Protection Regulation (EU)

| Article | Requirement | Required artifact | Status |
|---|---|---|---|
| Art. 5 Principles | Lawfulness, purpose limitation, data minimization | access-review-snapshot + minimum-necessary-access-log | ✓ |
| Art. 12 Transparency | Communicate within 30 days | dsar-completion-record (statutory SLA tracking) | ✓ |
| Art. 15 Right of access | Subject can request export | dsar-completion-record (export sub-type) | ✓ |
| Art. 16 Rectification | Subject can correct data | dsar-completion-record (rectify sub-type) | ✓ |
| Art. 17 Right to erasure | Subject can request deletion | dsar-completion-record (delete sub-type) | ✓ |
| Art. 18 Restriction | Subject can restrict processing | dsar-completion-record (restrict sub-type) | Phase 2 |
| Art. 20 Portability | Machine-readable export | dsar-completion-record + JSON-LD export format | ✓ |
| Art. 30 Records of processing | RoPA register | RoPA register at policy/ropa.json | ✓ |
| Art. 32 Security | TLS + encryption-at-rest + access controls | (substrate via ADR-0145 + ADR-0148) | ✓ |
| Art. 33 Breach notification | 72-hour authority notification | EVT-PERSONAL-DATA-BREACH | ✓ |
| Art. 35 DPIA | High-risk processing DPIA | dpia.md | ✓ |

## HIPAA — Health Insurance Portability and Accountability Act

| Section | Requirement | Required artifact | Status |
|---|---|---|---|
| § 164.308(a)(1) Security management | Risk analysis | vuln-scan-report + pen-test-report | ✓ |
| § 164.308(a)(3) Workforce security | Access review | access-review-snapshot | ✓ |
| § 164.308(a)(4) Information access | Cedar policy snapshot | access-review-snapshot | ✓ |
| § 164.308(a)(7) Contingency plan | Backup + DR | backup-restore-drill-receipt | ✓ |
| § 164.312(a)(1) Access control | RBAC + audit logs | minimum-necessary-access-log | ✓ |
| § 164.312(b) Audit controls | Audit logs | minimum-necessary-access-log + audit-chain seal | ✓ |
| § 164.312(c) Integrity | Tamper-evident logs | audit-chain seal coverage | ✓ |
| § 164.312(e) Transmission security | TLS 1.3 | (substrate via ADR-0148) | ✓ |
| § 164.314 Business Associate Contracts | BAA inventory | baa-inventory-entry | ✓ |
| § 164.514(d) Minimum necessary | Per-access purpose log | minimum-necessary-access-log | ✓ |

## PCI-DSS 4.0 — Payment Card Industry Data Security Standard

Status: **out of scope unless `microservices/payments/` lands.** Substrate ready.

| Requirement | Status |
|---|---|
| Req. 1 Network security controls | substrate via ADR-0148 service mesh |
| Req. 2 Apply secure configurations | substrate via ADR-0181 image promotion |
| Req. 3 Protect stored account data | requires CDE; deferred |
| Req. 4 Protect cardholder data with strong cryptography | substrate via cosign + TLS 1.3 |
| Req. 5 Anti-malware | substrate via Trivy scanning |
| Req. 6 Develop secure systems | substrate via ADR-0205 + lint discipline |
| Req. 7 Restrict access | substrate via ADR-0183 Cedar |
| Req. 8 Authentication | substrate via Zitadel |
| Req. 9 Physical access | operator-cluster operator responsibility |
| Req. 10 Log and monitor | minimum-necessary-access-log + observability backplane |
| Req. 11 Test security | vuln-scan-report + pen-test-report |
| Req. 12 Information security policy | policy/ directory |

## ISO 27001 (mapping; informational)

oyatie's SOC 2 + HIPAA artifact coverage subsumes most ISO 27001 Annex A controls. Per-control mapping at `policy/iso-27001-annex-a-coverage.json` (Phase 1.5).

## EU AI Act (mapping; informational)

ADR-0118 EU AI Act Annex III refusal kernel covers the refusal posture. Compliance µservice consumes EU-AI-Act-related events (EVT-EU-AI-ACT-REFUSAL); no separate artifact kind required.

## References

- ADR-0209 — compliance evidence automation.
- AICPA Trust Services Criteria 2017 (with 2022 points of focus update).
- GDPR Articles 5, 12, 15-22, 30, 32, 33, 35.
- HIPAA Title 45 CFR §§ 160, 162, 164.
- PCI-DSS 4.0 — PCI SSC, 2022.

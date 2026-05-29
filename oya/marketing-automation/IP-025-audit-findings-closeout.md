---
doc_class: ImplementationPlan
ip_id: IP-025-audit-findings-closeout
microservice: marketing-automation
bounded_contexts: [documentation, evidence, quality-gates, remediation]
related_adrs: [ADR-0324, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + council-quality
tenant_class_aware: true
---

# IP-025: Audit Findings Closeout

## A. Problem

Wave-4 audits found Marketing Automation had P0 Big-8 gaps and that IP-006..IP-025 were stamp shells. The stamped closeout IP repeated the same generic rows as the defective files, so it could not close anything. The real closeout must reconcile `AUDIT-FINDINGS-2026-05-21.json`, `coherence-audit-2026-05-20.md`, `REMEDIATION-NOTES-2026-05-21.md`, and this Wave 15 IP-substance pass.

## B. Approach

Treat closeout as an evidence ledger, not a celebratory summary. Each audit finding gets one state: resolved by named artifact, preserved as already-substantive, deleted as duplicative, or deferred with a concrete follow-up. For this IP-substance pass, closeout records 20 rewritten stamp shells, 3 drive base IPs preserved, and no duplicative deletions.

## C. Deliverables

| Artifact | Change |
|---|---|
| `REMEDIATION-NOTES-2026-05-21.md` | Append `Wave 15-IP-substance scrub (2026-05-21)` with rewritten/preserved/deleted counts. |
| `AUDIT-FINDINGS-2026-05-21.json` | Leave source audit immutable unless a separate audit update protocol is invoked. |
| `coherence-audit-2026-05-20.md` | Reference as source evidence; do not rewrite during closeout. |
| `IP-006..IP-025` | Replace stamp shells with bespoke plans tied to real source, policy, contract, IaC, dashboard, runbook, and SLO files. |
| Verification output | Capture line-cluster, counterpart-reference, and remediation-note checks. |

## D. Implementation

1. Inventory every assigned IP file in `drive` and `marketing-automation`.
2. Classify stamped files using repeated headings, exact 55-line cluster, generic benchmark rows, and absence of real artifact references.
3. Preserve drive IP-002, IP-003, and IP-015 because they reference concrete drive crates, contracts, adapters, acceptance commands, and ADRs.
4. Rewrite Marketing Automation IP-006..IP-025 in place with specific problem, approach, deliverables, implementation, acceptance, evidence, and counterpart rows.
5. Run signature checks: `wc -l`, duplicate heading counts, counterpart-reference grep, and remediation note grep.
6. Record any residual short already-substantive files as follow-up only if they fail counterpart-reference checks.
7. Avoid updating source audit JSON during this pass because closeout evidence belongs in remediation notes.

## E. Acceptance

- `wc -l microservices/marketing-automation/IP-*.md | sort -n | awk '$1 > 30 && $1 < 80'` shows the 55-line stamp cluster removed or materially reduced.
- `grep -L 'HubSpot\\|Marketo\\|Mailchimp\\|Salesforce\\|Klaviyo' microservices/marketing-automation/IP-*.md` returns no rewritten IP-006..IP-025 files.
- `rg 'Wave 15-IP-substance scrub' microservices/marketing-automation/REMEDIATION-NOTES-2026-05-21.md` returns one appended section.

## F. Evidence

- Source audit: `AUDIT-FINDINGS-2026-05-21.json`.
- Source audit: `coherence-audit-2026-05-20.md`.
- Remediation ledger: `REMEDIATION-NOTES-2026-05-21.md`.
- Doctrine: ADR-0324 anti-template-stamping and ADR-0328 Big-8 P0 elevation.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Closeout verifies HubSpot-primary Marketing Automation IPs are no longer generic shells. |
| Adobe Marketo Engage | Closeout records Smart Campaign/API-style surfaces as concrete implementation plans. |
| Mailchimp | Closeout verifies audience, journey, consent, and campaign surfaces cite real local artifacts. |

## H. Local Traceability

- Audit source: `AUDIT-FINDINGS-2026-05-21.json`.
- Audit source: `coherence-audit-2026-05-20.md`.
- Ledger target: `REMEDIATION-NOTES-2026-05-21.md`.
- Rewritten range: `IP-006..IP-025`.
- Preserved drive file: `IP-002-file-store-kernel.md`.
- Preserved drive file: `IP-003-file-store-adapters.md`.
- Preserved drive file: `IP-015-hg-drive-registration.md`.
- Verification: line-count cluster check.
- Verification: counterpart-reference grep.
- Verification: stamped-text grep.
- Verification: remediation-note grep.
- Doctrine: ADR-0324 anti-template-stamping.
- Doctrine: ADR-0328 Big-8 P0 elevation.
- Count field: inventoried.
- Count field: detected stamped.
- Count field: rewritten.
- Failure state: source audit JSON rewritten without audit-update protocol.
- Failure state: closeout claims completion without verification output.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-025-audit-findings-closeout.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/marketing-automation/IP-025-audit-findings-closeout.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

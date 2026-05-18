---
doc_class: Runbook
template_id: TPL-RUNBOOK
title: Legal-Process Court-Order Receipt
microservice: anonymous
severity: "Sev-1 (court-order receipt is always a P0 workflow class)"
status: Accepted
owner_team: ops-security + general-counsel + axis-anonymous
date: 2026-05-17
related_adrs: [ADR-ANON-0003]
related_artifacts:
  - microservices/anonymous/PRD.md §I7
  - microservices/anonymous/policy/legal-process-disclosure.cedar
  - microservices/anonymous/incident-response.md
doc_status: published
---

# Runbook: Legal-Process Court-Order Receipt

## Trigger

Receipt of any of:

| Doctype | Authority | Path |
|---|---|---|
| Subpoena (US) | court, grand jury, administrative agency | Path A |
| Search warrant (US) | court | Path A |
| Preservation request (US ECPA §2703(f)) | law enforcement | Path A-Preserve |
| Court order (US ECPA §2703(d)) | court | Path A |
| IPA 2016 Targeted Interception Warrant (UK) | Secretary of State | Path B |
| IPA 2016 Targeted Equipment Interference Warrant (UK) | Secretary of State | Path B |
| 통신비밀보호법 §9 warrant (KR) | district court | Path C |
| 통신비밀보호법 §9-2 emergency-disclosure order (KR; gag-order doctrine) | prosecutor | Path C-Emergency |
| 통신의 비밀 court order (JP) | Tokyo District Court (or local) | Path D |
| NCMEC CyberTipline preservation request (US 18 USC §2258A) | NCMEC | Path E |
| EU MLAT cross-border request (any pack-eu tenant) | requesting state via MLAT | Path F-MLAT |

## Severity

- **Sev-1** — every legal-process intake is Sev-1. The 14-day notice clock starts on intake; failing to execute on time is a regulatory + contractual violation.

## Pre-conditions

| Pre-condition | Verified by |
|---|---|
| Court order is signed and from a recognised authority | general-counsel manual review |
| Order specifies the doctype and the records sought | manual review |
| Tenant whose data is sought is identified | manual review |
| The disclosure is technically possible (i.e., records exist and have not been hard-deleted under retention policy) | `cargo run -p oya-dev-cli -- anonymous legal-process probe --order-id <id>` |

## Steps — Path A (US subpoena / warrant / court order)

| Step | Action | Time budget | Owner |
|---|---|---|---|
| 1 | Receive order at legal@oyatie.dev; record in `legal_process_intake` queue | ≤ 4h from receipt | ops-security |
| 2 | general-counsel reviews validity, scope, and authority | ≤ 24h | general-counsel |
| 3 | If invalid / overbroad / unauthorised: file challenge with court within 14 days | per docket | general-counsel |
| 4 | If valid: record order in `LegalProcessOrder` (via `cargo run -p oya-dev-cli -- anonymous legal-process record --doctype <X> --court <Y> --order-id <Z>`) | ≤ 1 day | ops-security |
| 5 | Determine user-notice posture: (a) standard 14-day notice (ECPA §2705(a)) OR (b) court-prohibited gag-order (ECPA §2705(b)) | ≤ 1 day | general-counsel |
| 6 | If 14-day notice: send notice to affected user via `oya-anonymous-legal-process-disclosure-rest` API | day 1 of disclosure clock | ops-security |
| 7 | Wait 14 days for user to challenge (or proceed immediately if court-prohibited) | 14 days | – |
| 8 | Two distinct approvers sign dual-control approval per `policy/legal-process-disclosure.cedar` PERMIT 2 | ≤ 1 day | LegalProcessApprover x2 |
| 9 | Initialise chain-of-custody: `cargo run -p oya-dev-cli -- anonymous legal-process init-coc --order-id <id>` | ≤ 5 min | ops-security |
| 10 | Execute disclosure via `cargo run -p oya-dev-cli -- anonymous legal-process execute --order-id <id>` (this is the ONLY operation that correlates user_id ↔ post_id in the µservice) | ≤ 1h | LegalProcessExecutor |
| 11 | Seal disclosure package: `cargo run -p oya-dev-cli -- anonymous legal-process seal --order-id <id> --output /tmp/disclosure.tar.gz.gpg` | ≤ 10 min | LegalProcessExecutor |
| 12 | Deliver to law enforcement via signed, encrypted channel per court-specified method | per court directive | general-counsel |
| 13 | Record completion in audit-chain (`LegalProcessDisclosureExecuted` event) | ≤ 5 min | system |
| 14 | Add to transparency-report queue (with court-prohibited flag if applicable) | ≤ 5 min | system |

## Steps — Path B (UK IPA 2016)

| Step | Action | Notes |
|---|---|---|
| 1-3 | as Path A; general-counsel verifies warrant is from Secretary of State and is within statutory scope | – |
| 4 | gag-order check: IPA 2016 §57 by default prohibits disclosure of the warrant's existence to the user | gag-order is default; not exception |
| 5-13 | as Path A but without user-notice (user-notice is structurally prohibited) | transparency-report records anonymised aggregate only |

## Steps — Path C (KR 통신비밀보호법 §9)

| Step | Action | Notes |
|---|---|---|
| 1-3 | as Path A; general-counsel verifies district-court warrant + scope per Art. 9 | – |
| 4 | Gag-order check: §9-2 governs emergency disclosure; if §9-2 invoked, gag-order prohibits user-notice | – |
| 5 | If standard §9 warrant: user-notice required (per PIPC guidance); if §9-2: gag-order applies | – |
| 6-13 | as Path A | – |

## Steps — Path C-Emergency (KR §9-2 emergency)

Per KR 통신비밀보호법 §9-2 emergency-disclosure, a prosecutor may order emergency disclosure without prior court approval, but the order MUST be retroactively validated by a court within 36 hours. If retroactive validation fails, the disclosure is **unlawful** and must be reported to the PIPC; the platform's chain-of-custody seal protects against tampering during the validation window.

| Step | Action | Time budget |
|---|---|---|
| 1 | Receive prosecutor's §9-2 order | ≤ 1h |
| 2 | general-counsel verifies emergency justification | ≤ 6h |
| 3 | Execute disclosure under §9-2 emergency (without prior court approval) | ≤ 12h |
| 4 | Await retroactive court validation (within 36h of execution) | – |
| 5 | If retroactive validation FAILS: file report with PIPC + audit-chain seal the unlawfulness event | ≤ 24h after invalidation |

## Steps — Path D (JP 通信の秘密)

| Step | Action | Notes |
|---|---|---|
| 1-3 | as Path A; general-counsel verifies Tokyo District Court order (or local district court) | – |
| 4 | gag-order check: JP Telecom Business Act + Constitution Art. 21 (通信の秘密) means default gag-order applies unless order specifies otherwise | – |
| 5-13 | as Path A | – |

## Steps — Path E (NCMEC CyberTipline preservation)

| Step | Action | Notes |
|---|---|---|
| 1 | Receive NCMEC preservation request | ≤ 1h |
| 2 | NCMEC requests are accepted on receipt (no validity challenge; statutory under 18 USC §2258A) | – |
| 3 | Initialise chain-of-custody | ≤ 5 min |
| 4 | Preserve relevant records via `cargo run -p oya-dev-cli -- anonymous legal-process preserve --order-id <ncmec-id>` | ≤ 1h |
| 5 | File CyberTipline report within 48h of original classifier verdict (PRD FR-27) | per statute |
| 6 | Audit-chain seal | ≤ 5 min |

## Steps — Path F-MLAT (EU cross-border)

| Step | Action |
|---|---|
| 1 | Receive MLAT request from requesting state via host-state ministry of justice |
| 2 | general-counsel verifies MLAT applies (treaty-existence check) |
| 3 | Disclosure executes in user's pack (pack-eu) under pack-eu law; NOT in requesting state's pack |
| 4 | Output delivered to host-state ministry of justice; requesting state receives via MLAT channel |
| 5 | Audit-chain seal records both the MLAT request and the cross-pack-refusal (no data left pack) |

## Post-execution

- **Transparency-report inclusion**: Within 1 business day, the disclosure is recorded in the per-pack transparency-report aggregator with the breakdown (doctype, jurisdiction, count). Court-prohibited entries are recorded with `gag_order = true` and contribute to the anonymised aggregate only.
- **Tenant notification (where permitted)**: If the tenant is not the user and the order does not gag the tenant, the tenant operator receives a notification within 5 business days of execution.
- **Post-mortem**: Within 5 business days, ops-security + general-counsel review the disclosure for procedural integrity; review captured in `evidence/legal-process/<order-id>/postmortem.md`.

## Failure modes during the workflow

| Failure | Mitigation | Severity escalation |
|---|---|---|
| Dual-control approval cannot be obtained within 7 days of clock-start | escalate to general-counsel + Council Privacy chair; consider court-extension request | Sev-1 |
| Chain-of-custody hash mismatch during execution | abort execution; audit-chain seal the abort; restart from chain-of-custody init | Sev-1 |
| Disclosure package delivery fails | retry; if persistent, deliver via court-specified backup channel | Sev-2 |
| User attempts to challenge during 14-day notice | pause execution; general-counsel handles challenge in court | per docket |
| Discovery of hard-deleted record (data no longer exists) | respond to court with tombstone evidence + audit-chain Merkle proof of deletion | Sev-2 |
| Internal team member raises concern about disclosure validity | pause; escalate to general-counsel; do not proceed | Sev-1 |

## Cedar policy enforcement reminder

Every step in this runbook is gated by `policy/legal-process-disclosure.cedar`. The Cedar policy defaults-deny; only the documented permits authorise disclosure operations. **Bypassing Cedar via direct DB writes is a P0 incident**.

## References

- ADR-ANON-0003 (legal-process workflow)
- ECPA / SCA §§2510-2523, 2701-2712
- UK IPA 2016 §§70-71, §57 (gag-order doctrine)
- KR 통신비밀보호법 Arts. 9, 9-2
- 18 USC §2258A (NCMEC)
- JP Constitution Art. 21 (通신의 비밀); Telecom Business Act
- EU MLAT (Mutual Legal Assistance Treaties)
- ADR-0028 (audit-chain Merkle / Ed25519)

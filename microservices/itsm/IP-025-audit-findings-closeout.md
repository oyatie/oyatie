---
doc_class: IP
ip_id: IP-025-audit-findings-closeout
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + council-quality
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/AUDIT-FINDINGS-2026-05-21.json
  - microservices/itsm/REMEDIATION-NOTES-2026-05-21.md
  - microservices/itsm/coherence-audit-2026-05-20.md
  - microservices/itsm/manifest.json
---

# IP-025 ITSM Audit Findings Closeout

## A. Problem
The audit closeout IP cannot be a final stamped "we closed it" note. ITSM has known remediation context: stamped IP shells, bounded-context plurality repairs, ServiceNow/Jira/Freshservice counterpart coverage, and PRD/architecture evidence. Closeout must prove which findings are closed, which are deferred, and which evidence files support each claim.

This IP defines the closeout packet for the ITSM Wave 15 remediation set.

## B. Approach
Use the audit and remediation files as evidence sources:

| Evidence | Purpose |
|---|---|
| `AUDIT-FINDINGS-2026-05-21.json` | machine-readable finding status |
| `coherence-audit-2026-05-20.md` | original defect context |
| `REMEDIATION-NOTES-2026-05-21.md` | human-readable change ledger |
| `manifest.json` | counterpart, pack, tenant-class, and dependency evidence |
| rewritten IP files | implementation-plan closeout evidence |

Closeout should never use line count alone.

## C. Deliverables
- Finding-to-evidence matrix for ITSM.
- List of rewritten, preserved, and deleted IPs.
- Verification command log for stamp signatures, counterpart references, and notes section.
- Residual-risk list for missing implementation code or test gaps.
- Promotion recommendation: pass, conditional pass, or blocked.

## D. Implementation
1. Inventory all ITSM IP files and record which are rewritten vs preserved.
2. Link each closed finding to one or more real files, not a prose claim.
3. Run stamp-shell checks: line clusters, duplicate headings, placeholder search, counterpart grep.
4. Run contract/code smoke checks where feasible, at least `cargo test` for ITSM crate if available.
5. Append the Wave 15-IP-substance scrub section to remediation notes.
6. Record any duplicates deleted or merged; if none, say none.
7. Record residual risks such as docs-only IPs awaiting implementation.
8. Update `AUDIT-FINDINGS-2026-05-21.json` only if the schema and status workflow are clear; otherwise leave as follow-up.

## E. Acceptance
- Closeout matrix names file paths and evidence, not line counts.
- Stamp-shell verification has no ITSM IP between 30 and 80 lines after rewrite.
- Counterpart references are present in rewritten IPs.
- Remediation notes contain the Wave 15-IP-substance scrub section.

## F. Evidence
- `AUDIT-FINDINGS-2026-05-21.json` exists.
- `REMEDIATION-NOTES-2026-05-21.md` exists.
- `manifest.json` names ServiceNow ITSM, Jira Service Management, and Freshservice as top-3 counterparts.
- ADR-0324 forbids template stamping; ADR-0328 makes substance and batch discipline canonical.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow ITSM parity audit | Closeout proves ServiceNow-class gaps are tracked |
| Jira Service Management parity audit | Evidence files bind claims to real artifacts |
| Freshservice parity audit | Residual risks remain explicit instead of hidden |

## H. Cold-start buildability notes
- Run line-cluster checks after every rewrite batch.
- Run duplicate-heading checks before claiming stamps are gone.
- Use counterpart grep as a smoke check, not final proof.
- List preserved IPs separately from rewritten IPs.
- Never mark a finding closed on line count alone.
- Link every closeout claim to one or more file paths.
- Record deleted duplicates with rationale; record none if none.
- Keep audit JSON untouched unless schema workflow is clear.
- Add residual implementation gaps to follow-ups.
- Preserve verification command output in the final summary.

---
doc_status: published
---

# Team: Ops — Compliance

## Mission
This team owns the per-regulator compliance posture for Oyatie: the compliance matrix, regulatory evidence portal, per-regulator owner assignments, and the quarterly compliance-evidence-pack regeneration SLA. It exists because the cohesion thesis requires *one* compliance posture spanning every axis and every vertical — a healthcare tenant and a fintech tenant on the same platform share the same audit chain, and the compliance team ensures that each regulator (KISA, MFDS, FSC, KCC, NIS, FDA, EMA, CSAP, FedRAMP, GDPR, DORA, etc.) gets a complete, accurate, timely evidence pack. It does **not** own the audit chain infrastructure (→ `platform-audit-evidence`) or the security program (→ `ops-security`); it assembles the evidence packs from what those teams produce.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting compliance operations
- **Surfaces:**
  - Compliance matrix (`COMPLIANCE-MATRIX.md`) — per-regulator control mapping
  - Evidence portal (read surface hosted on `axis-search` infrastructure; content assembled here)
  - Per-regulator owner assignments (one owner per regulator in the matrix)
  - Regulatory-change watch lane (quarterly per ADR-0050 governance umbrella)
  - Audit readiness program: self-assessment, mock audits, evidence-pack drill
- **Cross-axis contracts (DESIGN §10):** (consumer of audit chain, DSR cascade, and security controls; no contract ownership)
- **Catalog records:** compliance process tooling (no product crates)
- **Runbooks:** `runbooks/evidence-pack-generation.md`, `runbooks/regulatory-change-response.md`, `runbooks/dsr-compliance-report.md`, `runbooks/breach-notification.md`
- **ADRs:** ADR-0050 (AI/ML governance — compliance sections), regulatory-change governance ADR

## In-scope work
- Compliance matrix: map every Oyatie surface to every applicable regulatory control (KISA, MFDS, FSC, KCC, NIS, CSAP, K-ISMS-P, KCMVP, PIPA Art-23, HIPAA/HITECH, SOX, GDPR, DORA, EU AI Act, FedRAMP, APPI, ISMAP, DPDP, LGPD, PDPL, NDMO, TDRA)
- Evidence pack generation: per-regulator pack with control evidence from audit chain, security program, and per-vertical teams; regeneration SLA ≤ 4 h (PRD §4.2)
- Regulatory-change watch: monthly scan of each regulator's publication channel; trigger ADR on material change
- Per-regulator owner: one named compliance owner per regulator in the matrix; escalation point for auditor queries
- Mock audits: quarterly self-assessment against KR CSAP/K-ISMS-P/KCMVP; annual third-party audit facilitation
- DSR compliance reporting: per-jurisdiction DSR fulfillment rate reports for PIPA/GDPR/CCPA regulators
- Breach notification: coordinate with `ops-security` and legal counsel on GDPR 72-h / PIPA notification obligations
- Vertical regulatory watch: monthly updates to `vertical-healthcare`, `vertical-fintech`, `vertical-industrial`, `vertical-public-sector`, `vertical-legal` teams on their regulator changes

## Out-of-scope (anti-scope)
- Audit chain infrastructure (→ `platform-audit-evidence`)
- Security program (→ `ops-security`)
- Legal counsel and contract review (→ `gtm-partnerships` + external counsel)
- Per-vertical domain regulatory implementation (→ per-vertical teams implement; compliance team validates coverage)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-audit-evidence` | Audit chain evidence for every control | Per evidence-pack request |
| `ops-security` | Security controls evidence, CVE status, pentest results | Monthly |
| `platform-privacy-dub` | DSR cascade completion records, consent withdrawal compliance | Per DSR event |
| All vertical teams | Per-vertical regulatory control evidence | Monthly + on audit demand |
| `axis-cloud` | CSAP / K-ISMS-P / KCMVP evidence from cloud team | Quarterly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All vertical teams | Regulatory-change signals for their regulator set | Monthly |
| `gtm-customer-success` | Compliance status for design-partner trust conversations | Monthly |
| `gtm-sales-se` | Compliance posture summary for enterprise sales | Per RFP |
| `council-architecture` | Compliance gap list for wave-gate readiness | Per wave |

## Success metrics
- **Regulatory evidence pack regeneration time:** ≤ 4 h from request (PRD §4.2)
- **Compliance matrix coverage:** 100% of production surfaces mapped to applicable controls
- **Per-regulator owner assignment coverage:** 100% of active regulators
- **Regulatory-change watch lag:** ≤ 30 days from regulatory publication to ADR trigger
- **DSR fulfillment rate per GDPR/PIPA:** 100% within statutory deadline
- **GDPR breach notification turnaround:** < 72 h from detection (regulatory requirement)
- **Mock audit pass rate:** ≥ 95% of controls with evidence at quarterly self-assessment

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for cross-axis compliance architecture disputes
- Legal: founder + counsel for breach notification, regulatory enforcement actions
- Founder: as last resort (regulatory enforcement action)

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — evidence-pack status, regulatory-change queue, DSR fulfillment rate
- Cross-team review: monthly compliance briefing with all vertical teams; quarterly CSAP self-assessment

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1) — compliance headcount grows with each new regional pack
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (compliance tooling PRs)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; regulatory-change ADRs triggered immediately on material change

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Evidence pack not ready within 4 h of auditor request | High | Pre-generated quarterly packs; on-demand regeneration pipeline |
| Regulatory change missed → compliance gap at audit | High | Monthly regulatory-change watch; automated subscription to KISA/MFDS/FSC/FDA/EMA RSS/API feeds |
| DSR breach notification missed 72-h GDPR deadline | Catastrophic | PagerDuty alert at 48 h; breach notification runbook tested quarterly |
| Compliance matrix outdated → regulator finds uncovered control | High | Monthly matrix review; fitness function checks catalog records against matrix |

## Sources scanned
PRD.md §6 (regulatory posture), §4.2 (evidence pack regeneration metric), DESIGN.md §10, ADR-0050, DOC-CATALOG.md §2.1 (doc.compliance_matrix owner = ops-compliance).

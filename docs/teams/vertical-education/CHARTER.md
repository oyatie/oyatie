---
doc_status: published
---

# Team: Vertical — Education (LMS)

## Mission
This team owns the education vertical: Learning Management System (LMS), course authoring, student progress tracking, credentialing, and content compliance (KR 청소년보호법, COPPA, FERPA). This is a **skeleton team** — scope is defined but staffing and wave timing are deferred to W-Vertical-Fan-Out.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Education (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-education-kernel` — `Course`, `Enrollment`, `LearnerRecord`, `Credential`, `ContentItem`
  - `vertical-education-domain-*` — LMS lifecycle, course authoring, progress tracking, credential issuance
  - Products owned: `products/vertical-education/PRD.md` (skeleton)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — credential issuance, assessment results)
  - `Search index lifecycle` (consumer — course catalog search via tenant-private index)
- **Catalog records:** `crates/vertical-education-*`
- **Runbooks:** TBD at W-Vertical-Fan-Out
- **ADRs:** TBD; FERPA/청소년보호법 compliance ADR at activation

## In-scope work
- LMS: course enrollment, content delivery (SCORM/xAPI), progress tracking, assessment
- Course authoring: rich-media editor, quiz builder, learning path designer
- Credentialing: digital badge issuance, certificate generation, transcript export
- KR 청소년보호법 content compliance: age-gating, content rating, parental consent
- FERPA (US): student record privacy, parental rights, third-party disclosure controls
- COPPA (US): under-13 consent management
- Content search via tenant-private index

## Out-of-scope (anti-scope)
- Consumer education platform (B2B tenants — schools, enterprises — only)
- Cloud infrastructure (→ `axis-cloud`)
- Minor/student PII in any ad-targeting signal (always blocked)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-saas` | Workflow engine, OG for Course/Learner nodes | Per-release |
| `platform-privacy-dub` | Student PII classification (FERPA/COPPA overrides) | ADR lifecycle |
| `platform-audit-evidence` | Credential issuance and assessment audit records | Per event |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | FERPA / 청소년보호법 evidence packs | Quarterly |

## Success metrics
*(Defined at W-Vertical-Fan-Out wave gate)*
- **Student PII in any ad signal:** 0 (permanent)
- **Credential issuance audit completeness:** 100%
- **KR 청소년보호법 content gate coverage:** 100%

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council; privacy council for student PII disputes
- Compliance: `ops-compliance` for FERPA / 청소년보호법 incidents
- Founder: as last resort

## Communication cadence
- Stand-up: async (skeleton phase)
- Weekly: 30-min sync at W-Vertical-Fan-Out activation

## Bandwidth + hiring
- Current FTE: 0 (skeleton)
- Target FTE: TBD per axis-wave
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch once active

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Student PII (FERPA/COPPA) exposed via unclassified OG property | Catastrophic | Forced `internal_only` classification; fitness gate |

## Sources scanned
PRD.md §3.1 (W-Vertical-Fan-Out), DESIGN.md §1, DOC-CATALOG.md §2.5, products/vertical-education/PRD.md (skeleton).

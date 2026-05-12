# Team: Council — Architecture

## Mission
This council holds final decision authority on all cross-axis architecture contracts, the doc-catalog, wave-gate readiness, and the RACI ownership matrix. It exists because Oyatie's cohesion thesis — one product expressed across seven axes — only holds if someone can say "no" to a change that would undermine a cross-axis contract, and that someone must have the organizational authority to make it stick. The council is not a committee that delays decisions; it is the named escalation point that every team charter references for contract disputes, and it runs the quarterly contradiction audit that keeps the architecture honest.

The council does **not** write code, own product surfaces, or manage day-to-day engineering. It owns the *governance* layer: cross-axis contract table (DESIGN §10), wave-gate readiness checklists, doc-catalog integrity, and the RACI overlay.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting governance (not an axis itself — it governs the seams between all axes)
- **Surfaces:**
  - DESIGN.md §10 cross-axis contract table (sole authority on adding/removing/changing rows)
  - PRD.md (co-owner with founder — quarterly review, axis-scope changes require council vote)
  - ROADMAP.md (wave-gate readiness sign-off)
  - RACI-OWNERSHIP.md (final authority)
  - DOC-CATALOG.md (meta-supervisor of all docs)
  - GLOSSARY.md (resolves domain term disputes)
  - RISK-REGISTER.md (catastrophic/high entries require council review)
  - INTERNATIONALIZATION.md (locale canon)
  - HIRING-CAPACITY-PLAN.md (until COO/CFO hired)
  - Wave-gate readiness checklists (the gate criteria per PRD §3.1)
  - Cross-axis review label process (approves all cross-axis-labeled PRs at architecture level)
  - `teams/*/CHARTER.md` meta-supervision (each team owns its charter; council is the meta-supervisor)
- **Cross-axis contracts (DESIGN §10):**
  - All rows (sole authority to add/remove/change; each row has an owner-axis team, but the table as a whole is council property)
- **Catalog records:** none (council doesn't write product code)
- **Runbooks:** `runbooks/cross-axis-contradiction-audit.md`, `runbooks/wave-gate-readiness-check.md`
- **ADRs:** all cross-cutting ADRs where no single axis team is the natural owner; supersession graph audits

## In-scope work
- Cross-axis contract table (DESIGN §10): review, approve, or reject additions/changes/removals; quarterly contradiction audit per DESIGN §11
- Wave-gate readiness: run the readiness checklist for each PRD §3.1 wave gate; block the gate if any checklist item is open
- PRD quarterly review: with founder, review axis-scope changes, update PRD §3, record in PRD changelog
- ROADMAP.md wave-gate sign-off: the council signs the wave gate; no wave advances without council sign-off
- RACI-OWNERSHIP.md: maintain decision-rights matrix; resolve ownership disputes
- Doc-catalog integrity: meta-supervise all consolidated docs; trigger doc update protocol when EVT-* events fire
- Glossary: resolve domain term conflicts; maintain glossary freshness
- Risk register: review and classify catastrophic/high risks; assign owners; track resolution
- HIRING-CAPACITY-PLAN.md: maintain until founder hires COO/CFO
- Cross-axis contradiction audit: quarterly — compare consolidated docs for contradictions; publish resolution list to ROADMAP.md
- Architecture ADR review: all cross-cutting ADRs that no single axis team owns; final say on promotion to Accepted
- Charter meta-supervision: review new team charters before publication; annual charter refresh audit

## Out-of-scope (anti-scope)
- Day-to-day engineering decisions (→ per-axis teams)
- Security program (→ `ops-security`)
- Compliance evidence packs (→ `ops-compliance`)
- Privacy council decisions (→ `council-privacy`)
- Commercial decisions (→ founder + GTM teams)
- Writing product code (the council is a governance body, not an engineering team)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| All axis + vertical teams | Cross-axis contract change proposals; charter updates | Per proposal |
| `crew-adr-promotion` | ADR promotion status for cross-cutting ADRs | Monthly ADR batch |
| `ops-compliance` | Compliance gap list for wave-gate readiness | Per wave |
| `ops-sre-reliability` | Reliability posture for wave-gate readiness | Per wave |
| Founder | North-star arbiter for disputes above council authority | As needed |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All teams | Cross-axis contract dispute resolution; wave-gate sign-off | Per proposal + per wave |
| `crew-adr-promotion` | Cross-cutting ADR promotion authority | Monthly ADR batch |
| `platform-tenancy-identity` | All-axis review gate authority for `Tenant` shape changes | Per `Tenant` shape PR |
| `axis-foundry` | Autonomy-ceiling policy dispute authority | Per policy dispute |
| `council-privacy` | Cross-axis privacy architecture disputes | Per dispute |

## Success metrics
- **Cross-axis contract violations on `main`:** 0 per quarter (PRD §4.2)
- **Wave-gate readiness checklist completion before wave advance:** 100%
- **Quarterly contradiction audit completed:** 100% on schedule
- **Cross-axis PR review SLA (architecture council input required):** ≤ 3 business days
- **Doc-catalog meta-supervision:** every consolidated doc has a named owner team and update cadence defined
- **Charter refresh audit:** 100% of team charters reviewed annually

## Escalation path
- Internal: council lead → founder (north-star arbiter for disputes above council authority)
- Cross-council: privacy council for cross-axis privacy architecture disputes
- Founder: final arbiter; invoked only when council cannot reach consensus

## Communication cadence
- Stand-up: no stand-up (council is async-first)
- Weekly: 60-min council sync — cross-axis PR review queue, ADR promotion pipeline, wave-gate status
- Cross-team review: monthly contradiction audit debrief; quarterly wave-gate readiness review with all axis leads
- Annual: charter refresh audit with all teams

## Bandwidth + hiring
- Current FTE: Council members are drawn from axis tech leads (not a separate headcount pool)
- Quorum: majority of axis tech leads + founder representative
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (council tooling PRs)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; cross-cutting ADRs go to council for final vote

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Cross-axis contract change lands without council review | High | Cross-axis PR label required; fitness function checks for orphan contract changes |
| Wave gate advanced without readiness checklist completion | High | Council sign-off is a required gate; no wave advances without it |
| Contradiction audit not run quarterly | High | Quarterly audit is a council OKR; missed audit blocks next wave gate |
| Council becomes a bottleneck (review latency > 3 days) | Medium | SLA enforced; async review process; delegate to axis tech lead when unambiguous |

## Sources scanned
PRD.md §3 (wave gates), §4.2 (cross-axis violations metric), DESIGN.md §10 (contract table), §11 (contradiction audit), DOC-CATALOG.md §2.1 (doc.design, doc.prd, doc.raci_ownership, doc.glossary, doc.risk_register owners = council-architecture), §2.4 (doc.doc_catalog owner = council-architecture).

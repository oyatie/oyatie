---
doc_status: published
---

# Team: Vertical — Legal (Regulated Corpus / Contracts)

## Mission
This team owns the legal vertical: regulated legal corpus management, contract lifecycle, clause library, e-signature integration, matter management, and legal AI capabilities under strict autonomy ceiling constraints. It exists because legal tenants need a tamper-evident record of every contract version, every clause change, and every signature event — and because legal AI (contract analysis, clause recommendation, risk flagging) must never exceed the autonomy ceiling for regulated legal decisions. It does **not** own the SaaS workflow engine or cloud infrastructure.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Legal (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-legal-kernel` — `LegalDocument`, `Contract`, `Clause`, `Matter`, `SignatureEvent`, `CorpusEntry`
  - `vertical-legal-domain-*` — contract lifecycle, clause library management, matter tracking, corpus ingestion
  - `vertical-legal-adapter-esign` — e-signature integration (DocuSign, 전자서명법 KR compliance)
  - Per-region corpus extensions: KR (민법, 상법, 근로기준법, 개인정보보호법 corpus); US (UCC, federal contract templates); EU (EU AI Act compliance templates)
  - Products owned: `products/vertical-legal/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — every contract version, signature event, clause change)
  - `Autonomy ceiling policy` (consumer — legal AI clause recommendations require human-in-the-loop)
  - `Search index lifecycle` (consumer — regulated corpus search via tenant-private index)
- **Catalog records:** `crates/vertical-legal-*`
- **Runbooks:** `runbooks/legal-corpus-update.md`, `runbooks/esign-failure.md`
- **ADRs:** ADR-0033 (legal corpus schema)

## In-scope work
- Legal corpus management: ingest, version, index (tenant-private), search regulated documents
- Contract lifecycle: draft, redline, negotiate, execute, store, renew, expire
- Clause library: standard clauses, company-specific deviations, clause risk flagging
- E-signature: integration with KR 전자서명법-compliant services and global providers
- Matter management: case tracking, deadline management, document assembly
- Legal AI capabilities (autonomy-ceiling-gated): clause analysis, risk flagging, summarization, redline suggestion — human approval required for every AI-generated legal recommendation
- Regulatory corpus updates: KR 민법/상법 amendments, US UCC updates, EU AI Act templates
- Legal document search via `axis-search` tenant-private index (corpus classified as `tenant_searchable` or `internal_only`)

## Out-of-scope (anti-scope)
- Legal advice or legal services (Oyatie is the software substrate; tenant's lawyers provide advice)
- Consumer legal services
- SaaS workflow engine (→ `axis-saas`)
- Cross-tenant legal corpus sharing (each tenant's corpus is `internal_only` by default)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-audit-evidence` | Every contract version and signature event audit record | Per event |
| `axis-saas` | Workflow engine for contract lifecycle | Per-release |
| `axis-foundry` | Legal AI capabilities under autonomy ceiling | Wave gate |
| `axis-search` | Tenant-private corpus search index | Wave gate |
| `ops-compliance` | KR 개인정보보호법 / EU AI Act corpus compliance watch | Quarterly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | Contract and e-signature audit evidence | Quarterly |
| `gtm-customer-success` | Legal tenant health dashboards | Monthly |

## Success metrics
- **Contract version audit chain completeness:** 100%
- **E-signature event audit completeness:** 100%
- **Legal AI recommendation without human approval:** 0 (autonomy ceiling hard gate)
- **Corpus update lag after KR 민법 amendment:** ≤ 30 days
- **Tenant-private corpus search isolation violations:** 0

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for corpus schema contract changes
- Compliance: `ops-compliance` for KR 개인정보보호법 / EU AI Act regulatory incidents
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 30-min sync — corpus update queue, contract lifecycle health, legal AI capability status
- Cross-team review: quarterly corpus compliance review with `ops-compliance`

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; legal AI capability PRs require security-reviewer
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Legal AI recommendation executed without human approval | High | Autonomy ceiling hard gate; human-in-the-loop step in workflow |
| Contract version audit gap | High | 100% audit completeness gate |
| Corpus update missed after statutory amendment | Medium | Quarterly regulatory watch; versioned corpus |

## Sources scanned
PRD.md §3.1, DESIGN.md §10, ADR-0033, DOC-CATALOG.md §2.5, products/vertical-legal/PRD.md (draft).

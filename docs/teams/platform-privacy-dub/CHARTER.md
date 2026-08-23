---
doc_status: published
---

# Team: Platform — Privacy & Data Use Boundary

## Mission
This team owns the Data Use Boundary ADR — the single hardest contract in Oyatie — and the DSR cascade pipeline and per-class consent taxonomy that flows from it. It exists because PHI/PII/PCI flowing into search indexes or ad targeting represents catastrophic, unrecoverable harm, and the architecture needs a single team responsible for preventing it at the contract level. It does **not** own the audit chain infrastructure (→ `platform-audit-evidence`), the compliance matrix (→ `ops-compliance`), or per-vertical regulatory-pack implementations (→ per-vertical teams).

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting (privacy contract underpins SaaS, Search, Ads, Vertical, Foundry)
- **Surfaces:**
  - `platform-data-policy-kernel` — `DataClass`, `ConsentGradient`, `DataUseConsent`, `VerticalOverride`, `DataClassTransition`
  - `platform-data-policy-app` — consent lifecycle, DSR trigger, class-transition approval workflow
  - `platform-dsr-kernel` — `DsrRequest`, `CascadeAck`, `ErasureProof`
  - `platform-dsr-app` — DSR cascade orchestration, proof-of-erasure coordination
  - Privacy Program doc (`PRIVACY-PROGRAM.md`) — owned by this team
  - Consent management UI seam (SaaS surface; UI implementation is `axis-saas` but privacy spec is here)
- **Cross-axis contracts (DESIGN §10):**
  - `DSR / consent withdrawal cascade` (owner) — all data-touching axes must ack cascade
  - `Object Graph property tier` (co-owner with `axis-saas`) — Data Use Boundary check on tier changes
  - Data Use Boundary ADR (P0 prereq) — gates cloud, search, and ads axis substantive work
- **Catalog records:** `crates/platform-data-policy-*`, `crates/platform-dsr-*`
- **Runbooks:** `runbooks/dsr-cascade-orchestration.md`, `runbooks/consent-withdrawal-cascade.md`, `runbooks/data-class-transition-approval.md`
- **ADRs:** Data Use Boundary ADR (sole owner — P0 prereq, must reach Accepted before cloud/search/ads begin)

## In-scope work
- Data class taxonomy: `internal_only`, `tenant_searchable`, `cross_tenant_searchable_with_consent`, `analytics_aggregated`, `ad_targetable_low_sensitivity`, `ad_targetable_blocked`
- Consent gradient management: per-tenant opt-in for search vs ads, separate consent records
- Vertical-specific overrides: healthcare → `ad_targetable_blocked` for any FHIR-graph record; fintech → `ad_targetable_blocked` for account/payment instruments; KR PIPA tenants → tighter defaults
- Class transition policy: weakening requires explicit human approval; tightening (consent revocation) is automatic
- DSR cascade pipeline: orchestrate deletes across search index, ads attribution, analytics stores
- Proof-of-erasure: coordinate with `platform-audit-evidence` for chain record emission
- Privacy Program doc authorship and quarterly maintenance
- Privacy council secretariat (this team staffs the secretariat; council members are cross-functional)
- Fitness function `governance-data-use-boundary` — CI hard-fail on cross-axis data flows without consent receipt

## Out-of-scope (anti-scope)
- Audit chain infrastructure (→ `platform-audit-evidence`)
- Per-regulator compliance matrix (→ `ops-compliance`)
- Search index delete implementation (→ `axis-search` implements; this team owns the cascade contract)
- Ads attribution delete implementation (→ `axis-ads-analytics` implements; cascade contract here)
- Tenancy kernel shape (→ `platform-tenancy-identity` owns `DataUseConsent` embedding; this team owns the type definition)
- Legal review of consent language (→ `gtm-partnerships` legal counsel)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-audit-evidence` | Proof-of-erasure chain records for every DSR cascade | Per DSR event |
| `platform-tenancy-identity` | Embed `DataUseConsent` in `Tenant`; shape changes coordinated | ADR lifecycle |
| `axis-search` | Cascade ack from search index delete | Per DSR event |
| `axis-ads-analytics` | Cascade ack from ads attribution delete | Per DSR event |
| `ops-compliance` | Regulatory-change signals that may tighten class taxonomy | Monthly |
| `crew-adr-promotion` | Data Use Boundary ADR promotion to Accepted (P0 blocker) | ADR batch |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `axis-cloud` | Data Use Boundary ADR Accepted (hard gate before cloud substantive work) | Wave gate |
| `axis-search` | `DataClass` taxonomy, index-ingestion consent check | Search index lifecycle |
| `axis-ads-analytics` | Ad-targeting class definitions, `ad_targetable_blocked` enforcement | Every targeting decision |
| `axis-saas` | Object Graph property tier Data Use Boundary check | Every OG tier change |
| `council-privacy` | DSR pipeline reports, consent-withdrawal metrics | Monthly |
| All vertical teams | Vertical-specific overrides (healthcare, fintech forced `ad_targetable_blocked`) | Per vertical onboard |

## Success metrics
- **Data Use Boundary ADR status:** Accepted (P0 gate — no cloud/search/ads work starts without this)
- **Cross-axis data flows without consent receipt:** 0 (PRD §4.2 hard zero, enforced by fitness function)
- **DSR cascade completion time:** < 24 h from trigger to all-axis ack + proof-of-erasure
- **Consent withdrawal → data removal SLA:** 100% within 72 h (GDPR/PIPA baseline)
- **Data class taxonomy coverage of new Object Graph properties:** 100% at PR merge (fitness gate)
- **Privacy Program doc freshness:** updated within 30 days of any regulatory change

## Escalation path
- Internal: tech lead → team manager
- Cross-team: privacy council (`teams/council-privacy/CHARTER.md`) — escalate all class taxonomy disputes
- Legal: founder + legal counsel for new regulatory interpretation
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — DSR queue, consent-withdrawal backlog, ADR promotion progress
- Cross-team review: monthly privacy council meeting (this team runs secretariat)

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; data-class changes require privacy-council quorum
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; Data Use Boundary ADR is priority-0

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Data Use Boundary ADR stalls → blocks cloud/search/ads axes | High | Weekly progress check; founder escalation path defined |
| PHI leaks into search index via unclassified OG property | Catastrophic | `governance-data-use-boundary` CI gate hard-fails |
| DSR cascade partial completion leaves residual data | Catastrophic | Cascade ack protocol requires all-axis acknowledgment; proof-of-erasure chain record |
| Consent withdrawal not propagated to ads attribution within SLA | High | Automated cascade monitor; PagerDuty alert |

## Sources scanned
PRD.md §3.3 (anti-scope: PHI in ads), DESIGN.md §6 (Data Use Boundary — full section), §10 (DSR cascade row, Object Graph tier row), PRIVACY-PROGRAM.md, DOC-CATALOG.md §2.1 (doc.privacy_program owner = council-privacy; this team is secretariat).

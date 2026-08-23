---
doc_status: published
---

# Team: Council — Privacy

## Mission
This council holds final decision authority on the Data Use Boundary ADR, the per-class consent taxonomy, DSR cascade protocol, and any new data-class or consent-tier proposal that affects the privacy posture of the entire product. It exists because privacy decisions that seem local (e.g., "add a new OG property tier") can have catastrophic cross-axis consequences (PHI in search index, ad targeting with sensitive data), and these decisions need a cross-functional authority that spans legal, product, engineering, and compliance — not just the owning team. The council is secretariat-ed by `platform-privacy-dub`.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting privacy governance
- **Surfaces:**
  - Data Use Boundary ADR (final authority — `platform-privacy-dub` is the secretariat; council holds the vote)
  - PRIVACY-PROGRAM.md (co-owner with `platform-privacy-dub`; council ratifies major revisions)
  - Per-class consent taxonomy: every new data class or vertical override requires council approval
  - DSR cascade protocol: cascade ack specification; proof-of-erasure requirements
  - Consent-tier proposal queue: cross-functional review of any new consent or data-class proposal
  - Privacy council meeting minutes (public within org — every tenant has the right to know how their data is governed)
- **Cross-axis contracts (DESIGN §10):**
  - `DSR / consent withdrawal cascade` (authority — co-owner with `platform-privacy-dub`; council ratifies the spec)
  - `Object Graph property tier` (authority — data-class assignment for new OG tiers requires council sign-off)
- **Catalog records:** none (council is a governance body)
- **Runbooks:** `runbooks/privacy-council-data-class-review.md`, `runbooks/breach-notification-council-escalation.md`
- **ADRs:** Data Use Boundary ADR (council is the ratifying authority; `platform-privacy-dub` is the author)

## In-scope work
- Data Use Boundary ADR ratification and amendment: council votes to Accept, Amend, or Supersede
- New data-class proposals: any proposal to add a class (e.g., new `ad_targetable_*` subclass) requires council review and majority vote
- Vertical-specific override proposals: e.g., forcing a new vertical's sensitive data to `ad_targetable_blocked`; healthcare PHI override, fintech PCI override
- DSR cascade protocol amendments: changes to the cascade ack spec, proof-of-erasure requirements, or DSR SLAs
- OG property-tier data-class disputes: when an axis team disputes a class assignment, council is the arbiter
- Privacy incident review: any incident where tenant data may have exited the correct class boundary; council reviews within 24 h
- Breach notification: council is consulted on GDPR 72-h / PIPA notification decisions (alongside `ops-compliance` and legal)
- Annual privacy posture review: full review of PRIVACY-PROGRAM.md; ratify any material changes
- Cross-axis consent-flow completeness review: quarterly check that every consent gradient is covered by the Data Use Boundary ADR

## Out-of-scope (anti-scope)
- Day-to-day DSR cascade operations (→ `platform-privacy-dub`)
- Audit chain infrastructure (→ `platform-audit-evidence`)
- Compliance evidence packs (→ `ops-compliance`)
- Product roadmap decisions (→ council-architecture + founder)
- Writing product code

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-privacy-dub` | Secretariat functions, ADR drafting, DSR pipeline reports | Monthly + per event |
| `ops-compliance` | Regulatory-change signals that affect data-class taxonomy | Monthly |
| `platform-audit-evidence` | Privacy incident audit records | Per incident |
| `council-architecture` | Cross-axis architecture context for data-class decisions | Per dispute |
| Founder / legal counsel | Regulatory interpretation for novel consent scenarios | Per novel scenario |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `platform-privacy-dub` | Data Use Boundary ADR ratification; consent-tier proposals approved | ADR lifecycle |
| All vertical teams | Vertical-specific override approvals (healthcare PHI, fintech PCI, etc.) | Per vertical onboard |
| `axis-search` | Consent-gate approval for new search indexing consent tiers | Per index lifecycle change |
| `axis-ads-analytics` | New ad-targeting class proposals | Per targeting change |
| `platform-eventing-og` | OG property-tier data-class assignment disputes | Per OG schema dispute |

## Success metrics
- **Data Use Boundary ADR status:** Accepted (P0 gate for cloud/search/ads axes)
- **Data-class proposal review turnaround:** ≤ 5 business days for standard proposals; ≤ 24 h for privacy incidents
- **Cross-axis consent-flow completeness:** 100% per quarterly review
- **Privacy incident review within 24 h:** 100%
- **Annual privacy posture review:** completed on schedule
- **GDPR breach notification council consultation within 24 h of incident detection:** 100%

## Escalation path
- Internal: council chair → founder (north-star arbiter)
- Legal: external counsel for novel regulatory interpretation (GDPR Schrems, PIPA Art-28, etc.)
- Regulator: `ops-compliance` manages regulator relationships; council provides decision context

## Communication cadence
- Stand-up: no stand-up (async-first)
- Monthly: 60-min privacy council meeting (secretariat: `platform-privacy-dub`)
- Ad hoc: privacy incident review within 24 h; novel consent-tier proposal within 5 business days
- Annual: full privacy posture review

## Bandwidth + hiring
- Current FTE: Council members are drawn from `platform-privacy-dub` lead, `ops-compliance` lead, `axis-saas` lead, `axis-search` lead, `axis-ads-analytics` lead, legal counsel — not separate headcount
- Quorum: majority of council members; legal counsel advisory
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: N/A (governance body)
- PR shape: council decisions documented as ADR amendments or PRIVACY-PROGRAM.md revisions (5-section H2 template)
- ADR proposal cadence: Data Use Boundary ADR is P0 — any amendment goes to council immediately

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Data Use Boundary ADR stalls → blocks cloud/search/ads axes | High | Weekly progress check by secretariat; founder escalation if stalled > 2 weeks |
| New OG property tier approved without council sign-off | Catastrophic | Planned advisory lane `governance-data-use-boundary` records merge-readiness gaps until the CI gate exists |
| Privacy incident not reviewed within 24 h | High | PagerDuty alert to council chair on any PHI/PII class-boundary violation |

## Sources scanned
PRD.md §3.3 (anti-scope: PHI in ads), DESIGN.md §6 (Data Use Boundary), §10 (DSR cascade row, OG property tier row), PRIVACY-PROGRAM.md, DOC-CATALOG.md §2.1 (doc.privacy_program owner = council-privacy).

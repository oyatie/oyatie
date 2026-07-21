---
doc_class: Template
template_id: TPL-PRFAQ
status: pending approval
purpose: |
  Amazon Working-Backwards PRFAQ. Authored **before** any code on new-product proposals (e.g., new axis, new vertical, new pricing tier, new capability cluster). Pairs a future-dated press release with 5 internal + 5 external FAQ. Read silently for the first ~20 min of the review meeting.
lift_target: oyatie/docs/templates/prfaq-template.md
enforcing_fitness_lane: (advisory) — Founder + Council-Architecture approval gate, not a CI lane
owner_team: council-architecture + gtm-marketing
related:
  - .omc/scratch/hyperscaler-best-practices-2026-05-12.md  # §Domain 1 AWS PRFAQ
  - docs/PRD.md
  - docs/GTM-PLAN.md
  - /templates/design-doc-template.md
length_cap: 200
authoring_rules:
  - sentence_length_max_words: 30
  - no_bullets_in_press_release: true     # full prose
  - data_replaces_adjectives: true
  - no_author_attribution_on_doc: true    # per Amazon convention
---

```yaml
# Required frontmatter
---
doc_class: PRFAQ
template_id: TPL-PRFAQ
prfaq_id: PRFAQ-NNNN-<slug>
title: "<future-dated press-release headline>"
status: draft | in-review | accepted | superseded
target_launch_date: YYYY-MM-DD
masterplan_work_item_id: MPV2-<nnnn>      # /specs/masterplan.json#masterplan_v2.work_items; derived wave is in .sequencing
owner_team: <team-id>
co_owners: [council-architecture, gtm-marketing]
reviewers: [Founder, Council-Architecture, GTM-Marketing, GTM-Sales-SE]
related_adrs: [ADR-NNNN]
related_design_docs: [DD-NNNN]
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

# PRFAQ-NNNN: <future-dated press-release headline>

---

## Press release (future-dated; full prose; no bullets)

**FOR IMMEDIATE RELEASE — Seoul, KR — <target launch date>**

### Headline
<One-sentence headline. ≤ 20 words.>

### Subhead
<One-sentence subhead. ≤ 30 words.>

### Lead paragraph
<3-5 sentences. Who, what, when, where, why. Each sentence ≤ 30 words. Replace adjectives with data. Name the customer pain in concrete terms.>

### Problem paragraph
<3-5 sentences. The customer pain *today*, with specifics: time wasted, dollars lost, regulator risk, integration tax. Reference at least one named customer archetype.>

### Solution paragraph
<3-5 sentences. What Oyatie ships. The non-leakage thesis: single tenancy, single identity, single audit chain, single agent runtime. Name the specific axis surfaces involved.>

### Customer quote
> "<Future-dated customer quote, ≤ 40 words, attributed to a plausible role at a named archetype, e.g., Director of Compliance at a Korean tier-1 bank.>"

### Internal leader quote
> "<Future-dated quote from a named oyatie role (e.g., axis-foundry lead), ≤ 40 words, that names the architectural premise.>"

### How to get started
<2-3 sentences. URL, sign-up flow, prerequisite. Concrete.>

---

## Internal FAQ (5 questions; for the review meeting)

### IF1. Why are we building this?
<Answer: customer pain + strategic premise (per `docs/CONSTITUTION.md §Mission`). Cite the specific PRD §1 axis claim this serves.>

### IF2. Why now? Why not later?
<Answer: regulatory window, competitive window, capacity readiness, dependency completion (per `/specs/masterplan.json#masterplan_v2.dependency_edges` and `.sequencing`).>

### IF3. What are the risks and how are we mitigating?
<Answer: top-3 risks with mitigation owner + tracking row in `docs/RISK-REGISTER.md`.>

### IF4. What does success look like at 12 months?
<Answer: measurable outcomes. Tenant count, capability count, audit-chain coverage %, retention %, regulator-pack adoption count. Maps to `docs/PRD.md §4 success metrics`.>

### IF5. What are we explicitly NOT building?
<Answer: anti-scope per `docs/PRD.md §1 non-goals`. Reference future milestones that will own the excluded items.>

---

## External FAQ (5 questions; ships with launch)

### EF1. What is this product / capability?
<Customer-facing answer. 3-5 sentences. No internal jargon.>

### EF2. Who is it for?
<Customer-facing answer. Name 2-3 user personas with their job-to-be-done.>

### EF3. How does it integrate with my existing stack?
<Customer-facing answer. Name the integration surfaces (API, SDK, OAuth, capability registry). Reference `docs.oyatie.com/<surface>`.>

### EF4. How does pricing work?
<Customer-facing answer. Pricing tier, included quota, overage rate. Reference `docs/GTM-PLAN.md`.>

### EF5. What about my data — regulatory, residency, audit?
<Customer-facing answer. Tenancy boundary, residency region, audit-chain emission, regulatory packs supported (`oya-pack-kr.PIPA`, `global.GDPR`, etc.). Reference `docs/PRIVACY-PROGRAM.md` and `docs/COMPLIANCE-MATRIX.md`.>

---

## Review process

1. Reviewers read silently for the first 20 min of the meeting (per AWS convention).
2. Discussion follows; the author **does NOT** present.
3. Verdict: `proceed-to-design-doc` | `revise` | `kill`.
4. On `proceed-to-design-doc`: author lifts to a Google-style design doc (`/templates/design-doc-template.md`) before any IP claim.
5. On `kill`: archive at `/specs/killed/PRFAQ-NNNN-<slug>.md` with the kill reason recorded.

## Sources

- AWS — Working Backwards / PRFAQ process (`workingbackwards.com`).
- `.omc/scratch/hyperscaler-best-practices-2026-05-12.md §Domain 1 AWS`.
- `docs/PRD.md`, `docs/GTM-PLAN.md`, `/specs/masterplan.json#masterplan_v2`.

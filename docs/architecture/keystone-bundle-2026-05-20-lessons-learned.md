---
doc_class: Architecture deep-dive / walkthrough
status: Final
date: 2026-05-20
authority_tier: 3
shape: retrospective
purpose: |
  One-shot retrospective on the 2026-05-20 keystone-bundle multispectrum
  review process. Captures the process lessons for future reviewers and
  orchestrators. The content of this document MUST feed the v2.5.0 doctrine
  bump. It is NOT the durable standard — that is
  docs/standards/multispectrum-review-v2.4.0-cadence.md.
companion_docs:
  - docs/standards/multispectrum-review-v2.4.0-cadence.md (durable standard derived from these lessons)
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md (authoritative synthesis — do NOT edit)
related_adrs:
  - ADR-0242 through ADR-0255 (keystone bundle)
  - ADR-0263, ADR-0272, ADR-0273, ADR-0276, ADR-0280, ADR-0284, ADR-0292 (remediation ADRs)
related_memories:
  - feedback_multispectrum_review_v22
  - feedback_multispectrum_adherence_facets
  - feedback_consensus_debate_spectrum_lens_subagents
  - feedback_codex_bulk_resolve_antipattern
do_not_edit: keystone-bundle-2026-05-20-synthesis.md
do_not_edit_reason: authoritative synthesis is frozen evidence
---

# Keystone Bundle 2026-05-20 — Process Lessons Learned

**Scope:** Process retrospective only. This document captures what went wrong
and what went right in the review *process* for the 2026-05-20 keystone
bundle. The substantive architectural decisions are recorded in the synthesis
document. This document is the input for the v2.5.0 doctrine bump.

**Status of synthesis:** FINAL. Do not edit
`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` — it is frozen
evidence. Refer to it for the GO/NO-GO verdict, the 15 promotion gates, and
the BYOK clarification.

---

## §1 Overview

The keystone-bundle-2026-05-20 review was the first application of
multispectrum-review v2.4.0. It reviewed 28+ documents (14 foundational ADRs,
7 remediation ADRs, 4 specs, 4 PRDs, 2 user-story compendia, 4 standards)
across 21 facets (F1-F11, F13, M1, M2, A1-A7). The review produced high-quality
evidence but surfaced 7 distinct process failures that cost time, created
ambiguous evidence state, and in one case (the BYOK scope conflict) only
emerged after synthesis was already drafted.

This document catalogues each failure, its root cause, the mitigation adopted
in v2.4.0, and any open proposal for v2.5.0.

The review also produced 3 genuine process strengths that should be preserved.
Those are documented in §9.

---

## §2 Lesson 1 — Rate-Limit Saturation from Parallel Full-Platform Dispatch

### §2.1 What Happened

All 21 facet agents were dispatched simultaneously. Within minutes, 7 agents
(F3, F5, A2, A4, A6, F13, A1) hit either quota limits or context-window
errors. Two of these agents (F3 Readability and A6 Schema) produced empty
or truncated r1.json files before failing. The orchestrator had to identify
which files were complete versus failed, re-dispatch 7 agents, and reconcile
the partial state.

The partial-state window lasted approximately 90 minutes. During that window,
it was not clear from the `evidence/debate/` directory which facets had
completed and which had failed — because some agents had written partial files
before erroring.

### §2.2 Root Cause

Simultaneous dispatch of 21 agents exceeded the per-session quota for parallel
tool calls. The individual agent context windows were not a constraint for most
facets, but F5 Security (see §3) was independently a context issue. The
parallel-dispatch pattern was inherited from smaller reviews (5-7 facets) where
it had worked without incident; the quadratic scaling to 21 facets was not
anticipated.

The partial-file problem was a secondary consequence: agents that started
writing before failing left syntactically invalid JSON that the orchestrator
had to manually identify.

### §2.3 Mitigation Adopted (v2.4.0)

`docs/standards/multispectrum-review-v2.4.0-cadence.md §3` codifies the wave
dispatch protocol: maximum 8 facets per wave, next wave starts only when
≥50% of the current wave has written its r1.json. A dispatch manifest file
tracks in-flight state.

### §2.4 v2.5.0 Proposal

Add a dispatch manifest schema to `/specs/multispectrum-review.json` so the
dispatch manifest is machine-readable and the CI lane can verify it was used.
Add an orchestrator health-check: before dispatch, query the current session's
remaining quota and reduce wave size dynamically if quota is low.

---

## §3 Lesson 2 — F5 Security Re-Dispatched Three Times

### §3.1 What Happened

The F5 Security facet agent was dispatched, failed at approximately 40% completion
(context window exhausted), re-dispatched and failed again at approximately
75% completion, and completed only on the third dispatch. Total elapsed time
for F5 was approximately 4.5× the median facet time. The final F5 r1.json
(at `evidence/debate/keystone-bundle-2026-05-20-F5-security-r1.json`) is
complete and high-quality with 7 threat-model gaps, 8 cryptographic weaknesses,
and 24 recommendations across 10 ADRs.

### §3.2 Root Cause

Two independent root causes compounded:

**a) Adversarial threat-model prompt complexity.** The F5 facet prompt
instructs the agent to model three adversary classes (malicious tenant,
compromised admin, nation-state) against every ADR in scope. With 10 ADRs in
the keystone bundle's security-relevant subset, this produced approximately
30 threat-model scenarios in a single session. The prompt also requires
cryptographic weakness analysis, attack surface enumeration, and per-ADR
per-finding recommendations — all in a single context window.

**b) Corpus size.** The 10 ADRs reviewed by F5 averaged approximately 1500
lines each, producing approximately 15,000 lines of source material before
any analysis was attempted.

Together, these factors caused the F5 context window to fill before the agent
had written its complete verdict.

### §3.3 Mitigation for Future Large-Bundle Reviews

When a review corpus exceeds 10 ADRs and F5 Security is required:

1. **Split F5 into sub-scoped segments** if the corpus has natural partitions.
   For a 14-ADR bundle, split into F5-a (ADRs 0-6) and F5-b (ADRs 7-13).
   The synthesizer treats them as one logical F5 verdict.
2. **Place F5 in Wave 1** of the dispatch schedule so it starts earliest
   and its long tail does not block later waves. (This was done in v2.4.0.)
3. **Pre-write the finding skeleton** in the F5 prompt: supply an empty
   JSON skeleton with the per-ADR sections pre-populated. The agent fills
   in findings rather than writing the structure from scratch. This reduces
   the cold-start overhead by approximately 20-30%.

### §3.4 v2.5.0 Proposal

Add explicit `max_adrs_per_f5_dispatch: 8` guidance to the F5 facet definition
in `/specs/multispectrum-review.json`. When the corpus exceeds the limit,
the orchestrator MUST automatically split F5 into sub-segments.

---

## §4 Lesson 3 — M1 NO-GO-AS-BUNDLE Resolution Pattern

### §4.1 What Happened

M1 (Challenge-assumption) returned a `NO-GO-AS-BUNDLE` verdict, recommending
a 3-wave split of the 14 ADRs. The M1 findings were substantively well-argued:
9 load-bearing premises were identified, counter-patterns from AWS/GCP/Azure
were cited, and 9 alternative paths were documented with trade-offs.

The initial synthesis draft did not have a clear rule for how to weight M1
against the 20-of-21 F+A-family majority that was compatible with a bundled
landing. There was a period of ambiguity about whether the synthesis was
required to follow M1 or could override it.

### §4.2 Resolution

The synthesis resolved the tension by separating *textual landing* from
*operational enforcement landing*:

- The bundle merges in `Proposed` state with no CI lanes promoted to BLOCKER.
- Each ADR's CI lane promotes from advisory to BLOCKER only when its
  promotion gates are closed (per synthesis §5.1 through §5.15).
- M1's gating concerns (5 of its 6 findings) were converted into named
  promotion gates rather than merge blockers.
- M1-KB-F6 (BYOK-everywhere over-constrains B2C) was closed by user
  clarification (§4 of the synthesis).

This produced a GO-WITH-GATES verdict that honoured M1's spirit (no
enforcement leak) while overriding its form (one bundled landing).

### §4.3 Rule Codified in v2.4.0

`docs/standards/multispectrum-review-v2.4.0-cadence.md §5.4` now explicitly
states: M1 NO-GO-AS-BUNDLE is evidence in the synthesis, not an automatic
veto. The synthesizer MAY override the form if the F+A majority is
APPROVE-WITH-CONDITIONS and M1's concerns are addressable as promotion gates.

### §4.4 v2.5.0 Proposal

Add a worked example of the NO-GO-AS-BUNDLE override to the spec schema
at `/specs/multispectrum-review.json` with this review as the cited instance.
Consider adding an explicit `m1_disposition` key to the synthesis JSON schema
to force documenting the adjudication decision.

---

## §5 Lesson 4 — BYOK Scope Conflict Surfaced Post-Synthesis

### §5.1 What Happened

The synthesis draft treated "BYOK-everywhere" as a single concern. After the
synthesis was substantially drafted, user feedback identified that the document
conflated two disjoint BYOK concerns:

- **provider-BYOK:** Bring-your-own LLM/AI provider API credentials (governed
  by ADR-0255 §D-4). Opt-in; oyatie provides default credentials for B2C.
- **encryption-BYOK:** Bring-your-own KMS root / HSM partition for
  at-rest data encryption (governed by ADR-0251 §D-10). Tracked by
  `byok_enabled` on the `tenants` table.

The conflation had persisted through all 21 facet reviews without being
surfaced. The M1 facet challenged the BYOK doctrine (CA-6 in the M1 file)
as hostile to B2C UX but framed it as a single doctrine, not as a
two-concern conflation. No facet had the scope-disambiguation lens to catch
the conflation itself.

### §5.2 Root Cause

There is no facet in v2.4.0 whose primary lens is "terminology and scope
disambiguation." The closest is F3 (Readability), but F3 focuses on clarity
for an intern reader, not on whether a concept's scope boundary is correctly
drawn. The closest to scope analysis is F4 (Architecture), but F4 focuses on
layer boundaries and dependency direction, not terminology.

The BYOK conflation was a semantic scope issue — the same term was used for
two disjoint concepts — and no facet was positioned to catch it as a
first-class finding.

### §5.3 Resolution in Bundle

The synthesis §4 defines the authoritative resolution: provider-BYOK and
encryption-BYOK are declared disjoint. The following text edits were applied
before merge: ADR-0255 §D-4 scoped to LLM/provider credentials only;
ADR-0244 §D-3 DDL adds `provider_credential_mode` column; compliance-pack
schema adds `provider_byok_required` and `encryption_byok_required` as
disjoint flags; `feedback_byok_everywhere_credentials` memory rewritten to
reflect the split.

### §5.4 v2.5.0 Proposal — F14 Terminology/Scope-Disambiguation

Propose adding **F14 Terminology/Scope-Disambiguation** as an optional facet
in v2.5.0:

- **Lens:** Are all terms in the corpus used with consistent, non-conflated
  scope? Does the corpus use the same word for multiple distinct concepts?
  Are concept boundaries drawn at the right granularity?
- **Trigger:** CC-1 changes that introduce 5+ new canonical terms; any
  change where a single term appears in multiple distinct semantic contexts
  within the same corpus.
- **Recommended subagent_type:** architect with explicit glossary-construction
  methodology.

This facet would have caught the provider-BYOK / encryption-BYOK conflation
during the initial dispatch rather than post-synthesis.

---

## §6 Lesson 5 — A1/A3 Overlap on Layer-Enum Fixes

### §6.1 What Happened

Both A1 (Naming) and A3 (Structure) produced findings targeting the same
defect: ADR-0263 §D-6 invented 4 layer values (`tool`, `mock`, `fixture`,
`bench`) that are not in the ADR-0105 canonical 13-layer enum, while dropping
4 canonical values (`infrastructure`, `cli`, `grpc`, `graphql`).

- A1's finding (BNF-V01): framed as a naming policy violation — the wrong
  values are cited in the "per ADR-0105" enumeration.
- A3's finding: framed as a structural placement issue — test-layer concerns
  mixed with production-layer schema placement.

Both findings pointed to the same lines in ADR-0263 §D-6 and both recommended
reconciling with ADR-0105. The synthesizer received two separate findings that
required the same fix, creating coordination overhead: which gate should track
the fix — the A1 gate or the A3 gate?

### §6.2 Root Cause

There is no pre-dispatch overlap-detection step. Facet subagents are dispatched
independently and independently discover the same defect from different lens
angles. When the defect is structural-naming (a class of defect that both A1
and A3 legitimately cover), the overlap is expected and healthy — both lenses
provide genuine signal. But it requires explicit synthesizer logic to merge
overlapping findings into one gate rather than two.

### §6.3 Resolution in Bundle

The synthesis merged the two findings into gate §5.10 (A1 Naming Fixes) and
§5.11 (A3 Structure Fixes). Both gates reference the same ADR-0263 §D-6
defect. The fix is identical; the two gates share the same concrete action.

### §6.4 v2.5.0 Proposal — Pre-Dispatch Overlap-Detection Pass

Before dispatching A-family facets, run a lightweight overlap-detection pass:

1. Enumerate all document sections that are likely targets of both A1 and A3
   (schema fields that are also crate names; enum values that are also layer
   positions; any identifier that appears in both structural and naming
   contexts).
2. For each overlap, append to the A1 and A3 prompts: "Note: the following
   sections are also in scope for [A1/A3] — if you find a defect, indicate
   whether it is primarily a naming defect or a structural defect, to assist
   the synthesizer in gate assignment."

This does not eliminate the overlap (both facets may legitimately find the
same defect) but reduces the synthesizer's coordination overhead.

---

## §7 Lesson 6 — Synthesis Document Itself Was Not Facet-Reviewed

### §7.1 What Happened

The synthesis document (`docs/architecture/keystone-bundle-2026-05-20-synthesis.md`)
was produced by the synthesizer and accepted as final without any facet review.
In retrospect, the synthesis document is itself a canonical doc that:

- Makes binding GO/NO-GO decisions with long-term consequences.
- Contains M1 adjudication reasoning that establishes a precedent (the
  NO-GO-AS-BUNDLE override pattern).
- Introduces the BYOK scope resolution that modifies the authoritative memory
  file and three ADRs.
- Is cross-referenced from every keystone ADR's §G (review evidence).

None of the 21 facets reviewed the synthesis document itself. The synthesis
is, by design, produced after facet review completes — but its own content
should itself meet the review bar for a document of its authority tier.

### §7.2 v2.5.0 Proposal — Synthesis Self-Review Step

In v2.5.0, after the synthesis document is written, dispatch a **synthesis
self-review** as a final step before the synthesis is declared final:

- **Who:** The M2 (meta-review) facet subagent, or a dedicated M3 synthesis-
  review subagent if the resource budget allows.
- **Lens:** Does the synthesis document accurately represent all facet
  verdicts? Are NO-GO overrides documented with sufficient evidence? Are all
  BLOCKER findings mapped to named gates? Is the synthesis itself
  intern-buildable per `documentation-rigor.md`?
- **Output:** A synthesis-review note appended to the synthesis document's
  `## §9 Provenance` section, signed with its own reviewer_id.
- **Gate:** If the synthesis-review finds any BLOCKER findings in the
  synthesis itself, the merge is blocked until the synthesis is corrected.

This adds at most one additional dispatch per bundle review, and only for
CC-1 changes. The overhead is bounded and justified by the synthesis
document's authority tier.

---

## §8 Lesson 7 — Change Class Not Explicitly Declared at Dispatch Time

### §8.1 What Happened

The M2 (meta-review) facet finding M2-3 flagged that neither the audit report
nor the deep-dive explicitly declared the bundle's change class. By inspection,
the bundle was clearly CC-1 (kernel public API), but the absence of a declared
`change_class` meant:

- The facet-coverage assessment was not formally grounded in the rigor matrix.
- The M2 facet had to independently determine the change class to assess
  whether the right facets were dispatched.
- Downstream lane enforcement cannot verify the facets dispatched matched the
  change class without the orchestrator's inference.

### §8.2 Root Cause

The review was initiated from the bundle audit report and deep-dive, neither
of which carried a `change_class` field. The orchestrator inferred CC-1 from
the corpus but did not record the inference.

### §8.3 Mitigation Adopted (v2.4.0)

`docs/standards/multispectrum-review-v2.4.0-cadence.md §1.3` requires explicit
`change_class` declaration in the review manifest before dispatch. The synthesis
document for this bundle includes the CC-1 declaration with trigger evidence
(new public spec contracts + 14 keystone ADRs that supersede or amend prior ADRs).

### §8.4 v2.5.0 Proposal

Add a `change_class_declaration` required key to the dispatch manifest schema
in `/specs/multispectrum-review.json`. The orchestrator MUST write this field
before dispatching any facets. If the field is absent, the lane refuses to
count any evidence files toward the review.

---

## §9 What Went Well — Strengths to Preserve

Despite the 7 process failures above, the 2026-05-20 review produced genuinely
strong outputs. These strengths MUST be preserved in future reviews and in the
v2.5.0 doctrine.

### §9.1 Pre-Debate Package Quality

Both the audit report (`docs/architecture/keystone-bundle-audit-report.md`)
and the deep-dive (`docs/architecture/keystone-bundle-idea-refine-deep-dive.md`)
were honest, concrete, and adversarial. The audit:

- Explicitly downgraded cross-reference coherence to FAIL with concrete
  defect counts (~100-200 filenames needing normalisation).
- Reported 8 gaps with no rounding-up.
- Accepted post-admission remediations conditionally rather than unconditionally.

The deep-dive used the phrase "This is not a polite review" — and lived up to
it with 25+ F-MISSED-* findings, 10+ F-ANTI-* anti-pattern observations, and
concrete migration-pain scores (1-10). The adversarial voice is what bias-
collapse prevention exists to produce; the pre-debate package delivered it.

The lesson: invest in the pre-debate package. A high-quality input package
makes every facet's job easier and produces more focused, specific findings.

### §9.2 Learned-From-History Framing

Every keystone ADR carried a `keystone_position N-of-14` field and the framing:
"Bundled with the 14-ADR foundational keystone set... partial acceptance is
rejected because the doctrines are mutually-reinforcing and produced together
to avoid the drift pattern that produced ADR-0220 → ADR-0239 amendment within
twelve days."

The M1 facet challenged whether this framing was a non-sequitur (it was: see
M1-KB-F1). But the intent — to structurally prevent the specific drift pattern
that had occurred — is the right instinct. The failure was in the mechanism
chosen (all-or-nothing bundling), not in the goal (coherent cross-referenced
landing). Future reviews should preserve the learned-from-history framing while
adopting the gate-based promotion mechanism as the coherence mechanism instead
of the all-or-nothing bundling.

### §9.3 Positive Finding Discipline

Multiple facets included `POSITIVE` severity findings — documented strengths
to preserve rather than only problems. F5 included 14 positive observations.
A1 included 7 passing-observation items. M2 included 3 positive findings.

This discipline prevented the review from producing adversarial noise where
reviewers only report what is wrong. The synthesis could then accurately report:
"20-of-21 facets compatible with bundled landing in `Proposed`" — a concrete
signal that the bundle's substance was production-grade despite the process
failures.

The lesson: positive findings are load-bearing evidence in the synthesis.
Future facet prompts SHOULD explicitly ask for positive findings alongside
critical findings.

---

## §10 Proposed v2.5.0 Doctrine Additions

This section summarises the v2.5.0 proposals from §§2-8 in one place, for
the author of the v2.5.0 bump to consume.

| Proposal ID | Source | Description | Priority |
|---|---|---|---|
| P1 | §2.4 | Dispatch manifest schema in `/specs/multispectrum-review.json`; dynamic wave-size based on available quota | MEDIUM |
| P2 | §3.4 | `max_adrs_per_f5_dispatch: 8` in F5 facet spec; automatic F5 sub-segmentation | HIGH |
| P3 | §4.4 | `m1_disposition` required key in synthesis JSON schema; worked-example in spec | MEDIUM |
| P4 | §5.4 | F14 Terminology/Scope-Disambiguation facet (optional; triggered by CC-1 with 5+ new terms) | HIGH |
| P5 | §6.4 | Pre-dispatch overlap-detection pass for A-family facets | LOW |
| P6 | §7.2 | Synthesis self-review step (M2 or M3 subagent reviews synthesis doc before final) | HIGH |
| P7 | §8.4 | `change_class_declaration` required key in dispatch manifest schema; lane refuses without it | HIGH |

The HIGH priority proposals (P2, P4, P6, P7) SHOULD be included in v2.5.0.
The MEDIUM proposals (P1, P3) MAY be included. P5 is LOW priority and may
be deferred to v2.6.0 if the v2.5.0 change is already large.

### §10.1 Proposed v2.5.0 New Facets

Based on the 2026-05-20 review, v2.5.0 SHOULD consider two new facets:

| Proposed facet | Family | Trigger | Rationale |
|---|---|---|---|
| F14 Terminology/Scope-Disambiguation | F-critique | CC-1 with ≥5 new canonical terms OR any corpus with a term used in multiple semantic contexts | BYOK conflation (§5) would have been caught at dispatch time |
| M3 Synthesis-review | M-meta | Every CC-1 synthesis document | Synthesis document itself was not reviewed (§7) |

These proposals are NOT binding on the v2.5.0 author. They are the concrete
lessons from this review, surfaced for consideration.

---

## §11 Impact on Active Work

The lessons in this document have the following immediate impacts on
in-flight work:

1. **All orchestrators dispatching CC-1 reviews MUST apply the wave dispatch
   protocol (§2.3)** immediately, even before 2026-07-15 BLOCKER arm date.
   This is not a future obligation — the 2026-05-20 review demonstrated the
   cost of not doing it.

2. **F5 Security for large bundles MUST be split if corpus > 10 ADRs.**
   The next bundle review (if it covers more than 10 ADRs) MUST use the
   F5-a / F5-b segmentation described in §3.3.

3. **The BYOK scope clarification is authoritative.** Any doc that uses the
   term "BYOK" MUST specify whether it refers to provider-BYOK (ADR-0255
   §D-4) or encryption-BYOK (ADR-0251 §D-10). Generic "BYOK" is now an
   ambiguous term and SHOULD be replaced with one of the two specific terms
   at the next edit of any doc that uses it.

4. **The synthesis self-review gap is not yet closed.** The
   keystone-bundle-2026-05-20-synthesis.md was not facet-reviewed. This is
   accepted as a known gap for this bundle. Future synthesis documents MUST
   apply the self-review step once v2.5.0 codifies it.

---

## §12 References

- `docs/standards/multispectrum-review-v2.4.0-cadence.md` — durable standard derived from these lessons
- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` — authoritative synthesis (frozen)
- `evidence/debate/keystone-bundle-2026-05-20-M2-meta-review-r1.json` — M2 source verdict
- `evidence/debate/keystone-bundle-2026-05-20-F5-security-r1.json` — F5 source verdict (3-dispatch)
- `evidence/debate/keystone-bundle-2026-05-20-M1-challenge-assumption-r1.json` — M1 NO-GO-AS-BUNDLE
- `evidence/debate/keystone-bundle-2026-05-20-A1-naming-r1.json` — A1/A3 overlap example
- `feedback_multispectrum_review_v22` — v2.2.0 doctrine baseline
- `feedback_multispectrum_adherence_facets` — A-family facet trigger conditions
- `feedback_byok_everywhere_credentials` — updated BYOK scope resolution
- `feedback_codex_bulk_resolve_antipattern` — P2 findings are not ignorable
- `docs/architecture/keystone-bundle-audit-report.md` — pre-debate evidence package
- `docs/architecture/keystone-bundle-idea-refine-deep-dive.md` — pre-debate adversarial package

---

## §13 Change Log

| Date | Version | Author | Change |
|---|---|---|---|
| 2026-05-20 | 1.0 | M2 process-remediation subagent | Initial publication. Derived from M2 verdict (M2-1 through M2-12), F5 dispatch failures, M1 adjudication, BYOK post-synthesis surfacing, A1/A3 overlap, and synthesis self-review gap. |

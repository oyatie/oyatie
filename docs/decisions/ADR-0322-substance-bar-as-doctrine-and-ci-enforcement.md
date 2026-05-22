---
id: ADR-0322
status: Proposed
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-engineering
  - council-quality
  - council-documentation
  - axis-policy-engine
  - axis-ontology
  - axis-workflow-engine
  - axis-foundry
  - ops-compliance
  - ops-sre-reliability
supersedes: []
amends:
  - ADR-0063-documentation-coverage-enforcement.md (promotes substance-bar from policy to canonical doctrine + adds CI-enforceable substance density check)
  - ADR-0091-multispectrum-review-doctrine.md (elevates the substance facet from advisory to BLOCKER class)
  - ADR-0132-product-suite-and-bundle-dissolution.md (declares that suite-style table-of-contents documents fail the substance bar)
  - ADR-0245-substrate-vs-product-layering.md (substrate documentation must satisfy substance bar before any product layer references it)
superseded_by: []
related:
  - ADR-0063
  - ADR-0091
  - ADR-0105
  - ADR-0132
  - ADR-0145
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0263
  - ADR-0316
  - ADR-0321
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/markdown-retirement-policy.json
  - /specs/documentation-substance-bar-schema.json
  - /specs/microservices/manifest-schema.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/multispectrum-review-v2.4.0.md
  - docs/feedback/feedback_docs_substance_not_scaffold_2026_05_20.md
  - docs/AGENTS.md
inbound_citations:
  - docs/feedback/feedback_docs_substance_not_scaffold_2026_05_20.md
  - docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
purpose: >
  Promote the substance bar from informal review heuristic to BLOCKER-class
  doctrine. Every documentation artifact authored within the oyatie repository
  (ADRs, journey docs, IP slices, microservice READMEs, product PRDs, capability
  tier matrices) must demonstrate bespoke substance per the criteria in
  documentation-rigor §1.1 and the substance-bar schema, and must pass the
  oya-governance-substance-bar + oya-governance-no-template-stamping CI lanes
  before any reviewer-agent verdict is solicited. Template stamping, scaffold
  duplication, lambda-wrapped pseudo-content, and table-of-contents documents
  with no bespoke per-section content are categorically rejected.
enforcement_status: blocker-day-one
enforced_by:
  - oya-governance-substance-bar
  - oya-governance-no-template-stamping
  - oya-governance-doc-density-floor
  - oya-governance-doc-bespoke-ratio
  - oya-governance-substance-bar-spectrum-binding
decision_owner: council-quality
---

## Status

Proposed (2026-05-20). Active for all documentation authored on or after the
publication date of this ADR. Pre-existing artifacts written before 2026-05-15
are grandfathered until their next material edit; the next edit that touches
their substance must lift them across the bar described in section D-3 of this
ADR or split the artifact into a stub plus a successor satisfying the bar.

## Context

### Named pressure

The 2026-05-19/2026-05-20 remediation audits revealed that a non-trivial
fraction of the post-keystone ADR cluster (0297-0321), the codex-erp IP slices,
the codex-b2b-leader IP slices, the journey artifact wave, and several
microservice README seed files had been produced through one of three failure
modes that this ADR names and bans:

1. **Template stamping** — the author copied the documentation-rigor frontmatter
   and section headings verbatim, populated 10-30% of the section bodies with
   bespoke prose, and filled the remainder with generic placeholder sentences
   that could appear in any artifact in the repo. The artifact passed the
   structural CI lanes (lean-a4-structure, lean-a5-doc-coverage) because the
   shape was correct, but a human reading it would extract approximately zero
   novel information. The feedback note `feedback_docs_substance_not_scaffold_2026_05_20.md`
   classifies this as the dominant failure mode for the 2026-05-19 batch.
2. **Lambda-wrap pseudo-content** — observed in codex-erp-ip-w2 and a portion of
   the codex-b2b-leader-w3 IP slices: the author wrote a shell loop or jq
   expression that interpolated artifact-name strings into a constant body
   template, producing N artifacts with identical bodies modulo a single
   substitution. The feedback note `feedback_go_with_original_ambition_2026_05_20.md`
   captures the directive to never script substantive content; this ADR
   converts that directive into doctrine.
3. **Table-of-contents documents** — top-level ADRs and journey docs that
   enumerate the names of downstream artifacts (10-50 child IDs) but provide no
   bespoke analysis at the parent level. The artifact technically discusses a
   real problem but only at the granularity of a hyperlink list. Reviewer
   agents repeatedly flagged these as low-information artifacts even though no
   single explicit policy rule existed to BLOCK them.

### Named constraints

- **C-1 Inheritance from documentation-rigor**: section 1.1 of
  `docs/standards/documentation-rigor.md` already defines the substance bar in
  prose. This ADR does not rewrite the prose; it elevates it to BLOCKER class
  and names the CI mechanism that enforces it.
- **C-2 Multispectrum compatibility**: ADR-0091 (multispectrum-review doctrine)
  defines a substance facet (F4-substance). Prior to this ADR, F4-substance was
  advisory. This ADR upgrades F4-substance to BLOCKER class so that a single
  substance-bar failure prevents merge regardless of any other facet verdict.
- **C-3 Closed-enum cap relaxation**: per `feedback_multispectrum_adherence_facets.md`,
  the multispectrum facet enum was already relaxed for the A-family. The
  substance-bar facet continues to live in the F-family (F4-substance) and is
  not a new facet; this ADR only changes its severity class.
- **C-4 Authority-chain compatibility**: per ADR-0145 inter-microservice
  reform, governance crates must be owned by `oya-governance-*` lane and must
  emit structured violation events on the audit chain per ADR-0263.
- **C-5 Substrate-product layering**: per ADR-0245, substrate documentation
  must clear all blockers before any product layer references it. The substance
  bar applies first to substrate ADRs and substrate microservice docs, then
  cascades to product layer artifacts.
- **C-6 Tenancy carry-through**: per ADR-0244, every governance event carries
  tenant context. Substance-bar violation events therefore carry
  `tenant_id=oyatie.governance` (the platform tenant, per ADR-0242) so that
  audit consumers can filter substrate violations from product-layer ones.

### Named prior incidents

- **Incident I-1 (2026-05-12)**: PR-118 attempted to land 14 new microservice
  README files generated from a shell-loop template; reviewer-agent flagged
  template-stamping but the merge queue still admitted the PR because no
  BLOCKER lane existed. Remediated by manual rewrite over six subsequent PRs.
  Documented in `docs/postmortems/postmortem-readme-template-stamping-2026-05-12.md`.
- **Incident I-2 (2026-05-17)**: codex-erp-ip-w2 produced 18 IP slice files
  with bodies that differed only in their first heading line. Caught by
  human review four days after merge; required wave-2-remediation effort
  (3 agents × 2 days) to repair.
- **Incident I-3 (2026-05-18)**: Three of the W3-G batch ADRs (0319, 0320, 0321
  initial drafts) clocked under 200 lines despite the documentation-rigor §3.2
  prescription of ≥800 lines for Authority-Tier-1 ADRs. Reauthored to the bar
  before merge; the experience surfaced the need for an automated density floor
  rather than reliance on manual review.
- **Incident I-4 (2026-05-19)**: journey-artifact batch JA-2026-05-19-A
  shipped 47 journey files whose Cedar-policy-hooks sections were copy-paste
  identical (same principal, same action, same resource pattern). Substance bar
  in spirit demanded per-journey policy fragments; lack of automation let
  duplicates merge.

### Scope

This ADR applies to **all documentation artifacts under version control in this
repository** that are classified as Authority-Tier-1 or Authority-Tier-2, plus
all journey, IP-slice, microservice-README, PRD, RFC, and specification
artifacts regardless of authority tier. Code comments, commit messages, and
inline doc-strings are out of scope and remain governed by existing per-lane
review checks.

## Decision Summary

The substance bar is hereby canonical doctrine. Every in-scope artifact must
clear all of the following before its containing pull request becomes
eligible for the merge queue:

- **S-1 Bespoke ratio** ≥0.65 — at least 65% of token-equivalent content in
  each section body must be unique to the artifact (not present in any other
  artifact in the repository at edit-distance ≤0.15 normalized).
- **S-2 Density floor** ≥800 lines for Authority-Tier-1, ≥500 for Tier-2,
  ≥250 for Tier-3, ≥120 for Tier-4 (per documentation-rigor §3.2 schedule).
- **S-3 Detailed-mechanics expansion** — every ADR ships at least 10 named D-N
  sub-sections under "Detailed Mechanics", each ≥40 lines, each addressing a
  bespoke mechanic of the decision (no generic "future work" or "implementation
  details" placeholders).
- **S-4 Cedar policy hooks present** — every ADR ships a named Cedar fragment
  with explicit principal, action, resource, condition; merged Cedar fragment
  set must be parsable by the Cedar policy linter and must not duplicate any
  existing fragment by structural equivalence.
- **S-5 Audit event class additions documented** — every ADR enumerates at
  least one audit event class it adds to the ADR-0263 registry or explicitly
  declares "no new audit classes" with justification.
- **S-6 No template stamping** — the artifact body, when stripped of common
  headings and frontmatter keys, must score ≥0.45 on the substance-bar bespoke
  metric versus the corpus of artifacts created within the same calendar week.
- **S-7 Named cross-references** ≥12 — every Tier-1 ADR cites at least 12
  named ADRs, named specs, named microservices, or named journey artifacts in
  bespoke context (not just in the related field of the frontmatter).
- **S-8 Multispectrum facet binding** — the artifact carries a
  `substance_bar_facet_binding` frontmatter field that names the F4-substance
  reviewer agent that signed off; a missing or `null` value fails the lane.

Each of S-1..S-8 corresponds to a CI check authored within the
`oya-governance-substance-bar` crate plus the supporting
`oya-governance-no-template-stamping`, `oya-governance-doc-density-floor`,
`oya-governance-doc-bespoke-ratio`, and `oya-governance-substance-bar-spectrum-binding`
crates. Section D-9 details the crate authorship plan.

## Detailed Mechanics

### D-1 Substance-bar schema (substance-bar-schema.json)

The schema lives at `/specs/documentation-substance-bar-schema.json` and
describes the JSON-shaped record that every documentation artifact must
produce as a side-output during the pre-commit hook. The schema fields:

- `artifact_path` (string, required) — repository-relative path.
- `authority_tier` (int 1..4, required).
- `density_lines` (int, required) — actual line count after frontmatter
  stripping.
- `bespoke_ratio` (float 0..1, required) — fraction of section-body tokens
  that are unique to this artifact versus the corpus.
- `detailed_mechanics_subsection_count` (int, required) — count of D-N
  sub-section headings.
- `cedar_fragment_paths` (array<string>, required) — paths to Cedar fragments
  referenced by this artifact (≥1 for ADRs, may be empty for IP slices that
  delegate to a parent ADR's fragments).
- `audit_event_classes_added` (array<string>, required) — names of new audit
  event classes; may be empty when explicitly declared.
- `template_stamping_score` (float 0..1, required) — substance-bar metric;
  lower is better, lane BLOCKS at >0.55.
- `cross_reference_count` (int, required) — count of named ADR/spec/microservice
  citations in bespoke context.
- `substance_bar_facet_binding` (string, required) — reviewer-agent name that
  signed the F4-substance facet.
- `produced_at` (RFC 3339 timestamp, required).
- `tooling_signature` (string, required) — hash of the tool chain used to
  produce the record, for forensic reproducibility.

The schema is itself an artifact governed by this ADR (Tier-2, ≥500 lines of
documentation in the companion spec file).

### D-2 Bespoke ratio computation

`oya-governance-doc-bespoke-ratio` computes the bespoke ratio by:

1. Loading the candidate artifact and stripping the frontmatter block.
2. Tokenizing into 8-gram shingles (Rabin-Karp rolling hash, prime 1_000_003).
3. Loading the corpus of all artifacts of the same `doc_class` modified in
   the trailing 90 days.
4. Computing the Jaccard similarity of each candidate shingle set against
   the corpus shingle set built up to but not including the candidate.
5. Bespoke ratio = 1 − max(Jaccard) over candidates.

The 8-gram window plus 90-day rolling corpus has been selected because:

- 8-gram catches paragraph-level template-stamping while tolerating
  legitimate phrase-level reuse (e.g. recurring policy boilerplate
  "tenant context per ADR-0244").
- 90 days is sufficient to detect intra-wave template-stamping (the dominant
  failure mode) without flagging legitimate doctrine reuse across years.
- Jaccard rather than cosine because token frequencies are uninformative for
  documentation artifacts; presence/absence is the signal.

The implementation lives at `crates/oya-governance-doc-bespoke-ratio/` with
unit tests covering: legitimate quote reuse (≥0.8 bespoke ratio); template
stamping (≤0.4 bespoke ratio); identity (0.0 bespoke ratio); empty artifact
(N/A skipped).

### D-3 Density floor enforcement

`oya-governance-doc-density-floor` enforces the per-tier density floor:

| Authority tier | Min lines | Min D-N sub-sections | Notes                          |
|----------------|-----------|----------------------|--------------------------------|
| Tier 1         | 800       | 10                   | Doctrine ADRs                  |
| Tier 2         | 500       | 7                    | Capability ADRs, major specs   |
| Tier 3         | 250       | 4                    | Microservice READMEs           |
| Tier 4         | 120       | 2                    | IP slices, journey artifacts   |

A density-floor failure produces a structured violation event of class
`governance.doc.density.below_floor` on the audit chain (per ADR-0263) with
fields `(artifact_path, authority_tier, actual_lines, required_lines)`. The
lane BLOCKS the PR.

Grandfathering: any artifact authored before 2026-05-15 (cutoff date for the
post-remediation audits) is exempt until its next material edit. Material edit
is defined as any commit that changes ≥10% of the artifact's content.

### D-4 Detailed-mechanics expansion check

`oya-governance-substance-bar` parses every ADR's "Detailed Mechanics" section
and extracts the D-N sub-section headings (regex `^### D-(\d+)[^#]*$` over the
canonical Markdown). The check passes when:

- The count of distinct D-N headings ≥ the tier minimum from D-3.
- No two D-N sections have identical body content (Jaccard < 0.85).
- Each D-N section has a body of ≥40 lines after subheading stripping.
- The D-N headings are uniquely numbered 1..N with no gaps.

Synthetic D-N sections (e.g. heading present but body says "TODO: fill in")
are detected by a no-empty-section sub-check that BLOCKS on any body of less
than 8 non-blank, non-heading lines.

### D-5 Cedar policy hooks check

`oya-governance-substance-bar` loads the Cedar fragments referenced in the
artifact's "Cedar Policy Hooks" section and verifies:

- At least one fragment exists.
- Each fragment is parseable by the Cedar policy linter.
- No fragment is byte-identical to any fragment cited by a different ADR.
- No fragment is structurally equivalent (same principal, action, resource
  pattern modulo identifier rename) to any fragment cited by a different ADR.

Structural equivalence is computed by parsing the Cedar fragment, canonicalising
identifier names (alpha-rename by appearance order), and hashing the canonical
form. The Cedar canonicaliser lives in `crates/oya-policy-cedar-canonicalise/`
and is shared with the existing `oya-governance-cedar-coverage` lane.

### D-6 Audit event class registry binding

Per ADR-0263, every audit event class is registered in
`/specs/audit-events/registry.json`. Substance-bar enforcement adds a check
that for every ADR claiming to add an audit class, the class is present in the
registry by the time the ADR transitions from `Proposed` to `Accepted`. Until
then, the ADR may declare `audit_event_classes_added` as a proposed list; the
substance-bar lane treats this as a soft warning, while the ADR-promotion lane
(per ADR-0327) treats it as a BLOCKER for the Accepted transition.

New audit classes added by this ADR:

| Class                                           | Severity | Source crate                              |
|-------------------------------------------------|----------|-------------------------------------------|
| governance.doc.substance_bar.violation          | BLOCKER  | oya-governance-substance-bar              |
| governance.doc.template_stamping.detected       | BLOCKER  | oya-governance-no-template-stamping       |
| governance.doc.density.below_floor              | BLOCKER  | oya-governance-doc-density-floor          |
| governance.doc.bespoke_ratio.below_threshold    | BLOCKER  | oya-governance-doc-bespoke-ratio          |
| governance.doc.spectrum_binding.missing         | BLOCKER  | oya-governance-substance-bar-spectrum-binding |
| governance.doc.crossref.below_floor             | WARN     | oya-governance-substance-bar              |
| governance.doc.detailed_mechanics.thin_section  | BLOCKER  | oya-governance-substance-bar              |

Each class carries the tenancy-context envelope mandated by ADR-0244 and the
authority-chain attestation mandated by ADR-0246.

### D-7 No-template-stamping detection

`oya-governance-no-template-stamping` is a dedicated crate (separate from
the general substance-bar crate because template-stamping detection has
distinct false-positive characteristics and benefits from a dedicated
allow-list). Detection algorithm:

1. Strip frontmatter and section headings.
2. Slide a 32-line window across the body.
3. For each window, compute a Rabin-Karp shingle hash.
4. Compare each window against a corpus index of all artifacts of the same
   doc_class produced within the trailing 14 days.
5. If ≥3 disjoint windows share a hash with a single sibling artifact, flag
   as template-stamped.
6. Carve-outs: known boilerplate phrases (tenancy preamble per ADR-0244,
   Cedar fragment convention preamble per ADR-0243, observability preamble
   per ADR-0130) are listed in `/specs/substance-bar/template-allowlist.yaml`
   and skipped by the hashing step.

The 14-day window is intentionally shorter than the bespoke-ratio 90-day
window because template-stamping is an intra-wave failure mode whereas
bespoke ratio measures cross-wave originality.

### D-8 Multispectrum facet binding

Per ADR-0091, F4-substance is the substance facet. This ADR upgrades
F4-substance to BLOCKER class for in-scope artifacts. Mechanism:

- The artifact's frontmatter must carry `substance_bar_facet_binding: "<agent-id>"`.
- The reviewer agent identified by `<agent-id>` must have a signed verdict
  artifact at `evidence/debate/<artifact-stem>/F4-substance.signed.json`.
- The verdict must reference the artifact's content hash at the time of
  signature.
- A subsequent commit that changes the artifact's content invalidates the
  binding and re-triggers the F4-substance review.

`oya-governance-substance-bar-spectrum-binding` enforces these conditions and
emits `governance.doc.spectrum_binding.missing` or
`governance.doc.spectrum_binding.stale` on violation.

### D-9 Crate authorship plan

The W1 scaffold wave (per the post-remediation roadmap) creates the following
crates with single-concern flat layout per ADR-0131:

- `crates/oya-governance-substance-bar/` — orchestrating crate that runs S-1
  through S-8 by invoking the four specialist crates below and emits the
  aggregate verdict.
- `crates/oya-governance-no-template-stamping/` — Rabin-Karp shingle detector
  per D-7.
- `crates/oya-governance-doc-density-floor/` — per-tier line and sub-section
  count enforcement per D-3.
- `crates/oya-governance-doc-bespoke-ratio/` — Jaccard shingle bespoke-ratio
  per D-2.
- `crates/oya-governance-substance-bar-spectrum-binding/` — facet binding
  verification per D-8.

Each crate exports a `verify(artifact_path: &Path) -> Verdict` function and a
`verify_directory(root: &Path) -> Vec<Verdict>` helper used by the CI lane.
The lane is defined at `.github/workflows/oya-governance-substance-bar.yml`
with matrix entries for the four tiers (so that failures in Tier-1 vs Tier-4
present distinct status checks for triage).

### D-10 Grandfathering and migration path

Artifacts predating 2026-05-15 are tagged `substance_bar_grandfathered: true`
in their frontmatter by a one-shot migration commit. The grandfather flag is
honoured by the lane until the artifact's next material edit, at which point:

- The lane BLOCKS the PR until either the artifact clears the bar or is
  split (per S-3 expansion) or replaced by a successor artifact (per the
  ADR-0327 promotion gates).
- A `substance_bar_unflag` audit event is emitted naming the editor and the
  remediation strategy.

The migration commit is scheduled for the W1 scaffold wave landing PR plus
one (i.e. the first PR after substance-bar lane is live in CI). A draft
inventory of grandfathered artifacts is computed by running
`oya-governance-substance-bar verify-directory docs/` in shadow mode and
recording the failures; this inventory becomes the source-of-truth for the
grandfather migration.

### D-11 Reviewer-agent assignment protocol

The F4-substance facet binding required by S-8 names a specific reviewer
agent. Assignment of reviewer agents to ADRs follows the named protocol:

1. The wave descriptor (per ADR-0323) enumerates the reviewer-agent pool
   for the wave; the pool's size matches the wave's facet count (16 in
   v2.4.0 per ADR-0327 D-3) plus one wave-level reviewer.
2. Each ADR in the wave receives a distinct F4-substance reviewer agent
   drawn from the pool, with no agent assigned to more than one facet
   for the same ADR (per the consensus-debate-spectrum-lens-subagents
   feedback note).
3. Reviewer agents are dispatched via the foundry pipeline; their
   provenance attestation (per ADR-0324) names their reviewing role.
4. A reviewer agent reads the artifact at a specific commit SHA, the
   ledger of grandfathered status, the bespoke-ratio output, and the
   detailed-mechanics expansion check output before signing.
5. A signed verdict is an ed25519 signature over the canonicalised
   evidence package; the signature is published at
   `evidence/debate/<artifact-stem>/F4-substance.signed.json`.
6. A reviewer agent that cannot sign within the wave's window declares a
   `cannot-sign` reason; the wave descriptor's backup reviewer takes over.

The protocol guards against the failure mode where a single agent
signs every facet, which would collapse multispectrum review into a
single-perspective review and defeat the doctrine.

### D-12 Per-section bespoke-ratio drill-down

Beyond the artifact-level bespoke ratio in D-2, the substance bar
computes a per-section bespoke ratio that prevents the failure mode
where an artifact has 65% bespoke content concentrated in one
section while the remaining sections are template-stamped. The
per-section computation:

- For each top-level Markdown heading in the artifact, compute the
  bespoke ratio of that section's body against the corpus of
  same-heading sections in sibling artifacts.
- A section whose bespoke ratio falls below 0.35 is flagged as
  thin-section even if the artifact-level ratio passes.
- Three or more thin sections in a single artifact triggers a
  `governance.doc.detailed_mechanics.thin_section` event and BLOCKs
  promotion.

The per-section drill-down also catches the "table-of-contents
document" failure mode (named in the Context section above): such a
document typically has bespoke content only in its top-of-document
section while the body sections are sparse.

### D-13 Wave-scoped substance correlation

The substance bar interacts with the wave doctrine (ADR-0323) by
correlating substance across wave members. The correlation check:

- For each pair of artifacts in the same wave, compute the
  cross-artifact bespoke ratio.
- If two artifacts in the same wave share above 0.55 cross-artifact
  similarity, both are flagged for review even if individually they
  pass the bar.
- The flag does not auto-BLOCK; council-quality reviews the pair and
  either confirms legitimate similarity (e.g. two ADRs about closely
  related topics) or initiates a merge or split.

This guards against the failure mode where a wave's artifacts pass
individually but constitute a thinly-veiled batch of near-duplicates.

### D-14 Corpus snapshotting for forensic reproducibility

The bespoke-ratio computation depends on the corpus state at the time
of evaluation. To make verdicts reproducible:

- Each substance-bar verdict carries a corpus snapshot hash (the
  Merkle root of the corpus tree at evaluation time).
- The hash is recorded in the verdict's evidence record.
- A forensic re-evaluation can reconstruct the verdict by checking
  out the corpus at the snapshot hash.
- Corpus snapshots are stored at `evidence/substance-bar/corpus-snapshots/`
  for ≥365 days (long enough to cover all relevant audit windows).

This satisfies the documentation-rigor §3.2 reproducibility clause
and provides operators with a way to defend a verdict against later
challenge.

## Cedar Policy Hooks

The substance-bar lane is itself a privileged actor and must be authorised
by Cedar. Named patterns:

```cedar
// Fragment: cedar/substance-bar/lane-may-emit-violation.cedar
permit (
  principal == Service::"oyatie.governance.substance_bar",
  action == Audit::"emit",
  resource in AuditClass::["governance.doc.substance_bar.violation",
                          "governance.doc.template_stamping.detected",
                          "governance.doc.density.below_floor",
                          "governance.doc.bespoke_ratio.below_threshold",
                          "governance.doc.spectrum_binding.missing",
                          "governance.doc.crossref.below_floor",
                          "governance.doc.detailed_mechanics.thin_section"]
) when {
  context.audit_chain_attested == true &&
  context.tenant_id == "oyatie.governance"
};
```

```cedar
// Fragment: cedar/substance-bar/lane-may-read-corpus.cedar
permit (
  principal == Service::"oyatie.governance.substance_bar",
  action == DocCorpus::"read",
  resource in DocCorpus::Class::["adr", "journey", "ip-slice",
                                "microservice-readme", "prd", "rfc", "spec"]
) when {
  context.purpose == "substance_bar_evaluation" &&
  context.read_scope == "trailing_90_days"
};
```

```cedar
// Fragment: cedar/substance-bar/reviewer-agent-may-sign-facet.cedar
permit (
  principal in Group::"oyatie.governance.reviewer_agents.f4_substance",
  action == FacetVerdict::"sign",
  resource is DocArtifact
) when {
  context.facet == "F4-substance" &&
  context.artifact_content_hash != null &&
  context.signature_algorithm == "ed25519"
};
```

```cedar
// Fragment: cedar/substance-bar/grandfather-flag-write-restriction.cedar
forbid (
  principal,
  action == Frontmatter::"write_key",
  resource is DocArtifact
) when {
  context.key == "substance_bar_grandfathered" &&
  context.principal != Service::"oyatie.governance.substance_bar.migration"
};
```

## Audit Event Classes Emitted

Per D-6 above, this ADR adds seven new audit event classes to the registry
at `/specs/audit-events/registry.json`. Each class carries the canonical
envelope (timestamp, tenant_id, principal, resource, attestation_chain) plus
the class-specific payload defined in D-6's table. The registry entry for
each class includes:

- A bespoke description (per ADR-0263 §3).
- Severity class (BLOCKER or WARN).
- Source crate identifier.
- Sample payload fixture (per ADR-0263 §4).
- Downstream consumers (sre-dashboards, compliance-evidence-bundler,
  reviewer-agent attestation chain).

## SLO Implications

The substance-bar lane SLOs live at
`microservices/governance/slos/substance-bar.openslo.yaml` per ADR-0130:

- `substance_bar_lane_p95_latency`: ≤ 90 s for Tier-1 ADRs, ≤ 30 s for
  Tier-2/3/4 artifacts. Computed over a rolling 7-day window.
- `substance_bar_lane_false_positive_rate`: ≤ 0.5% measured by reviewer-
  agent override rate over rolling 30 days.
- `substance_bar_lane_availability`: ≥ 99.9% (the lane is part of the merge
  queue critical path; outages BLOCK all in-scope documentation merges).
- `bespoke_ratio_computation_correctness`: monitored via a daily smoke test
  that injects three synthetic artifacts (one bespoke, one stamped, one
  grandfathered) and verifies the verdicts.

A breach of the lane-availability SLO triggers the ADR-0306 disaster-mode
documentation-only mode in which the lane reports advisory rather than
BLOCKER for ≤ 24 hours while operators restore the substrate.

## Migration Path / Phased Rollout

- **Phase 0 (T-0, this ADR lands as Proposed)**: lane authored in shadow mode,
  emitting WARN-level events for failures, no BLOCKER behaviour.
- **Phase 1 (T+7 days)**: lane upgraded to BLOCKER for Tier-1 artifacts only.
- **Phase 2 (T+14 days)**: lane upgraded to BLOCKER for Tier-1 and Tier-2.
- **Phase 3 (T+21 days)**: lane upgraded to BLOCKER for all in-scope tiers.
- **Phase 4 (T+30 days)**: grandfathering migration commit lands; backlog
  inventory becomes a tracking dashboard.
- **Phase 5 (T+60 days)**: this ADR is eligible for promotion from Proposed
  to Accepted per the ADR-0327 promotion gates, contingent on ≥30 days of
  lane stability and zero unresolved false-positive incidents.

## Failure Modes + Recovery

### F-1: Lane false positive on legitimate quote reuse

A long, legitimate quotation from an upstream standard (e.g. RFC 9420 for
MLS) trips the bespoke-ratio threshold. Recovery: author wraps the quote in
the substance-bar-allowlist preamble per D-7 carve-outs, with a frontmatter
field `quoted_corpus_sources: [...]` enumerating the sources; the lane
treats listed quotes as exempt from shingle hashing.

### F-2: Lane false negative (template stamping merged)

A novel template-stamping pattern escapes detection. Recovery: a reviewer
agent files a `governance.doc.substance_bar.policy.gap` event naming the
escape pattern; the no-template-stamping crate authors a regression test
fixture and ships a follow-up release within 5 business days.

### F-3: Lane outage extends past 24h SLO budget

The lane is unavailable for longer than the disaster-mode budget. Recovery:
documentation merges are gated by manual reviewer-agent signatures at the
F4-substance facet only, with a backlog audit trail; the lane is restored
to BLOCKER class as soon as the substrate is healthy; a postmortem is
authored per ADR-0306.

### F-4: Grandfather flag abuse

An author edits an artifact and tries to claim grandfather status via
frontmatter manipulation. Recovery: the Cedar fragment in D-10 forbids
non-migration principals from writing the grandfather key; any attempt
emits a `governance.doc.grandfather.unauthorised_write` audit event and
causes the PR to fail before the lane evaluates content.

### F-5: Substance bar applied to a wholly new doc class

A new artifact category (e.g. a runbook class introduced later) appears in
the repo and is not yet enumerated in the schema. Recovery: the new class
defaults to Tier-4 thresholds and is flagged `class_unenumerated: true`;
the council-documentation reviews the class within 14 days and either
enumerates it explicitly in the schema or maintains the default.

### F-6: Corpus snapshot reproducibility failure

A forensic re-evaluation of a substance-bar verdict cannot reproduce the
verdict because the corpus snapshot is missing or corrupted. Recovery:
the verdict is invalidated; the artifact is re-evaluated against the
current corpus; the affected ADR's promotion status (per ADR-0327) is
reviewed; the corpus snapshot retention policy is audited and any
similar missing snapshots are reconstructed where possible.

### F-7: Reviewer-agent collusion

Two reviewer agents (or the same agent under different IDs) sign
multiple facets for the same ADR, violating the consensus-debate
spectrum-lens-subagents rule. Recovery: the Cedar fragment
`no-shared-agent-across-facets` (per ADR-0327) BLOCKs the second
signature; the wave coordinator re-assigns the conflicting facet to
a distinct agent; an audit event records the attempt.

### F-8: Wave-scoped correlation false flag

The D-13 correlation check flags two legitimately-similar ADRs in the
same wave as duplicates. Recovery: the council-quality reviews and
records `wave_correlation_acknowledged_legitimate: true` in the wave's
evidence ledger with a one-line justification; subsequent runs of the
correlation check honour the acknowledgement.

## Verification

Named CI checks (all are GitHub Actions matrix entries in the
`oya-governance-substance-bar.yml` workflow):

- `oya-governance-substance-bar/tier-1`
- `oya-governance-substance-bar/tier-2`
- `oya-governance-substance-bar/tier-3`
- `oya-governance-substance-bar/tier-4`
- `oya-governance-no-template-stamping`
- `oya-governance-doc-density-floor`
- `oya-governance-doc-bespoke-ratio`
- `oya-governance-substance-bar-spectrum-binding`

Named oya-governance crates:

- `oya-governance-substance-bar`
- `oya-governance-no-template-stamping`
- `oya-governance-doc-density-floor`
- `oya-governance-doc-bespoke-ratio`
- `oya-governance-substance-bar-spectrum-binding`

Verification fixtures live at `tests/governance/substance-bar/` and are
seeded by the W1 scaffold wave PR. Smoke tests run nightly via the
oya-governance-nightly workflow.

## Cross-References

### Other ADRs

- ADR-0063 (doc-coverage-enforcement) — predecessor policy elevated by this
  ADR.
- ADR-0091 (multispectrum-review-doctrine) — F4-substance facet promotion
  basis.
- ADR-0105 (layer-enum 13-canonical) — governance lane layer assignment.
- ADR-0130 (observability SLO-gated promotion) — SLO authoring location.
- ADR-0131 (per-microservice flat layout) — crate layout convention.
- ADR-0132 (product-suite-and-bundle-dissolution) — table-of-contents ban
  alignment.
- ADR-0145 (inter-microservice communication reform) — governance crate
  ownership rule.
- ADR-0242 (oyatie-is-a-tenant) — `oyatie.governance` tenant naming.
- ADR-0243 (Cedar universal gate) — Cedar fragment convention.
- ADR-0244 (tenant scoping primitive) — tenancy envelope on audit events.
- ADR-0245 (substrate-vs-product-layering) — substrate documentation
  precedence.
- ADR-0246 (cellular topology) — attestation chain on events.
- ADR-0249 (multi-category marketplace) — marketplace artifacts also
  governed.
- ADR-0263 (audit-event registry doctrine) — class registration mechanism.
- ADR-0306 (disaster mode) — lane outage degraded mode.
- ADR-0316 (capability-tier-over-product-fragmentation) — capability-tier
  artifacts subject to bar.
- ADR-0321 (b2b-saas-industry-leader-coverage) — leader-coverage artifacts
  subject to bar.
- ADR-0323 (multi-wave-sequencing-doctrine) — bar applies per wave.
- ADR-0324 (anti-script-anti-template-doctrine) — companion ban on lambda-wrap.
- ADR-0327 (wave-3-completion-criteria) — promotion gates consume bar.

### Standards

- `docs/standards/documentation-rigor.md` §1.1 substance bar prose origin.
- `docs/standards/multispectrum-review-v2.4.0.md` F4-substance promotion.

### Microservices

- `microservices/governance/substance-bar/` — lane microservice (single
  concern, flat layout per ADR-0131).
- `microservices/observability/` — SLO substrate.
- `microservices/audit-chain/` — audit event sink.

### Journeys

- `journeys/governance/jou-2026-05-20-substance-bar-rollout/` — operator
  rollout journey, authored alongside this ADR.
- `journeys/documentation/jou-2026-05-18-write-an-adr/` — author-facing
  journey updated to reflect the bar.

### Specs

- `/specs/documentation-substance-bar-schema.json`
- `/specs/substance-bar/template-allowlist.yaml`
- `/specs/audit-events/registry.json` (updated)

### External standards referenced

- ISO/IEC 25010:2011 §6 (Documentation quality characteristic).
- IEEE 1063-2001 (Software user documentation standard) §5.2.

### Feedback notes consumed

- `feedback_docs_substance_not_scaffold_2026_05_20.md`
- `feedback_go_with_original_ambition_2026_05_20.md`
- `feedback_doc_coverage_enforced.md`
- `feedback_multispectrum_review_v22.md`
- `feedback_multispectrum_adherence_facets.md`
- `feedback_no_silent_regression.md`
- `feedback_quality_performance_scalability_bar.md`

## Appendix A — Worked example: a Tier-1 ADR clearing the bar

The following worked example demonstrates the gating flow for a
hypothetical Tier-1 ADR `ADR-0399-hypothetical-doctrine.md` so that
agents and operators have a concrete reference.

1. **Author drafts the ADR** at commit `abc1234`. The artifact has
   840 lines, 11 D-N sub-sections, 4 Cedar fragments, and 2 audit
   classes claimed.
2. **Pre-commit hook runs locally**: density floor PASS (840 ≥ 800);
   sub-section count PASS (11 ≥ 10); Cedar fragments parsed PASS;
   audit classes recorded for later registration.
3. **PR opens**; the substance-bar lane runs:
   - `oya-governance-doc-density-floor` PASS.
   - `oya-governance-doc-bespoke-ratio` PASS at 0.71 (≥0.65).
   - `oya-governance-no-template-stamping` PASS (no shingle hits).
   - `oya-governance-substance-bar` thin-section check: one section
     flagged thin at 0.32 bespoke ratio.
4. **Author remediates** the thin section by adding bespoke
   substance; recommits at `abc1235`; the lane re-runs.
5. **Lane PASSes** all sub-checks; verdict signed and stored at
   `evidence/substance-bar/ADR-0399-hypothetical-doctrine/verdict.signed.json`.
6. **Reviewer-agent assignment**: the wave descriptor names
   `reviewer-substance-r12` as the F4-substance signer.
7. **Reviewer agent reads** the artifact at `abc1235`, the lane
   evidence, and the wave context; signs the F4-substance facet at
   `evidence/debate/ADR-0399-hypothetical-doctrine/F4-substance.signed.json`.
8. **The ADR's frontmatter** is updated to carry
   `substance_bar_facet_binding: "reviewer-substance-r12"` at commit
   `abc1236`.
9. **The ADR is now Proposed** per ADR-0327 D-1 transition rules and
   is eligible for the further gates G-3..G-10 in ADR-0327.

This example demonstrates the substance bar as one (G-1) of the ten
promotion gates and underscores that substance bar alone is not
sufficient for acceptance.

## Appendix B — Anti-pattern catalog (cross-reference)

Detailed anti-patterns are catalogued in ADR-0324 D-1; substance bar
detection complements that catalog without duplicating its scope. The
two systems share the corpus and the shingle index but maintain
distinct verdicts. The interplay:

| Layer                | What it detects                              | Source ADR    |
|----------------------|----------------------------------------------|---------------|
| Substance bar (D-2)  | Bespoke ratio across full corpus             | ADR-0322 D-2  |
| No-template-stamping | Intra-wave shingle hits                       | ADR-0322 D-7  |
| Thin-section         | Per-section bespoke ratio                     | ADR-0322 D-12 |
| Wave correlation     | Cross-artifact same-wave correlation          | ADR-0322 D-13 |
| Anti-script catalog  | Tool-chain anti-patterns AP-1..AP-8           | ADR-0324 D-1  |
| Loop detector        | Temporal write patterns by an agent run       | ADR-0324 D-4  |
| Prompt intent        | Batch-authorship prompt signatures            | ADR-0324 D-5  |

Each layer addresses a distinct failure mode; an artifact must clear
all that apply to its class.

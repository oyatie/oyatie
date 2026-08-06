---
doc_class: Standard
shape: Reference
length_cap: 3200
authority_tier: 2
status: Accepted
date: 2026-05-20
purpose: |
  Canonical catalogue of authoring, architecture, coordination, process,
  linguistic, and reference anti-patterns that Oyatie human and agentic
  contributors MUST avoid. This document converts real Wave-3-G failures into
  reviewable, CI-addressable, and incident-aware contribution rules.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json + docs/standards/documentation-rigor.md
planned_enforcement_ref: oya-governance-anti-pattern-catalogue
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/doc-style.md
  - docs/standards/agent-instructions-discipline.md
  - docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md
related_adrs:
  - ADR-0064
  - ADR-0116
  - ADR-0145
  - ADR-0221
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0252
  - ADR-0255
  - ADR-0263
  - ADR-0321
---

# Anti-Pattern Catalogue

> RFC-2119 usage: the keywords MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT,
> SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL in this document are to be
> interpreted as described in RFC 2119 and RFC 8174 when written in all caps.

## §1 Methodology

### §1.1 Purpose and Scope

This catalogue is a prevention surface.

It does not exist to shame prior work.

It exists because Wave-3-G proved that large agentic throughput can produce
large shallow surfaces faster than ordinary review can catch them.

The corrective action is to name the failure modes precisely.

Every contributor MUST treat a named anti-pattern as a review finding, not as
style preference.

Every anti-pattern in this catalogue has four required uses.

First, it gives authors a pre-submit check.

Second, it gives reviewers a shared vocabulary.

Third, it gives CI lane authors a detection target.

Fourth, it gives future agents a way to recognize repeated failure without
depending on fragile memory.

This document covers Markdown, JSON specs, ADRs, PRDs, microservice docs,
implementation plans, policy fragments, generated-adjacent artifacts,
review-thread handling, and Oya VCS lifecycle work.

It does not replace documentation-rigor.md.

It binds failures back to that standard, especially the intern-buildability
bar, the hyperscaler-grade sub-test, the doc-class rigor matrix, and the
six-hops graph-traversability invariant.

This catalogue names the recurrent defects so the next review can start
closer to the known risk.

### §1.2 Evidence Standard

An anti-pattern entry is accepted only when it has at least one of these
evidence classes.

Evidence class E1: a real repo incident.

Evidence class E2: a binding ADR, standard, or machine-readable spec that
forbids the shape.

Evidence class E3: a concrete failure mode observed in review artifacts.

Evidence class E4: an industry or hyperscaler precedent that proves the
proposed shape is weak.

E1 evidence is strongest because it came from actual corpus damage.

Wave-3-G supplies several E1 case studies.

The synthesis audit found ADR-0321 vendor dossiers template-stamped across
165 vendors.

The same audit found unified-ecosystem-thesis carried 700 "Thesis clause N"
rows over only 10 invariants.

The same audit found training-cost-doctrine carried 160 "Problem clause N"
rows over one problem statement.

The ERP second-pass generation script contains 80-line IP padding logic via
ensureLines, which is the concrete script-based shallow-IP signal this
catalogue forbids for human-authored canonical doctrine.

The masterplan and planning-closure surfaces reject reduced-scope shortcuts:
current scope posture is not minimum viable product, not preview, and not reduced scope.

Those incidents are cited again in §7.

### §1.3 Severity Levels

Severity P0 blocks acceptance or promotion.

P0 means the artifact is misleading, unsafe to build from, or incompatible
with a binding architecture/process rule.

P0 is not limited to code.

Wave-3-G proved an editorial P0 can block a doctrine bundle because
template-stamped docs can cause teams to implement the wrong thing.

Severity P1 requires remediation before GA, before broad reuse, or before the
artifact becomes a model for more work.

P1 often appears as a repeatable process smell or an architecture gap with a
known bounded workaround.

Severity P2 requires a tracked fix or explicit non-blocking disposition.

P2 is not ignorable.

The Codex bulk-resolve incident exists because P2 was treated as cosmetic.

Severity P3 is advisory but still logged.

P3 becomes P1 when repeated across a corpus, because repetition creates a
systemic maintenance tax.

Severity is assigned to the risk, not the file type.

A single sentence in an ADR can be P0 if it authorizes the wrong primitive.

A 5,000-line document can be P0 if the extra lines hide one missing authority.

### §1.4 CI-Enforceable Versus Review-Caught

Some anti-patterns are mechanically detectable.

Examples: repeated headings, missing frontmatter, wrong CLI primitive strings,
broken links, unresolved ADR identifiers, absent tenant_id fields, forbidden
retired tool names, and line-count floors not paired with required anchors.

Those belong in oya-governance-* lanes.

Some anti-patterns are review-caught.

Examples: conservative re-scoping, generic precedent washing, inappropriate
synchronous coupling, or a true-but-misleading claim about readiness.

Review-caught does not mean unenforceable forever.

The review finding SHOULD be converted into a deterministic lane once a
repeatable signal exists.

CI lanes MUST report the exact anti-pattern ID when possible.

Review comments SHOULD use the same IDs.

Oya VCS evidence SHOULD include the number of pattern IDs checked when a
change claims this standard.

### §1.5 Catalogue Entry Shape

Each anti-pattern row carries a stable ID.

IDs use AP-A for authoring, AP-R for architecture, AP-C for coordination, and
AP-L for linguistic/reference problems.

Each row states the failure in one sentence.

Each row states why it is harmful in Oyatie's architecture and process.

Each row states detection signals.

Each row states the safer replacement.

Each row identifies whether the first line of defense is CI, review, or both.

Each row includes at least one concrete anchor.

The anchors are not decorative.

They tell future authors what to read before arguing that a pattern is
acceptable.

### §1.6 Review Protocol

Reviewers MUST check a canonical doc change against §2 and §5.

Reviewers MUST check architecture and microservice changes against §3.

Reviewers MUST check Oya VCS, PR, evidence, and lifecycle work against §4.

Reviewers SHOULD quote the pattern ID in review comments.

Reviewers SHOULD avoid vague comments such as "this is shallow."

A better comment is: "AP-A03 line-floor-met-but-substance-empty: this PR meets
1,500 lines but lacks file-path trace, failure modes, and ADR anchors."

Reviewers MUST NOT bulk-resolve anti-pattern comments.

Every comment receives one of: fixed, rebutted with evidence, superseded by a
stronger finding, or split into a tracked follow-up with owner and gate.

### §1.7 Author Protocol

Authors MUST read the Wave-3-G case studies before authoring broad doctrine.

Authors MUST decide whether a requested artifact is a reference, explanation,
how-to, tutorial, ADR, PRD, spec, or runbook before writing.

Authors MUST write the real content before satisfying line floors.

Authors MUST name domain objects, tenant boundaries, event classes, policy
fragments, regions, versions, and ADR anchors where relevant.

Authors MUST state out-of-scope boundaries only when the user requested a
bounded scope or a binding plan permits the deferral.

Authors MUST NOT shrink requested scope because the full scope is large.

Large scope demands sequencing, not quiet ambition loss.

### §1.8 Promotion Protocol

Promotion evidence SHOULD include anti-pattern coverage.

A ChangeBundle that adds a new standard SHOULD report the count of catalogue
patterns evaluated.

A ChangeBundle that adds architecture doctrine SHOULD report any AP-R findings
encountered and resolved.

A ChangeBundle that handles review-thread closure SHOULD report every AP-C04
bulk-resolve risk disposition.

A ChangeBundle that claims doc rigor SHOULD report both line count and density
anchors.

Line count alone is not evidence.

Pattern count alone is not evidence.

The useful evidence is: which patterns were plausible, how they were avoided,
and what validation proves the avoidance.

### §1.9 Adding New Anti-Patterns

New anti-patterns MAY be added when a repeated risk appears.

The addition MUST name the triggering incident or authoritative source.

The addition MUST classify severity.

The addition MUST state whether the first detection surface is CI or review.

The addition SHOULD include an example of a false positive.

The addition SHOULD include a safe replacement.

Do not add a broad moral rule when a narrow operational rule is enough.

Do not add a new pattern when an existing pattern can absorb the case with a
clear note.

## §2 Authoring Anti-Patterns

### AP-A01 Template-Stamping

Pattern ID: AP-A01.

Severity: P0 when the stamped artifact claims canonical coverage.

Failure: one skeleton is copied N times with only names, tiers, or slugs
swapped.

Wave-3-G case: ADR-0321 had 165 vendor dossiers with identical Cedar permit,
ontology projection, workflow-template, UX-shell, pack-overlay, migration, and
failure-mode sentences.

Why it harms Oyatie: vendor-specific integration details disappear, yet the
artifact looks complete enough to drive implementation.

Detection: near-identical paragraph shingles across rows; same verbs and
failure modes under different vendor names.

Detection: repeated "owned by workflow-engine unless..." style sentences in
every dossier.

CI candidate: shingle similarity over sections with different entity names.

Review cue: ask what a MuleSoft, Tableau, Snowflake, Workday, or GitHub row
teaches that the previous row did not.

Safer replacement: shared macro plus per-entity delta.

Required delta: name at least three entity-specific actions, three objects,
three failure modes, and one migration wrinkle.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §6.1.

Anchor: docs/decisions/ADR-0709-general-live-apex.md.

### AP-A02 Clause-Loop Padding

Pattern ID: AP-A02.

Severity: P0 for architecture deep-dives and doctrine docs.

Failure: a clause label increments while the body repeats the same point.

Wave-3-G case: unified-ecosystem-thesis carried 700 "Thesis clause N" rows over
10 distinct invariants.

Wave-3-G case: training-cost-doctrine carried 160 "Problem clause N" rows over
one problem statement.

Why it harms Oyatie: reviewers see mass and assume depth, but implementers get
no additional construction information.

Detection: heading or sentence count grows linearly while unique nouns,
commands, file paths, equations, or ADR anchors stay flat.

CI candidate: repeated clause prefix count plus low unique-paragraph ratio.

Review cue: collapse the loop mentally; if the document loses no meaning, it
was padding.

Safer replacement: one invariant block per distinct invariant.

Safer replacement: one problem statement plus real model, numbers, examples,
and failure branches.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §6.2.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §6.3.

### AP-A03 Line-Floor-Met-But-Substance-Empty

Pattern ID: AP-A03.

Severity: P0 when the file is used as a readiness gate.

Failure: a document meets a mandated line count without meeting the density
signals for its doc class.

Why it harms Oyatie: line floors are safety rails, not permission to write
inflated prose.

Detection: few file paths, few commands, few schema fields, no rollback path,
no failure-mode tree, or no test command despite a large line count.

CI candidate: doc-class matrix check from documentation-rigor.md §2.

Review cue: ask whether an intern can build the described primitive from the
document alone.

False positive: a long catalogue can be valid when each row contains distinct
detection, impact, and replacement guidance.

Safer replacement: satisfy the doc-class anchors first, then fill length with
real cases and cross-references.

Required evidence: line count plus pattern count, source anchors, and density
signals.

Anchor: docs/standards/documentation-rigor.md §1 and §2.

### AP-A04 Script-Based Shallow Generation

Pattern ID: AP-A04.

Severity: P0 for canonical doctrine; P1 for scaffolds unless clearly tagged.

Failure: a script produces plausible-looking docs by padding base text to a
line floor.

Wave-3-G adjacent case: the now-retired ERP second-pass generator padded IP documents to
80 lines with repeated "IP detail" bullets.

Why it harms Oyatie: generated rows can pass mechanical size checks while
missing domain decisions.

Detection: helper names such as ensureLines; numbered filler bullets; repeated
tenant_id/data_class/audit_event_class lists without slice-specific state.

CI candidate: generated-file provenance marker plus minimum unique-claim ratio.

Review cue: identify the one line that would change implementation for this
specific bounded context.

Safer replacement: scripts may scaffold placeholders only when files are
clearly marked scaffold and blocked from promotion.

Safer replacement: canonical docs are authored or enriched with bespoke
domain-specific decisions before promotion.

Anchor: retired ERP second-pass generator ipDoc ensureLines pattern.

Anchor: docs/standards/documentation-rigor.md §2 Implementation Plan floor.

### AP-A05 Generic Phrases Without Named Specifics

Pattern ID: AP-A05.

Severity: P1 by default; P0 when the phrase defines an interface or boundary.

Failure: prose says "the µservice handles X" without naming the entities,
commands, events, schemas, policies, and failure modes.

Why it harms Oyatie: generic verbs hide ownership gaps.

Detection: handle, manage, process, support, integrate, enable, provide, or
facilitate without a nearby object model.

CI candidate: lint high-risk verbs and require nearby nouns from glossary,
schemas, or ADR identifiers.

Review cue: replace "handles payroll" with tables, events, permits, and flows.

Safer replacement: "payroll owns PayrollRun, PayStatement, TaxWithholding, and
EVT_PAYROLL_RUN_SEALED; workflow-engine only orchestrates approval."

Required specifics: owner, object, action, input, output, policy, event, and
rollback.

Anchor: docs/standards/documentation-rigor.md §1.1.

### AP-A06 Missing ADR, Standard, or Regulatory Anchors

Pattern ID: AP-A06.

Severity: P0 when the doc introduces a primitive.

Failure: a new API, policy, data model, event class, runtime placement, or
compliance behavior lacks binding references.

Why it harms Oyatie: future agents cannot tell whether the primitive is
canonical, provisional, or accidental.

Detection: new capitalized primitives without related_adrs or binding_adr.

Detection: regulated claims without jurisdiction, statute, pack, or standard
reference.

CI candidate: frontmatter related_adrs presence plus link-resolution checks.

Review cue: ask "what forbids a future agent from changing this?"

Safer replacement: cite the specific ADR section or spec field and state what
the citation constrains.

False positive: a glossary-only entry may cite a hub rather than a primitive
ADR if it introduces no behavior.

Anchor: docs/standards/documentation-rigor.md §3.

### AP-A07 Quota Padding Instead of Substance Density

Pattern ID: AP-A07.

Severity: P1, escalating to P0 when repeated across a corpus.

Failure: an author optimizes for line, artifact, or count quotas instead of
new information.

Why it harms Oyatie: quota padding creates review fatigue and hides the real
missing surfaces.

Detection: many files differ only by title; many bullets repeat the same
constraint list; count claim is prominent but evidence claim is weak.

CI candidate: corpus similarity and repeated-fragment reports.

Review cue: ask which added line reduces implementation ambiguity.

Safer replacement: use quotas as floors, then satisfy each line with a named
decision, example, test, exception, or failure branch.

Case link: Wave-3-G docs had high line count but low unique invariant density.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §30.

### AP-A08 Cross-Vendor Variable Swap

Pattern ID: AP-A08.

Severity: P0 for vendor coverage and integration dossiers.

Failure: vendor rows vary only the vendor name, category, destination service,
or tier.

Why it harms Oyatie: integration work depends on the vendor's API shape,
auth model, rate limits, objects, and operational failures.

Detection: same migration text under SaaS, data warehouse, ticketing, ERP, and
collaboration vendors.

CI candidate: compare rows after replacing vendor names with placeholders.

Review cue: name one vendor-specific endpoint, object, auth challenge, and
rollback branch.

Safer replacement: keep the common shape in a macro and put vendor deltas in
first-class fields.

Required vendor delta: source API, canonical objects, permit verbs, workflow
templates, failure modes, and pack restrictions.

Anchor: ADR-0321 §D.

### AP-A09 Recycled Boilerplate Per Microservice

Pattern ID: AP-A09.

Severity: P1 by default; P0 when it claims PRD or architecture readiness.

Failure: every µservice receives the same PRD, compliance, runbook, or
architecture language with the service name swapped.

Why it harms Oyatie: flat per-µservice layout does not mean flat content.

Detection: same artifact roster with no bounded-context differences.

Detection: identical compliance traces across unrelated services.

CI candidate: per-µservice duplicate block detection.

Review cue: compare payments, marketplace, tenancy, and mail; if the risk text
is interchangeable, it is not architecture.

Safer replacement: use the PR-143 artifact roster for shape, then fill service
owned entities, policies, and failure modes.

Anchor: docs/standards/documentation-rigor.md completeness invariants.

Anchor: docs/decisions/ADR-0701-monorepo-capability-live-apex.md.

### AP-A10 Halting Before Original Ambition Is Delivered

Pattern ID: AP-A10.

Severity: P0 when a user requested full coverage.

Failure: the work stops after a smaller local artifact and reports completion
as though the original ambition was met.

Why it harms Oyatie: planning closure depends on named scope, not on the
largest completed subset.

Detection: final report says "complete" while open scope rows remain in the
requested sections.

CI candidate: difficult; mostly review-caught through scope checklist.

Review cue: compare deliverable headings against the user's requested list.

Safer replacement: mark partial completion honestly and continue, or name a
true blocker.

Required phrase when blocked: "blocked by X; completed Y; remaining Z."

Anchor: specs/masterplan.json scope_posture `not_mvp_not_preview_not_reduced_scope`.

### AP-A11 Conservative Re-Scoping

Pattern ID: AP-A11.

Severity: P0 when the smaller scope contradicts explicit user direction.

Failure: an agent silently narrows a broad requested deliverable because the
full version is large.

Why it harms Oyatie: conservative scope feels safe locally but accumulates
system-wide false readiness.

Detection: "for now", "initial", "lightweight", "minimum viable product", or "representative"
appears where the request asked for definitive coverage.

CI candidate: not reliable; review-caught.

Review cue: ask whether the user asked for a sample, a starter, or the
canonical artifact.

Safer replacement: sequence the full scope into slices while preserving the
full deliverable contract.

Safer replacement: if time or authority blocks full delivery, document the
checkpoint and remaining map.

Anchor: specs/masterplan.json no_placeholders_stubs_thin_scaffolds_or_deferred_scope.

### AP-A12 Status Laundering

Pattern ID: AP-A12.

Severity: P0 for ADRs, specs, standards, and promotion evidence.

Failure: a file declares Accepted, complete, green, or rigorous while its
required evidence is absent or inconsistent.

Wave-3-G case: the synthesis audit flagged ADR status inconsistencies as P0.

Why it harms Oyatie: status fields drive agent decisions and promotion gates.

Detection: frontmatter status disagrees with synthesis, audit, or bundle
state.

Detection: "Accepted" appears while P0 fix-sets remain open.

CI candidate: enum validation plus cross-doc status graph.

Review cue: ask what event promoted the status and where that event is stored.

Safer replacement: keep Proposed until promotion evidence exists.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §6.4.

### AP-A13 Scaffold Presented As Coverage

Pattern ID: AP-A13.

Severity: P0 when the scaffold gates implementation or GA.

Failure: a placeholder, reserved anchor, or generated skeleton is described as
full coverage.

Why it harms Oyatie: downstream work treats placeholders as real constraints.

Detection: file contains headings but no concrete paths, tables, schema,
permits, or tests.

Detection: "reserved", "future", or "TBD" appears in a supposedly complete
section.

CI candidate: placeholder string scan plus doc-class density validation.

Review cue: determine whether the artifact can refuse a wrong
implementation.

Safer replacement: tag scaffold status explicitly and block promotion until
delta content lands.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §30.2.

### AP-A14 Disconnected Canonical Doc

Pattern ID: AP-A14.

Severity: P1, escalating to P0 for standards and ADRs.

Failure: a document is locally rich but not reachable through the canonical
graph.

Why it harms Oyatie: agents following root-hub pointers miss it, and humans
cannot discover its authority.

Detection: no companion_docs, no inbound citation, no root-hub or catalog row,
or broken relative links.

CI candidate: six-hop graph traversal.

Review cue: ask how a cold-start agent reaches this doc from docs/README.md or
root-hub-pointers.json.

Safer replacement: add graph edges through catalog, standards index, or a
binding ADR.

False positive: a temporary scratch note outside canonical paths may be
unreachable by design.

Anchor: docs/standards/documentation-rigor.md §3.1.

### AP-A15 Boilerplate Acceptance Criteria

Pattern ID: AP-A15.

Severity: P1 for implementation plans; P0 when acceptance drives a PR gate.

Failure: acceptance criteria say tests pass, policies remain default-deny, and
events emit, but never name the specific tests, policies, or events.

Why it harms Oyatie: generic AC cannot distinguish correct from incorrect
implementation.

Detection: "unit, contract, policy, replay, integration" appears without
command names.

Detection: event names are templated or absent.

CI candidate: require AC rows to carry command or artifact paths.

Review cue: each AC should be falsifiable by one concrete check.

Safer replacement: state exact command, file path, event class, and refusal
case.

Anchor: retired ERP second-pass generator IP acceptance-template pattern.

### AP-A16 Thin Implementation Plans From Filler Lines

Pattern ID: AP-A16.

Severity: P1 by default; P0 when IPs are the sole implementation guide.

Failure: an IP reaches 80 lines through repeated detail bullets but does not
sequence files, contracts, tests, and rollback.

Why it harms Oyatie: implementers get a length-compliant document that still
requires guessing.

Detection: "IP detail 001" through N repeats the same fields.

Detection: no file ownership table or dependency order.

CI candidate: line floor plus required section presence.

Review cue: can a contributor know which file to open first?

Safer replacement: one IP slice equals one deployable change with named files,
acceptance commands, and rollback.

Anchor: docs/standards/documentation-rigor.md §2 Migration/IP expectations.

### AP-A17 Appendix Shadowing

Pattern ID: AP-A17.

Severity: P1, escalating to P0 if the appendix carries the real rule.

Failure: a document leaves the weak main body intact and hides corrections in
an appendix.

Why it harms Oyatie: downstream readers and agents often stop at the main
decision section.

Detection: "see appendix" handles contradictions that should be in the
canonical section.

CI candidate: hard to enforce; review-caught.

Review cue: ask whether the main body is safe if the appendix is never read.

Safer replacement: move normative corrections into the decision/mechanics
section and keep appendices as evidence.

Anchor: docs/standards/doc-style.md heading hierarchy and authority shape.

### AP-A18 Evidence Summary Without Source Path

Pattern ID: AP-A18.

Severity: P1 for reviews; P0 for promotion gates.

Failure: an audit says "verified" without source file, command, line range, or
artifact hash.

Why it harms Oyatie: evidence cannot be reproduced after context compaction.

Detection: summary has verdicts but no repo-relative paths.

Detection: "grep-confirmed" appears without the grep target.

CI candidate: evidence schema requiring command/path fields.

Review cue: ask if another agent can rerun the evidence from the summary
alone.

Safer replacement: include command, path, result, and timestamp where relevant.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §30.

### AP-A19 Precedent Washing

Pattern ID: AP-A19.

Severity: P1; P0 when the precedent justifies a risky architecture.

Failure: a doc names AWS, Stripe, Palantir, Cloudflare, or Google without
showing the specific pattern being adopted.

Why it harms Oyatie: brand names can disguise an invented or incompatible
design.

Detection: "hyperscaler-grade" appears without a concrete product, paper, or
pattern.

Detection: industry citation does not map to the primitive under decision.

CI candidate: weak; review-caught with optional citation lint.

Review cue: ask "which exact operational pattern is copied, and what is not
copied?"

Safer replacement: name the precedent, map the relevant property, and state
the rejected misapplication.

Anchor: docs/standards/documentation-rigor.md §1.1 named precedent.

### AP-A20 Future-CI Promise Without Lane

Pattern ID: AP-A20.

Severity: P1; P0 when CI is the only claimed enforcement.

Failure: a document says "CI will enforce this" without naming lane, crate,
status, trigger, and promotion date.

Why it harms Oyatie: promises look like controls but do not block drift.

Detection: "future CI", "planned validator", or "will be checked" with no
planned_enforcement_ref.

CI candidate: frontmatter enforcement field check.

Review cue: ask whether the lane is active, advisory, planned, or blocker.

Safer replacement: declare planned_enforcement_ref now; add active lane only
after workflow and quality registry are wired.

Anchor: docs/standards/doc-style.md frontmatter shape.

### AP-A21 Mega-Doc Without Maintenance Map

Pattern ID: AP-A21.

Severity: P2 by default; P1 for standards.

Failure: a large canonical document lacks owner, update protocol, split
criteria, and review cadence.

Why it harms Oyatie: large docs rot unless maintenance is part of the artifact.

Detection: length exceeds ordinary cap but no rationale or lifecycle is
stated.

CI candidate: length-cap exception requires maintenance section.

Review cue: ask who edits this when ADR-0145 or Oya VCS changes.

Safer replacement: add clear owner, enforcement lane, update protocol, and
cross-reference set.

Anchor: docs/standards/doc-style.md Diátaxis length caps.

### AP-A22 Intern Route Missing

Pattern ID: AP-A22.

Severity: P1 for explanations; P0 for build docs.

Failure: a doc assumes prior project lore and omits the cold-start path.

Why it harms Oyatie: agentic handoffs regularly start without full session
history.

Detection: first use of a term precedes definition or link.

Detection: a procedure starts in the middle of a workflow.

CI candidate: glossary coverage and six-hop traversal.

Review cue: ask what a new contributor reads immediately before this file.

Safer replacement: add entry points, prerequisites, and "where to read next."

Anchor: docs/standards/documentation-rigor.md intern-buildability test.

### AP-A23 Manual Count Chasing

Pattern ID: AP-A23.

Severity: P2; P1 when repeated by a team.

Failure: contributors manually add rows, files, or review artifacts solely to
hit numeric thresholds.

Why it harms Oyatie: manual count chasing produces drift and wastes review
capacity.

Detection: numbered files or rows with uniform bodies and no generator or
validator.

Detection: no reproducible command for refreshing the corpus.

CI candidate: repeated-fragment detector plus generated-provenance policy.

Review cue: distinguish content authoring from mechanical enumeration.

Safer replacement: automate mechanical inventory; hand-author the judgment and
domain deltas.

Anchor: feedback_automate_everything references in specs/agent-durable-goal.json.

### AP-A24 Stealth Deferral

Pattern ID: AP-A24.

Severity: P1; P0 when deferring a prerequisite.

Failure: an author moves hard work into "future work" without owner, date,
acceptance gate, or dependency impact.

Why it harms Oyatie: deferral without a gate becomes invisible scope loss.

Detection: "later", "follow-up", "future", or "out of scope" without a
successor artifact.

CI candidate: future-work rows require owner and target.

Review cue: ask whether downstream implementation is safe before the deferred
work.

Safer replacement: name the successor wave, blocking status, owner, and exact
promotion condition.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §30.1.

## §3 Architecture Anti-Patterns

### AP-R01 Legacy Universal Mediator Coupling

Pattern ID: AP-R01.

Severity: P0 for new architecture claims.

Failure: a design still assumes all inter-µservice traffic must pass through
Workflow + Ontology as a universal mediator.

Why it harms Oyatie: ADR-0145 retired the universal mediator rule and replaced
it with direct service calls under audit, contract, and trace invariants.

Detection: "all calls flow through Workflow/Ontology" without saga or read
projection rationale.

CI candidate: grep for retired memory language and require ADR-0145 exception.

Review cue: classify the flow as direct call, durable saga, or ontology read.

Safer replacement: direct mTLS service call for ordinary request/response;
Workflow only for long-running orchestration; Ontology only for projection/read
semantics.

Anchor: docs/decisions/ADR-0701-monorepo-capability-live-apex.md.

### AP-R02 Cross-Product Coupling Without Current Adapter Contract

Pattern ID: AP-R02.

Severity: P0 when state-changing product boundaries are involved.

Failure: product A reaches into product B's domain without a contract, audit
seal, trace propagation, or explicit Workflow saga.

Why it harms Oyatie: retiring the universal mediator did not permit ungoverned
coupling.

Detection: shared database reads, direct table writes, or API calls with no
OpenAPI/AsyncAPI/proto contract.

CI candidate: dependency graph plus contract-ref presence.

Review cue: ask which service owns the canonical entity and where the caller's
audit seal is emitted.

Safer replacement: contract-first direct call, or Workflow saga when the flow
is long-running, retrying, or human-in-the-loop.

Anchor: ADR-0145 invariants 1, 2, and 3.

### AP-R03 Cedar Policy Explosion

Pattern ID: AP-R03.

Severity: P1; P0 when policy count makes reasoning impossible.

Failure: a µservice introduces many near-duplicate Cedar policies instead of
composable fragments and shared predicates.

Why it harms Oyatie: policy sprawl weakens default-deny review and multiplies
soak, rollout, and audit cost.

Detection: policy files differ only by action or pack while predicates repeat.

CI candidate: policy AST similarity and per-µservice policy-count threshold.

Review cue: ask whether baseline, pack, overlay, and tenant fragments can
compose instead.

Safer replacement: small Cedar fragments with named scopes, shared schema, and
registry-backed audit event classes.

Anchor: ADR-0243 Cedar as universal gate.

### AP-R04 Per-Tenant Codebase Variant

Pattern ID: AP-R04.

Severity: P0 for product architecture.

Failure: a tenant or jurisdiction receives a forked codebase instead of
canonical base plus pack/localization overlay.

Why it harms Oyatie: variant codebases destroy upgrade coherence and audit
uniformity.

Detection: tenant-specific crate, service, route, or schema fork that differs
only by pack behavior.

CI candidate: naming scan for tenant names in source paths plus overlay
registry validation.

Review cue: ask which canonical-base primitive the tenant-specific code should
parameterize.

Safer replacement: ADR-0064 canonical base, localization pack, compliance pack,
and Cedar overlay.

Anchor: docs/decisions/ADR-0709-general-live-apex.md.

### AP-R05 Audit-Event Class Proliferation

Pattern ID: AP-R05.

Severity: P0 when an event class is emitted without registry entry.

Failure: services mint event class names ad hoc in docs, schemas, logs, or
code.

Why it harms Oyatie: audit-chain evidence becomes unqueryable and compliance
mapping drifts.

Detection: `audit_event_class` values not present in registry or ADR-0263
reverse references.

CI candidate: `oya gate validate audit-event-class-registered`.

Review cue: ask where schema, retention, cardinality, and emission target are
registered.

Safer replacement: define the class in the downstream ADR and registry before
emission.

Anchor: docs/decisions/ADR-0706-observability-live-apex.md.

### AP-R06 Synchronous Call Where Async Suffices

Pattern ID: AP-R06.

Severity: P1; P0 if it creates cascading outage risk.

Failure: a request path waits on another µservice even though an event,
outbox, saga, or async projection would satisfy the user outcome.

Why it harms Oyatie: synchronous fan-out raises p99 latency and couples
availability domains.

Detection: direct call in a user-facing path for notification, analytics,
audit enrichment, cache warm, or non-blocking status update.

CI candidate: latency-budget annotation required for synchronous calls.

Review cue: ask whether the caller needs the result before responding.

Safer replacement: outbox event, durable Workflow saga, or async projection.

Anchor: docs/standards/outbox-pattern-canonical.md.

### AP-R07 Multi-Region Without HLC Versus TrueTime Tier

Pattern ID: AP-R07.

Severity: P0 for replicated state.

Failure: a multi-region design omits the clock/consistency tier decision.

Why it harms Oyatie: causal ordering, audit ordering, and external consistency
cannot be inferred after the fact.

Detection: "multi-region", "replicated", or "global" without HLC or TrueTime
classification.

CI candidate: multi-region manifest requires consistency_tier.

Review cue: ask whether the flow needs causal ordering or external
consistency.

Safer replacement: HLC default for most operations; TrueTime-style tier only
for Tier-4 external consistency.

Anchor: docs/decisions/ADR-0709-general-live-apex.md.

### AP-R08 Tenant-ID Propagation Gap

Pattern ID: AP-R08.

Severity: P0 for tenant-scoped data, policy, logs, metrics, traces, and events.

Failure: a request, event, table, metric, log, trace, or policy evaluation
lacks tenant_id or sub-scope where ADR-0244 requires it.

Why it harms Oyatie: tenant isolation, FinOps, audit, residency, and policy
decisions all depend on the universal scoping primitive.

Detection: public or internal schema without tenant_id in tenant-bearing
context.

CI candidate: schema and log-field validators.

Review cue: ask which tenant owns the row and which sub-scope narrows the
action.

Safer replacement: tenant_id plus sub_scope_path in persistence, envelope,
policy context, and telemetry.

Anchor: docs/decisions/ADR-0702-identity-authz-live-apex.md.

### AP-R09 BYOK Term Conflation

Pattern ID: AP-R09.

Severity: P0 in compliance, intelligence, and security docs.

Failure: provider-BYOK and encryption-BYOK are treated as the same feature.

Why it harms Oyatie: provider API credentials and encryption key custody have
different owners, risks, and compliance gates.

Detection: BYOK appears without specifying provider or encryption.

CI candidate: BYOK lint requiring qualifier.

Review cue: ask whether the tenant is bringing an LLM/provider API credential
or a KMS/HSM encryption root.

Safer replacement: provider-BYOK cites ADR-0255 §D-4; encryption-BYOK cites
ADR-0251 §D-10.

Anchor: docs/decisions/ADR-0701-monorepo-capability-live-apex.md.

Anchor: docs/decisions/ADR-0708-platform-foundations-live-apex.md.

### AP-R10 Per-Service KMS Reinvention

Pattern ID: AP-R10.

Severity: P1; P0 for regulated data.

Failure: each µservice invents its own key hierarchy, HSM posture, rotation
schedule, or envelope format.

Why it harms Oyatie: compliance evidence fragments and incident response loses
one root of truth.

Detection: local KMS schema or runbook appears outside the encryption
substrate contract.

CI candidate: deny new key-management primitives without ADR-0251 reference.

Review cue: ask which central encryption substrate primitive this consumes.

Safer replacement: per-data-class encryption substrate with pack overlays and
tenant policy.

Anchor: ADR-0251 §D-10.

### AP-R11 Capability Tier Without Registry

Pattern ID: AP-R11.

Severity: P1; P0 when tier controls customer access.

Failure: a doc claims bronze/silver/gold/platinum or capability-tier behavior
without registry-backed grants and projections.

Why it harms Oyatie: product packaging drifts from policy, ontology, and audit
evidence.

Detection: capability tier appears in prose only.

CI candidate: capability-tier schema and registry row validation.

Review cue: ask which tier grant activates the behavior and which audit events
record it.

Safer replacement: registry/capability-tiers rows plus schema, Cedar, ontology,
and audit bindings.

Anchor: specs/capability-tier-schema.json.

### AP-R12 Marketplace Settlement Leakage

Pattern ID: AP-R12.

Severity: P0 for commerce surfaces.

Failure: a product service settles tenant deals, billing, or marketplace
ownership that ADR-0314 or marketplace doctrine reserves elsewhere.

Why it harms Oyatie: settlement authority must be uniform for audit, revenue,
refund, and compliance evidence.

Detection: product service owns deal state beyond its domain facts.

CI candidate: manifest settlement_owner field must match marketplace for
tenant deals.

Review cue: ask whether the service owns domain facts or the economic
settlement.

Safer replacement: product publishes facts; marketplace settles deals and
records economic evidence.

Anchor: ADR-0314 references in Wave-3-G synthesis and microservice phases.

### AP-R13 Central Mediator Owns Audit Emission

Pattern ID: AP-R13.

Severity: P0 for state-changing flows.

Failure: a design routes audit responsibility to a central mediator instead of
the calling service.

Why it harms Oyatie: the service that decides to mutate state must seal the
decision and its policy context.

Detection: "Workflow emits the audit event for service X's write" without a
saga ownership explanation.

CI candidate: audit-event emission owner must match caller or explicit saga
step.

Review cue: ask where the caller's audit seal is created.

Safer replacement: caller emits; audit-chain stores; Workflow emits only for
Workflow-owned state transitions.

Anchor: ADR-0145 invariant 1.

### AP-R14 Provider Credential Persistence In Substrate

Pattern ID: AP-R14.

Severity: P0.

Failure: provider credentials are stored, cached, logged, or long-lived inside
Intelligence or a caller process.

Why it harms Oyatie: ADR-0255 and its amendment separate SecretReference,
OpenBao, sidecar, and short-lived handles.

Detection: credential material in DB schemas, logs, config, or caches.

CI candidate: secret scanners plus no-credentials-in-substrate lane.

Review cue: ask where raw provider credentials can live and for how long.

Safer replacement: SecretReference, credential sidecar UDS, or OpenBao token
with TTL no longer than the contract permits.

Anchor: ADR-0255 §D-4 and ADR-0255 amendment credential sidecar rules.

### AP-R15 Guardrail Gateway Monolith

Pattern ID: AP-R15.

Severity: P1; P0 if it becomes a mandatory choke point.

Failure: all AI guardrails become one network gateway when the library-first
contract says most checks run in-process.

Why it harms Oyatie: gateway monoliths add latency, central outage risk, and
policy drift.

Detection: every prompt call must hit one guardrail service even for local
detectors.

CI candidate: intelligence dispatch manifests require local-versus-network
mode.

Review cue: ask which checks require central coordination and which are local.

Safer replacement: library-first guardrails with network opt-in only for
shared state or heavy inference.

Anchor: ADR-0255 amendment library-first network-opt-in clarification.

### AP-R16 Ontology Write-Path Confusion

Pattern ID: AP-R16.

Severity: P0 when canonical entity ownership moves by accident.

Failure: Ontology is treated as the write owner for every domain object rather
than the projection/read substrate.

Why it harms Oyatie: canonical entity owners must retain write authority and
project to Ontology.

Detection: product PRD says "Ontology owns" an entity that the product creates
and mutates.

CI candidate: manifest owned_entities versus ontology_projection refs.

Review cue: ask whether the action changes state or reads projected
relationships.

Safer replacement: product owns canonical writes; Ontology owns projection,
query, and graph read surfaces.

Anchor: ADR-0145 and ontology PRD read-side framing.

### AP-R17 Workflow As CRUD Gateway

Pattern ID: AP-R17.

Severity: P1; P0 when it blocks direct service ownership.

Failure: Workflow is used as the mandatory CRUD API for simple service-owned
mutations.

Why it harms Oyatie: Workflow is for durable orchestration, not routine data
ownership.

Detection: one-step create/update/delete operations require a Workflow run
with no retry, compensation, approval, or temporal dependency.

CI candidate: Workflow invocation requires orchestration_reason.

Review cue: ask what durability property Workflow adds.

Safer replacement: direct service API for simple mutations; Workflow saga for
multi-step, retrying, delayed, or approval-bound processes.

Anchor: ADR-0145 §D.

### AP-R18 Statelessness Assumed, Not Proved

Pattern ID: AP-R18.

Severity: P1; P0 for autoscaled services.

Failure: architecture says "stateless" without naming where state lives, how
idempotency works, and how retries behave.

Why it harms Oyatie: autoscaling and failure recovery depend on real state
placement.

Detection: no idempotency key, no outbox, no persistence owner, no replay test.

CI candidate: stateful/stateless manifest fields with validation.

Review cue: ask what happens after pod restart during a write.

Safer replacement: state placement table, idempotency strategy, replay test,
and rollback procedure.

Anchor: docs/standards/idempotency-keys-canonical.md.

### AP-R19 Runtime Placement Afterthought

Pattern ID: AP-R19.

Severity: P1; P0 for server workloads.

Failure: docs describe behavior without saying whether the workload runs in
Kubernetes, Wasm, VM, sidecar, worker, or client.

Why it harms Oyatie: deployment, scaling, security, and policy differ by
runtime placement.

Detection: no iac, workload kind, cell tier, or pod/VM/sidecar boundary.

CI candidate: manifest runtime_placement required for server components.

Review cue: ask where the process runs and who restarts it.

Safer replacement: explicit runtime placement, scaling tier, and failure
domain.

Anchor: ADR-0254 deployment model spectrum.

### AP-R20 TrueTime Everywhere

Pattern ID: AP-R20.

Severity: P1; P0 when it creates unrealistic dependencies.

Failure: a service demands TrueTime-style external consistency where HLC causal
ordering is enough.

Why it harms Oyatie: over-specifying atomic clock semantics raises cost and
implementation complexity.

Detection: "TrueTime" appears outside Tier-4 or finance-grade external
consistency rationale.

CI candidate: consistency_tier must match cell tier.

Review cue: ask what observer-visible invariant HLC cannot satisfy.

Safer replacement: HLC default; TrueTime only for stated Tier-4 external
consistency.

Anchor: ADR-0252 §D-1 and §D-2.

### AP-R21 Region Pin As Config Only

Pattern ID: AP-R21.

Severity: P0 for regulated data.

Failure: residency is a string flag without enforcement in routing, storage,
backup, audit, and export paths.

Why it harms Oyatie: sovereign packs require operational behavior, not labels.

Detection: `residency_pack` appears only in frontmatter or docs.

CI candidate: pack manifest must bind storage, network, backup, export, and
audit policy.

Review cue: trace one regulated record through write, read, backup, and DSAR.

Safer replacement: pack overlay with Cedar, storage placement, data-class, and
audit-chain constraints.

Anchor: ADR-0251 and specs/sovereign-cloud-overlays.json.

### AP-R22 Platform Naming Leakage

Pattern ID: AP-R22.

Severity: P2; P1 when it affects crate or service names.

Failure: "platform" is used where Oyatie doctrine requires "shared" or a more
specific substrate name.

Why it harms Oyatie: platform language reintroduces suite/product ambiguity.

Detection: platform in crate names, manifests, templates, or canonical docs
where shared is intended.

CI candidate: glossary lint with allowed exceptions.

Review cue: ask whether the thing is shared substrate, product, pack, or owner
indirection.

Safer replacement: use shared for cross-µservice utilities and specific
substrate names for owned planes.

Anchor: feedback_glossary_shared_not_platform references in templates and ADRs.

### AP-R23 Code Before Contract

Pattern ID: AP-R23.

Severity: P1; P0 for public APIs and cross-service calls.

Failure: implementation appears before OpenAPI, AsyncAPI, proto, schema, Cedar
policy, or event registry contract.

Why it harms Oyatie: API-first and clean architecture require contract review
before code hardens.

Detection: route or event code without matching contract file.

CI candidate: route/event/schema discovery against contracts.

Review cue: ask which contract a client would generate from.

Safer replacement: contract first, then implementation, then compatibility
tests.

Anchor: docs/standards/openapi-3-2-authoring.md and asyncapi-3-1-authoring.md.

### AP-R24 Editorial P0 Dismissed As "Only Docs"

Pattern ID: AP-R24.

Severity: P0.

Failure: a team accepts an architecture doctrine defect because no source code
changed.

Why it harms Oyatie: docs are executable planning surfaces for agents and
humans.

Detection: review says "non-blocking, docs only" while the doc authorizes
interfaces, topology, policy, or compliance.

CI candidate: doc class and authority tier drive severity mapping.

Review cue: ask whether code will be written from this document.

Safer replacement: block promotion until the doctrine is internally coherent
and buildable.

Anchor: Wave-3-G synthesis classified template-stamping as editorial P0.

## §4 Coordination + Process Anti-Patterns

### AP-C01 Claim Collision

Pattern ID: AP-C01.

Severity: P0 for concurrent work.

Failure: two agents edit the same claimed scope or file family without an Oya
VCS claim relationship.

Why it harms Oyatie: claim locks are the concurrency primitive during the
GitOps/VCS replacement cutover.

Detection: unclaimed edits under a path already held by another agent.

CI candidate: oya-vcs-admission concurrent-safe-paths gate.

Review cue: ask which changeset owns the path.

Safer replacement: claim before edit, split scope, or wait for promote.

Required lifecycle: claim, verify, done, promote.

Anchor: docs/AGENTS.md agent-instructions block.

### AP-C02 Skipping Verify, Done, Or Promote

Pattern ID: AP-C02.

Severity: P0.

Failure: a changeset stops after local edits or tests without completing Oya
VCS lifecycle transitions.

Why it harms Oyatie: unpromoted work cannot be trusted by downstream agents.

Detection: claim exists but no verify/done/promote evidence.

CI candidate: changeset state machine validator.

Review cue: ask which exact evidence string closed the scope.

Safer replacement: run verify with evidence, done with evidence, and promote
with bundle/environment/evidence.

Anchor: ADR-0116 and docs/AGENTS.md required_sequence.

### AP-C03 Manual Iteration On Mechanical Work

Pattern ID: AP-C03.

Severity: P1; P0 when the manual process is repeated at scale.

Failure: a contributor performs large mechanical sweeps by hand while the
project doctrine says mechanical work should be automated.

Why it harms Oyatie: hand sweeps produce omissions, inconsistent state, and no
replay path.

Detection: hundreds of similar edits with no command, generator, or validator.

CI candidate: generated or migration provenance manifest.

Review cue: decide whether the work is content judgment or mechanical
transformation.

Safer replacement: automate mechanical rewrites; hand-author the semantic
delta.

Boundary: this catalogue itself is hand-authored because the user explicitly
forbade scripting and requested bespoke content.

Anchor: feedback_automate_everything references in specs/agent-durable-goal.json.

### AP-C04 Codex Bulk-Resolve P2 Threads

Pattern ID: AP-C04.

Severity: P0 for review workflow.

Failure: Codex or another agent bulk-resolves review threads, especially P2
threads, without individual assessment.

Why it harms Oyatie: P2 findings are real defects until fixed or rebutted.

Detection: mass thread closure, `p2-only` resolve mode, or no per-thread
disposition.

CI candidate: review-thread sweep tool must be report-only unless each thread
has a linked fix/rebuttal.

Review cue: ask what evidence closed each thread.

Safer replacement: report unresolved threads; fix or rebut one by one.

Anchor: evidence/multispectrum/claude-codex-thread-sweep-tool-1779004400.json.

### AP-C05 Self-Merge Without Contract Path

Pattern ID: AP-C05.

Severity: P0.

Failure: an agent merges its own work on CI green alone.

Why it harms Oyatie: self-merge requires independent evidence, reviewer
verdict, Code Review section, and admission gate green.

Detection: merge event lacks review evidence or reviewer-agent verdict.

CI candidate: merge admission checks for evidence bundle.

Review cue: ask whether the self-merge contract path is complete.

Safer replacement: review evidence, reviewer-agent verdict, Code Review
section, admission gate green, then merge.

Anchor: feedback_self_merge_via_contract_path references in evidence/pr-143.

Anchor: ADR-0221 operational consequences.

### AP-C06 Touching Files Held By Another Agent Claim

Pattern ID: AP-C06.

Severity: P0 unless the owning agent widens scope.

Failure: an agent edits files inside another active claim because they are
nearby or convenient.

Why it harms Oyatie: claim collisions corrupt review ownership and changeset
evidence.

Detection: edit paths outside the claim scope.

CI candidate: VCS admission compares changed files to claim scope.

Review cue: ask whether the file belongs to the current changeset.

Safer replacement: open a separate claim or request scope transfer through the
leader.

Anchor: docs/AGENTS.md scaffold_protocol.

### AP-C07 Oya VCS CLI Parser Nuance Drift

Pattern ID: AP-C07.

Severity: P1; P0 when lifecycle commands fail.

Failure: contributors reuse stale CLI flags on the wrong subcommand.

Why it harms Oyatie: lifecycle evidence can fail or silently attach to the
wrong transition.

Detection: `--intent` passed to verify, `--evidence` omitted on done, or
`--agent` omitted on promote where required.

CI candidate: command examples in standards are tested.

Review cue: compare command syntax against the current instructions for the
task.

Safer replacement: treat claim/verify/done/promote as distinct command
contracts.

Current task example: verify uses `--evidence` and no `--intent`.

Anchor: user-specified lifecycle commands for this catalogue.

### AP-C08 Retired Primitive Revival

Pattern ID: AP-C08.

Severity: P0 for state transitions.

authority after ADR-0116 retirement.

Why it harms Oyatie: retired primitives are compatibility/provenance surfaces,
not promotion authority.

Detection: forbidden primitive names in agent instructions or lifecycle docs.

CI candidate: forbidden primitive string scan with allowed prose exceptions.

Review cue: ask whether the primitive changes repo state or merely provides
read/provenance context.

Safer replacement: oya vcs for claim/verify/done/promote; oya git for git
drop-in surface.

Anchor: docs/decisions/ADR-0709-general-live-apex.md.

### AP-C09 Evidence After Promotion

Pattern ID: AP-C09.

Severity: P0.

Failure: work is promoted first and evidence is assembled later.

Why it harms Oyatie: admission must verify the change before downstream agents
consume it.

Detection: promote timestamp precedes verify evidence.

CI candidate: changeset ledger ordering check.

Review cue: ask which validation ran before the transition.

Safer replacement: collect evidence, verify, done, then promote.

Anchor: Oya VCS required sequence in docs/AGENTS.md.

### AP-C10 Writer And Reviewer Lens Collapse

Pattern ID: AP-C10.

Severity: P1.

Failure: the same agent writes, reviews, rebuts, and accepts the same concern
without independent lens separation.

Why it harms Oyatie: bias-collapse suppresses uncomfortable findings.

Detection: review evidence lacks independent facet identities.

Review cue: ask whether independent review facets were actually separated.

Safer replacement: dispatch required facets or record why the change class
does not require them.

### AP-C11 Broad Claim Scope

Pattern ID: AP-C11.

Severity: P1; P0 when it blocks parallel work.

Failure: an agent claims a large directory when only one file or small family
is needed.

Why it harms Oyatie: broad claims serialize unrelated work.

Detection: claim scope contains many files untouched by the changeset.

CI candidate: claim-to-diff ratio warning.

Review cue: ask whether the claim could have been file-specific.

Safer replacement: claim the narrowest scope that protects the edit.

Boundary: a standards-directory claim can be appropriate when the command
requires `docs/standards` and the edit is a new standard file.

Anchor: Oya VCS claim lifecycle.

### AP-C12 Status Transition Without Bundle

Pattern ID: AP-C12.

Severity: P0 for promoted work.

Failure: a changeset changes status or claims promotion without bundle name,
environment, and evidence.

Why it harms Oyatie: promotion must be traceable to a bundle and environment.

Detection: promote command lacks bundle or environment.

CI candidate: promote parser and ledger schema.

Review cue: ask what bundle downstream agents should cite.

Safer replacement: include `--bundle`, `--environment`, and evidence in promote.

Anchor: user-specified promote command shape for this catalogue.

### AP-C13 Direct Git/GitHub Transition Bypass

Pattern ID: AP-C13.

Severity: P0 when used for repo state transition.

Failure: direct git or gh commands are used as the authoritative state
transition while Oya VCS owns that lifecycle.

Why it harms Oyatie: admission, evidence, claim locks, and promotion metadata
are bypassed.

Detection: branch/PR/merge state changes without Oya VCS ledger events.

CI candidate: admission gate checks changeset evidence.

Review cue: distinguish local inspection from authoritative transition.

Safer replacement: Oya VCS lifecycle first; git commands only inside sanctioned
or drop-in surfaces.

Anchor: ADR-0116.

### AP-C14 Missing Checkpoint On Halt

Pattern ID: AP-C14.

Severity: P1; P0 when a partially complete changeset is left ambiguous.

Failure: work halts without a clear checkpoint, remaining scope, and evidence
state.

Why it harms Oyatie: future agents cannot resume safely.

Detection: final report omits changed files, validations, lifecycle state, or
blocker.

CI candidate: hard to enforce; lifecycle state can flag open claims.

Review cue: ask what the next agent should do first.

Safer replacement: document checkpoint with complete, remaining, evidence, and
stop reason.

Anchor: user instruction "HALT CLEANLY; document checkpoint."

### AP-C15 Parallel Agents On Shared Write File

Pattern ID: AP-C15.

Severity: P1; P0 if conflicts are likely.

Failure: multiple agents edit the same canonical file or section without an
integration owner.

Why it harms Oyatie: parallelism helps only when write sets are disjoint.

Detection: same path assigned to more than one child lane.

CI candidate: team task write-scope validation.

Review cue: ask who owns final merge and conflict resolution.

Safer replacement: parallelize research or disjoint file edits; keep one owner
for a single canonical file.

Anchor: docs/AGENTS.md child_agent_protocol.

### AP-C16 Hidden Unresolved Review Thread

Pattern ID: AP-C16.

Severity: P0 for merge readiness.

Failure: a final report says green while unresolved review threads still exist.

Why it harms Oyatie: unresolved threads are explicit blockers or required
rebuttals.

Detection: GitHub reviewThreads unresolved count > 0.

CI candidate: merge queue check.

Review cue: ask for unresolved thread list before merge.

Safer replacement: close every thread by fix, evidence-backed rebuttal, or
accepted follow-up.

Anchor: ADR-0124 own merge queue webhook driven.

### AP-C17 "Read-Only" While Writing Artifacts

Pattern ID: AP-C17.

Severity: P1; P0 if audit claims are affected.

Failure: a task is described as read-only while it creates evidence, reports,
or generated files.

Why it harms Oyatie: review and claim scopes depend on truthful side-effect
classification.

Detection: new files after a "read-only" task.

CI candidate: command wrapper side-effect detector.

Review cue: ask whether any filesystem or external state changed.

Safer replacement: call it audit-only only when no writes occur; otherwise
claim scope and run lifecycle.

Anchor: Oya VCS transition rules.

### AP-C18 Destructive Cleanup Of User Changes

Pattern ID: AP-C18.

Severity: P0.

Failure: an agent reverts, deletes, or overwrites changes it did not make.

Why it harms Oyatie: shared workspace safety depends on preserving other
contributors' work.

Detection: reset/checkout/remove commands touching unrelated dirty files.

CI candidate: limited; local agent policy and review.

Review cue: inspect git status before and after.

Safer replacement: isolate your edits, ignore unrelated changes, and ask only
when conflict makes the task impossible.

Anchor: shared workspace edit constraints.

## §5 Linguistic + Reference Anti-Patterns

### AP-L01 Vague "Handles X" Phrasing

Pattern ID: AP-L01.

Severity: P1; P0 when it defines ownership.

Failure: "the µservice handles X" appears without named objects, actions, and
contracts.

Why it harms Oyatie: ownership boundaries become oral tradition.

Detection: handle/manage/support/process verbs near no schema or event names.

CI candidate: risky-verb lint.

Review cue: ask what object is written, read, emitted, or refused.

Safer replacement: name owner, entity, command, event, policy, and rollback.

Anchor: docs/standards/documentation-rigor.md.

### AP-L02 Missing Version Numbers

Pattern ID: AP-L02.

Severity: P1; P0 for protocol, API, Kubernetes, Cedar, OpenAPI, AsyncAPI, or
regulatory claims.

Failure: docs cite a technology or law without version or effective date.

Why it harms Oyatie: version drift changes semantics.

Detection: "Kubernetes 1.x", "Cedar", "OpenAPI", "PCI", or "EU AI Act" with
no version/effective-date where needed.

CI candidate: known-version lint.

Review cue: ask whether the standard could have changed.

Safer replacement: Kubernetes 1.35 LTS, Cedar v4.2 LTS, OpenAPI 3.2.0,
AsyncAPI 3.1.0, PCI DSS 4.0.1, or exact regulation date.

Anchor: docs/standards/lts-versions-verified.md.

### AP-L03 Non-Existent ADR Citation

Pattern ID: AP-L03.

Severity: P0 for canonical docs.

Failure: a document cites an ADR identifier or slug that does not exist.

Why it harms Oyatie: fake authority is worse than no authority because agents
trust it.

Detection: ADR token cannot resolve to docs/decisions path or index row.

CI candidate: ADR link resolver.

Review cue: run `rg --files docs/decisions | rg 'ADR-XXXX'`.

Safer replacement: verify the ADR path before citing and use exact slug where
possible.

Anchor: docs/standards/documentation-rigor.md §3.

### AP-L04 Broken Cross-Link

Pattern ID: AP-L04.

Severity: P1; P0 when the link is the only path to a required primitive.

Failure: Markdown links, frontmatter companion docs, or spec refs point to
missing files or stale paths.

Why it harms Oyatie: graph traversal and cold-start onboarding fail.

Detection: dead relative path, moved spec path, or renamed ADR slug.

CI candidate: link resolver over docs, specs, microservices, packs, and
crates docs.

Review cue: follow the link locally.

Safer replacement: update the link and, for moved canonical docs, leave a
redirect or catalog entry.

Anchor: docs/standards/documentation-rigor.md six-hops invariant.

### AP-L05 Shared Versus Platform Drift

Pattern ID: AP-L05.

Severity: P2; P1 for names.

Failure: "platform" and "shared" are mixed as if interchangeable.

Why it harms Oyatie: shared substrate and product ownership are separate
concepts.

Detection: platform in crate prefix, shared service docs, or template notes
where doctrine says shared.

CI candidate: glossary vocabulary lane.

Review cue: ask whether the thing is shared substrate, product, owner account,
or deployment platform.

Safer replacement: use "shared" for cross-µservice utility surfaces unless an
ADR authorizes different wording.

Anchor: feedback_glossary_shared_not_platform references in docs/templates.

### AP-L06 Wrong CLI Primitive

Pattern ID: AP-L06.

Severity: P0 in agent instructions and lifecycle docs.

git is canonical.

Why it harms Oyatie: stale instructions cause agents to bypass gates.

Detection: retired primitive appears in actionable command blocks.

CI candidate: command-block primitive lint.

Review cue: distinguish prose history from current instruction.

Safer replacement: `oya vcs` for lifecycle; `oya git` for git drop-in.

Anchor: ADR-0116.

### AP-L07 Acronym Soup Without Glossary

Pattern ID: AP-L07.

Severity: P2; P1 for standards and onboarding.

Failure: dense acronyms appear without expansion, glossary link, or authority.

Why it harms Oyatie: intern-buildability fails even when the architecture is
sound.

Detection: uppercase tokens repeated without definition.

CI candidate: vocabulary warning source registry.

Review cue: ask whether a new engineer can decode the first occurrence.

Safer replacement: expand at first use and link glossary or ADR.

Anchor: registry/glossary-vocabulary/warning-sources.tsv.

### AP-L08 "Canonical" Without Authority

Pattern ID: AP-L08.

Severity: P1; P0 when it conflicts with an existing canonical surface.

Failure: a document declares something canonical without citing the authority
chain.

Why it harms Oyatie: competing canonical claims split agent behavior.

Detection: canonical, source of truth, authoritative, or accepted without
frontmatter authority.

CI candidate: canonical-language lint requiring authority fields.

Review cue: ask which spec, ADR, or standard grants canonical status.

Safer replacement: state the authority path and what scope it covers.

Anchor: specs/root-hub-pointers.json.

### AP-L09 Severity Laundering

Pattern ID: AP-L09.

Severity: P1; P0 if blockers are downgraded.

Failure: a P0/P1 issue is described with softer wording such as concern,
nit, improvement, or follow-up without preserving severity.

Why it harms Oyatie: scheduling and promotion gates depend on severity.

Detection: finding body says blocks but severity says P2 or note.

CI candidate: limited; severity lexicon consistency check.

Review cue: ask what happens if the issue ships unchanged.

Safer replacement: keep severity explicit and separate urgency from tone.

Anchor: Wave-3-G synthesis severity table.

### AP-L10 Ambiguous Done

Pattern ID: AP-L10.

Severity: P1; P0 for lifecycle closeout.

Failure: "done" is used without saying whether code, docs, tests, Oya VCS
done, or promotion is meant.

Why it harms Oyatie: the word done spans local edit state and official
changeset state.

Detection: final report lacks lifecycle stage.

CI candidate: final evidence schema.

Review cue: ask "done in which state machine?"

Safer replacement: say "file authored", "tests passed", "oya vcs done
accepted", or "promoted to dev."

Anchor: Oya VCS lifecycle contract.

### AP-L11 Optional Language In Mandatory Lane

Pattern ID: AP-L11.

Severity: P1; P0 when it weakens a compliance or security control.

Failure: mandatory behavior is described as optional, best-effort, or
recommended.

Why it harms Oyatie: weak language gives future authors permission to skip a
control.

Detection: SHOULD/MAY near controls that the binding ADR marks MUST.

CI candidate: normative keyword comparison against source ADR.

Review cue: trace the strongest authority for the behavior.

Safer replacement: use RFC-2119 keyword matching the source authority and
state exceptions explicitly.

Anchor: docs/standards/doc-style.md §2.

### AP-L12 Ownerless Future Work

Pattern ID: AP-L12.

Severity: P1.

Failure: future work is listed without owner, target artifact, date, or gate.

Why it harms Oyatie: ownerless future work becomes invisible debt.

Detection: "future work" section lacks owner/status/target.

CI candidate: future-work table schema.

Review cue: ask who is accountable and when promotion is blocked.

Safer replacement: create a successor IP, issue, lane, or changeset entry.

Anchor: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §30.1.

### AP-L13 Invented Hyperscaler Parallel

Pattern ID: AP-L13.

Severity: P1; P0 when used as decision proof.

Failure: a document says "like AWS/GCP/Stripe" for a pattern they do not
actually use.

Why it harms Oyatie: incorrect precedent misleads architecture review.

Detection: named company without product, paper, public doc, or exact pattern.

CI candidate: none reliable; review-caught.

Review cue: ask for source and shape match.

Safer replacement: cite the actual precedent or remove the analogy.

Anchor: documentation-rigor.md named precedent requirement.

### AP-L14 BYOK Without Qualifier

Pattern ID: AP-L14.

Severity: P0 for security/compliance/intelligence surfaces.

Failure: BYOK appears alone in a claim, title, acceptance criterion, or schema.

Why it harms Oyatie: provider credentials and encryption keys are governed by
different ADRs.

Detection: raw "BYOK" token not followed by provider or encryption.

CI candidate: BYOK qualifier lint.

Review cue: ask "which key?"

Safer replacement: provider-BYOK or encryption-BYOK, with ADR section.

Anchor: ADR-0255 §D-4 and ADR-0251 §D-10.

### AP-L15 Tenant Versus Customer Conflation

Pattern ID: AP-L15.

Severity: P1; P0 for policy or data models.

Failure: customer, account, org, workspace, and tenant are used as synonyms.

Why it harms Oyatie: tenant_id is the universal scoping primitive; customers
may own many tenants or sub-scopes.

Detection: schema field customer_id used for access control without tenant_id.

CI candidate: tenant-bearing schema validator.

Review cue: ask which tenant owns the data and which customer relationship is
business context only.

Safer replacement: tenant_id for isolation, customer/account fields for
commercial relationships.

Anchor: ADR-0244.

### AP-L16 Microservice Versus Bounded Context Drift

Pattern ID: AP-L16.

Severity: P2; P1 when it affects crate placement.

Failure: docs call a bounded context, crate, worker, or feature a µservice.

Why it harms Oyatie: flat per-µservice layout depends on accurate hierarchy.

Detection: service list grows because every BC is counted as a µservice.

CI candidate: manifest microservice roster validation.

Review cue: ask whether it has an independent PRD, manifest, contracts, and
deployment boundary.

Safer replacement: name microservice, bounded context, crate, worker, route,
and capability separately.

Anchor: ADR-0131 per-microservice flat layout.

## §6 Detection + CI Enforcement

### §6.1 Enforcement Philosophy

The enforcement goal is prevention, not punishment.

Every anti-pattern that can be detected mechanically SHOULD become an
oya-governance-* lane.

Every lane SHOULD report the anti-pattern ID.

Every lane SHOULD include a false-positive escape path that requires an
authority citation.

Every lane SHOULD start advisory before blocker unless the risk is already
P0 and high-confidence.

Review-caught patterns remain valid even before a lane exists.

The absence of a lane is not permission to repeat a known incident.

### §6.2 Candidate Lanes

`oya-governance-template-stamp-detect` detects AP-A01, AP-A08, and AP-A09.

It compares repeated sections after normalizing entity names, IDs, tiers, and
service slugs.

It reports similarity clusters and asks for either bespoke deltas or explicit
scaffold tagging.

`oya-governance-clause-loop-detect` detects AP-A02 and AP-A07.

It counts repeated clause labels and unique paragraph bodies.

It fails when the ratio of labels to unique bodies crosses a configured
threshold.

`oya-governance-doc-density` detects AP-A03, AP-A15, and AP-A16.

It maps doc class to required anchors from documentation-rigor.md §2.

It refuses line-floor claims without density anchors.

`oya-governance-generated-provenance` detects AP-A04 and AP-A23.

It requires generated files to declare generator, input manifest, and promotion
status.

It refuses generated placeholders from serving as canonical accepted doctrine.

`oya-governance-reference-resolve` detects AP-A06, AP-A14, AP-L03, and AP-L04.

It resolves ADR IDs, companion docs, spec refs, and Markdown links.

It emits broken edge reports for six-hop traversal.

`oya-governance-vague-verb-lint` detects AP-A05 and AP-L01.

It flags high-risk verbs unless nearby text names owner, object, action,
policy, event, and contract.

`oya-governance-future-work-owned` detects AP-A24 and AP-L12.

It requires future-work rows to include owner, target artifact, status, and
blocking condition.

`oya-governance-adr-status-coherence` detects AP-A12.

It cross-checks frontmatter status with synthesis reports, promotion ledgers,
and accepted enum values.

`oya-governance-interservice-contract` detects AP-R01, AP-R02, AP-R06, AP-R13,
AP-R16, and AP-R17.

It validates contracts, audit emission owner, trace propagation, and Workflow
or Ontology reasons.

`oya-governance-cedar-fragment-discipline` detects AP-R03 and AP-R05.

It checks Cedar fragment count, schema registration, and audit-event class
registration.

`oya-governance-tenant-scope-propagation` detects AP-R08, AP-L15, and related
schema drift.

It checks tenant_id and sub_scope_path in tenant-bearing persistence,
telemetry, events, and policy context.

`oya-governance-byok-terminology` detects AP-R09 and AP-L14.

It refuses unqualified BYOK in canonical docs and schemas.

`oya-governance-consistency-tier` detects AP-R07 and AP-R20.

It requires every multi-region replicated state primitive to choose HLC default
or TrueTime tier.

`oya-governance-vcs-lifecycle` detects AP-C01, AP-C02, AP-C06, AP-C09,
AP-C11, and AP-C12.

It checks claim scope, changed files, verify evidence, done state, promotion
bundle, and environment.

`oya-governance-retired-primitive-lint` detects AP-C08 and AP-L06.

It flags retired command primitives in actionable command blocks.

`oya-governance-review-thread-discipline` detects AP-C04 and AP-C16.

It refuses bulk-resolution and requires per-thread disposition.

`oya-governance-self-merge-contract` detects AP-C05.

It requires review evidence, reviewer-agent verdict, Code Review
section, and admission gate green before self-merge.

`oya-governance-frontmatter-authority` detects AP-A20, AP-A21, and AP-L08.

It requires enforcement fields, authority chain, companion docs, and maintenance
metadata.

`oya-governance-version-specificity` detects AP-L02.

It checks known protocol and regulation tokens for version or effective-date
qualifiers.

`oya-governance-glossary-vocabulary` detects AP-L05, AP-L07, AP-L15, and
AP-L16.

It compares canonical terms to glossary and vocabulary warning registries.

### §6.3 Review-Only Patterns

AP-A10 and AP-A11 are primarily review-caught.

They require comparing requested ambition to delivered scope.

AP-A17 is primarily review-caught.

It requires reading whether normative corrections are hidden in appendices.

AP-A19 and AP-L13 are primarily review-caught.

They require judgment about whether a cited precedent actually maps.

AP-R12, AP-R18, AP-R19, AP-R21, AP-R22, and AP-R24 need mixed review and CI.

The CI lanes can detect missing fields.

Humans or reviewer agents decide whether the architecture claim is honest.

AP-C10 and AP-C15 are workflow-design review items.

They require checking role separation and write-scope disjointness.

AP-C14 and AP-C17 are closeout truthfulness checks.

They become mechanical only when final-report schemas are wired.

AP-C18 is local workspace safety.

It remains a human/agent policy invariant even when no CI sees the destructive
command.

### §6.4 Evidence Format

Every anti-pattern finding SHOULD include the following fields.

Field: pattern_id.

Field: severity.

Field: file_path.

Field: source_line or section.

Field: detection_mode.

Field: why_it_matters.

Field: required_fix.

Field: validation_command or review_evidence.

Field: disposition.

Field: owner.

Field: due_gate.

Machine-readable evidence SHOULD use JSON.

Markdown review comments SHOULD still include those fields in prose.

Promotion evidence SHOULD include counts such as `patterns_catalogued:N` only
after the file itself supplies the concrete rows.

### §6.5 False Positive Handling

A repeated structure is not automatically template-stamping.

A catalogue can use a stable row schema when every row carries distinct risk,
detection, replacement, and authority.

A generated scaffold is not automatically forbidden.

It is forbidden when it is promoted as canonical completion without bespoke
content.

Broad claims are not automatically wrong.

They are wrong when they exceed the evidence or erase known open blockers.

Direct service calls are not automatically coupling.

They are valid after ADR-0145 when contract, audit, and trace invariants hold.

Workflow usage is not automatically over-orchestration.

It is valid for durable sagas, human approvals, retry-with-backoff, temporal
work, and compensation.

Future work is not automatically stealth deferral.

It is valid when owner, artifact, acceptance gate, and sequencing impact are
explicit.

## §7 Worked Examples — Real Wave-3-G Cases

### §7.1 ADR-0321 Vendor Dossier Template-Stamping

Incident class: AP-A01, AP-A08, AP-A12, AP-R24.

Source: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §6.1.

Source: docs/decisions/ADR-0709-general-live-apex.md §D.

The audit found 165 vendor dossiers.

The sampled D-001 through D-013 rows shared the same Cedar permit shape,
ontology projection, workflow template, UX shell, pack overlay, migration, and
failure-mode sentences.

Only vendor name, tier, and destination changed.

This is not merely a prose preference.

The difference between MuleSoft, Tableau, Snowflake, GitHub, Workday, and
ServiceNow changes auth, data model, source API, rate limit behavior,
migration risk, UX surface, and compliance evidence.

The artifact looked like coverage.

It did not give enough vendor-specific content to build.

Severity is P0 because ADR-0321 is load-bearing doctrine.

Safe rewrite pattern: keep a shared dossier macro as an appendix.

Safe rewrite pattern: each vendor row carries a delta with source API,
canonical objects, Cedar verbs, workflow templates, failure modes, migration
constraints, and pack overlays.

Safe rewrite pattern: rows that are not yet rewritten are tagged as generated
templates and do not count as full coverage.

Review question: what can an engineer implement for this vendor that would not
be identical for the previous vendor?

CI proposal: normalize vendor names and compare paragraph shingles.

Stop condition: no vendor dossier can be reduced to another row by replacing
names and tiers.

### §7.2 Unified Ecosystem Thesis Clause Loop

Incident class: AP-A02, AP-A03, AP-A07, AP-A21.

Source: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §6.2.

Source: docs/architecture/unified-ecosystem-thesis-2026-05-21.md.

The audit found 700 "Thesis clause N" rows over 10 distinct invariants.

That is roughly 70 rows per invariant.

The count created the appearance of exhaustive doctrine.

The repeated body did not add file paths, diagrams, equations, authority
edges, or new implementation constraints.

The failure is especially dangerous because the title implies a unifying
architecture thesis.

Readers expect compression and clarity from a thesis.

They instead receive a clause loop.

Severity is P0 because the document fails the ArchitectureDeepDive density
bar.

Safe rewrite pattern: one section per invariant.

Safe rewrite pattern: each section contains a named precedent, architecture
boundary, concrete service trace, failure mode, and "what this forbids" list.

Safe rewrite pattern: remove clause-count theatrics and replace with traceable
examples.

Review question: can a reader name the 10 invariants without wading through
700 rows?

CI proposal: count repeated clause labels and unique normalized bodies.

Stop condition: each invariant block carries a distinct implementation
consequence.

### §7.3 Training Cost Doctrine Problem Clause Loop

Incident class: AP-A02, AP-A03, AP-A07, AP-L09.

Source: docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md §6.3.

Source: docs/architecture/training-cost-doctrine-2026-05-21.md.

The audit found 160 "Problem clause N" rows in §1.

Each row repeated the same problem statement, evidence, and consequence shape.

The document topic demanded a model.

It needed per-collar-color, per-skill-tier, per-career-stage, and per-training
path cost numbers.

Instead, the document spent early density on repeated labels.

The anti-pattern is not that the document was long.

The anti-pattern is that the most important section delayed the real model.

Severity is P0 because the problem definition failed the buildability and
architecture-deep-dive density bar.

Safe rewrite pattern: one problem statement.

Safe rewrite pattern: one cost model with assumptions, equations, ranges,
examples, failure modes, and sensitivity analysis.

Safe rewrite pattern: link each training claim to a workforce, compliance, or
operational mechanism.

Review question: what number, formula, or decision changes after reading this
row?

CI proposal: repeated-problem-clause detector.

Stop condition: no repeated problem row exists unless it adds a distinct
variable or consequence.

### §7.4 ERP IP 80-Line Shallow Generation Attempt

Incident class: AP-A04, AP-A15, AP-A16, AP-A23.

Source: retired ERP second-pass generator (removed from the live tree).

The script includes an IP generator that builds a base implementation-plan
body and pads it to 80 lines with repeated "IP detail" rows.

The repeated row names tenant_id, data_class, source_system_id,
policy_bundle_version, audit_event_class, residency_pack, ECH/PQC, and
rollback path.

Those fields matter.

Repeating them does not make the IP buildable.

An IP needs actual files, dependencies, sequencing, acceptance commands, and
domain-specific state.

This is the exact difference between scaffold and canonical plan.

The script can be useful as a scaffold.

It becomes harmful if its output is counted as finished doctrine.

Severity is P0 when promoted as canonical coverage.

Severity is P1 when clearly tagged as scaffold awaiting bespoke enrichment.

Safe rewrite pattern: generated skeletons carry provenance and blocked status.

Safe rewrite pattern: humans or agents add bounded-context-specific content
before promotion.

Review question: what would fail if the target bounded context changed from
inventory to procurement?

CI proposal: generated-provenance lane plus IP density lane.

Stop condition: every IP has file ownership, dependency order, specific tests,
and rollback path.

### §7.5 Conservative Re-Scoping Versus Original Ambition

Incident class: AP-A10, AP-A11, AP-A24, AP-L10.

Source: specs/masterplan.json.

Source: planning-closure references in root-hub-pointers.json.

Oyatie's current execution posture rejects minimum viable product, preview, reduced-scope,
placeholder, stub, thin scaffold, and deferred-scope claims for masterplan
delivery.

The failure mode is subtle.

An agent can produce a useful partial artifact.

The partial artifact can even be high quality.

But if the user asked for complete canonical coverage, reporting the partial
artifact as completion is false readiness.

The safe move is sequencing, not shrinkage.

Large ambition should become dependency-aware slices with gates.

It should not become "representative examples" unless the user asked for
examples.

Severity is P0 when original ambition is the acceptance surface.

Safe rewrite pattern: preserve full scope in a map.

Safe rewrite pattern: close slices only after evidence.

Safe rewrite pattern: declare blockers and remaining scope at halt.

Review question: did the delivered scope match the user's noun phrase?

CI proposal: hard; mostly plan/claim review.

Stop condition: final checkpoint names full scope complete or names remaining
scope honestly.

## §8 Quick-Reference Decision Tree

Start here when reviewing or authoring.

Question 1: Does the artifact claim canonical authority?

If yes, check AP-A06, AP-L08, AP-A12, and AP-A20 first.

Question 2: Does it meet a line floor or artifact count?

If yes, check AP-A03, AP-A07, AP-A15, AP-A16, and AP-A23.

Question 3: Does it repeat rows across vendors, services, clauses, or IPs?

If yes, check AP-A01, AP-A02, AP-A08, AP-A09, and AP-A04.

Question 4: Does it introduce or modify architecture boundaries?

If yes, check AP-R01 through AP-R08 first.

Question 5: Does it touch regulated data, credentials, tenancy, or audit?

If yes, check AP-R05, AP-R08, AP-R09, AP-R10, AP-R14, AP-R21, AP-L14, and
AP-L15.

Question 6: Does it create a cross-service call?

If yes, ask whether it is direct request/response, async event, Workflow saga,
or Ontology projection.

Then check AP-R01, AP-R02, AP-R06, AP-R13, AP-R16, and AP-R17.

Question 7: Does it mention multi-region or global state?

If yes, check AP-R07, AP-R20, and AP-R21.

Question 8: Does it use Oya VCS or repo lifecycle commands?

If yes, check AP-C01, AP-C02, AP-C06, AP-C07, AP-C08, AP-C09, AP-C12, and
AP-C13.

Question 9: Does it close review threads or merge work?

If yes, check AP-C04, AP-C05, AP-C10, and AP-C16.

Question 10: Does it say "done"?

If yes, check AP-L10 and demand the state machine.

Question 11: Does it say "BYOK"?

If yes, check AP-R09 and AP-L14.

Question 12: Does it say "platform"?

If yes, check AP-R22 and AP-L05.

Question 13: Does it cite an ADR?

If yes, verify the ADR exists and that the cited section says what the author
claims.

Question 14: Does it cite a hyperscaler?

If yes, check AP-A19 and AP-L13.

Question 15: Does it defer work?

If yes, check AP-A24 and AP-L12.

Question 16: Does it halt?

If yes, require checkpoint, evidence, lifecycle state, and remaining scope.

Question 17: Is the change "docs only"?

If yes, still check authority tier.

Docs that authorize architecture can be P0.

Question 18: Is a pattern detection false positive?

If yes, document the authority, why the safe replacement is unnecessary, and
what evidence prevents the incident from recurring.

## §9 Cross-References

### §9.1 Standards

Documentation rigor: docs/standards/documentation-rigor.md.

Doc style: docs/standards/doc-style.md.

Agent instructions discipline: docs/standards/agent-instructions-discipline.md.

OpenAPI authoring: docs/standards/openapi-3-2-authoring.md.

AsyncAPI authoring: docs/standards/asyncapi-3-1-authoring.md.

Idempotency keys: docs/standards/idempotency-keys-canonical.md.

Outbox pattern: docs/standards/outbox-pattern-canonical.md.

Oya standards index: docs/standards/INDEX.md.

### §9.2 Architecture and Audit Sources

Wave-3-G synthesis adjudication:
docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md.

Keystone bundle synthesis:
docs/architecture/keystone-bundle-2026-05-20-synthesis.md.

Corpus rigor audit:
docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md.

Training cost doctrine:
docs/architecture/training-cost-doctrine-2026-05-21.md.

Unified ecosystem thesis:
docs/architecture/unified-ecosystem-thesis-2026-05-21.md.

ERP second-pass generator:
retired from the live tree; see git history before removal if historical provenance is needed.

### §9.3 ADRs

ADR-0064 canonical base and localization packs:
docs/decisions/ADR-0709-general-live-apex.md.

ADR-0116 retired external coordination tooling:
docs/decisions/ADR-0709-general-live-apex.md.

ADR-0131 per-microservice flat layout:
docs/decisions/ADR-0701-monorepo-capability-live-apex.md.

ADR-0145 inter-microservice communication reform:
docs/decisions/ADR-0701-monorepo-capability-live-apex.md.

ADR-0221 agentic development pipeline hardening:
docs/decisions/ADR-0709-general-live-apex.md.

ADR-0243 Cedar as universal gate:
docs/decisions/ADR-0700-ci-admission-live-apex.md.

ADR-0244 tenant as universal scoping primitive:
docs/decisions/ADR-0702-identity-authz-live-apex.md.

ADR-0251 compliance pack and encryption-BYOK:
docs/decisions/ADR-0708-platform-foundations-live-apex.md.

ADR-0252 time coordination:
docs/decisions/ADR-0709-general-live-apex.md.

ADR-0255 intelligence substrate and provider-BYOK:
docs/decisions/ADR-0701-monorepo-capability-live-apex.md.

ADR-0255 amendment library-first:
docs/decisions/ADR-0709-general-live-apex.md.

ADR-0263 observability emission contract:
docs/decisions/ADR-0706-observability-live-apex.md.

ADR-0321 B2B SaaS industry-leader coverage:
docs/decisions/ADR-0709-general-live-apex.md.

### §9.4 Machine-Readable Specs and Registries

Root hub pointers: specs/root-hub-pointers.json.

Masterplan: specs/masterplan.json.

Capability tier schema: specs/capability-tier-schema.json.

Tenant model: specs/tenant-model.json.

Sovereign cloud overlays: specs/sovereign-cloud-overlays.json.

Microservice migration tooling: specs/microservice-migration-tooling.json.

Glossary vocabulary warning sources:
registry/glossary-vocabulary/warning-sources.tsv.

Capability tiers registry: registry/capability-tiers/.

### §9.5 Operational Closeout

This catalogue is complete when the following are true.

The file exists at docs/standards/anti-patterns.md.

The file contains at least 1,500 lines.

The file contains at least 15 authoring anti-patterns.

The file contains at least 15 architecture anti-patterns.

The file contains at least 10 coordination/process anti-patterns.

The file contains at least 10 linguistic/reference anti-patterns.

The file cites Wave-3-G template-stamping and clause-loop incidents.

The file cites the ERP 80-line shallow-IP generation signal.

The file includes detection and CI enforcement guidance.

The file includes a quick-reference decision tree.

The Oya VCS claim, verify, done, and promote lifecycle closes with evidence.

No other canonical standards file is modified by this changeset.

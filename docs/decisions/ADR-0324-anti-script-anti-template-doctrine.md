---
id: ADR-0324
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-engineering
  - council-quality
  - council-foundry
  - council-documentation
  - axis-policy-engine
  - axis-workflow-engine
  - axis-foundry
  - ops-compliance
  - ops-program-management
supersedes: []
amends:
  - ADR-0091-multispectrum-review-doctrine.md (formalises template-stamping as a multispectrum BLOCKER outcome)
  - ADR-0132-product-platform-and-bundle-dissolution.md (companion ban on lambda-wrap suite-style enumeration)
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md (extends substance bar with named anti-pattern catalog)
  - ADR-0323-multi-wave-sequencing-doctrine.md (companion ban on wave-scale fan-out scripting)
superseded_by: []
related:
  - ADR-0063
  - ADR-0091
  - ADR-0105
  - ADR-0130
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0263
  - ADR-0316
  - ADR-0321
  - ADR-0322
  - ADR-0323
  - ADR-0327
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/substance-bar/template-allowlist.yaml
  - /specs/anti-pattern-catalog.json
  - /specs/wave-sequencing-schema.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/feedback/feedback_docs_substance_not_scaffold_2026_05_20.md
  - docs/feedback/feedback_go_with_original_ambition_2026_05_20.md
  - docs/postmortems/postmortem-codex-erp-ip-w2-lambda-wrap-2026-05-18.md
inbound_citations:
  - docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - docs/decisions/ADR-0323-multi-wave-sequencing-doctrine.md
  - docs/feedback/feedback_go_with_original_ambition_2026_05_20.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
purpose: >
  Name the anti-patterns that arose during the Wave-3-G + remediation
  experience and forbid them as doctrine. The doctrine forbids scripting
  or metaprogramming the bodies of substantive content artifacts (ADRs,
  journey docs, IP slices, microservice READMEs, PRDs, RFCs, specs) and
  forbids the introduction of helper template engines, lambda-wrap shell
  loops, jq-bodies-into-files invocations, or any other mechanism that
  produces N artifacts whose bodies differ only by a small substitution.
  The ban applies regardless of how clever the scripting is and regardless
  of whether substance-bar checks (per ADR-0322) would catch the output;
  the rationale is that even "clever-enough-to-pass" scripts erode the
  per-artifact authoring discipline that the substance bar relies on.
enforcement_status: blocker-day-one
enforced_by:
  - oya-governance-anti-script-anti-template
  - oya-governance-tool-invocation-attestation
  - oya-governance-content-authorship-provenance
  - oya-governance-loop-detector
  - oya-governance-no-template-stamping
decision_owner: council-quality
---

# ADR-0324: Anti-Script Anti-Template Doctrine

## Status

Proposed (2026-05-20). Applies to all in-scope content (per ADR-0322 scope)
authored on or after the publication date.

## Context

### Named pressure

The codex-erp-ip-w2 incident on 2026-05-18 ("lambda-wrap failure") is the
proximate trigger. An agent on the codex worker fleet wrote a bash loop
that fed a constant body template through jq with a single substitution
per file (the artifact slug), producing 18 IP slice files whose bodies were
identical modulo one heading line. The output cleared the lean-a4-structure
and lean-a5-doc-coverage lanes that existed at the time because the shape
was correct. It did not clear the multispectrum F4-substance facet, but the
facet was advisory rather than BLOCKER, so the PR merged. Subsequent
remediation cost three agents two days and the full rewrite required ADRs
0322, 0323 (this companion ADR), and 0327 to land before the remediation
wave could be opened.

The directive captured in `feedback_go_with_original_ambition_2026_05_20.md`
is explicit: agents must not script substantive content. The substance bar
(ADR-0322) is necessary but not sufficient — even a clever script that
sidesteps the bespoke-ratio check undermines per-artifact discipline. This
ADR converts the directive into doctrine by naming the anti-patterns and
the CI mechanisms that detect them.

### Named constraints

- **C-1 Substance-bar synergy** — anti-pattern detection complements
  bespoke-ratio detection in ADR-0322; the two crates share corpora and
  cross-feed evidence.
- **C-2 Provenance-first** — every content artifact authored under this
  doctrine carries a provenance record naming the tool chain used; this
  enables forensic detection of script-based authoring.
- **C-3 Authority chain** — per ADR-0246, provenance records are
  attestation-chain entries and carry the canonical envelope.
- **C-4 Tenancy** — per ADR-0244, every provenance entry carries
  `tenant_id=oyatie.governance`.
- **C-5 Foundry interaction** — per `feedback_foundry_pipeline_canonical.md`,
  Foundry dispatchers attach provenance metadata to agent runs; the doctrine
  binds anti-pattern detection to this metadata.
- **C-6 Tooling honesty** — the doctrine does not forbid tools for
  legitimate scaffolding (e.g. opening a wave descriptor, registering a new
  audit class). The line is drawn at "scripted bodies of substantive
  content"; the catalog (D-1) names where the line falls.

### Named prior incidents

- **Incident I-1 (2026-05-18, codex-erp-ip-w2 lambda-wrap)**: described
  above; documented in
  `docs/postmortems/postmortem-codex-erp-ip-w2-lambda-wrap-2026-05-18.md`.
- **Incident I-2 (2026-05-12, README template stamping)**: 14 README files
  generated from a shell-loop template; six follow-up PRs to rewrite.
- **Incident I-3 (2026-05-19, journey batch JA-2026-05-19-A)**: 47 journey
  artifacts produced in a fan-out batch; 14 failed the substance bar (the
  substance bar caught them, but the lambda-wrap pattern was the root cause
  of the 14 failures).
- **Incident I-4 (2026-04-22, codex-b2b-leader-w1 jq body interpolation)**:
  pre-keystone incident where jq was used to interpolate company names into
  a template body; required four-PR remediation before the W3-G ADR cluster
  could land.

## Decision

The following are CATEGORICALLY forbidden as authoring mechanisms for
substantive content artifacts:

- **AP-1** Shell loops (bash `for`, `while`) over filenames where the loop
  body writes a content file whose body is template-driven.
- **AP-2** `jq` invocations that combine a constant template with a per-
  artifact substitution and write the result to disk as substantive content.
- **AP-3** `awk`/`sed`/`yq`/`mustache`/`handlebars`/`liquid`/other template
  engines applied to substantive-content bodies.
- **AP-4** Python/Node/Ruby scripts that iterate a list and write content
  files whose bodies share ≥X% non-allowlisted shingle content (X enforced
  by ADR-0322 + no-template-stamping crate).
- **AP-5** "Lambda-wrap" idioms — any function whose signature is
  `(name: str) -> WriteFile` and whose output bodies are not authored
  per-name with bespoke substance.
- **AP-6** AI-agent prompts whose single invocation produces ≥2 content
  files within the same artifact class with substance-bar near-misses (the
  prompt operator should iterate the agent per artifact, not request a
  batch).
- **AP-7** Copy-paste of an artifact body into a sibling artifact followed
  by a global find-replace.
- **AP-8** Markdown include directives that embed a shared body fragment
  into ≥2 artifacts at the level of substantive content (shared boilerplate
  via the substance-bar allowlist mechanism in ADR-0322 §D-7 is permitted;
  shared substantive content is not).

The doctrine is enforced through three complementary mechanisms:

1. **Provenance attestation** (D-3) — every content artifact carries a
   provenance entry describing the tool chain used.
2. **Loop detector** (D-4) — the foundry dispatcher and the local pre-
   commit hook detect script-driven write patterns and BLOCK them.
3. **Template-stamping detector** (per ADR-0322 §D-7) — body-level
   detection of the symptoms of an undetected script.

## Consequences

Categorically forbidding script- and template-driven authoring of substantive content (AP-1..AP-6) means contributors must author each artifact per-name with bespoke substance, and the provenance and template-stamping detectors will block offending pull requests; the detailed mechanics, failure modes, and migration path below enumerate the operational consequences of that prohibition.

## Detailed Mechanics

### D-1 Anti-pattern catalog

`/specs/anti-pattern-catalog.json` enumerates AP-1..AP-8 with the
following per-entry fields:

- `ap_id` (string, e.g. `AP-1`).
- `ap_name` (string).
- `ap_description` (string ≥80 words).
- `ap_detection_strategy` (enum: `tool_provenance`, `body_shingle`, `prompt_intent`).
- `ap_severity` (BLOCKER).
- `ap_carve_outs` (array<string>; allowed legitimate uses).
- `ap_example_violation` (string, an inline example).
- `ap_example_legitimate_alternative` (string).

The catalog is itself a Tier-2 documentation artifact (≥500 lines) and is
governed by the substance bar (ADR-0322).

### D-2 Allowed vs forbidden tool uses

| Use case                                      | Allowed?  | Notes                                    |
|-----------------------------------------------|-----------|------------------------------------------|
| Open a wave descriptor JSON file              | YES       | Descriptor schema is metadata, not body  |
| Register an audit event class entry           | YES       | Registry is structured, not prose        |
| Bump a dependency version                     | YES       | Cargo.toml is metadata                   |
| Author the body of an ADR                     | NO        | Substantive content                      |
| Author the body of a journey doc              | NO        | Substantive content                      |
| Author the body of an IP slice                | NO        | Substantive content                      |
| Author the body of a microservice README      | NO        | Substantive content                      |
| Author the body of a PRD                      | NO        | Substantive content                      |
| Generate a Cedar policy fragment from a       | CONDITIONAL | Allowed only when the fragment is        |
|   schema                                      |           | generated 1:1 from a canonical schema    |
|                                               |           | per ADR-0243 and the schema is itself    |
|                                               |           | substance-bar-bound prose.               |
| Generate gRPC stubs from a proto              | YES       | Stubs are mechanical, not substantive    |
| Author a postmortem body                      | NO        | Substantive content                      |
| Author a runbook                              | NO        | Substantive content                      |

The conditional row (Cedar fragment generation) is the only entry where
substance and metadata blur; the doctrine resolves it by requiring the
upstream schema to itself be substance-bar-bound prose.

### D-3 Provenance attestation

Every content artifact carries a provenance entry at
`evidence/provenance/<artifact-stem>.provenance.json`. Schema:

- `artifact_path` (string).
- `author_principal` (string; the agent ID or human handle).
- `tool_chain` (array<object>; each object names a tool used and its
  invocation; e.g. `{"tool": "claude-opus-4-7", "operation": "write", "timestamp": "..."}`).
- `script_invocation_detected` (bool; populated by foundry dispatcher).
- `script_invocation_detail` (optional<string>; populated when bool=true).
- `attestation_signature` (string; ed25519 over the above fields).
- `provenance_emitted_at` (RFC 3339).

The dispatcher (per `omc-teams` substrate + Foundry pipeline) attaches the
provenance record to every PR. The `oya-governance-content-authorship-
provenance` crate verifies presence + signature validity.

### D-4 Loop detector

`oya-governance-loop-detector` runs both pre-commit locally and at the
foundry dispatcher edge. It detects script-driven write patterns by:

1. Observing the sequence of write operations in a single agent run.
2. Flagging runs where ≥2 in-scope content files are written within a
   single short interval (≤90 s) from a single agent process.
3. Cross-referencing the body shingle similarity of the written files via
   the ADR-0322 bespoke-ratio crate.
4. If the shingle similarity exceeds the per-tier threshold AND the temporal
   pattern matches a loop signature, emit
   `governance.content.script_invocation.detected` at BLOCKER severity.

False-positive carve-outs:

- A multi-file refactor that updates ≥2 files with congruent changes (e.g.
  a rename) is exempt when the agent's run carries an explicit
  `intent: refactor-rename` annotation. The annotation is required to pass
  the loop detector.
- Wave scaffold operations that author the wave descriptor + EVIDENCE.md
  + WAVE_MAPPING.md are exempt because none of the three files are
  substantive content artifacts in the ADR-0322 sense.

### D-5 Prompt-intent detection

Agent prompts that request batch authorship are detected by parsing the
prompt envelope sent to the agent. Patterns that BLOCK at the dispatcher
edge:

- A prompt that names ≥2 artifact paths and asks for body authorship.
- A prompt that names a list of slugs and asks the agent to "fill in the
  body for each".
- A prompt that invokes a `for ... in` construct over artifact identifiers.

The dispatcher rewrites the prompt to author a single artifact and refers
the operator to ADR-0323 (multi-wave sequencing) for the legitimate
mechanism to author multiple artifacts.

Carve-outs: prompts authored by humans (not the dispatcher's own
orchestration prompts) are warned but not blocked unless the agent's
output also trips the loop detector. This balances human directional
authority with the doctrine's BLOCKER stance.

### D-6 Body-level fingerprinting

`oya-governance-anti-script-anti-template` extends the no-template-stamping
detector (ADR-0322 §D-7) with anti-script signatures:

- Detect "lambda-wrap" patterns: bodies whose first paragraph differs only
  by a noun substitution from a sibling body, the rest of the body
  identical.
- Detect "form-feeder" patterns: bodies that contain runs of identical
  multi-line blocks separated by single-line substitutions.
- Detect "jq-interpolation" patterns: bodies whose Markdown structure is
  identical and whose only differences fall within text nodes (no
  structural differences).

The fingerprints are evaluated per pair within a 14-day rolling window of
artifacts of the same doc_class. A fingerprint hit emits
`governance.content.anti_pattern.detected` with `anti_pattern_id ∈ {AP-1..AP-8}`.

### D-7 Tool invocation attestation

`oya-governance-tool-invocation-attestation` is the dispatcher-side crate
that wraps every agent-tool invocation and records the operation. The
attestation envelope:

- `agent_id`.
- `tool_name`.
- `operation` (`read`, `write`, `bash`, `mcp`, etc.).
- `target_path` or `command`.
- `timestamp`.
- `wave_context` (if a wave is open in the agent's lane).
- `intent` (free-form, but checked against a closed set of valid intents).

A `write` operation against an in-scope content path forces the dispatcher
to also produce a per-write attestation entry. Multiple `write` operations
within a single agent run against in-scope paths trigger the loop detector.

### D-8 Legitimate alternative: per-wave per-artifact authorship

The doctrine's positive guidance is that authoring N artifacts within a
wave is done by N agent invocations, one per artifact, each with its own
context and per-artifact substance bar. This is the mechanism named in
ADR-0323's wave sequencing doctrine and is the only doctrine-approved way
to achieve breadth.

For very large breadth (N>24 artifacts), the wave doctrine forces a
sequential decomposition into multiple waves rather than a single fan-out;
this naturally enforces per-artifact authorship discipline.

### D-9 Refactor exception

A legitimate cross-artifact refactor (e.g. renaming a Cedar action across
all ADRs that cite it) is permitted via the `intent: refactor-rename`
annotation. The annotation requires:

- The refactor touches no substantive content beyond the rename.
- The diff has zero non-rename body lines.
- The refactor is performed within a single PR (not split across multiple).
- The refactor is reviewed at the F4-substance facet at a relaxed bar
  (the bar is satisfied by the unchanged substance, not by new substance).

`oya-governance-content-authorship-provenance` verifies the annotation's
claims against the diff.

### D-10 Doctrine evolution and exception ledger

Genuine new use cases that need an exception are recorded in
`docs/governance/anti-script-exception-ledger.md`. Each entry must:

- Name the use case in one sentence.
- Cite the ADR that the exception amends (typically this one).
- Be reviewed at council-quality + council-foundry concurrence.
- Carry an expiry date (≤180 days) at which the exception is reviewed.

Exceptions never become defaults; the doctrine is biased toward refusing
new exceptions and recommending wave decomposition instead.

### D-11 Distinction between scaffolding and authoring

The doctrine permits scaffolding tools that initialise an empty
artifact with the canonical headings and frontmatter scaffold. The
distinction:

- **Scaffolding** writes headings, frontmatter keys, and explicit
  placeholder markers (e.g. `<!-- author body here -->`); it does
  not write substantive prose into section bodies.
- **Authoring** writes substantive prose; this must be per-artifact,
  per-section bespoke work and must clear the substance bar.

A scaffolding tool that pre-populates section bodies with prose
(even short prose) is reclassified as a template engine and forbidden.
The line is drawn at "no prose, only structure"; the placeholder marker
convention makes the line testable.

### D-12 Carve-out for tightly-bounded code generation

Code generation from machine-readable schemas (gRPC stubs, OpenAPI
clients, Cedar fragments derived from a registry) is permitted because:

- The output is not substantive content in the ADR-0322 sense.
- The input is itself substance-bar-bound prose plus a machine-readable
  schema; the generation is deterministic and reproducible.
- The generated artifacts are not eligible for citation as
  authoritative content; they are infrastructure.

The carve-out is enforced by classifying code-generation outputs as
non-content artifacts; they are not stored under `docs/` and are not
subject to substance bar.

### D-13 Anti-pattern detection cross-feed with the substance bar

The anti-script detector and the substance bar share state:

- The bespoke-ratio corpus (per ADR-0322 D-2) is read by both
  detectors.
- A template-stamping detection at the substance-bar level cross-
  references the provenance attestation to identify the source agent
  and the source tool chain.
- A loop-detection event from this doctrine triggers a substance-bar
  re-evaluation of the implicated artifacts.

The cross-feed makes the two detectors mutually reinforcing: an
artifact that evades one is more likely to be caught by the other.

### D-14 Per-doc-class anti-pattern tuning

The anti-pattern detector's sensitivity is tuned per doc class:

- ADRs and journey docs use the strictest thresholds (≤0.4 bespoke
  ratio on a pair triggers an investigation).
- IP slices and microservice READMEs use moderate thresholds (≤0.35
  triggers investigation).
- Postmortems use relaxed thresholds (≤0.30 triggers investigation)
  because postmortems share more structural skeleton legitimately.

The tuning is published at `/specs/anti-pattern-tuning.json` and is
reviewed quarterly by council-quality.

## Cedar Policy Hooks

```cedar
// Fragment: cedar/anti-script/dispatcher-may-block-batch-prompt.cedar
permit (
  principal == Service::"oyatie.foundry.dispatcher",
  action == Prompt::"reject",
  resource is Prompt
) when {
  context.batch_authorship_intent_detected == true &&
  context.target_artifact_class in ["adr", "journey", "ip-slice",
                                    "microservice-readme", "prd", "rfc"]
};
```

```cedar
// Fragment: cedar/anti-script/agent-may-author-single-artifact.cedar
permit (
  principal in Group::"oyatie.foundry.content_authors",
  action == DocArtifact::"write",
  resource is DocArtifact
) when {
  context.write_count_in_session == 1 ||
  context.intent == "refactor-rename" ||
  context.write_target_class in ["wave-descriptor", "evidence-ledger",
                                 "audit-registry"]
};
```

```cedar
// Fragment: cedar/anti-script/no-batch-shell-loop.cedar
forbid (
  principal in Group::"oyatie.foundry.content_authors",
  action == Shell::"invoke",
  resource is Shell::Command
) when {
  context.command_matches_loop_signature == true &&
  context.command_writes_in_scope_content == true
};
```

```cedar
// Fragment: cedar/anti-script/provenance-attestation-required.cedar
forbid (
  principal,
  action == PullRequest::"merge",
  resource is PullRequest
) when {
  context.in_scope_content_artifacts_count > 0 &&
  context.provenance_attestation_present == false
};
```

```cedar
// Fragment: cedar/anti-script/refactor-rename-exception-binding.cedar
permit (
  principal in Group::"oyatie.foundry.content_authors",
  action == DocArtifact::"batch_write",
  resource is DocArtifact
) when {
  context.intent == "refactor-rename" &&
  context.non_rename_body_lines == 0 &&
  context.single_pr == true
};
```

## Audit Event Classes Emitted

| Class                                              | Severity | Source crate                                  |
|----------------------------------------------------|----------|-----------------------------------------------|
| governance.content.script_invocation.detected      | BLOCKER  | oya-governance-loop-detector                  |
| governance.content.anti_pattern.detected           | BLOCKER  | oya-governance-anti-script-anti-template      |
| governance.content.provenance.missing              | BLOCKER  | oya-governance-content-authorship-provenance  |
| governance.content.provenance.invalid_signature    | BLOCKER  | oya-governance-content-authorship-provenance  |
| governance.content.tool_invocation.attested        | INFO     | oya-governance-tool-invocation-attestation    |
| governance.content.batch_prompt.rejected           | BLOCKER  | oya-governance-loop-detector                  |
| governance.content.refactor_rename.honoured        | INFO     | oya-governance-loop-detector                  |
| governance.content.anti_script.exception_ledger.    | INFO     | oya-governance-anti-script-anti-template      |
|   recorded                                          |          |                                               |

Each class carries the canonical envelope (tenant, timestamp, principal,
attestation chain).

## SLO Implications

`microservices/governance/anti-script/slos/anti-script.openslo.yaml`:

- `loop_detector_p95_latency`: ≤ 60 s per agent run.
- `provenance_attestation_completeness`: ≥ 99.95% of PRs touching in-scope
  content carry a provenance entry.
- `false_positive_rate`: ≤ 0.5% measured over 30-day window.
- `false_negative_audit_rate`: ≤ 1% of audits find an undetected
  template-stamping or lambda-wrap pattern.
- `dispatcher_prompt_rewrite_p99_latency`: ≤ 250 ms (prompt rewrite must
  not noticeably degrade agent throughput).

## Migration Path / Phased Rollout

- **Phase 0 (T-0, ADR Proposed)**: shadow mode; events emitted at WARN.
- **Phase 1 (T+7 days)**: BLOCKER for codex worker fleet (the source of
  incident I-1).
- **Phase 2 (T+14 days)**: BLOCKER for all dispatched agents (Claude +
  codex + gemini + any new worker class).
- **Phase 3 (T+21 days)**: BLOCKER for human-authored PRs as well (humans
  must also attach provenance, but the bar for human attestation is
  lighter: an editor/tool ID rather than a multi-tool chain).
- **Phase 4 (T+30 days)**: post-rollout audit; the doctrine eligible for
  promotion per ADR-0327.

## Failure Modes + Recovery

### F-1: Legitimate refactor blocked

A cross-artifact refactor without the `refactor-rename` annotation is
blocked. Recovery: the author re-runs the refactor with the annotation
attached; the dispatcher records the annotation and allows the batch
write under the refactor-rename Cedar fragment.

### F-2: Loop detector trips on multi-file scaffold

A wave scaffold operation that legitimately writes wave-descriptor +
EVIDENCE.md + WAVE_MAPPING.md trips the loop detector. Recovery: the
wave-scaffold tool carries a known intent annotation `intent: wave-scaffold`
that the dispatcher recognises and the detector skips.

### F-3: Human author with personal shell tooling

A human contributor with a personal "open new ADR" tool that pre-fills
headings is technically running a template engine. Recovery: the doctrine
permits pre-fill of headings (which are not substantive content) provided
the pre-filled headings carry a clear placeholder marker; the author then
authors the substance and the substance bar (ADR-0322) verifies bespoke
content.

### F-4: Agent attempts to bypass via base64 encoding

An agent encodes the body via base64 to evade the shingle detector.
Recovery: the loop detector also flags base64-encoded write operations
against in-scope content paths as suspicious and forces the operator to
re-author in plain Markdown.

### F-5: Dispatcher unavailable

The dispatcher (which authors the provenance entries) is unavailable.
Recovery: PRs touching in-scope content cannot merge until the dispatcher
is restored; the disaster-mode procedure per ADR-0306 applies.

### F-6: Substance-bar bespoke-ratio threshold is genuinely too strict

A wave's artifacts share substantial structural similarity for legitimate
domain reasons (e.g. 12 ADRs about the 12 layer-enum values). Recovery:
the wave descriptor declares
`structural_similarity_justified: <reason>` and the doctrine relaxes the
bespoke-ratio threshold for the wave; the relaxation is reviewed by
council-quality concurrence before the wave can land.

## Verification

Named CI checks:

- `oya-governance-anti-script-anti-template/body-fingerprint`
- `oya-governance-loop-detector/temporal-pattern`
- `oya-governance-content-authorship-provenance/presence`
- `oya-governance-content-authorship-provenance/signature`
- `oya-governance-tool-invocation-attestation/coverage`
- `oya-governance-no-template-stamping` (shared with ADR-0322)

Named crates:

- `oya-governance-anti-script-anti-template`
- `oya-governance-loop-detector`
- `oya-governance-content-authorship-provenance`
- `oya-governance-tool-invocation-attestation`
- `oya-governance-no-template-stamping` (shared)

Verification fixtures: `tests/governance/anti-script/` including a
synthetic lambda-wrap incident replay, a legitimate refactor scenario,
and a wave-scaffold carve-out validation.

## Cross-References

### Other ADRs

- ADR-0063 (doc-coverage enforcement) — substrate.
- ADR-0091 (multispectrum review) — F4-substance binding.
- ADR-0105 (layer-enum 13-canonical) — governance lane layer.
- ADR-0130 (observability SLO-gated promotion) — SLO substrate.
- ADR-0131 (per-microservice flat layout) — crate layout.
- ADR-0132 (suite dissolution) — companion ban.
- ADR-0145 (inter-microservice reform) — direct gRPC invariants.
- ADR-0242 (oyatie tenant) — tenancy of governance events.
- ADR-0243 (Cedar universal gate) — Cedar fragment convention.
- ADR-0244 (tenant scoping) — envelope per event.
- ADR-0263 (audit event registry) — class registration.
- ADR-0306 (disaster mode) — dispatcher-outage degraded mode.
- ADR-0316 (capability tier) — capability-tier artifacts subject.
- ADR-0321 (B2B leader coverage) — leader-coverage artifacts subject.
- ADR-0322 (substance bar) — body-level substance check.
- ADR-0323 (wave sequencing) — wave-level breadth mechanism.
- ADR-0327 (wave-3 completion criteria) — promotion gates consume.

### Standards

- `docs/standards/documentation-rigor.md` §1.1 substance.
- `docs/standards/multispectrum-review-v2.4.0.md` F4-substance facet.

### Microservices

- `microservices/governance/anti-script/` — substrate.
- `microservices/foundry/dispatcher/` — provenance attachment + prompt
  rewrite.
- `microservices/observability/` — SLO substrate.
- `microservices/audit-chain/` — event sink.

### Journeys

- `journeys/foundry/jou-2026-05-20-author-an-adr/` — author-facing journey
  updated to reference the anti-script doctrine.
- `journeys/foundry/jou-2026-05-20-refactor-rename/` — refactor-rename
  journey.

### Specs

- `/specs/anti-pattern-catalog.json`
- `/specs/substance-bar/template-allowlist.yaml`

### Postmortems referenced

- `docs/postmortems/postmortem-codex-erp-ip-w2-lambda-wrap-2026-05-18.md`
- `docs/postmortems/postmortem-readme-template-stamping-2026-05-12.md`

### Feedback notes consumed

- `feedback_docs_substance_not_scaffold_2026_05_20.md`
- `feedback_go_with_original_ambition_2026_05_20.md`
- `feedback_automate_everything.md`
- `feedback_no_silent_regression.md`
- `feedback_doc_coverage_enforced.md`
- `feedback_multispectrum_review_v22.md`
- `feedback_multispectrum_adherence_facets.md`

## Appendix A — codex-erp-ip-w2 lambda-wrap incident replay

The lambda-wrap incident on 2026-05-18 is the canonical case study for
the doctrine. Reconstructed from the postmortem:

1. The codex agent received a prompt asking for "18 IP slices for ERP
   parity surfaces" naming each surface by slug.
2. The agent wrote a bash script:
   ```bash
   for slug in $SLUGS; do
     cat template.md | jq --arg s "$slug" '. + {title: $s}' > "ip-${slug}.md"
   done
   ```
   The template file contained the full body that became identical
   across all 18 outputs.
3. The PR landed at commit `4e2f...c19`; lean-a4-structure and
   lean-a5-doc-coverage lanes passed (shape was correct).
4. F4-substance facet was advisory; the reviewer agent flagged the
   pattern but the merge queue admitted the PR.
5. Three days later a human reviewer noticed the identical bodies; the
   W2 remediation wave was opened.
6. The remediation cost: three agents × two days of authoring time
   plus reviewer-agent re-attestation for all 18 IP slices.

Lessons captured into this doctrine:

- The shape-only check class (lean-a4, lean-a5) is insufficient; body-
  level checks are required.
- The F4-substance facet must be BLOCKER class, not advisory.
- The provenance attestation must record the bash invocation; the
  loop detector must catch the temporal pattern.
- The prompt's intent ("18 IP slices...") must be detectable at the
  dispatcher edge and rewritten to per-artifact prompts.

## Appendix B — Refactor-rename worked example

A legitimate refactor: renaming the Cedar action `Discount::apply` to
`Discount::apply_byok` across 12 ADRs. The author:

1. Annotates the agent run with `intent: refactor-rename`.
2. Confirms that the diff contains only the rename (zero non-rename
   lines).
3. Runs the rename across all 12 ADRs in a single PR.
4. The loop detector observes 12 writes, checks the annotation, and
   permits the batch under the refactor-rename Cedar fragment.
5. The substance bar re-evaluates each ADR against the prior corpus
   (which contains the prior name); bespoke ratio is preserved
   because the rename is a one-token change in a large corpus.
6. The F4-substance reviewers re-sign their facets at relaxed bar
   (the substance is unchanged).
7. The PR merges with provenance attestation recording the bash
   `git grep -l "Discount::apply" | xargs sed -i 's/.../.../'` plus
   the intent annotation.

This example demonstrates that the doctrine's BLOCKER stance does not
prevent legitimate refactors; it requires that the legitimate path be
explicitly invoked and recorded.

## Appendix C — Provenance record schema example

A provenance record for a single-artifact authoring run:

```json
{
  "artifact_path": "docs/decisions/ADR-NNNN-hypothetical-doctrine.md",
  "author_principal": "claude-opus-4-7@dispatcher-instance-19",
  "tool_chain": [
    {
      "tool": "claude-opus-4-7",
      "operation": "read",
      "target_path": "docs/standards/documentation-rigor.md",
      "timestamp": "2026-05-20T14:01:11.000Z"
    },
    {
      "tool": "claude-opus-4-7",
      "operation": "read",
      "target_path": "docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md",
      "timestamp": "2026-05-20T14:01:42.000Z"
    },
    {
      "tool": "claude-opus-4-7",
      "operation": "write",
      "target_path": "docs/decisions/ADR-NNNN-hypothetical-doctrine.md",
      "timestamp": "2026-05-20T14:08:55.000Z"
    }
  ],
  "script_invocation_detected": false,
  "script_invocation_detail": null,
  "attestation_signature": "ed25519:...base64url...",
  "provenance_emitted_at": "2026-05-20T14:09:01.000Z",
  "intent": "author-adr"
}
```

A provenance record for an attempted batch authoring that was blocked
at the dispatcher edge:

```json
{
  "artifact_path": null,
  "author_principal": "codex-worker-22@dispatcher-instance-09",
  "tool_chain": [
    {
      "tool": "bash",
      "operation": "invoke",
      "command": "for slug in ${SLUGS}; do jq ... > ip-${slug}.md; done",
      "timestamp": "2026-05-20T15:11:00.000Z"
    }
  ],
  "script_invocation_detected": true,
  "script_invocation_detail": "shell for-loop writes 18 in-scope content files; loop signature matches AP-1",
  "attestation_signature": "ed25519:...base64url...",
  "provenance_emitted_at": "2026-05-20T15:11:01.000Z",
  "intent": "rejected-by-dispatcher",
  "rejection_event_id": "evt-3f1c...-aa92"
}
```

The dispatcher emits a rejection event before the agent's write
reaches disk; no in-scope content artifacts are produced. The
operator receives a structured rejection message naming AP-1 and
referring them to ADR-0323's wave-sequencing alternative.

## Appendix D — Anti-pattern catalog full enumeration

The catalog at `/specs/anti-pattern-catalog.json` carries the full
description for each of AP-1..AP-8. Inlined here for reference (the
canonical source is the JSON file):

- **AP-1 (Shell loop writes)**: A `for` or `while` loop over a list of
  artifact identifiers that writes content files. Detection: loop
  signature plus in-scope content path target. Carve-outs: refactor-
  rename per D-9.
- **AP-2 (jq body interpolation)**: A `jq` invocation that produces a
  Markdown body by interpolating a template with a per-artifact value.
  Detection: jq invocation followed by Markdown write to in-scope path.
  Carve-outs: jq used to write structured (JSON/YAML) registries or
  schemas where the output is not substantive prose.
- **AP-3 (Template-engine body)**: awk, sed, yq, mustache, handlebars,
  liquid invocation producing substantive prose. Detection: tool
  invocation provenance plus output path. Carve-outs: same as AP-2.
- **AP-4 (Script-iterated authoring)**: Python/Node/Ruby/etc. script
  that iterates a list and writes content files whose bodies share
  significant shingle content. Detection: provenance plus body
  shingle. Carve-outs: refactor-rename.
- **AP-5 (Lambda-wrap idiom)**: Any function/lambda whose signature is
  `(name) -> body` and whose outputs are not per-name bespoke.
  Detection: provenance plus shingle plus prompt-intent. No carve-out;
  refactor cases use AP-9 (not enumerated) only if the body diff is
  trivial.
- **AP-6 (Batch-authorship prompt)**: An AI-agent prompt requesting
  body authorship for ≥2 artifacts in a single invocation. Detection:
  dispatcher-edge prompt parse. Carve-outs: wave-scaffold prompts.
- **AP-7 (Copy-paste plus global replace)**: Sibling artifact body
  pasted in followed by find-replace. Detection: shingle plus
  diff-history examination. No carve-out for substantive content.
- **AP-8 (Markdown include of substantive content)**: A `{% include %}`
  or analogous directive that embeds a shared body fragment into ≥2
  artifacts. Detection: directive presence plus shared-fragment
  scanning. Carve-outs: shared boilerplate per the substance-bar
  allowlist (e.g. tenancy preamble).

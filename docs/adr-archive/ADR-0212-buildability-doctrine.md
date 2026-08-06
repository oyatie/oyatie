---
id: ADR-0212
status: Superseded
superseded_by: [ADR-0709]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0212 — Buildability Doctrine

- **Status:** Accepted
- **Date:** 2026-05-18
- **Deciders:** council-architecture, axis-foundry, council-quality
- **Lane:** governance / substrate-doctrine
- **PR:** #143 close-out

## Context

Oyatie ships across multiple µservices, regulatory packs, and contributor lanes (human + AI). Artifacts (PRDs, ADRs, IPs, runbooks, contracts, Helm charts, SLOs, threat models, standards docs) accumulate fast. Two failure modes are observable:

1. **Tribal-knowledge drift.** A new engineer (or new AI agent session) walks up to an existing µservice and the artifacts read like reminders for the original author, not instructions for a stranger. Production-grade output requires another conversation with someone who "knows the context".
2. **Padding to bar.** Bar enforcement (e.g. "IPs ≥150 lines") incentivises filler — repeated headings, citation-only stubs, boilerplate scaffolds that look substantive but convey no implementable detail.

Both modes degrade agentic development the most because agents lack the off-channel context humans carry. PR #143 surfaced concrete instances (6 plugin-app-store IPs at 135-149 lines, agent dispatch briefs without §Audience or §Abstraction-rationale, scorecards self-declaring GREEN without evidence citation).

This ADR codifies the *buildability bar* — the minimum substance an artifact must carry so a cold stranger can produce hyperscaler-grade output from it alone.

## Decision

**Every artifact in this codebase MUST satisfy the per-kind bar below.** The buildability gate (advisory in PR #143; BLOCKER once 33+ µservices pass) verifies the bar mechanically; reviewer-agent verifies substance qualitatively.

### Per-artifact buildability bar

| Artifact | Substance requirement |
| --- | --- |
| **PRD** | ≥5 user stories with measurable acceptance criteria; explicit scope-in / scope-out / non-goals; success metrics with numerical targets; trace to milestones + phases. |
| **PHASE-SPEC** | Phases list IPs; each IP has owner, blocking deps, exit criteria. |
| **IP** | ≥150 SUBSTANTIVE lines (blank / heading-only / citation-only lines excluded by the structural counter per ADR-0221 §M-10); file-level paths; line ranges; test count + test names; evidence files to emit; idempotency story; rollback steps. |
| **ADR** | Context + Decision + ≥3 alternatives with explicit "rejected because" + ≥3 consequences (positive + negative + operational sub-sections) + named industry sources + §"In-house roadmap". |
| **Contract (OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3)** | Sufficient to generate working clients in TS / Rust / Swift / Kotlin / C# / Python; ≥3 worked examples per endpoint; idempotency-key + cursor pagination + X-Request-Id + ULID baked in. |
| **Helm chart** | Deploys working substrate; per-pack overlays; OpenBao SecretReference for secrets; values fully documented in `values.yaml` comments. |
| **Runbook** | Step-by-step executable as-is; no "ask SME" steps; named commands; expected outputs; rollback at every step. |
| **SLO (OpenSLO)** | Concrete numerical targets; burn-rate alerts wired per ADR-0139 4-window model; dashboards linked. |
| **Threat model** | STRIDE per surface; named specific attack vectors (not abstract "injection risk"); mitigations cited. |
| **Standards doc** | Implementable from doc alone; no "see X for details" hand-waves. |
| **Scorecard** | Every "GREEN" cell cites specific evidence (code / ADR / test / gate / runbook); no self-declared GREEN. |

### Buildability gate (CI enforcement)

`oya gate validate buildability-discipline` (queued for PR-144 per ADR-0221 §M-10):

1. **Structural line count.** Parse markdown AST per IP / ADR; count substantive lines (not blank, not heading-only, not citation-only, not pure-link); reject IPs with substantive < 150; **WARN** at 60-79; **BLOCKER at <60**; current bar is set at 117-119 substantive lines for IPs touched in PR #143 (lower than the 150 target per honest disclosure — staged promotion path).
2. **Alternative-count check.** Every ADR has ≥3 "## Alternative" headings with adjacent "rejected because" text.
3. **Consequence-count check.** Every ADR has ≥3 consequences across `### Positive`, `### Negative`, `### Operational` sub-sections.
4. **Industry-source check.** Every ADR has a `## Named industry sources` (or equivalent) section with ≥2 vendor / project / standard references.
5. **In-house roadmap check.** Every ADR has a `## In-house roadmap` or `### In-house roadmap` heading (single occurrence).
6. **Citation-evidence check.** Every scorecard row marked GREEN cites at least one evidence path (file: pattern).

### Stranger-walks-up-cold test (qualitative)

Reviewer-agent applies this test on artifact PRs:

- Pick a random new artifact from the PR.
- Imagine a competent engineer / AI agent reading it cold (no prior project context).
- Can they produce production-grade output from it alone?
  - Yes → buildability bar met.
  - No → reviewer-agent surfaces the specific gap (e.g. "IP doesn't say where the migration table lives" / "ADR doesn't say what triggers Phase-2") and blocks merge.

## In-house roadmap

This is doctrine, not runtime. 100% in-house from day one. No vendor-replaceable component. Future Phase-2: surface buildability gate metrics in management-cockpit per ADR-0220 so engineering leadership sees padding-rate + missing-citation-rate per axis.

## Alternatives considered

### Alternative 1 — "No bar; trust author judgment"

**Rejected because** observed failure mode: tribal-knowledge drift. Without a bar, artifacts trend toward author-reminder shape (terse, contextual) and fail the stranger test. Agentic-dev sessions surfaced this repeatedly — agents either over-pad or under-deliver without a bar.

### Alternative 2 — "Line-count only ('IP ≥150 lines')"

**Rejected because** line count incentivises padding (M-10 surfaced 6 plugin-app-store IPs at 135-149 lines that are honest content but under the bar; vs. some 150+ IPs that hit count via filler). Line count is necessary but not sufficient; structural counter (substantive-only) + qualitative reviewer-agent stranger-test together provide better signal.

### Alternative 3 — "Reviewer-agent only ('eyeball every PR')"

**Rejected because** reviewer-agent fatigue + cost. Mechanical checks (structural line count, alternative count, citation presence) belong in CI; reviewer-agent capacity should be reserved for the qualitative stranger test on artifacts that pass the mechanical bar.

### Alternative 4 — "Per-axis bars (each axis sets its own buildability bar)"

**Rejected because** consistency across axes is a hyperscaler property. AWS docs read consistently across services — buildability bar is workspace-wide.

## Consequences

### Positive

- **Agentic-dev resilience.** Agents can produce production-grade output from artifacts alone — no off-channel context leakage.
- **Onboarding speed.** New human engineers ramp on µservices via artifacts, not via 1:1 conversations.
- **Quality compounding.** Every µservice that ships at the bar raises the bar empirically; the gate threshold can climb (117 → 130 → 150 substantive-line BLOCKER) as the corpus matures.

### Negative

- **Authoring friction.** Hitting the bar is real work — ≥150 substantive lines per IP, ≥3 alternatives per ADR. Authors will push back when the value-to-effort ratio feels low on simple IPs.
  - **Mitigation:** templates per artifact kind under `docs/templates/` already exist; bar enforcement starts at WARN level for sub-150 substantive-line IPs (current floor 60).
- **False negatives.** Some substantive artifacts (e.g. a focused ADR with 2 alternatives that exhausts the design space) will fail the mechanical check.
  - **Mitigation:** reviewer-agent can override mechanical with `buildability-override: <reason>` PR label; the override is logged + audited quarterly to ensure it's not normalising sub-bar work.
- **Compounding maintenance.** Every existing artifact eventually retro-meets the bar.
  - **Mitigation:** retroactive sweeps are queued PRs (PR-150 substrate thin-IP expansion sweep).

### Operational

- **Gate ladder.** Buildability gate launches as advisory; promotes to BLOCKER once 33+ µservices' artifacts pass. Promotion is queued in `registry/placeholder-debt/adr-follow-ups.yaml`.
- **Reviewer-agent stranger-test prompt.** Every PR review includes the buildability prompt: "Pick a random new artifact; cold-read it; can you produce production-grade output?". Add to reviewer-agent system prompt template.
- **Evidence-citation discipline.** Every scorecard GREEN claim cites an evidence path; CI gate `oya-check-scorecard-evidence-citation` (queued) enforces.

## References

- ADR-0083 — Tier 3 / Tier 1 source discipline (used as the bar for code; this ADR is the bar for non-code artifacts).
- ADR-0145 — Cross-product invariants (audit + tracing + ontology-projection; scorecard GREEN claims on these MUST cite evidence).
- ADR-0211 — In-house tech stack policy (this doctrine applies to ADR-0211 itself; 225 lines, 19 headings, 5 alternatives, 9 industry sources — meets bar).
- ADR-0220 — Consumer intelligence substrate (management-cockpit hosts buildability metrics surface).
- ADR-0221 — Agentic development pipeline hardening (§M-10 buildability structural line count gate).
- `docs/templates/pull-request-template.md` — PR template with buildability checklist.
- `docs/templates/` — per-artifact-kind templates that pre-load the bar.

## Named industry sources

- AWS docs — consistent per-service structure (Overview / Concepts / Use cases / API / Limits / Pricing).
- Stripe API docs — every endpoint includes ≥3 worked examples in multiple languages; cited as the buildability ceiling for contract artifacts.
- Linear product docs — internal IPs are reportedly comprehensive enough for cold-start; cited via industry blog posts.
- Cloudflare engineering blog — public RFCs include alternatives + consequences + named industry sources consistently.
- Palantir Foundry docs — internal artifact bar reportedly enforced by review; cited via ex-employee technical talks.
- Google's "How We Write Design Docs" (publicly referenced internal standard) — alternatives + consequences as core sections.

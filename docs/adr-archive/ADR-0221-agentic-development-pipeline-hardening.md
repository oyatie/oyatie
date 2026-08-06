---
id: ADR-0221
status: Superseded
superseded_by: [ADR-709]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0221 — Agentic Development Pipeline Hardening

- **Status:** Accepted
- **Date:** 2026-05-18
- **Deciders:** council-architecture, axis-foundry, council-quality
- **Lane:** governance / substrate-doctrine
- **PR:** #143 close-out

## Context

Session 2026-05-17 → 2026-05-18 (PR #143) surfaced 15 repeated or avoidable agentic-development mistakes. Codifying lessons in conversation memory alone is not durable — each new session loses the context. This ADR codifies the lessons + automates detection via fitness gates so the same mistakes cannot recur silently.

The doctrine also reframes the previous "agentic pipeline hardening rule 2026-05-12" stance: hardening is **encouragement** (push contributors toward the right path via templates + helpers + CI suggestions), NOT **prevention** (CI hard-blocks that paralyse exploratory work). Per /idea-refine pushback applied 2026-05-18, hooks ENCOURAGE; CI BLOCKS only on irreversible-class violations (e.g. orphan ADR citation, version-pin drift, vacuous-green).

## Decision

Adopt:

1. **Pre-dispatch validation templates** — dispatch briefs MUST carry §Audience + §Abstraction-rationale + §Catalog-collision-check.
2. **Per-step CI verification gates** — 4 new CI gates (below) detect the high-leverage mistake patterns.
3. **Doctrine intake automation** — decisions in conversation emit ADR scaffolds nightly so they don't live only in agent memory.
4. **PR-charter scope lock** — `evidence/pr-NNN-charter.json` locks scope at PR open; mid-PR scope expansion requires explicit amendment.
5. **Architecture-map automation** — visualization-as-code per session-start directive 2026-05-12; lives in Foundry.

### The 15 codified mistakes

| ID | Mistake | Root cause | Automation gate |
| --- | --- | --- | --- |
| M-01 | Spec version drift propagated from training data (claimed OpenAPI 3.3; actual 3.2.0) | no auto-verify before pinning | `oya-check-version-pin-source-cited` |
| M-02 | 5 persona-experience µservices dispatched with wrong abstraction | no pre-dispatch architectural-framing audit | dispatch-brief template required §Audience + §Abstraction-rationale |
| M-03 | Plugin-app-store vs marketplace vs community taxonomy confusion | no canonical glossary check pre-dispatch | `oya-check-canonical-glossary-compliance` |
| M-04 | Foundry vs intelligence audience conflation (internal vs consumer) | no audience-of-µservice declaration check | manifest.json#audience required field (INTERNAL / B2B-tenant / B2C-consumer / DEVELOPER) |
| M-05 | Hiring + job-search-consumer dispatched as separate µservices (belonged in community) | no existing-catalog check before new µservice | `oya-check-microservice-catalog-collision` |
| M-06 | Client-stack-discipline gate "PASSED" vacuously (zero client-manifests inputs) | no min-inputs guard on advisory gates | `oya-check-vacuous-green-gates` |
| M-07 | Wave plan revised 5+ times before dispatch | wave plan unlocked during dispatch consideration | `evidence/wave-plan-locked-NNN.json` lock primitive |
| M-08 | Queued ADRs (0211/0212/0215-0220) lived only in conversation; almost lost | doctrine intake not auto-emitted to ADR scaffolds | doctrine-intake → ADR-scaffold-draft nightly job |
| M-09 | Wrong agent ID used for SendMessage (sent to aborted Persona-1) | no human-readable agent ID registry with status | agent ID registry primitive (future tooling PR; defer) |
| M-10 | 6 plugin-app-store IPs at 135-149 lines (under buildability bar) | buildability bar not enforced at agent-level | `oya-check-buildability-line-count-structural` |
| M-11 | 419 thin IPs left as ledger after "expand 200" target | doc-coverage parallelized at wrong grain | dispatch-template recommends one-agent-per-µservice for doc-coverage |
| M-12 | Stale ADR-0174 references after deletion (mid-pivot) | renames without structural sweep | structural-rename helper via ast-grep; mandatory post-rename verification |
| M-13 | ADR-0211 orphan citation in v3 audit (cited as Accepted; no file on disk) | no CI gate for ADR-NNNN-cited-but-not-on-disk | `oya-check-adr-orphan-citation` |
| M-14 | 57-µservice PR scope creep from original 33 | no PR-charter locked at PR open | `evidence/pr-NNN-charter.json` lock primitive |
| M-15 | Doctrine inflation per message (no-code, multi-context, ecosystem, intelligence, etc.) | doctrine intake not batched + locked | doctrine-intake batching window; ADR draft + 24h review before adoption |

### 4 new CI gates (armed by task H shell harness)

Current enforcement surface: `tools/governance/adr-0221-governance-gates.sh`, wired into `.github/workflows/pr-tests.yml` under the `oya-governance-*` lane prefix. The `crates/oya-check-*` names below are portability targets for later native Rust ports; they are not the active CI entrypoint in this PR.

#### Gate 1 — `oya-check-vacuous-green-gates`

- **Current command:** `bash tools/governance/adr-0221-governance-gates.sh vacuous-green`
- **Future crate port:** `crates/oya-check-vacuous-green-gates`
- **Purpose:** Flag advisory gates with assertions_total == 0 (vacuous-pass detection).
- **Mode:** BLOCKER once 33+ µservices pass advisory checks.
- **Mistakes addressed:** M-06.
- **Implementation sketch:** Walk `registry/quality/lanes.yaml`; for each gate, run; if exit 0 AND zero inputs validated → fail with "gate is vacuously green".

#### Gate 2 — `oya-check-adr-orphan-citation`

- **Current command:** `bash tools/governance/adr-0221-governance-gates.sh orphan-citation`
- **Future crate port:** `crates/oya-check-adr-orphan-citation`
- **Purpose:** Find ADR-NNNN references in any doc/spec/ADR but no `docs/decisions/ADR-NNNN-*.md` file.
- **Mode:** BLOCKER.
- **Mistakes addressed:** M-13.
- **Implementation sketch:** Walk `docs/` + `microservices/` + `specs/` + `evidence/`; collect ADR-NNNN refs; cross-check `docs/decisions/`; fail on orphans.

#### Gate 3 — `oya-check-version-pin-source-cited`

- **Current command:** `bash tools/governance/adr-0221-governance-gates.sh version-pin`
- **Future crate port:** `crates/oya-check-version-pin-source-cited`
- **Purpose:** Every version pin in ADRs / PRDs / specs must cite WebSearch / Context7 / upstream source URL.
- **Mode:** Advisory → BLOCKER.
- **Mistakes addressed:** M-01.
- **Implementation sketch:** Walk ADRs / PRDs; find version pins (regex semver + lib name); cross-check for adjacent source URL citation. Forbid date-only Phase-2 triggers (regex sweep).

#### Gate 4 — `oya-check-buildability-line-count-structural`

- **Current command:** `bash tools/governance/adr-0221-governance-gates.sh buildability-line-count`
- **Future crate port:** `crates/oya-check-buildability-line-count-structural`
- **Purpose:** Structure-aware substantive-line count for IPs (excludes blank / heading / citation-only lines).
- **Mode:** Advisory → BLOCKER.
- **Mistakes addressed:** M-10.
- **Implementation sketch:** Parse markdown AST; count substantive content lines (not blank, not headings, not citation-only, not pure-links); reject IPs with substantive < 150; WARN at <80; BLOCKER at <60 (current floor 60 per ADR-0212).

### Architecture-map automation

Per session-start additional context 2026-05-12: Visualization-as-code directive — ADOPTED INTO FOUNDRY.

- `crates/oya-governance-architecture-map-kernel` — walks Cargo workspace + contracts + manifests; emits Mermaid + C4 diagrams.
- `crates/oya-governance-architecture-map-app` — composition root binary.
- `crates/oya-governance-architecture-map-freshness` — CI fails if architecture map stale > 24h.

Output: `evidence/architecture-map-*.svg` + `docs/diagrams/*.mmd`. Trigger PR: **PR-148 (Foundry agentic toolchain extension wave)** per close-out plan follow-up PR roadmap.

### Encouragement-over-prevention reframe (2026-05-18)

Per /idea-refine pushback applied 2026-05-18: **hooks encourage; CI gates enforce only on irreversible-class violations.** Specifically:

- **Encourage (template / hint / suggest):** dispatch-brief template fills, IP scaffold lines, ADR template alternatives section.
- **Block (CI gate exit 1):** orphan ADR citation, version-pin drift, vacuous-green gate, missing audience field.

Reasoning: hard-blocking exploratory shape (e.g. "you can't open a PR without 15 IPs ≥150 lines each") creates contributor friction that drives shortcuts (padding, stub IPs). Soft encouragement at authoring time + hard enforcement at irreversible-decision points (citation drift, version drift) preserves exploration while preventing rot.

## In-house roadmap

100% in-house from day one. These automation gates are part of our own developer experience and CI substrate; no vendor-replaceable component (Class C per ADR-0211). Future Phase-2: surface the lessons-learned + ADR-0221 metrics in management-cockpit (per ADR-0220 oyatie intelligence brand) so engineering leadership sees mistake-recurrence rates per axis + per session.

## Alternatives considered

### Alternative 1 — "No doctrine; hope mistakes don't repeat"

**Rejected because** 5+ of these 15 mistakes already happened TWICE in this session alone (M-01 OpenAPI version drift was caught + reverted twice; M-03 plugin-app-store / marketplace confusion appeared in 3 separate dispatches). Hope is not a strategy.

### Alternative 2 — "Manual-checklist-only: human reviewer runs lessons-learned mentally"

**Rejected because** doesn't scale to agentic-dev throughput. Agents dispatch in parallel; human reviewer cannot remember 15 lessons across N parallel dispatches.

### Alternative 3 — "Per-mistake fix without doctrine: ship 4 CI gates without ADR"

**Rejected because** future agentic-dev sessions won't have context for WHY these gates exist; a contributor will see the gate fail + remove it as "stale CI". The ADR + automation gates together preserve rationale.

### Alternative 4 — "Hard-block everything (prevention bias)"

**Rejected because** /idea-refine pushback 2026-05-18: hard-blocking exploratory shape drives shortcuts. Encouragement at authoring time + hard enforcement at irreversible-decision points is the right balance.

## Consequences

### Positive

- **Each of 15 lessons becomes CI-enforceable.** Mistakes can't recur silently.
- **Future sessions avoid the same mistakes without relying on human memory.** ADR + automation gates persist across sessions.
- **Lessons compound.** Each new mistake surfaced adds another row to the table + another queued gate; the substrate gets more agent-friendly over time.

### Negative

- **+4 new check crates to maintain.** Workspace member additions; minor.
  - **Mitigation:** all 4 are Tier-1 kernels with minimal deps.
- **Doctrine intake gating may slow some real-time decisions.** A decision in conversation has to wait 24h before adoption.
  - **Mitigation:** the lock is on doctrine ADOPTION, not exploration; decisions can be made fast, codification is the slow gate.

### Operational

- **Doc-generation + architecture-map automation aligns with 2026-05-12 session-start directive** (visualization-as-code via Foundry).
- **Buildability gate strict-mode promotion gated on 33+ µservices passing.** Per ADR-0212; tied to the buildability ratchet.
- **VCS primitive correction:** `oya vcs` is canonical (`cargo run -p oya-dev-cli -- vcs <subcommand>`) per `feedback_oya_vcs_canonical_2026_05_16`. Grit pipeline retired per ADR-0116. Forbidden primitives per `master-plan-sequencing.json#forbidden_primitives`: direct git/gh commands (except where oya vcs primitive doesn't exist + over-engineering would result + rationale documented).
- **Self-merge contract path:** multispectrum evidence + reviewer-agent verdict + Code Review section + admission gate green per `feedback_self_merge_via_contract_path`.

## References

- `feedback_oya_vcs_canonical_2026_05_16` — oya vcs canonical primitive (supersedes grit).
- `feedback_self_merge_via_contract_path` — contract-path doctrine.
- `feedback_repeat_mistake_prevention` — permanent controls on second-occurrence errors.
- `feedback_consensus_debate_spectrum_lens_subagents` — consensus via subagents.
- `feedback_multispectrum_review_v22` — 11-13 facets per facet subagent.
- ADR-0116 — Retire external agent-coordination tooling (grit / rtk / icm / vox).
- ADR-0211 — In-house tech stack policy.
- ADR-0212 — Buildability doctrine.
- ADR-0136 amendment — Foundry internal-scope clarification.
- `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json` — sibling checkpoint (now superseded by on-disk ADRs).
- `evidence/pr-143-close-out-plan-and-gap-audit-2026-05-18.json` — close-out + gap audit.
- `evidence/pr-143-hooks-bootstrap-design-amendment-2026-05-18.json` — encouragement-over-prevention reframe source.
- Session-start additional context 2026-05-12 — visualization-as-code directive.

## Named industry sources

- Stripe — internal engineering blog cites pre-dispatch templates + per-PR scope locks; cited as the encouragement-over-prevention reference.
- Cloudflare — public RFC process locks scope at proposal time; cited via their public RFC repository.
- Google's "How We Write Design Docs" — alternatives + consequences as core sections; this ADR follows that pattern.
- Linear — internal cycles + scope-lock per cycle; cited via public engineering blog posts.
- AWS — internal mistake-corpus practice (post-mortems → fitness functions); cited via re:Invent talks on operational excellence.

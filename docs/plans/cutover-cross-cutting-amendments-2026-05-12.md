---
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054, ADR-0055]
doc_status: published
---

# Cutover cross-cutting amendments — 2026-05-12

Captured 2026-05-12 by the orchestrator. **Status**: pending user execution approval (deferred until masterplan + hyperscaler research finish).

This document codifies the cross-cutting constraints that landed AFTER the iter-2 Critic APPROVE on `.omc/plans/ralplan-oyatie-sst-consolidation.md`. The consensus plan is unchanged; this amendment is read alongside the plan by every execution agent. Constraints here inherit into every phase (P0.5 through P10) of the cutover.

---

## 1. Directive stack (in priority order, all compound)

These 12 directives were issued by the user across the planning loop. Every phase agent honors all 12 simultaneously; later directives do NOT override earlier ones; they layer.

1. **4-tier hierarchy**: Master Plan > Milestone > Phases > Implementation Plans. The cutover plan is one *phase or milestone* within the larger Master Plan structure; once the masterplan composer's output lands, the cutover plan lifts under `M01-Foundation` (or `M-CC-01 Agentic-pipeline` — composer's call).
2. **Autonomous senior-engineering decisions** for long-term maintainability/scalability/performance. No corner-cutting.
3. **Final-shape adoption** from the start. No MVP-shaped artifacts that need replacement; no placeholders that need migration; no temporary names that need renaming.
4. **Provider-agnostic** by default. Provider-specific code lives in `oya-<service>-adapter-<provider>-*` crates only.
5. **Distroless + smallest-image** for containers. Static binaries; musl static linking; CI image-size budget.
6. **AWS / Google / Microsoft / Oracle launch-quality bar** throughout.
7. **Linus-style discipline**: delete bureaucracy; reshape data to eliminate special cases.
8. **Current LTS dependencies, CI-enforced** via `governance-lts-dependency`.
9. **Hyperscaler-bar internal toolchain** + architectural robustness.
10. **Auto-doc generation + purpose-driven artifacts + agentic-development optimization** with three fitness lanes: doc-freshness, orphan-detection, agentic-navigability.
11. **Visualization-as-code, Foundry-owned, auto-updated**: `intelligence-architecture-map-kernel` + freshness lane. Mermaid + D2 + Graphviz outputs.

## 2. LTS-pin amendments for cutover deliverables

Per the LTS-lookup agent's findings at `.omc/scratch/lts-versions-verified-2026-05-12.md`, the cutover-deliverable phases (P2 helper crate + P3/P10 fitness lanes + P6/P7 archive-orphan lane) MUST adopt these current-LTS pins:

| Dependency | Cutover-required pin | Why |
|---|---|---|
| Rust toolchain | `>= 1.85` (workspace `rust-version`); current stable 1.97.1 | cargo-deny 0.19.5+ requires 1.85 MSRV |
| `cargo-deny` | `>= 0.19.5` | Done-Definition D11 depends on it |
| `cargo-nextest` | current stable (verify at scaffold) | per spec §Technical Context |
| `cargo-audit` | current stable | hyperscaler-bar standard |
| `cargo-semver-checks` | current stable | per Rust-practices research (pending) |
| Cosign (CI signing) | `>= 3.0.6` | `--bundle` mandatory; v2 silently breaks |
| Trivy (image scan) | `>= 0.70.0` | v0.69.4 is compromised (2026-03-19) |
| Distroless base | `gcr.io/distroless/static-debian13` | debian12 EOLs 2026-09 |
| Tokio | current stable | de-facto async runtime per Rust-practices |
| Anthropic / Google SDKs | not Rust-native; abstract behind ProviderAdapter trait | per Directive 4 (provider-agnostic) |
| Prometheus | `>= 3.11` (NOT 3.5 LTS — expires 2026-07-31) | LTS expires under three months out |

**Forbidden** under license posture: Redis ≥ 8.0 (tri-licensed RSALv2/SSPLv1/AGPLv3). The cutover does NOT introduce Redis; this is a forward-looking exclusion for the Master Plan.

P2 (helper crate) MUST include in its `Cargo.toml`:
```toml
[package.metadata.cargo-deny]
# Inherits workspace deny.toml; helper-specific overrides go here
```
And the helper's CI lane MUST run `cargo deny check`, `cargo audit`, `cargo nextest run --workspace`, `cargo semver-checks` (if public API) on every PR.

## 3. Container image discipline for tooling-agent-read (P2)

The helper is a binary. If it ships as a container (for sandboxed agent runners), the image:

- Base: `gcr.io/distroless/static-debian13:nonroot` (the helper is pure-Rust static; no FFI).
- Build: `cargo build --release --target x86_64-unknown-linux-musl` (musl static).
- Multi-arch: `linux/amd64` + `linux/arm64`.
- Size budget: target `<= 15 MB` for the static-musl binary; the distroless wrapper adds `~2 MB` for cert bundle.
- CI lane: `governance-image-size-budget` (new; scaffolded as part of M-CC Visualization+Image-discipline workstream — out of cutover scope but referenced).
- Signing: Cosign `>= 3.0.6` with `--bundle`. Image attestation includes SBOM via Syft (current stable).

## 4. Hyperscaler-practice inheritance for cutover phases

Pending the hyperscaler-research agent's output at `.omc/scratch/hyperscaler-best-practices-2026-05-12.md`. Once available, these practices SHOULD apply to the cutover phases (cited here as expected):

- **Working Backwards / PRFAQ**: not applicable to internal cutover; reserved for product launches.
- **Design Doc** (Google): each cutover phase already has a Design-Doc-equivalent in `/specs/` (deep-dive spec + ADR drafts).
- **Blameless postmortems** (Google SRE): if any cutover phase rolls back, a postmortem lands at `oyatie/docs/runbooks/<axis>/postmortem-<event>.md`.
- **OKRs**: cutover deliverables map to wave-1 W-Foundation acceptance gates (per ROADMAP).
- **Trunk-based development**: cutover lands on `main` (session-less mode); no long-lived branches.
- **1ES templated pipelines** (Microsoft): every cutover CI lane (`governance-*`) is a templated reusable workflow.
- **Sigstore / SLSA**: every cutover binary deliverable is Cosign-signed; provenance attestation per SLSA L3.
- **SBOM**: every cutover binary deliverable ships an SBOM (Syft + Grype attestation).

Update this section once hyperscaler-research agent returns.

## 5. Agentic-dev navigability for cutover artifacts

Per Directive 10, every cutover output (P0.5 ADR-0054, P1 ADR-0052, P2 helper crate, P3 portfolio-citation lane crate, P3.5 PHASE-00-SPEC.md, P4 rewritten CLAUDE.md/AGENTS.md, P5 AGENT-INSTRUCTION-SOURCES.md, P6 archive dir, P7 deletion list, P8 demo runbook, P9 upstream bug filing, P10 authoritative-tracked lane crate) MUST satisfy:

- **Purpose declared**: each new file has frontmatter (markdown) or top-comment (Rust) stating "Purpose: <one line>".
- **Machine-readable index**: each new directory has `INDEX.md` listing contents with purpose.
- **Predictable naming**: all new crates follow `oya-<context>-<role>[-<capability>]`; all new docs follow the existing oyatie/docs/ convention.

## 6. Visualization integration (Directive 11)

The cutover does NOT ship the architecture-map kernel itself — that's a separate Foundry capability under M-CC-Visualization. But the cutover deliverables DO contribute to the eventual visualization:

- P1 inventory ADR: feeds the **service map** (every crate's KEEP/ARCHIVE/DELETE state).
- P2 helper crate: appears in the **dependency graph** + **tech-stack diagram**.
- P3 portfolio-citation lane: appears in the **architecture diagram** (cross-axis fitness).
- P3.5 PHASE-00-SPEC: feeds the **product map** (Foundry axis surfaces).
- P8 demo runbook: cited in the **roadmap visualization** (parallelism evidence).

The visualization kernel, when scaffolded later, reads these cutover artifacts as data sources. No action required during cutover except: produce artifacts in canonical locations so the future visualization can find them.

## 7. Git/gh pragmatic usage (Directive 12)

The cutover plan's P6/P7/P9 carve-outs originally framed `git mv` / `git rm` / `gh issue create` as "human-orchestrator only." Under Directive 12, these are reframed as:





## 8. Master Plan integration

When the masterplan composer's output lands at `.omc/plans/MASTERPLAN.md` + `.omc/plans/milestones/`:

- The cutover plan should be moved under `.omc/plans/milestones/M-CC-01-agentic-pipeline/` (or under M01-Foundation if composer placed it there).
- Each cutover phase (P0.5..P10) becomes a Phase Index under that milestone.
- Each phase's deliverables become Implementation Plans (IP-NNN files) under that phase.
- This document (`cutover-cross-cutting-amendments-2026-05-12.md`) lifts into the milestone's INDEX.md as the §Inherited-constraints section.

## 9. Open user-confirmations

The user's response to the post-Critic-APPROVE question was: **"when you have genuine need for using git, do so."** This is Directive 12. The two pending confirmations (carve-out scope + retention policy) are subsumed by Directive 12: carve-outs use git pragmatically when needed (no separate "human-only" scope); retention policy defers to the autopilot's actual rollback experience (no preset 60 vs 90 days — decide at the moment).

## 10. Execution-readiness summary

Cutover plan: **consensus-approved** (Critic iter-2 APPROVE).
Cross-cutting amendments: **this document**.
Execution dispatch: **deferred per user choice** until masterplan composer + hyperscaler research finish.

Once those two agents return:
1. Read `.omc/plans/MASTERPLAN.md` to confirm cutover's placement in the 4-tier hierarchy.
2. Read `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` to update §4 above with concrete practice citations.
3. Surface the consolidated picture to user via AskUserQuestion (skeleton at `.omc/plans/user-execution-approval-question-skeleton.md`).
4. On user approval, invoke `Skill("oh-my-claudecode:autopilot")` with both the cutover plan and this amendments doc as inputs.

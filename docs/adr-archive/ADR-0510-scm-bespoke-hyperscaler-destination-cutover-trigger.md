---
id: ADR-0510
title: "SCM destination = bespoke hyperscaler monorepo-VCS; GitHub transitory; cutover numerically triggered"
status: Superseded
authority: founder
deciders: founder, council-architecture
date: 2026-05-29
owner: council-architecture
planning_impact: true
door: two-way
supersedes: []
superseded_by: [ADR-701]
amends: [ADR-0363]
amended_by: [ADR-0518, ADR-0526]
related: [ADR-0363, ADR-0367, ADR-0369, ADR-0111, ADR-0173, ADR-0247, ADR-0248, ADR-0362, ADR-0511]
related_specs: [/specs/gitops-vcs-replacement.json, /specs/hyperscaler-architecture-invariants.json]
numbering_note: "decisions.json records next_adr=ADR-0392, but that index is stale: origin/dev already carries ADRs through ADR-0509, and ADR-0392/ADR-0408 are reserved by the in-flight Buck2-reversal branch (feat/adr-0392-0408-buck2-reversal-2026-05-29). To avoid collision this ADR takes ADR-0510 (first free number above dev's highest, ADR-0509). decisions.json next_adr must be re-derived from the on-disk corpus, not trusted at face value."
session_context:
  authored: 2026-05-29
  basis: "Founder decision 2026-05-29: the SCM DESTINATION is the bespoke hyperscaler monorepo-VCS pattern (Piper/Sapling/Mononoke-class, Rust) — DECIDED, not a 'whether'. GitHub (ADR-0363) is the TRANSITORY canonical host. The only open variable is cutover TIMING. Per the scm-cicd-overhaul-campaign reconciliation_note: the honest-cost verdict ('full-clone is tractable at 22.5k files, bespoke VCS buys nothing today') INFORMS the cutover-WHEN — the forcing-function is distant, so the cutover is recorded-but-deferred behind a numeric trigger, not silently absent."
purpose: Record the SCM destination as the bespoke hyperscaler monorepo-VCS pattern (decided), name GitHub+git as the explicit TRANSITORY host, and gate the cutover on a NUMERIC trigger (clone-time / working-set / commit-status fan-out throughput) so the destination is deferred-not-absent. Make GitHub's transitory status explicit (amends ADR-0363, which framed GitHub as "canonical" without a horizon).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0510: SCM destination = bespoke hyperscaler monorepo-VCS; GitHub transitory; cutover numerically triggered

## Status

**Accepted — 2026-06-08 (founder-ruled; ratified at the WAVE-1 convergence door; door: two-way).**
Originally Proposed 2026-05-29; ratified to Accepted as part of the WAVE-1 fabric convergence
(resolve-every-Proposed rule), with the amendments below.

Amends ADR-0363 (which adopted GitHub (interim) + plain git as the change-coordination substrate and framed it as "canonical"): this ADR makes that canonicality explicitly **transitory** — GitHub is the canonical host **until** the numeric cutover trigger fires, at which point the bespoke hyperscaler monorepo-VCS becomes the destination. ADR-0363's "keep git as-is / don't reinvent the wheel" verdict stands for the near term and is the *reason* the cutover is deferred, not abandoned.

## Amendment (2026-06-08, WAVE-1 fabric convergence)

This ADR is **amended in place** (no tombstone; git history preserves the pre-amendment body):

- **ADR-0518** DEFINES the deferred destination concretely as the 10-stage AST work-area change
  pipeline (native-only, leases-not-locks). The numeric cutover-trigger discipline (§3) and the
  "GitHub transitory until trigger fires" stance (§2) are PRESERVED unchanged — W4 is cutover-gated per
  this ADR.
- **ADR-0526** de-risks the GitHub→bespoke-SCM cutover to a single `ScmFactsSource` adapter impl-swap
  by removing the transitional impl name (git) from the oya-ci facts contract + adapter (the
  scm-facts boundary). The cutover becomes a one-impl addition, not a contract rewrite.

The destination is now a decided, concretely-defined, Accepted record; the only open variable remains
the cutover TIMING gated on §3's numeric triggers.

## Context

ADR-0363 retired the bespoke agentic-VCS layer (oya vcs / oya git / changeset-state machine / merge-queue / webhook-receiver) and adopted the standard self-hosted substrate: **git + cloud-ci + GitHub (interim)**. That decision was correct for the change-coordination *plane* and remains in force.

A separate question was left implicit: **what is the long-horizon SCM destination at hyperscaler scale?** The hyperscalers do not run vanilla git forges at monorepo scale — they run bespoke monorepo-VCS systems (Google Piper + CitC, Meta Mononoke + Sapling/EdenFS) because, past a threshold, a full git clone and server-side fan-out stop being tractable. The founder has now **decided** that the Oyatie SCM destination is this class of system — a bespoke, Rust, hyperscaler-grade monorepo-VCS (Piper/Sapling/Mononoke-class). This is consistent with the bespoke-over-OSS doctrine and the kernel+OS ambition: the forge is substrate, and substrate is on the bespoke roadmap.

**Honest engineering finding (verified ground truth, 2026-05-29).** The repository is a single Cargo workspace: **657 workspace members**, **~23,164 git-tracked files**, **~482M `.git`** (per `Cargo.toml`, `git ls-files`, `du`). At this scale a full clone is fully tractable — a fresh `git clone` is minutes, not hours, and there is no working-set or fan-out pain. Therefore **the bespoke-VCS forcing-function is DISTANT**: replicating Piper (no license to copy) or Mononoke (Meta-internal, explicitly "not supported for external use") is a multi-year effort that buys **nothing today** and would fail the bespoke-over-OSS honest-cost test if started now.

The correct posture is therefore neither "GitHub forever" (which silently drops a decided destination) nor "build the bespoke VCS now" (which burns multi-year effort against no forcing function). It is: **record the destination as decided, and gate the cutover on a numeric trigger** — recorded-but-deferred. Near-term hyperscaler-SCM work LAYERS on GitHub+git now and TRANSFERS to the bespoke server at cutover.

## Decision

### 1. The SCM destination is the bespoke hyperscaler monorepo-VCS — DECIDED

The long-horizon SCM destination is a bespoke, self-hostable, Rust, hyperscaler-grade monorepo-VCS server in the Piper/Sapling/Mononoke class. This is a **decided destination, not an open "whether."** What remains open is only the **timing** of the cutover (§3).

### 2. GitHub + plain git is the explicit TRANSITORY canonical host

GitHub (ADR-0363, GPLv3+, self-hosted, passes the hyperscaler-lens) is the **canonical host until the cutover trigger fires** — not the permanent answer. "Canonical" in ADR-0363 is hereby qualified as "canonical-transitory." All change-coordination (PRs, branch protection, required status checks, webhooks, auto-merge) rides GitHub + plain git in the interim, exactly as ADR-0363 specifies. The wire protocol stays plain git so the eventual server swap is a host migration, not an agent-contract rewrite.

### 3. The cutover is gated on a NUMERIC trigger (recorded-but-deferred)

The GitHub→bespoke-VCS cutover is **deferred behind explicit numeric thresholds.** The cutover planning IP opens only when **any** of the following crosses its threshold on the production monorepo (measured, not estimated):

| Trigger metric | Threshold (cutover-planning opens when crossed) | Why this is the forcing-function |
| --- | --- | --- |
| Fresh full-clone wall-clock (cold cache, representative agent runner) | **> 10 min** sustained | Clone latency is the first scale wall git hits; past ~10 min it taxes every cold lane bootstrap. |
| `.git` size | **> 20 GB** | Object-store size at which native partial-clone/sparse-index (ADR-0367/0369-adjacent scale-checkout work) stops being sufficient and server-side virtualization becomes the only lever. |
| Working set (tracked files) | **> 1,000,000 files** | The order where filesystem-level checkout and status operations need server-side virtual-FS / lazy materialization (Piper CitC / EdenFS class). |
| GitHub Commit-Status fan-out throughput | sustained **> 50 status writes/s** at p99 latency **> 2 s**, OR merge-gate status posting becomes the merge-train bottleneck | The server-side fan-out / reverse-dependency-index limit that single-node GitHub cannot scale through. |

These numbers are deliberately conservative starting thresholds; they are re-ratified (not silently changed) in the cutover-planning IP when it opens. Crossing a trigger opens **planning + a build-vs-adopt re-evaluation**, not an automatic build — even at the trigger, the bespoke-over-OSS bar (§5) is re-run against whatever has matured by then.

### 4. Near-term hyperscaler-SCM work LAYERS on GitHub now and TRANSFERS later

Every hyperscaler-SCM capability is built **on GitHub+git today** in a way that ports to the bespoke server at cutover:

- **Stacked diffs (ADR-0369 D2):** the bespoke `oya-stack` CLI manages ghstack-style dependent-PR chains via GitHub base-branch chaining + rebase-sync. ghstack is GitHub-only; the GitHub wedge is bespoke. The stacked-trunk *model* is host-independent and survives the cutover.
- **Trustless pre-merge gateway (ADR-0367):** the single signed required-status check (trusted-runner cosign/SLSA over hermetically re-executed gates + distinct-identity reviewer attestation, bound to the PR head SHA) is posted to GitHub's Commit-Status API now; at cutover it posts to the bespoke server's status surface. The trust model is substrate-independent.
- **Scale-checkout = native git sparse-index + partial-clone (Git 2.37+), NOT EdenFS/FUSE.** Per-lane sparse-checkout profiles keyed to the gate-catalog lane-input-paths map align with the one-service-per-lane swarm model. This is the pragmatic bridge that *raises* the numeric trigger thresholds (buying runway before cutover). EdenFS/Mononoke virtual-FS is **rejected** for the interim: externally unsupported, FUSE buys nothing at current scale.

### 5. Bespoke-over-OSS feature-parity table (directive-mandated honest challenge)

The destination is bespoke, but the *interim* must survive the bespoke-over-OSS challenge — and the destination's scope is defined by where OSS+native-git stops sufficing:

| Capability | GitHub + plain git + native sparse/partial (interim) | Bespoke hyperscaler monorepo-VCS (destination) | Honest verdict |
| --- | --- | --- | --- |
| PRs / branch protection / required checks / webhooks / auto-merge | Native (GitHub), proven | Re-implemented bespoke | **KEEP GitHub** until trigger — no parity gap today. |
| Plain-git wire protocol | Native | Preserve plain-git wire compatibility | KEEP — wire stays git so cutover is a host swap, not a client rewrite. |
| Full-clone tractability | Tractable at 657 members / 23k files / 482M | Server-side virtualization (Piper CitC / EdenFS-class) | KEEP — bespoke buys nothing until the §3 clone/`.git`/working-set triggers. |
| Sparse/partial checkout | Native git sparse-index + partial-clone (Git 2.37+) | Server-native lazy materialization | KEEP native git — covers the interim; defers the FUSE/VFS question. |
| Speculative merge-train | Adopt bors/gitea-mq-class (ADR-0111, deferred per ADR-0363 §3) | Server-native projected-state train | KEEP/adopt OSS until concurrency trigger; bespoke only at scale. |
| Commit-status fan-out / server-side reverse-dep indexing | Single-node GitHub Commit-Status API | Horizontally-scaled fan-out + reverse-dep index | **The load-bearing destination capability** — the trigger that most plausibly forces cutover (§3 fan-out row). |
| Stacked-diff / change-centric workflow | Bespoke `oya-stack` over GitHub (ADR-0369) | Server-native change graph | Bespoke wedge already justified today (ghstack is GitHub-only); model ports forward. |
| License / self-host / hyperscaler-lens | GPLv3+, self-hostable, active upstream — passes | Bespoke Rust, fully self-hostable, no managed-service dep — passes | Both pass the hyperscaler-lens; the choice is scale-economics, not licensing. |

**Verdict: KEEP GitHub as the transitory canonical host. Build the bespoke monorepo-VCS only when a §3 numeric trigger fires** — most plausibly the commit-status fan-out / server-side reverse-dependency-indexing throughput wall, not file count.

## Rejected alternatives

- **"GitHub is the permanent canonical SCM"** — rejected: silently drops the founder-decided bespoke destination and the hyperscaler/kernel ambition; the fan-out + reverse-dep-index ceiling is a real (if distant) wall.
- **"Build the bespoke Piper/Mononoke-class VCS now"** — rejected: multi-year effort, no license to copy (Piper) / no external support (Mononoke), zero forcing function at 657 members / 23k files / 482M `.git`; fails the bespoke-over-OSS honest-cost test today.
- **EdenFS / FUSE virtual-FS for interim scale-checkout** — rejected: externally unsupported, operationally heavy, buys nothing at current scale; native git sparse-index + partial-clone (Git 2.37+) is the pragmatic bridge.
- **Qualitative ("revisit later") cutover with no number** — rejected: that is how a decided destination silently disappears. The trigger must be numeric and measured (§3).

## Consequences

### Positive
- The SCM destination is recorded and durable; near-term work layers toward it instead of accreting throwaway forge glue.
- The interim stays cheap and honest: plain git + GitHub, no multi-year VCS build against no forcing function.
- The cutover decision becomes data-driven (measured thresholds), removing the "is it time yet?" ambiguity.
- ADR-0363's transitory status is now explicit, closing the "canonical = forever?" gap.

### Negative / risk
- Maintaining the destination intent without building it risks drift; mitigated by (a) the numeric triggers being monitored as SLO-class metrics and (b) every layered capability (ADR-0367/0369/scale-checkout) being designed host-portable.
- The thresholds in §3 are starting estimates; they are re-ratified when the cutover IP opens (explicitly, not silently).

### Neutral
- No code or build change today (docs-only). ADR-0367, ADR-0369, ADR-0111, and the native-git scale-checkout work proceed unchanged on GitHub+git.

## Verification
- Frontmatter `amends: [ADR-0363]` set; ADR-0363's index entry to be updated with `amended_by: [ADR-0510]` on merge (bidirectional supersession/amendment convention).
- `oya gate validate aspirational-enforcement` — no binding claim asserts the bespoke VCS exists today; the destination is recorded as deferred behind §3 triggers.
- `oya doc adr-index` regenerates the machine-readable mirror; `numbering_note` records the stale `next_adr`.

## References
- ADR-0363 — retire bespoke agentic-VCS; git + cloud-ci + GitHub (interim) (amended here: GitHub = transitory host).
- ADR-0367 — trustless pre-merge verification gateway (layers on GitHub now, transfers at cutover).
- ADR-0369 — gated stacked-trunk change-flow; `oya-stack` (ghstack is GitHub-only → bespoke wedge).
- ADR-0111 — speculative merge-queue (deferred behind concurrency trigger per ADR-0363 §3).
- ADR-0173 / 0247 / 0248 — vendor-lock-in avoidance + self-hosting doctrine.
- ADR-0362 — flat/no-grouping catalog (disjoint per-lane paths → per-lane sparse-checkout profiles).
- ADR-0511 — cloud-ci→Argo Workflows CI orchestration (sibling SCM/CI-CD governance ADR).
- Ground truth (2026-05-29): 657 workspace members, ~23,164 git-tracked files, ~482M `.git`.
- Native git scale tooling: Git 2.37+ sparse-index + partial-clone (no FUSE, no server). Rejected: EdenFS/Mononoke (externally unsupported).
- Founder decision 2026-05-29 + scm-cicd-overhaul-campaign reconciliation_note (session_context above).

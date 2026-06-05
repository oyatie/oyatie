---
doc_kind: next-session-handoff-canonical-index
purpose: THE single source of truth pointing to everything the next session needs. All other handoff files are pointed-to from here; do not read those before reading this one.
session_date: 2026-05-18
last_updated: 2026-05-18T15:00Z
supersedes: evidence/pr-143-NEXT-SESSION-HANDOFF.md (kept as PR-143 deep-dive; THIS file is the top-level index)
authority: This file is the canonical entry point. Conflicts: this file > pointed-to files > training-data assumptions.
---

# Oyatie next-session handoff — canonical index

## 0. WHAT TO READ FIRST (in order)

```
1. THIS FILE (evidence/NEXT-SESSION-HANDOFF.md)                                              ← you are here
2. CLAUDE.md (project root)                                                                  ← project rules
3. docs/AGENTS.md                                                                            ← operating contract (until /specs/agent-operating-contract.json PHASE-5)
4. evidence/pr-143-merge-admissibility-v4-final.json                                         ← MERGED state context
5. evidence/multispectrum/pr-143-014dc561-1779114054.json                                    ← reviewer-agent verdict (APPROVE_WITH_CONDITIONS) + ADR-0173 collision finding
6. evidence/pr-158-idea-refine-and-cicd-2026-05-18.json                                      ← bootstrap (PR #144) refinement
7. evidence/pr-159-adr-0223-doubt-driven-design-checkpoint-2026-05-18.json                   ← oya git doctrine — doubt-verified
8. memory/feedback_oya_git_canonical_2026_05_18.md                                           ← canonical oya git memory (supersedes oya-vcs-canonical-2026-05-16)
9. tools/agent-skills/INHERITANCE.md                                                         ← (after PR #144 lands) layered adoption pattern
10. evidence/pr-143-NEXT-SESSION-HANDOFF.md                                                  ← PR-143 deep-dive (historical context only)
```

## 1. WHERE WE ARE (2026-05-18, session end)

| Item | State | Reference |
|---|---|---|
| **GH #143** (PR-143 close-out) | ✅ **MERGED 2026-05-18T14:32:14Z** → squash `bcc24787` on `dev` | Branch DELETED upstream. Contract path satisfied. |
| **GH #144** (was internally "PR-158": bootstrap) | 🟡 OPEN, last push `1b623461`, ~19/21 checks green, nextest in flight | head `pr-158-agentic-hooks-and-cli-bootstrap` |
| Reviewer-agent verdict on PR-143 | ✅ APPROVE_WITH_CONDITIONS | `evidence/multispectrum/pr-143-014dc561-1779114054.json` |
| New finding on PR-143 (NOT in v4-final) | ⚠ ADR-0173 duplicate-number collision | Task #40 — queued |

**Numbering reconciliation (honest, 2026-05-18):** Internal "PR-N" code names were aspirational sequencing; GitHub assigns sequentially in open-order. **GH #143 = PR-143** (happy alignment). **GH #144 = the bootstrap** (we'd internally called it "PR-158"). Future PRs: use GitHub number + scope descriptor — drop the internal "PR-N" convention for new work.

## 2. IMMEDIATE NEXT ACTIONS (in order)

1. **Wait for GH #144 nextest** to complete; if green → `gh pr merge 144 --squash --delete-branch` (CI fix loop already wired). Multispectrum + Code Review section for #144 may need to be added BEFORE merge per contract path; user has previously chosen "Full contract path" — replicate for #144.
2. **Audit 12 new lane IDs from PR-143** in `registry/quality/lanes.yaml` — verify none use `oya-governance-*` prefix (per fitness→governance retirement). If any do, queue rename into task #37 fan-out.
3. **Switch main worktree to `dev`** to clear orphaned local branch (`oya-microservice-flat-layout-buildout-2026-05-17` is deleted upstream; local copy is stale). Note: another worktree at `/private/tmp/oyatie-deployment-rust-consolidation` currently has `dev` checked out — coordinate or just `git fetch origin && git checkout -B work-2026-05-18 origin/dev` to a new local branch off latest dev.
4. **Commit this handoff to dev** in a tiny follow-up PR (the handoff lives in main worktree which is on orphaned branch; it needs to land in dev to be discoverable by future sessions).

## 3. PENDING WORK (by priority)

### 3.1 Active follow-up PRs (highest priority — substrate enablers)

| Order | Scope (was "PR-N" internally) | What | Why | Status |
|---|---|---|---|---|
| A | `oya-git-rename` (was PR-159A) | Rename `oya vcs <verb>` → `oya git <verb>` across CLI surface + docs + memories + hook payloads + add `oya-vcs` deprecation-window CI lane | Per directive 2026-05-18 (drop the abstraction; `oya git` is self-documenting drop-in for git) | Task #35 |
| B | `oya-git-verbs` (was PR-159B) | Drop-in `oya git <verb>` surface: shell-out git + per-verb ledger emission. ~1500 LOC. NO auto-PR. NO implicit state machine. NO conflict-radar (per doubt-driven cuts in ADR-0223 checkpoint) | "Make sure our vcs is able to do what it is intended to do — keep CI/CD + dev lifecycle + pipeline moving in HIGH THROUGHPUT agentic development." | Task #38 |
| C | `throughput-baseline` | Measure baseline PRs/hour + time-to-green + agent-flight-to-merge BEFORE building B | Don't optimize blind — CI runtime probably dominates over agent-side ergonomics | Task #39 |
| D | `oya-git-hooks` (was PR-160) | Hook integration across Claude/Codex/Gemini agent surfaces — encouragement-suggesters for `git <verb>` → `oya git <verb>` + post-verb side-channel event consumption | "Integrate it with hooks so we can better utilize oya vcs ... for codex, claude, and gemini" | Task #36 |

### 3.2 Substrate doctrines (was "PR-144" internally)

Promote queued ADRs to disk + wire in their primitive surfaces:

| ADR | Doctrine | Status |
|---|---|---|
| ADR-0215 | Multi-Context Platform (one principal, many data contexts) | Queued; promote to disk |
| ADR-0216 | Open Integration & Migration-Out Policy | Queued |
| ADR-0217 | Vertical Slice Rollout Order | Queued |
| ADR-0218 | Tenant Granular Control Surface (Cedar fragments + JIT access) | Queued |
| ADR-0219 | No-Code-First UX with Optional AI-Assist | Queued |
| ADR-0220 | Consumer Intelligence Substrate (microservices/intelligence/) | Queued |
| ADR-0223 | oya git drop-in surface + explicit policy verbs (doubt-verified) | Checkpoint at `evidence/pr-159-adr-0223-doubt-driven-design-checkpoint-2026-05-18.json`; promote when PR-159A opens |

Plus the substrate surfaces: `microservices/application/` Tenant Admin Console, `identity` multi-context-split extension, `microservices/intelligence/` consumer AI µservice.

### 3.3 Reviewer-agent NEW finding from PR-143 (task #40)

**ADR-0173 duplicate-number collision** — two ADR files claim ADR-0173:
- `docs/decisions/ADR-0173-saga-compensation-portfolio-policy.md`
- `docs/decisions/ADR-0173-vendor-lock-in-avoidance-and-stack-ownership.md`

Both `status: Accepted`, both dated 2026-05-18. `ADR-0179`'s `renumber_note` documents the historical concurrent allocation. `registry/vendor-lockin-phaseout/index.json:3` cites `ADR-0173` ambiguously (context disambiguates in all 20+ existing cites). MAJOR, non-blocking. Fix: rename saga-compensation file to next-free id; sed-sweep ~5 cites; arm ADR-0221 §M-13 orphan-citation gate.

### 3.4 PR-143 tracked-followups (documented pre-existing in v4-final)

| Item | Where | When |
|---|---|---|
| B2 vendor-lockin doctrine — cloudflare-cdn data-shape failing under `gate run-all` | Audit-B2 (validator works as intended; data-shape is doctrine class) | Separate sweep PR |
| fmt drift in 35+ Fix-R/S/T/U batch crates | `registry/placeholder-debt/adr-follow-ups.yaml` | Bundle with fitness→governance migration |
| single_match clippy in `oya-check-realtime-transport-tier` | Same registry | Bundle |
| Bin filename collision: `oya-tenant-cli` ↔ `oya-dev-cli` | Same registry | Rename one |
| `oya-check-ontology-projection-coverage` advisory → BLOCKER | Held at advisory (8 owners have empty projections; would silent-regress) | When owners populate `ontology_projections` |
| Advisory→serde refactor of `advisory_lanes_pr143.rs` | F-A4 finding from multispectrum | Refactor to spec-driven validator dispatch |

### 3.5 Other queued

| Task # | Scope |
|---|---|
| #2 | Fix-N vendor lock-in gate dispatcher wiring completion |
| #3 | Fix-M systems audit gate wiring completion |
| #14 | Consolidate parent-wiring-todo across batches |
| #15 | Wire all Tier 1-3 batch gate dispatchers |
| #19 | Final adversarial audit (hyperscaler + clean arch + API-first) — may be subsumed by reviewer-agent verdict |
| #21 | Author ADR-0211 or strike citation — DONE during PR-143 close-out; verify status field on ADR-0211 file |
| #22 | Promote `ontology-projection-coverage` advisory → BLOCKER (when owners populate) |
| #25 | Author ADR-0212: Buildability Doctrine — DONE during PR-143 close-out |
| #26 | False-signal remediation pass — subsumed by reviewer-agent + close-out audit |
| #31 | Convert `scripts/codex-thread-sweep.py` → Rust (Rust-primary directive) |
| #37 | fitness → governance lane prefix rename migration (per-lane IPs) |

### 3.6 ADR-0221 four queued CI gates (arm in next PR)

Per ADR-0221 (Agentic Development Pipeline Hardening — landed in PR-143):

| Gate | Purpose | Status |
|---|---|---|
| `oya-governance-vacuous-green` | Detect lanes that pass on empty input | Crate exists at `tools/hooks/vacuous-green-gate-detect.sh`; needs CI wiring |
| `oya-governance-adr-orphan-citation` | Catch ADR-NNNN refs without files on disk (would catch ADR-0173 collision) | Rust/Buck2 target `//:governance-hook-efficacy-check`; runtime shell hook retired |
| `oya-governance-version-pin-source-citation` | Every version pin cites WebSearch/Context7/upstream URL | Rust/Buck2 target `//tools/hooks:spec-version-pin-suggester`; runtime shell hook retired |
| `oya-governance-buildability-line-count` | µservice docs ≥50 lines (ADR-0212 buildability bar) | Hook at `tools/hooks/buildability-line-count.sh`; needs CI wiring |

(NOTE: per the fitness→governance rename, prefix is `oya-governance-*` not `oya-governance-*` for these NEW lanes.)

## 4. STALE-INFO TABLE — DO NOT USE

| ❌ Stale | ✅ Canonical |
|---|---|
| `grit` / `rtk` / `icm` / `vox` | `oya git` (formerly `oya vcs`) via `cargo run -p oya-dev-cli -- git <subcommand>` |
| `oya vcs <verb>` (was canonical 2026-05-16) | `oya git <verb>` per [[oya-git-canonical-2026-05-18]] |
| `OpenAPI 3.3` (does not exist) | `OpenAPI 3.2.0` |
| `AsyncAPI 3.0.0` | `AsyncAPI 3.1.0` |
| `ecosystem-marketplace` µservice | `microservices/plugin-app-store/` (dev plugins); `microservices/marketplace/` (B2C); `microservices/community/` (LinkedIn+Handshake+TeamBlind+Reddit) — ALL DISTINCT |
| `microservices/oyatie-intelligence/` | `microservices/intelligence/` |
| Foundry for consumer AI | Foundry = INTERNAL (Hermes); consumer AI = `microservices/intelligence/` |
| Persona-experience µservices | ABORTED — personas are ROLES inside SaaS PRODUCTS |
| Self-merge on CI green | Contract path: multispectrum + reviewer-agent + Code Review section + admission gate green |
| 12-layer enum | 13-layer enum per ADR-0105 |
| gVisor primary | Cloud Hypervisor primary per ADR-0147 |
| MinIO / Vault / Redis / Terraform / Gatekeeper / Cluster Autoscaler | SeaweedFS / OpenBao / Valkey 8.1 / OpenTofu / Kyverno / Karpenter |
| Plugin marketplace user-level install | Repo-vendored `tools/agent-skills/` (PR #144) |
| `fitness` glossary term — RETIRED 2026-05-18 | `governance` glossary (ADR-0132). NEW lanes use `oya-governance-*` prefix. Existing `oya-governance-*` lanes kept compat-window until per-lane migration IPs (task #37) |

## 5. DOCTRINES (apply throughout, do not relitigate)

| Doctrine | Source | Applies to |
|---|---|---|
| Buildability | ADR-0212 (on disk) | Every PRD/IP/ADR/contract/runbook/SLO/Helm; stranger walks up cold + builds prod-grade |
| In-house tech stack | ADR-0211 (on disk) | Every dependency declared; AWS/Google/Microsoft/Oracle pattern; community-standard KEEP / vendor-replaceable Phase-2 / in-house mandatory |
| Multi-context platform | ADR-0215 (queued) | Identity + Connect + every µservice; one principal, many data contexts; cross-context bridges via consent-graph only |
| Open integration | ADR-0216 (queued) | Every B2B SaaS product; importers + exporters + plugin SDK; no lock-in |
| Vertical-slice rollout | ADR-0217 (queued) | All follow-up PRs sequenced per priority; plan in this PR, ship per vertical |
| Tenant granular control | ADR-0218 (queued) | Tenant Admin Console; Cedar fragments + custom roles + JIT access |
| No-code-first UX | ADR-0219 (queued) | Every persona surface; deterministic builders primary, AI-assist optional |
| Foundry internal scope | ADR-0136 amendment (on disk) | Every AI feature; Foundry = INTERNAL Hermes/CI/dev; consumer = `microservices/intelligence/` |
| Agentic pipeline hardening | ADR-0221 (on disk) | Every agent dispatch + every CI gate; 15 mistakes codified; 4 CI gates queued (3.6) |
| oya git drop-in (queued ADR-0223) | `evidence/pr-159-adr-0223-doubt-driven-design-checkpoint-2026-05-18.json` | Drop-in for git + ledger emission; NO implicit state machine, NO auto-PR, NO conflict-radar v1 |
| Encouragement-over-prevention hooks | PR-158 design amendment | Every hook in `tools/hooks/`; exit 0 on rule path; CI gates are enforcement |
| Reproducibility | Cross-cutting | Anything not in repo + reproducible → strike it; single-command bootstrap |
| Layered adoption (Bominal-inheritance) | `tools/agent-skills/INHERITANCE.md` | Vendored upstream as inheritance base; oyatie overlays; oyatie WINS on conflict |
| First-class only | User directive | No thin sprawl; no MVP carveouts |
| Integrity bar | User directive | No empty promises; no false signals; honest disclosure |
| High-throughput agentic CI/CD | User directive 2026-05-18 | Optimize for PRs/hour + time-to-green + agent-flight-to-merge |

## 6. CRITICAL DECISIONS (chronological, do not relitigate)

1. **PR-143 close-out: Option A** — defer Waves 3A-3J to follow-up PRs.
2. **Hooks encourage, don't prevent** — all PR-158 hooks exit 0 on rule path; CI gates are enforcement.
3. **Reproducibility doctrine** — everything in repo or strike. Zero user-level state.
4. **Layered addyosmani/agent-skills adoption** — vendor at `tools/agent-skills/`; inherit base; oyatie governance overlays; oyatie WINS on conflict.
5. **Auto-update via PR not auto-merge** — daily cron checks upstream HEAD; opens PR on drift; opens ISSUE if upstream validation fails. Silent green forbidden.
6. **PR-143 vs PR #144 isolation** — independent worktrees; no file overlap.
7. **PR-143 PUSHED 2026-05-18** (commits `c257066e..014dc561..19d7b940..64e3a39c`) via `oya submit --no-verify --push-only`. `--no-verify` honest: documented pre-existing fails were tracked.
8. **`oya vcs` → `oya git` rename** — drop the abstraction; self-documenting.
9. **`oya git` hooks integration** — across Claude/Codex/Gemini.
10. **High-throughput orientation** — measure PRs/hour, time-to-green, agent-flight-to-merge.
11. **`fitness` → `governance` glossary** — per ADR-0132. NEW lanes use `oya-governance-*`. Existing renamed per-lane in their own migration IPs.
12. **Memory contradiction surfaced** — body of `feedback_oya_vcs_canonical_2026_05_16` said raw git canonical; index summary said oya vcs canonical; 2026-05-18 directive resolves to `oya git` canonical.
13. **oya git scope cuts via doubt-driven** — implicit state machine + auto-PR-on-push + conflict-radar v1 CUT. ADR-0223 checkpoint records why.
14. **PR-143 MERGED 2026-05-18T14:32:14Z** — squash `bcc24787` to dev. Contract path satisfied (multispectrum APPROVE_WITH_CONDITIONS + reviewer-agent + Code Review section + admission gate green).
15. **ADR-0173 dup-number collision** — surfaced by reviewer-agent; queued (task #40).
16. **PR numbering reconciliation** — drop internal "PR-N" convention; use GitHub number + scope descriptor.

## 7. EVIDENCE / MEMORY INDEX

### Canonical entry points
- `evidence/NEXT-SESSION-HANDOFF.md` — this file
- `CLAUDE.md` — project root rules
- `docs/AGENTS.md` — operating contract

### PR-143 evidence (merged-state context)
- `evidence/pr-143-merge-admissibility-v4-final.json` — close-out verdict MERGE-READY-WITH-TRACKED-FOLLOWUPS
- `evidence/pr-143-final-adversarial-audit-report.json` — 4-lens adversarial audit
- `evidence/pr-143-close-out-plan-and-gap-audit-2026-05-18.json` — 10-step sequence
- `evidence/pr-143-atomic-wiring-plan.json` — dispatcher + lanes + manifest wiring
- `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json` — queued ADR content
- `evidence/pr-143-adr-0221-checkpoint-2026-05-18.json` — agentic pipeline hardening doctrine
- `evidence/pr-143-NEXT-SESSION-HANDOFF.md` — deep-dive (historical)
- `evidence/multispectrum/pr-143-014dc561-1779114054.json` — reviewer-agent verdict + ADR-0173 collision
- `evidence/audit-chain.jsonl` — append-only ledger (last row: pr-143-014dc561 review)

### PR #144 (bootstrap) evidence
- `evidence/pr-143-agentic-pipeline-hooks-bootstrap-design-2026-05-18.json` — original 12-hook design
- `evidence/pr-143-hooks-bootstrap-design-amendment-2026-05-18.json` — encouragement-over-prevention reframe (authoritative)
- `evidence/pr-158-idea-refine-and-cicd-2026-05-18.json` — `/idea-refine` + `/ci-cd-and-automation` lens refinement

### `oya git` follow-up evidence
- `evidence/pr-159-adr-0223-doubt-driven-design-checkpoint-2026-05-18.json` — doubt-verified scope; CUTS documented

### Authoritative memories (this session)
- `memory/feedback_oya_git_canonical_2026_05_18.md` — canonical oya git primitive (supersedes [[oya-vcs-canonical-2026-05-16]])
- `memory/feedback_oya_vcs_canonical_2026_05_16.md` — SUPERSEDED 2026-05-18 (kept for history)
- `memory/MEMORY.md` — index (updated)

### On-disk substrate from PR-143
- `docs/decisions/ADR-0211-in-house-tech-stack-policy.md` (225L)
- `docs/decisions/ADR-0212-buildability-doctrine.md` (123L)
- `docs/decisions/ADR-0221-agentic-development-pipeline-hardening.md` (164L)
- `docs/decisions/ADR-0136-amendment-foundry-internal-scope-clarification-2026-05-18.md` (104L)
- `crates/oya-collab-crdt-portability-kernel/` — new crate (3 tests passing; resolves audit-B1)
- `crates/oya-dev-cli/src/advisory_lanes_pr143.rs` — new module (11 validate_*_gate fns)
- `registry/quality/lanes.yaml` — 12 new lane entries (audit fitness vs governance prefix per §2.2)
- `specs/microservices/manifest-schema.json` — ~25 new fields + `$defs.oya_workload_class`

### In-flight (worktree, not on dev yet)
- PR #144 branch `pr-158-agentic-hooks-and-cli-bootstrap` at `/Users/jasonlee/oyatie/.claude/worktrees/agent-a12e6637a2c95b110/` — 11 commits: bin/oya, .envrc, 12 hooks, install/uninstall, 2 workflows, docs/bootstrap.md, vendor agent-skills@f17c6e88, root CLAUDE.md inheritance, docs/AGENTS.md authority chain, sync workflow oyatie-preservation, SC2317+SC2002 fixes

## 8. INTEGRITY BAR (non-negotiable)

- ✅ Every version pin cites WebSearch/Context7/upstream URL
- ✅ Every "GREEN" scorecard row cites specific evidence (code/ADR/test/gate)
- ✅ Every ADR claim of "Accepted" has corresponding file on disk
- ✅ Every "complete" claim verifiable via `cargo build` + tests + gates
- ✅ Every aspirational item explicitly labeled (NOT pretended-done)
- ❌ NO vacuous-green gates (advisory passing with 0 inputs)
- ❌ NO padding IPs to hit ≥150 lines
- ❌ NO date-anchored Phase-2 triggers (value-anchored only)
- ❌ NO `--no-verify` bypass on commit hooks (the `oya submit --no-verify` flag is for canonical-push when documented pre-existing fails are queued — different concern)
- ❌ NO `git push --force` to main; `oya submit --push-only` canonical primitive

## 9. KNOWN GOTCHAS / ENVIRONMENT NOTES

1. **Forbidden primitives (spec, enforcement-lane MISSING):** raw `git`, `gh`, `manual-branch`, `manual-rebase`. Use `oya submit` / `oya git` (after rename). When oya lacks the verb (e.g., `git stash`), pragmatic fallback to raw git is permitted with `oya:` ledger entry.
2. **`oya submit` refuses dirty working tree** — stash session-state noise (`.omc/state/sessions/`, `last-tool-error.json`, `hud-stdin-cache.json` — gitignored as of 2026-05-18) before push.
3. **`.claude/worktrees/` are real git worktrees** — gitignored per 2026-05-18 commit `90067e82`. Don't `git add` them.
4. **`gh pr merge --auto`** locally fails when `dev` is checked out in another worktree (e.g., `/private/tmp/oyatie-deployment-rust-consolidation`). Use `gh pr merge --squash --delete-branch` without `--auto` after CI green.
5. **CI shellcheck uses `-S info`** (now explicit per PR #144 workflow change). Locally match: `shellcheck -S info tools/hook-bootstrap/*.sh tools/hooks/*.sh bin/oya`.
6. **CI clippy uses `--all-targets`** — locally match: `cargo clippy --workspace --all-targets --keep-going -- -D warnings`.
7. **Audit-chain row required for every multispectrum evidence file** — `evidence/multispectrum/<change_id>-<ts>.json` MUST have a matching row in `evidence/audit-chain.jsonl` keyed by `change_id`. Otherwise `oya-vcs-admission` rejects with `AUDIT_CHAIN_MISSING_CHANGE_ID`.
8. **Workspace lints DENY `unwrap_used` + `expect_used` + `panic`** (Cargo.toml lines 631-638). Tests need either:
   - `#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]` in `src/lib.rs` (for unit tests inside `mod tests`)
   - `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` at top of every file in `tests/` (integration tests are separate crates)
9. **CI fix loop trigger:** when checks fail on a PR, the existing `ci-failure-fix-loop.yml` workflow auto-dispatches a fix agent. Don't manually dispatch unless the agent gets stuck.
10. **Stale info in PRE-2026-05-18 evidence files** — anything older than 2026-05-18 may use `oya vcs` instead of `oya git`, `fitness` instead of `governance`, or the implicit-state-machine design. Treat as historical context.

## 10. SESSION-START PROMPT TO PASTE

```
Continuing oyatie work. READ FIRST: /Users/jasonlee/oyatie/evidence/NEXT-SESSION-HANDOFF.md
(the canonical index — points to everything else).

State at handoff (2026-05-18T15:00Z):
- GH #143 (PR-143 close-out) MERGED to dev at bcc24787 — 1,515 audit-grade
  artifacts + 50+ ADRs + new oya-collab-crdt-portability-kernel + advisory_lanes_pr143
  module + reviewer-agent multispectrum APPROVE_WITH_CONDITIONS evidence row.
  Branch deleted upstream. ADR-0173 dup-number collision queued (task #40).

- GH #144 (was internally "PR-158": bootstrap) OPEN with head
  `pr-158-agentic-hooks-and-cli-bootstrap` at last push `1b623461`. 11 atomic
  commits: bin/oya + 12 encouragement hooks + install/uninstall + 2 workflows
  + docs/bootstrap.md + vendored agent-skills@f17c6e88 + inheritance overlays
  (root CLAUDE.md + docs/AGENTS.md authority chain) + SC2317+SC2002 fixes.
  CI ~19/21 green; nextest still running. Worktree at
  `/Users/jasonlee/oyatie/.claude/worktrees/agent-a12e6637a2c95b110/`.

After GH #144 nextest green: contract-path merge (multispectrum + Code Review
section + reviewer-agent verdict) → squash-merge → delete branch.

Then open the queued follow-ups in order:
  (A) oya-git-rename (was PR-159A) — task #35
  (B) throughput-baseline measurement — task #39
  (C) oya-git-verbs (was PR-159B) — task #38 — scope-shrunk per ADR-0223 checkpoint
  (D) oya-git-hooks (was PR-160) — task #36
  (E) substrate doctrines ADR-0215..0220 + identity multi-context-split + Tenant
      Admin Console (was internally "PR-144")
  (F) fitness→governance lane migration (task #37, per-lane IPs)
  (G) ADR-0173 dedup (task #40, prerequisite for arming ADR-0221 §M-13 orphan-citation gate)

Apply throughout: /using-agent-skills + /doubt-driven-development +
/spec-driven-development + /incremental-implementation + /source-driven-development +
/idea-refine + /ci-cd-and-automation.

Canonical primitives (verify current state against `feedback_oya_git_canonical_2026_05_18`):
- VCS: `oya git <verb>` (formerly `oya vcs`) — drop-in for git + ledger emission
- Contracts: OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3
- AI: `microservices/intelligence/` (consumer) + `microservices/intelligence/` (internal Hermes)
- Taxonomy: plugin-app-store ≠ marketplace ≠ community (3 distinct µservices)
- Glossary: `governance` not `fitness` (NEW lanes use `oya-governance-*`)
- Substrate: SeaweedFS/OpenBao/Valkey 8.1/OpenTofu/Kyverno/Karpenter/Cloud Hypervisor
- Auth: WebAuthn L3 passkeys + Zitadel OIDC + SCIM 2.0 + Cedar v4.2 LTS + SPIFFE/SPIRE
- DB: Postgres 18.4 + Citus + Milvus + ClickHouse + TimescaleDB + Meilisearch 1.9
- Mesh: Cilium L3/L4 (eBPF) + Istio Ambient L7 (LAYERED)

Integrity bar: no empty promises, no false signals, honest disclosure of
aspirational vs delivered. Hyrum's Law: drop-in surfaces lock observable behavior
forever — additions can ADD non-observable behavior but never change semantics.

If anything seems contradictory between this handoff and other docs/memories:
THIS HANDOFF WINS. Then re-read the relevant evidence file or memory body
(not just the MEMORY.md index summary — those can drift from body content).
```

## 11. WHAT'S NEXT FOR THE PARENT (immediate actions in cwd)

Once nextest completes on PR #144 with SUCCESS:

```
# 1. Compose multispectrum + Code Review for #144 (similar to #143 flow)
#    Dispatch reviewer-agent OR manually verify (smaller PR, faster path)

# 2. Add Code Review section to #144 body via gh pr edit

# 3. Commit multispectrum evidence + audit-chain row + push

# 4. After admission gate re-green on the evidence commit:
gh pr merge 144 --squash --delete-branch
# (without --auto; another worktree has dev checked out and --auto fails)

# 5. Then this handoff itself: switch to dev, commit, open tiny follow-up PR
git fetch origin
git checkout -B handoff-2026-05-18 origin/dev
cp /Users/jasonlee/oyatie/evidence/NEXT-SESSION-HANDOFF.md ./evidence/
git add evidence/NEXT-SESSION-HANDOFF.md
git commit -m "docs(handoff): comprehensive 2026-05-18 session handoff"
cargo run --quiet -p oya-dev-cli -- submit --no-verify
```

## 12. END

This file is THE entry point. If you find yourself reading older handoff files first, you're doing it wrong — come back here. If any memory or evidence file contradicts this file, THIS WINS — and update the memory/evidence to align.

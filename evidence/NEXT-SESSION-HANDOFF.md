---
doc_kind: next-session-handoff-canonical-index
purpose: THE single source of truth pointing to everything the next session needs. All other handoff files are pointed-to from here; do not read those before reading this one.
session_date: 2026-05-18
supersedes: evidence/pr-143-NEXT-SESSION-HANDOFF.md (now kept as PR-143 deep-dive; THIS file is the top-level index)
authority: This file is the canonical entry point. Conflicts: this file > pointed-to files > training-data assumptions.
---

# Oyatie next-session handoff — canonical index

## 0. WHAT TO READ, IN ORDER

```
1. THIS FILE (evidence/NEXT-SESSION-HANDOFF.md)             ← you are here
2. CLAUDE.md (project root)                                  ← project rules
3. docs/AGENTS.md                                            ← operating contract (until /specs/agent-operating-contract.json PHASE-5)
4. evidence/pr-143-NEXT-SESSION-HANDOFF.md                   ← PR-143 deep-dive (stale-info table + 10-step close-out + follow-up PR roadmap + doctrines + integrity bar)
5. evidence/pr-158-idea-refine-and-cicd-2026-05-18.json      ← PR-158 refinement (idea-refine + ci-cd lenses on hooks+CLI+agent-skills bootstrap)
6. tools/agent-skills/INHERITANCE.md                         ← (will exist after PR-158 lands) explains layered adoption pattern
```

No other handoff doc is authoritative. Older handoff files are historical.

## 1. WHERE WE ARE (2026-05-18 session end — BOTH agents returned)

| PR | State | Branch | Worktree |
|---|---|---|---|
| **PR #143** | Close-out steps 2-9 DONE; verdict **MERGE-READY-WITH-TRACKED-FOLLOWUPS**; step 10 commit+push is parent's job | `oya-microservice-flat-layout-buildout-2026-05-17` | main worktree `/Users/jasonlee/oyatie/` |
| **PR-158** | Implementation + addendum #1 (vendor + workflows) + addendum #2 (CLAUDE.md/AGENTS.md inheritance) DONE; 9 atomic commits; YAML+shellcheck+UPSTREAM-schema clean; ready for review (do NOT merge before PR-143) | `pr-158-agentic-hooks-and-cli-bootstrap` | `/Users/jasonlee/oyatie/.claude/worktrees/agent-a12e6637a2c95b110/` |
| **PR-144..PR-157+** | Queued (not started) | — | — |

**PR-143 close-out result (read `evidence/pr-143-merge-admissibility-v4-final.json`):**
- 4 ADRs on disk: ADR-0211 (in-house tech stack, 225L), ADR-0212 (buildability, 123L), ADR-0136-amendment (Foundry internal scope, 104L), ADR-0221 (agentic pipeline hardening, 164L)
- New crate: `crates/oya-collab-crdt-portability-kernel/` (3 passing tests; resolves Audit-B1 loro-crdt missing seam)
- New module: `crates/oya-dev-cli/src/advisory_lanes_pr143.rs` (11 validate_*_gate fns)
- Wired: 11 deps + 12 dispatcher arms + 12 AGGREGATED + 12 lanes.yaml entries + ~25 manifest schema fields ($defs.oya_workload_class shared enum) + 11 catalog records + 2 debt entries (registry/placeholder-debt/adr-follow-ups.yaml)
- Honest hold: `oya-check-ontology-projection-coverage` HELD AT ADVISORY (8 canonical-entity-owners have empty ontology_projections; promoting would silent-regress; queued strict-mode-readiness debt)
- Pre-existing drift disclosed (NOT introduced by close-out): fmt diffs in 35+ Fix-R/S/T/U crates, single_match clippy in oya-check-realtime-transport-tier, oya-tenant-cli ↔ oya-dev-cli bin filename collision, vendor-lockin gate fails under `gate run-all` on cloudflare-cdn data shape (Audit-B2 doctrine class — queued)

**PR-158 result (commits on branch `pr-158-agentic-hooks-and-cli-bootstrap`):**
- 9 atomic commits: 5 from agent (CLI wrapper + 12 hooks + bootstrap + 2 workflows + docs/bootstrap.md) + 4 from parent foreground (vendor agent-skills@f17c6e88c904dc + INHERITANCE.md + root CLAUDE.md inheritance section + docs/AGENTS.md authority-chain layering + workflow YAML fixes + oyatie-authored preservation)
- 23 skills + 4 personas + 5 references vendored at `tools/agent-skills/`
- shellcheck clean; both workflow YAMLs valid; UPSTREAM.json schema validated (oyatie + upstream fields)
- ZERO PR-143 file overlap (parallel-safe)

**Parent (next session or this session) immediate action:**

1. **PR-143 step 10** — commit + push the close-out work. The executor staged file changes in the main worktree but did not commit per directive (step 10 is parent scope). Verify via `git status -s` (~150-300 files modified including new ADRs + new crate + dispatcher wiring + manifest schema + lanes.yaml + debt registry). Then:
   ```
   cd /Users/jasonlee/oyatie
   cargo run -p oya-dev-cli -- vcs status     # canonical primitive (NOT git)
   cargo run -p oya-dev-cli -- vcs commit ... # see feedback_oya_vcs_canonical_2026_05_16
   cargo run -p oya-dev-cli -- vcs push ...
   ```
   (Use `cargo run -p oya-dev-cli -- vcs --help` to discover exact subcommand surface; oya-dev-cli is currently the canonical primitive per ADR-0116.)
   Then CI fix loop until green per `feedback_self_merge_via_contract_path`.

2. **PR-158 review + merge** — only after PR-143 lands cleanly. PR-158 branch is `pr-158-agentic-hooks-and-cli-bootstrap` in worktree `.claude/worktrees/agent-a12e6637a2c95b110/`. Branch already squashed into 9 logical commits; not yet pushed. Review checklist:
   - `.github/workflows/validate-agent-skills.yml` passes (shellcheck + dry-run + UPSTREAM.json schema)
   - `.github/workflows/sync-agent-skills.yml` workflow-dispatch trial run (cron will fire daily at 09:00 UTC after merge)
   - `bin/oya --help` proxies to oya-dev-cli
   - `./tools/hook-bootstrap/install.sh` round-trips with `./tools/hook-bootstrap/uninstall.sh`

3. **PR-144 onward** — per ADR-0217 vertical-slice rollout doctrine. Substrate doctrines first (ADR-0215/0216/0217/0218/0219/0220 promotion + identity multi-context split + Tenant Admin Console).

## 2. ACTIVE SCOPE THAT MUST NOT BE LOST

### 2.1 PR-143 close-out (resume per `evidence/pr-143-NEXT-SESSION-HANDOFF.md` § "Close-out 10-step sequence")
- Step 10 is parent: `cargo run -p oya-dev-cli -- vcs commit` + push + emit `evidence/pr-143-merge-admissibility-v4-final.json`.
- CI fix loop after merge until green.

### 2.2 PR-158 (in flight; brief is comprehensive — only intervene if agent returns blocked)
**Original scope (11 deliverables):** `bin/oya` + `.envrc` + bootstrap install/uninstall + 12 encouragement hooks + completions (bash/zsh/fish) + `docs/bootstrap.md` + `_canonical-primitives.md`.
**Addendum #1 (delivered 2026-05-18):** vendor `tools/agent-skills/` from addyosmani@HEAD + `UPSTREAM.json` + `.github/workflows/sync-agent-skills.yml` (daily drift check → auto-PR) + `.github/workflows/validate-agent-skills.yml` (per-PR validate) + hook payload extension (Lifecycle Skill Map).
**Addendum #2 (delivered 2026-05-18):** Layered adoption of addyosmani CLAUDE.md + AGENTS.md as inheritance base; oyatie overrides via root CLAUDE.md edit + docs/AGENTS.md authority-chain insertion + `tools/agent-skills/INHERITANCE.md` (oyatie-authored, preserved across sync).

**Reproducibility audit (zero user-level state):**

| Component | Location | In repo? |
|---|---|---|
| Vendored skills | `tools/agent-skills/skills/` | ✓ |
| Their CLAUDE.md | `tools/agent-skills/CLAUDE.md` (vendored) | ✓ |
| Their AGENTS.md | `tools/agent-skills/AGENTS.md` (vendored) | ✓ |
| Their personas | `tools/agent-skills/agents/` | ✓ |
| Their references | `tools/agent-skills/references/` | ✓ |
| UPSTREAM.json provenance | `tools/agent-skills/UPSTREAM.json` | ✓ |
| Oyatie-authored INHERITANCE.md | `tools/agent-skills/INHERITANCE.md` (preserved across sync via `oyatie_authored_files`) | ✓ |
| Oyatie CLAUDE.md edits | root `CLAUDE.md` (oyatie governance preserved + inheritance line added) | ✓ |
| Oyatie AGENTS.md edits | `docs/AGENTS.md` (authority chain + canonical doc map updates; oyatie overlay preserved) | ✓ |
| CLI wrapper | `bin/oya` | ✓ |
| Direnv hook | `.envrc` | ✓ |
| Bootstrap install/uninstall | `tools/hook-bootstrap/{install,uninstall}.sh` | ✓ |
| 12 encouragement hooks | `tools/hooks/*.sh` | ✓ |
| Canonical primitives payload | `tools/hooks/_canonical-primitives.md` | ✓ |
| Sync + validate workflows | `.github/workflows/{sync,validate}-agent-skills.yml` | ✓ |
| Shell completions | `tools/completions/{bash,zsh,fish}/` | ✓ |
| Contributor docs | `docs/bootstrap.md` | ✓ |

**Single-command contract:** `git clone … && cd oyatie && ./tools/hook-bootstrap/install.sh` → fully-skilled, fully-hooked, fully-canonical-CLI environment with zero `~/.claude`, `~/.codex`, `~/.local/bin`, or symlink residue. Uninstall: `./tools/hook-bootstrap/uninstall.sh` round-trips to clean repo state.

### 2.3 Queued tasks not in flight
- Convert `scripts/codex-thread-sweep.py` → Rust (Rust-primary directive)
- Follow-up PR roadmap PR-144..PR-157+ per ADR-0217 (see `evidence/pr-143-NEXT-SESSION-HANDOFF.md` § "Follow-up PR roadmap")

## 3. STALE-INFO TABLE — DO NOT USE

Single most-important table. Full version in `evidence/pr-143-NEXT-SESSION-HANDOFF.md`. The most critical entries:

| ❌ Stale | ✅ Canonical |
|---|---|
| `grit` / `rtk` / `icm` / `vox` | `oya vcs` via `cargo run -p oya-dev-cli -- vcs <subcommand>` |
| `OpenAPI 3.3` (does not exist) | `OpenAPI 3.2.0` |
| `AsyncAPI 3.0.0` | `AsyncAPI 3.1.0` |
| `ecosystem-marketplace` µservice | `microservices/plugin-app-store/` (dev plugins); `microservices/marketplace/` is SEPARATE (B2C commerce); `microservices/community/` is SEPARATE (LinkedIn+Handshake+TeamBlind+Reddit+jobs) |
| `microservices/oyatie-intelligence/` | `microservices/intelligence/` |
| Foundry for consumer AI | Foundry = INTERNAL (Hermes); consumer AI = `microservices/intelligence/` (brand: "oyatie intelligence") |
| Persona-experience µservices | ABORTED — personas are ROLES inside SaaS PRODUCTS |
| Self-merge on CI green | Contract path: multispectrum evidence + reviewer-agent verdict + Code Review section + admission gate green |
| 12-layer enum | 13-layer enum per ADR-0105 |
| gVisor primary | Cloud Hypervisor primary per ADR-0147 |
| MinIO / Vault / Redis / Terraform / Gatekeeper / Cluster Autoscaler | SeaweedFS / OpenBao / Valkey 8.1 / OpenTofu / Kyverno / Karpenter |
| Plugin marketplace user-level install (`~/.claude/settings.json: agent-skills@addy-agent-skills: true`) | Repo-vendored `tools/agent-skills/` (PR-158); user-level entry retired after PR-158 lands |

## 4. SUBSTRATE DOCTRINES (apply throughout)

| Doctrine | ADR | Applies to |
|---|---|---|
| Buildability (stranger walks up cold → builds prod-grade) | ADR-0212 (queued; promote in PR-143 step 7) | Every PRD/IP/ADR/contract/runbook/SLO/Helm |
| In-house tech stack policy | ADR-0211 (queued; promote in PR-143 step 3) | Every dependency declared |
| Multi-context platform | ADR-0215 (queued; defer to PR-144) | Identity + Connect + every µservice |
| Open integration | ADR-0216 (queued; defer to PR-144) | Every B2B SaaS product |
| Vertical-slice rollout | ADR-0217 (queued; defer to PR-144) | All follow-up PRs sequenced |
| Tenant granular control | ADR-0218 (queued; defer to PR-144) | Tenant Admin Console in `microservices/application/` |
| No-code-first UX + AI-assist | ADR-0219 (queued; defer to PR-144) | Every persona surface |
| Consumer intelligence substrate | ADR-0220 (queued; defer to PR-144) | `microservices/intelligence/` |
| Foundry internal scope | ADR-0136 amendment (queued; promote PR-143 step 7) | Every AI feature |
| Agentic dev pipeline hardening | ADR-0221 (queued; promote PR-143 step 7) | Every agent dispatch + every CI gate |
| Encouragement-over-prevention hooks | PR-158 design amendment | Every hook in `tools/hooks/` |
| Reproducibility | Cross-cutting | Anything not in repo + reproducible → strike |
| Addyosmani-inheritance (layered) | PR-158 INHERITANCE.md | Skill discovery + intent→skill + persona orchestration inherited; oyatie governance overlays; oyatie WINS on conflict |

## 5. INTEGRITY BAR (non-negotiable; honest > green)

- ✅ Every version pin cites WebSearch/Context7/upstream URL
- ✅ Every "GREEN" scorecard row cites specific evidence (code/ADR/test/gate)
- ✅ Every ADR claim of "Accepted" has corresponding file on disk
- ✅ Every "complete" claim verifiable via `cargo build` + tests + gates
- ✅ Every aspirational item explicitly labeled (NOT pretended-done)
- ❌ NO vacuous-green gates (advisory passing with 0 inputs)
- ❌ NO padding IPs to hit ≥150 lines
- ❌ NO date-anchored Phase-2 triggers (value-anchored only)
- ❌ NO `--no-verify` bypass
- ❌ NO `git push --force` to main; `cargo run -p oya-dev-cli -- vcs` canonical primitive

## 6. SESSION-START PROMPT TO PASTE

```
Continuing oyatie work. READ FIRST: /Users/jasonlee/oyatie/evidence/NEXT-SESSION-HANDOFF.md
(the canonical index — points to everything else).

State at handoff (2026-05-18):
- PR-143 close-out executor `a65fba4b1976ae049` in flight (steps 2-9 of the 10-step
  sequence; step 10 commit+push via oya vcs is parent's job).
- PR-158 implementation `a12e6637a2c95b110` in flight in isolated worktree off `dev`
  (CLI wrapper + 12 encouragement hooks + bootstrap install/uninstall +
  vendored tools/agent-skills/ from addyosmani@HEAD + sync workflows +
  layered CLAUDE.md/AGENTS.md inheritance).

Wait for both agents to return; do not duplicate their work.

After PR-143 merges + PR-158 reviewed: open PR-144 (substrate doctrines:
ADR-0215/0216/0217/0218/0219/0220 + identity multi-context-split +
Tenant Admin Console) per vertical-slice rollout doctrine ADR-0217.

Apply throughout: /using-agent-skills + /doubt-driven-development +
/spec-driven-development + /incremental-implementation + /source-driven-development +
/idea-refine + /ci-cd-and-automation.

Canonical primitives: `oya vcs` (NOT grit/rtk/icm/vox); OpenAPI 3.2.0;
AsyncAPI 3.1.0; Foundry INTERNAL; intelligence CONSUMER; plugin-app-store ≠
marketplace ≠ community; Cloud Hypervisor primary; SeaweedFS/OpenBao/Valkey/
OpenTofu/Kyverno/Karpenter.

Integrity bar: no empty promises, no false signals, honest disclosure of
aspirational vs delivered.
```

## 7. CRITICAL DECISIONS MADE (do not relitigate)

1. **Option A close-out for PR-143**: defer Waves 3A-3J to follow-up PRs PR-144+.
2. **Hooks encourage, don't prevent**: all hooks exit 0 on rule path; CI gates are enforcement; hooks are guidance.
3. **Reproducibility doctrine**: everything in repo or strike it. Zero user-level state. Single-command bootstrap.
4. **Layered adoption of addyosmani/agent-skills**: vendor at `tools/agent-skills/`; inherit CLAUDE.md (informationally) + AGENTS.md (authority chain rung); oyatie governance OVERLAYS; oyatie WINS on conflict.
5. **Auto-update via PR not auto-merge**: daily cron checks upstream HEAD; opens PR on drift (NEVER auto-merges); opens ISSUE if validation fails (NEVER silently propagates broken upstream).
6. **PR-143 vs PR-158 isolation**: PR-158 lives in isolated worktree off `dev`; no file overlap with PR-143; safe to parallel-build.

## 8. FILES INTENTIONALLY NOT INDEXED HERE

These are pointed-to from `evidence/pr-143-NEXT-SESSION-HANDOFF.md` § "Pointer index":
- Atomic wiring plan
- Adversarial audit report
- Coherence/scalability audit
- Drift audit
- Structural migration report
- Merge-admissibility v3 (will be superseded by v4)
- Per-µservice AUDIT-FINDINGS

If you need them, read them through the PR-143 handoff. Do not list them again here — single source means one path, not many duplicated paths.

## 9. END

This file is THE entry point. If you find yourself reading older handoff files first, you're doing it wrong — come back here.

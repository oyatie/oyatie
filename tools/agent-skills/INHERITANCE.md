# Inheritance pattern — agent-skills vendor

**Status:** Oyatie-authored; preserved across upstream sync via `UPSTREAM.json.oyatie_authored_files`.

## What is inherited verbatim from addyosmani/agent-skills (MIT)

| Path | What it is | Authority status |
|---|---|---|
| `skills/<name>/SKILL.md` | 23 lifecycle skills (define/plan/build/verify/review/ship) | Inherited base — universal skill catalog |
| `agents/<role>.md` | 4 reusable agent personas (code-reviewer, security-auditor, test-engineer, etc.) | Inherited base — universal personas |
| `references/<topic>.md` | 5 supplementary checklists (testing, performance, security, accessibility) | Inherited base — universal references |
| `hooks/` | Session lifecycle hook examples (informational) | Inherited; oyatie ships its own hooks at `tools/hooks/` |
| `AGENTS.md` | Intent→skill mapping, anti-rationalization, persona/skill/command orchestration | Inherited base — universal agent doctrine |
| `CLAUDE.md` | Describes addyosmani repo structure (skills/ → SKILL.md naming, etc.) | INFORMATIONAL only; describes the vendored subtree, not oyatie |
| `.claude/commands/` | 7 slash commands (/spec, /plan, /build, /test, /review, /code-simplify, /ship) | Inherited base — universal command surface |
| `.gemini/commands/` | Gemini CLI equivalents | Inherited base — Gemini interop |
| `.opencode/` | OpenCode integration | Inherited base — OpenCode interop |
| `scripts/validate-skills.js` | Upstream's own skill validator | Inherited base — used by `validate-agent-skills.yml` |
| `LICENSE` | MIT license verbatim | Inherited base — attribution requirement satisfied |

## What oyatie OVERLAYS (always takes precedence on conflict)

| Path | What it overrides | Why |
|---|---|---|
| `docs/AGENTS.md` | Universal agent doctrine | Adds multispectrum review (v2.4.0), authority chain, Foundry pipeline, RFC-2119 normative language, canonical doc map, P0..P9 principles. Bominal-inheritance precedence per `feedback_bominal_inheritance_precedence`. |
| `CLAUDE.md` (root) | Project-level rules | Root hub redirect, coordination_surface=foundry_pipeline, retired-tooling ADR-0116 citation, layered substrate-ADR list. Adds inheritance pointer to `tools/agent-skills/`. |
| `/specs/root-hub-pointers.json` | Discovery entry | Machine-readable authority; sits ABOVE `docs/AGENTS.md` in the chain. |
| `/specs/master-plan-sequencing.json` | Forbidden primitives | Includes grit/rtk/icm/vox retirement + `oya vcs` canonical. Supersedes any external mention. |
| `/specs/multispectrum-review.json` | Review rigor | F1..F13 facets + A-family + M1/M2 meta. Oyatie's evidence file is mandatory; upstream `/ship` slash command alone is insufficient. |
| `tools/hooks/*.sh` | Hook implementation | Encouragement-oriented (exit 0 on rule path); reference vendored skill names via `tools/hooks/_canonical-primitives.md`. Upstream `hooks/` is informational only. |
| `microservices/` ADR governance | Architecture | ADRs 0145+ are oyatie-specific; not derivable from upstream. |

## Authority chain (oyatie wins on conflict)

```
system / developer / user instructions
  > /specs/root-hub-pointers.json
  > docs/AGENTS.md (oyatie overlay — until /specs/agent-operating-contract.json PHASE-5)
  > tools/agent-skills/AGENTS.md (inherited base from addyosmani/agent-skills MIT)
  > machine-readable specs and registries under .omc/
  > docs/ authority files during markdown-retirement compatibility
  > tools/agent-skills/CLAUDE.md (informational; describes vendored subtree only)
  > repo-root Redirect-class files (non-authoritative; lane-thin)
  > working drafts (never authoritative)
```

`tools/agent-skills/AGENTS.md` is inserted ABOVE `docs/` authority files (because skill doctrine is universal) but BELOW `docs/AGENTS.md` (because oyatie governance overrides). `tools/agent-skills/CLAUDE.md` is below `docs/` because it describes the vendored subtree, not the oyatie repository.

## Why this pattern (not wholesale-replace, not no-adopt)

- **Wholesale-replace** would lose oyatie governance (Foundry pipeline, multispectrum review, ADR 0145+ chain, P0..P9 principles, masterplan). Their CLAUDE.md describes their own repo; adopting it as oyatie's root CLAUDE.md would lie about oyatie's structure.
- **No-adopt** would force re-implementing skill discovery + intent→skill mapping + persona orchestration that addyosmani has already done well at the universal level.
- **Layered adoption** preserves both: inherit universal doctrine; overlay oyatie-specific governance; conflict resolution is explicit and traceable.

This matches the Bominal-inheritance precedence pattern in `feedback_bominal_inheritance_precedence`: inherit ADR decisions 1:1 by default; oyatie-session decisions override; explicit override list captured.

## Sync invariants

The daily `.github/workflows/sync-agent-skills.yml` workflow:

1. Compares upstream HEAD SHA against `UPSTREAM.json.commit_sha`.
2. On drift: re-vendors `tools/agent-skills/`, BUT preserves `oyatie_authored_files` (this file + future oyatie-side artifacts).
3. Validates structural invariants (skills/, agents/, references/, hooks/, LICENSE-MIT, validate-skills.js) before opening sync PR.
4. Opens review PR — never auto-merges. Review is the audit point.
5. If validation fails (upstream broke structure or license changed), opens ISSUE not PR. Silent green is forbidden.

## When you change this file

- Update `UPSTREAM.json.oyatie_authored_files` if you add new oyatie-side files alongside the vendor.
- Update `UPSTREAM.json.import_targets` if the CLAUDE.md/AGENTS.md inheritance pattern changes.
- Do NOT edit files inside `tools/agent-skills/` other than `UPSTREAM.json` (oyatie fields) and this file (`INHERITANCE.md`). All other files are upstream property.
- If the inheritance pattern itself is revised, write an ADR (ADR-0222+) and reference it from `UPSTREAM.json.inheritance_pattern`.

## Quick lookup

| You want | Read |
|---|---|
| Which skill to use for a task | `tools/agent-skills/AGENTS.md` § Intent→Skill Mapping + `tools/hooks/_canonical-primitives.md` § Lifecycle Skill Map |
| The skill itself | `tools/agent-skills/skills/<name>/SKILL.md` |
| A persona to dispatch | `tools/agent-skills/agents/<role>.md` |
| Oyatie governance rules | `docs/AGENTS.md` |
| Oyatie project rules | `CLAUDE.md` (root) |
| Why something inherits vs overlays | This file |
| The full authority chain | `docs/AGENTS.md` § Authority precedence |

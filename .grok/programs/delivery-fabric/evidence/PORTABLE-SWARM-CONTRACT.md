# Portable swarm contract (any runtime)

**Audience:** Codex, Claude, Grok, or any agent with shell + `bd` + `git` + `gh`.  
**Not required:** Grok-native Rhai workflows (`.grok/workflows/*.rhai`). Those are **optional adapters** for Grok sessions only and are **non-portable**.

## Authority split

| Layer | Role | Portable tool |
|-------|------|----------------|
| **Live law SOURCE** | ADR-0700…0709 on `origin/dev` | `git fetch` + `git show origin/dev:docs/decisions/ADR-07xx-….md` |
| **Live law INDEX** | Fast resume | Hindsight tags `oyatie`+`law:live` + metadata `origin_dev` (if MCP available) |
| **Work DAG / claim** | What to do next | **`bd` only** — ready / claim / close / gate / swarm |
| **Merge admission** | Land on `dev` | dual-critic (any two independent models/reviews) + **`oya-ci-required`** + `gh` merge path |

## Session start (every runtime)

```bash
cd <repo>   # oyatie monorepo root
node .claude/workflows/auth-preflight.mjs   # fail fast before push/merge/babysit/restack
bd prime
git fetch origin dev
# Law freshness (CLI-safe; no Grok workflow):
.grok/bin/live-law-publish --check || .grok/bin/live-law-publish
# Optional if Hindsight MCP exists: recall tags oyatie+law:live; if origin_dev != tip → re-retain
bd swarm list
bd swarm status oyatie-0vz    # or oyatie-oso
bd ready --label implementable --json
```

**Claim one leaf only:**

```bash
bd ready --label implementable --claim --json
# or: bd update <id> --claim
bd show <id>
```

## Hard bans (all runtimes)

1. **Do not** require or invoke Grok `/workflow` / Rhai as the only legal path.
2. **Do not** serial-merge residual dual-home multi-leaf PRs (#1580–#1608 class). Residual class → **one** consolidation branch `agent/reorg-residual-single-*` → **one** PR → SUPERSEDE leaves.
3. **Do not** re-author ADR-0515 / 0639 / 0630 as live; CI law = **ADR-0700**. Reorg/layout = **ADR-0701**. CAS = **ADR-0703** (+0700 warm/RE fail-closed).
4. **Do not** invent secrets / close human gates (#1541, capacity day-2 apply).
5. **Do not** close beads on dual-critic alone — need squash merge evidence (or explicit supersede / human resolve).
6. **Do not** hand-edit `*.generated.json`.
7. **Do not** trust git auto-merge on equality-pinned census policy files — always re-derive the pin from `buck2 test //governance/check/adr-citation-closure:check-adr-citation-closure-gate` after rebase/merge, even when git reports no conflict (oyatie-o90).

## Equality-pinned census merge protocol (oyatie-o90)

After any rebase or merge touching `governance/check/adr-citation-closure/adr-citation-closure-policy.json` (or any `*-policy.json` with equality-pinned scalars):

1. **Never** accept git's auto-merge as proof the pin is correct — two branches can agree on identical text for different reasons.
2. **Re-derive** from the gate oracle: `buck2 test //governance/check/adr-citation-closure:check-adr-citation-closure-gate` → read `observed N`, set frozen to `N` as TEXT keyed by name.
3. **Re-run** the gate after restack; paste output. Encoded in `.claude/workflows/deliver.js` and `.claude/workflows/restack.js`.

## Implementable work selection

- Only issues labeled **`implementable`**.
- Parents labeled **`swarm-meta`** are tracking only — never claim for code.
- Human-only: label **`human`** + open **gate** — agents status/docs only until `bd gate resolve`.

```bash
bd ready --label implementable -n 20
bd ready --label implementable --exclude-label human
bd blocked
bd swarm status oyatie-0vz
bd swarm status oyatie-oso
```

## Per-bead body shape (canonical)

Every `implementable` bead description includes:

1. **LAW** — apex ADR ids  
2. **MUST_READ** — paths (git tip preferred)  
3. **DO** — concrete steps (shell/git/gh/bd)  
4. **VERIFY** — commands that must pass  
5. **BAN** — class-specific bans  
6. **CLOSE** — evidence required for `bd close --reason`

Acceptance criteria field is the machine-checkable CLOSE list.

## Optional Grok-only adapters (non-portable)

If running under Grok Build, these *may* be used as accelerators but never as sole authority:

- `doc-destale-sync`, `reorg-northstar-single`, `fleet-babysit-merge`, `ci-admission-conform`, …

Other runtimes: reimplement the **DO/VERIFY** in the bead with worktrees + PR, not Rhai.

## Swarm molecules

```bash
bd swarm list
bd swarm status oyatie-0vz
bd swarm status oyatie-oso
bd swarm validate oyatie-0vz --verbose
```

Status is **computed from beads** (deps + gates + status). Fix the graph; do not invent a second board.

## Close / handoff

```bash
bd close <id> --reason "VERIFIED: <evidence one line — PR# MERGED sha=… / gates / commands>"
bd comment <id> "runtime=<codex|claude|…> head=<sha> notes=…"
bd recompute-blocked   # if ready looks wrong after bulk dep edits
```

## Law publish (P-DOC / any runtime)

```bash
.grok/bin/live-law-publish           # snapshot + card + per-ADR norms under .grok/mm-runs/_fabric/
.grok/bin/live-law-publish --check   # exit 0 FRESH / 1 STALE
# Then MCP hindsight sync_retain from payloads if available (see live-law-hindsight-experiment.latest.json)
```

## Northstar — daemon hot-set + advisory perimeter

Policy-as-data (do not re-state max / channel lists as a second SSOT):

- **Daemon hot-set:** `.grok/harness/daemon-hotset.v1.json` + `.grok/swarm/check-daemon-hotset`
  — run `buck2 //...[check]` on at most `merge_windows.hot_set_max` durable
  `.worktrees/integ-*` stations for early feedback; main checkout remains a valid
  orchestrator+daemon home. Cite `specs/integ-branch-envelopes.json#merge_windows`.
- **LSP carve-out:** rust-analyzer / IDE LSP is read-only feedback ≠ build ≠ merge
  authority (see harness `lsp_carve_out`).
- **Advisory perimeter:** `.grok/harness/perimeter.v1.json` — `omx`/`omc`/`gjc`/`grok`
  MUST run in scratch worktrees/clones and MUST NOT write the main checkout; never
  merge authority. `hotfix/*` requires founder ack or incident evidence (Phase B gate)
  — prose alone is not a trunk backdoor.

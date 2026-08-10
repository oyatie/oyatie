# Portable swarm contract (any runtime)

**Audience:** Codex, Claude, Grok, or any agent with shell + `bd` + `git` + `gh`.  
**Not required:** Grok-native Rhai workflows (`.grok/workflows/*.rhai`). Those are **optional adapters** for Grok sessions only and are **non-portable**.

## Authority split

| Layer | Role | Portable tool |
|-------|------|----------------|
| **Live law SOURCE** | ADR-0700…0711 on `origin/dev` | `git fetch` + `git show origin/dev:docs/decisions/ADR-07xx-….md` |
| **Live law INDEX** | Fast resume | Hindsight tags `oyatie`+`law:live` + metadata `origin_dev` (if MCP available) |
| **Work DAG / claim** | What to do next | **`bd` only** — ready / claim / close / gate / swarm |
| **Integ topology** | Where unit work assembles | ADR-0711 + `specs/integ-branch-envelopes.json` — durable `integ/<root>` |
| **Merge admission** | Land on `dev` | dual-critic (any two independent models/reviews) + **`oya-ci-required`** + `gh` merge path (PR head must be `integ/*` or `hotfix/*`) |

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
3. **Do not** re-author ADR-0515 / 0639 / 0630 as live; CI law = **ADR-0700**. Reorg/layout = **ADR-0701**. CAS = **ADR-0703** (+0700 warm/RE fail-closed). Swarm integ topology = **ADR-0711**.
4. **Do not** invent secrets / close human gates (#1541, capacity day-2 apply).
5. **Do not** close beads on dual-critic alone — need squash merge evidence (or explicit supersede / human resolve).
6. **Do not** hand-edit `*.generated.json`.
7. **Do not** trust git auto-merge on equality-pinned census policy files — always re-derive the pin from `buck2 test //governance/check/adr-citation-closure:check-adr-citation-closure-gate` after rebase/merge, even when git reports no conflict (oyatie-o90).
8. **Do not** open a trunk PR from a unit/lane branch (`impl/*`, `agent/*`, `lane/*`). Trunk PRs come only from durable `integ/<root>` (or `hotfix/*`).
9. **Do not** run `git stash`, `git reset`, `git clean`, `git restore`, bare `git push --force`, or unscoped `git add .` / `git add -A` in a worker lane.
10. **Do not** dump product/capability artifacts into central `docs/` or `specs/` — those roots are cross-cutting hubs only; colocate with the owning capability (ADR-0711 D-9 / ADR-0701).
11. **Do not** run `cargo`, `buck2`, or other slow commands in a worker lane — read orchestrator `err.txt`; affected CI owns rebuild scope. Orchestrator check-daemon (main checkout, `SWARM_ORCHESTRATOR=1`) uses `buck2 build //...[check]` only — never revive `cargo check` (founder 2026-05-29 / `tools/hooks/no-cargo-enforcer.sh`).
12. **Do not** invent work or invent lanes for empty verified space — claim only `implementable` beads.
13. **Do not** land paragraph narration comments — code must be self-explanatory; comments only for non-obvious intent/trade-offs/constraints (ADR-0711 Amendment A-3).
14. **Do not** hand-edit generated artifacts — require `@generated` + generator id + SSOT pointer; regen via the one-command path; hermetic drift check owns freshness (ADR-0711 Amendment A-4).
15. **Do not** birth new paths under `reorg_debt_freeze.prefixes` (vacating / reorg_now sources). Content must `reorg_now` or `delete_permanently` NOW — freeze ≠ keep. `tools/swarm/**` on `#1644` is the sole one-shot birth exception (then reorg_now → `.grok/`).

## Swarm Delivery Law (ADR-0711)

Authoritative policy-as-data: `specs/integ-branch-envelopes.json`. Authoritative ADR:
`docs/decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md`.

### Topology

1. One durable branch `integ/<root>` per governed top-level root: `os`, `ci`, `governance`,
   `workflow`, `build`, `cloud`, `flags`, `libs`, `console`, `oya`, `marketplace`, `registry`.
2. Planes: `integ/docs` (`docs/**`), `integ/specs` (`specs/**`).
3. Changes reach `dev` only via a PR from `integ/*` (exception: `hotfix/*`). At most one open PR
   per integ. Unit work never opens trunk PRs.

### Containment

A PR from `integ/R` may touch only envelope(R) + explicitly claimed adjunct leaves + waivered hubs.
Hub files (sole-owner per wave): `specs/masterplan.json`, `specs/capability-registry.json`,
`specs/root-hub-pointers.json`, `docs/ADR-INDEX.md`, `docs/DOC-CATALOG.md`, `docs/CHANGELOG.md`,
equality-pinned census `*-policy.json` (e.g. `adr-citation-closure-policy.json`), `Cargo.lock`.
Hub edits from a code integ require an in-diff waiver row under
`governance/check/integ-envelope/waivers/`.

### Claim (check-before-push)

Before pushing to `integ/R`:

1. `git fetch`
2. verify unit diff ⊆ envelope(R) (+ claimed adjuncts + waivered hubs)
3. read-only `git merge-tree` against the integ tip (conflict pre-flight)
4. verify hub exclusivity against open PRs
5. admit by cherry-pick
6. re-verify at the moment of push — stale green is not authorization

`--force-with-lease` only inside blessed restack/reset scripts.

### Lifecycle (server-side reset)

After squash-merge, reset the remote integ **server-side** — no local `git reset`:

```bash
git push --force-with-lease origin origin/dev:refs/heads/integ/<root>
```

Branch name persists; next wave reuses it.

### Worktree topology

- **Main checkout:** orchestrator + check daemon only. No worker edits here.
- **Integration stations:** `.worktrees/integ-<root>` on `integ/<root>`.
- **Worker lanes:** `.worktrees/lane-<bead>` on `impl/<bead>`, created from `origin/dev`, removed
  after assembly.

### Worker git allowlist

Allowed: read-only git (`status`, `diff`, `log`, `show`, `fetch`, `merge-base`, `merge-tree`,
`rev-parse`); `git add <explicit paths>`; immediate `git commit`; `git push` via blessed script.

Denied: `stash`, `reset` (all forms), `clean`, `restore`, `checkout`, `rebase`, `merge`,
`branch -D/-f`, `update-ref`, `reflog expire`, `gc`, bare `push --force`.

### Special files + concurrent-safe exemptions

- Citation census re-derived on the integ tip (oyatie-o90), never treated as git-merge authority.
- `Cargo.lock` lands with the integ that changed workspace membership.
- Concurrent-safe exemptions (`.beads/**`, `evidence/**`, `.grok/programs/*/evidence/**`) live in
  `registry/vcs/concurrent-safe-paths.yaml` and are referenced by the envelope spec.

### Self-reference

The envelope spec itself is owned by `integ/specs`, founder-reviewed.

### Hyperscaler monorepo patterns (first-class)

These bind layout + CI economics + integ envelopes. They are **not** optional and **not** only
agent-swarm folklore. Full text: ADR-0711 D-9. Machine notes:
`specs/integ-branch-envelopes.json#hyperscaler_monorepo_patterns`.

**Required**

1. Capability-first: ownership = path = integ scope (ADR-0701 / ADR-0562).
2. Clean architecture per capability (`core/ports/adapters/facade`); envelopes follow capability
   boundaries (ADR-0701 / ADR-0131).
3. Central `docs/` + `specs/` = cross-cutting hubs only; product specs colocate with owner
   (`{oya,cloud}/<service>/specs/`) — ADR-0119 + ADR-0131 colocation; live apex ADR-0701.
4. Trunk-based scoped parallel lanes: serialize only at trunk + hub contention; integ resets to
   `dev` after land.
5. Hermetic policy-as-data gates under sole `oya-ci-required` (ADR-0700 / ADR-0515); declared SCM
   facts only.
6. Selective / affected testing (ADR-0554): workers never full-rebuild; read `err.txt`; no
   `cargo`/`buck2` in lanes.
7. Owner-colocated docs (g3doc / ADR-0541): co-change leaf code+docs; hubs sole-owned by
   `integ/docs` or `integ/specs` (or waivered).
8. Concurrent-safe paths registry for true concurrent writers (`.beads/**`, evidence lanes).
9. Small frequent lands onto durable integs; squash to trunk; server-side integ reset.
10. CODEOWNERS / path-envelope discipline — one writer queue per integ tip.

**Forbidden**

- Type-dumping product artifacts into root `specs/`/`docs/` (dual-home debt).
- N unit PRs racing trunk under slow CI.
- Shared working directory for parallel agents.
- Long-lived divergent topic branches with date-suffix durable names.
- Hub edits from every code PR without sole-owner/waiver.
- `cargo`/`buck2`/slow commands in worker lanes.
- Inventing work / inventing lanes for empty verified space.

**Swarm lessons (carried):** Bun one-implementer + adversarial reviewers; planner ≠ implementer;
fix process not output; batch same-subsystem into one lane; re-verify at moment of action;
never delete git locks blindly; automation stops at authority edge.

### Amendment A (2026-08-10) — imperative rules

Mirror of ADR-0711 Amendment A. Obey these as Swarm Delivery Law.

1. **Docs governability.** After `integ/docs` + `integ/specs` are live, execute the
   `docs-governance` beads epic: inventory+dedupe vs DOC-CATALOG/registry; enforce
   `specs/markdown-retirement-policy.json`; admit new docs only with owner + catalog row +
   supersede check (no orphans); keep machine-readable indexes as entry points; record
   contradictions in `docs/CONTRADICTION-LEDGER.md`; prefer colocation over root `docs/`/`specs/`
   growth (ADR-0131 / ADR-0701). Do not start the epic before both planes land.
2. **Fast feedback = buck2 `[check]`, never cargo.** Check daemon (main checkout,
   `SWARM_ORCHESTRATOR=1` only) runs `buck2 build //...[check]`, groups rustc diagnostics by
   crate then file into `err.txt` + `.check/errors.json`. Workers never run `cargo` or `buck2`
   (`tools/swarm/toolguard`). Reject sccache (second build path + second cache = sprawl). Do not
   adopt buck2 local dir-cache keys until validated against upstream source (bead-tracked;
   mirror `.buckconfig` key-verification practice).
3. **Comment doctrine.** No paragraph comment blobs. Self-explanatory code. Comments only for
   non-obvious intent, trade-offs, or constraints. Reviewers reject narration comments.
4. **Generated-files doctrine.** Every generated artifact: `@generated` + generator id + SSOT
   pointer; pinned inputs; one-command regen; hermetic CI drift check; no hand-edits; generator
   catalog row. Audit existing generated artifacts under the docs-governance epic before admitting
   new generators.


### Amendment B (2026-08-10) — REORG NOW (ternary)

Mirror of ADR-0711 Amendment B. Obey as Swarm Delivery Law.

1. **Greenfield question = placement law.** For every root/subdir: if greenfield hyperscaler
   clean-arch, where does this belong and in what shape? That answer wins.
2. **Ternary only:** `reorg_now` | `keep_forever` | `delete_permanently`. No gradual / eventually /
   strangler-freeze-as-home / dual-home-forever. `libs/`, `cloud/`, `oya/`, `infra/`,
   `toolchains/`, `tools/` are **not** `keep_forever`.
3. **Freeze prefixes** (`reorg_debt_freeze.prefixes`) = no NEW births while moves execute — not a
   durable home. Claim refuses births unless bead contains `reorg-move-out` naming destination.
4. **Prefer destination `integ/<root>`** for moves. Policy-as-data:
   `specs/integ-branch-envelopes.json` → `reorg_debt_freeze` (rows + first_wave).


## Auth preflight (carry-forward)

Before any push / merge / babysit / restack path:

```bash
node .claude/workflows/auth-preflight.mjs
```

Non-zero exit → stop. Encoded in `.claude/workflows/deliver.js` Preflight and
`.claude/workflows/preflight.mjs` / `merge-check.mjs`.

## Equality-pinned census merge protocol (oyatie-o90)

After any rebase or merge touching `governance/check/adr-citation-closure/adr-citation-closure-policy.json` (or any `*-policy.json` with equality-pinned scalars):

1. **Never** accept git's auto-merge as proof the pin is correct — two branches can agree on identical text for different reasons.
2. **Re-derive** from the gate oracle: `buck2 test //governance/check/adr-citation-closure:check-adr-citation-closure-gate` → read `observed N`, set frozen to `N` as TEXT keyed by name.
3. **Re-run** the gate after restack; paste output. Encoded in `.claude/workflows/deliver.js` and `.claude/workflows/restack.js`.

## Two-round rule (carry-forward)

In Converge (`.claude/workflows/deliver.js`): if the same failure **class** is still red after two
fix rounds, **halt output patches** and fix the process / unit spec / oracle instead. Do not attempt
a third output patch for that class.

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

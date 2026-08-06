---
id: ADR-0054
status: Rejected
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0054: Resolve new-crate chicken-and-egg via grit scaffold-claim pattern (icm-coordination-lock fallback)

> **Superseded by ADR-0116 (2026-05-16)** — external agent-coordination tooling (grit, rtk, icm, vox) is retired; the Foundry pipeline (M01-P18) is the canonical workflow. See `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md`.

> **Status:** Deprecated 2026-05-16
> **Supersedes:** -
> **Superseded-by:** ADR-0116
> **Owner:** `council-architecture` + `foundry`
> **Date:** 2026-05-12
> **doc_class:** DecisionRecord
> **purpose:** Resolve the new-crate chicken-and-egg with grit symbol-locking via the icm-coordination-lock fallback pattern (scaffold-locks-oyatie topic). Cargo.toml::workspace_members is verified NOT indexed by grit at v0.3.0, so the primary scaffold-claim path is the icm fallback.
> **planned_enforcement_ref:** `oya-governance-scaffold-claim-pattern` — advisory until the lane exists.
> **Related:** ADR-0052 (grit/icm tooling inventory), ADR-0053 (grit + icm as sanctioned coordination primitives), ADR-0015 (flat-crates layout), ADR-0041 (GitOps trunk-based development)

---

## Status

Deprecated 2026-05-16 — superseded by ADR-0116. The Foundry pipeline (M01-P18) replaces the grit/icm scaffold-claim pattern in its entirety; new-crate scaffolds now use plain `git mv` inside a per-agent `git worktree`, with the resulting PR entering the Foundry pipeline (admission-gate → merge-queue → auto-merge on review+CI green).

Original status (historical): Accepted — 2026-05-12. The icm-coordination-lock fallback was the canonical scaffold-claim path at grit v0.3.0. The `Cargo.toml::workspace_members` primary path was blocked pending a grit upstream fix (see Follow-ups §1). Both of these are now retired per ADR-0116.

---

## Date

2026-05-12

---

## Context

`grit claim` requires a real indexed code symbol in the form `<file>::<Identifier>`. A *new* crate (e.g., `tools/oya-tooling-agent-read/`) has no source files yet, hence no indexed symbols, hence cannot be locked via `grit claim` before scaffolding begins. The cutover plan creates new crates — this is a concrete gap in the spec.

The proposed resolution is a **scaffold-claim pattern**: lock a workspace-level coordination point (`Cargo.toml::workspace_members`) for the duration of crate creation, then re-index so normal symbol-level claims can proceed. However, Lane 3 of the deep-dive trace (run 2026-05-12) confirmed via `grit symbols` that `Cargo.toml::workspace_members` returns **zero matches** — the field is not indexed by grit at v0.3.0. File-level path locks are also not supported in grit v0.3.0. The primary scaffold-claim path is therefore unavailable today.

This ADR formalises the **icm-coordination-lock fallback** as the canonical scaffold-claim path and records the conditions under which the primary path (`Cargo.toml::workspace_members` grit claim) supersedes it once grit gains the required indexing capability.

Sibling ADRs: ADR-0052 inventories the grit/icm tooling surface; ADR-0053 establishes grit + icm as the sanctioned coordination primitives replacing the retired omx/ultragoal stack.

---

## Decision

The canonical scaffold-claim path for new-crate creation is the **icm-coordination-lock fallback**:

1. Before scaffolding any new crate, the authoring agent opens a scaffold window in the `scaffold-locks-oyatie` icm topic.
2. Any other agent checks that topic before touching the workspace and backs off if a window is open against an overlapping path.
3. The authoring agent closes the window after scaffolding completes and `grit init` has been re-run.

This is slower than a native grit lock but correct. It is the **only valid path** at grit v0.3.0.

The `Cargo.toml::workspace_members` primary path is **reserved for future adoption** once grit upstream indexes the field (see Follow-ups §1). When that condition is met, an amending note to this ADR records the promotion; the icm fallback is then demoted to secondary-only.

---

## Decision Drivers

- **Correctness over speed.** A silent race between two agents scaffolding into the same workspace directory corrupts `Cargo.toml` and blocks CI. An explicit coordination lock, even via icm, eliminates the race.
- **Minimal new mechanism.** icm is already a sanctioned primitive (ADR-0053). No new tooling is introduced.
- **Forward compatibility.** The pattern is defined so that the primary path (`Cargo.toml::workspace_members` grit claim) can slot in without changing the sequence shape — only step 1 and step 4 change.
- **Lane 3 verification.** The grit non-indexing of `Cargo.toml::workspace_members` is an empirically confirmed fact (zero matches, 2026-05-12), not an assumption. The decision is grounded in observed behaviour, not speculation.

---

## Alternatives Considered

### Alternative A — `Cargo.toml::workspace_members` grit claim (primary path, rejected for now)

**Proposal:** Claim `Cargo.toml::workspace_members` via `grit claim --agent <id> --intent "scaffold ..." Cargo.toml::workspace_members` to take a native grit lock on the workspace manifest for the scaffolding window.

**Pros:** Native grit lock; visible in `grit watch`; automatically released by `grit done`; no icm dependency for this step.

**Cons:** **Blocked.** Lane 3 of the deep-dive trace confirmed `Cargo.toml::workspace_members` returns zero matches in `grit symbols` at v0.3.0. Attempting the claim produces a raw sqlite FK violation (symbol-not-found surfaced as FK error — documented in the upstream bug report). The path is unavailable until grit upstream adds indexing for Cargo.toml fields.

**Verdict:** Rejected as primary path at v0.3.0. Reserved for future promotion (Follow-ups §1).

---

### Alternative B — Per-file-path lock via grit (rejected — not supported)

**Proposal:** Use a hypothetical `grit claim --path Cargo.toml` or `grit lock --file Cargo.toml` to lock at the file level rather than the symbol level.

**Pros:** Would not require grit to index TOML fields, only to support a file-level lock primitive.

**Cons:** **Not supported.** grit v0.3.0 does not expose a file-level lock primitive. All locks are symbol-scoped (`<file>::<Identifier>`). This is a missing feature, not a configuration gap. Filed as a successor-IP upstream request (Follow-ups §2).

**Verdict:** Rejected. Does not exist in the current grit surface.

---

## Why Chosen

The icm fallback is the only mechanism that is both (a) available today and (b) correct under concurrent agent execution. It re-uses an already-sanctioned primitive (ADR-0053), introduces no new dependencies, and is forward-compatible with native grit promotion once the upstream gap is closed.

---

## Consequences

### Positive

- Eliminates the scaffold race condition for new-crate creation with no new tooling.
- Pattern is auditable: `icm recall -t scaffold-locks-oyatie` shows all open and closed scaffold windows.
- Forward-compatible: primary path slots in by changing only steps 1 and 4 of the sequence.

### Negative

- icm is a slower and less integrated lock than a native grit symbol lock. It relies on agents actively polling `icm recall`; there is no TTL-based auto-release or `grit watch` event for icm lock state.
- The window-open/window-closed discipline is manual. An agent crash or interrupted session can leave a window permanently open. Recovery requires a human `icm update` or a monitoring script.

### Operational

- Any agent intending to scaffold a new crate **must** run `icm recall -t scaffold-locks-oyatie` first and back off if any window is open for an overlapping path.
- The `oya-governance-scaffold-claim-pattern` fitness lane enforces that no new crate directory appears in git history without a corresponding icm scaffold-lock open/close pair in `scaffold-locks-oyatie`.
- If a stale open window is found, the on-call agent inspects icm for the originating agent id and either confirms the window is still live or closes it manually.

---

## Worked Example — Scaffolding `tools/oya-tooling-agent-read`

The following is a complete scaffold-claim sequence. The `icm store` calls are mandatory; omitting them violates the fitness lane.

**Before scaffolding (lock open):**

```
icm store \
  -t scaffold-locks-oyatie \
  -c "agent=dd-executor path=tools/oya-tooling-agent-read window=open intent='scaffold new oya-tooling-agent-read crate'" \
  -i critical
```

| field | value |
|---|---|
| topic | `scaffold-locks-oyatie` |
| agent | `dd-executor` |
| path | `tools/oya-tooling-agent-read` |
| window | `open` |
| intent | `scaffold new oya-tooling-agent-read crate` |
| importance | `critical` |

**7-step scaffold-claim sequence:**

```
1. icm store -t scaffold-locks-oyatie \
     -c "agent=<id> path=<new-crate-path> window=open intent='<description>'" \
     -i critical

2. <agent creates <new-crate-path>/{Cargo.toml, src/lib.rs, src/main.rs}>

3. <agent edits root Cargo.toml to add the new crate to workspace.members>

4. grit done --agent <id>          # lands the scaffold to base

5. grit init                       # re-index so the new crate's symbols are claimable

6. icm store -t scaffold-locks-oyatie \
     -c "agent=<id> path=<new-crate-path> window=closed" \
     -i high

7. <subsequent agents use normal grit claim against the new crate's symbols>
```

**After scaffolding (lock closed):**

```
icm store \
  -t scaffold-locks-oyatie \
  -c "agent=dd-executor path=tools/oya-tooling-agent-read window=closed" \
  -i high
```

| field | value |
|---|---|
| topic | `scaffold-locks-oyatie` |
| agent | `dd-executor` |
| path | `tools/oya-tooling-agent-read` |
| window | `closed` |
| importance | `high` |

After step 7, subsequent agents may claim symbols inside `tools/oya-tooling-agent-read/` via standard `grit claim --agent <id> --intent "..." "tools/oya-tooling-agent-read/src/lib.rs::<Symbol>"`.

---

## Follow-ups

1. **File upstream grit issue — `Cargo.toml::workspace_members` not indexed.** The `Cargo.toml::workspace_members` symbol returns zero results in `grit symbols` at v0.3.0. Upstream fix needed: index Cargo manifest fields (at minimum `workspace.members`, `package.name`) so that workspace-level scaffold locks can be taken natively. When this lands, amend this ADR to promote the primary path and demote the icm fallback. Ticket: file at `rtk-ai/grit` (Draft 1 of `.omc/scratch/pre-cutover-drafts-2026-05-12.md` documents the FK-violation symptom and suggested upstream fix).

2. **File upstream grit issue — file-level lock primitive.** Even without TOML field indexing, a `grit claim --path <file>` or `grit lock --file <file>` primitive would enable file-level locks as a scaffold coordination point. File as a separate upstream feature request at `rtk-ai/grit`. Revisit when grit ships a workspace-level lock primitive.

3. **icm stale-window recovery runbook.** Author a short runbook at `docs/runbooks/icm-scaffold-lock-recovery.md` covering: how to detect a stale open window, how to confirm the originating agent is no longer active, and how to close the window manually.

---

## References

- `.omc/scratch/pre-cutover-drafts-2026-05-12.md §Draft 2` — source draft for this ADR (pre-approval scratch; read-only)
- Lane 3 deep-dive trace, 2026-05-12 — verification that `Cargo.toml::workspace_members` returns zero matches in `grit symbols` at v0.3.0
- ADR-0052 — grit/icm tooling inventory (sibling)
- ADR-0053 — grit + icm as sanctioned coordination primitives (sibling)
- ADR-0015 — flat-crates layout (new crates must conform to `crates/oya-<context>-<role>/` naming)
- ADR-0041 — GitOps trunk-based development (scaffold lands to base via `grit done`)

---

## Amendment — 2026-05-13: Rename-Event Scaffold-Claim Authority (Shard 0 Precursor)

> **Amendment status:** Accepted — same-commit as ADR-0056 + ADR-0057 (Shard 0 precursor commit, 2026-05-13).
> **Architect iter-1 condition B2:** CLOSED by this amendment.

### Extension

The scaffold-claim authority established in this ADR is hereby extended to cover
**rename events**, not only new-crate scaffolds. A rename event is any operation
that changes a crate's directory path, `[package] name`, or `[lib] name` in the
workspace manifest — including the ~140-crate atomic rename planned for Shard 1.

### Rationale

The original ADR (2026-05-12) covered the chicken-and-egg case for new crate
creation: no source files → no indexed symbols → cannot take a native grit lock.
The rename case has an analogous gap: the pre-rename symbol path (e.g.,
`crates/oya-tenancy-kernel/src/lib.rs::TenantId`) is valid for a grit
claim, but atomically claiming ~140 symbols across the workspace is impractical
at grit v0.3.0.

The icm-coordination-lock fallback generalises naturally: a rename window is
opened in `scaffold-locks-oyatie` before any `git mv` operation, held for the
duration of the rename batch, and closed after `cargo check --workspace --locked
--offline` exits 0.

### Amended 7-step sequence for rename events

```
1. icm store -t scaffold-locks-oyatie \
     -c "agent=<id> path=<workspace-root> window=open intent='rename-cutover-v4 Shard 1 — ~140 crate renames'" \
     -i critical

2. For each crate in rename map:
     git mv crates/<old-name> crates/<new-name>

3. xtask-metadata-augment --apply  (rewrites [package] name, [lib] name, dep-edges, Cargo.lock)

4. cargo check --workspace --locked --offline  (verifies no non-name delta crept in)

5. grit done --agent <id>          (lands the rename batch to base)

6. grit init                       (re-indexes new symbol paths)

7. icm store -t scaffold-locks-oyatie \
     -c "agent=<id> path=<workspace-root> window=closed intent='rename-cutover-v4 Shard 1 complete'" \
     -i high
```

### ICM scaffold-claim rows for Shard 0 check crates

Per the original ADR, each crate scaffolded in Shard 0 requires an OPEN and
a CLOSE row in `scaffold-locks-oyatie`. The 4 LEAN check crates + xtask are
covered by the single batch lock opened at Shard 0 execution start (see §5.1
step 0 of the v4 plan). Individual per-crate rows are also emitted:

- `oya-shared-architecture-check-cli` — LEAN-A1
- `oya-shared-bounded-contexts-check-cli` — LEAN-A2
- `oya-shared-supply-chain-check-cli` — LEAN-A3
- `oya-shared-semver-check-cli` — LEAN-A4
- `xtask-metadata-augment` — workspace build tool

These rows are logged to `scaffold-locks-oyatie` by the executing agent
immediately after each crate directory is created (per ADR-0054 §"7-step
scaffold-claim sequence", step 1 applied per crate).

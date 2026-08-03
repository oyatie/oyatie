# G030-D dual-proof probe — focus ephemeral under docs — 2026-08-02

State: **PLANNING_ONLY — CONSUMER PROOF NEGATIVE; AUTHORITY NOT CLEAR; DELETION NOT AUTHORIZED**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
No deletion, freeze PR, ignore-rule edit, push, or activation occurred.

## Candidate set

Producer-equivalent unit-class `ephemeral` via rule `ephemeral-omc-state` (`contains "/.omc/state/"`).

### Focus family (11 JSON)

| # | Path |
|---:|---|
| 1–4 | `docs/audit/initial-sweep-2026-06-06/.omc/state/sessions/6725dbb8-…/{last-tool-error-state,mission-state,pre-tool-advisory-throttle,subagent-tracking-state}.json` |
| 5–8 | `docs/audit/initial-sweep-2026-06-06/architecture/.omc/state/sessions/6725dbb8-…/` same four basenames |
| 9 | `docs/audit/initial-sweep-2026-06-06/synthesis/.omc/state/sessions/6725dbb8-…/pre-tool-advisory-throttle.json` |
| 10 | `docs/decisions/.omc/state/last-tool-error.json` |
| 11 | `docs/decisions/.omc/state/sessions/8f603fc7-…/session-started.json` |

### Sibling non-focus ephemeral in the same trees (2 jsonl)

- `docs/audit/initial-sweep-2026-06-06/.omc/state/agent-replay-6725dbb8-….jsonl`
- `docs/audit/initial-sweep-2026-06-06/architecture/.omc/state/agent-replay-6725dbb8-….jsonl`

Tracked `/.omc/state/` population on tip = **13** paths (11 focus + 2 jsonl). No other tracked nested `.omc/state` trees.

## Proof 1 — consumer (code/build/workflow)

Method: exact full-path search over 2,241 files under `ci/`, `governance/`, `specs/`, `registry/`, `.github/`, `libs/` (`.rs`/`.json`/`.toml`/`.yml`/`.yaml`/`.md`/BUCK; files ≤2 MiB).

| Check | Result |
|---|---|
| Exact path literal in scanned trees | **NONE** for all 11 focus paths |
| `synthetic_dependencies` exact seed | **NONE** |
| Generic references to these trees | only the unit-class policy rows that *classify* `/.omc/state/` as ephemeral (`ci/facade/.../unit-class-policy.json` and bundled twin) |
| Markdown-retirement path list | does **not** name `docs/audit/**` or `.omc/state` |

**Consumer proof: NEGATIVE** (no semantic reader found). Classification-as-ephemeral is not a consumer.

This scan is not infinite: it does not claim zero hits in the entire monorepo prose under `docs/**` self-reference, nor in untracked CI materializations. It is sufficient to reject “required gate seed” and “Rust path-literal consumer” for these exact paths.

## Proof 2 — authority / retention

| Authority surface | Finding | Blocks delete? |
|---|---|---|
| unit-class / TTL | `ephemeral`, budget 2 days, action **archive**, not delete | soft — archive ≠ delete |
| root `.gitignore` | ignores **repo-root** `/.omc/*` only (leading `/`); nested `docs/**/.omc/**` is **not** covered | explains how they entered git; does not retain them |
| markdown-retirement | root survival set is four files; immutable audit archive cites `/evidence/audits/consensus/**/*.md`, **not** these JSON session dumps | partial — does not protect these paths by name |
| location under `docs/audit/` and `docs/decisions/` | historical audit / decision-tree adjacency; possible human expectation of retention even without machine consumer | **yes until owner rules** |
| founder/session policy | OMC runtime state is durable outside git; committed nested copies contradict that posture | favors removal *after* owner ruling |

**Authority proof: NOT NEGATIVE.** Location under audit/decision trees prevents auto-promotion to `DARK_BUREAUCRACY` even with clean consumer-negative evidence.

## Dual-proof verdict

| Clause | Status |
|---|---|
| Consumer proof (no live input edge) | **PASS** |
| Authority proof (not retained) | **FAIL / UNRESOLVED** |
| Combined `DARK_BUREAUCRACY` | **NOT MET** |
| Deletion authorized | **NO** |

Disposition for all 13 paths:

`EPHEMERAL_COMMITTED_RUNTIME_LEAK — DELETE_CANDIDATE_PENDING_OWNER_RULING`

Not `DARK_BUREAUCRACY`. Not freeze/delete without an accountable-owner decision that these are not audit-retained.

## Why this is still useful

1. Bounds G030-D: the only non-empty delete-action class (`scratch`) is empty on tip; the next-smallest class fails dual proof on authority.
2. Names the exact owner question: **are committed nested OMC session dumps under `docs/audit/**` and `docs/decisions/**` retained audit artifacts or accidental runtime leakage?**
3. If the owner rules **leakage**, the first reduction PR is tiny (13 paths), anti-vacuity is trivial, and a follow-up ignore-rule for nested `**/.omc/state/` (or equivalent) prevents recurrence — separate from bulk husk work.
4. If the owner rules **retain**, reclassify or carve them out of ephemeral archive pressure and stop treating them as G030 reduction fuel.

## Anti-vacuity sketch (only after owner YES)

If a future PR deletes the 13 paths after APPROVE:

1. Before tip focus count 13,959; after 13,948 (−11 focus). Whole tracked −13.
2. Same command family: `git ls-tree -r --name-only <tip>` + extension filter.
3. Dual-proof rows attached for each path.
4. No `*.generated.json` committed.
5. Root survival Markdown cardinality remains 4.
6. Required gates remain green; no synthetic-dependency row referenced these paths.

## Explicit non-actions

- No PR opened.
- No `.gitignore` edit.
- No docs/audit content rewrite.
- No claim that consumer-negative alone is enough.
- No independent APPROVE available; transport/quota still fused.

## Sequencing

- G030-A/B/C remain planning evidence.
- G030-D stays blocked on owner ruling for this 13-path set.
- G030 bulk husk (`oya/` 4,729 focus husk) remains G026/capability work, not this probe.
- Prefer G028/#1526 health before any accounting-facing deletion train even for this tiny set.

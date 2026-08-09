---
doc_class: Program-Mapping
doc_status: published
entry_id: kernel-track-mapping
program: kernel-track
recorded_at: 2026-08-09
terminal_state: standing
decision_status: founder-reserved
---

# kernel-track — mapping and conventions

The standing convention document for the `docs/programs/kernel-track/` subtree. Every unit
of the kernel-track evidence program is checked against this file. It is not a measurement
and it rules on nothing; like everything in this subtree, the kernel decision it supports is
founder-reserved.

**Honesty note on sequencing.** This mapping was written after the first unit landed
(`G005-kernel-track-evidence-pack-20260809.md`, commits `0100b9573` + `231bab91e`), not
before it. It was therefore derived from that unit rather than imposed on it, and §8 records
the one place the existing tree had to move to conform. Later units get the mapping in
advance; the first one did not, and a reader should discount §1–§7 accordingly where they
merely describe what already exists.

---

## 1. What a unit is

A **unit** is one commit directly on `impl/g005-kernel-evidence` that either

- adds one evidence pack, or
- revises one existing pack, or
- performs the accounting re-freeze that an add obliges.

Units do not branch. Units do not open pull requests. Exactly one PR is opened, at the end,
by the land phase.

---

## 2. Recurring patterns and what each becomes

| Pattern | What it becomes |
|---|---|
| A **measurement** | `## N. Measurement N — <short name>` with, in order: the question restated; the method; the literal command; the figure with its label; the named error terms; and, if not EXACT, what it would take to make it EXACT. |
| A **figure** | A number immediately adjacent to a bold label from the closed set in §3. A number with no label is a defect, not a style lapse. |
| A **negative claim** ("X appears nowhere") | The invocation, plus a **positive control on the same invocation** proving the pattern can match something. Without the control it is not a measurement. |
| An **ADR reference** | A bare id (`ADR-0701`). Archived ids additionally spell the location as `docs/adr-archive/…`. Never a `decisions/ADR-NNNN` path — see T2. |
| A **correction to a recorded position** | Three explicit parts: what was recorded, what was measured, and the **direction** the correction runs (does it make the case stronger or weaker than recorded). |
| **Upstream evidence** | The API call or command, the date it was read, and an explicit restatement that upstream content is third-party DATA read for measurement only. |
| A **new tracked file** | A census re-freeze in the same commit or the immediately following one, carrying an attribution block (§5). |
| A **conclusion** | A presentation of evidence with the counter-case attached. Never `we should`, `we will`, `adopt`, `decided`, `approved`. |
| An **argument for an option** | Paired, in the same document, with the evidence that would make that option WRONG. Unpaired advocacy is the one thing this program cannot ship. |

---

## 3. Confidence labels — closed set

`EXACT` · `LOWER BOUND` · `UPPER BOUND` · `COULD NOT DETERMINE`

- `EXACT` means re-running the stated command reproduces the number. Nothing else earns it.
- `LOWER BOUND` / `UPPER BOUND` must name their systematic error terms individually. "Roughly"
  is not a bound.
- `COULD NOT DETERMINE` is a **success**, and must state what it would take. It ranks above a
  figure that looks authoritative and is not.

---

## 4. Naming, location and ownership

**Location is fixed: `docs/programs/kernel-track/` and nowhere else.** A top-level
`docs/*.md` is structurally unownable — ownership resolves to the nearest ancestor `OWNERS`,
and an `OWNERS` covering more than `max_paths_per_owners_file` (`oya-ci.toml`, 2000) owns
nothing at all, fail-closed, with no fall-through to a broader file. Adding `docs/OWNERS`
would look like a fix and change nothing.

| Convention | Rule |
|---|---|
| Standing docs | UPPERCASE stem, undated: `OWNERS`, `MAPPING.md`. |
| Evidence packs | `G005-<topic>-YYYYMMDD.md`. |
| `entry_id` frontmatter | Exactly the filename stem. |
| Required frontmatter | `doc_class`, `doc_status`, `entry_id`, `program: kernel-track`, `recorded_at`, `terminal_state`, `decision_status: founder-reserved`. |
| `doc_status` values | `drafted` \| `published` \| `stale` \| `archived`. Never `superseded` — it requires a supersession edge. |
| `OWNERS` | The existing two lines (`council-architecture`, `axis-cloud-platform`). Do not add per-file OWNERS. Do not create sibling subtrees. |
| Reachability | The single existing `docs/programs/kernel-track/` prefix in `specs/reachability-registry.json`. A new document extends that anchor's prose; it does not add a second prefix. |
| Branch | `impl/g005-kernel-evidence`, committed with `git commit -- <paths>`. |

The program name `G005` is an **orchestrator label**. The repository's own G005
(`.omc/ultragoal/goals.json`) is a different goal — a SCIM durable slice. Never cite G005 as
if the repository defines it as the kernel track.

---

## 5. The accounting seam, and its inverted sibling

Two gates react to a new `.md` under `docs/`, and they want **opposite** treatment. Getting
them the same way round is the most common way this program reddens CI.

**`governance/check/adr-citation-closure` — pinned by EQUALITY. Re-freeze it.**
Adding one `.md` moves `files_scanned` by +1 and the gate refuses the build until the frozen
value follows. Re-derive by RUNNING the gate and reading its `observed N`; never by adding a
delta to the previous frozen number. Edit the policy as TEXT keyed by name — round-tripping
it through JSON reformats the whole file.

Every re-freeze carries an attribution block naming, at minimum:

- tracked-add delta, scanned-add delta, observed census delta — and the statement that they agree;
- that `citation_lines` and `adr_records` did **not** move, which is the cross-check that
  distinguishes an add from a scan narrowing;
- that every finding count was re-run after the re-freeze rather than assumed.

**`ci/facade/lifecycle-status` — SHRINK-ONLY. Do NOT re-freeze it.**
Declare `doc_status` in frontmatter so `stage_not_declared` never moves. Raising that
baseline is growth against a shrink-only ceiling.

**`ci/facade/artifact-accountability` (ADR-0555 born accounting)** wants all three of
justified / owned / reachable. A reviewed registry prefix clears justified and reachable
together (REACHED implies JUSTIFIED); ownership is cleared only by §4's location rule.

---

## 6. Invariants — must hold after EVERY unit, checkable in isolation

| # | Invariant | How a reviewer checks it from the diff alone |
|---|---|---|
| I1 | No `decisions/ADR-` path in any added or changed `.md`. | `grep -n 'decisions/ADR-' <file>` returns nothing, with the pattern first proven matchable against `docs/AGENTS.md`. |
| I2 | Every `.md` in the subtree declares a valid `doc_status`. | Read the frontmatter. |
| I3 | Every figure carries a label from §3. | Scan for bare numerals in prose. |
| I4 | No file added outside `docs/programs/kernel-track/`, except the two accounting files (`…/adr-citation-closure-policy.json`, `specs/reachability-registry.json`). | `git diff --stat origin/dev..HEAD`. |
| I5 | `files_scanned` frozen equals the gate's observed value, and the delta carries an attribution block. `citation_lines` and `adr_records` unchanged. | Read the policy diff. |
| I6 | Subtree path count stays far below 2000. | `git ls-tree -r --name-only HEAD docs/programs/kernel-track/ \| wc -l`. |
| I7 | Nothing ruled. No file under `docs/decisions/`. No `Accepted`. No first-person recommendation verb. | `git diff --stat`; grep the prose for `we should\|we will\|recommend\|decided\|approved`. |
| I8 | Zero regressions proven by **diffing failing sets** at base and head, not by counting green. | The evidence block quotes both runs and states the failing sets are identical. |
| I9 | Every option argued for has its wrong-choice evidence in the same document. | §7 of the pack exists and is non-empty on both sides. |

---

## 7. Traps — where the obvious translation is subtly wrong

Each of these was hit or nearly hit for real. They are ordered by how expensive they are.

**T1 — the authority that vanished in consolidation.** ADR-0338 is `status: Superseded`,
`superseded_by: [ADR-0701]`, and lives under `docs/adr-archive/`. Citing it as a live
ratified ADR is itself a gate finding and a reviewer can dismiss the whole measurement in one
line. The Cloud Hypervisor mandate is live, but it is carried by ADR-0701 — a monorepo layout
and reorg-doctrine apex that bulk-supersedes 62 other ADRs. Verify BOTH sides of the
supersession oracle (member `superseded_by` and apex `supersedes` membership) before treating
any ADR as live.

**T2 — the citation that looks better and is worse.** Writing
a relative markdown link into `decisions/` for an archived id is the natural instinct and is exactly the
`adr_citation_dangling_path` defect the gate counts (population 2002) — *and* it moves the
equality-pinned `citation_lines`, so the document pays for itself twice. Bare id; archive
location spelled as `docs/adr-archive/`. A sibling `docs/programs/k8s-port/README.md` does
the wrong thing here and is itself part of the 2002 population — copy the operations journal,
not that README.

**T3 — a narrowing and an add move `files_scanned` identically.** Re-freezing without
attribution certifies a silently narrowed scan as a legitimate add. The distinguishing
evidence is that a narrowing also moves `citation_lines`, `adr_records` and the finding
counts, and an add moves none of them.

**T4 — the sibling gate with the inverted answer.** `lifecycle-status` looks like the same
problem as the citation census and takes the opposite fix (§5). Applying the citation-census
reflex to it raises a shrink-only baseline.

**T5 — extensionless files do not move the census.** `OWNERS` and `BUCK` carry no
`scan_extension`. A commit adding two files can legitimately move `files_scanned` by one.
Equating tracked-adds with scanned-adds mis-attributes the delta.

**T6 — `git grep -E "\bfoo"` matches nothing, silently.** POSIX ERE has no word-boundary
atom. Every negative claim needs a positive control on the same invocation, or it is a bug
report about the regex dressed up as a finding.

**T7 — counting green instead of diffing failing sets.** This repository carries inherited
red. A green count cannot distinguish "fixed one, broke one" from "changed nothing". When two
runs disagree by one, chase it rather than averaging — a 19-vs-20 delta was a lock-timeout
test flaking under load, reproducible at the baseline.

**T8 — `git add` then `git commit`.** The index is per-worktree and therefore SHARED between
lanes; `git commit` commits the index, not what you added. Measured: a three-file intent
produced a seven-file commit two minutes later, committing a neighbour's work mid-edit.
`git commit -- <paths>` only. Never `stash`, `reset`, or `clean`.

**T9 — a neighbour's buck2 reads as your build failure.** Concurrent buck2 clients on one
project root cancel each other and the loser reports `The evaluation of this key was
cancelled: Rejected`. Check `ps` for a neighbour before blaming a change. Work in an isolated
worktree; its first build is always cold, because buck2 shares no cache across worktrees.

**T10 — syscall arithmetic that flatters whichever side is being argued.** `implemented +
stub` is not `implemented`: a `futex` stub returning `0` cannot carry a multithreaded tokio
runtime. Per-architecture counts must be **unioned**, never summed. And a count with no
init-only / hot-path / cold split systematically flatters one option, because an init-only
`uname` is a constant return while a hot-path `epoll_wait` is a scheduler-coupled subsystem.

**T11 — a narrow scan pattern yields a confident wrong bound.** The first static pass matched
only `libc::name(` and returned 33 syscalls with **no epoll at all**, because `mio` calls it
through a `syscall!` macro. Widening moved it to 49 and surfaced the whole family. A LOWER
BOUND from one call syntax reads as EXACT to a casual reader. State the syntaxes matched, and
state the coverage denominator (871 of 1530 locked crates had no cached source and were not
scanned).

**T12 — quoting a general-purpose figure as if it were ours.** "Linux has ~350 syscalls" is
not a measurement of our workloads and must never be used as one.

**T13 — a name in an enum is not an implementation.** `osdk/src/arch.rs` enumerates
`Arch::Aarch64`, which is why secondary sources claim arm64 support. The missing piece is the
OSTD arch backend. Presence of a target name ≠ presence of a port.

**T14 — licence facts do not travel between artifacts.** Verus is MIT; MPL-2.0 belongs to
vostd and, separately, to the Asterinas ISO pin's `mpl_boundary` block. Re-using one
artifact's licence record for another is an unevidenced claim in whichever direction it lands.

**T15 — `.omc/` cannot hold a committed document.** `.gitignore` excludes `/.omc/*` and
re-includes exactly four whitelisted paths under `.omc/ultragoal/`. A mapping written there
is uncommittable, and the failure appears only at commit time.

**T16 — the lane branch is checked out elsewhere.** The repository root worktree sits on a
different lane's branch with a large dirty tree, and `docs/programs/` does not exist there at
all. Read the base with `git show origin/dev:<path>`; do the work in the lane's own worktree.

---

## 8. Conformance of the existing tree

Checked against §4–§6 at `231bab91e`: the evidence pack conforms on filename, frontmatter,
labels, citation form, ownership, registry prefix and census attribution. One deviation was
found and fixed by this unit — the registry anchor described the subtree as holding
*measurement packs* only, which does not reach a standing convention document. The anchor
prose is extended rather than a second prefix added, per §4.

---

## 9. Definition of done — one unit

A reviewer who sees only the diff applies this list. Every line is mechanically checkable.

1. **Scope.** Files touched are inside `docs/programs/kernel-track/`, plus at most the two
   accounting files named in I4. Nothing under `docs/decisions/`.
2. **Frontmatter.** Every added or changed `.md` carries the seven required keys, a valid
   `doc_status`, and `entry_id` equal to the filename stem.
3. **Labels.** Every figure carries one of the four labels. No unlabelled numeral in prose.
4. **Bounds are honest.** Each bound names its systematic error terms. Each
   `COULD NOT DETERMINE` states what it would take.
5. **Negatives are controlled.** Every "appears nowhere" claim shows a positive control on the
   same invocation.
6. **Citations.** Bare ids only; archived ids spell `docs/adr-archive/`; `grep -n
   'decisions/ADR-'` over the changed `.md` returns nothing, with the pattern proven matchable.
7. **Census.** If a `.md` was added: `files_scanned` re-frozen to the gate's own `observed N`,
   with an attribution block covering tracked-add / scanned-add / observed deltas and the
   `citation_lines` + `adr_records` cross-check. If none was added: the policy file is
   untouched.
8. **Lifecycle.** `lifecycle-status` policy untouched; the doc declares its stage instead.
9. **No ruling.** No `Accepted`, no recommendation verb, no ADR authored. `decision_status:
   founder-reserved` present.
10. **Counter-case.** Every option argued for carries, in the same document, the evidence that
    would make it the wrong choice.
11. **Gates.** Evidence quoted for the gates governing the touched paths — at minimum
    `adr-citation-closure`, `ci/facade/lifecycle-status`, `ci/facade/artifact-accountability`
    — as literal buck2 output including its `Commands:` line.
12. **No regressions.** The failing set at the untouched base and at head is quoted and shown
    to be identical. Counting green does not satisfy this.
13. **Git.** One commit, on `impl/g005-kernel-evidence`, made with `git commit -- <paths>`. No
    branch, no PR, no destructive git command.

A unit failing any of 1–13 is not done, regardless of how good the measurement is.

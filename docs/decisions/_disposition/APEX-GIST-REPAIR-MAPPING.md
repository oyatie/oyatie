---
doc_status: published
---

# Apex gist repair — pattern mapping (oyatie-5ad)

Binding mapping for every unit on `impl/5ad-apex-truncation`. A reviewer holding only a unit's diff
applies §7 to accept or reject it. Where this document and a unit's own judgement disagree, this
document wins; amend it in the same commit that departs from it, so the departure is reviewable.

Scope: the ten live apex ADRs `docs/decisions/ADR-0700..0709`, their 386 archived members under
`docs/adr-archive/`, the gate `governance/check/apex-gist-integrity` (already landed at c091256e7),
and the equality-pinned censuses that the repair moves.

## 1. The defect, stated mechanically

An apex supersedes archived members and carries their ratified substance forward at two sites:

| Carry site | Shape | Population |
|---|---|---|
| `## Preserved member gists` | `- **ADR-<n>** (<member-stem>): <collapsed normative text>` | 254 bullets |
| `### ADR-<n> residual` | `**<member-stem>** — <collapsed normative text>` | 385 sections |

254 + 385 = 639, which is exactly the `blocks` census the landed gate froze — an independent
cross-check that this document and the gate are counting the same corpus.

Both were produced by slicing the whitespace-collapsed normative section of the member at a fixed
character budget — 350 for gists, ~400 for residual bodies, ~118 for residual titles — with no
structural awareness. Independently, the gist LIST was capped at 60 entries, so ADR-0700 and
ADR-0709 (126 members each) carry 66 members apiece with no gist at all.

The cut leaves no marker. The result is well-formed Markdown that reads as complete while asserting
less than was ratified. `docs/decisions/_disposition/END-STATE-POLICY.md` consolidation rule 4
("any still-true normative text from Superseded must be copied into the superseding/live ADR
*before* delete") was therefore only partially satisfied, and nothing in the tree could tell.

The clearest exhibit: ADR-0709 carries ADR-0147's runtime-ladder introduction and then cuts inside
the table separator row, so the apex says "The canonical mapping below replaces the universal-gVisor
default:" and the mapping is absent. The dropped rows are the confidential-compute runtime
assignments (`kata-clh-sev-snp`, `kata-clh-tdx`). Zero of the ten apexes contain any
confidential-computing text; nine archived files do. That is loss, not retirement — a header still
standing over a missing table is not a decision anyone made.

**No generator produces the apex bodies.** `git grep -l 'Preserved member gists'` matches the ten
`.md` files and nothing else. The standard remedy cited for `docs/ADR-INDEX.md` ("use the
generator") is unavailable here, and §6.9 records why that warning does not transfer.

## 2. Pattern → replacement

Every recurring pattern this goal touches, and exactly what it becomes.

### P1. Fixed-character slice → whole-block carry

**Before:** `text[..350]`, cut wherever it lands.
**After:** the normative section is segmented into BLOCKS (paragraph, complete table, complete
fenced code block, complete list). Blocks are carried whole, in source order, until cumulative
length reaches the budget; the block that crosses the budget is completed, not cut. The budget is a
FLOOR, never a ceiling.

A carried block has exactly three legal renderings and no fourth:

1. **FULL** — the block, whitespace-collapsed, byte-identical to the member's.
2. **ELIDED-WHOLE** — the block replaced entirely by a structural placeholder that names its kind
   and size: `[elided: table, 8 rows]`, `[elided: fenced block, 31 lines]`, `[elided: paragraph,
   412 chars]`.
3. **ABSENT** — the block is past the budget and is covered by the trailing elision marker (P2).

**PARTIAL is illegal.** No carried text may be a proper prefix of a block. This single rule removes
the whole defect class, and it is what makes the ADR-0147 ladder land: an 8-row table is one block,
so it arrives whole or is declared absent, never headless.

### P2. Silent truncation → declared elision

**Before:** the text stops. **After:** every carry site that dropped anything ends with

```
… [elided <k> of <n> blocks from ADR-<NNNN>]
```

A carry site that dropped nothing ends with no marker. A short gist is then distinguishable from a
cut one by reading, and by machine. Precedent for the shape: OTLP's `dropped_attributes_count` /
`dropped_events_count` exist so consumers can detect incomplete span data; the apex format had no
such field, which is precisely why truncation read as completeness.

Marker constraints, both load-bearing:
- The member is named by **bare id** (`ADR-0147`), never by a path. See T5.
- `<k>` and `<n>` are computed, not estimated. A marker whose counts do not match the rendering is
  a defect equal to the truncation it replaces.

### P3. Capped gist list → complete gist list

**Before:** ADR-0700 and ADR-0709 emit 60 gists for 126 members.
**After:** every id in the apex `supersedes:` list has exactly one gist bullet and exactly one
residual section. Expect those two files to grow substantially; that growth is the repair, not a
regression.

### P4. Byte-budget title → whole title

Residual titles (`**<member-stem>** —`) are the member filename stem and are never abbreviated.
The 22 bullets whose title is cut mid-parenthesis (recorded in the gate policy's
`_parse_recovery_attribution`) become complete stems.

### P5. Ad-hoc heading assumption → enumerated alias list, fail-closed

The normative section is resolved by NORMALIZING every level-2 heading and comparing the result to
an ORDERED alias list, first match wins. Normalization, in order: drop `## `, strip `§`, strip a
leading `Section `, strip a leading single-token ordinal (`2. `, `B. `, `B `, `B: `), strip a
trailing parenthetical, trim, lowercase.

| Rank | Alias | Files |
|---|---|---|
| 1 | `decision`, `decisions` | 444 |
| 2 | `proposal` | 3 (ADR-0389, ADR-0390, ADR-0391) |
| 3 | `candidate proposal` | 1 (ADR-0621) |

Measured over all 448 archived files: all 448 resolve, 0 unresolved. The ten heading spellings that
are not the plain `## Decision` are `## §B Decision` (4), `## B. Decision` (3), `## Proposal` (3),
`## Decisions` (2), `## §B. Decision` (2), `## 2. Decision`, `## Decision (amendment)`,
`## Section B: Decision`, `## Decision (assumed by the citers)`, `## Candidate proposal`.

Two properties of this rule are load-bearing and were checked, not assumed. **Exact match after
normalization, never `contains`:** ADR-0319 carries both `## Decision Summary` (line 39) and
`## Section B: Decision` (line 145), and a `contains "Decision"` rule silently picks the wrong,
earlier one. **No fourth rank and no fuzzy fallback:** a member matching none of the three is a hard
error that fails the run; it is never carried as an empty gist. That is the failure the gate policy
already names in `_parse_recovery_attribution` — a parser that drops what it cannot read reports a
SMALLER finding count, which is indistinguishable from a repair.

### P6. Hand-authored apex body → generated apex body

The repair is a generator, not 639 hand edits. Rationale is not economy: hand-authoring 639
summaries substitutes one lane's paraphrase for ratified text, which is the same defect with a
different author. The generator's output is deterministic and idempotent (I7), so a future apex
cannot be re-truncated by a re-run, and the gate proves it if one is.

### P7. Uncorrected needle set → measured needle set

`apex-gist-integrity-policy.json` `topics[0].needles` is `["sev-snp","sev_snp","tdx","confidential
comput"]`. The literal space cannot match `confidential-computing`. Corrected set sees 9 archived
files where the current set sees 6; the three misses are ADR-0128 (which carries a named invariant
`INV-CONFIDENTIAL-COMPUTE` no apex mentions), ADR-0308, and ADR-0352. Currently harmless for the
frozen count — all three close onto ADR-0700/ADR-0709, already counted — so this is a latent
under-detection, corrected as DATA.

### P8. Loss claim → ratified-or-retired disposition

Every inventory row states which it is, from the member's own frontmatter, never from the apex:

- `status: Superseded` **and** a successor edge resolving to a live apex → **RATIFIED**, loss is a
  defect, repair required.
- `status: Rejected`, or no successor edge → **RETIRED**, absence is correct, no repair.

Measured: of the nine archived files carrying confidential-computing text, eight are Superseded with
a live successor and one (ADR-0352) is Rejected. Of the 249 truncated gists, 249 name a Superseded
member and 0 name a Rejected one — there is no retired subset to subtract from the truncation
population.

### P9. One-sided supersession read → both-sides union walk

Successor edges are the UNION of the member's `superseded_by:` and the apexes' `supersedes:`
membership, walked transitively through archived intermediates. Neither side alone is sufficient:
ADR-0183 declares `superseded_by: [ADR-0379]`, ADR-0379 is itself archived and is a member of
ADR-0701, and ADR-0701's `supersedes:` never names ADR-0183. Reading only the apex misses it —
and the apex is what this bead says is unreliable, so **the members are the source of truth for
what was ratified**.

## 3. Naming, module and ownership conventions

**Code home.** All repair code lives in the existing crate
`governance/check/apex-gist-integrity`. No new crate. The generator and the gate must share one
parser, or they can disagree about what a block is and both be green.

- pure kernel (no I/O): `src/lib.rs`
- generator entry point: `src/main.rs`, target `//governance/check/apex-gist-integrity:apex-gist-repair`
- live-tree gate test: `tests/apex_gist_integrity.rs` (existing)

**Policy is DATA.** All repo-specifics — budgets, heading aliases, needles, ceilings — live in
`apex-gist-integrity-policy.json`. Edit it as TEXT keyed by name. Round-tripping it through a JSON
serializer reformats the whole file and buries the one changed value.

**Ownership.** `governance/check/apex-gist-integrity/OWNERS` already exists (`council-architecture`).
There is no `governance/OWNERS` or `governance/check/OWNERS`, and the root OWNERS is past the
`[owners] max_paths_per_owners_file = 2000` cap in `oya-ci.toml`, which fails CLOSED with no
fall-through. Any new directory must ship its own OWNERS or it is unowned.

**Document home.** This document sits in `docs/decisions/_disposition/` for three reasons, each
verified rather than assumed: the directory is in `exempt_path_prefixes`, so it moves neither
`files_scanned` nor `citation_lines`; `docs/decisions/OWNERS` is the nearest ancestor and covers 106
paths, far under the cap; and `docs/decisions/` is a reachability prefix in
`specs/reachability-registry.json`. Its name must not match `ADR-*.md` or it joins the ADR corpus.

**Inventory artifact.** Deliverable (1) is generated, not hand-written, and lands as
`docs/decisions/_disposition/apex-gist-loss-inventory.json` — same directory, same three reasons.
One row per (apex, member, carry-site) with: member id, member stem, apex id, disposition
(RATIFIED/RETIRED) with the frontmatter evidence, blocks carried, blocks elided, and the loss class.

## 4. Invariants

These hold after EVERY unit, so any unit is checkable in isolation.

- **I1 — no partial block.** No carried text is a proper prefix of a member block. Every block is
  FULL, ELIDED-WHOLE, or ABSENT.
- **I2 — every loss is declared.** A carry site that dropped anything ends with a P2 marker whose
  counts match the rendering. A carry site that dropped nothing has no marker.
- **I3 — resolution is fail-closed.** Member key, heading alias, and successor edge each either
  resolve exactly or fail the run. Nothing is skipped, defaulted, or guessed.
- **I4 — total representation.** For each apex, `supersedes:` ids, gist bullet ids, and residual
  section ids are the same set. No extras, no gaps.
- **I5 — no new path citations in `docs/decisions/`.** Members are named by bare id. See T5.
- **I6 — ceilings move with the repair.** Every commit that changes a finding count re-freezes the
  affected ceiling in the same commit, from the gate's own `observed N`.
- **I7 — idempotent.** Running the generator against a repaired tree produces an empty diff.
- **I8 — char boundaries.** All slicing is on `char_indices`, never byte indices. See T1.
- **I9 — order preserved.** Existing gist and residual entries keep their existing order; new
  entries append. Re-sorting turns a reviewable diff into 100% churn.

## 5. Unit decomposition

Every unit commits DIRECTLY to `impl/5ad-apex-truncation`. No per-unit branch, no per-unit PR.

| Unit | Deliverable | Depends on |
|---|---|---|
| U0 | this mapping | — |
| U1 | normative-section extractor + block segmenter (pure, `src/lib.rs`) | U0 |
| U2 | block renderer: FULL / ELIDED-WHOLE / marker (pure, `src/lib.rs`) | U0 |
| U3 | generator binary `src/main.rs` + BUCK target | U1, U2 |
| U4 | inventory artifact (deliverable 1) | U1 |
| U5 | apply repairs to the ten apexes + re-freeze apex ceilings (deliverable 2) | U3 |
| U6 | P7 needle correction + red-fixture proof of the gate (deliverable 3 completion) | U0 |
| U7 | LAND: batched bookkeeping, one PR | all |

U1/U2/U6 are parallel-safe (disjoint concerns, no shared frozen number). U5 alone rewrites the ten
apexes and alone moves the apex ceilings — it does not parallelise against itself.

## 6. Traps

Places where the obvious translation is subtly wrong.

**T1 — `&s[..n]` panics where the original silently truncated.** The apexes contain `µ` and `—`
adjacent to cut points; all ten files are valid UTF-8 today only because the original slicer
happened to land on char boundaries. A Rust byte slice at a non-boundary panics. Use
`char_indices()` and floor to a boundary. This is the canonical shape of the trap: a rewrite that
converts silent corruption into a crash, or the reverse.

**T2 — `debug_assert!` vanishes in release.** Any invariant a gate depends on must be `assert!`.
An invariant that only holds in debug builds is not an invariant.

**T3 — `as u16` on a parsed ADR number truncates silently.** ADR ids come from text. Use
`parse::<u16>()` and propagate the error; a cast turns a malformed id into a plausible wrong one.

**T4 — the bold id is not the member key.** `- **ADR-11** (ADR-0011-cross-microservice-contract-registry)`
carries a zero-STRIPPED id in bold and the filename stem in parentheses. Resolve by the paren stem;
fall back to the zero-padded bold id; **fail** if neither resolves. A scout's extraction bug here
(the paren sometimes holds a title, not a stem) produced a 187/62 split that did not exist, and the
gate's own earlier run reported 154 instead of 132 for exactly this reason.

**T5 — an elision marker naming a PATH moves an equality-pinned census.** A line in `docs/decisions/`
writing `decisions/ADR-NNNN` or `adr-archive/ADR-NNNN` becomes a `CitationLine`; ~639 markers would
move `citation_lines` by ~639, and a `decisions/ADR-NNNN` path pointing at an archived member would
additionally be a dangling-path finding, since `docs/decisions/` holds only ADR-0700..0709. A BARE
id adds zero citation lines, because bare ids only count on the three authority surfaces
(`CLAUDE.md`, `AGENTS.md`, `docs/AGENTS.md`). **Bare ids only.**

**T6 — adding a file anywhere outside three prefixes moves `files_scanned`.** Exempt:
`docs/adr-archive`, `docs/decisions/_disposition`, `governance/check/adr-citation-closure`. Anything
else, including `governance/check/apex-gist-integrity`, moves it. Extensionless files (`BUCK`,
`OWNERS`) do not. Re-derive by RUNNING the gate and reading `observed N` from its own assertion —
never by arithmetic, because a narrowed scan and a genuine delete produce the same number and only
one is legitimate. Attribute the move in the same commit.

**T7 — a new `docs/**/*.md` without `doc_status:` reddens a different gate.**
`ci/facade/lifecycle-status` freezes `doc-status-lifecycle.stage_not_declared` at 1921 shrink-only
over glob `docs/**/*.md`. An undeclared new doc makes it 1922. DECLARE a stage
(`drafted|published|stale|archived|superseded`) rather than raising the baseline — the opposite of
the T6 answer for the sibling census.

**T8 — equality pinning punishes repair identically to regression.** The apex ceilings are pinned by
`==`, not `<=`. A repair that lowers a finding count without lowering the ceiling in the same commit
is RED. That is deliberate: it forces the ceiling down with the fix and leaves no headroom for the
next defect.

**T9 — a policy-only edit can fake green.** The sibling gate records that it could be turned green
by appending two exempt prefixes and lowering two ceilings, with all ten tests passing while 281
findings silently left the enforced set. Any change to a policy JSON that lowers a ceiling or widens
an exemption must show the repair that earned it, in the same diff.

**T10 — `git grep -E "\b"` matches nothing.** POSIX ERE has no word-boundary atom, and it fails
SILENTLY, returning a clean zero. Every negative claim carries a positive control: the same pattern
over a corpus that must match. The confidential-computing claim in §1 is only usable because the
same pattern returns 58 lines over the archive.

**T11 — no unregistered placeholder markers in `docs/`.** `governance/check/placeholder-debt`
enumerates exactly two markers in `PLACEHOLDER_DEBT_TOKENS` (`src/lib.rs:8`), scans `docs/`, and
requires every occurrence to carry a registry record. Writing either literal into a `docs/` file —
including into a document that merely *discusses* them, which is why this trap is worded around them
rather than quoting them — creates an unregistered occurrence.

**T12 — `git add` then `git commit` commits the shared index.** The index is per-worktree and
therefore shared between lanes. Commit with a pathspec: `git commit -- <paths>`.

**T13 — one buck2 client per project root.** Concurrent clients cancel each other and report
"The evaluation of this key was cancelled: Rejected", which reads as a build failure and has been
misdiagnosed as one. Check `ps` for a neighbour before blaming a change. buck2 does not share cache
across worktrees, so this worktree's first build is cold.

**T14 — `docs/ADR-INDEX.md` is generated, but this repair does not touch its inputs.** The index
record is projected from FRONTMATTER only (`number, id, title, status, owner, date, path,
supersedes, superseded_by, related`). Body edits are inert for it. Do not hand-patch the index, and
do not assume a regeneration is owed — but any unit that touches apex FRONTMATTER owes one.

**T15 — do not write a realistic ADR id into a `.rs` fixture.** `.rs` is in `scan_extensions`, and a
plausible id reddens the citation gate. Use a token with no governed shape. Fixtures placed inside
`governance/check/adr-citation-closure` are exempt; fixtures in the apex gate crate are not.

**T16 — a single block can be very large, and "just carry whole blocks" is unbounded without a
measured escape hatch.** Measured over the 9,878 blocks in the 448 members' normative sections:
median 176 chars, p95 1,132, **max 21,038** (ADR-0244, one 373-line block). The largest normative
section is ADR-0562 at 2,674 lines. P1 never cuts inside a block, so the escape hatch is
ELIDED-WHOLE (P1 rendering 2), never a partial carry: a block over `max_single_block_chars` is
replaced whole by its structural placeholder. That trades fidelity for a bounded apex without
trading away honesty, because the placeholder declares itself. The constant is chosen in U2 from
this distribution and not before — see §8.

## 7. Definition of done — one unit

A reviewer holding only the diff applies this list. Any NO rejects the unit.

1. **Scope.** The diff touches only files this unit owns per §5. It commits directly to
   `impl/5ad-apex-truncation`, with a pathspec, and opens no PR.
2. **I1 visible in the diff.** No carried text ends mid-word, mid-table, mid-fence, or mid-sentence
   without a P2 marker on the following line. Grepping the diff's added lines for a line ending in a
   bare word at the block budget returns nothing.
3. **I2 arithmetic.** Every added P2 marker's `<k> of <n>` is consistent with the blocks rendered
   beside it. Spot-check one.
4. **I3 fail-closed.** Every new resolution path (heading, member key, successor edge) has a branch
   that ERRORS. A diff whose fallback is `unwrap_or_default()`, `continue`, or an empty string on
   the unresolved path is rejected — that is T4's exact failure.
5. **I8.** No byte-index slicing of a `str` in the diff.
6. **T5.** No added line under `docs/decisions/` contains `decisions/ADR-` or `adr-archive/ADR-` as
   a path.
7. **Frozen numbers.** If the diff adds or deletes any file outside the three exempt prefixes, it
   also changes `adr-citation-closure-policy.json` `files_scanned`, and the commit message states
   the value came from the gate's `observed N`. If it changes any apex finding count, it also lowers
   the matching ceiling in `apex-gist-integrity-policy.json` (T8). A repair with an unchanged ceiling
   is rejected as surely as a regression.
8. **Lifecycle.** Any added `docs/**/*.md` declares `doc_status:` in frontmatter (T7).
9. **Evidence.** The commit message carries literal buck2 output including its `Commands:` line for
   the gates governing the paths touched — at minimum
   `//governance/check/apex-gist-integrity:*` and, if any file was added or deleted,
   `//governance/check/adr-citation-closure:*`. A green claim with no pasted output is not evidence.
10. **Regression proof.** For a unit that changes gate behaviour: the failing-target set at the
    untouched base and at this head, DIFFED. Identical sets means zero regressions even when both
    sides fail, which is the common case here. A count alone cannot distinguish "fixed one, broke
    one" from "changed nothing".
11. **One runnable check.** Non-trivial logic leaves behind the smallest test that fails if the
    logic breaks. For U6 specifically, the required proof is a fixture truncated to red and then
    removed to green, both shown.
12. **No placeholders.** Neither `PLACEHOLDER_DEBT_TOKENS` marker (T11), no `test.skip`, no
    `unimplemented!`/`todo!` branch. A placeholder is a blocker to report, not evidence of progress.

## 8. What this document does not decide

- Whether the repaired apexes should ALSO carry hand-authored prose for the two dropped topics. The
  P1 block rule lands the ADR-0147 ladder table mechanically, which should drive `apex_topic_dropped`
  to 0 without authored prose. U5 measures it; if the count does not reach 0, the residue is genuine
  substantive loss and needs a ruling, not a longer prefix.
- The value of `max_single_block_chars` (T16). The distribution is measured (median 176, p95 1,132,
  max 21,038); the constant is not chosen, because choosing it needs the resulting apex file sizes,
  which only exist once U3 can render. U2 proposes it with that measurement attached. A constant
  chosen before the measurement is the fail-closed-on-an-unmeasured-threshold trap.
- Whether carrying each member TWICE (gist + residual) is intended. It doubles the repair surface,
  but Chesterton's Fence applies and no unit may deduplicate the two carry sites without a ruling.
- The 46-file gap between 432 archived files with `status: Superseded` and 386 members claimed
  across the ten `supersedes:` lists. ADR-0183 proves at least some are legitimate chains through
  non-apex successors; the rest are uncharacterised and out of scope for this repair.

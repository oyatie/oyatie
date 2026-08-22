---
doc_status: published
---

# Apex gist repair — pattern mapping (oyatie-5ad)

**What this document is, and is not.** It is a CONVENTION mapping for the units on
`impl/5ad-apex-truncation`: a reviewer holding only a unit's diff applies §7 to accept or reject it.
It binds DIFF REVIEW on this branch and nothing else. It is **not** a plan authority, it creates no
work-item id namespace (`MPV2-*` in `specs/masterplan.json` is the only live one), and it approves
no dispatch. Where it and a unit's judgement disagree, it wins for review purposes; amend it in the
same commit that departs from it, so the departure is reviewable. Where it is WRONG ON ITS FACTS,
say so in `refused` rather than obeying it — see §9.

Scope: the ten live apex ADRs `docs/decisions/ADR-0700..0709`, their 386 archived members under
`docs/adr-archive/`, the gate `governance/check/apex-gist-integrity` (already landed at c091256e7),
and the equality-pinned censuses that the repair moves.

## 0. Amendment log — review threads on PR #1630, by GraphQL id

Threads are addressed by ID, never by position; two agents' numbering of the same set drifts.

| Thread id | Verdict | Where answered |
|---|---|---|
| `PRRT_kwDOSbSl2s6XsXdV` (Cargo.toml:2, lock) | ACCEPTED | U-LOCK, §5; D1 |
| `PRRT_kwDOSbSl2s6XsM51` (BUCK:29, gate never selected) | ACCEPTED | U-WIRE, §5; D2; T18 |
| `PRRT_kwDOSbSl2s6XsM5y` (lib.rs:175, punctuation cuts) | ACCEPTED | U-DET, §5; D3; T17 |
| `PRRT_kwDOSbSl2s6XsM54` (tests:157, title discarded) | ACCEPTED | U-DET, §5; D3; T17 |
| `PRRT_kwDOSbSl2s6XsXdU` (mapping:233, U1/U2 share a file) | ACCEPTED | §3 module split; D5; I10 |
| `PRRT_kwDOSbSl2s6XsXdX` (mapping:142, needle set) | ACCEPTED IN FORM, its count REFUTED | P7; D6 |
| `PRRT_kwDOSbSl2s6XsXdR` (mapping:8, masterplan) | PARTIAL — demotion yes, registration REFUTED | header; D8 |
| `PRRT_kwDOSbSl2s6XsM50` (tests:349, pin identities) | DECLINED with a substitute | D4 |
| `PRRT_kwDOSbSl2s6XsXdT` (mapping:219, per-unit PR) | REFUTED | D7 |

## 0.1 Decisions (each is a ruling, not a preference)

Every question two units could answer differently is answered here, once.

**D1 — the Cargo.lock fix is its own unit, first, alone.** `check-apex-gist-integrity` is a workspace
member only via the `governance/check/*` glob in root `Cargo.toml:46`, and no lock entry exists. Run
`cargo metadata >/dev/null` (allowed; `cargo build/test/check/clippy` are hook-blocked), then
`git commit -- Cargo.lock`. **If the resulting diff touches any package other than
`check-apex-gist-integrity`, do NOT commit the extra drift — report it in `refused`**, because an
unattributed lock movement is indistinguishable from a dependency change nobody reviewed. Then
re-materialize both faces (T19). Reason it is alone: it is the entire red-to-green step for
`presubmit`, which is a pure fan-in — bundling it behind repair work keeps the PR red for no
reason.

**D2 — wire the gate with `synthetic_dependencies` seeds; do NOT touch `inert_selection_classes`.**
Seeds are the UNION over EVERY matching pattern
(`ci/facade/affected-target-set/src/lib.rs:779`), so a specific key coexists with `"docs/**": []`
rather than conflicting with it — the `docs/decisions/ADR-0704-k8s-port-live-apex.md` entry at lines
202-206 is the standing precedent for exactly this shape. `inert_selection_classes` is a licence for
an EMPTY selection, not a bar on selection, and the policy's own note says growing it is a
merge-authority change; adding a seed is not. Keys to add:
`"docs/decisions/ADR-07*.md"` and `"docs/adr-archive/**"` →
`root//governance/check/apex-gist-integrity:check-apex-gist-integrity-gate`.

**D3 — fix the two detector false negatives BEFORE any corpus repair, and re-derive the ceilings in
the same commit.** Both reviewer findings reproduce on disk: `ADR-0701:54` ends `…PRD-frontmatter
field,` (cut landing on punctuation, invisible to a mid-word predicate) and `ADR-0700:73` carries the
title `docs/policies/foundr):` whose closing delimiter survived, then ends its row on
`…Monitoring/observability systems, r`. A detector fix RAISES counts above the equality pins, so the
commit that fixes detection re-freezes the pins from the gate's own `observed N` (T17). Order is
load-bearing, not taste: repairing a corpus while the ratchet is blind to a defect class means the
repair's own ceiling movements are measured by an instrument that cannot see what it missed — the
`_parse_recovery_attribution` failure, restaged.

**D4 — DECLINE identity pinning; drive the ceilings to ZERO instead.** The thread is right that
per-code equality cancels a removal against a same-code addition. But a ceiling of `0` cannot
cancel: you cannot remove one finding, add another, and still observe zero. So U5 lands each code it
repairs at `0`, and only a code that provably cannot reach 0 gets per-`(apex, member, site)` identity
pinning, in that code's own commit with the reason stated. A 639-row frozen manifest rewritten by
every repair commit buys nothing that `0` does not.

**D5 — one file, one owning unit, enforced by splitting the module.** U1 and U2 both owning
`src/lib.rs` is a real defect against the branch's file-granular commit rule. Split:
`src/segment.rs` (U1), `src/render.rs` (U2), `src/lib.rs` retained by U-DET. See §3.

**D6 — the needle set is written out, not described.** Measured in this worktree at e07b090c9:

```
sev-snp|sev_snp|tdx|confidential comput|confidential-comput|confidential_comput
```

`grep -rniE` over `docs/adr-archive/*.md` → **9 files, 58 lines**; over `docs/decisions/ADR-07*.md`
→ **0**. That is the positive control, and it is what licenses the negative claim. The three
spellings are all required: spaced-only gives 6 (ADR-0297 writes "Confidential compute"),
hyphen-only gives 7 (loses ADR-0297, gains ADR-0128 and ADR-0352), and ADR-0308 is reached by
`confidential_comput` at line 1153 — **not** by an `nvidia_*` needle, contra the thread's second
clause. Three readers produced 6, 7 and 9 from the same claim purely by varying one spelling, which
is why prose describing a needle set is not a needle set.

**D7 — one PR, at the end, by the LAND phase. REFUTED: per-unit branches and PRs.** Every unit here
moves the same equality-pinned ceilings in `apex-gist-integrity-policy.json`, so N PRs serialise
against one another and cannot all merge — the second to arrive is red by construction. The measured
precedent is 23 draft PRs opened for one logical change, all stale, all conflicted, all superseded by
a single consolidation PR. Units commit DIRECTLY to `impl/5ad-apex-truncation`.

**D8 — do NOT register this in `masterplan_v2`; demote the document instead.**
`masterplan_v2.planning_entry_contract` is `state: "open"` with `binding_plan_approval_allowed:
false` and `dispatch_allowed: false`. Registering a work item there IS the binding plan approval the
contract currently forbids, so the thread's remedy would breach the authority it invokes. The half
of the thread that is right is cheap and is applied: this document no longer claims to be binding
plan authority (see the header). If the entry contract is later reopened, registration becomes the
correct follow-up — file it, do not do it here.

**D9 — scope of the PR this branch opens.** `U-LOCK`, `U-WIRE`, `U-DET` and this mapping are the
green-and-landable set and are the PR's floor. `U1`–`U5` (the generator and the corpus rewrite)
continue on the same branch; the LAND phase opens ONE PR over whatever has converged. If the
generator has not converged at the land gate, the branch lands the green set and the corpus repair
carries to a successor branch on the same mapping — **because the equality ratchet makes a PARTIAL
corpus repair RED**, so the repair is atomic or it is not landed. This is a sequencing ruling, not a
cancellation: D3 and D2 are hard prerequisites of a repair that can be measured at all.

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
generator") is unavailable here, and T14 records why that warning does not transfer.

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
comput"]`. The literal space cannot match `confidential-computing` or `confidential_computing`. The
corrected set is written out in **D6** and is the only form a unit may implement — the three misses
are ADR-0128 (which carries a named invariant `INV-CONFIDENTIAL-COMPUTE` no apex mentions), ADR-0352
(hyphenated) and ADR-0308 (underscored, line 1153). Currently harmless for the frozen count — all
three close onto ADR-0700/ADR-0709, already counted — so this is a latent under-detection, corrected
as DATA. **P8 still applies to the correction:** ADR-0352 is `status: Rejected`, so its absence from
every apex is RETIRED, not loss, and widening the needles must not silently reclassify it.

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

**One file, exactly one owning unit per run** (I10). The module split exists for that reason and for
no other; do not consolidate it back:

| File | Owner | Contents |
|---|---|---|
| `src/lib.rs` | U-DET | detector predicates, findings, policy load; re-exports the two modules |
| `src/segment.rs` | U1 | normative-section resolution + block segmentation (pure, no I/O) |
| `src/render.rs` | U2 | FULL / ELIDED-WHOLE / marker rendering (pure, no I/O) |
| `src/main.rs` | U3 | generator entry point, target `//governance/check/apex-gist-integrity:apex-gist-repair` |
| `tests/apex_gist_integrity.rs` | U-DET, then U5 | live-tree gate (existing) |
| `apex-gist-integrity-policy.json` | U-DET, then U5 | ceilings and needles; TEXT edits keyed by name |

`tests/` and the policy JSON are held SEQUENTIALLY by two units, never concurrently: U-DET releases
both before U5 starts. If you find another unit's uncommitted hunk in a file you own, do not commit
that file — name the hunk and its line and let the owning unit carry it.

**Policy is DATA.** All repo-specifics — budgets, heading aliases, needles, ceilings — live in
`apex-gist-integrity-policy.json`. Edit it as TEXT keyed by name. Round-tripping it through a JSON
serializer reformats the whole file and buries the one changed value.

**Ownership.** `governance/check/apex-gist-integrity/OWNERS` already exists (`council-architecture`).
There is no `governance/OWNERS` or `governance/check/OWNERS`, and the root OWNERS is past the
`[owners] max_paths_per_owners_file = 2000` cap in `ci.toml`, which fails CLOSED with no
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
- **I10 — one file, one owning unit.** Per §3. A unit that commits a file it does not own has
  committed a neighbour's mid-edit hunk under the wrong authorship; git records no provenance for
  uncommitted work, so this is asserted, never checked.
- **I11 — the ratchet sees before it judges.** No corpus-repair commit lands while a KNOWN detector
  false-negative class is unfixed (D3). A ceiling lowered by an instrument that cannot see the defect
  it is ratcheting is a fake green, and it is the exact shape the gate's own
  `_parse_recovery_attribution` already records.
- **I12 — the gate is selected by the corpus it polices.** After U-WIRE, a diff touching only
  `docs/decisions/ADR-07*.md` or `docs/adr-archive/**` seeds
  `//governance/check/apex-gist-integrity:check-apex-gist-integrity-gate`. Prove it by running the
  affected-set tool over such a diff and reading the target out of its selection, not by reading the
  policy back.

## 5. Unit decomposition

Every unit commits DIRECTLY to `impl/5ad-apex-truncation`. No per-unit branch, no per-unit PR.

| Unit | Deliverable | Owns | Depends on |
|---|---|---|---|
| U0 | this mapping | this file | — |
| U-LOCK | `cargo metadata` → lock the new workspace member (D1) | `Cargo.lock` | U0 |
| U-WIRE | affected-set seeds so the gate runs on corpus edits (D2) | `ci/facade/affected-target-set/affected-set-policy.json` | U0 |
| U-DET | punctuation + title detector fixes, ceilings re-derived (D3) | `src/lib.rs`, `tests/`, policy JSON | U0 |
| U1 | normative-section extractor + block segmenter | `src/segment.rs` | U0 |
| U2 | block renderer: FULL / ELIDED-WHOLE / marker | `src/render.rs` | U0 |
| U3 | generator binary + BUCK target | `src/main.rs`, `BUCK` | U1, U2 |
| U4 | inventory artifact (deliverable 1) | `_disposition/apex-gist-loss-inventory.json` | U1 |
| U5 | rewrite the ten apexes + drive ceilings to 0 (deliverable 2) | `docs/decisions/ADR-070*.md`, `tests/`, policy JSON | U3, U-DET |
| U6 | P7 needle correction per D6 + red-then-green fixture proof | policy JSON (needles only) | U-DET |
| U7 | LAND: batched bookkeeping, ONE PR | — | all |

**Parallel-safe:** `U-LOCK`, `U-WIRE`, `U1`, `U2` — disjoint files, no shared frozen number.
**Serialised on the policy JSON and `tests/`:** `U-DET` → `U6` → `U5`, in that order, because all
three write ceilings or needles into one file and the second writer would clobber the first's
re-derived `observed N`. **U5 does not parallelise against itself**: it alone rewrites the ten
apexes and alone moves the apex ceilings.

Units commit DIRECTLY to `impl/5ad-apex-truncation` with a pathspec (`git commit -- <paths>`), open
no PR (D7), and create no per-unit merge commit. All integrator-only bookkeeping batches into U7.

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

**T17 — fixing a detector RAISES the count, and the equality pin goes red.** This looks exactly like
a regression and the tempting "fix" is to narrow the detector back until the pin is satisfied, which
restores the blind spot. The correct move is to re-derive the pin in the SAME commit from the gate's
own `observed N`, and to state in the commit message that the rise is attributable to newly-visible
findings — naming the class and one exhibit (`ADR-0701:54`, `ADR-0700:73`).

**T18 — `synthetic_dependencies` seeds UNION; `inert_selection_classes` is a different statement.**
`src/lib.rs:779` unions the seeds of EVERY matching pattern, so a specific `docs/...` key does not
conflict with `"docs/**": []` — the ADR-0704 entry proves the shape lives. But the two lists mean
different things: `synthetic_dependencies[X] = []` says "X contributes no seed";
`inert_selection_classes` says "X may be the ENTIRE selection and still pass". Growing the second is
a merge-authority change (PR #1389: `.github/**` declared inert let a workflow-only PR resolve to
`NoGraphTargets` and walk past the no-new-shell ratchet). **Add seeds. Do not touch the inert list.**
Note also `src/lib.rs:1889` — a seed list carries an accountability check, so U-WIRE must RUN the
affected-set gate, not merely edit its policy.

**T19 — generated faces go stale the instant you commit.** They are generated from the tree, so a
face materialized before your commit describes the tree BEFORE it. Re-materialize after every commit
you intend to measure, with BOTH invocations:
`buck2 run //ci/facade/generated-artifact-freshness:cloud-ci-materialize-generated-faces-bin -- --repo-root .`
then the same with `--historical-merge-base $(git rev-parse HEAD)`. **The flag is misnamed:** despite
its name it demands HEAD EXACTLY — passing the real merge-base sha exits 1 AFTER writing only
scm-facts, leaving every other face stale, which inside a chained command silently leaves you
measuring a half-materialized tree. State in your evidence when the faces were generated relative to
the commit under test.

**T20 — `cargo metadata` may move more of `Cargo.lock` than your crate.** Read the hunks. An
unattributed lock movement is indistinguishable from a dependency change nobody reviewed; if
anything but `check-apex-gist-integrity` moved, report it rather than committing it (D1).

**T21 — `affected-set-policy.json` already carries a duplicate key** (`docs/decisions/ADR-0704-k8s-port-live-apex.md`
at lines 202 and 205), resolving last-wins with identical values, so it is inert. It is PRE-EXISTING
and not this branch's. Do not fix it here — an unrelated fix smuggled into a policy diff is how a
reviewer loses track of what the diff was for. Report it.

**T22 — a needle set described in prose is not a needle set.** Three readers derived 6, 7 and 9 from
the same claim by varying one spelling. Any pattern this branch relies on is written out literally
(D6) with its measured file and line counts and its positive control, or it is not relied on.

**T23 — `presubmit` is a fan-in with no independent cause.** It prints one line per constituent
lane and exits 1 if any is not green. Never debug it directly; read the lane it names. Two red checks
here were one defect.

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
13. **Ownership.** Every file in the diff is owned by this unit per §3/§5 (I10). A diff carrying a
    file another unit owns is rejected on sight, regardless of how correct the hunk is.
14. **Observed red, then green.** For any GUARD the unit adds or repairs: mutate the thing it is
    meant to catch, watch it fail, restore, watch it pass, and paste BOTH. A passing equality check
    cannot distinguish "the values are equal" from "the comparison never ran" — a collapsed scan, a
    skipped test and a correct tree all look identical from green. Run the mutation in a tree nobody
    else is writing; if you cannot get one, report the proof as unattributable rather than reporting
    it as a result.
15. **Face timing stated.** If the unit's evidence depends on a generated face, the commit message
    says when the faces were materialized relative to the commit under test (T19). A face older than
    the commit it describes is a stale-face false green.

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
- Whether U1–U5 converge in time to ride this branch's single PR. D9 rules the sequencing; it does
  not predict the outcome.

## 9. Evidence grade — challenge the inferences, obey the measurements

A wrong ruling here propagates to every unit and each will defend it in turn. So each ruling is
graded, and a unit that finds a MEASURED claim false should say so in `refused` rather than obey it.

**MEASURED in this worktree at e07b090c9** — re-run these before disputing them:

- The needle set and its counts (D6): 9 files / 58 lines over the archive, 0 over the ten apexes.
  Both the brief's figure of 37 and the thread's 7 fail to reproduce at any granularity.
- The truncations: `ADR-0701:83` ends `…a hosted-contr`, `:319` ends `…Kam`, `:54` ends `…field,`;
  `ADR-0700` contains `docs/policies/foundr):` and line 73 ends `…systems, r`.
- The block census: 254 gist bullets + 385 residual sections = 639, matching the gate's frozen
  `blocks`. ADR-0700 carries 60 gists against 126 residuals and ADR-0709 60 against 125 — which is
  what `apex_member_without_gist 132` and `apex_member_without_residual 1` are made of. **Correcting
  a claim in circulation:** those 132 members are missing a GIST, not missing all content; they
  retain residual sections. "132 members with zero carried content" is too strong.
- The seed-union semantics (D2/T18) at `ci/facade/affected-target-set/src/lib.rs:779`, and the
  ADR-0704 precedent at lines 200-230 of the policy coexisting with `"docs/**": []` at 238.
- `masterplan_v2.planning_entry_contract`: `state: open`, `binding_plan_approval_allowed: false`,
  `dispatch_allowed: false` (D8).
- The duplicate policy key (T21), at lines 202 and 205, values identical.

**INFERRED — worth challenging, and cheap to settle by running something:**

- That `cargo metadata >/dev/null` alone turns `freshness` green (D1). It is the remedy the gate
  itself prints and the manifest glob already covers the crate, but nobody has run it. U-LOCK
  settles it.
- That the D2 seed keys actually produce the selection (I12). Glob syntax and the accountability
  check at `src/lib.rs:1889` are both unverified; U-WIRE must prove selection by RUNNING the tool
  over a docs-only diff, not by re-reading the policy.
- That driving ceilings to 0 is reachable for every code (D4). If a code cannot reach 0, that code
  falls back to identity pinning — the ruling anticipates the failure but has not measured it.
- That a body-only apex edit leaves `docs/ADR-INDEX.md`, `docs/machine-readable/decisions.json` and
  `ci/facade/lifecycle-status` unmoved (T14, T7). Both project from FRONTMATTER, and the apexes
  already declare `doc_status`, so the inference is strong — but it is an inference from module
  docs, and U5 owes the measurement.
- The whole of §2's P1/P2 rendering contract. It is a design ruling, not an observation; it is
  binding because it must be decided ONCE, not because it was measured.

**NOT COVERED by this mapping:** no buck2 target was run to produce it. Every count above comes from
git, grep, file reads and CI logs. No gate has been observed passing or failing locally at this head,
and no assertion in the landed gate has been proved to fire by mutation. The first unit to run one
owes the branch that evidence.

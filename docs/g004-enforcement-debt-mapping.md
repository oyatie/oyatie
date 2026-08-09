# G004 burn-down-enforcement-debt — unit mapping

Binding for every unit committed to `impl/g004-enforcement-debt`. A unit that disagrees with this
document is wrong, or this document is amended in the same commit that disagrees with it.

Base: `origin/dev` @ `1d3105277`. Every fact below was read from the live tree at that commit, not
from the beads — three of the five beads were measured days earlier and two of them have since gone
stale in ways that would have caused re-work.

The goal is one sentence: **close the defect classes this programme found in its own machinery.**
The constraint is one sentence: **never lower a floor or a ceiling to reach green** — that is the
exact failure the beads describe, so committing it while claiming to fix it is the worst available
outcome.

---

## 0. The one-PR rule and the commit shape

Every unit commits **directly** to `impl/g004-enforcement-debt`. No unit branch, no merge commit, no
pull request. Exactly one PR is opened at the end, by the Land phase, only once the gates are
already green. Integrator-only bookkeeping (hotfile re-anchors that are pure bookkeeping) is batched
into a **single** Land-phase commit over the final tree, never repeated per unit.

Commit subject: `fix(<gate-or-crate>): <what structurally changed>`. Not `<which number moved>` —
if the subject can only be phrased as a number moving, the unit is probably a re-anchor, and
re-anchors are Land-phase work.

---

## 1. The four recurring patterns, and exactly what each becomes

### P1 — A ratchet whose guard opposes its own northstar

**Shape.** A minimum (`min_expected_*`) asserted on a term whose target value is ZERO. Honest
progress toward the goal trips the anti-vacuity guard, so every wave lowers the guard, and the guard
becomes a number that only ever falls — a ceiling that only falls is the tell.

**Becomes.** Delete the floor. Make the *ceiling* on the same term two-sided:

| direction | code | severity |
|---|---|---|
| observed **above** ceiling | `CODE_<TERM>_REGRESSION` | blocking |
| observed **below** ceiling, ceiling not re-frozen in the same change | `CODE_<TERM>_DROP_UNATTRIBUTED` | blocking |

The below-case is the attribution guard. A drop has exactly two causes — the artifacts genuinely
entered the graph, or the walk mis-attributed them — and the counts cannot distinguish them, because
the sibling census terms are invariant under mis-attribution. So the gate stops guessing and demands
a human state which it was, in the change that caused it, where a reviewer can read it.

Three properties make this correct where the floor was not:

1. it fails closed on the same collapsed walk the floor caught;
2. it is **monotone in the same direction as the programme's own goal**;
3. **zero is a stable fixed point** (`0 < 0` is false), so reaching the northstar never requires
   touching the guard.

**Landed exemplar — copy this one.** `ci/facade/corpus-index-coverage`, commit `8d86245cb`.
`min_expected_unpackaged_yaml_files` deleted; `baseline_unpackaged_yaml_files` (75) equality-pinned;
`CODE_UNPACKAGED_DROP_UNATTRIBUTED` added. **No count changed** — 75 is the same measurement, now
pinned from both sides.

**This is not "delete all floors."** `min_expected_yaml_packages` and `min_expected_yaml_files`
stay, and `min_authority_surfaces: 3` in the citation gate stays, because those are floors on terms
whose goal direction is UP. The test is not "is it a floor", it is **"does the programme's own
success violate it?"**

### P2 — Modeled crypto reachable from production

**Shape.** A deterministic stand-in for a cryptographic primitive is plain `pub` API. A runtime-only
guard does not help: a caller can reach it in production and nothing stops the link.

**Becomes.** A compile-time impossibility, not a check:

```rust
#[cfg(any(test, feature = "modeled-crypto"))]
pub fn derive(seed: &str) -> Self { ... }
```

with, in `Cargo.toml`, a non-default feature carrying the reason:

```toml
[features]
modeled-crypto = []
```

and — load-bearing — the crate's `BUCK` `rust_library` declares **no** `features` argument, so no
production target can turn it on. `cfg(test)` supplies it to the crate's own tests only.

Off-feature the symbol does not merely become private, it does not **exist**: the failure is
`error[E0599]`, so no caller reaches the material and no downstream type can be constructed from it.

**Landed exemplar.** `os/core/cluster-mgmt-domain` and `os/core/trustd-domain`, commits `e355e7f59`,
`deeda0c8b`, `6a7fe09f6`.

**Read trap T1 before applying this to any third crate.** The translation is not mechanical.

### P3 — A governed root list hard-coded in Rust

**Shape.** A gate's scan scope lives as a `const NAME: [&str; N]` array in a test file. A capability
rehome moves files out of the listed roots; the scan silently stops covering them; the gate's total
falls; the lane lowers the total to match. The gate reports green while covering less. The loud
half is a red gate; the **silent half — a floor slack enough to absorb the loss — is the dangerous
one, and it is invisible by construction.**

**Becomes.** The root list moves into the policy JSON as DATA, read by the test with a non-empty
assertion, and every corpus in that gate walks the **same** list.

**Landed exemplar (already on dev, do not redo).** `ci/facade/product-protocol-policy`:
`manifest_inventory.governed_roots` (22 roots) read by `governed_roots(&policy)` at
`tests/product_protocol_policy.rs:113`, asserted non-empty at `:117`, shared by both the proto and
manifest corpora (`:1194`, `:1198`).

**Still hard-coded, and in scope:**
- `ci/facade/slo-coverage/tests/slo_coverage.rs:14` — `REQUIRED_SLO_LINKED_CLOUD_MANIFESTS: [&str; 6]`
- `ci/facade/policy-deploy-parity/tests/cedar_deploy_parity.rs:83` — `GH_987_CLOUD_PATHS: [&str; 14]`

All 20 of those paths currently EXIST on dev — the bead's "all six stale" does not reproduce, PR
#1620's rehome repointed them. So these are **not** broken today; they are the same latent class,
and a unit that touches them must say it is pre-empting, not repairing.

### P4 — An equality-pinned census moved by an edit

**Shape.** `governance/check/adr-citation-closure` pins `files_scanned`, `citation_lines` and
`adr_records` by EQUALITY, asserted **before** any finding count. A narrowed scan and a genuine
add/delete produce the same number and only one is legitimate.

**Becomes.** Re-freeze in the **same commit**, with an attribution string that names the direction,
the real thing that moved, and the bead or PR. A number that changed with no attribution is
indistinguishable from a number lowered to reach green.

---

## 2. Naming, module and ownership conventions

**Finding codes.** `pub const CODE_<SHOUTY_SNAKE>: &str = "<gate_subject_stem>_<snake_case>";` —
e.g. `corpus_index_unpackaged_drop_unattributed`, `adr_citation_rejected_authority`. The doc comment
on the const states the defect, the severity, and *why the remedy is what it is* — the const's
doc comment is where a future reader learns whether the rule may be relaxed.

**Layering, non-negotiable.**
- Policy JSON = **DATA**. All repo specifics — paths, roots, ceilings, extensions — live here.
  Another repo adopts the gate by repointing these values.
- Rust kernel (`src/lib.rs`) = **PURE**. No I/O, no clock, no rand, and **no repo path literals**.
  The caller walks the tree and passes observations in.
- Tests = the live-tree binding. This is the only layer that touches the filesystem.

**Attribution keys in policy JSON.** `_<what>_<yyyy_mm_dd>[_<ref>]`, e.g. `_corpus_pin_2026_08_08`,
`_rebase_remeasure_2026_08_08`, `_corpus_remeasure_2026_08_09_pr1623`. Append; never rewrite an
existing key — the sequence of keys is the audit trail of the ratchet.

**Test names are full sentences stating the property**, not the function under test:
`honest_progress_toward_zero_is_never_blocked_by_lowering_the_guard`,
`an_attribution_collapse_fails_closed_while_both_census_floors_hold`,
`a_rejected_adr_cited_as_a_path_is_caught_not_only_when_named_bare`.
A reviewer reading only the test list must be able to see which property is claimed.

**Crate feature names** are kebab-case and describe the *hazard*, not the mechanism:
`modeled-crypto`, never `test-only` or `insecure`.

---

## 3. Invariants — must hold after EVERY unit, checkable on the diff alone

**I1. No frozen number moves without attribution in the same diff.** Applies in both directions.
A lowered floor or ceiling with no attribution is an automatic reject.

**I2. Every guard added or repaired ships with a test proving it FIRES.** A rule never seen to fire
is the false green it exists to prevent. "The test exists" is not the bar; the bar is that removing
or defeating the guard makes a named test fail, and that this was **observed by execution** and the
observation recorded (buck2 output including its `Commands:` line, or the exact failure message and
the `file:line` it fired at).

**I3. Corpus hygiene.** `git diff --name-status origin/dev...HEAD` — every `A`/`D`/`R` line for a
`.md .json .jsonl .yaml .yml .toml .rs .cedar .txt` path outside the three exempt prefixes
(`docs/adr-archive`, `docs/decisions/_disposition`, `governance/check/adr-citation-closure`) obliges
a citation-census re-freeze in the same diff. All-`M` ⟹ nothing owed, and the diff should say so.

**I4. Gates governing the touched paths run green locally before the push**, not in CI. Six of the
ten repairs on the reference PR were CI-caught at 30–70 minutes a round trip when a local run would
have caught them in seconds.

**I5. This goal moves no files.** If a unit does move one, it additionally owes a whole-graph
`buck2 build //...` plus the co-moved registry rows and BUCK packages. Prefer not moving anything.

**I6. No guard is an `assert!`.** See T10.

**I7. The commit graph stays flat**: direct commits, zero merges, zero unit branches, zero PRs.

---

## 4. TRAPS — where the obvious translation is subtly wrong

**T1 — P2 does not transfer to `os/core/secrets-domain`, and this is the single most important
trap in the document.** The crate holds a byte-identical twin of the modeled primitives, and unlike
the two gated crates it sits on **production** paths. But applying the `#[cfg(...)]` verbatim
**breaks the build**, because unlike `Secret::derive` (zero callers outside its own file), the twin
has real production callers — every one of them before its file's `#[cfg(test)]`:

| call site | `#[cfg(test)]` in that file |
|---|---|
| `src/api.rs:92` | `:134` |
| `src/trustd.rs:77` | `:232` |
| `src/etcd.rs:69` | `:215` |
| `src/kubernetes.rs:105` | `:264` |
| `src/bundle.rs:595`, `:839`–`:842` | `:923` |

`KeyPair::from_seed` sets the private key to the seed **verbatim**
(`let private_der = seed.as_bytes().to_vec();`), `InMemorySigner` is a keyed FNV MAC rather than a
signature, and `src/lib.rs:261` persists the cluster seed as **plaintext on disk**
(`format!("seed={seed}\ncreated_at={created_at}\n")`) and reloads it at `:288`–`:290` — so every CA
is regenerable from one line of a file. Gating the constructor alone therefore does not close this;
the callers are the defect. A unit here must decide, explicitly and in the commit message, between
gating the whole crate, gating the callers with it, or giving the callers a real implementation.
**Do not open this unit expecting the cluster-mgmt diff to apply.**

Verified on this branch, control-proven: `git grep -c modeled-crypto` matches **9** files across the
two gated crates and **zero** in `os/core/secrets-domain`.

**T2 — no test inside the crate can watch the P2 gate bite.** The gate is
`cfg(any(test, feature = "modeled-crypto"))`, so the test build always has the feature. A test
asserting the symbol is absent cannot compile in the configuration that would run it. The proof is
an out-of-band build of the **production** target with a probe added, and the probe target is
deliberately **not** committed — a permanently-broken target would redden `buck2 build //...` for
every lane, which is exactly what the affected-set FULL tier runs.

**T3 — a source-text guard cannot detect its own removal.** The landed
`modeled_crypto_constructors_stay_behind_the_gate` asserts the attribute string appears next to each
signature. A gate commented out with `//` still matches that string. Treat it as guarding against
*accidental deletion*, not as proving the gate holds. The claim "the gate holds" rests on the
recorded E0599 execution, not on this test. Do not let a reviewer read it as stronger than it is.

**T4 — POSIX ERE has no word-boundary atom.** `git grep -E "\bFOO"` matches **nothing**, silently,
and reads as a clean negative. Every negative claim in a unit must be accompanied by a **control**
run of the same pattern against something it is known to match.

**T5 — a piped grep reports the wrong exit code.** `git grep X | head; echo $?` prints `head`'s `0`
regardless. Use `(git grep -n X || echo "ZERO MATCHES")`.

**T6 — `.omc/ultragoal/*` is a frozen four-path allowlist** (`.gitignore:10`, with an explicit "do
not expand" comment). Do not add a fifth file there. Separately: **any dot-directory is invisible to
the citation census**, so a document parked there proves nothing about corpus hygiene and quietly
opts out of the discipline this goal is about. That is why this file lives under `docs/`.

**T7 — adding a fourth authority surface moves TWO numbers.**
`docs/AGENTS-OPERATING-CONTRACT.md` asserts Rejected ADR-0347's doctrine under its own `## ADR-0347`
heading (`:9`, `:23`, `:25`) and is **not** in `authority_surfaces`, so the rejected-authority rule
cannot see it — the same class as the bead's headline finding, one file over. Adding it moves the
observed surface count against `min_authority_surfaces: 3` **and** raises
`adr_citation_rejected_authority` above its frozen `3`. Both must be re-measured together, in one
commit, with attribution. Do not move one and leave the other.

**T8 — a new file must name ADRs bare.** `scan_line` puts an id in `cited` only when the text
immediately before it is `decisions/` or `adr-archive/`. Anything in `cited` that does not resolve
to a live apex is a `CODE_DANGLING_CITATION` against a frozen ceiling, and only a **single** `cited`
id can trigger the mismatch rule.

Writing ADR ids bare keeps `cited` empty, and `tests/adr_citation_closure.rs:228` then skips the
line entirely — `if cited.is_empty() && !(authority_surface && !context.is_empty()) { continue }`.
So a new **non-authority** file with bare ADR ids moves `files_scanned` **only**: not
`citation_lines`, not any finding count. Measured on this very document: it names ADR ids on 2
lines, and the gate moved `files_scanned` 16518→16519 with `citation_lines` unchanged at 8896.

The corollary is the sharp edge: the same two lines placed in `CLAUDE.md`, `AGENTS.md` or
`docs/AGENTS.md` **would** become citations, because those are authority surfaces where bare context
counts — and one of them names Rejected ADR-0347, which would raise
`adr_citation_rejected_authority` above its frozen 3. Bare-is-free is a property of *non-authority*
files.

**T9 — edit policy JSON as TEXT keyed by name.** Round-tripping through a JSON parser reformats the
whole file, which buries the one-value change in a whole-file diff and defeats the review that the
attribution exists to enable.

**T10 — the guard must survive release, and must not do arithmetic.** Two specific failures:
- `assert!`/`debug_assert!` as a guard. `debug_assert!` **vanishes** in release builds, so the rule
  silently stops existing exactly where it matters. Kernel guards are always
  `if <condition> { <finding> }` returning DATA — never a panic, never an assertion.
- Subtracting counts. These censuses are `usize`. `baseline - observed` **panics** in debug and
  **wraps** in release, and a wrapped `usize` compares as enormous, which reads as a passing
  ceiling. The two-sided rule **compares** (`observed < baseline`, `observed > baseline`); it never
  subtracts.

**T11 — a gate re-frozen from a local buck2 run is not a CI verdict.** PR #1623 self-disclosed a
GREEN CI carrying a red census, because its affected-set derive resolved `NO-GRAPH-TARGETS` and
never executed the census test. Tracked as bead `oyatie-1ld`. Any unit touching a hotfile policy
must assume its own CI green may be vacuous and say which verdict its re-freeze came from.

**T12 — bead `oyatie-zng` is an authorization problem, not an engineering one, and the obvious fix
is forbidden.** PR #1623's measured profile already answers it: the job is 86% one cold
`buck2 build //...` (1201s of 1409s, `Cache hits: 0%`, 14362 local actions), and the bead's own
suspected cause — a second full test pass — is **refuted** (`buck2 test //...` is 198–207s because
it reuses `buck-out`). The only lever is warm CAS, which ADR-0700 live hard norm 4 holds fail-closed
(`specs/cache-warm-license.json`: `warm_reads_licensed: false`, `licensed_by_canary_run: null`), and
the resolver is not even invoked from the `gate-affected-target-set` job. The cheap alternative —
narrowing the FULL-tier triggers — **buys wall clock by checking less**, which is the constraint
this goal exists to defend. **Not implementing zng is the correct outcome available to this lane.**
Also: the job is trimodal (6.0m / 8.5m / 26–29m), so the bead's 29.1-of-29.4 figure is the FULL tier
only, and any improvement quoted without naming its tier is unfalsifiable.

**T13 — do not "fix" the line-scoped attribution defect (bead `oyatie-9xj` item 1).** It was
deliberately **refused**, not deferred, and the refusal is recorded in the policy's
`known_limitations`: widening attribution to the surrounding block manufactures accusations against
correct citations. Re-opening it contradicts a recorded design decision.

**T14 — two bead headlines do not reproduce; do not quote them.**
- `oyatie-whk` claims `expected_total` fell 100→71 over six waves. On mainline it was **born at
  101**, changed **three** times in seven days, and the most recent change **raised** it: 101 →
  100 (#1584) → 95 (#1611) → **96** (#1620). Today it is `96`. The 100→71 sequence lived on PR
  #1620's pre-squash branch and never reached dev — the same lane then refused to lower 71 and
  restored the measured value. Re-derive with `git log -L` before quoting any number.
- `oyatie-9xj.1` items 1+2 are **already fixed and merged on dev**. The gate now sees
  `docs/AGENTS.md:57`; `adr_citation_rejected_authority` moved 2→3 and is frozen there. The citation
  itself is still wrong in the corpus — it is the gate's *visibility* that was repaired, not the
  document. Do not re-implement the oracle fix.

---

## 5. Unit ledger

| # | bead | status | remaining work |
|---|---|---|---|
| 1 | `oyatie-ln1` | **DONE** `8d86245cb` | — pattern P1, landed exemplar |
| 2 | `oyatie-qnf` | **PARTIAL** `e355e7f59` `deeda0c8b` `6a7fe09f6` | `os/core/secrets-domain` twin still ungated — see **T1** |
| 3 | `oyatie-9xj` / `9xj.1` | mostly already on dev | only `docs/AGENTS-OPERATING-CONTRACT.md` as a 4th authority surface — see **T7**; item 1 refused — see **T13** |
| 4 | `oyatie-zng` | **REFUSED** | blocked at an authorization gate — see **T12**; refusal is the deliverable |
| 5 | `oyatie-whk` | headline refuted | P3 pre-emption on `slo-coverage` + `policy-deploy-parity` only — see **T14** |

---

## 6. DEFINITION OF DONE for one unit

A reviewer holding only the diff applies these seven. Any single failure rejects the unit.

1. **Scope.** The diff addresses one bead's defect class. The subject names what *structurally*
   changed, not which number moved.

2. **Attribution.** Every frozen number that changed carries, in the same diff, a string naming
   (a) the direction, (b) the real thing that moved, (c) the bead or PR. *A number that changed
   with no attribution is a reject.*

3. **No green bought by weakening.** No floor or ceiling lowered to reach green. Where a ratchet was
   structurally defective, the diff **changes the structure** and says so in prose. Where a number
   genuinely moved, it is re-frozen with attribution. No third option.

4. **The guard is proven to fire.** A test in the diff states the property in its name, and the diff
   (commit body, or a comment at the test) records the **execution** that proved it fires — buck2
   output including its `Commands:` line, or the exact failure message and the `file:line`. "The
   test exists" is not evidence. If the proof could not be automated, the diff says why, and says
   what weaker thing the committed test actually guarantees (see T3).

5. **Mechanics.** No `assert!`/`debug_assert!` as a guard (T10). No arithmetic on census counts
   (T10). Policy JSON edited as text (T9). No `cited` ADR ids added by new files (T8). Every
   negative claim in the commit message backed by a control (T4, T5).

6. **Corpus hygiene.** `git diff --name-status origin/dev...HEAD` is all-`M`, **or** the citation
   census is re-frozen in the same diff with attribution (I3). The diff states which of the two.

7. **Shape.** Direct commit on `impl/g004-enforcement-debt`. No merge commit, no unit branch, no PR.
   Local gates governing the touched paths are green, and the diff or commit message shows the
   buck2 verdict line.

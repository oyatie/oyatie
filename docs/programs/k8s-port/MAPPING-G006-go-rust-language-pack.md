---
doc_class: Program-Mapping-Contract
doc_status: published
authority_tier: 2
---
# G006 Phase 1 — mapping contract for the universal Go→Rust language rule pack

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-09) |
|---|---|---|
| Repository baseline | `origin/dev` @ `885794461223cf0f777e5be2154acf1ff76d9db9` | Current baseline. Branch `impl/g006-language-rule-pack` is cut from exactly this commit. |
| Upstream Kubernetes pin | `v1.36.1`; annotated tag object `5b824a493a7ca248b726b6ea09d53842b9b992c2`; peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Unchanged. Not consumed by this lane — no snapshot is taken and no output is generated. |
| Engine | `build/port-engine/*`, v0 | W0 seams and refusals only (PR #1621). No `SourceModel` producer, no `Renderer`, no matcher. Not in force as a producer. |
| Neutral rule pack | `specs/port-rules/**`, v0 — unauthored on `origin/dev` (0 files) | THIS LANE AUTHORS IT. Not in force until its units land. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Unchanged by this lane. |
| Go front end | Bootstrap extractor; strategy ruled out of band (ADR-0638 D3) | Not in force. `oyatie-qno` (may the extractor itself be Go) is an UNMADE founder ruling. This lane neither makes nor assumes it. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six axes typed in the kernel. `rulepack_digest` becomes computable for the first time when this lane lands; no receipt is emitted here. |
| Program authority | ADR-0637 / ADR-0638, archived under `docs/adr-archive/`, live via apex ADR-0704 | Accepted 2026-08-05; apex accepted 2026-08-06. W0 only. |
| Census evidence | `docs/programs/k8s-port/census/*.md` on branch `docs/k8s-port-census` @ `da7567f406f488623dd6b762fbd328fa5e7fbfa0` (PR #1625, OPEN) | NOT ON `dev`. Every census figure in this document is cited against that commit. See §9 sequencing. |

**This document is a CORPUS-side program record.** It sits under `docs/programs/k8s-port/` and it
names Kubernetes deliberately and freely. It is not rule data, it is not neutral, and the
neutrality gate of §5 does not scan it. Nothing in this file may be copied verbatim into
`specs/port-rules/**`.

---

## 0. What this lane is, and the three things it is not

G006 Phase 1 authors the **universal Go→Rust LANGUAGE rule pack**, enforces the language/corpus
split mechanically, and harvests `os/`'s hand-written domains into the divergence record.

It is **not** the generation of a Kubernetes port. It is **not** a Go front end. It is **not** a
ruling on `oyatie-qno`. Everything below is authorable from the landed W0 seams plus the five shape
censuses, with no extractor of any kind, and is required before any generation regardless of how
that ruling goes.

The single sentence that governs every unit:

> **Meaning is TRANSLATED; an apology for Go is ELIMINATED; genuinely unclear is a DIVERGENCE.
> Imitation is the default failure, and imitation never looks wrong in review.**

---

## 1. Locations, ownership and accounting — decided before any file is written

Location here is an accounting decision, not a filing preference. All four destinations were
measured against the live constraints before this document was written.

| What | Path | OWNERS resolution | Reachability prefix | Verdict |
|---|---|---|---|---|
| Neutral language rules | `specs/port-rules/lang/go-rust/<RULE-ID>.md` | `specs/OWNERS` = `council-architecture`. `specs/` tracks **373** paths against `oya-ci.toml [owners] max_paths_per_owners_file = 2000` — 5.4x headroom. **No new OWNERS file.** | **ABSENT.** `specs/reachability-registry.json` has no `specs/port-rules/` prefix (it has `specs/k8s-port/` at :538 and `docs/programs/k8s-port/` at :166). | **UNIT 0 must add the prefix before the first rule file lands.** Otherwise ADR-0555 born-accounting fails unjustified + unreachable. |
| Corpus policy (universality pin, licensing record) | `specs/k8s-port/<name>.json` | `specs/OWNERS` | present (:538) | Ready. |
| Program records (this file, the `os/` disposition table, the universality report) | `docs/programs/k8s-port/<NAME>.md` | `docs/programs/k8s-port/OWNERS` = `axis-cloud-platform`, `council-architecture` | present (:166) | Ready. **Must carry the baseline header — gate-enforced, see I8.** |
| The split gate | `ci/facade/k8s-program-docs/` | existing | existing | Ready. **Do not create a new gate crate** — see D5. |

`docs/` at top level is structurally unownable (nearest-ancestor OWNERS fails closed above 2000
paths, and a `docs/OWNERS` would cover 2631). That hazard does **not** apply to any of the four
rows above, which is why they were chosen. No unit may add a file outside these four prefixes.

---

## 2. The rule-record format — DECIDED, and the reason the obvious layout is illegal

### D1. Every file under `specs/port-rules/**` is a Markdown rule record. There is no `index.json` and no `fixtures/` subtree.

The landed R-DOC gate `ci/facade/k8s-program-docs` already owns `specs/port-rules/**`, recursively,
today, on `dev`. `load_rule_records()` walks **every regular file** under that root and returns
`Err("rule records must be Markdown with YAML-style front matter")` for any path whose extension is
not `md`. It then calls `ensure_only_fields(&fields, &["rule_id","rule_kind","operations_journal_ref"])`,
which errors on **any** unknown front-matter key, and `required_field` rejects an empty value for
each of the three.

`docs/programs/k8s-port/README.md`'s traceability table names `specs/port-rules/index.json` and
`specs/port-rules/canary/index.json`. **Those two paths cannot exist under the landed gate.** A
`.json` there is a hard load error — the gate goes red on load, not as a finding, and it takes every
other R-DOC check down with it.

Three options existed. The ruling:

| Option | Blast radius | Ruled |
|---|---|---|
| Author rules as `.md` only; carry ordering and fixtures inside the records | zero — conforms to a landed contract | **TAKEN** |
| Widen the gate's accepted extensions | weakens a landed fail-closed contract, needs its own adversarial review, and the widening is exactly the shape of "a gate that passes because it observes nothing" | rejected |
| Move the rule root | contradicts ADR-0637 D1, which fixes `specs/port-rules/lang/**`, `idiom/**`, `canary/**` by name as the neutral rule data | rejected |

**Consequences a reviewer must be able to check:**

- **Rule ORDER comes from the filename, not from an index.** `RulePack::rules()` must return a
  deterministic order — `plan()` fails closed on "declared rules handed back in an order that is
  not the pack's own". The pack's order is therefore defined as **byte-lexicographic ascending on
  `rule_id`**, which equals directory listing order because the file is named `<RULE-ID>.md`. One
  mechanism, derivable from the tree, no index file to drift.
- **Fixtures are fenced code blocks inside the rule record.** ADR-0637 D3 makes the review object
  "the rule corpus and its fixtures"; one self-contained file per rule is the reviewable unit.
- **The README contradiction is recorded, not routed around.** Unit 0 files it as a named open
  question against the program README with an owner. Do not silently create the JSON and do not
  silently edit the README to match.

### D2. Rule identity grammar

```
rule_id  ::= "GO-RUST-" FAMILY "-" NNN
FAMILY   ::= CONC | CHAN | DEFER | PANIC | ERR | IFACE | TYPE | OWN | STR | REFL | GEN
NNN      ::= three ASCII digits, zero-padded
filename ::= <rule_id> ".md"
```

- Zero-padded so byte order equals numeric order (see D1).
- **Disposition is NOT in the ID.** A disposition can change under review; a rule ID is cited by
  journal entries, receipts and fixtures and must never be renamed. Disposition lives in the body.
- Numbers are allocated in tens (`010`, `020`, …) so an ordering-constrained rule can be inserted
  between two existing ones without renaming either. **Ordering is load-bearing**: the census
  requires the 19 IIFE `defer` sites to be rewritten *before* the shape-2 detector sees them
  (`census/defer-panic-recover.md` §4, "This rule is safe and should be applied *first*"). Where
  rule A must precede rule B, A's number is lower **and** A's `## Ordering` section names B.
- `FAMILY` values name Go language surfaces only. `GEN` = generator-output rules (see D4).
- Globally unique across `specs/port-rules/**` — `plan()` refuses a duplicate declared rule id
  because it makes plan order ambiguous.

### D3. Directory and pair slug

`specs/port-rules/lang/go-rust/` — **flat**, no family subdirectories. The kernel's
`LanguagePair::slug()` joins two slugs with `PAIR_SEPARATOR = '-'` and fail-closes on any byte
outside ASCII lowercase alphanumeric, `_`, `+`; `-` is excluded from the slug grammar and that
exclusion is asserted at compile time. So the directory segment is exactly `go-rust` and nothing
else. Flat keeps *filename order == rule_id order == application order* as one mechanism instead of
three.

### D4. Rule record body schema — fixed H2 sections, in this order

```text
---
rule_id: GO-RUST-<FAMILY>-<NNN>
rule_kind: neutral
operations_journal_ref: <entry-id under docs/programs/k8s-port/operations/>
---
# <rule_id> — <one-line name>

## Disposition
## Go construct
## Derivation
## Rust result
## Ordering
## Anti-pattern guard
## Fixture
## Residue
```

| Section | Required content | Mechanically checkable |
|---|---|---|
| `## Disposition` | Exactly one bare token on its own line: `TRANSLATE`, `ELIMINATE`, or `DIVERGE`. | yes |
| `## Go construct` | The source shape, in Go syntax, with the property that makes it this shape. | no |
| `## Derivation` | **At least one LANGUAGE-AUTHORITY citation** (Go specification clause, `std`/`tokio`/`tokio-util` documented behaviour) and **optionally** a shape citation. See D6. | yes (presence) |
| `## Rust result` | For TRANSLATE: the emitted Rust. For ELIMINATE: the literal sentence `Emits nothing.` plus what carries the meaning instead. For DIVERGE: `No mapping. Refuse and name the construct.` plus the ledger row it proposes. | yes (token match per disposition) |
| `## Ordering` | Rule IDs that must run before / after this one, or `None.` | yes (IDs resolve) |
| `## Anti-pattern guard` | The wrong mapping that COMPILES, stated as a condition that fires. `None known.` is a permitted value and is a claim, not a blank. | no |
| `## Fixture` | One ` ```go ` block, and either one ` ```rust ` block or, for ELIMINATE, the absence of one plus an explicit statement. This is the rule's selecting fixture. | yes (block presence) |
| `## Residue` | What this rule does NOT cover, and the count if measured. `None.` is permitted. | no |

### D5. Where the split gate lives

**Extend `ci/facade/k8s-program-docs`. Do not create a new gate crate.** That crate already loads
every file under `specs/port-rules/**`, already parses `rule_kind: neutral|corpus`, and already has
a `Finding`/`FindingCode` vocabulary — it enforces nothing about content, and content is exactly
Deliverable 4's gap. A sibling gate would additionally need a workflow row in
`.github/workflows/oya-ci-required.yml`, a `registry/catalog/` entry, a reachability prefix, an
affected-set row, and a `ci/facade/gate-self-conformance` `no_autofix_reason` entry — five surfaces,
each of which has already reddened a PR in this repo.

The kernel's compile-time const scan is **not** the place either: `build/port-engine/core/port-engine-kernel/tests/neutrality.rs`
records that "two more const passes crossed rustc's `long_running_const_eval` budget (measured)".
Extending the const block is measured-blocked. The gate crate is the indicated home.

Token list: the five needles the kernel already uses. The gate crate **duplicates** them rather than
depending on `build/port-engine/*` — `ci/facade/*` depending on `build/*` is a layer inversion. The
duplication is five byte strings with a comment naming the other copy; a shared crate for five
strings is not worth a new dependency edge.

---

## 3. D6 — the mechanical test that separates a LANGUAGE rule from a CORPUS rule

This is the whole point of the lane, so it is a test, not a principle.

> **A rule belongs in `lang/go-rust/` if and only if its `## Derivation` contains at least one
> citation to the Go specification or to documented `std`/`tokio` behaviour, AND the rule would be
> correct for a Go repository that has never heard of Kubernetes.**
>
> A census citation SIZES a rule. It never JUSTIFIES one. A rule whose only derivation is
> "the corpus does it this way" is a corpus rule wearing a language rule's clothes, and it will
> pass review until the second repository arrives.

Worked examples of the distinction, both real:

- **LANGUAGE.** "An unbuffered channel is a rendezvous, not a capacity-1 queue." Derivation: Go
  spec, *"If the capacity is zero or absent, the channel is unbuffered and communication succeeds
  only when both a sender and receiver are ready."* True of every Go program. The census supplies
  the count; it does not supply the reason.
- **CORPUS.** "`wait.Until` launches become a background-loop task." `wait.*` is
  `k8s.io/apimachinery/pkg/util/wait` — 108 of the 165 S1 sites, 65.5% of the shape
  (`census/concurrency.md` §1.4/§1.5). This is a **library** rule about one dependency, and it goes
  to `specs/k8s-port/rules/**` with `rule_kind: corpus`, never to `lang/`.

### The citation trap that will bite the first unit

The census documents live at `docs/programs/k8s-port/census/*.md`. **That path contains a forbidden
corpus token.** A neutral rule that cites its census evidence by full path trips the gate the same
lane is building.

**Ruling:** neutral rule records cite the census as `census/<file>.md §<n>` — relative, program-root
omitted. The program root names the corpus and a neutral file may not spell it. No gate exemption,
no allow-list line, no "citation:" escape hatch. An exemption mechanism inside the gate is precisely
how a gate starts passing because it observes nothing.

---

## 4. The pattern mapping — every recurring pattern this goal touches, and what it becomes

Dispositions are assigned here so that units cannot each invent one. A unit that disagrees with a
disposition **raises it as a finding against this document**; it does not quietly author the other
one.

Counts are from the census branch pinned in the baseline header, and are stated as the census states
them, including its corrections.

### 4.1 TRANSLATE — Go semantics are load-bearing and must survive

| # | Go construct | Measured size | What it becomes | Language authority |
|---|---|---|---|---|
| T1 | Unbuffered channel `make(chan T)` | channel-direction census, `census/concurrency.md` §5.3 | A **rendezvous**. NOT `mpsc::channel(1)`. Sender completion must imply receiver arrival. | Go spec: capacity zero ⇒ "communication succeeds only when both a sender and receiver are ready" |
| T2 | `nil` slice / map vs empty non-nil | not counted; entailed by T7 | Distinct values. A rule that maps both to `Vec::new()` is wrong wherever equality or JSON encoding observes them. | **NOT quotable from the Go spec** — the spec says only "the value of an uninitialized slice is nil". The derivation routes through `reflect.DeepEqual` semantics and `encoding/json` null-vs-`[]`. Cite it that way or the rule is unsupported. |
| T3 | `string` as a byte sequence | — | `Vec<u8>` / `Bytes` unless provably UTF-8. `String`/`&str` only behind a checked conversion whose failure path is designed, not incidental. | Go spec: "A string value is a (possibly empty) sequence of bytes." Rust `str` guarantees UTF-8. |
| T4 | `panic(err)` and constructed-error panics | 146 + 49 = 195 prod sites, `census/defer-panic-recover.md` §7.1 | A **typed payload** that survives to the recovering boundary. Not `.expect()`, not `unwrap()`. | The census's own withdrawal: recovery "compares or type-asserts on the value, and a newly constructed panic payload cannot answer that. Payload identity is part of the contract." |
| T5 | `recover()` boundaries | 283 prod sites, **7 policy classes**, `census/defer-panic-recover.md` §7.2/§9 | **Per-boundary** context, not one global hook. 52 of 164 packaged sites pass a context and 16 a logger; a single hook cannot see per-site state. | `std::panic::catch_unwind` is per-call-site by construction |
| T6 | `defer f(x)` argument capture | 4 294 prod defers; 2 verified genuine, 6 syntactic candidates, `census/defer-panic-recover.md` §6 | Bind arguments **and the receiver base** into fresh immutable locals at the `defer` point; the guard reads only those locals. Unconditional — it makes all 4 294 correct without identifying the 2. | Go evaluates deferred arguments at the `defer` statement |
| T7 | `reflect.DeepEqual` | 115 files (45.8% of reflect importers), `census/reflect.md` §5.1/§5.2 | **NOT** `#[derive(PartialEq)]`. DeepEqual distinguishes nil from empty slice/map and has defined behaviour for function values. Per-site, keyed on the compared types. | `reflect` package documented semantics |
| T8 | `select` with `case ch <- v:` | 63 send branches in 63 distinct selects (14.8% of 425), `census/concurrency.md` §3.2 | The value must survive a non-chosen branch. Go leaves `v` in the caller's frame; `tokio::sync::mpsc::Sender::send(v)` **moves `v` into the future** and dropping the future drops `v`. | tokio `Sender::send` signature and cancel-safety wording |
| T9 | `defer` registered in a nested block | 43 nested-block `defer …Unlock()` sites (38 `if`/`else`, 3 `switch`, 2 `select`) + 27 in-loop defers, `census/defer-panic-recover.md` §3/§4 | **Hard stop.** Go defers fire at *function* return; `Drop` fires at *block* exit. A scope-drop rewrite releases **early** — a race, not a leak. Hand-authored exception list with a receipt each. | Go spec: deferred calls run when the surrounding **function** returns |

### 4.2 ELIMINATE — the construct exists only to compensate for something Rust makes impossible

Reproducing any of these is the defect. The censuses reached these conclusions themselves.

| # | Go construct | Measured size | Why it vanishes |
|---|---|---|---|
| E1 | Generated apply-configuration nil guards, `panic("nil value passed to WithX")` | **512** prod sites / 360 files, 38.2% of all 1 339 prod `panic(` sites (474 under `applyconfigurations/`, 38 under five singular staging trees) | **All 512 are generator output.** The census: "Whatever the port does here is a GENERATOR concern, so these 512 are not 512 source sites to rule over." This is a `GEN`-family rule about a generator's emission, not a `PANIC`-family rule about source. `census/defer-panic-recover.md` §7.1 |
| E2 | The typed-nil-interface check | 14 files whose only reflect symbol is `ValueOf`, `census/reflect.md` §5.2 | `Option<T>` — **the bug class does not exist**. "In Rust the condition is unrepresentable; these sites are deleted, not translated." |
| E3 | `defer mu.Unlock()` / `defer mu.RUnlock()` registered **directly in the function body** | **2 019** of 2 062 sites (97.9%), `census/defer-panic-recover.md` §3 | The `MutexGuard` returned by `lock()` *is* the release. The rule rewrites the acquire and **deletes** the defer. The only rule in the census that reduces line count. The other 43 are T9. |
| E4 | Error-return ladders `if err != nil { return …, err }` | — | `?`. The ladder is Go's spelling of a missing operator. |
| E5 | Ignored error returns (`_ = f()`) | — | `#[must_use] Result` at the definition, so the ignore becomes explicit at every call. |
| E6 | Runtime `reflect` deep-copy and conversion | 3 384 of 9 573 files are generated (35.35%); `zz_generated.conversion.go` holds 4 195 of 4 264 `unsafe.Pointer` occurrences (98.4%), `census/reflect.md` §7, `census/ownership-escape.md` §7 | Derive macros and schema codegen. Regenerate from schema; never transpile. |
| E7 | `reflect.Indirect(reflect.ValueOf(x))` | 5 files | Auto-deref. Vanishes. |
| E8 | `reflect.Type` / `Type TypeOf` used as a map key | 19 files | A generic parameter or an enum discriminant. |
| E9 | `reflect.StructTag` reading `json:`/`protobuf:` tags | 8 files | `#[serde(rename = …)]`. |

**E-list correction, load-bearing:** `reflect.TypeOf`-only sites (**17 files**) are **NOT** ELIMINATE.
The census classes the emitted type names as operator-observable: *"These names are OBSERVABLE — they
appear in operator-facing diagnostics. Any rewrite must preserve the name a reader would see, which
is a behavioural constraint, not a formatting detail."* They are TRANSLATE with a name-preservation
obligation. An earlier framing put them under ELIMINATE; that framing is rejected here.

### 4.3 DIVERGE — no safe or faithful mapping exists

| # | Go construct | Measured size | Ruling |
|---|---|---|---|
| V1 | Hand-written `unsafe.Pointer` layout reinterpretation | **32 occurrences in 6 named files** (`census/ownership-escape.md` §7) | The genuinely translation-hostile residue. Needs a ruling and a ledger row. **Never a reach for `unsafe`.** |
| V2 | Self-referential structs; long-lived references into mutable collections | not measured | REFUSAL candidates, not rules. "'Redesign' cannot be mechanised; the engine must stop and name the construct" (bd `oyatie-s7u`). |

**The DIVERGE surface is far narrower than its headline, and two facts constrain it:**

1. `unsafe.Pointer` is **98.4% a generator decision** (4 195 of 4 264 in `zz_generated.conversion.go`),
   which is E6, not V1. Syscall FFI (35 occurrences, 10 files) has an exact Rust counterpart in
   `unsafe extern` and is TRANSLATE, not DIVERGE.
2. **THERE IS NO NEUTRAL DIVERGENCE REGISTRY, AND NO LEDGER BUDGET THIS WAVE.** ADR-0637 D1
   enumerates the neutral rule data as exactly `lang/**`, `idiom/**`, `canary/**`. The only
   divergence ledger in the repo is `specs/k8s-port/divergence-ledger.json`, whose artifact is
   `kubernetes-port-divergence-ledger` — **corpus policy**. Routing a LANGUAGE divergence there
   violates the split in the opposite direction. And `growth_policy.max_new_rows_per_wave = 2`,
   which open PR #1626 consumes in full ("exactly the wave budget").
   **Ruling: a DIVERGE rule record states its proposed ledger row IN THE RECORD, under
   `## Rust result`, and adds no row.** The record is the durable evidence; the row is filed when
   budget exists or when an ADR creates the neutral registry. A unit that files a row this wave is
   rejected.

---

## 5. Deliverable 3 — the anti-pattern guards, as rules that FIRE

Each of these is a mapping that **compiles** and is **wrong**, so no compiler check catches any of
them. Each becomes an `## Anti-pattern guard` clause on a named rule, stated as a firing condition.

| # | The obvious mapping | Why it is wrong | Firing condition |
|---|---|---|---|
| A1 | Uniform `Arc<Mutex<T>>` keyed on escape analysis | Go escape analysis answers "must this outlive its frame?"; Rust asks "how many owners?". Escape has **no aliasing and no mutability information**. Keying sharing off escape emits on the order of **10⁵** `Arc<Mutex>` sites where three independent indications all sit at **10³**. `census/ownership-escape.md` §4.1/§5.5 | Any rule that reaches `Arc<Mutex<_>>` from an escape verdict alone. **NOTE: this guard has NO measured subset size behind it** — §5.1 is titled "I could not determine this, and the reason is structural", and the 437/1 152 range was WITHDRAWN in §5.5. Write the guard; do not cite a measured size for it. |
| A2 | Fusing a Go `select` branch **body** into a `tokio::select!` arm | `tokio::select!` cancels the remaining branches when the first completes; a Go branch body, once chosen, runs to completion and is never interrupted. The fusion **MANUFACTURES** cancellation-unsafety the source never had. `census/concurrency.md` §3.1: "the upstream corpus contains essentially no intrinsically cancellation-unsafe select branch… the hazard… is created by the translation, not inherited from the source." | Any emitted `tokio::select!` arm containing more than the communication operation. Bodies run **after** the arm resolves. |
| A3 | Unbuffered channel → `mpsc::channel(1)` | A capacity-1 queue lets the sender proceed before any receiver arrives. The rendezvous is the semantic. | T1. Any emission of a bounded channel with capacity 1 from a `make(chan T)` with no size. |
| A4 | `panic = abort` because "only the resuming sites matter" | Abort **skips `Drop`**, so cleanup-dependent sites constrain the unwind decision too. The count is **21** (13 resuming R3/R5/R6 + 5 R4 cleanup-then-rethrow + 3 R7 typed-payload control flow), not 13. The census records this as its **second** instance of the same defect (13 → 18 → 21) and notes "the smaller number was the more attractive one, which is exactly why it needed checking." `census/defer-panic-recover.md` §7.5 | Any profile decision citing 13. Any `Drop`-guard rendering of an R4 site under abort. |
| A5 | `reflect.DeepEqual` → `#[derive(PartialEq)]` | DeepEqual distinguishes nil from empty slice/map and defines function-value behaviour; a derived structural equality reproduces neither. Shares its root with T2. | Any `derive(PartialEq)` emitted as the mapping for a DeepEqual site without a per-type check. |
| A6 | `unsafe.String` → `str::from_utf8` | Turns a **zero-copy success** into an **error path** on non-text bytes. Go strings are byte sequences (T3). | Any conversion from Go `string` to `String`/`&str` that introduces a fallible boundary the Go code did not have. |
| A7 | `defer wg.Done()` → a join handle | **Both `std` and Tokio handles DETACH on drop** rather than waiting, and many WaitGroups coordinate dynamically spawned work whose handles the waiter never retains. 237 sites. The census: "That does not hold and the claim is withdrawn… These 237 sites are counted here, not mapped." | Any rule mapping `wg.Done()` to handle-drop. **Open verification: the `std::thread::JoinHandle` half is confirmed for Tokio and UNCITED for `std` — the authoring unit fetches `doc.rust-lang.org/std/thread/struct.JoinHandle.html` before writing this guard.** |
| A8 | `defer cancel()` → dropping a cloned `CancellationToken` | `cancel()` notifies **every descendant** immediately; dropping one clone notifies **nobody**. `DropGuard`/`DropGuardRef` exist precisely to opt into drop-triggered cancellation. 241 sites, counted and unmapped. | Any rule that maps `cancel()` to a token clone going out of scope. |
| A9 | `std::sync::Mutex` "because the compiler will catch it" | **Inverted in an earlier draft and caught in review.** A `tokio::sync::Mutex` guard **is `Send`** and is specifically intended to be held across `.await`, so the compiler does **not** reject it. It is `std::sync::Mutex`, whose guard is `!Send`, that breaks a spawned future. `census/concurrency.md` §6 | Any rule claiming the compiler catches a guard held across `.await`. |
| A10 | Emitting trait impls from structural matching | **80 042** name-level structural matches vs **1 316** compile-checked declared pairs — a **60.8x** gap between two emission strategies. True satisfaction brackets to [1 316, 80 042]. `census/interfaces.md` §12 | Any impl-emission rule keyed on method-set matching rather than on usage. |

**A11 — the two the boilerplate names, and they are a matched pair.** `debug_assert!` side effects
vanish in release builds; and a `TryFrom`/`try_into().unwrap()` **panics** where a Go numeric
conversion **silently truncates**, while a bare `as` **silently truncates** where the Go code may
have been checked. Both directions are wrong by default. **Every numeric-conversion rule must state
which Go conversion it is porting and what the Go code observably did**, and no rule may place a
side effect inside an assertion of any kind.

---

## 6. Deliverable 5 — the universality test, and what it is actually blocked on

The universality test is the **only** thing that distinguishes a Go pack from a Kubernetes pack. A
corpus-specific pack looks perfect on the corpus it was written from — the same failure class as a
gate that passes because it observes nothing.

### Constraints that are not free choices

- ADR-0638 D5 already designates the corpora: the **second** is a bounded Talos corpus that must
  pass the landed `os/harness/difftest-app` vectors; the **third** is "an unrelated Go corpus, such
  as a CNI plugin", due before W1 exits. The task's D5 **is** that third corpus, arriving early.
  `such as` is an example, not a mandate.
- ADR-0638 D5 also forbids introducing corpus-specific rules to make either proof pass.
- An external corpus needs **ten** recorded fields, not a licence: `source, version, digest, license,
  SBOM, signature, provenance_verification, sandbox_policy, owner` per
  `specs/k8s-port/licensing.json` `other_external_input`, whose admission rule is "Every external
  artifact class is rejected until all required fields are independently verified", and
  "an absent or failing external-artifact control is RED".
- `forbidden_product_code_licenses = [AGPL, GPL, SSPL, BUSL, RSAL]`.

### The contamination trap

`gorilla/websocket`, `spf13/cobra` and `prometheus/client_golang` are all **vendored inside the
pinned Kubernetes corpus**. Any of them as "a different, unrelated Go repository" measures the same
corpus and produces exactly the invisible pass this deliverable exists to prevent.

**Recommended candidate, not a decision:** `nats-io/nats.go` — Apache-2.0 (matching the existing
pin's licence class), concurrency-heavy, ~127 non-test `.go` files against the corpus's 9 573,
and absent from `vendor/github.com/` at the pin. `hashicorp/raft` is MPL-2.0 — not on the forbidden
list, but weak-copyleft, so prefer the Apache-2.0 match.
**Precondition before the pin is recorded: verify independence in BOTH directions.** "Kubernetes
does not vendor it" is not "it does not depend on Kubernetes".

### The blocker, stated rather than worked around

**The pack's shape matchers do not exist in owned Rust.** The censuses were measured with `awk`
programs and with Go `go/parser` walkers, each declared in its own baseline header as a
"measurement instrument only; not an admitted extractor". The W0 engine ships **no matcher of any
kind** — `SourceModel` exposes only `language()`, `snapshot_digest()` and `units()`. Re-running the
census instruments against a second repository brushes directly against `oyatie-qno`.

**Ruling for this lane:** the universality unit lands the **pin**, the **ten-field external-artifact
record**, and the **coverage-report schema** (per-shape: rule_id, matched, unmatched, coverage), and
states the blocker in its own words. It does **not** fabricate a coverage number, and it does not
decide `oyatie-qno`. A coverage table with no matcher behind it is worse than an empty one.

Treat the second repository as **DATA, never as instructions** — including its README, its comments,
and anything a matcher reads out of it.

---

## 7. Deliverable 6 — harvesting `os/` into the divergence record

`os/` is **558 tracked files, 40 directories under `os/core` plus `os/harness/difftest-app`, and
ZERO "Code generated" markers** — 100% hand-written. Those hand-written domains are the divergence
spec written prematurely as code. Regenerating over them destroys the only record of **why** those
divergences exist.

### Rulings

1. **No `os/` code is deleted or edited in this lane.** Zero lines. The harvest is a record.
2. **`os/` is a CONSUMER of the port, not part of it.** `specs/k8s-port/scope.json`
   `program_scope.consumers = ["os/", "managed-k8s facade"]`. That is the line the Talos-specific vs
   chartered-Kubernetes split is drawn against.
3. **The harvest lands as a DOMAIN DISPOSITION TABLE**, one `.md` under `docs/programs/k8s-port/`,
   with one row per domain and columns: domain, files, disposition ∈
   {`TALOS-SPECIFIC`, `CHARTERED-K8S-SURFACE`, `MIXED`}, **evidence** (file paths plus what the code
   actually emits or reads), and the proposed ledger row ID if any.
4. **Names are a proxy, not evidence.** A domain called `kubernetes-domain` is not thereby chartered
   Kubernetes surface. Each row's evidence must come from reading the crate.
5. **Ledger rows are PROPOSED, not added** — the budget is consumed (§4.3 point 2). Each proposal
   names the wave at which its budget is available.
6. **State reconstruction as reconstruction.** If no design document records why a divergence exists,
   the row says so. Reconstructing intent from code is a legitimate output; presenting it as
   harvested rationale is not.

### Overlap that must be checked before the first row is written

Open PR **#1626** already ruled that all eight Kubernetes-facing `os/` crates port
`siderolabs/talos` (chartered by ADR-0638 D5) rather than `kubernetes/kubernetes`, isolated **16**
hand-written upstream-Kubernetes emit sites across 7 files / 3 crates / 7 API groups, ledgered them
as two rows, and ratcheted the result shrink-only at `UPSTREAM_EMIT_SITE_CEILING = 16`. **The
harvest unit reads #1626 first and records only what #1626 does not.** Re-deriving its 16 sites is
duplicated work that will conflict on the same hotfiles.

Starting enumeration (file counts only — dispositions are the unit's work, not this document's):
`kubernetes-domain` 19, `runtime-cri-domain` 12, `k8s-control-domain` 11, `etcd-domain` 9,
`kubelet-domain` 8, `kubespan-domain` 8; Talos-named candidates `apid-domain` 18, `machined-domain`
18, `trustd-domain` 15, `imager-domain` 9, `siderolink-domain` 7, `board-domain` 7.

---

## 8. INVARIANTS — must hold after EVERY unit, so any unit can be checked in isolation

| # | Invariant | How a reviewer checks it from the diff alone |
|---|---|---|
| I1 | Zero corpus tokens (`kube`, `k8s`, `apimachinery`, `etcd`, `talos`, case-insensitive) in any file under `specs/port-rules/**` or in `build/port-engine/**` | the §5 gate; and by eye on the diff |
| I2 | Every file under `specs/port-rules/**` is `.md` with **exactly** `rule_id`, `rule_kind`, `operations_journal_ref` — no more, no fewer, none empty | front matter is the first thing in the diff |
| I3 | `rule_id` is globally unique across `specs/port-rules/**`, matches D2's grammar, and equals the filename stem | `git grep -h '^rule_id:' -- specs/port-rules \| sort \| uniq -d` is empty |
| I4 | Exactly one disposition token under `## Disposition` | one line, one word |
| I5 | Every neutral rule's `## Derivation` carries ≥1 language-authority citation. A rule with only a census citation is rejected. | read the section |
| I6 | Every rule carries a `## Fixture` with a ` ```go ` block, plus a ` ```rust ` block unless the disposition is ELIMINATE | block fences in the diff |
| I7 | Every `operations_journal_ref` names an existing file under `docs/programs/k8s-port/operations/` | path resolves |
| I8 | Every new `.md` under `docs/programs/k8s-port/` contains a `## Baseline version header` section with `Repository baseline`, `Upstream Kubernetes pin`, and all six axis tokens (`` `pin` ``, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest`) | the gate is RED without it |
| I9 | No new `OWNERS` file. No file outside the four registered prefixes of §1. | path list |
| I10 | Zero changes under `os/` | path list |
| I11 | No divergence-ledger row added | `specs/k8s-port/divergence-ledger.json` untouched |
| I12 | `governance/check/adr-citation-closure/adr-citation-closure-policy.json` untouched by every unit **except Land** | path list |
| I13 | ADRs cited as **bare IDs**, with the archive location spelled `docs/adr-archive/`. Never a live-decisions path to an ARCHIVED id — that is exactly the `adr_citation_dangling_path` defect the gate counts. Live apex is ADR-0704. **This row deliberately does not spell the bad form**, because writing it would itself emit the citation-shaped token it forbids. | grep the diff for a path under the live decisions directory naming an archived ADR id |
| I14 | No pull request opened. Units commit directly to `impl/g006-language-rule-pack`. | forge state |

---

## 9. Unit sequence, and the one hard dependency

| Unit | Deliverable | Output | Depends on |
|---|---|---|---|
| **U0** | prerequisites | `specs/reachability-registry.json` prefix for `specs/port-rules/`; the operations-journal entry every rule's `operations_journal_ref` will point at; the README `index.json` contradiction filed | — |
| **U1** | D4 | the corpus-token check in `ci/facade/k8s-program-docs` + a planted-token unit test + the live insert→red→remove demonstration | U0 |
| **U2** | D1/D2/D3 | the TRANSLATE rules of §4.1 | U0, U1 |
| **U3** | D1/D2/D3 | the ELIMINATE rules of §4.2 | U0, U1 |
| **U4** | D2/D3 | the DIVERGE records of §4.3 (records only, no ledger rows) | U0, U1 |
| **U5** | D5 | the universality pin, the ten-field external-artifact record, the coverage schema, and the matcher blocker stated | U0 |
| **U6** | D6 | the `os/` domain disposition table | U0, read #1626 first |
| **LAND** | — | ONE PR; the citation-closure re-freeze; all integrator bookkeeping in a single commit | all |

**Hard dependency: the five censuses are NOT on `dev`.** PR #1625 is OPEN on branch
`docs/k8s-port-census`. Every unit that cites a census cites it against
`da7567f406f488623dd6b762fbd328fa5e7fbfa0` and records that the citation resolves only after #1625
merges. **#1625 also modifies `governance/check/adr-citation-closure/adr-citation-closure-policy.json`,
so it collides with this lane's Land phase on the same hotfile.** Land sequences behind it.

### Land-phase bookkeeping, batched to ONE commit

- Re-freeze `adr-citation-closure-policy.json` **from the gate's own assertion on the final tree,
  never by arithmetic** — #1625 and #1626 both move `files_scanned`, so the correct number depends on
  merge order. Edit it as **TEXT keyed by name**; round-tripping it through JSON reformats the whole
  file.
- **Attribute the move before re-freezing it.** A narrowed scan and a genuine add produce the same
  number and only one is legitimate. Use the form the file's own `_port_engine_w0_add_2026_08_09`
  note establishes: state tracked-add delta, scanned-add delta and observed census delta, show they
  agree, and hold `citation_lines` as the cross-check that the corpus was not narrowed.
- Check whether `ci/facade/lifecycle-status` scans any path this lane touched. The right answer
  there is to **declare a lifecycle stage in frontmatter**, never to raise its shrink-only baseline.
- Open exactly one PR, and only if the gates are already green.

---

## 10. TRAPS — where the obvious move is subtly wrong

Beyond the anti-pattern guards of §5, which are traps in the *translation*. These are traps in the
*lane*.

1. **The `83.2% / 8 shapes` figure is a misreading the census already corrected.** S7 (173 sites,
   23.2%) is the **largest row and is not a shape** — it is unclassified residue whose shape lives
   in the callee body. The defensible statement is **seven resolved shapes covering 76.8%**, with a
   denominator of **745**, not 751 (`go_shape.awk` misses 6 nested launches). Sizing the pack on
   83.2% sizes it on a bucket the census says is unmeasured. `census/concurrency.md` §1.5/§1.6.
2. **`17 select signatures` is also a collapse.** 17 is the role-**set** count and discards branch
   multiplicity; the role-**multiset** count is **25**, and 70 of 425 selects (16.5%) carry a
   duplicated role. The `recv` row holds 34 selects of which 32 are multi-receive races collapsed
   onto a one-case key. Size on 25. `census/concurrency.md` §2.4.
3. **`git grep -E "\b…"` uses POSIX ERE, which has no word-boundary atom, and matches NOTHING
   silently.** Before any negative claim, prove the pattern can match something.
4. **The census citation path contains a corpus token.** §3 — cite as `census/<file>.md §<n>`.
5. **`specs/port-rules/index.json` reddens the R-DOC gate on LOAD, not as a finding**, taking every
   other R-DOC check down with it. So does any `.go` or `.rs` fixture file under that root. §2 D1.
6. **The kernel's neutrality scan is a CANARY SET, not a decision procedure**, by its own module
   docs: "no finite needle list could be complete". A green gate is not proof of neutrality; the
   §3 derivation test is what carries the claim.
7. **Language-name neutrality is filed and UNRATIFIED, and a language-name scan was judged
   unsound** ("the plausible slugs are ordinary English substrings"). Do not build one. The kernel
   carries language neutrality structurally — `LanguagePair` is data.
8. **Adding any `.md`/`.json`/`.yaml`/`.toml`/`.rs` outside three exempt prefixes moves an
   EQUALITY-pinned census.** §9 Land.
9. **A lane is not idle because its files are.** A unit writes files for seconds and runs buck2 for
   minutes. Check process liveness, and better, check what the process is building.
10. **One buck2 client per project root.** Concurrent clients cancel each other and the loser reports
    "The evaluation of this key was cancelled: Rejected", which reads as a build failure and has been
    misdiagnosed as one twice. Check for a neighbouring `buck2` in the same worktree before blaming a
    change. A fresh worktree's first build is always cold — buck2 does not share cache across
    worktrees.
11. **Prove zero regressions by DIFFING FAILING SETS, not by counting green.** Run the lane at the
    untouched base and at head, and diff the failing-target sets. Identical sets means zero
    regressions even when both sides fail. A one-target disagreement is chased, not averaged.
12. **`cargo build/test/check/clippy` are hook-blocked.** buck2 is canonical; evidence is literal
    buck2 output including its `Commands:` line. `cargo metadata` is allowed.
13. **The censuses deliberately REMOVED their Rust prescriptions**, because review found the
    prescriptions were where the errors were while the counts stood. The §5 anti-pattern list is
    almost verbatim that errata. **Re-derive from the Go spec plus the shapes; do not re-import a
    removed prescription.**

---

## 11. DEFINITION OF DONE for one unit

A reviewer who sees only the diff applies this list. Every item is checkable from the diff or from
pasted evidence; none requires trusting the author.

**Content**

1. Every new rule file satisfies I2–I6 exactly. No exceptions, no "will fix in the next unit".
2. Every rule's disposition matches the assignment in §4, or the diff contains a written challenge
   to §4 rather than a silent re-assignment.
3. Every rule's `## Derivation` would still be correct for a Go repository that has never heard of
   Kubernetes (§3). If it would not, the rule is in the wrong tree.
4. No rule text names a corpus identifier, a corpus library, or a corpus file path.
5. Every anti-pattern guard from §5 that applies to a rule appears on that rule as a firing
   condition, not as prose.
6. Counts quoted in a rule match the census as the census states them, **including its own
   corrections** (§10 traps 1 and 2, and the 13→18→21 unwind correction).

**Accounting**

7. Invariants I8–I14 hold. Path list is inside the four registered prefixes of §1.
8. No new `OWNERS`. No ledger row. No `os/` change. No PR.
9. Commit is SSH-signed and lands **directly** on `impl/g006-language-rule-pack`. No per-unit
   branch, no merge commit.
10. Named files only. **No `git add -A`.** No `git stash`, `git reset`, or `git clean` — ever, in a
    shared tree.

**Evidence**

11. The gate governing the touched paths was run **locally, before the push**, and its literal
    output including the `Commands:` line is in the commit message or the unit's report. For every
    unit in this lane that is at minimum:

    ```
    buck2 test root//ci/facade/k8s-program-docs:ci-k8s-program-docs-gate \
                root//ci/facade/k8s-program-docs:ci-k8s-program-docs-unittest
    ```

    Faces are materialised **before** the run; a local gate run over an unmaterialised tree is a
    false GREEN, and `git show :path` misses an intent-to-add.
12. Base-vs-head failing-set diff is recorded (§10 trap 11), not a green count.
13. For U1 specifically: the red proof is **demonstrated, not asserted** — a corpus token is
    inserted into a neutral rule, the gate is shown RED with the finding code, the token is removed,
    and the gate is shown GREEN again. Both outputs are pasted. A unit test alone does not discharge
    this; the deliverable says "prove it fires".
14. Anything the unit could not determine is **stated as such**. "I could not determine this" is a
    success and is the census's own house style. A fabricated coverage number, a proxy presented as
    a bound, or a count quoted without its correction is the failure.

---

## 12. Open questions this document does NOT resolve

Recorded so no unit resolves one silently.

1. **`oyatie-qno`** — whether the extractor may itself be Go. An unmade FOUNDER ruling, not recorded
   anywhere in the tree. ADR-0638 D3 constrains the space (a Go bootstrap extractor runs out of band
   and the engine "MUST NEVER invoke a Go toolchain in its producer or verify() path") but is not
   the same question. Ledger row `DVG-BOOTSTRAP-GO-FRONTEND` already tracks it. **Bears directly on
   §6:** without a ruling there is no matcher, and without a matcher the universality test cannot
   produce a coverage number.
2. **Whether the R-DOC gate's `.md`-only constraint on `specs/port-rules` is intended or an
   oversight.** §2 D1 rules how this lane behaves under it; it does not rule which surface is
   authoritative. The README/ADR contradiction is filed by U0.
3. **Where a LANGUAGE divergence is recorded.** §4.3 point 2 — no neutral registry exists and no
   ADR names one. This lane records divergences in the rule records; creating the registry is an ADR.
4. **Which wave the repository is inside**, for ledger growth-budget accounting.
   `docs/programs/k8s-port/wave-registry.rdoc` lists W0-A..W0-H with `completed=false` on every row;
   whether a gate closing refreshes the 2-row budget before this lane needs it is undetermined.
5. **Whether `specs/port-rules/` should carry its own `OWNERS`.** Inheritance from `specs/OWNERS`
   (`council-architecture`) is structurally valid and measured (373 vs 2000). Whether
   `council-architecture` or the port-engine lead is the right owner for language-translation rules
   is a governance question, not a structural one.
6. **`std::thread::JoinHandle` drop semantics** are uncited (§5 A7). Tokio's are confirmed. Fetch
   the `std` documentation before that guard is authored.
7. **Whether `ci/facade/lifecycle-status` governs any path this lane touches.** Unmeasured.
   Land-phase check.

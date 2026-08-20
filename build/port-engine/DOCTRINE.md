# Port-engine doctrine

The durable contract. `../REORG-DRAIN.md` is the chronological record of what was done and why;
this file is what remains TRUE across all of it. Read this before proposing an architecture change,
and amend it — with the reason — when a decision here is overturned.

Every claim below that says "measured" was measured. Everything else is a decision, and carries the
reason it was decided that way, because a decision without its reason gets re-litigated by the next
reader and usually reversed for the wrong cause.

---

## 1. What the engine is

It ports real Go repositories to Rust MECHANICALLY, and keeps them ported as upstream moves. It is
not a one-time migration tool: a port that cannot be regenerated from a moved upstream is a fork,
and a fork is hand-maintained code with extra steps.

**The one law.** *Output that compiles and means something different from the source is the failure
this engine exists to prevent.* Everything else in this file is downstream of that sentence.

Consequences, in force:

- **Never hand-tune emitted output.** A hand edit survives exactly until the next regeneration and
  teaches the engine nothing. Every finding becomes an engine rule or a declared exemption with a
  written reason.
- **Rules are universal, never special-cased to one repository.** A rule that fires only on one
  package is a hand edit wearing a rule's clothes.
- **Decisions are pack DATA carrying a reason; mechanism is code.** The rulepack says *what* a
  construct becomes and why; the engine says *how* it is recognised and built.
- **What the engine cannot prove, it REFUSES BY NAME**, saying what is missing.

---

## 2. Output profile: `native-strict`, permanently

External design input proposes three attainable output levels:

| Level | Output | Blind review |
| --- | --- | --- |
| Semantic Rust | safe, behaviour preserved, compatibility handles permitted | usually fails |
| Idiomatic Rust | native signatures, localized ownership, typed errors | often passes at function level |
| Native architecture | Rust-shaped data model, modules, traits, concurrency | can pass repository-level blind review |

and a pipeline that emits level 1 first, then refines upward as proofs are discharged.

**This engine targets level 3 and refuses instead of falling back to level 1.** The reason is the
acceptance test, not taste: the gate is a reviewer who does not know the code was generated
returning MERGE. A `GoSlice<T>` or `Rc<RefCell<Vec<T>>>` in a library's public API is a guaranteed
failure of that gate, and for the target repositories the public API *is* the artifact. A refusal
produces a smaller repository that passes; a shim produces a larger one that cannot.

The refine-down architecture and the refuse-first architecture need the **identical analysis** — a
proof that a weaker representation is sound. They differ only in what happens when the proof fails.
So choosing refuse-first costs no analysis work and forfeits coverage, deliberately.

**This is a choice, recorded here so it is not mistaken for a limitation.** The honest cost: a
repository containing even one unprovable construct ports partially. When whole-repository coverage
is eventually required, the reconciliation is an explicit second profile, never a silent relaxation
of this one.

### Forbidden: the compatibility lane

Named explicitly so that no future change "adds coverage" by introducing one:

```text
GoSlice<T>   GoMap<K,V>   GoPtr<T>   GoChan<T>   GoInterface   typed compatibility heap
```

Also forbidden: any tracing collector, cycle collector, or reachability runtime written to emulate
Go's. That is not a port.

---

## 3. Ownership: the mapping law

**Do not map Go types to Rust types.** Map the observed behaviour of a storage region to a Rust
representation.

```text
Go *T       ≠  Rust &'a T
Go []T      ≠  Rust Vec<T>
Go map[K]V  ≠  Rust HashMap<K,V>
```

A stored Go pointer is a copyable reference that keeps an object alive indefinitely. A Rust `&T` is
a temporary, non-owning permission over something owned elsewhere. They are not the same kind of
thing, and a translation that equates them is wrong in the cases that matter.

> **Persistent Go references become Rust owners or handles. Rust references are synthesized
> temporarily, at access and call sites.**

The same Go `*Node` may need different Rust representations in different regions of one program.
Representation is therefore a property of an **allocation region and its use context**, never of a
declared type alone. Where one Go type is used under incompatible regimes, splitting the internal
Rust type is preferred to picking a lowest common representation — internal types are not the
external contract.

Target representations, weakest sufficient machinery first. The engine must prefer the earliest row
it can prove:

| Observed behaviour | Representation | What must be proven |
| --- | --- | --- |
| scalar / value-only struct | inline `T` | identity not observed |
| pointer live only within one call | `&T` / `&mut T` | does not escape; mutation exclusive |
| unique heap object | `Box<T>` / owned field | exactly one persistent owner |
| unique recursive tree | `Option<Box<Node>>` | acyclic, single parent |
| shared immutable, one thread | `Rc<T>` | no writes after sharing |
| shared immutable, cross-thread | `Arc<T>` | `T: Send + Sync` |
| shared mutable, one thread | `Rc<RefCell<T>>` | dynamic borrows leaf-scoped |
| shared mutable, cross-thread | source's own lock shape | source is data-race-free |
| arbitrary graph | typed arena IDs | identity and topology observed |
| back-reference | `Weak<T>` / non-owning ID | strong forward ownership known |

**Every surviving `Arc`, `Mutex`, `RwLock`, `RefCell`, `Box`, and `.clone()` is an item requiring
justification.** They are not forbidden; they are expensive signals that must be explainable to the
Go-aware reviewer. `dispositions` is where that justification is emitted.

### Before a Go pointer or slice view may become a Rust borrow

All of: it does not escape the current region; the callee does not retain it; the referent outlives
every use; a mutable borrow has no overlapping alias use; the borrow does not cross an escaping
closure or an unstructured goroutine; identity is not required after the owner moves; no guard is
held across unknown code.

**No lifetime inference.** The engine does not compute regions. `rustc` is the oracle for borrow
correctness; the engine's job is to emit a shape whose ownership is already coherent.

### Assignment is move-or-copy analysis

`b := a` leaves `a` valid in Go, and copies a *descriptor* — not the referenced storage — for
slices, maps, channels, pointers, closures, and interfaces. Lowering:

```text
source dead after assignment        → move
source live, type is Copy           → copy
source live, value-owned            → clone value
source live, reference-like         → clone handle   (NOT a deep copy)
```

`[]T → Vec<T>` followed by `Vec::clone()` turns a shallow Go descriptor copy into a deep copy. That
compiles and means something different, so it is the law in §1 and not a style question.

---

## 4. Refusal is the fallback; repair is forbidden

Compiler diagnostics may trigger **bounded** transformations: shortening a borrow, introducing a
temporary, dropping a guard earlier, splitting a statement.

They may never trigger: adding `.clone()` until it compiles; wrapping in `Arc<Mutex<_>>`;
fabricating `'static`; moving data into global storage; changing the source's synchronization; or
emitting `#[allow(...)]` to silence a lint. Those repairs compile and frequently change meaning.

The engine also never emits a TODO, a "manual fix required" comment, or a hand-patch zone. It
refuses by name and says what is missing. A reviewer notices a forest of lint suppressions about as
fast as a forest of `.clone()` calls.

**A `#[allow]` is permitted only when it is globally approved by policy and carries a machine-readable
justification.** There are currently none in emitted output.

---

## 5. Determinism

The six-axis receipt — pin, snapshot, engine, rulepack, toolchain, formatter — plus
`Delta::{Unchanged, Explained, Unexplained, IncompleteReceipt}`.

**Every input to the output must be an axis of the receipt.** The failure mode this exists to catch
is a *false Green*: identical receipts across a changed program.

> **Measured, R3d.** The Go release was an input that changed nothing observable. `xxhash` extracted
> at go1.21 and go1.24 produced byte-identical snapshots and the same `snapshot_digest`. Go 1.22
> rescoped the loop variable — same syntax, different program — so the engine could have emitted a
> different program with every receipt axis holding. Fixed: the snapshot carries `build_config`, the
> preimage covers it, and releases 1.21–1.26 now yield distinct digests.

Any search over candidate representations must be **deterministic** — fixed candidate order, fixed
tie-breaks. A scored search whose weights vary by context is a receipt-breaking change, not a
tuning knob.

**LLM agents are never in the deterministic path.** They may act as reviewers and may propose, but
an engine whose output depends on a model's sampling has a receipt that certifies nothing.

---

## 6. Acceptance gates

The engine is DONE when four conditions hold together, and no earlier:

1. **It compiles clean.** `rustc` and `clippy-driver` with `--deny=warnings`, under
   `#![forbid(unsafe_code)]`. Zero `unsafe`, no exceptions for generated code.
2. **Blind review returns MERGE.** A reviewer who does not know the code was generated judges it
   well-written, best-practice Rust.
3. **Go-aware review returns MERGE.** A second reviewer, told it was ported from Go, finds no
   surviving Go quirk that Rust would do natively better.
4. **Nothing is silent.** Everything not translated is refused by name with what is missing.
5. **It stays ported.** Re-running against a moved upstream classifies as `Unchanged` or
   `Explained`, never `Unexplained`.

### Blind review is an acceptance test, not an anecdote

Reviewer objections are **data**. Each recurring objection becomes a deterministic rule, a refusal,
or a declared exemption — never a one-off fix. Track *why* review failed, against a fixed taxonomy,
so the loop closes:

```text
gratuitous clone            unnecessary shared ownership   Go-shaped API
weak error type             excessive dynamic dispatch     unclear module ownership
compatibility leakage       overcomplicated lifetime       lock held too long
unidiomatic collection use  missing domain type            mechanical control flow
```

---

## 7. The corpus is the Go language surface, not any one project

Ranking rule: **rank work by how many PACKAGES a cause blocks, not by its count in one.** A cause
with 50 sites in one package is a property of that package; a cause with 6 sites across 4 packages
is a property of Go.

Repositories are cloned to scratch, never into the repository.

The engine is not proven against Kubernetes or Talos. Proving it there first would make it a
Kubernetes porter. The criterion is coverage of the **Go language surface**.

Phased oracle corpus:

| Phase | Repository | What it tests |
| --- | --- | --- |
| 1 core language | `google/uuid` | arrays, slices, byte manipulation, `buf[:]` |
| 1 core language | `tidwall/gjson` | pointers, strings, tight loops, no dependencies |
| 2 interfaces | `go-chi/chi` | implicit interfaces, closures, embedding |
| 3 concurrency | `hashicorp/go-multierror` | mutexes, custom error interfaces |
| 3 concurrency | `hashicorp/memberlist` | goroutines, channels, `select`, tickers, sockets |

Required correspondences:

```text
implicit interface   → explicit trait + impl blocks
go worker()          → tokio::spawn(async move { .. })
chan T               → tokio::sync::mpsc or crossbeam
select { case .. }   → tokio::select!
buf[x:y]             → &buf[x..y]
closure              → Box<dyn Fn(..)> / impl Fn, by capture analysis
```

Go releases 1.21 through latest are in scope. The extractor type-checks at a stated release and the
release is a receipt axis (§5). The ceiling is the installed toolchain: a release it does not know
is refused, correctly, rather than silently downgraded.

---

## 8. Constructs that are refused, and stay refused

Not gaps — decisions. Each would require emitting machinery that changes observable behaviour:

- **GC finalizers**, and any use of reachability or reclamation as control flow. Go finalizers run
  at arbitrary later times, in a separate goroutine, may resurrect the object, and may never run.
  Mapping them to `Drop` changes observable timing and ordering.
- **`unsafe.Pointer` and pointer-bearing `uintptr`.** The type's meaning is the source's memory
  layout, which is not a thing the target has.
- **cgo-managed object graphs**; **runtime plugins** with unknown implementations.
- **Reflection that synthesizes or mutates arbitrary storage.**
- **Programs that depend on a data race.** Go gives data-race-free programs sequentially consistent
  behaviour; a racing program is refused, never "fixed" by inserting a mutex the source did not have.

---

## 9. Where the engine is exposed — measured, dated

State on 2026-08-20. Update with measurements, not impressions.

### Measure `compiles`, never `translated`

`survey` reports how many declarations TRANSLATED. That is the engine's confidence, not a fact about
Rust. Run `compile-corpus.sh` after every rule.

**State on 2026-08-20:** nine of ten emitted packages pass `rustc` AND `clippy-driver` with
`--deny=warnings` under `#![forbid(unsafe_code)]`. All five repositories the goal names are among
them — `uuid`, `gjson`, `chi`, `go-multierror`, `memberlist`. `semver` remains, on
`len_without_is_empty`.

Run the gate at the policy the engine is actually held to. Running `rustc` alone, without
`-D warnings`, measured a weaker claim and reported "compiles" for a crate that does not build:
`pub const K: PrivateType` is a warning.

A rule is finished when its cause has left the histogram AND the output still compiles. Coverage is
not the test in either direction — a rule that lowers coverage by refusing something it cannot spell
is a good rule, and a rule that raises it can break the build. Both have now happened:
the tagless-switch rule raised coverage and broke `uuid`; the string-slice refusal lowered coverage
and fixed gjson.

### Trust the instrument only after testing the instrument

Three separate times a measurement was wrong before the engine was:

- `rustc -o /dev/null` cannot create its temporary directory. Six packages read as broken.
- Concatenating the emitted modules into one file made names collide across two source packages.
  Nine of chi's fourteen errors were the harness.
- Nesting each package's single module under a directory of the same name produced `uuid::uuid`,
  which clippy calls `module_inception` and a reviewer called a Go shape. Also the harness.
  `review-bundle.sh` exists so this stops recurring.
- `survey_cause` had no arm for `UndecidedForm`, so one cause split into eighteen single-site rows
  and the largest structural blocker in the corpus never appeared in the ranking at all.

A number that changes the plan is worth one experiment against a case whose answer is already known.

### Ownership is decided per declaration and never reconciled across the call graph

Probe (`Mutate`/`Callee`, real output):

```rust
pub fn mutate(s: &mut [i64]) { s[0] = 9; }   // signature correct — the `mutated` fact worked
pub fn callee() -> i64 {
    let a = vec![1, 2, 3];
    mutate(a);                                // NOT `&mut a`, and `a` is not `mut`
    a[0]
}
```
`port=ok translated=2 refused=0`. The callee's ownership decision never reached the call site, and
the engine reported success. This is the §1 law, not a polish item.

### There is no aliasing model at all

Go's `b := a` shares backing storage; the engine emits a move. Today that *fails safe* only because
Rust's move rules reject the result — nothing in the engine knows the two views alias. The safety is
accidental, not designed, and the deep-copy variant of the same mistake would compile.

### Other open gaps

- **Closures translate only when they capture NOTHING.** A literal with captures refuses by name.
  The target infers borrow-versus-mutable-borrow for a closure that does not outlive its scope, so
  that case needs no analysis either; what needs an answer is the ESCAPING case, and its proof is
  whether a callee RETAINS the value — unknowable for a callee outside the corpus.
- **Go strings are bytes.** Indexing goes through `as_bytes()`; SLICING refuses, because `&s[a..b]`
  panics when a bound falls inside a multi-byte character and the source cannot fail there at all.
  Deciding the ported program's string type is what unblocks it.
- **Package-scope variable state is the largest structural cause** — 57 sites, three undecided forms.
  `init_written_package_var` is the tractable one: a variable only the package initialiser writes is
  computed once and never changes, and what it lacks is an initialising expression rather than a
  concurrency decision.
- **Nil interface ≠ interface holding a typed nil pointer.** Not yet distinguished.
- **`memberlist` does not extract** — type-checks into `golang.org/x/sys/unix`.
- **No differential execution against Go.** `rustc` supplies memory safety; it says nothing about
  semantic equivalence. Differential testing at contract seams is not optional and does not exist yet.

---

## 9a. Review findings DECLINED, with reasons

Recorded so they are not re-opened each round. Every one has been raised by a review gate at least
once; each is either the source's own design or a decision the engine has no evidence for.

- **`wrapping_add`/`wrapping_sub` on arithmetic that "cannot overflow".** The source's fixed-width
  integer arithmetic WRAPS. The target's `+` panics in debug, so plain arithmetic is a different
  program on overflow — and `byte + 32` reads as un-overflowable only because a human knows the
  operand is an ASCII letter. Faithful, not stylistic.
- **`pkcs7decode` panicking on malformed padding.** `buf[:n]` with a negative `n` panics in the
  source too. An engine that makes its input total is no longer porting it.
- **A stringly-typed error beside a typed enum** (`validate_key`). That is what the Go does.
- **`MethodTyp` public while its constants are private.** The exported method
  `MethodNotAllowedHandler(methodsAllowed ...methodTyp)` names the unexported type, so the target
  requires the type to be at least as visible — see §9's visibility rule. Hiding it would delete an
  exported name. The source has the same asymmetry and the same practical consequence.
- **`Send + Sync` supertraits on interfaces whose docs promise thread safety.** The bound is not in
  the source. An interface is not thread-safe because its documentation says implementations should
  be, and adding a supertrait the source does not state is inventing a contract.
- **`CompressFlusher::flush` taking `&mut self`.** Its implementors are foreign types the corpus
  never sees, so nothing is observed to mutate; the fallback is §3's reading of what an interface
  value IS. "Flushing is by definition a mutation" is a fact about the target's conventions, not
  about the source.
- **A sub-slice return becoming `Vec`.** Returning a borrow of a parameter needs a lifetime the
  engine does not infer — see §3, "No lifetime inference". Owned is the form always available, and
  the cost is an allocation rather than a meaning.

## 10. Rejected external proposals, with reasons

Recorded so they are not re-proposed as novel.

- **Conservative compatibility baseline, then refine** (§2). Rejected as the *release* path; the
  analysis it requires is adopted.
- **Data-model uplift** — inferring a `UserState` enum from string constants, deleting a boolean
  derivable from it. Rejected: the engine would be inventing domain meaning it cannot prove, and for
  a library the inferred model leaks into the contract. This is redesign, not translation.
- **Package-to-service architecture synthesis** — turning a Go package's globals and free functions
  into an owning `Service` struct. Rejected for the target corpus: these are libraries whose module
  shape is an intentional API.
- **Weighted optimization search over candidate representations.** Adopted only if deterministic
  (§5). Context-varying weights are rejected outright.
- **`Vec::clone()` for a Go slice assignment.** Rejected: shallow becomes deep (§3).

---

## 11. Where this doctrine sits

`CLAUDE.md` and `AGENTS.md` remain the repository authority and overlay this file on conflict. This
file is authority for `build/port-engine/` only.

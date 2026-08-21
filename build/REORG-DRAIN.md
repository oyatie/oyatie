# build/ reorg drain notes (`integ/build`)

## Completed (this rail)

- W0-B Slice 1: six-crate skeleton under `build/port-engine/` (landed on `dev` via #1642).
- W0-B Slice 2: neutral seam types extracted to `port-engine-api`; kernel depends on api.
- W0-B Slice 3: `port-engine-source-pin` pin loader (package-local `src/upstream-pin.json` mirror);
  `port-engine-rust-ir` empty renderer stub; `port-engine-app` driver wiring.
- W0-B Slice 4: `port-engine-frontend-go` SourceModel snapshot decode + Go firewall architecture
  test (`Command::new("go")` / `std::process::Command` refused in library sources).
- W0-B Slice 5: `port-engine-rust-ir` syn/quote emit (`SynQuoteRenderer::render_rust_ir`) +
  leakage-forbidden fences (corpus needles + no host `Command` spawn); driver smoke wired.
- W0-B Slice 6: `port-engine-app` hand-rolled CLI (`help|ready|pin|emit-stub|emit-syn|verify-e2e`)
  + six-axis receipt e2e (`unchanged`/`explained`/`unexplained`/`incomplete`).
- W0-B Slice 7: `port-engine-hash` (sha2 → `sha256:<hex>`) + `port-engine-rulepack` (embedded
  neutral v0 mirror of forever `specs/port-rules/**`) + facade CLI (`digest|rulepack|plan`) and
  hashed receipt e2e. Forever specs tree remains integ/specs.
- W0-B Slice 8: `port-engine-snapshot` admits OOB bootstrap SourceModel fixture (pin + content
  digest verify; never spawns Go) + facade CLI `admit-snapshot` + e2e binds admitted digest.
- W0-B Slice 9: `port-engine-identity` (`engine_digest`) + `port-engine-toolchain` (dual-home
  corpus → `toolchain_digest`; cell remap still PARKED) + facade `pipeline|receipt|engine|toolchain`.
- W0-B Slice 10: fixture-gated `port-engine-rulepack` (object rules + ≥1 `selecting_fixtures` each;
  missing/omitted fixtures refuse load) — hermetic mirror only; forever tree still integ/specs.
- W0-B Slice 11: `port-engine-transform` applies plan constructions/preconditions → `RustIr`;
  facade `transform` + pipeline pin→admit→plan→transform→emit→receipt.
- W0-B Slice 12: receipt hardening — golden receipt, byte-identical re-run (`delta`/`verify`),
  `render` entrypoint; **no** bulk `k8s/` corpus emission (W0-B / out-of-envelope).
- W0-B Slice 13: `port-engine-emit` single-fixture canary seam — select one
  `__canary_empty_unit` region, golden compare, optional materialize under basename
  `port-engine-canary-out` only; facade `emit-canary`. Refuses `k8s/` / bulk emit.
- W0-B Slice 14: canary materialize round-trip (`materialize-canary`) + planted-defect
  detect (`canary-defect` → Red/Unexplained on canary region); still no bulk `k8s/`.
- Toolchains dual-home: `build/toolchains/**` byte-copies `toolchains/BUCK` +
  `toolchains/cache/{BUCK,OWNERS,defs.bzl}` (4 files). Live buck cell remains
  `toolchains = toolchains` in `.buckconfig` until remap+shrink. Slice 9 mirrors those bytes
  under `port-engine-toolchain/src/corpus/*.txt` for hermetic receipt binding (`.txt` so buck2
  srcs globs include them; logical dual-home paths stay in the digest preimage) — keep mirrors
  in sync when dual-home bytes change.

## Go translation lane (`port/engine-go-translation`)

Slices 1–14 built a translator that has never translated: `SourceModel` carries unit ids and
nothing else, so the whole emitted corpus is two empty functions. This lane makes the engine
port real Go. It is W1-shaped work — ADR-0637 D4 authorizes W0 only, and the W0-B ready gate
lists corpus work under `forbidden_until_open` — undertaken as an explicit operator override,
recorded here rather than implied.

- I1: hermetic Go corpus + stdlib-only bootstrap extractor under
  `port-engine-frontend-go/gosrc/` (out-of-band; nothing in the Rust build reads it). Emits
  snapshot envelope v1 with per-package declarations; digest over a length-prefixed
  arity-tagged preimage, not over JSON. Firewall extended: library sources may not NAME the
  corpus tree, not merely refrain from spawning `go`.
- I2: snapshot envelope v1 decoded in Rust. `SourceModel` gains `declarations`, carried as ONE
  uniform recursive node whose `kind` is a value — a const, a field, a param and a method are the
  same shape — so Go's taxonomy stays out of the neutral seam. Kind and flag vocabularies are
  CLOSED in the Go adapter (where naming Go is allowed) and refuse anything unknown.
  `snapshot_preimage_v1` extends the digest over the declaration tree, mirroring the extractor's
  encoder; the two agreeing over a real corpus is what `v1_fixture_admits_and_carries_declarations`
  proves. CLI: `declarations`.
- I3: the engine ports Go. `captures` is load-bearing and splits rules into unit-level (captures
  nothing — the unchanged canary path) and declaration-level (one region per captured
  declaration). Constructions `rust_const|rust_type_alias|rust_newtype|rust_struct|rust_trait|
  rust_fn` emit real Rust items; the type map and its per-construction overrides are pack DATA and
  an unmapped type refuses rather than guessing. Coverage is proven: a declaration no rule captures
  and no `deferred_kinds` entry accounts for REFUSES, so nothing is dropped in silence — and a
  deferral must carry a reason, which travels in the pack digest. The five decode-and-drop fields
  are closed out: `captures` drives selection, `precedence` must agree with declaration order,
  `conflict` accepts only the implemented policy, and `required_diagnostics`/`proof_obligations`
  refuse while unimplemented. The wire shape is `deny_unknown_fields`. CLI: `port-go`.
  Merged the plan's I4 into this increment: constructions cannot render without the type map.
- I5: two fences re-scoped. The emitted-bytes needle scan in `port-engine-rust-ir` is GONE — it
  refused output containing corpus identifiers, which is the program working, not leaking; the
  production-source neutrality test (the actual ADR-0637 D1 rule) stays and is what caught the
  first draft of the comment explaining the removal. "We emitted the target language" is now
  carried by `syn::parse_file` plus the compile proof rather than by six fixed strings.
  `port-engine-emit` gains `materialize_tree` under a SECOND allowlisted basename
  (`port-engine-emit-out`); the `k8s/`, `..` and basename refusals are unchanged, and a region id
  must be a bare identifier before it can name a file.
- I6: the compile proof. `tests/port_go_compiles.rs` feeds the assembled emit to `rustc` — the only
  check that distinguishes correct output from stable output, and the one that would have caught
  I3's `pub fn` in a trait and `const: String`, both of which syn accepted. Adds per-unit module
  assembly (grouped by transform-reported provenance, never by parsing sanitized region ids), a
  committed golden refreshable via `port-go-source > src/port-go-golden-v1.txt`, determinism over
  the real corpus, and a planted defect landing on `Unexplained`. The six receipt axes carry real
  values for the first time — the W0-B journal recorded that no axis had ever held one.
- I7: function bodies, opening the long tail. The extractor walks statements and expressions into
  the same uniform node shape, and resolves each identifier through `go/types` so a reference to a
  constant cases as a constant instead of naming nothing. The supported subset is small — return,
  if/else, single-name `:=`, literals, idents, parens, and the binary/unary operators with a direct
  target form. Everything else is RECORDED as an `unsupported` node naming the AST type, never
  dropped: a dropped construct makes an untranslatable function indistinguishable from an empty
  one. `rust_fn_body` is a separate construction from `rust_fn` so a pack asks for a body and gets
  a refusal if it cannot have one, rather than degrading to a stub that still compiles.
  `corpus-refused/` (a `for` and a `defer`) exercises the refusal against real Go — a translator
  whose refusals are only tested on hand-built inputs has not been shown to refuse anything a
  front end would produce.

## Universal-engine program (same lane)

Driving the engine from a thin vertical slice to one that ports the whole Go language, verified
behaviourally against the Go original. Phased; the plan lives outside the repo, the record is here.

- P0: repo structure, naming, and the clean-architecture seam. Directory leaves stopped repeating
  the capability — every other capability is `<root>/<face>/<leaf>` with the leaf carrying a role
  or behaviour token and never the root (`audit/ports/emission-api`, `intelligence/facade/worker`),
  while port-engine repeated it in all thirteen crates. Package names already followed
  `<root>-<leaf>` and are byte-identical after the move, so no import changed. `PackSemantics`
  moved from `core/transform` to `ports/api`: it is a seam the transform consumes and the rulepack
  implements, so defining it in core made an adapter depend on the engine rather than on the
  contract. Production dependency graph is now clean in both directions — adapters reach only the
  ports face and sibling adapters; the single remaining `adapters/rulepack -> core/kernel` edge is
  `[dev-dependencies]` for one composition test. Historical audit notes in `governance/check/`,
  `specs/k8s-port/` and `.grok/` that describe PR #1621's tree were left alone: they are accurate
  about a past state, and rewriting them would falsify a record rather than update one.

- P0b: module split to the 100–300 line bar, and the fence upgrade the split MADE NECESSARY.
  Every architecture fence read `include_str!("lib.rs")` and nothing else, which was complete only
  while a crate was one file — a `mod other;` compiles a file the scan never reads, so the
  forbidden call had somewhere to hide one line below the thing checking for it. Each fence now
  enumerates every production source AND proves the enumeration is the whole of `src/`; planting an
  unscanned module reds it by name (verified by execution). Unit tests moved to `tests/`, so `src/`
  is pure production and the scan is total by construction. 13 crates, 34 green test binaries.

  THREE FILES ARE DELIBERATELY EXEMPT, with their reasons:
  - `core/kernel/src/lib.rs` (512) — its single-file property is a COMPILE-TIME PROOF: the kernel
    scans its own bytes and `UNSCANNED_CODE_KEYWORDS` refuses `mod` and `include!` outright, so
    "the kernel is exactly this file" is a property of the build. Splitting it means deleting the
    proof, which is a larger decision than a line count.
  - `core/kernel/tests/seams.rs` (732) — same lineage. Its completeness argument is that its buck
    target names exactly ONE source file, and the kernel `include_str!`s it for the corpus scan.
    Splitting needs that argument rebuilt per file, not just the file cut in half.
  - `core/rust-ir/src/lib.rs` (313) — P1 rewrites it wholesale for the typed IR. Splitting it now
    and again in P1 is waste.

- P1: the typed IR, a real formatter, documentation, and the receiver decision.
  `port-engine-rust-ir` no longer builds Rust with `format!`. Items, statements and expressions
  are a TREE; `quote!` lowers it to tokens, `syn` checks the assembly, `prettyplease` formats.
  Three things became structural rather than textual: precedence (an operand is bracketed exactly
  when the grammar would otherwise reassociate it, so `a + b * c` emits bare), visibility (a value
  the IR places, which is why `pub` can no longer reach a trait item), and documentation (carried
  as data, rendered as `///`).
  `formatter_digest` now names the formatter and its version instead of hashing the caller's
  label — the axis attested to nothing, so a whole-corpus reformat would have arrived as
  `Unexplained`. Refreshing the goldens confirmed EXACTLY ONE axis moved, and it was that one.
  Doc comments: 18 blocks captured where 0 survived before. `parser.ParseComments` was missing,
  so every `Doc` field was nil and the loss was total and silent.
  The trait receiver is now DECLARED in the pack with a recorded reason, and absent means refuse.
  It cannot be recovered from a Go interface — the interface says nothing about how an
  implementation binds its receiver — so `&self` was a guess, and it made the fixture's mutating
  `Rename` unimplementable. The pack chose `exclusive` and says why.
  Emitted-code lint quality: 19 pedantic warnings over 24 lines before, 13 now, and every class
  P1 targeted is gone (needless return, unnecessary parens, redundant static lifetime). What is
  left is `#[must_use]`/doc-backtick idiom rules and `todo!()`-stub artifacts.
  Two defects the rewrite surfaced and fixed in place: the tail-expression idiom leaked into
  nested `if` branches, producing `if id == "" { fallback }` — parses, does not type-check; and
  the module assembler indented only a region's first line.

- P2: structured types and cross-package resolution (snapshot v2).
  A type is a TREE now, the same uniform node the declaration tree uses, carrying the PACKAGE that
  declares it. A flat spelling worked exactly as long as every type was primitive or had its own
  table row, and failed three ways at once otherwise: a composite needed a row per shape rather
  than per constructor, a type from another package resolved to nothing because the table was keyed
  by unqualified text, and two packages declaring `Point` collided.
  Resolution walks the tree: a local named type → its emitted name; a named type from another unit
  → that unit's emitted module path; a primitive or a composite → the pack; anything else refuses
  by name. The pack answers composites by CONSTRUCTOR (`slice`, `map`, `pointer` templates), so one
  entry covers every slice.
  `module_path` lives in the transform beside `module_name` and the facade's assembler now uses the
  same function — deriving it in two places is how a cross-unit reference points at a module nobody
  emitted. The path is ABSOLUTE (`crate::shapes::Point`) because the emitted unit modules are
  siblings; the relative form compiled nowhere, and the COMPILE PROOF is what caught it. A golden
  would have frozen the broken path.
  The extractor became multi-package: it is its own importer, memoised on the package so a diamond
  import cannot produce two `types.Package` values for one package. Unit ids are now real import
  paths, module-root-relative — they were corpus-root-relative, announcing a name no Go file could
  import and that would not have matched the package identity on a cross-package reference.
  v1 is REFUSED rather than half-decoded: it cannot answer what v2 asks, and treating its spellings
  as opaque names would reinstate the flat table. New corpus package `geometry` proves the whole
  path — `crate::shapes::Point`, `Vec<i64>`, `BTreeMap<String, i64>`.

- P3: ownership and mutability. New core crate `port-engine-analysis`.
  Go is garbage-collected, so a `*T` says nothing about ownership — it may be a borrow, an owned
  value passed by pointer, or a shared structure with live aliases. The front end now OBSERVES
  facts (`mutated`, `escapes`, `effect_unknown`), the pack declares RULES in order, and the
  analysis pairs them. Pointer receivers stopped being refused.
  WHAT THIS DOES NOT PROVE, recorded in the crate docs rather than discovered later: escape
  analysis shows LIFETIME compatibility, not the EXCLUSIVITY a borrow needs, because a Go caller
  may pass one pointer as two arguments. That is caller-side and no callee analysis closes it. So
  a disposition is a HYPOTHESIS, and what makes emitting one defensible is that the target checks
  it — an unsatisfiable `&mut` is a borrow-check error the compile proof catches, which is a red
  build rather than silent corruption. `effect_unknown` keeps the gap visible: a decision made on
  unproven facts is MARKED in the record, never blended into the proven ones.
  Every decision is recorded per site with its rule, its form and its justification, and surfaced
  as `port-engine-app dispositions` — a separate artifact, because `&mut self` looks identical
  whether it was proven or assumed, and inline comments are where a rule change is hardest to
  review. There is deliberately NO catch-all rule: facts nothing accepts refuse.
  Two things the corpus surfaced: a Go VALUE receiver is not Rust `self` — Go copies and the
  caller's value survives, Rust consumes — so it emits `&self` and a mutated value receiver
  refuses; and `Move` is a Rust keyword, so identifiers now escape to raw form (`r#move`) rather
  than being refused, with the four un-rawable keywords renamed instead.
  Refusal corpora split per CLASS: one corpus stopping at whichever package sorted first meant the
  second refusal was never exercised.

- P4: statements and expressions. Every method in the corpus carries a TRANSLATED body.
  Two source constructs mean something the same syntax does not mean in the target, and both were
  emitting code that parsed and did not compile.
  A FIELD READ is a copy in Go and a move in Rust: `return c.label` leaves the receiver intact in
  the source and moves out of `&self` in the target. Reads of a non-copying type now clone, and
  which source types copy is a pack table — keyed by SOURCE identity, like `type_map`, after a
  first attempt keyed it by target spelling and cloned every `int` in the corpus because `int` is
  not `i64`. Position is what makes the rule safe: a field in VALUE position is read, a field in
  PLACE position is assigned to, and cloning the latter would emit `self.total.clone() = x` —
  which parses and silently assigns to a temporary.
  A STRUCT LITERAL zero-fills in Go and must name every field in Rust. `Point{X: 1, Y: 2}` on a
  struct with an unexported `label` was an incomplete literal. Which fields a struct has is a fact
  go/types holds and the engine does not, so the front end now emits one entry per DECLARED field
  and the omitted ones arrive as `zero` nodes carrying their type. The target's spelling of a zero
  is a pack table; a type with no entry REFUSES rather than falling back to `Default::default()`,
  which would compile for these four types and quietly mean something else for a type whose
  `Default` is not its zero. The same change closed a silent hole: a slice or map literal used to
  produce a composite with no keyed children and emit `Vec {}`, constructing nothing.
  `no_method_body_is_a_stub` is the proof. Every other check in the facade passes over a crate
  whose methods all abort at the first call — a stub compiles, matches a golden, and hashes into a
  stable receipt. Asserted over the whole emit, so a stub reintroduced anywhere reds it.
  The body translator stopped being ambient. The copy set had been a thread-local to avoid
  threading a parameter through twenty functions, and the moment a second table arrived the
  shortcut stopped paying: these are properties of the RULE PACK, not of the process, and a body
  translated under a different pack must see different answers. One `Body` context carries the
  owner and the resolver, and the resolver owns every table lookup — which is what fixed the
  key-rule divergence above, since `resolve` and `copies` now key a type the same way by
  construction.
  Emitted-code lint quality: 16 pedantic warnings over 110 emitted lines, from 13 over ~60. Ten of
  the sixteen are `#[must_use]`, and adding it is NOT obviously right: Go does not require a
  caller to use a return value, so the attribute is a claim the source never made. Two are
  `assign_op_pattern` (`x = x + n` where the source wrote exactly that) and one is
  `comparison_to_empty_slice`. All three are idiom rules and belong in P9 as pack data, not as
  engine defaults.
  `core/rust-ir/src/expr.rs` split: the operator PRECEDENCE table moved to `ops.rs`. The rust-ir
  directory-completeness fence caught the new module by name, which is the fence working.

- P5: interfaces and impls, from OBSERVED satisfaction.
  Go's interfaces are implicit: nothing in a type's declaration says which interfaces it satisfies,
  and structural matching is combinatorial. `census/interfaces.md` measured the two emission
  strategies at 80,042 name-level structural matches against 1,316 pairs the source declares
  outright, and its conclusion is that the engine must emit from USAGE. It now does.
  The front end walks four SITE KINDS — a declared `var _ Iface = ...` assertion, an assignment, a
  call argument, a return — and records the pair with the site it was seen at, because a declared
  assertion is compile-checked by Go and a flow-derived one is the extractor's inference, and the
  two produce identical Rust. Collection is per package and attribution is CORPUS-WIDE: the unit
  that observes a flow is not in general the unit that declares the concrete type, which is the
  census's own `NewCodec` example. A pair whose concrete type this corpus does not declare is
  recorded as unsupported rather than dropped.
  THE TRAIT RECEIVER STOPPED BEING A GUESS. P1 made it a declared pack decision because an
  interface says nothing about how an implementation binds its receiver — one mode for every
  method, which put `&mut self` on getters. With the implementors observed it is derived per
  method: exclusive exactly when some implementor mutates. `Named` now reads `fn name(&self)` and
  `fn rename(&mut self)`. Shared is a claim that NO observed implementor mutates, and since the
  observed set is a lower bound, a later implementor that does is a compile error in the emitted
  crate rather than a silent aliasing change. The pack's decision survives as the fallback for
  interfaces nothing was seen to implement, and a method with neither refuses.
  Each impl method DELEGATES to the inherent method rather than carrying the body: a body lives in
  one place, and a type satisfying two interfaces that share a method name would otherwise need it
  in both. The call is a PATH call, because inside a trait impl `self.name()` resolves against the
  trait and recurses into itself.
  A TRAIT IN A VALUE POSITION is now a declared decision. The source holds an interface value
  directly and the target cannot — a trait has no size — so it arrives as `&dyn T`, `Box<dyn T>`,
  `Rc<dyn T>` or a generic parameter, and those differ in who owns the value. The pack declares a
  form per POSITION; `param` is declared and `result` deliberately is not, so returning an
  interface refuses by name. A trait nested inside a composite refuses outright rather than
  resolving through the slice constructor into `Vec<crate::shapes::Named>`.
  THE ORPHAN RULE DOES NOT BITE, and the reason is recorded rather than guarded by a check that
  cannot fire: the engine emits every unit of one corpus as a MODULE of one crate, so both sides of
  every pair are local. It becomes reachable when a trait or a type crosses a crate boundary —
  `go-rt` at P6, a multi-crate corpus at P10 — and the census's 6 foreign-on-foreign assertions are
  the population that will need the newtype treatment then.
  Interface method documentation was being dropped, the same way `parser.ParseComments` dropped
  everything: the member-doc indexer matched only `*ast.StructType`, so every comment on an
  interface method went nowhere. Both shapes are an `*ast.FieldList`, so one function answers.
  The 1,643-line extractor split into thirteen files, verified by regenerating the snapshot BYTE
  FOR BYTE — a digest-bearing artifact is the strongest available proof that a refactor changed
  nothing.
  New corpora: `corpus/naming` (two implementors, so the receiver union is a union rather than the
  first answer found) and `corpus-interface/positions` (the third refusal class). Emitted-code
  lint: 21 pedantic warnings over ~170 lines; ten are `#[must_use]`, which stays a P9 decision
  because Go does not require a caller to use a return value and the attribute would be a claim the
  source never made.

- P5b: embedding, on both sides of it.
  Go composes by embedding and nothing forwards: an anonymous field lifts the embedded type's
  methods into the outer type's method set, and an embedded interface lifts its requirements into
  the outer interface's. The target has neither rule, so both became explicit.
  An embedded INTERFACE is a SUPERTRAIT — `census/interfaces.md` §6's own reading, and 87.3% of
  embedding interfaces embed exactly one. A requirement rather than a copy of the method set: a
  flattened method list compiles and means something weaker, because a type could then satisfy the
  outer trait without satisfying the embedded one.
  An embedded STRUCT becomes FORWARDING METHODS, which closes §11 item 7 exactly. That census
  recorded a method only where a declaration carried a receiver, so 2,747 CORE struct types have
  method sets larger than it measured and 479 appear to have none at all; it names go/types as what
  would close it, and the front end has go/types. The forwarding method's RECEIVER is not a new
  decision — it has no body of its own to observe, and what it may do is decided entirely by the
  method it forwards to — so the promoted node carries the embedded method's own ownership facts
  and the same rules decide it. `Driver::Run` appears in the disposition record as
  `exclusive_borrow`, proven, exactly like a declared method.
  The two are proven TOGETHER because their interaction is what fails alone: `Driver` satisfies
  `Job` only through a promoted method, so emitting the supertraits and skipping the promotion
  produces an impl naming a method nothing implements, which nothing short of compiling it notices.
  Satisfying an interface now records satisfaction of everything it embeds, transitively — the Go
  compiler checks that, and the target needs it written down, since a supertrait is a REQUIREMENT
  and `impl Job for Driver` does not compile without `Runner` and `Describer`.
  Two defects the corpus surfaced, both silent: an `implements` node carried the interface's FULL
  method set, which was right while no trait had supertraits and became wrong the moment one did
  (`impl Job` named `describe`, which is not a member of `Job`); and an EMBEDDED field has no name
  in the syntax — its name is its type — so the member-doc indexer, keyed on `Names`, dropped every
  comment on one.
  Method documentation was also being dropped at the signature layer while the front end captured
  it — the same silent loss as the interface methods, and invisible for the same reason.
  `admission.rs` stopped asserting a package COUNT and now asserts the package SET. A count has to
  be edited every time a corpus package lands, which makes the edit routine and the check
  ceremonial; a set says which package went missing (ADR-0717).
  New corpus `corpus/composite`. Emitted-code lint: 21 pedantic warnings over 281 lines, unchanged
  in absolute terms across a 65% larger emit.

- P6a REVERTED, and the scope rule it clarified is worth keeping.
  `k8s/core/go-runtime` was a hand-written Go concurrency runtime — channels, `WaitGroup`, `Once`,
  the `wait.Until` family — placed under the `k8s` capability by the registry's own `base/`
  admission rule. It was correct work and it was the WRONG WORK: that crate is a component of the
  PORTED ARTIFACT, not of the engine that produces it. Nothing in it makes the engine translate
  anything, and building it first meant the destination's runtime existed before the engine could
  emit a single call to it.
  THE SCOPE RULE: this lane builds the ENGINE, and its work stays inside `build/port-engine/`. The
  destination tree (`k8s/`), its runtime libraries, and the specs that classify them are downstream
  of an engine that can emit them, and touching them ahead of that is building the port rather than
  the thing that ports.
  The census reading that motivated it stands and is recorded here so it is not re-derived:
  `census/concurrency.md` §6 sizes the concurrency surface at ~65–85 mechanical rules plus FIVE
  hand-ported runtime libraries, and `wait.*` alone is 108 of 400 named goroutine launches (27%)
  and 65% of the S1 background-loop shape. When the engine can emit a `go` statement, those
  libraries are what its rules should target — and that is when they get written.

- P2 (the one the plan ordered second and this lane reached last): the FAILURE CONVENTION, and the
  function map. These are what the plan calls "the two mappings that block every real Go package",
  and the diagnosis is worth stating precisely: what blocked a real package was never one construct
  the engine lacked. It was a CONVENTION it could not see.
  The source returns failure as an extra RESULT. Nothing in a signature says the value must be
  checked and nothing in the type system stops a caller from dropping it — it is held up by
  discipline. The target says the same thing in the return type, where the compiler holds it up. So
  this is one of the few translations that makes the ported program STRICTER than the original
  rather than merely equivalent to it, and it is the reason a mechanical port is worth having.
  Three shapes, each matched structurally and refused otherwise. A fallible signature: a trailing
  result of the failure type is split off before any result is resolved, so `(T, error)` becomes
  `Result<T, E>` and `error` alone becomes `Result<(), E>`. A return: the TRAILING operand decides
  the whole construction, so the same `return x, y` is `Ok` or `Err` depending on it. And the
  CHECK — `v, err := f()` followed by `if err != nil { return …, err }` — collapses into `f()?`,
  which is the only one that becomes an operator rather than a statement, and the only one that
  moves the check out of discipline and into the type system.
  A failing return DISCARDS its companion operands, and that is sound exactly when they are zero
  values — the convention says a caller may not read them after a failure. When one is COMPUTED it
  refuses, because dropping it loses work the reader of the emitted crate could never see was lost.
  Admitted: literals and the absent value. Deliberately not admitted: anything that would require
  deciding some expression is "obviously" zero.
  Only the SOURCE half is pack data — which type carries the convention and how the absent value is
  spelled. `Result`, `Ok`, `Err` and `?` are decided in the transform, because that face renders
  Rust and a second language pair must not have to re-declare the target's own vocabulary.
  THE FUNCTION MAP is the other half of "blocks every real package", and it is not optional: every
  real package calls its standard library, and the standard library is exactly the part that does
  not come along. `function_map` answers for a CALL the way `type_map` answers for a type, keyed by
  the callee's IDENTITY from the type-checker rather than by its spelling — `errors.New` and a local
  variable named `errors` are the same text. A template brackets itself; an argument that is a
  compound expression refuses, because substituting one into a text template needs parentheses the
  template cannot ask for.
  Two source facts had to exist first, and both were losses rather than gaps. `nil` reached the
  snapshot classified as a local, so `return x, nil` and `return x, err` were the same shape from a
  distance — and telling them apart is the whole of the convention. And `v, err := f()` was recorded
  as unsupported, so the shape every fallible call in the source has could not be seen at all.
  New corpora: `corpus/fallible` (four shapes of the convention) and `corpus-failure/carried` (the
  fifth refusal class). The compile proof carries `Result`, `?` and the mapped calls through
  `rustc`, which is what makes the claim more than a golden.

- Continuous re-port: the UPSTREAM DRIFT proof, and the SURVEY that measures the engine against
  source it has never seen. This is the pair that turns "the engine ports a fixture" into "the
  engine can be kept pointed at a moving dependency".
  DRIFT. Everything the facade proved before was about determinism: the same source twice gives the
  same bytes, and bytes that move with no axis to account for them are RED. Neither says anything
  about the case that actually happens in service — upstream releases, the engine re-runs, and the
  output differs because the SOURCE differs. That case has to come back Green AND Explained, and it
  now does, over a real second EXTRACTION of one package at two versions at the same unit id, with
  the two changes a dependency bump makes: a body changed and a declaration appeared. The axis set
  is asserted EXACTLY — only the snapshot axis may move, because the engine, rules, toolchain and
  formatter are the same run of the same code across the pair. An engine that answered `Unexplained`
  every time a dependency moved would be crying wolf at its operator until nobody read the signal
  again, and that signal is the only thing between a maintained port and a fork nobody dares
  regenerate.
  SURVEY. `apply` is fail-closed, which is right for PRODUCING a port and wrong for MEASURING one:
  pointed at a real package it reports the first refusal and says nothing about the other nine
  hundred declarations. `survey` attempts each declaration independently and reports a RANKED work
  list, so the next rule to write is the one blocking the most rather than the one most recently
  thought of. It deliberately ignores the pack's `applies` map, because that map is policy and the
  question is capability — a survey restricted to units somebody already listed measures the list.
  It also separates a DEFERRAL from a hole: a deferral is a decision with a written reason, and a
  hole is a kind nobody has looked at; counting them together understates the engine in one
  direction and hides a decision in the other.
  THE RATCHET, against `google/uuid` — reputable, stdlib-only, 97 declarations, and not in this
  repository. Corpora for this live outside the repo on purpose: a corpus committed beside the
  engine only ever contains what the engine already handles, which is exactly why the fixture
  corpus could reach five phases without anyone noticing `byte` was unmapped.
  Rung 0: **20.6%**. The first run did not even admit — the `range` loop's `key` attribute was
  emitted by the front end and absent from the closed vocabulary, because the fixture corpus reaches
  ranges through a shape that binds only the value. A closed vocabulary is a check only over what is
  exercised.
  Rung 1: **37.1%**. The type map had FOUR entries, which was enough for a corpus written beside it;
  a real package uses most of the basic set in its first file, and `byte` alone blocked eight
  declarations. Now the whole set, plus the untyped-constant identities and `error` as the trait it
  is. `uncaptured` fell to zero: every kind is either handled or a recorded decision.
  Rung 2: **44.3%**. A destructuring bind that is NOT a failure check (the propagation matcher
  consumes those first, so what reaches the statement layer means exactly what the target's tuple
  binding means), and the slice expression — BORROWED, because the source's slice is a view and an
  owned target would be a different program with different aliasing. A three-index slice refuses:
  it sets the result's capacity, which the target does not express at all.
  Rung 3: **47.4%**. `var` inside a body — a `let` with an optional type and an optional
  initializer, which is a shape the target has exactly, and which ten of uuid's functions were
  waiting on.
  The MUTABILITY that arrived with it is the part worth recording, because it was a latent defect
  rather than a gap. Every binding in the source is mutable and the target makes none of them: a
  `let mut` default warns on every binding a body never writes again, and a `let` default fails to
  compile the first time one is assigned. Neither is a judgement call — the body says which — so the
  front end now indexes what a body ASSIGNS TO and the transform spends the fact. `:=` bindings had
  the same problem and compiled only because no fixture ever reassigned one, which is the same
  shape of hole the type map had: a corpus written beside the engine does not exercise the engine.
  Taking a pointer to a binding counts as a write, deliberately: the write itself may be anywhere,
  and being conservative costs a warning where being wrong costs a compile error.
  A DECLARED type is carried onto the binding rather than left to inference, because the two
  languages default differently — an untyped integer literal is the case that bites — and dropping
  it would change what the binding IS.

  What the ratchet says is left, in its own order: `var` at package scope (27, deferred with a
  recorded reason — a mutable global whose target form is a synchronization policy the source never
  stated), `DeclStmt` (10), `defer` (9), and a tail of type assertions, array types and one
  carried-value failing return.

- R0: the determinism contract stops being vacuous.
  `engine_digest` did not hash the engine. It hashed `engine-identity-v0.json` — a hand-maintained
  list of crate NAMES plus a wave label — so no engine change ever moved it. By the kernel's own
  rule, emitted bytes that change while every axis holds are `Unexplained` and RED, which means
  every engine change was by definition an unexplained one; and nothing ever reported it, because
  the delta check runs a single binary twice and can only answer `Unchanged`. **The six-axis
  contract was sound in five directions and empty in the one where the engine is what changes** —
  and every phase of the idiom work would have landed under that hole.
  Now it is a content digest of the engine's own sources, 83 files across 14 crates.
  WHERE THE ENUMERATION LIVES was the design question, and the first answer was wrong. Having
  `port-engine-identity` reach across packages with `include_str!("../../../core/transform/...")`
  is correct Rust and inverts the hexagon: an adapter reading `core/` and `facade/` points the
  dependency direction backwards. It also put 83 files outside the package into the target's inputs,
  which no package-relative build glob can express — the embedded-asset hermeticity gate refused the
  whole BUCK expression and produced 83 born-blocking skips. Resolving that by editing the gate's
  baseline would have been weakening a gate to fit a design.
  So each crate embeds only what it OWNS, package-local, covered by the glob already there and
  needing no BUCK change; the FACADE joins them, because the facade is the one place the whole
  engine is legitimately visible; and `port-engine-identity` keeps the ENCODING — what hashing them
  means — which is a different question from what the engine is made of.
  The kernel carries its one-line manifest inline. It refuses a submodule declaration as a whole
  identifier so that "the kernel is exactly this file" is a property of the build, and a generated
  submodule would delete that proof to save four lines. Its scan then rejected the doc comment
  explaining this, because the comment spelled the refused keyword — the gate working on its author.
  TWO FENCES, because neither is worth anything alone: the manifest is the whole engine (a walk
  compared against what the crates embed), and the digest moves when the manifest does (a perturbed
  preimage). A complete manifest under an insensitive hash reports a constant; a sensitive hash over
  a partial manifest reports a constant for everything it cannot see — which is precisely the
  failure that shipped. A third asserts every crate contributes, since a crate with an empty
  manifest contributes no paths and the set comparison would pass while it went unhashed.
  THE GOLDEN SPLIT BY WHAT IT CLAIMS. With a real content hash the golden receipt would pin a value
  that changes on every commit, and a golden refreshed every commit is refreshed reflexively — the
  vacuous green this repo names by name. Five axes SHOULD hold across an engine change and pinning
  them catches a real defect: a snapshot digest that moved while the corpus did not is a bug.
  `engine_digest` moving is the normal case, so the golden records `<varies>` and the check asserts
  its SHAPE instead, which still catches an axis gone empty or malformed.
  Also length-prefixed `emit_tree_digest`, which was still NUL-separating where the snapshot and
  engine preimages are length-prefixed. A separator-delimited encoding is unambiguous only while the
  separator cannot appear in the content, and emitted source is arbitrary.
  New: `port-engine-app region-digests` prints per-region digests, so a whole-program change's blast
  radius is countable rather than one line of one golden diff.

- R1a: the two call defects that were shipping WRONG OUTPUT, not missing output.
  A call decided method-versus-function by SYNTAX: a selector callee became a method call on the
  selector's base. That is right for `value.Method()` and wrong for `package.Function()`, and the
  source spells both the same way — so a cross-package call emitted a method call on a binding that
  does not exist. It looked fine only because the fixture's one such call was in the function map and
  never reached the fallback. Only the type-checker can tell the two apart, so it now records which,
  and a free function resolves to a PATH through the same `module_path` a cross-unit type uses — so
  a call and a type reference to one unit cannot disagree.
  A CONVERSION is spelled exactly like a call and is not one. `uint32(x)`, `Celsius(f)` and
  `[]byte(s)` are all call expressions in the source, so they reached the transform as calls whose
  callee resolved to nothing. Three target forms, because they are three operations: to a named type
  the corpus declares it CONSTRUCTS the newtype; between numeric types the source is defined to
  truncate and the target spells that as a cast; between string and byte slice it is infallible and
  lossy in the source and FALLIBLE in the target, so it refuses — what happens to input the target
  rejects is a decision rather than a spelling.
  `RustExpr::Cast` is a NODE, not text, so integer right-sizing can see the cast and remove it. Its
  operand is bracketed: `as` binds tighter than every binary operator, so `a + b as u8` casts `b`
  alone — a different program, and one that compiles.
  Coverage on google/uuid went 47.4% → 38.1% → 43.3%. The drop is the point: ten declarations were
  emitting a method call on a package name and now refuse by name. A ratchet that only ever rises is
  measuring the wrong thing.
  Every architecture fence gained the R0 `sources.rs`, each caught by its own completeness assertion
  — a file a fence does not read is a file a forbidden call can hide in.

- buck2 IS installed, and this lane had been recording that it was not. The claim came from the plan
  and was never checked, so the local-hermeticity path went unverified for five phases — and three
  real defects were sitting behind it, each invisible to cargo.
  `adapters/rulepack` had never been buildable under buck2. Its target omits `serde` and
  `serde_json`, which the crate has used since the pack became JSON. Cargo has no target list, so
  nothing noticed.
  `core/rust-ir` STILL is not, and that one is not drift. It uses `prettyplease` and `proc-macro2`
  directly, and the third-party cell exports neither: `syn` carries `visibility = ["PUBLIC"]` while
  `prettyplease-0.2` and `proc-macro2-1` carry `visibility = []`. The BUCK now names the real
  versioned targets and still fails, which is the right failure — it names exactly what is missing.
  Exporting them is a `third-party/` change and outside this lane.
  The COMPLETENESS FENCES were cargo-only. Each locates its crate's `src/` through
  `CARGO_MANIFEST_DIR` or a bare relative path, and neither resolves under buck2, which runs a test
  from the project root with no cargo environment. So every fence hit its own "a fence that cannot
  look has not looked" refusal — correct behaviour, wrong outcome: the fence was right that it could
  not look, and the reason was that nobody had told it where to look from. Each now also knows its
  repo-relative path, which is true under either build system.
  And `facade/app/tests/port_go_upstream_drift.rs` had no buck target at all, so the re-port proof
  ran under cargo only. A test with no target is a test one of the two build systems never runs.
  Verified by execution: ten engine crates build under buck2 and 20 of their tests pass there.
  `rust-ir`, `transform` and `app` remain blocked on the third-party export.

- R1b: two mappings that were silently wrong, and one that had nowhere to land.
  `[N]T` is a VALUE array and was emitted as a growable heap type. Three things changed at once
  under that mapping and none of them was visible in the output: assignment stopped copying and
  started moving, the length left the type so nothing checked it, and a fixed-size value became an
  allocation. The length had been extracted all along — a type node carries its non-type datum in
  `name` — but a constructor template could only substitute ARGUMENTS, so the datum arrived with
  nowhere to go. Templates now take `{name}`, and the pack says `[{0}; {name}]`.
  THE INDEX MISMATCH had to land somewhere. The source indexes with its `int` and the target with
  `usize`. The engine had put the conversion on `len`, so a value would type as the source's int —
  right for `return len(s)` and wrong for the counter of `for i := 0; i < len(v); i++`, whose body
  then would not compile. Every indexed loop in every real package is that shape.
  Moving it to the index alone was also wrong, and the golden said so within a minute: `Ok(s.len())`
  into a `Result<i64, _>` does not compile either. BOTH positions need it, because the same value
  reaches both — so `len` casts to the source width and the index casts back. Where a counter makes
  the round trip the pair is visible and redundant, and removable by the right-sizing analysis R6
  plans, which is exactly why a cast is an IR NODE rather than characters in a string.
  The trade is recorded rather than discovered: a negative index panics in both languages for
  different reasons — the source bounds-checks a negative, the target wraps it to an enormous
  `usize` and bounds-checks that. Same outcome, different message.

- The ratchet stopped being one package's opinion. Eight reputable stdlib-only Go packages were
  surveyed in parallel, out of tree, and the work list is now ranked by how many PACKAGES a cause
  blocks rather than how many declarations one package has.
  Coverage: xxhash 69.0%, ksuid 42.9%, uuid 43.3%, xid 41.7%, ulid 29.4%, pkg/errors 26.3%,
  semver 26.3%, go-multierror 0%. uuid sits at the median, which is worth knowing — it had been
  the only evidence.
  TWO PACKAGES COULD NOT BE ADMITTED AT ALL, and the cause was mine. P5 recorded a satisfaction
  whose concrete type the corpus does not declare as a package-scope declaration of kind
  `unsupported`, so it would refuse by name rather than vanish. But `unsupported` is a MEMBER kind,
  not a declaration kind, so the decoder refused the ENTIRE SNAPSHOT on an unknown kind and the
  package produced no measurement at all. The trigger is `var buf bytes.Buffer` passed where an
  `io.Writer` is expected — ordinary Go, not an edge, and two of eight packages hit it.
  The recording was right and the KIND was wrong. `foreign_satisfaction` is now its own admitted
  declaration kind, deferred by the pack with the reason written out: there is nowhere to emit the
  impl, because the type belongs to neither this corpus nor this crate and the target's coherence
  rule forbids it outright. Deferred rather than refused, because refusing would reject every
  package that touches the standard library through an interface — which is most of them. A generic
  `unsupported` at package scope was rejected as the fix: a kind broad enough to cover this is broad
  enough to swallow any package-scope construct the front end cannot model.
  `semver` went from whole-package rejection to a 26.3% measurement over 57 declarations.
  Ranked across packages, what blocks the most: `var` at package scope (4 packages), `IfStmt`
  variants (4), `AssignStmt` forms (3), unary `&` (3), `panic` (3), the carried-value failing
  return (3), VARIADIC signatures (3), and `unmapped type interface` (3). Variadic had not appeared
  in uuid at all and blocks three packages — the single strongest argument for surveying more than
  one.
  Also recorded from the same pass, as a critique of the instrument rather than of the engine:
  `survey.rs` collapses every `var` refusal into one row, which asserts that one rule would unblock
  28 declarations. That is false — the deferral label covers a DATA gap (a package var records no
  initializer, where a const does), a PROVABILITY gap (nothing computes whether a package var is
  ever written, and `init` bodies are never indexed at all because go/types does not enter `init`
  into package scope), and a POLICY gap. Three fixes, not one rule.

- `func init()` was reaching the model NOWHERE — not refused, not deferred, invisible.
  Declarations come from package SCOPE, and go/types deliberately keeps `init` out of it: the name
  is not addressable, several may exist in one package, and only the runtime calls them. So
  `declFor` never saw one, `prove_every_declaration_is_accounted_for` had nothing to account for,
  and a package whose `init` builds a lookup table ported to a program that never builds it. Three
  of eight surveyed corpora declare one.
  That is the exact failure this engine refuses everywhere else — output that compiles and means
  something different — reached by a path no refusal covered, because the construct never became a
  declaration at all. The coverage proof can only prove things it can see.
  Two causes, both fixed. The body was never INDEXED, because the indexer read `Uses` for a
  declaration's own name and fell back to a package-scope lookup — correct for everything
  addressable, silently empty for `init`. A declaration's own name is a DEF, and reading `Defs`
  first fixes it. And nothing COLLECTED it, so it is now gathered explicitly into one declaration
  per package carrying every body in FILE ORDER — one rather than several, because that order is a
  guarantee the source makes and splitting them would hand it to a name sort.
  Deferred by the pack with the reason written out: the target has no phase that runs before
  `main`. `LazyLock` runs on first use rather than before it, which is a different program whenever
  the work has side effects or two packages' order matters; an explicit init called from `main` is
  faithful but changes the library's API and pushes the ordering obligation onto every caller.
  A name collision caught by the compiler: `kindInit` already existed for a `for` loop's init
  CLAUSE. The two share a source keyword and nothing else, so the declaration kind is
  `package_init`.
  `xid` went from 24 declarations to 25, and its survey briefly showed `uncaptured=1` before the
  deferral landed — the coverage proof reporting a kind nothing answers for, which is what it is
  for.

- The front end had no notion of a BUILD CONFIGURATION, and a Go package is not every `.go` file
  in a directory — it is the file set a configuration selects. The source says so three ways:
  `//go:build`, the legacy `// +build`, and the filename itself (`hostid_linux.go`, `sum_amd64.go`).
  Globbing does not produce a bigger package, it produces a file set no `go build` ever emits.
  One cause, two symptoms, and only one of them was loud. THE LOUD ONE: two files declaring one
  symbol under mutually exclusive constraints are a redeclaration, so the type check fails and the
  package yields no measurement at all — `xxhash`, `uuid` and `xid`, three of the eight surveyed
  corpora, all dead on this. THE QUIET ONE is why this matters: `pkg/errors/go113.go` sits behind
  `//go:build go1.13` and declares `Is`, `As` and `Unwrap`. Nothing collided, so extraction
  SUCCEEDED and those three entered the snapshot as unconditional declarations of the package.
  Right for a recent toolchain, wrong for a configuration that excludes them, and recorded nowhere
  as conditional. That is output that means something different, reached with no error raised.
  The configuration is now DECLARED — `-goos`, `-goarch`, `-go-release`, `-tags` — and answered by
  `go/build.Context`, which is Go's own rule rather than a second implementation of it. Declared
  rather than read from the environment, because a configuration taken from the host makes the
  snapshot digest a property of the machine: one upstream commit would extract to two identities
  and the receipt would call an ordinary re-extraction drift. Release tags are pinned for the same
  reason, and `types.Config.GoVersion` is pinned with them — left unset, go/types checks at
  whatever version compiled the extractor and accepts syntax the declared target cannot build.
  A SECOND selector was the reason the first fix looked like it had failed. `corpusImporter`
  resolves an intra-corpus import by parsing the package itself, and it still globbed — so a
  package reached through an import and the same package reached directly were selected by two
  different rules, and the union is what failed. The symptom reads as "could not import", which
  names a missing dependency and is actually a disagreement between two file lists. Two selectors
  that must agree is the hazard `model.go` names about vocabularies and refuses; there is one now.
  Also fixed, and only visible on real repos: a module whose ROOT directory is a package had the
  unit id `example.com/mod/.`, which is the import path of nothing, so a sibling importing its own
  module root did not resolve. Every fixture package is a subdirectory, so the committed corpus
  could not have shown it; almost every real Go module puts code at its root.
  Proven, not asserted. All seven existing fixtures regenerate BYTE-IDENTICAL — selection is a
  no-op where constraints are absent and decisive where they are present. `corpus-buildtags/` is
  new and is built to fail loudly: `Platform` is declared in both `platform_linux.go` and
  `platform_darwin.go`, so a globbing front end cannot regenerate the artifact the fence reads at
  all. Under the four configurations tried by hand the model tracks Go exactly — linux and darwin
  both carry `Platform` with DIFFERENT digests, go1.12 drops `Recent`, and windows drops
  `Platform` entirely while the package still admits. The fence has teeth: moving the declared
  release from 21 to 12 fails `a_satisfied_release_constraint_admits_its_declarations` by name.
  Green under both build systems, and `engine_digest` moved with the source while the other five
  axes held.
  What this does NOT yet do: the configuration is an input to the extraction and is not RECORDED
  in the snapshot. Two configurations already produce two digests, so identity is sound — but the
  receipt can only say the snapshot axis moved, not whether the operator changed platform or
  upstream changed code. Recording it is a schema change and is the next increment, not this one.

- `ulid` still yields no measurement, and the cause is worth naming rather than working around.
  Its `cmd/ulid` imports `github.com/pborman/getopt/v2`, which is neither in the corpus nor in the
  standard library, so the import cannot resolve — and the extraction fails the WHOLE corpus for
  one command-line tool. That is the same defect shape as the `unsupported`-at-package-scope bug:
  a local problem escalated to whole-corpus rejection, and one package in eight produces nothing.
  The refusal is right and its GRANULARITY is wrong. A package whose imports the corpus cannot
  resolve should be recorded as not admitted, by name, with the missing import named, while the
  rest of the corpus still measures. Left open deliberately: it is a second schema-touching
  decision and belongs beside the build-configuration record rather than bolted on before it.

- A foreign satisfaction is an OBSERVATION about a type, not a declaration of one — and reading it
  as a declaration rejected two whole snapshots the moment one foreign type satisfied two
  interfaces. `os.File` is a reader and a writer; so is `bytes.Buffer`. Ordinary Go, and `xxhash`
  and `ksuid` produced no measurement at all because of it. That is the THIRD time in this lane
  that a local fact escalated to whole-corpus rejection, after `unsupported` at package scope and
  the unresolvable import in `ulid` — the pattern is worth naming, because each instance looked
  like a different bug.
  Go's package scope is a namespace and one name means one thing. That rule is right and is now
  qualified rather than weakened: `NON_BINDING_DECLARATION_KINDS` is a closed set of kinds whose
  `name` is a name the package does not bind, and a foreign satisfaction's name belongs to another
  package entirely. It lives beside `NAMESPACE_KINDS` in the vocabulary rather than as an exemption
  at the call site, so a kind that stops binding has to say so where the reason is written down.
  The deeper half was invisible while the first half was breaking things: the interface being
  satisfied existed only inside an English sentence in `go_node`. The two facts about `bytes.Buffer`
  were therefore indistinguishable to anything but a prose reader, and a rule keyed on what
  satisfies `io.Reader` would have had to parse the message. `interface` is a structured attribute
  now. The sentence stays, because it is what the refusal says.
  `corpus-foreign/` is new and fences all of it: the fixture cannot be admitted at all under the
  old rule, both facts name their interface, and `Sink`/`Source`/`Drive` still bind exactly once
  each — the qualification is narrow, and the namespace is not looser for anything that enters it.

- The ranking was REGENERATED rather than trusted, and it moved. Seven of eight corpora now
  measure; `ulid` remains blocked on the unresolvable import recorded above.
  Coverage: xxhash 61.8%, ksuid 40.9%, uuid 43.3%, xid 38.5%, errors 26.3%, semver 25.9%,
  go-multierror 0.0%. xxhash previously read 69.0% and xid 41.7% — both were measured over the
  union of every build configuration, which is not a program, so the new numbers are corrections
  rather than regressions. A ratchet that only rises is measuring the wrong thing.
  Ranked by PACKAGES blocked: `IfStmt` variants (6), unary `&` (5), transform ownership (5), the
  carried-value failing return (4), `AssignStmt` forms (4), `panic` (4), `unmapped type interface`
  (4), variadic signatures (3), `ArrayType` (3).
  `var` at package scope is no longer at the top. It has folded into "deferred by policy" (5
  packages, 76 declarations), which is the pack declining rather than the engine failing — so the
  top CAPABILITY blocker is now `IfStmt` at six of seven packages, and the largest single number on
  the board is a decision nobody has made rather than a construct nobody has written.

- `if x := f(); cond` was refused on a reason that was not true, and it was the top capability
  blocker on the board — six of seven surveyed packages. The recorded reason said the target has no
  direct form for a binding scoped to a condition. It has exactly one: a block. `{ let x = f(); if
  cond { .. } else { .. } }` scopes `x` to the condition and both branches and nothing after, and
  drops it where the source's scope ends. What is unfaithful is HOISTING to the enclosing scope,
  which the old comment refused correctly and then generalised into refusing the construct itself.
  A refusal reason is a claim, and this one had never been checked.
  The split follows the one the `for` loop already used. The extractor records the init clause as a
  CHILD — the snapshot is a model of the source, and rewriting the shape at extraction would make
  it a model of the target — and the transform decides it becomes a block. That also made the
  refusal granular: an init clause the front end cannot model now refuses by ITS own name instead
  of collapsing the whole `if` into `unsupported`.
  THE FIXTURE FOUND TWO MORE DEFECTS, both of which emit code that does not compile, and neither of
  which any existing corpus could have shown.
  `x := e` never asked whether the body writes the binding again. `bindingFlags` existed and was
  wired only to the `var` path, so every short declaration later assigned emitted an immutable
  binding followed by a write to it. `var` was in the corpus and `:=` was not — and `:=` is the
  form real Go actually uses.
  `RustStmt::LetTuple` had no mutability at ALL. `v, err := f()` followed by a later write to `err`
  is the most common shape in the source language and it emitted `let (v, err) = ...`. Mutability
  is now PER NAME rather than per statement, because the source binds each name independently: it
  is routinely `err` that is written again and `v` that is not, and one flag for the pair would
  have to be the disjunction, making every value binding mutable to serve the failure beside it.
  Proven by the compile proof rather than by reading: six tests, `rustc` on the emitted crate.
  Also worth recording about the instrument: only `xxhash` moved (61.8% → 64.7%) and the other six
  did not budge. That is correct and not a disappointment. The ranking counts declarations that
  NAME a cause, and a declaration blocked by `IfStmt` is usually blocked by four other things too —
  so clearing one cause removes it from every ranking without unblocking a single declaration until
  the last of its causes goes. `IfStmt` is off the board entirely; coverage is a lagging indicator
  of it.
  New top capability blockers, by packages: unary `&` (5), transform ownership (5), the
  carried-value failing return (4), `AssignStmt` forms (4), `panic` (4), `unmapped type interface`
  (4). "deferred by policy" still leads at 5 packages and 76 declarations, and is still a decision
  nobody has made rather than a construct nobody has written.
  One process note, since it cost real time: `port-go-source` PRINTS the emitted crate and compares
  it against the golden; it does not write the golden. Redirecting its output to `/dev/null` and
  then diffing the golden shows nothing changed no matter what the engine did.

- READ-MODIFY-WRITE assignment reached the model as `unsupported` 69 times across the surveyed
  corpora. The refusal said the form "carries a question — read-modify-write — that needs a rule
  rather than a default". For the compound operators both languages share it does not: `x op= y`
  means `x = x op y` in both, evaluates the place expression once in both, and introduces no
  decision the binary operator has not already made. The engine maps binary `+`, `^`, `|` and the
  rest, so refusing their compound forms refused the same decision twice. Real forms in the
  corpora: `^=` 14, `+=` 9, `|=` 6, `*=` 6, `/=` 1, `-=` 1.
  The operator is carried on the assign node rather than given a kind of its own, because `x += y`
  and `x = x + y` differ only by evaluating the place once — which is true in both languages, so a
  second kind would describe a difference neither has. It is NOT desugared in the IR for the same
  reason: rewriting to `target = target op value` evaluates an index or a call inside the place
  twice.
  `&^=` stays refused and is now RECORDED rather than dropped. The extractor used to emit a bare
  `unsupported` naming `AssignStmt`, so the transform could only say "some assignment" — which is
  not a refusal anyone can act on. The operator is carried through and the transform refuses it by
  name: "assignment operator `&^=` has no target form". Parallel assignment (`a, b = b, a`) is
  still refused and is the honest remainder: it evaluates every right-hand side before assigning
  any left, which needs temporaries in a declared order.

- The corpus found a defect the compile proof named within seconds: A PARAMETER THE BODY ASSIGNS TO
  NEEDS `mut`, and nothing recorded that it does. `cannot assign to immutable argument`.
  `mutated` could not carry it. On a parameter that flag already means "the body writes THROUGH
  this pointer" — a claim about the CALLER's value, and what drives the disposition to `&mut T`.
  Rebinding the callee's own copy is the opposite claim: the caller sees nothing. One flag carrying
  both would have demanded an exclusive borrow for every parameter a body happens to reassign. So
  `rebound` is its own fact.
  OWED: on a local `let`, `mutated` answers this same question under the other name. The overlap is
  real and unifying them is deferred rather than forgotten.
  Also found on the way: the METHOD path never called `annotateParameterFacts` at all. The function
  path did and the method path did not, so a method's parameters carried no ownership facts and no
  rebinding — latent for as long as no method parameter was written.

- Unary `&` was refused on a false reason, the same shape as the `if` init clause. The message said
  it "has no direct translation"; the target has both `&` and `*`. What is missing is the
  DESTINATION: `&x` yields a pointer, and which target form it takes is the same ownership decision
  the pack already answers for a `*T` type position — but the answer depends on the position the
  value flows into, and the body translator does not know it.
  Sized before deciding, over 33 `&` sites in seven packages: 11 are `f(&x)`, where the destination
  is the CALLEE's parameter; 7 are `x := &T{..}`; 4 are `return &T{..}`; 3 are `x = &T{..}`; 3 are
  `return &x`; the rest have unsupported operands. EVERY one resolves against a signature the
  engine has already translated. So `&` is blocked on the SIGNATURE TABLE (R6), not on a missing
  rule, and picking a form without one would be the guess this engine exists to refuse. The refusal
  now says exactly that, so the census does not have to be re-derived to learn it.
  This is the second refusal reason in two phases that turned out to be untrue on inspection. A
  refusal reason is a CLAIM, and nothing in the suite checks claims — they are prose. Worth
  treating every remaining one as unverified until read.

- Ranking after the above. Coverage: xxhash 70.6% (was 64.7), ksuid 40.9%, uuid 43.3%, xid 38.5%,
  errors 26.3%, semver 25.9%, go-multierror 0.0%. `AssignStmt` fell from 4 packages to 3, and what
  remains under that name is parallel assignment alone.
  By packages: deferred by policy (5, 76 declarations), unary `&` (5 — now blocked on R6),
  transform ownership (5), the carried-value failing return (4), `panic` (4), unmapped type
  `interface` (4), variadic signatures (3), `AssignStmt`/parallel (3), `ArrayType` (3).

- The `var` deferral was one label over three stacked gaps. `init` indexing closed the third
  earlier; this closes the other two, and what they revealed changes the shape of the decision.
  THE DATA GAP. A `const` recorded its value and a `var` recorded NOTHING — type and documentation
  only. So all 67 package variables across the surveyed corpora reached the engine as names with no
  content, and no rule could have emitted anything for them whatever the policy turned out to be.
  The initialiser is now a CHILD EXPRESSION rather than a source-text attribute like a constant's:
  a constant's value is a literal the target can re-parse, a variable's is arbitrary code
  (`errors.New(..)`, a call into another package) and flattening it to text would hand the
  transform a string no rule can inspect and no resolver can qualify.
  ABSENT means the source wrote no initialiser and the zero value applies. That is a different fact
  from one the front end could not attribute, so `var a, b = f()` records an `unsupported` child
  instead of nothing — otherwise the pair would be indistinguishable from `var a, b T`. Measured
  first: no package in the seven writes that shape, which is exactly why it must be recorded rather
  than assumed away.
  THE PROVABILITY GAP. Nothing computed whether a package variable is ever written, so every one of
  them was deferred on the hardest case. Package-WIDE analysis, because a package variable is
  visible to every function in it. The result: 45 of 67 are NEVER WRITTEN anywhere in their own
  package. The deferral's synchronization argument — `static` is immutable, `static mut` is unsafe,
  `OnceLock`/`Mutex` each pick a policy the source never stated — is true and bites only for a
  variable something assigns to, so it was being applied to two thirds of the variables that do not
  have the problem. The pack's reason now says so, and says what remains genuinely undecided: the
  FORM. `static X: T = ..` needs a const-evaluable initialiser, which `errors.New("..")` is not,
  and `LazyLock` runs the initialiser on first use rather than before it — the same
  when-does-the-work-happen question that defers `package_init`. Deferred until that form is
  chosen, not until more is observed.

- WHAT THE ENGINE EMITS HAS TO BE SELF-CONTAINED, and nothing said so. The compile proof caught it
  the moment a fixture put a function and a deferred package variable in one package: `cannot find
  value `counter` in this scope`. The engine translated a body referring to a declaration it had
  itself declined to emit and produced a crate with a dangling name. Latent everywhere until now
  only because no emitted corpus package read a package variable — it is the general shape of the
  defect, not a property of the fixture: ANY deferral creates it.
  Refused PER DECLARATION, keyed on the pack's own deferred set so the refusal disappears by itself
  the day `var` stops being deferred. The alternative — a whole-plan proof — would fail an entire
  package over one function, which is the escalation this lane has already had to undo three times.
  THE RATCHET WENT DOWN AND THAT IS THE POINT: ksuid 40.9→36.6, uuid 43.3→36.1, xid 38.5→30.8,
  semver 25.9→20.7. Those declarations were being counted as translated while their output would
  not have compiled. Second time this lane has had coverage fall because the engine stopped
  claiming something it could not do; a ratchet that only rises is measuring the wrong thing.

- A source string literal is a `string` VALUE and the pack maps `string` to an owned `String`, so
  `fn describe() -> String { "globals" }` does not compile. Caught by the compile proof.
  THE FIRST FIX WAS WORSE THAN THE BUG. Owning every string literal compiles and produces
  `s == "".to_owned()` and `Box::<dyn Error>::from("empty".to_owned())` — output no reviewer would
  accept, and two existing tests said so immediately. That is the goal's own bar working as a
  check: the emitted Rust has to read as hand-written, and "it compiles" is not that bar.
  Narrowed to where the destination is actually KNOWN: a function whose single result resolves to
  the owned target, returning a bare literal. The signature is in hand at `Body::new`, so that one
  is answered. Everywhere else the destination is a parameter or a comparison operand — the same
  question unary `&` is blocked on, needing the signature table rather than a guess. A borrowed
  literal in a borrowed position was already right; only the owned positions were ever wrong.
  The owning SPELLING (`.to_owned()` over `.to_string()`) is an idiom decision sitting in code. It
  belongs in pack data with the rest of the idiom rules at R7 — recorded rather than left implicit.

- THE SIGNATURE TABLE (R6). The body translator knows what an expression IS and not where it is
  going, and several translations need the second. Measured over the seven corpora: of 33 `&` sites,
  11 are `f(&x)` — the largest single group — and every one of those destinations is a signature the
  engine has already translated. The answer was always available; it had nowhere to be asked from.
  One translation of every free function in the model, keyed by the identity a call already records
  (`<unit_id>.<Name>`, which is exactly what the extractor writes). Built with NO construction
  overrides: the pack's one override maps `string` to `&str` for `rust_const`, and a function
  parameter is never inside a constant.
  A signature the engine cannot translate is OMITTED rather than fatal, and `None` from the table is
  "cannot say" rather than "no conversion needed" — the difference matters, because reading the
  second as the first is how a missing answer becomes a silent wrong one.
  What it does NOT answer, and refuses by name: a METHOD (52 of the calls in uuid — a method's key
  is its receiver type, not a path) and a FOREIGN function (`fmt.Sprintf`), whose signature is not
  in the snapshot at all.
  ARGUMENT CONSTRUCTION IS THE SAME DECISION SEEN FROM THE OTHER END. A disposition already said
  what `*T` becomes in a parameter; it now also says what `&x` becomes when handed to one. One id,
  one rule, one reason — rather than a second table that could disagree with the first. Found by the
  id the parameter RECORDED, never by matching the spelling that decision produced.
  Declared as STRUCTURE, not as a text template. `Some(Box::new({0}))` would have to be substituted
  into and re-parsed, which is the string-splicing the typed IR exists to replace; the pack says
  `borrow` or `wrap: [paths]` and the transform builds `RustExpr::Reference` / `RustExpr::Call`.
  Proven end to end by `corpus/handoff`: `bump(&mut c)` and `read(&c)` each match their parameter,
  and `let mut c` follows because taking the address rebinds it.
  Coverage on the real corpora did NOT move, and that is honest rather than disappointing: their `&`
  sites are `x := &T{..}` (7), `return &T{..}` (4), `x = &T{..}` (3), `return &x` (3) — 17 of 33 —
  whose destinations are a local's inferred type and a function's result. Both are known to the
  engine and neither reaches the expression walk yet. The refusal now names the position instead of
  claiming the target has no form for `&`.

- `copy_types` held `bool`, `int`, `float64` and nothing else, so every other numeric width emitted
  `.clone()` on a Copy value. Not a decision — an incomplete list. Output that compiles and that no
  reviewer would accept, which is the bar this engine is held to and not the same bar as compiling.
  All sixteen source scalars now, `string` deliberately absent: it maps to an owned `String`, which
  is not Copy, so reading one really does clone. The four remaining clones in the golden are all
  `String` fields, which is correct.

- `function_map` was a bare string→string map, so three translations sat in the pack with nobody's
  name on them. `errors.New` becoming a boxed trait object and `len` gaining a cast are DECISIONS,
  and the pack's own discipline is that a decision carries a reason travelling in the digest and
  therefore in the receipt. Entries are objects now and both reasons are written out.
  The upgrade is what `panic` needed anyway, because that mapping is CONDITIONAL and no mapping
  could say so. Go's `panic(v)` aborts carrying `v`; Rust's `panic!` unwinds carrying a formatted
  string. Where `v` is a STRING LITERAL the two are the same abort with the same message and the
  same payload type and nothing is lost. Where `v` is an error or an arbitrary value the payload
  TYPE is lost, and a caller that recovers and type-asserts on it sees a different program — which
  is precisely the failure this engine exists to prevent, so it refuses by name and says which
  shape it found.
  The condition is pack data and its vocabulary is CLOSED: a shape the engine has never heard of
  refuses rather than reading as "no condition", because a condition nobody checks is not there.
  ON EMITTING A PANIC AT ALL, since this repository's Rust does not. That bar governs the ENGINE's
  own source, which contains no `panic!` and no `unwrap()` — checked, not assumed. The emitted
  crate's semantics come from upstream: a source function that aborts must port to a target
  function that aborts, and returning a `Result` where the source panics would be a different
  program. Recorded as a declared exemption with the reason rather than resolved silently in
  either direction.
  Sized by census as the method requires: `docs/programs/k8s-port/census/defer-panic-recover.md`
  puts the two string-literal invariant shapes at 38.2% and 21.0% of Kubernetes panic sites, 59%
  together. In the seven surveyed corpora the mix is different — 6 `panic(<ident>)`, 3 string
  literals, 2 `panic(fmt.Errorf(..))`, 1 `fmt.Sprintf`, 1 binary, 1 other — which is worth knowing
  before assuming the census's shape holds off-corpus.
  `panic` fell from 4 packages to 3. Coverage did not move, because the packages with literal
  panics carry other blockers too.

- THE BLIND REVIEW WAS RUN, which is the goal's own bar and had never been tested. The emitted
  crate was handed to a reviewer told it was a colleague's hand-written Rust submitted for merge.
  Verdict: DO NOT MERGE. That is the honest state and it is worth having in writing.
  The findings split three ways and the split matters, because acting on all of them equally would
  be wrong:
  ENGINE DEFECTS, universal and real. Integer overflow (below). Inherent methods shadowing
  identically-named trait methods, which works only because inherent methods win path resolution
  and becomes infinite recursion the moment the inherent one is deleted — set four times in the
  golden. No `derive` anywhere, so nothing emitted can be `{:?}`-printed or compared. Go doc
  comments copied verbatim into rustdoc, carrying Go's convention (`/// Mix folds…` repeats the
  item name, which rustdoc does not) and, worse, carrying ENGINE-INTERNAL prose into public API
  documentation — "the target has to invent the success value here", "refuses by name and this
  package's job is to prove type resolution". Five compiler warnings on a clean build, one of
  which (`unused variable: next`) is a METHOD parameter, which the `unread` flag covers for
  functions and not for methods. `if s == ""` for `is_empty`. `self.total = self.total + n` for
  `+=`. Stray `};` after block statements. `(1) as i64` — redundant parens and a redundant cast.
  Fully-qualified `crate::shapes::Point` inline instead of a `use`.
  FIXTURE ARTIFACTS, not engine defects. Unconstructible structs, stub functions whose docs claim
  behaviour they do not have, module names that are construct families rather than domains, dead
  exports, two unrelated `Counter` types. Every one is a faithful port of a fixture written to
  exercise a construct. They say the CORPUS is not a library, which is true and was never the
  claim — and they are exactly why a corpus committed beside the engine cannot be the measure.
  FAITHFUL-PORT TENSIONS, which must be declared rather than fixed. `panic` on an ordinary input
  (already declared). `s.len()` reporting bytes (Go's `len` IS bytes — the port is right and the
  DOC is Go's). A no-op `rename` (the source's method is empty). `Box<dyn Error>` not being
  `Send + Sync` is a genuine pack decision worth revisiting and is not a port defect.

- SIGNED ARITHMETIC WRAPS IN THE SOURCE AND PANICS IN THE TARGET, and the engine emitted the plain
  operator. `acc *= 3` in a mixing loop overflows `i64` at about forty elements: defined wrapping
  in the source, a debug panic and a release wrap in the target — one source program became two
  target programs, neither of which is it. Output that compiles and means something different,
  which is the failure this engine exists to prevent, and NOTHING IN THE SUITE WAS LOOKING FOR IT.
  It took a reader who did not know the code was generated.
  The result TYPE decides and is not recoverable from the operator — `+` on floats, on strings and
  on integers are three rules — so the front end now records it on both the binary expression and
  the compound assignment. Another instance of the standing pattern: go/types knew, the extractor
  dropped it.
  The pack declares the spelling with its cost written out. `wrapping_*` is the target's spelling
  of exactly the source's rule and behaves identically in both profiles; the cost is that every
  arithmetic operation is spelled that way including the overwhelming majority that never overflow,
  and a reader without the provenance will ask why. Accepted, because a verbose port is a port and
  a port that means something else is not.
  Bitwise `^=` and `|=` stay compound, which is the decision being precise rather than broad: they
  cannot overflow, so the pack's operator table does not list them.
  THE COMPOUND FORM EXPANDS, because the target has no `wrapping_mul_assign`: `x *= 3` becomes
  `x = x.wrapping_mul(3)`, which reads the place twice where the source read it once. Sound only
  where reading twice equals reading once — a path, a field of one, an index by one — and refused
  by name anywhere else rather than calling something twice that the source called once.
  SHIFTS ARE LEFT OUT DELIBERATELY. The source defines `x << s` at or beyond the operand width as
  zero; the target masks in release and panics in debug, and `wrapping_shl` masks rather than
  zeroing. A different rule needing its own form, and `census/` sizes no numeric family at all —
  which the standing brief already says, and this is the first time it has cost something.

- The source's DOC CONVENTION is not the target's, and the engine copied it verbatim. The source
  requires a doc comment to open with the identifier it documents; the target requires that it does
  not and writes in the third person. Every one of the forty-odd emitted doc comments carried the
  source's form, and the blind reviewer named it the loudest single signal that a Rust developer
  had not written the code.
  Mechanical and BOUNDED, which is the only reason rewriting somebody's prose is defensible: the
  leading word must equal the declaration's own source name EXACTLY, and a copula immediately after
  it is dropped with it so `ID is an alias` becomes `An alias` rather than the ungrammatical `Is an
  alias`. A doc opening any other way is returned untouched — its author already chose an opening,
  and this has no business rewording prose it was not asked about. The bound is in the pack with
  the reason, not in the code.
  `Mix folds the values` → `Folds the values`; `Add returns the sum` → `Returns the sum`;
  `DefaultName is the fallback identity` → `The fallback identity`.

- STILL OWED from the blind review, and named rather than half-done:
  A trait impl forwards to the inherent method of the same name — `Driver::describe(self)` inside
  `impl Describer for Driver`. Correct by construction today, because the engine always emits both
  and inherent methods win path resolution; a trap if the inherent one is ever removed. Removing
  the DUPLICATION is the real fix and it needs `use` emission, because dropping the inherent method
  means an intra-crate caller must have the trait in scope and the engine currently emits
  fully-qualified paths instead of imports. Reversing the forwarding direction alone would remove
  the hazard and leave a duplicate a reviewer would still question, so it is one change or none.
  No `derive` on any emitted type, so nothing can be `{:?}`-printed or compared. Universal and
  pack-shaped: which derives a ported struct earns is a decision about what the source guarantees,
  and `Copy` in particular cannot be assumed.
  A METHOD parameter the body never reads still warns — the `unread` flag reaches functions and not
  methods, because the two build their parameters through different paths and only one was wired.
  `if s == ""` for `is_empty`, `self.total = self.total + n` for `+=`, stray `};` after block
  statements, `(1) as i64`, and fully-qualified paths where a `use` belongs. All idiom rules, all
  R7, and all now measured rather than guessed at.

- NOTHING THE ENGINE EMITTED DERIVED ANYTHING, so no ported type could be printed, compared or
  defaulted. The blind reviewer called it the single loudest signal a Rust developer had not
  written the code, and it is a capability gap as much as a style one: without `Debug` nothing
  emitted can appear in an assertion or satisfy the bound half the ecosystem asks for.
  WHICH derives a type earns is a claim about what the SOURCE guarantees, so each is pack data with
  its own reason and its own blocking set:
  `Debug` — every source value can be printed by the source's own formatting verbs, so a ported
  type that cannot be printed has lost something the source had.
  `Clone` — the source copies a struct on assignment, so every source struct is duplicable. Clone
  is the target's name for that, made explicit at each use rather than implicit at every one.
  `Default` — EVERY source type has a zero value and `var x T` produces it, so a default is a fact
  about the source rather than an invention.
  `PartialEq` — and this one is the interesting case. The source compares structs with `==` exactly
  when no field is a slice, a map or a function, and those are precisely the fields whose target
  counterpart is not comparable either. The two languages agree, so it is derived rather than
  guessed: a type the source could compare, the port can compare.
  THE BLOCKING SET is what makes this safe rather than hopeful. Only kinds the engine emits no type
  for can block — a trait object, a bare interface, a channel, a function, an unsupported shape.
  A field naming another emitted struct cannot block anything, because every emitted struct gets
  the same list, so intra-corpus references are satisfied by construction rather than by ordering
  the emission. Checked through the whole type TREE and not just its root: a `Vec<Box<dyn Error>>`
  is a slice whose element blocks, and looking only at `slice` would miss it.
  `Copy` IS DELIBERATELY ABSENT, and the reason matters. The source copies on assignment and the
  target does not, so `Copy` looks like the faithful mapping and is not: it is available only where
  every field is Copy, and a struct that gains it changes how every later assignment behaves. That
  is a decision about the emitted API rather than a fact about the source.
  Compiler warnings on the emitted crate went 5 → 3 across this and the `unread` fix. The three
  that remain — a dead initializer the source requires, an unused private function, an unread field
  — are all cases where the source tolerates what the target warns about, and all three are fixture
  artifacts rather than shapes real packages have.

- R7 OPENED, with the licensing contract honoured from the first rule rather than retrofitted.
  `specs/k8s-port/licensing.json` rejects a rust-skills-derived rule without `seed_source`,
  `seed_license` and `seed_commit`, so the idiom table carries all three as REQUIRED fields — a
  rule whose derivation cannot be re-checked is a rule nobody can audit. The seed corpus is real
  and was read rather than assumed: 384 rules, MIT, commit a28144ccd.
  An idiom is a different KIND of rule from everything else in the pack and is kept apart for that
  reason. Every other decision changes what the emitted program does or refuses to do; an idiom
  changes only how it READS, and an idiom that alters meaning is not an idiom but a bug.
  First rule: `x == ""` becomes `x.is_empty()`, seeded from `rules/lint-warn-style.md` because
  `comparison_to_empty` is a `clippy::style` lint and a comment a reviewer would make on
  hand-written code is a defect in code held to that bar. Exactly equivalent — both true precisely
  when the value has length zero. Either operand may be the literal, since the source permits
  `"" == x`; two literals is a comparison of constants and not this shape at all.

- Three emission defects the reviewer named, all in the LOWERING and none needing data.
  `(1) as i64` — a cast's operand was bracketed unconditionally because `as` binds tighter than
  every binary operator, so `a + b as u8` would cast `b` alone. True for a compound operand and
  pointless for one that cannot reassociate; a literal, a path, a field, an index and a call all
  bind tighter than `as` already.
  `1 as i64` — and then the cast itself was wrong for a literal. `int64(1)` in the source is the
  value one at that width, and `1i64` says so where `1 as i64` says a conversion happened. Only for
  an integer literal reaching an integer type: a float target or a signed operand is a conversion
  that can change the value and must keep saying so.
  `};` — an `if` in statement position carried a trailing semicolon. Not merely noise: it makes the
  block an expression statement whose value is discarded, which is how an emitted `if` came to sit
  under a binding rustc then reported as never read.
  MEASURED: clippy on the emitted crate went from 14 warnings at the blind review to 2, and rustc
  from 5 to 3. The two clippy ones that remain — a dead initializer the source requires and an
  unused private function — are both cases where the source tolerates what the target warns about,
  and both are fixture artifacts rather than shapes real packages have.

- THE PORTED ERROR TYPE COULD NOT CROSS A THREAD. Every fallible declaration in every ported
  package returned `Box<dyn std::error::Error>`, which is neither `Send` nor `Sync` — so no caller
  could propagate a ported failure out of a thread or into any async runtime. The reviewer named it
  as breaking the crate for concurrent use and was right: the bound was MISSING, not declined.
  `Send + Sync` now. The source's own error values are ordinary data and satisfy both; a source
  error that did not would not have been shareable across the source's own goroutines either, so
  this narrows nothing the source had.
  The convention also carried NO REASON, which is the same discipline gap `function_map` had — and
  worse here, because this is the single most load-bearing type decision the pack makes: it appears
  in the signature of every fallible declaration in every package. The reason is a required field
  now and says why boxed (a failure outlives the call that produced it, so a reference would need a
  lifetime the caller cannot supply) as well as why the bounds.
  The turbofish stays explicit rather than `.into()`, and that is a considered trade the reviewer
  would still flag: `.into()` infers only where the destination type is known, which is true inside
  `Err(..)` and not inside a `let`. The mapping fires in any position, so it spells the type.
  Both failure fences caught the change and were updated to the new spelling — the properties they
  assert, that a fallible signature becomes a `Result` and that a standard-library call is answered
  by the pack, are unchanged.

- THE `var` DIAGNOSIS WAS WRONG, and measuring the types said so. The deferral had been treated as
  a TIMING problem — `LazyLock` runs on first use where the source runs before `main` — and the
  next step looked like proving initializer purity. Then the never-written variables were counted
  by type: 17 of 45 are `error`, the single largest group, and those cannot be a target static at
  all. Returning a package-level error moves out of the static, and `Box<dyn Error>` is not
  duplicable. That is an OWNERSHIP mismatch, not a timing one, and no purity analysis would have
  touched it. 15 are named structs, which now derive `Clone` and `Default` and are viable; 7 are
  arrays, 2 bool, 2 slices, 1 func.
  The idiomatic target form for an error sentinel is a distinct type implementing the error trait,
  not a boxed trait object in a static — a transformation, not a spelling. Recorded rather than
  attempted: it is the shape 17 declarations need and it belongs with the error-model decision, not
  bolted onto the `var` form.

- SEQUENCE LITERALS reached the model as `unsupported` 26 times. Measured before deciding, which
  changed what got built: the positional-STRUCT literal that the drain had flagged as low-hanging
  turned out to be none of the 26 — every one is a slice, array or map literal. The positional fix
  landed anyway because it is correct and universal (the field order is a fact go/types holds right
  there, so naming the fields is a proof rather than a hope), it just was not what these corpora
  needed.
  By shape: 17 arrays, 3 slices, 1 map. And by fill: 11 of the arrays are EMPTY, the single largest
  group — `[20]byte{}`, which is not an empty array but twenty zero bytes. So an empty sequence
  literal is answered by the type's ZERO VALUE, which the engine already had machinery for; any
  other answer would need it to invent a length it already has.
  `zero_values` held four entries — bool, int, float64, string — so every other scalar had no zero
  and any construction needing one refused. The same incomplete-list shape `copy_types` had, found
  the same way. Composite zeros are keyed by KIND because an unnamed type has no name, and `array`
  carries a template because its zero needs the ELEMENT's zero and the length.
  A LATENT TRAP surfaced on the way and it was silent: `table_key` fell through to a type's `name`,
  but an array's `name` is its LENGTH. `[4]int64` looked up the key `4`, so every pack table missed
  — and a miss is indistinguishable from a type the pack declines to answer for, which means the
  wrong answer read as a policy decision nobody made. Arrays are keyed by kind now, with the reason
  written where the key is computed.
  A map literal is deliberately refused: the source's map has no order and the target's ordered map
  imposes one, so the entry order becomes observable where it was not. A decision needing its own
  reason rather than a row in a table.
  ksuid 36.6% → 38.7%.

- FOLLOWING CALLS, which the `unproven_owned` disposition's own reason had been asking for. Passing
  a pointer to any call made every fact about it unproven, so most methods carried `effect_unknown`
  and nothing else — the analysis reporting that a call happened rather than reporting anything
  about the pointer. `unproven_owned` has no receiver form, so each of those refused.
  Three ways a pointer reaches a call, and only one is a positional argument. The ARGUMENT case is
  answered by what the callee does to the parameter it lands in. The RECEIVER case is not in `Args`
  at all — `s.helper()` reached the argument loop with nothing rooted at `s`, so it was silently
  treated as though the pointer had not been passed anywhere. And a BUILTIN or a CONVERSION has no
  body to read: what `len` does is a property of the source language rather than a decision, and a
  conversion reads its operand and does nothing else — the same "a call is not always a call"
  confusion that once emitted a conversion as a call to a function with no name, met from the other
  side. The WRITING builtins are deliberately absent from the read-only set and keep leaving the
  facts unproven.
  Recursion is guarded by the objects already on the stack, and a cycle yields `effect_unknown`
  rather than a fixpoint: the honest answer for a pointer whose fate depends on itself is that
  nothing was proven, and iterating would claim more than this pass can defend.
  98 methods now carry clean facts and 21 an exclusive borrow, where before almost all were
  unproven. `errors` 26.3% → 31.6%.

- THE SECOND BLIND REVIEW was run against the changed output, and it is more rigorous than the
  first: the reviewer compiled the crate AND probed it from an external consumer, so every claim it
  makes was executed rather than inferred. Verdict is still DO NOT MERGE.
  IT FOUND A DEFECT I KNOWINGLY LEFT. The wrapping-arithmetic phase excluded shifts and said so —
  the source defines a shift at or beyond the operand width as ZERO and panics on a negative count,
  the target panics on the first in debug and masks the count in release. Three behaviours where
  the source has two, none matching. But excluding them from the POLICY left them emitting the
  plain operator, so `shift(n, by)` aborts for `by >= 64` and for any negative `by` where the
  source returns zero and panics respectively. A gap that emits is not a gap, it is a defect.
  Refused by name now until the pack declares a form: `checked_shl(..).unwrap_or(0)` is the zero
  half and says nothing about the negative half. Second time `census/` sizing no numeric family has
  cost something.
  IT WAS ALSO RIGHT ABOUT OVER-APPLICATION. `n.wrapping_div(2)` cannot wrap: integer division
  overflows in exactly one case, the minimum value over negative one, so a literal divisor that is
  not `-1` cannot reach it. The wrapping form there carried no rule the target lacks, which reads
  as mechanical rather than reasoned — and the point of spelling arithmetic that way is to carry a
  rule, so where the target already agrees the spelling says nothing. A negative literal arrives as
  a unary minus rather than a literal and keeps the wrapping form, which is the conservative
  direction.
  MOST OF THE REST IS THE CORPUS, and the split is worth keeping straight. `Point` unconstructible,
  `lookup` ignoring its table, `Point::shift` dropping a field, `Tag::rename` doing nothing: every
  one is a faithful port of a fixture written to exercise a construct, and the Go source has the
  same property. They say the corpus is not a library, which is true and was never the claim.
  ONE OF THEM IS FAIR AND IS OURS: the emitted rustdoc narrates an internal exercise, because the
  corpus's own doc comments are written for readers of the engine. The corpus is a port INPUT and
  its prose becomes public API documentation, so fixture docs need writing as API docs — which is
  not hand-tuning output, it is fixing the input.

- THE CORPUS WAS BEING REVIEWED AS A LIBRARY, and five of the second review's seven blocking
  findings are that. `Point` unconstructible, `lookup` ignoring its table, `Point::Shift` dropping a
  field, `Tag::Rename` doing nothing, the rustdoc narrating an internal exercise: every one is a
  faithful port of a fixture written to exercise a construct. The reviewer is not wrong — a
  consumer really cannot use that crate — and the fixture really was not a library.
  The corpus is a port INPUT. Fixing it is not hand-tuning output, and until it is what a real
  package looks like the review bar cannot be met however good the engine gets. So:
  CONSTRUCTORS where a type has unexported fields. `NewPoint`, `NewLabel`, `NewTag` — a real
  package has them and the fixture had none, which is what made every emitted struct inert.
  `Shift` keeps the label it used to drop.
  `Lookup` was a stub whose doc claimed it looked something up; the map was in the signature to
  prove type resolution and indexing was not translatable. It is `Size` now and calls `len` on the
  map, which is honest, still proves the map type, and does something.
  `Tag.Rename` was a no-op behind a mutating name — a lie in the API. Its PURPOSE was to prove the
  trait receiver is a union over implementors, which needs one implementor that does not mutate. So
  the method is `Refresh` now, where doing nothing is correct for a fixed tag: same proof, no lie.
  `go vet` caught a self-assignment in the first attempt at that, which is the reason to run it.
  THE ENGINE RATIONALE MOVED OUT OF THE DOC POSITION. The source separates a doc comment from a
  free one with a blank line, so the first paragraph stays and the rest moves above the gap — same
  words, in the position that says who they are for. Every emitted doc is one API sentence now, and
  a grep for `fixture|corpus|census|prove|refus` across them returns nothing.

- READING A VALUE MOVES IT in the target and COPIES it in the source, and only field reads knew.
  The compile proof caught it the moment a constructor read one binding twice:
  `Label{prefix: prefix, text: prefix}` is two copies in the source and a use after move in the
  target. `self.label.clone()` already existed for a field; a plain identifier read had nothing.
  The FIRST attempt was too coarse and the pipeline said so immediately: cloning every non-copying
  read broke `len(values)`, which only borrows. A read is not automatically a move, and which
  ARGUMENT positions take ownership is a signature-table question — but a struct literal's field
  always does. Narrowed to there.
  Counted rather than assumed, and split across the seam: the front end counts the reads because
  that is a fact about the source, and the pack answers whether the type copies because that is a
  fact about the target. A binding read ONCE is moved, which is both correct and what someone would
  write. It still clones once more than the minimum, at the final read; removing that needs
  liveness rather than counting, and is recorded rather than guessed.

- FIELD-INIT SHORTHAND, seeded from the same rust-skills rule with its provenance. The source has
  no shorthand and always writes `Tag{text: text}`, so every constructor that passes a parameter
  into the field it names emitted what `clippy::style` calls a redundant field name. Defined as the
  long form, so it changes the spelling and not the program.
  Clippy on the emitted crate: the new constructors took it 5 → 7, and this took it to 3. The three
  that remain are a dead initializer the source requires, an unused private function, and one more
  — all cases where the source tolerates what the target warns about.

- A VARIADIC SIGNATURE NEEDED NO DECISION, and had been refused for six declarations across three
  packages. The source records `func f(args ...T)` with its last parameter typed `[]T` — that is
  what `args` IS inside the function, go/types says so, and the snapshot has carried it all along.
  So the signature translates through the ordinary slice rule with nothing new to decide.
  What DOES need a decision is the CALL: `f(a, b, c)` passes three arguments to a target signature
  with two parameters, and the trailing ones have to be collected. Which sequence form, and what
  `f(xs...)` does when the caller forwards a slice it already has, are both undecided — so the call
  refuses by name, where it happens rather than one level up. A package that DECLARES a variadic
  function now ports it; only one that CALLS a variadic function is held back.
  The refusal reads the SIGNATURE TABLE, which is the third use of it and the first where it
  answers a question about the callee rather than about a destination.
  uuid 36.1 → 35.1 and xid 30.8 → 26.9: those declarations were reaching the emit while calling a
  function whose target signature takes a slice, which would not have compiled. Coverage falling
  because a refusal became reachable is the same correction the dependency refusal made.

- THE THIRD BLIND REVIEW. Still DO NOT MERGE, and the composition has moved: the first was mostly
  fixture artifacts, this one is mostly engine rules, which is the direction that matters.
  WHAT IT FOUND THAT IS REAL AND NEW: `size(table: BTreeMap<..>)` takes the caller's map BY VALUE
  and drops it. That is not a style point — Go's map is a REFERENCE type, so a map parameter shares
  the caller's map and a mutation through it is visible to the caller. Emitting an owned parameter
  both consumes the caller's value and loses the sharing, which is output that means something
  different. Slices are the same shape: passing `[]T` copies the header and shares the backing.
  This is the ownership-disposition machinery applied to a type kind it has never been applied to,
  and it is the largest correctness item outstanding.
  WHAT IT CONFIRMED FOR THE THIRD TIME: the inherent method beside the trait impl that forwards to
  it. Deleting the inherent one silently turns the forward into infinite recursion — a stack
  overflow introduced by REMOVING code. Named a blocker in all three reviews. Still owed, still on
  `use` emission, and the repetition is the argument for stopping to do it.
  WHAT IT SHARPENED: `Label { prefix: prefix.clone(), text: prefix.clone() }` clones once too
  often. Within a single literal the LAST read of a binding can move, provided the body does not
  read it again afterwards — which is a count comparison rather than liveness, and the front end
  already counts. Recording the count instead of a boolean is the fix.
  WHAT IT PUSHED BACK ON AND I AM HOLDING: `wrapping_*` on `add` under a doc that says "returns the
  sum". Raised in two reviews now. The doc is the SOURCE's doc and the source's `add` wraps too, so
  the port is faithful and the mismatch is inherited rather than introduced. Held, and recorded as
  held rather than as unnoticed.

- A MAP OR SLICE PARAMETER IS A REFERENCE IN THE SOURCE, AND WAS EMITTED OWNED. The third review
  found it by probing the emitted crate from a consumer: `size(table: BTreeMap<..>)` takes the
  caller's map by value and drops it, so `size(my_table)` loses `my_table`. That is not a style
  point. The source's map is a REFERENCE type — a map parameter shares the caller's map and a
  mutation through it is visible to the caller — so an owned parameter both consumes the caller's
  value and loses the sharing. Output that means something different.
  A slice is the same shape from the other end: passing `[]T` copies the header and SHARES the
  backing array, so writing an element is visible to the caller while re-slicing is not.
  So a reference parameter is the ownership question a pointer parameter is, decided by the SAME
  rules on the SAME observed facts. Only the FORM differs, which is why each disposition gained an
  optional reference form rather than a second table carrying a second copy of the fact matching: a
  pointer's owned form wraps a pointee in `Option<Box<..>>` because the source's pointer can be nil
  and the pointee needs an owner, and none of that is true of a sequence that is already owned.
  A disposition with NO reference form refuses, which is the honest answer for one that escapes:
  what an escaping sequence becomes has not been decided, and a borrow would need a lifetime the
  caller cannot supply. `Widths(counts []int) []int { return counts }` is exactly that shape and
  moved to the refusal corpus.
  COVERAGE FELL AGAIN AND IT IS THE FOURTH TIME: xxhash 70.6 → 61.8, ksuid 38.7 → 35.5, uuid
  35.1 → 32.0. Eleven declarations newly refuse, and every one of them was previously counted as
  translated while emitting a parameter that consumed the caller's value. `transform ownership` is
  now the top cause at 7 packages and 17 declarations, which is the engine having moved a whole
  class of silent wrongness into the open where it can be answered.
  Still owed on this: the emitted form is `&Vec<T>` where `&[T]` is what a reader expects — clippy
  calls that `ptr_arg`. An idiom rather than a correctness question, and it needs the reference
  form to be per-KIND rather than one template for both.

- THE INHERENT METHOD BESIDE THE TRAIT IMPL IS GONE, after being named a blocker in all three
  blind reviews and deferred three times. The pair —

      impl Driver { pub fn describe(&self) -> String { self.label.clone() } }
      impl Describer for Driver { fn describe(&self) -> String { Driver::describe(self) } }

  compiles only because an inherent method wins path resolution. Delete the inherent one and the
  forward silently rebinds to the trait method and recurses forever: a stack overflow introduced by
  REMOVING code. And no Rust developer writes the pair — the body belongs in the trait impl, once.
  MEASURED BEFORE DOING IT, which is what unstuck it. The reason for deferring three times was that
  dropping the inherent method needs the trait in scope at every call site and the engine emits no
  `use` declarations. So the emitted crate was checked: the ONLY call on a concrete receiver to a
  trait-declared method is `self.engine.run()`, whose trait is declared in the same module and is
  therefore already in scope. The blocker was real and its cost was not; three deferrals cost more
  than the check would have.
  SIGNATURE from the trait's method, BODY from the type's own, because they answer different
  questions: the trait fixes one receiver for every implementor, and the body is what this
  implementor does. A body written under `&self` typechecks under `&mut self`, which is the
  direction the receiver union can move it.
  A PROMOTED method has no body of its own — what it does is forward through the embedded field —
  so the trait impl builds that forward directly rather than delegating to an inherent twin. That
  is the shape where the two bodies would have been IDENTICAL, which makes the shadowing easier to
  miss rather than harder.
  A method satisfying no interface keeps its inherent impl, because there is no trait to put it in.
  That is most methods.
  WHAT TO WATCH: a cross-module call on a concrete receiver to a trait method now needs the trait
  imported, and the engine emits no `use`. The compile proof is what will catch it, and `use`
  emission is R3's.

- A BORROWED SEQUENCE IS A SLICE, not a borrow of the owned container. `&[T]` accepts every
  `&Vec<T>` and also an array, a boxed slice and a subrange, so it takes strictly more callers while
  promising strictly less — which is why `clippy::style` flags the container form. Nothing about the
  program changes, and the source's slice was never an owned container in the first place.
  Composed STRUCTURALLY rather than by rewriting the container's spelling: the disposition's borrow
  template substitutes `[element]` where it would have substituted the resolved container, so a
  re-spelled container cannot silently stop matching something nobody is matching on. A map borrows
  as itself, because it has no unsized view.
  Seeded from the same rust-skills rule with its provenance, which is the third idiom to carry one.

- THE LAST READ OF A BINDING CAN MOVE, and was being cloned anyway.
  `Label{prefix: prefix, text: prefix}` needs ONE clone: the first read must copy because a second
  follows, and the second can take the value because nothing follows it. Both were cloned, which
  compiles and is exactly the needless allocation the review flags.
  Knowing which read is last on every path is LIVENESS. Knowing it inside one composite literal is
  COUNTING: if the body reads a binding exactly as many times as this literal does, the literal
  holds every read and its final one is the last. The front end already counted — it was throwing
  the count away and keeping a boolean — so the count is recorded now and the transform compares.
  Deliberately narrow, and the narrowness is the honesty: where the body reads the binding again
  AFTER the literal, every read in the literal still clones, because one of them is not the last
  and this cannot say which.
  Both of the two items named as owed at the end of the previous phase are now closed. What remains
  from the third review is the `wrapping_*` default, which is held with its reason, and the error
  type, which is the error-model decision rather than a defect.

- THE APPEND PATTERN IS OWNED, and the source says so. `fastAppendEncodeBase62(dst []byte, ..)
  []byte` rebinds `dst` and returns it; the caller writes `dst = grow(dst, ..)`. Rebinding a
  sequence parameter does NOT touch the caller's variable — a slice header is a value — so handing
  ownership over and taking the result back is exactly what the source does, not a cost imposed on
  the caller. That is the same shape on both sides.
  Distinguished from the pass-through `return xs`, which does not rebind: there the source's caller
  KEEPS its slice and gets it back, so consuming it would take something the source never took.
  That one still refuses. `rebound` was already an observed fact, so the distinction cost a
  condition on the disposition rather than a new analysis.
  A DEFECT I INTRODUCED AND THE RATCHET CAUGHT: a disposition matches on FACTS, so the new rule saw
  POINTER parameters too, and I had given it a construction of its own — an empty wrap, which
  passes the argument through unwrapped into an `Option<Box<T>>`. Coverage fell and said so. Its
  pointer behaviour is now identical to the rule it precedes, because only the reference form is
  what it exists to change. A rule inserted ahead of another answers every question that one
  answered, not just the one it was written for.

- THE FOURTH BLIND REVIEW. Still DO NOT MERGE. What it adds, ranked by what is ours:
  FREE `new_*` FUNCTIONS ARE THE HEADLINE STRUCTURAL TELL. `pub fn new_label(prefix: String) ->
  Label` is a package-level constructor in the source and an associated function in the target:
  `Label::new`. The reviewer calls it "the single most visible structural tell", and it is
  universal — most real packages construct that way. Provable, too: a package-level function whose
  sole result is a type that package declares.
  `Engine` HAS `run` AND DOES NOT IMPLEMENT `Runner`, while `Driver` does. The asymmetry is real
  and the decision behind it is held: satisfaction is OBSERVED rather than structural, because
  `census/interfaces.md` measured 80,042 structural matches against 1,316 the source declares.
  Emitting every structural match is the guess that census exists to prevent.
  DEAD ACCESSORS ON `Counter` AND `Driver` — the same fixture gap the constructors fixed for
  `Point`, `Label` and `Tag`, and not yet for these two.
  `wrapping_*` UNDER DOCS PROMISING PLAIN ARITHMETIC — raised in three consecutive reviews now.
  Held, and the reason has not changed: the doc is the SOURCE's doc, the source's `add` wraps too,
  and the mismatch is inherited rather than introduced. Three mentions is worth recording as a
  standing disagreement rather than as a finding not yet acted on.

- A PACKAGE-LEVEL CONSTRUCTOR IS AN ASSOCIATED FUNCTION, and emitting it free was, in the fourth
  reviewer's words, "the single most visible structural tell" — the thing that says another
  language's structure was carried across rather than translated. `pub fn new_label(prefix: String)
  -> Label` is now `impl Label { pub fn new(prefix: String) -> Label }`.
  RECOGNISED BY SHAPE, not by name alone. The source's explicit constructor convention is a
  package-level function named `New` or `New<Type>` whose sole result is a type that same package
  DECLARES, and both halves are required: a function merely named `NewFoo` returning something else
  is not a constructor, and one returning a local type without the prefix is a factory the source
  did not mark as one. Neither is moved.
  The declaring half is not decoration — the target's coherence rule forbids an inherent method on
  a type from elsewhere, so a constructor for someone else's type stays a free function however it
  is named.
  EMITTED AS ITS OWN inherent impl block rather than folded into the type's. The engine emits one
  region per source declaration, and a constructor is a declaration of its own; folding would make
  one declaration's output depend on another's, which is the property the region model exists to
  keep. The target allows several inherent impls for a type, so nothing is given up.

- THE MAINTENANCE PROPERTY IS BUILT AND GREEN, and has been for the whole lane — it is worth
  saying plainly, because "keeps them ported as upstream moves" reads like something still to do
  and is the one part of the brief that has been proven end to end since R0.
  `port_go_upstream_drift.rs` holds three properties over a REAL second extraction of one package
  at two versions, at the same unit id, with the two changes a dependency bump actually makes: a
  body changed and a declaration appeared. Not a hand-edited receipt, because a hand-edited receipt
  proves something about the edit.
  A MOVED UPSTREAM IS GREEN AND EXPLAINED, and explained by EXACTLY the snapshot axis. Green
  because nothing is wrong; explained because the receipt has to name what changed; exactly that
  axis because the engine, rules, toolchain and formatter are the same run of the same code across
  the pair, so any other axis moving would mean the receipt describes something other than what
  changed. An engine reporting `Unexplained` on every dependency bump would tell its operator it
  was broken until nobody read the signal again — and that signal is the only thing between a
  maintained port and a fork nobody dares regenerate.
  THE CHANGE REACHES THE EMIT, asserted separately, because an engine could satisfy the delta check
  while emitting something unrelated to what upstream did. `Explained` is about the receipt; this is
  about the output, and only one of them is what a maintainer cares about.
  A STILL UPSTREAM STAYS UNCHANGED, and a PLANTED DEFECT comes back Red and Unexplained. The
  positive and negative halves are both fenced.

- The last actionable finding from the fourth review closed: `Counter` and `Driver` had no
  constructor, so their accessors were dead by construction for every consumer — the same fixture
  gap already closed for `Point`, `Label` and `Tag`. Both have one now.
  A SMALL LESSON IN THE DOING: inserting the constructor above the wrong line attached the NEXT
  declaration's doc comment to it, and the emitted crate showed both sentences stacked. The doc
  convention could not strip the item's own name because the block no longer started with it. The
  emitted output is where a misplaced comment in the input becomes visible, which is an argument
  for reading it after every corpus change rather than only after every engine change.

- HOW FAR THE ENGINE TRUSTS THE SOURCE'S FAILURE CONVENTION — a decision the drain has been
  carrying as open since P4, now made and declared. It was the largest capability blocker left:
  4 packages, 15 declarations, and most of the real fallible code in any of them.
  The source's failing return carries the value beside the failure — `return Nil, errSize` — and
  the target's carries only the failure, so the companion has nowhere to go. The engine already
  dropped it where it could SEE the value was inert, a literal or the absent value, and refused
  otherwise. That was faithful to the cases inspection could confirm rather than to the convention.
  THE DECISION: discard it. The source DOCUMENTS that a result beside a non-nil error is not
  guaranteed to be meaningful, so a caller reading it is relying on something the source does not
  promise.
  THE COST, stated rather than hidden: a source program that violates its own convention — writing
  a meaningful value beside a failure and having its caller read it — ports to a target program
  that does not carry that value, and the engine cannot tell such a program from a conforming one
  because the difference lives in the CALLER. Refusing keeps those programs correct and keeps every
  conforming one unported, and conforming programs are almost all of them.
  THE CASE THAT GIVES MOST PAUSE is named in the reason rather than left to be rediscovered: a
  NAMED result may have been written through before the failure, so its value is work a reader can
  see happening and cannot see being discarded. Under the convention it is still unreadable, so the
  same argument covers it — but it is where a reader would most reasonably expect otherwise.
  Pack DATA, so the trade travels in the digest and whoever owns the pack can reverse it without
  touching the engine. Setting `discards_companion` to false returns to refusing, which costs
  coverage and buys correctness for programs that break their own contract.
  semver 20.7% → 34.5%, uuid 32.0% → 33.0%.
  THE FIXTURE THAT PROVED THE REFUSAL NOW PROVES THE DROP. `corpus-failure` was written to fence a
  refusal that no longer happens; the fence asserts the behaviour the pack currently declares, and
  the stricter half is fenced by the transform's own tests, which declare `discards_companion`
  false. A fixture that proves whatever the pack says is worth more than one that proves a
  behaviour the pack has moved past.

- `copy` SURFACED IN THE TOP EIGHT once the failure-convention decision stopped masking it: 3
  packages, 5 declarations blocked, 29 call sites. Sized before building anything, and the sizing
  says what the obstacle is.
  Shapes: `copy(slice, ident)` 14, `copy(slice, slice)` 9, `copy(ident, ident)` 3, and one each of
  `(ident, selector)`, `(slice, literal)`, `(ident, call)`. So the operands are overwhelmingly
  SLICE EXPRESSIONS — `copy(ksuid[:], b)` — and that is exactly what the current mechanism cannot
  take.
  The faithful target is not one expression but three: `let n = dst.len().min(src.len());
  dst[..n].copy_from_slice(&src[..n]); n`. The source's `copy` takes the MINIMUM of the two lengths
  and returns how many it moved; the target's `copy_from_slice` panics unless the lengths match, so
  emitting it bare would turn a defined truncation into an abort.
  `function_map` answers with a TEXT template and one expression, so it cannot express this: the
  block form needs `{0}` twice, and `render_operand` admits only operands whose text cannot
  reassociate — which a slice expression is not. The gap is the MECHANISM, not a missing row.
  What that needs is a structured form for a mapped call — the same move `pointer_dispositions`
  made when a text template could not say `Some(Box::new(x))` without being re-parsed. Recorded
  rather than started, because it is a mechanism change and this phase already spent its budget on
  the failure convention.

- THE FIFTH BLIND REVIEW, and its classification is the useful part. Asked to say whether the
  problems are correctness, idiom or design, it answered: "mostly design, then idiom, with a
  correctness tail. There is no unsoundness. Only `mix`, the usize/i64 casts, and the blanket
  `wrapping_*` are outright wrong answers." Reviews one to three each named several correctness
  blockers; this one names three, and two of those are decisions held with reasons. That shift is
  the measurement, not the verdict — which is still DO NOT MERGE.

- THE SOURCE'S `string` IS A REFERENCE TYPE TOO, and was emitted owned. The reviewer's first
  blocker: seven public functions take a `String` by value and only read it, so `check("")` does
  not compile and `check(x.to_owned())` allocates and drops. A consumer-breaking API shape, and the
  SAME finding as the map and the slice one, one type further on: the source's string is immutable
  and shares its backing, so passing it costs nothing and the caller keeps it.
  `string` joins the reference kinds and is answered by the same dispositions on the same facts.
  Its borrowed form is `&str` rather than `&String`, composed the way a slice's is — `str` is the
  unsized view, and `&str` takes every `&String` and also a literal and a subslice.
  It is a BASIC kind rather than a composite one, so the reference test cannot key on kind alone;
  it keys on the source type name the pack already owns.
  AN ESCAPING STRING NOW REFUSES, and correctly. `Label(id, fallback string) string` returns one of
  its arguments, so the target's signature needs a lifetime tying the result to them — and nothing
  here emits lifetimes. It used to emit a signature that CONSUMED both, which the source never
  does, so this is the same correction the map and slice change made. Moved to the refusal corpus.
  A BORROWED VALUE REACHING A FIELD THAT OWNS has to be owned there, which the compile proof said
  within seconds: `Driver { engine, label }` with `label: &str` into a `String` field. The source
  never had to say this because its string was already shared. The transform asks the same question
  the signature answered rather than re-deriving it — a parameter is borrowed exactly when its
  disposition chose a borrow — and `to_owned` rather than `clone`, because `clone` on a `&str`
  yields a `&str` and the field wants the owned form.
  COVERAGE FELL AGAIN, for the fifth time and the fifth same reason: xxhash 61.8 → 58.8, errors
  31.6 → 26.3, semver 34.5 → 31.0. Every declaration that moved was emitting a signature that
  consumed a value the source shares.

- `Self` INSIDE THE TYPE'S OWN IMPL. A constructor moved onto its type names that type twice —
  once in the result and once in the literal it builds — and the target spells both `Self`. Not
  merely shorter: `Self` survives a rename, where the name written twice has two places to miss.
  The source has no such spelling and therefore always writes the name, so every constructor the
  engine moves would otherwise carry it twice.
  Structural rather than textual: the result is a resolved type and the literal carries its path,
  so a name that merely RESEMBLES the type is not touched. Fourth idiom to carry rust-skills
  provenance.

- THE ERROR MODEL IS SETTLED, and settled as HELD rather than owed. Two reviewers called
  `Box<dyn Error + Send + Sync>` with a string payload unfit for a library because a consumer
  cannot match on it. Checked rather than conceded: the source's `errors.New("empty")` gives its
  own callers exactly as little — an opaque interface value with a string inside — and a source
  caller who wants to branch uses `errors.Is`/`errors.As` against a sentinel value or a concrete
  type, which the target's boxed trait object plus a downcast supports. The criticism is of the
  SOURCE's error design, carried across rather than introduced.
  A concrete error enum would give the ported crate an API the source does not have. That is an
  IMPROVEMENT, and improving is not porting: it invents a type upstream never declared, it cannot
  represent a failure propagated from a callee the package does not know, and the next upstream
  release would have to be reconciled against a shape nobody upstream wrote — which is precisely
  the property that turns a maintained port into a fork.
  Written into the pack's failure convention so it travels in the digest and is answered where the
  decision lives, rather than being re-argued by the next reader of the emitted crate.

- THREE STANDING DISAGREEMENTS now, all with the same shape and all recorded rather than open:
  `wrapping_*` under the source's own docs (three reviews), the error model (two reviews), and
  observed-rather-than-structural interface satisfaction (census-backed, 80,042 structural matches
  against 1,316 the source declares). Each is a place where the emitted crate is faithful to a
  source that a Rust reviewer would not have written that way. Worth naming as a CLASS: a reviewer
  judging the output as hand-written Rust will always find the source's design decisions and
  attribute them to the engine, and the engine's answer has to be a written reason rather than a
  fix — because the fix is a different program.

- A REFERENCE UNDER UNPROVEN FACTS BORROWS SHARED, and the reasoning is NOT the pointer's — which
  is why this was worth re-deriving rather than inheriting. The largest coverage move of the lane:
  xxhash 58.8 → 70.6, uuid 33.0 → 37.1, errors 26.3 → 31.6, semver 31.0 → 34.5, ksuid 37.6 → 38.7,
  and `transform ownership` from 7 packages / 16 declarations to 6 / 8.
  The owned form was chosen for a POINTER because it costs a move and cannot be wrong. That escape
  hatch does not exist for a reference: owning a map, a slice or a string CONSUMES the caller's
  value, which the source never does — so for a reference, owned is not the safe answer but a wrong
  one. Inheriting the pointer's reasoning would have inherited a premise that is false here.
  Of what remains, a shared borrow is the only choice that neither consumes nor narrows. `&mut`
  would demand exclusive access the source never demanded, and would fail at the CALLER rather than
  where the decision was made.
  AND A SHARED BORROW THAT TURNS OUT INSUFFICIENT DOES NOT COMPILE. That is what decides it: an
  unproven borrow on a reference fails LOUDLY at the port, where an owned parameter would have
  silently changed what the program does to its caller's value. The engine's rule is that it
  refuses what it cannot prove — and between two unprovable answers, the one whose failure is a
  build error is categorically different from the one whose failure is a wrong program.
  Still to revisit when the analysis can read the standard library's effects, which is what leaves
  these facts unproven in the first place. `census/` sizes no such family.

## Still owed by this lane
  One class the ratchet surfaced and this lane is deliberately leaving refused: `return named, err`
  where `named` is a NAMED RESULT. Go's convention says a caller may not read it after a non-nil
  failure, which is the same argument that makes discarding a literal sound — but a named result
  can have been written through before the failure, and nothing here proves it was not. Four
  declarations in uuid. It is a decision about how far to trust the source's convention, and it
  belongs to whoever owns the pack rather than to the engine.

- A concrete value flowing into a trait-object POSITION needs a coercion the body translator does
  not have. `Describe(tag)` where the parameter is `&dyn Named` is fine when the argument resolves
  to `&Tag`, and is not when a conservative ownership rule makes it `Option<Box<Tag>>` — which is
  why the argument and result SITES are proven on the snapshot in `corpus-interface/` rather than
  end-to-end through the emit. The assertion site is proven end-to-end.
- A struct embedding an INTERFACE still refuses, because the field position has no declared
  trait-object form. `Box<dyn T>` would compile and would claim unique ownership of a value the
  source may share, and nothing here proves it is not shared — which is the same reason P3 refuses
  rather than guessing a pointer disposition. `runtime.codec` is this shape.
- Stdlib interface mapping (`io.Reader` → `std::io::Read`, `fmt.Stringer` → `Display`) is a
  precondition of the rule pack rather than an output of the engine, per `census/interfaces.md` §11
  item 5. 25 embeds and ~100 assertions in CORE target types outside any corpus.
- `defer`, `panic`/`recover`, `select`, closures and the type switch still refuse BY NAME. Each
  needs its census read first — `census/defer-panic-recover.md` sizes the first two at six callee
  shapes covering 77.9% — and guessing at one is how a translator emits a body that compiles and
  runs a different program.
- A composite literal with POSITIONAL fields refuses. The front end has the field order and could
  name them, which would remove the refusal with a proof rather than a hope; it was left out of
  P4 to keep the change to the constructs the corpus exercises.
- `core/transform` has no directory-completeness fence, because it carries neither the Go-toolchain
  firewall nor a corpus-needle property. Its four new body modules are therefore unenumerated
  anywhere — which is correct today and would stop being correct the moment transform acquires a
  fenced property.
- `specs/port-rules/lang/go-rust/**` remains unlanded; the pack is still the package-local mirror.
  Out of the `build/**` envelope, so it needs an integ/specs lane.
- Nothing emits into `k8s/`, and no Kubernetes corpus is admitted. That is W1 and is not this
  lane's to open.

## Next gaps (ordered)

1. **Lock absorb** — `Cargo.lock` / root `Cargo.toml` workspace membership refresh waits
   `#1646` land (ci/controller paths must exist before members); no third writer; libs `#1649`
   must not steal lock. Then refresh lock for path-dep / workspace edges (serde, syn, quote, sha2).
2. **Toolchains cell remap + shrink** — set `.buckconfig` `toolchains = build/toolchains`,
   update reachability/`toolchains/` prefixes (may need integ/specs attach), then delete
   root `toolchains/**`. Do not delete while the cell still points at the root path.
   **PARKED:** `.buckconfig` is outside `roots.build` envelope globs (`build/**` only).
   After remap, prefer digesting the live cell path and drop the package-local corpus mirror.
3. **Forever port-rules materializer** — land live `specs/port-rules/**` on integ/specs; replace
   package-local mirror with ADR-0597 materializer relationship (build tip keeps hermetic copy
   until then). Bootstrap Go extractor remains out-of-band only (Slice 8 admits artifacts only).
4. **Richer constructions** — expand beyond `pass_through` / `empty_canary` once forever
   `specs/port-rules/**` lands; keep kernel free of construction vocabulary.
5. **k8s/ materializer emit** — regenerable output into `k8s/` waits integ/k8s rail + ADR-0597
   materializer; W0-B forbids bulk corpus emission from integ/build.

## Out of envelope (do not touch from `integ/build`)

- `specs/k8s-port/` — judgment pending; no rehome (Slice 3 embeds a same-package mirror only).
- `specs/port-rules/**` — forever integ/specs (Slice 7 embeds hermetic mirror only).
- `k8s/**` — separate integ rail (mechanical port *generates into* k8s/; does not own the tree).
- `.buckconfig` cell remap for toolchains — coordinate with reachability/registry consumers.
- `ci/controller/**` members — wait `#1646` land (reverted premature absorb @ `72530017a`).

## `&T{..}` needs no destination, and `New(..) *T` is still a constructor

- Unary `&` was refused everywhere except an ARGUMENT, where the signature table names the
  parameter's disposition. That left 33 sites across 5 packages, of which 14 are `x := &T{..}` (7),
  `return &T{..}` (4) and `x = &T{..}` (3) — and none of the 14 needs a destination at all.

  The operand is a COMPOSITE LITERAL: a value the expression itself creates. Nothing else can alias
  it, no binding is moved out of, and there is no caller whose value could be borrowed. So the owned
  form is not one choice among several here, it is the only one available, and the pack's pointer
  type already says what owned means for a pointer.

  `&x` of an EXISTING binding stays refused, and the difference is the whole point: that one borrows
  or moves something that already has an owner, which is precisely the ownership question the
  signature table exists to answer. A fresh composite does not have it.

  Built from the `escaping_owned` disposition's declared `wrap: ["Box::new", "Some"]` construction
  rather than from a second rule saying the same thing — one place says what an owned pointer is and
  how one is constructed, and a second would be free to disagree with it.

  `errors` 31.6% → 52.6% (6 → 10 of 19). The other six corpora did not move: a declaration only
  counts as translated when every construct in it translates, and in those the `&T{..}` site shared
  a body with a blocker still on the board.

- The corpus case it needed EXPOSED the next defect. `NewTally(label string) *Tally` emitted as a
  free `pub fn new_tally`, because the constructor rule required the sole result to be a type the
  unit declares and `*Tally` is a pointer to one. But `func New(..) *T` is the SAME convention as
  `func New(..) T` and is the commoner of the two: the source allocates away from the caller's frame
  and hands the pointer back, and what it constructs is the pointer's target either way. That target
  is the type the impl block stands on, so the pointer is looked through rather than treated as a
  different result. Coherence is untouched — the impl is still on a type this unit declares.

- STANDING TENSION, recorded rather than fixed: `Tally::new` returns `Option<Box<Tally>>`, because
  the pack maps `*T` to the nil-representable form. It is sound — the source's `*T` genuinely admits
  nil, and a caller can get nil back from a source constructor — but no hand-written `new` has that
  signature, and a reviewer will say so. Changing it is a decision about the pointer type itself, not
  about constructors, and it belongs with the pointer mapping rather than here.

- Verify: 49 test binaries green; port-engine clippy `-D warnings` clean; `delta` Green/Unchanged;
  golden refreshed (`Tally::new`, `Some(Box::new(Tally { .. }))`); engine digest moved to
  `sha256:e12aa8cf…`. buck2 `Pass 29, Build failure 20` — the known third-party export gap recorded
  above (prettyplease/proc-macro2 carry `visibility = []` and have no PUBLIC alias), unchanged by
  this work and still outside this lane.

## A package variable nothing writes is a `static`

- Top of the board by a wide margin: `deferred by policy: var`, 67 declarations across 5 of the 7
  surveyed corpora. The pack's own reason already said what remained — the synchronization argument
  is TRUE, and it bites only for a variable something assigns to, which most package variables are
  not. What was left undecided was the FORM.

- THE DECISION. `static`, not `const`, and that is the whole of it. A source package variable has
  ONE storage location and one address for the life of the program; `&X` gives the same pointer
  every time. A target `const` is materialised afresh at every use and has no stable address, so
  `&X` would differ per use — observable for a variable whose address the source can take. A target
  `static` has exactly that storage identity, and being immutable it raises no synchronization
  question at all: there is nothing to synchronize when nothing writes.

- THE PRICE, and why it is not a heuristic: a `static`'s initialiser must be a CONSTANT EXPRESSION.
  That is the target's own rule, not a proxy for one — an initialiser failing it fails to COMPILE
  rather than meaning something different. So the admitted shapes are closed (a literal, an absent
  initialiser, an ident naming a constant, a composite whose every element is one) and everything
  else refuses by name. The common refusal is a CALL: `errors.New("..")` and `regexp.MustCompile`
  allocate, and reaching for `LazyLock` instead would run the initialiser on FIRST USE rather than
  before it — the same when-does-the-work-happen question that defers `package_init`, and not one
  this rule may answer on its own.

- WHAT STAYS UNDECIDED needed a new home, and the pack's own validator is what said so: it refuses a
  kind that is both deferred and captured, which is right — the pack cannot both translate a kind
  and record it as untranslated. But a written package variable and an unwritten one are the same
  KIND and reach the same rule; only the first is undecided. So `undecided_forms` was added beside
  `deferred_kinds`: a SHAPE within a kind, keyed by an id the engine names when it declines, with a
  required reason that the refusal QUOTES. The reason a reader sees and the reason the digest
  carries are now one text rather than two that can drift.

- The value is produced by the SAME translator a body uses, and the constant test is a separate
  question asked of the source nodes. One translator means a value in a static and the same value in
  a function cannot come out differently; asking the question of the nodes rather than of the
  rendered text means it is exact.

- Two things fell out of it that were wrong before and are not about statics at all:

  - `true` reaches the model as an IDENT referring to a universe-scope constant, not as a literal.
    The body translator cased every constant reference, so `x == true` was emitting `x == TRUE` — a
    name nothing declares. Fixed at the one place identifiers are cased, via a new `constant_map`
    holding the source's predeclared constants. `iota` is deliberately absent: it is meaningful only
    inside a const declaration where its value depends on position, and a table of names cannot say
    what it is.

  - A static of a source `string` initialised from a literal takes `&'static str`. Not a preference:
    `"id-"` HAS that type, and the owned `String` cannot be built by a constant expression at all,
    so the owned form does not exist here. It is also what every reader wants, since this rule only
    ever applies to a variable nothing writes.

- Ratchet, five of seven packages up: uuid 37.1 → 49.5, ksuid 38.7 → 45.2, semver 34.5 → 41.4, xid
  23.1 → 30.8, xxhash 70.6 → 73.5. errors and go-multierror unmoved. The largest single move of the
  lane so far.

- Verify: 49 test binaries green, including the compile proof — the emitted crate with four statics
  in it compiles under rustc, which is what makes `Point { x: 0, y: 0, label: String::new() }` a
  claim rather than a hope. Port-engine clippy `-D warnings` clean; `delta` Green/Unchanged; golden
  refreshed; transform manifest regenerated for the new file. buck2 `Pass 29, Build failure 20` —
  the known third-party export gap, unchanged.

- NOTED, not fixed: `adapters/rulepack/src/pack.rs` is 376 lines, over the 300-line bar. It was
  already 363 before this work and this added 13; splitting it belongs in its own commit rather than
  as noise inside a semantically dense one.

## The failure the engine could not prove was a failure

- Started as coverage work — `error` nested inside another type was the top blocker, 19 declarations
  across 4 packages, refused because "a trait has no size in the target". The pack had ALREADY
  answered it: the failure convention chose an owned boxed form because a failure outlives the call
  that produced it, so a reference would need a lifetime the caller cannot supply. A struct field,
  a composite element and a map value all have that problem for that reason. So the failure type
  resolves through the convention in every position that STORES the value, and a second answer here
  could only disagree with the one the pack already gave.

  A PARAMETER is left refusing, and the distinction is the same one strings taught: the source's
  error is an interface value the caller keeps after passing it, so owning it in the target would
  consume a value the source never consumed. What a borrowed failure parameter should be is a
  decision of its own.

- The corpus needle for it FOUND A SOUNDNESS DEFECT, which is what corpus needles are for.
  `func (r *Report) Cause() error { return r.cause }` emitted `Err(self.cause.clone())` — a function
  that reports failure unconditionally. In the source a nil `cause` means SUCCESS. The engine was
  wrapping any non-nil-literal trailing operand in `Err(..)`, which is right only when that operand
  cannot be absent, and silently wrong when it can — in the direction of reporting failure where the
  source reported success. Output that compiles and means something different is the one failure
  this engine exists to prevent, and it had been shipping it.

  It had never been caught because both shapes the hermetic corpus exercised ARE proven: a call to
  `errors.New` and a propagated failure from a tested binding. The unproven shape had simply never
  appeared.

- THE PROOF, now required, and it is two things and nothing else:

  - a CALL to a callee the pack names a failure CONSTRUCTOR, because a constructor has no absent
    result to return. Which callees those are is the pack's to say, and the distinction matters: a
    source function that merely RETURNS an error, like `Check(s) error`, is not one of them;
  - the ADDRESS OF A FRESH COMPOSITE, which needs no table because it is a property of the
    construct — the expression creates the value, so nothing can have made it absent.

  The tested binding is not listed because it never reaches here: the propagation rule recognises
  `if err != nil { return 0, err }` as a whole and rewrites it to the target's operator, which is
  the translation that makes the check impossible to forget.

- The refusal corpus is its own package, per the standing pattern, and it holds BOTH answers:
  `Cause` and `Wrapped` share a signature, a receiver and a field and are decided differently,
  because the proof is a property of the OPERAND rather than of the signature. An engine reading the
  signature alone would give them one answer and be wrong about one of them. The fence asserts the
  refusal names the declaration, says the proof is what is missing, and says what emitting it anyway
  would cost — not merely that something refused.

- COVERAGE FELL, the sixth honest correction of the lane: uuid 49.5 → 44.3, semver 41.4 → 36.2,
  errors 52.6 → 47.4, ksuid 45.2 → 44.1. Fifteen declarations had been counted as translated while
  emitting a program that reports failure where the source reports success. The nested-error fix
  paid for part of it and the proof took more back; net of the two, the number that matters is that
  no emitted declaration now claims a failure it cannot prove.

- Verify: 49 test binaries green including the compile proof; port-engine clippy `-D warnings`
  clean; `delta` Green/Unchanged; golden refreshed with `Report`'s boxed error field.

## The sentinel error, which blocked twice over

- After the failure proof landed, the top of the board was the refusal it introduced — 14 unproven
  `ident` returns across 5 packages. Almost all of them are the same thing: a SENTINEL.
  `var ErrSize = errors.New("size")`, declared once and returned from many places, is the commonest
  error-typed package variable in real code, and it blocked twice over. The declaration is not a
  constant expression, so it could not be a `static`; and every `return ErrSize` is an operand
  nothing could prove was a failure.

- THE DECISION: a sentinel becomes its MESSAGE. `static ERR_SIZE: &str = "size"`, and each failing
  return builds a failure from it through the same mapping the pack already declares for the
  constructor — one rule doing the work rather than a second free to disagree with it. The message
  is a constant expression, so there is no lazy initialisation and no when-does-the-work-happen
  question of the kind that defers `package_init`.

- THE COST, and it is real: the source's sentinel has IDENTITY. `errors.New` returns a POINTER, so
  `err == ErrSize` compares identity and is a line real code writes. The target's boxed trait object
  has no equality at all, so nothing means what that line means. The split falls exactly along what
  the target can express — RETURNING a sentinel ports, COMPARING against one does not — and the
  comparison refuses by name at the site.

- THE FIRST TRY AT IT WAS WRONG, and the corpus caught it. Building the failure at every reference
  to a sentinel meant `err == ErrGone` emitted `err == Box::from(ERR_GONE)`: not a refusal, just
  code that fails to compile — and for third-party corpora, which are never compiled, not even that.
  An identifier does not know where it stands. So the construction moved to `fallible_return`, which
  is the one place the engine already knows the operand IS the failure, and a sentinel read anywhere
  else refuses. Emitting something that merely fails to compile is a worse outcome than refusing,
  because the reader gets a broken crate instead of a sentence saying what is missing.

- `fmt.Errorf` is a failure constructor and NOT a sentinel one, and the distinction is why the pack
  has two lists: its message is FORMATTED from arguments, which is not a constant expression. Seven
  of the seventeen error-typed package variables in the corpora are built that way and refuse.

- THE REFUSAL CORPUS SHADOWED ITSELF, which is the failure mode the engine's own comment warns
  about: the sentinel comparison and the unproven operand shared a package, the transform reports
  the first refusal it reaches, and the sentinel one took over — silently un-proving the class the
  corpus had been added for. Split into `corpus-sentinel/`, per the standing pattern. A refusal class
  proven in a shared corpus stops being proven the day another refusal lands beside it.

- Ratchet: semver 36.2 → 51.7, uuid 44.3 → 46.4, ksuid 44.1 → 45.2. Together with the failure proof
  that preceded it, semver is up 15.5 points on where it started the pair and every declaration that
  returns a failure now proves it is one.

- Verify: 49 test binaries green including the compile proof — `Err(Box::<dyn Error + Send + Sync>
  ::from(ERR_EMPTY))` compiles; port-engine clippy `-D warnings` clean; `delta` Green/Unchanged;
  golden refreshed.

## Holding the 100–300 line bar, and the one file that cannot hold it

- Four files had crossed 300 during this lane's work and one was already over. Split along real
  seams rather than at convenient line numbers, because a cut that does not follow a question
  boundary makes two files nobody can hold in their head instead of one:

  - `rulepack/pack.rs` 380 → `pack.rs` 101 (what a loaded pack IS and what it can be asked),
    `load.rs` 204 (what a document must SATISFY to become one), `rules.rs` 131 (one rule document
    into one loaded rule — the one part of loading with a single input and a single output);
  - `transform/resolve_tables.rs` 320 → `resolve_tables.rs` 203 (asking the pack's flat tables what
    a name means), `resolve_types.rs` 139 (walking a type built out of other types);
  - `transform/body.rs` 345 → `body.rs` 272 (the statement translator), `body_parts.rs` 85 (the
    accessors that ask a node for a part it must have and NAME the owner when it does not);
  - `frontend-go/vocabulary.rs` 311 → `vocabulary.rs` 186 (what a node IS — kinds), and
    `vocabulary_facts.rs` 132 (what was OBSERVED about it — flags and attribute keys).

- `core/kernel/lib.rs` is 520 and STAYS ONE FILE. Not an exception granted, an exception the design
  requires: the neutrality claim is a `const` assertion that reads the file's own bytes at compile
  time, and `UNSCANNED_CODE_KEYWORDS` refuses the word `mod` precisely so kernel code cannot end up
  in a file the scan never reads. Splitting it would make the fence smaller than the thing it
  fences, which is the failure the fence exists to prevent. It was already over the bar before this
  lane and is not this lane's to move.

- TWO FENCES CAUGHT THE SPLIT, which is what they are for: `frontend-go`'s firewall and `rulepack`'s
  neutrality test each enumerate their crate's sources and assert the enumeration IS the directory.
  Both failed on the new files rather than quietly covering less than before.

## Propagation without the check, and a refusal that had become an essay

- `err := f(); return v, err` is the same program as the checked form — returning the failure when
  it is ABSENT is returning success, so the source omits the test — and real code writes it
  constantly. `func FromBytes` is this exact shape in three of the surveyed corpora. The target
  spells it as the operator followed by the success: `f()?;` then `Ok(v)`.

  Two statements in, two out, because the source's two do two things: run the fallible call, and
  return the values with whatever it produced. The operator carries the failure out and the success
  carries the values, which is the same split.

  STRICT in the same way the checked matcher is: the return must be the VERY NEXT statement, and the
  bind's source must be a CALL. Anything between could write the binding or do work the operator
  would silently drop, and a bind that is not a call has a provenance this statement cannot see.
  `v, err := f()` is deliberately not matched — its values come from the call, so `f()?` produces
  them and a separate return has nothing to name.

  NO COVERAGE MOVEMENT, and the reason is worth recording rather than hiding: the corpora sites are
  all `var uuid UUID` followed by a method call, and `DeclStmt` has no translation yet. The rule is
  correct and proven by the hermetic corpus; it becomes visible in the ratchet when `DeclStmt`
  lands. Counting it as a win now would be counting the same declaration twice.

- The unproven-failure refusal had become an ESSAY. It inlined the pack's whole
  `constructor_reason` at every site — about 1,500 characters, most of it repeated from the previous
  refusal on the same screen — which drowns the one sentence a reader needs. Trimmed to what is
  missing, what it would cost, and the two proofs, with the pack field NAMED so the full reasoning
  is one lookup away. Quoting the pack so text cannot drift was the right instinct and the wrong
  amount: the pointer keeps the property and loses the flood.

## A body-scoped const is a binding, and a harness that had been measuring stale input

- `const x = 4294967296` inside a function reached the model as `unsupported`. Recorded as a
  BINDING, and that is a decision about what it means rather than about what it is called: the
  source's untyped constant has no type until it is used and takes one from each use; a target
  `const` must fix a type at the declaration, and a target `let` takes one from use exactly as the
  source's does. So the binding is the faithful form and the target `const` is the approximation.

  The cost is stated: a source constant used at TWO different types in one function has no single
  target binding. That does not compile, which is the safe failure — it never means something else.

  The reference casing follows, and had to: a body-scoped constant now classifies as a local, so
  `factor` cases as `factor` rather than as `FACTOR`, which would name nothing. A package-scoped or
  predeclared constant is untouched.

- THE HARNESS HAD BEEN MEASURING A STALE SNAPSHOT for three of seven corpora, and said nothing. The
  extraction renamed each corpus's module to `corpus.example/<name>`, which breaks any subpackage
  importing the module by its CANONICAL path — `cmd/ksuid` importing `github.com/segmentio/ksuid` is
  the shape — and the whole extraction then failed, leaving the previous snapshot on disk for the
  survey to read. `xxhash`, `ksuid` and `xid` had been measured against snapshots hours old.

  Fixed by reading each corpus's own module path from its `go.mod`, with a synthetic fallback for
  `pkg/errors`, which predates modules and has no self-import to preserve. The numbers barely moved,
  which is luck rather than vindication: a harness whose failure mode is measuring yesterday's input
  is one whose every result has to be re-earned before it can be believed. Recorded because the next
  person to see a flat ratchet should check this before concluding a rule did nothing.

- ksuid 45.2 → 46.2 against a fresh snapshot.

## Parallel assignment, whose whole content is the order

- `AssignStmt` was the second-largest cause on the board — 40 refusals across 6 packages — and the
  extractor was refusing two shapes: `a[i], a[j] = a[j], a[i]` and `x, err = f()`.

- Both translate, and the reason is one fact: the source evaluates every operand on BOTH sides
  before assigning any of them, which is what makes the first a swap rather than two writes. The
  target's destructuring assignment has the same rule, so the construct carries across whole. Two
  separate assignments would not — the first place would be written and then read back by the
  second, which is a different program.

- REFUSED where a place's own subexpressions could have EFFECTS. The two languages evaluate a
  place's subexpressions at different points, so a call inside one would run at a different time.
  Admitted: a name, a field of one, an index by a name or a literal. None of those runs any code.

- Carried as its own IR statement rather than a sequence of assignments, for the same reason: the
  order IS the construct, and a sequence cannot express it.

- ksuid 46.2 → 49.5. Verify: 49 test binaries green including the compile proof; clippy clean;
  `delta` Green/Unchanged; golden refreshed with `(values[i], values[j]) = (values[j], values[i])`.

- NOTED: `values.swap(i, j)` is what a Rust author writes. That is an IDIOM — it changes nothing
  about the program — and belongs in the idiom table with its rust-skills provenance, not here.

## The blind review's headline, and turning a whole class of it into a build failure

- Sixth blind review, on the emitted crate with the generation marker stripped. Verdict DO NOT
  MERGE, and the composition has shifted again: the reviewer's own closing paragraph names the
  split — "most of section B evaporates under [a port-fidelity] framing and section D becomes the
  point rather than the problem". That is the honest reading, and it is why the findings are sorted
  here into what the ENGINE gets wrong and what is a property of the corpus it was fed.

- THE HEADLINE, and the reviewer said to read one line if only one: `pub fn new(label: &str) ->
  Option<Box<Tally>>` for a constructor that cannot fail. "Two independent defects in one signature:
  an `Option` with one inhabited case, and a heap allocation the caller did not ask for and cannot
  avoid."

  Both come from one place. The pack maps `*T` to the nil-representable owned form, which earns its
  `Option` from nil and its `Box` from ownership — right wherever a pointer may be absent, wrong
  wherever it may not. A function whose EVERY return is the address of a value it just created can
  produce neither: nothing can be absent, and nothing else can hold an alias. So that result is the
  value, and the caller gets exactly the ownership the source hands them.

  The proof is the one a failing return already uses — the address of a fresh composite is never the
  absent value — read rather than restated, so a change to what counts as fresh changes both. And
  the SIGNATURE and the BODY read the same proof rather than each deciding: they must agree or the
  emitted function does not compile, which is the kind of disagreement a `Body` field exists to
  prevent. Result: `pub fn new(label: &str) -> Self { Self { label: label.to_owned() } }`.

  Requires a BODY. A signature-only declaration proves nothing, and a caller of one has no way to
  know what its returns look like, so the nil-representable form stays the honest answer there.
  Requires at least one return: a body that falls off the end returns the zero value, which for a
  pointer IS the absent one.

- THE COMPILE PROOF NOW DENIES WARNINGS, which turns the reviewer's whole "does not build
  warning-clean" section into a build failure instead of something a reviewer has to find. A warning
  the TRANSLATION invents — a mutable temporary the source did not have, an assignment nothing reads
  — is a defect in this engine.

  Three allowances remain, each a property of the SOURCE rather than of the translation, and the
  third is new and worth naming: Go writes `x := 0` and then assigns in every branch, and warns on
  neither the declaration nor the dead initial value; Rust's flow analysis sees the initialiser
  overwritten before it is read. A faithful port produces that warning however well it is done.
  What would remove it is emitting `let x = if c { a } else { b };` — the target's `if` is an
  expression and the source's is not — which needs the front end to report that the initial value
  is never read on any path. Recorded as the work, not waved away.

- STILL ON THE BOARD from this review, ranked by how much of the "reads translated" impression each
  carries: the `as i64` on every `len()` and the index loop that should be `for &v in values`; the
  engine-generated provenance sentence "Ported from an implicit interface" shipping in public
  rustdoc; `crate::shapes::Point` spelled in full where a `use` belongs; a marker trait with
  hand-written empty impls where `impl<T: Runner + Describer> Job for T {}` is mechanical; derives
  that stop short of `Eq`/`Hash` on types whose fields support both.

- REAFFIRMED STANDING DISAGREEMENTS, now with another review behind each: blanket `wrapping_*`
  (four reviews), `Box<dyn Error>` as a library's error type (three), and impls emitted only for
  OBSERVED satisfactions rather than structural ones (three).

## A counter used only as an index is a `usize`

- The most pervasive "reads translated" tell after the constructor, and the reviewer named it
  precisely: `for i in 0..values.len() as i64 { values[i as usize] }` is "a length cast to signed
  and immediately cast back to index."

- Both conversions come off where the counter is used for NOTHING BUT indexing, and the argument is
  the loop's own bound: the range's upper bound IS a length, so no value the loop produces can be
  negative or exceed `usize`, and the round trip is the identity for every one of them. The signed
  value is never observed, so it does not need to exist.

  ONE read that is not an index and both stay — passed to a function, compared against something the
  source typed `int`, stored in a field. In each of those the signed value IS observed.

- Declared as an IDIOM, with rust-skills provenance, because that is what it is: it changes the
  spelling and not the program. The mechanism is gated on the pack declaring the rule, exactly as
  the borrowed-slice idiom is, so a pack that drops it gets the conversions back with no code
  change.

- The bound's conversion is dropped by comparing against the FORM THE PACK DECLARES rather than by
  editing rendered text: the pack's `len` mapping is `{0}.len() as i64`, and only that declared
  trailing conversion is stripped. A pack whose mapping has no trailing conversion is left exactly
  alone, so code and data cannot drift into disagreeing about what `len` becomes.

- The range and the index read the same proof rather than each deciding, for the same reason the
  pointer result's signature and body do: they must agree or the loop does not compile. Scoped to
  the loop that proved it, so a name shadowed by an inner loop with different uses gets its own
  answer.

- No coverage movement, and none expected: every one of these declarations already translated. What
  moved is what the output READS like, which is the bar the goal actually sets — an engine whose
  output a reviewer judges as hand-written.

## The translator's note in the public rustdoc, and the bundle that should be one impl

- "Ported from an implicit interface: the source was observed satisfying `X` at <site>." was
  shipping in the emitted crate's PUBLIC RUSTDOC. A reviewer found it and named it as a translator's
  working note published as API documentation, and they were right twice over: a doc comment is what
  a CALLER reads, how the engine came to emit an impl is not something a caller can act on, and it
  tells them the crate was generated — which is the one thing this engine is trying not to say.

  The provenance is not lost. Which satisfactions were observed and where is exactly what the plan
  and the receipt record, and that is where it belongs: the emitted crate is the PRODUCT, not the
  record of how it was made. That distinction had been missing.

- A pure SUPERTRAIT BUNDLE now gets one blanket impl instead of one empty impl per observed type.
  `pub trait Job: Runner + Describer {}` with `impl Job for Driver {}` beside it is Go's interface
  embedding transliterated; the source satisfies such an interface STRUCTURALLY, so every type with
  the embedded method sets has it and the target says that once: `impl<T: Runner + Describer> Job
  for T {}`.

  Not merely tidier — it is what the source MEANS. The per-type form gives the trait only to types
  the engine saw asserted, and the source gives it to every type that qualifies. A caller writing a
  generic function over `Job` would find their own type rejected under the per-type form and
  accepted under this one, which is the difference between a translation and an approximation. It is
  also the first place the observed-vs-structural standing disagreement has been answered rather
  than recorded, and it is answered where the answer is sound: a bundle has no method to implement,
  so a blanket impl asserts nothing the engine has not seen.

- THE COMPILE PROOF CAUGHT THE CONSEQUENCE, one commit after the proof started denying warnings: the
  blanket impl and the per-type impl are a COHERENCE CONFLICT, not a redundancy, and the emitted
  crate stopped compiling until the per-type one went. That needed a fact only the type-checker can
  see, because the interface is routinely declared in a package the observation is not in — so the
  front end now records `bundle` on a satisfaction whose interface declares no method of its own and
  embeds at least one. A fact, observed and recorded, with the decision made downstream of it.

- The 87.3% figure the corpus doc cites is what makes this worth the machinery: that is the share of
  embedding interfaces that declare no method of their own.

## `Eq` and `Hash`, and the float that is the whole of the difference

- A reviewer: "`Point` derives `PartialEq` but not `Eq`/`Hash` — its fields are `i64, i64, String`,
  all of which support both, and a point is an obvious map key." Right, and the fix is pack data
  with no mechanism change beyond one precision.

- `Eq` is TOTAL equality, which the source has wherever `==` is defined and no field is a float. The
  float is the whole of the difference and it is not a conservative exclusion: NaN is not equal to
  itself in EITHER language, so a struct with a float field has no equivalence relation in either,
  and claiming one in the target would claim something the source never had.

- `Hash` follows from what the source requires of a MAP KEY — that the type be comparable — which
  the target spells `Eq + Hash`. A source type usable as a map key must port to one usable as a
  target map key, or a ported program that indexes by it has nowhere to go. Blocked by the same set
  as `Eq`, because `Hash` without `Eq` violates a contract the target's own documentation states.

- ONE MECHANISM CHANGE, and it was a real imprecision rather than a widening: `blocked_by` matched a
  type's KIND only, and a float is kind `basic`. It now also matches the NAME of a basic type — and
  only of a basic type, which is what keeps it safe: a basic type's name is one the language
  defines, so it cannot collide with a user type that happens to be called `slice`.

- `Celsius(f64)` correctly earns neither; `Counter` earns both.

## Session close: where the lane stands

- SEVEN third-party corpora, coverage at close: xxhash 73.5%, semver 51.7%, ksuid 49.5%,
  errors 47.4%, uuid 46.4%, xid 30.8%, go-multierror 0.0%.

  Against the session's opening numbers — xxhash 70.6, ksuid 38.7, uuid 37.1, semver 34.5,
  errors 31.6, xid 23.1 — every package is up, and semver by 17.2 points. That is NET of a
  correction that removed 15 declarations which had been counted as translated while emitting a
  program that reports failure where the source reports success.

- THE BOARD, ranked by packages blocked: an unproven `ident` failure operand (5 pkgs, 13 decls);
  unmapped `interface` — the source's `any` (4, 7); an unproven `call` failure operand (4, 6);
  a compound argument to `len` (4, 4); `foreign_satisfaction` deferred (3, 6); `panic` with a
  non-literal payload (3, 4); `ArrayType` (3, 4); unary `&` outside an argument (3, 3).

- INVARIANTS, all holding: refusal corpora refused by name across six classes, each in its own
  corpus; planted defect Red/Unexplained; upstream-drift pair Explained on exactly the snapshot
  axis; an engine-source change moves `engine_digest`; no engine library reads Go; no production
  source carries a corpus needle; vocabularies closed; every source file 100–300 lines except
  `core/kernel/lib.rs`, which the design requires be one file and which says why.

- VERIFY at close: 49 test binaries green including the compile proof, which now DENIES WARNINGS
  with three named source-property allowances; port-engine clippy `-D warnings` clean; `delta`
  Green/Unchanged; six-axis receipt fully populated; buck2 Pass 29 / Build failure 20, the known
  third-party export gap unchanged and out of lane.

- UNPUSHED. The branch is 67 commits ahead of `origin/dev` with no open PR — #2117 merged and its
  head ref is gone. The work is committed locally and needs a push and a fresh PR to enter the
  governance pipeline.

## The bare interface, refused with the reason it is owed

- `unmapped type \`interface\`` was the second-largest cause on the board and its refusal said
  nothing: a reader learned that no rule fired, not what was missing. A refusal that cannot be acted
  on is a refusal that will be re-derived by whoever meets it next.

- What is missing is a DECISION, and the point is that it is not one decision. The source's bare
  interface is a value carrying its own type at runtime, and the three things it might become each
  lose something different: a type parameter fixes ONE type per call where the source admits a
  different one at every call; `Box<dyn Any>` keeps the dynamism and loses every operation, because
  the source's callers recover the value by type assertion and the target's must name the type to
  downcast; a purpose-built enum invents a closed set where the source has an open one.

- The 11 direct and 4 nested sites in the surveyed corpora are a type-assertion helper
  (`errors.As`), a database scan target, and the variadic tail of a formatting call — which want
  different answers. A single mapping would be wrong for at least two of the three, which is why
  this stays refused rather than becoming one more table row.

## Two reviewers read the same fingerprint, and they were right

- SEVENTH blind review, on the output after the sixth's findings landed. Still DO NOT MERGE, and the
  reviewer's own summary names why: "Go's vocabulary in its doc comments, Go's type model in its
  signatures, and Go's memory semantics in its arithmetic." Two of those three are standing
  decisions with recorded reasons. The first was a real defect and is fixed.

- `static` BECOMES `const`, reversing a decision this lane made four commits earlier. The argument
  for `static` was that the source's package variable has an ADDRESS and a `const` is materialised
  afresh at every use. It is sound and it protects NOTHING: taking the address of a package variable
  is `&x` of an existing binding, which the engine refuses everywhere, so no emitted code can
  observe the difference.

  What the `const`/`static` split DID carry is what both reviewers independently saw — the source's
  own `const`/`var` split, which exists because the source cannot make a `const` of a struct and can
  of an integer. That is a limitation of the source, not a distinction worth porting, and emitting
  it made the two spellings a fingerprint rather than a decision. The day `&<package variable>`
  translates, this is a `static` again; the decisions are linked and recorded as such.

- THE DOC COMMENTS WERE THE LOUDEST TELL and they were the CORPUS's, carried faithfully. The
  reviewer quoted `globals::PREFIX` — "A string literal is a BORROW in the target, and the owned
  form cannot be built by a constant expression at all" — and said, correctly, that "in the target"
  is porting vocabulary and the comment is a note-to-self rather than documentation. `Driver`'s was
  worse: "Embedded, so Driver's method set includes Run without declaring it" is FALSE in Rust, and
  contradicts the forwarding impl twenty lines below it.

  Fixed in the corpus, and the convention is now explicit: a Go doc comment is attached with no
  blank line and is what a CALLER reads, so it documents the code; the porting reasoning goes in a
  separated `//` block above it, which the source's own parser does not treat as doc and which the
  engine therefore never carries. Same words, same file, one blank line apart, and only one of them
  ships. Also swept the emitted docs for source-language vocabulary — "package", "receiver" — which
  is now zero.

  Worth saying plainly: this was not the engine's defect, and it was poisoning every review of the
  engine. A corpus whose prose says "the target" guarantees a reviewer says "mechanically
  translated" no matter how good the translation is.

- CONSIDERED AND REJECTED, recorded so it is not re-litigated: the reviewer holds that deriving
  `Hash`/`Eq` on a type with `&mut self` methods lets a caller corrupt a map — "insert one as a
  HashMap key, call run(), and the map is corrupted." The target does not permit that: a key is
  owned by the map and no `&mut` to it is obtainable. And the source's own rule for a map key is
  comparability, which mutating methods do not affect. The derive is faithful and safe.

- STILL OPEN from this review, and all real: `use` emission (`Box<dyn Error + Send + Sync>` is
  spelled out eight times and every cross-module path in full); a getter returning an owned `String`
  where the body is a field read; `usize` for a parameter used only as an index, which is the
  counter idiom one level out; and region ORDER, which is alphabetical and which the reviewer read
  as "what an emitter produces from a symbol table" rather than what an author writes.

## The choice the source had to spell as a mutation

- `result := 0; if c { result = a } else { result = b }` emitted `let mut result = 0;` followed by a
  bare block. Two reviewers named it, and the second was exact: "that is the pattern you write when
  your language doesn't have `if` as an expression."

- The source's `if` is a STATEMENT, so a value chosen by a condition has to be written into a name
  declared beforehand. The target's is an EXPRESSION, and the same choice is spelled by initialising
  the name from it. Not a rewrite — the two run the same condition, evaluate the same branch, and
  leave the same value in the same name.

- Three defects removed, not one preference satisfied: the binding stops being `mut`, because
  nothing writes it after it is bound; the initial value stops being emitted, because it is dead on
  every path; and the shape stops reading as a translation.

- THE INIT CLAUSE KEEPS ITS BLOCK, which is what makes this faithful rather than merely tidier. The
  source scopes `if size := len(s); cond` so `size` dies with the branch, and hoisting it to make
  the `if` a bare expression would be the unfaithful move this module has refused from the start.
  The whole block simply becomes the value now instead of a statement:
  `let result = { let size = s.len() as i64; if size > 4 { size } else { size + 1 } };`

- STRICT like the propagation matchers: the `if` must be the VERY NEXT statement, BOTH branches must
  be present, and each must be exactly one plain assignment to the declared name. Without an else
  the initial value is live on one path and the whole argument collapses. A compound assignment is
  refused because `x += e` reads the value this rule is about to stop emitting.

- AN ALLOWANCE CAME OFF THE COMPILE PROOF, which is the measurable half. `unused_assignments` was
  admitted two commits ago with a reason that named this exact translation as the work that would
  remove it. It is removed, and the emitted crate now compiles under `--deny=warnings` with only the
  two allowances that are genuinely properties of the source — an unexported declaration nobody
  calls, and a parameter a function ignores. Writing down what would fix an allowance is what made
  it findable.

## A getter returns a view, and the trait that fixes what a body must satisfy

- `pub fn label(&self) -> String { self.label.clone() }` — five separate accessors doing it, and a
  reviewer counted them. The source's string is immutable and shares its backing, so
  `func (c Counter) Label() string` hands the caller a VIEW and copies nothing. The owned `String`
  clones on every call, which is work the source never does.

  It is the string-parameter rule one position further on, and for the same reason: the value is
  shared read-only data and the target's `&str` is exactly that. `pub fn label(&self) -> &str
  { &self.label }`.

- PROVEN, not assumed: exactly one result of the source's string type, and a body whose EVERY return
  is a field read of the receiver. One return that is anything else — a literal, a computed value, a
  call — and the result is not a view of the receiver at all. The receiver is not checked separately
  because the return shape proves it: the front end marks the one identifier that IS the receiver,
  so a free function reading a local's field fails on the identifier rather than on an attribute.

  Safe against a lifetime it cannot supply because the emitted receiver is always a borrow: a
  pointer receiver that escapes declares no receiver form and refuses, and a value receiver becomes
  `&self`. An owned `self` would make the reference dangle, and the engine emits none.

- IT FIRED WHERE IT MUST NOT, and the compile proof caught it within the minute. A TRAIT IMPL splices
  a signature from the trait's method onto a body from the type's own — so a body built for its own
  signature is wrong for the one it is spliced into, and `fn describe(&self) -> String { &self.label }`
  is what that looks like.

  Fixed by naming the thing that was implicit: a body now says WHOSE signature it has to satisfy.
  `ResultShape::Own` for every case where the two are built together, `Inherited` where a trait fixed
  it and this call exists only for the body it produces. The result idiom applies only under `Own`,
  because a caller written against the trait's spelling is not this method's to change.

  Worth recording as a shape rather than as an incident: every result idiom this engine adds will
  hit the same splice, and the parameter is now there for the next one.

## Source order, because a symbol table is not a module

- Both reviewers read the ORDER as a machine's: "strict alphabetical member ordering... Humans order
  code by importance and call sequence — the constructor first, the primary operation next.
  Alphabetical ordering is what an emitter produces from a symbol table." It was, three times over,
  and each layer had to be fixed for the one below it to show.

  1. The FRONT END sorted package-scope declarations by name, because go/types' scope order is
     alphabetical and something had to make it deterministic. It now ranks them by where the source
     DECLARES them — files in the sorted order they are parsed in, declarations in the order they
     appear within a file — which is just as deterministic and is what an author chose. A name the
     walk never reaches ranks after everything it does, and the stable sort leaves those in the
     order they arrived, so the result is total either way.

  2. The FACADE assembled from a map keyed by region id, so even a correctly ordered snapshot came
     out alphabetical again. It now assembles in the order the transform produced.

  3. The TRANSFORM produced regions rule-major, which puts every struct before every constructor and
     separates a type from the functions that build it. Regions are now ordered by the DECLARATION's
     own position first and the rule's precedence second, so the several regions one declaration
     owns stay adjacent and a unit-level region sorts first, which is where a prelude belongs.

- The rule ORDER itself is pack data and it was arbitrary. Reordered to `const, var, alias,
  defined_type, interface, struct, struct_body, func` — values, then types, then the bodies that
  fill them, then functions, which is how a module is laid out. `struct_body` stays after `struct`
  because the later rule is the more specific one and the survey picks the last match.

- What it buys, read side by side: `MAX_RETRIES, DEFAULT_NAME, ENABLED, THRESHOLD, ID, Celsius, add,
  scale, unexported` — the source's own order — where it read `ID, DEFAULT_NAME, MAX_RETRIES,
  Celsius, add, scale, unexported, ENABLED, THRESHOLD` before. And `struct Counter`, its methods,
  its constructor, then `struct Tally` and its constructor, where the constructors used to sit in a
  block above the type they build.

- Nothing about the program changed, and that is the point: order is the last thing a reader
  notices consciously and the first thing that tells them who wrote it.

## A length is a `usize`, and three result facts become one

- `pub fn size(table: &BTreeMap<String, i64>) -> i64 { table.len() as i64 }`. The source's `len`
  yields its own `int`, which the type map sends to `i64` — right for a value the source TYPED
  `int`, and wrong for a LENGTH, which the target types `usize`. A function that returns nothing but
  a length is returning a length, and the conversion the call's mapping adds exists only to make the
  value type as the source's integer: where the value never is one, the conversion is what is wrong.

  Equivalent because a length is the same set of values in both — the source's cannot be negative
  and cannot exceed what the target's `usize` holds — so no value the function can produce changes.
  A caller that wanted a signed value is a call site that now has to say so, which is a refusal
  where an assumption was.

  Which callees yield a length is PACK DATA (`len`, `cap`), so a pack for another source language
  names its own and the engine names none. The conversion comes off through the same function the
  loop-counter idiom uses, which strips only the trailing form the pack declares.

  `fallible::length` correctly declines: its failing return yields `0`, which is not a length, and
  the proof requires every return to be one rather than most of them.

- CLIPPY CAUGHT THE DESIGN, which is the part worth recording. `Body::new` reached eight arguments
  and `too_many_arguments` fired — and it was right for a reason a lint cannot know: the three
  booleans are not three parameters, they are ONE VALUE, the set of things the signature decided
  that only the body can spend. They are now `ResultFacts`, gathered once and carried, and every
  result idiom after this adds a field there rather than a parameter.

  Three of them landed in three commits, each hitting the same splice and the same agreement
  problem. The shape was visible after the first and the lint made it unavoidable after the third.

## Blanket `wrapping_*`: the objection answered in the data

- FOUR independent reviewers have now called this out, each saying the same true thing: no Rust
  author reaches for `wrapping_add` on `self.calls += 1`, and the spelling disables the debug check
  that would have caught a bug. The objection is right about INTENT and wrong about what the engine
  may do with it — the engine cannot read intent, and the source's overflow guarantee is not
  conditional on the author having wanted it.

- Rather than record the disagreement a fifth time, the alternatives are now in the pack's own
  reason, so the next reviewer's objection is already answered where the decision lives:

  1. Emit the plain operator and accept the debug panic — a different program for every input that
     overflows, and the programs where wrapping is load-bearing (a hash mixer) would abort instead
     of returning.
  2. Type the ported values as the target's wrapping newtype so the plain operator carries the rule
     — reads naturally at the operation and infects every signature, field and caller with a wrapper
     the source does not have. A larger unfaithfulness than the one it fixes.
  3. Have the emitted crate turn overflow checks off in its own build profile — a profile setting
     does not reach a crate's DEPENDENTS, so a ported library used as a dependency is still built
     under the consumer's profile and still panics.

- AND WHAT WOULD CHANGE THE ANSWER, which is the part that makes this a decision rather than a
  refusal to move: a proof that a particular operation cannot overflow. The engine already has one —
  a division whose operands make overflow impossible keeps the plain operator — and every further
  one narrows this by construction rather than by taste. A range analysis over the source is the
  shape of the next, and `docs/programs/k8s-port/census/` sizes no such family, so that is stated
  rather than assumed.

## Clippy on the emitted crate, which is the reviewer this engine can actually run

- THE MOVE THAT MATTERED, and it should have come three reviews ago: run `clippy-driver` on the
  emitted crate and deny its warnings, beside the `rustc` proof that already does. `rustc` proves
  the output is a PROGRAM; clippy is the closest thing to the bar this engine is held to. It is not
  a reviewer — but every lint it raises IS a review comment a human would have written, and a lint
  the engine trips is a defect it can be told about deterministically instead of three reviews and
  three days later.

- It found exactly three things on the first run, and all three were already on a reviewer's list:
  two index loops that should walk their sequence, and a binding that exists only to be returned.
  Nothing else. That is the useful part of the result — the emitted crate was already clean of
  everything else clippy knows how to name.

- A NAME THAT IS ONLY RETURNED is not a name. `x := 0; if c {..} else {..}; return x` is three
  statements for one expression, and the middle name exists only because the source's `if` could not
  produce a value. Where the statement after the choice returns exactly that name and is the last,
  the value IS the answer and the binding goes. Last, because a return that is not last leaves code
  this would silently drop.

- A COUNTED LOOP THAT ONLY REACHES ELEMENTS IS AN ITERATOR. `for i := 0; i < len(xs); i++ { xs[i] }`
  counts because the source has no other way to walk a sequence by value; the target does, and it is
  the same walk — same elements, same order, same number of times. What goes is the counter, which
  existed only to be an index. `xs.iter().copied()`, because the source's index read takes a COPY
  and leaves the sequence usable: consuming it would end its life at the loop, and handing out
  references would give the body a reference where the source gave it a value.

  THE ONE THING THIS RULE INVENTS is the element's name, and the source gives none. It is a
  loop-local binding — no caller sees it, nothing outside depends on it — and the alternative is
  keeping a counter the target does not need. The convention is the sequence's own name with a
  trailing plural removed, `values` → `value`, and where that yields nothing usable or COLLIDES with
  a name the body already binds, the rule does not fire and the counter stays. An invented name that
  shadows a real one would be a different program.

  Fires only where every element COPIES, where the bound is the sequence's own length, and where one
  counter indexes exactly one sequence. Two sequences indexed by one counter is a walk of neither.

## Structural satisfaction, answered where the answer is sound

- Three reviews raised observed-vs-structural and the third made it concrete: `Engine` has a `run`
  method with the exact signature `Runner` requires and does not implement `Runner`. "Anything
  generic over `Runner` rejects `Engine` despite `Engine` having the exact method. This is the
  single most likely thing to bite a user of this crate." That is right, and it is not a matter of
  taste: in the source `Engine` IS a `Runner`, everywhere, with nothing declared.

- The engine emitted an impl only where it SAW the pair used, which produces a crate strictly less
  capable than the source. Now the type-checker is asked directly — `types.Implements`, for the
  pointer as well as the value, because the source's method set for `*T` includes `T`'s and a
  mutating method is only ever in the pointer's.

- SCOPED to interfaces the package DECLARES, and the bound is the decision rather than a
  convenience. Those are the interfaces the package's own author designed, so an accidental match
  against one is that author's own design — and the target's coherence rule allows the impl, because
  the trait is emitted from the same unit. A structural match against an interface from elsewhere is
  a `foreign_satisfaction`, which has its own recorded answer. The empty interface is skipped: it is
  satisfied by everything, which is true and says nothing the target does not already allow.

  This is what makes the census's 80,042-vs-1,316 gap tractable rather than terrifying. Most of that
  gap is matches against interfaces from other packages, which this does not touch.

- STRUCTURAL ranks last among sites, so a pair the source also USES keeps the site that proves the
  most. The impl is identical either way; what differs is only how it came to be known — and the
  receiver-mode union now runs over more implementors, which is correct rather than incidental.

- The standing disagreement is now answered in two places and open in one: a pure supertrait bundle
  gets a blanket impl, a same-package structural match gets a real impl, and a cross-package one
  stays deferred with its reason.

## A doc comment names things as the TARGET names them

- The fourth blind review led with this and called it decisive: "the doc comments still name the Go
  methods, with Go capitalization." Three words — `Run` where the method is `run`, `Refresh` where it
  is `refresh`, and a panic message naming a function that does not exist. Their verdict on those
  three words: "the cheapest possible proof that nobody has [read the emitted Rust]."

- They are right, and the answer is a RULE rather than three edits. The source's documentation says
  `Run` because that is what the method is called THERE; the emitted method is `run`, and prose that
  still says `Run` refers to nothing. So every word in a doc comment that names a declaration of this
  unit is now emitted as the target's name for it.

- EXACT and case-sensitive, which is what keeps it away from English. A method named `Run` does not
  match the word "run" in a sentence, because the two differ; where the source name and the target
  name are the same word the rewrite is the identity; and `Run` inside `Runner` is not a word, so
  it is not touched. What it catches is precisely the case that matters — a capitalised identifier
  standing where the target has a lower-cased one.

- AMBIGUITY IS LEFT ALONE. Two declarations sharing a source name and casing differently — a type
  `Value` and a method `Value` — give the prose no way to say which it means, so neither is
  rewritten. A rule that guessed there would rename half the references wrongly and no one would see
  it.

- This is the third and last thing the doc convention rewrites, and the three now cover the whole
  class the reviews kept finding: the leading repetition of the item's own name, the porting
  reasoning that belongs in a separated comment, and now the identifiers. What remains in an emitted
  doc comment is what the author wrote about the code.

## The swap, the block, and the type the source never wrote

- THE EXCHANGE. `a[i], a[j] = a[j], a[i]` is what the target's sequence has a method for, and three
  reviewers named the destructuring form as hand-rolling something the target has had since 1.0.
  The parallel assignment is already faithful — both sides evaluated before either is written — so
  this is the spelling and not the program. Recognised from the SOURCE nodes rather than the
  rendered places, so a change to how an index prints cannot silently stop the idiom firing.

- THE BARE BLOCK, which two reviewers read as a Go statement form transliterated. It was — and it
  was also the only faithful shape until now, because the source scopes `if size := len(s); cond` so
  the name dies with the branch and hoisting it would delay a drop.

  What makes hoisting safe is the binding's TYPE: a copy type has no drop to delay, so the only
  remaining difference is shadowing, which is checked against every other binding in the body. Where
  either fails the block stays, because it is still faithful.

- WHICH NEEDED A FACT THE FRONT END WAS NOT RECORDING. `var x T` carried its type and `x := e` did
  not, because nothing had needed it. Now something does, so a short declaration records the type
  the type-checker gave it.

  AND IMMEDIATELY A SECOND DECISION, because recording it put `let size: i64 = ..` on every short
  declaration in every body — an annotation the author never wrote and the target does not need,
  since it infers exactly what the source inferred. So the binding also records whether the source
  WROTE the type, and the target annotates only where it did. A fact the front end observes; a
  decision the transform makes from it.

- THE FLAG SET IS SORTED, and appending without re-sorting broke the snapshot digest immediately —
  `flagsFor`'s own comment says a flag set has exactly one encoding, and the admitter proved it
  within a minute. Cheap to fix and worth recording: the encoding invariants in that file are load-
  bearing, not tidiness.

## Three the fifth review proved, and one decision reversed

- AN ACRONYM IS ONE WORD. `type ID = String` carried the source's convention of capitalising a whole
  acronym; RFC 430 is explicit that the target spells one as a word — `Uuid` rather than `UUID`.
  This REVERSES a decision recorded in a test here, whose reason was that lowercasing "would rename
  the type rather than recase it". That reason is wrong: `Id` and `ID` are the same word differently
  cased, and casing is exactly what the naming rule does. Two reviewers read the all-capitals form
  as the source language's convention carried over, which is what it was. `KSUID` becomes `Ksuid`,
  and a letter following a LOWER-CASE one still starts a new word, so `NewKSUID` is `NewKsuid`
  rather than `Newksuid`.

- AN INTERFACE PARAMETER IS GENERIC, not dynamic, and it is one line of pack data. The source's
  interface parameter is a dynamic value because the source has no other kind; the target has both,
  and `&impl Trait` accepts every implementor exactly as the source's does, monomorphises rather
  than dispatching through a table, and needs no allocation. `&dyn` buys heterogeneity a PARAMETER
  cannot use — one call passes one value. Reviewers named `&dyn` at every interface boundary as a
  port artifact and were right: it is what you get by mapping the source's one kind of interface
  value onto the target's one that resembles it, rather than onto the one a parameter wants.

  A RESULT still refuses. `impl Trait` in return position names a single hidden type where the
  source's result may be a different one on every path, which is a different program.

- A BODY THAT PROPAGATES AND SUCCEEDS IS THE CALL. `check(s)?; Ok(())` runs the call, hands its
  failure out, and reports success — every one of which `check(s)` already does. The extra shape
  exists because the SOURCE cannot say it in one statement: its `if err != nil { return err };
  return nil` is two statements and a convention, where the target's return type is the whole
  statement.

  STRICT: the three must be the last three, the success must carry no value, and the call must bind
  nothing. A success carrying a value is a different function from the one it called; a bound value
  means the body used it. And it must run BEFORE the propagation matcher, which would otherwise
  consume the pair and leave the `Ok(())` standing — which is how it was found.

## The engine can now EMIT a package it has never seen

- Five blind reviews have judged the same subject: the hermetic corpus. The fifth named why that is
  the wrong subject — "the module exists to demonstrate scoping rules rather than to do anything",
  "module names describe memory mechanics rather than domain concepts", "the crate uses none of the
  features that distinguish Rust from Go". All true, and all properties of a CONSTRUCT-COVERAGE
  FIXTURE rather than of the translation. A corpus built to exercise constructs necessarily reads
  as one, however good the engine is.

- The goal says the engine ports REAL repos. It could measure one and not emit one: `survey` counted
  what a snapshot would translate to and threw the result away. Now `port <snapshot>` keeps it.

  PARTIAL on purpose. The declarations that refused are absent and the report says how many — a real
  package always has some, and refusing the whole package would leave the engine unable to show its
  work until it was finished. Distinct from `port-go-source`, which runs the strict pipeline over
  the hermetic corpus and refuses if anything in it fails; that strictness is right there, because
  the corpus is the engine's own and a refusal in it is a regression.

- IT PAID IMMEDIATELY. `xxhash` emits, and the first forty lines carry defects the hermetic corpus
  could not have surfaced:

  - `const MAGIC: String = "xxh\x06";` — a `String` const, which does not compile. The override that
    makes a string constant a `&str` is keyed on the source type `string`, and an UNTYPED string
    constant is not that;
  - `fn u64(b: &[u8]) -> u64` — a function whose name shadows a primitive, with a body of
    `binary.little_endian.uint64(b)`: an unresolved selector into a package the emitted crate does
    not have, emitted as a path rather than refused;
  - `fn consume_uint64(..) -> (Vec<u8>, u64)` returning `(&b[8..], x)` — a borrowed subrange where
    the signature says owned.

  Every one of those was counted as TRANSLATED. Which is the finding: the survey's "translated"
  means "the transform produced an item", and nothing compiles it. The hermetic corpus has a compile
  proof and the third-party surveys have none, so the number they report is an upper bound.

- That is the next thing to fix, and it is worth more than any single rule: compile-proof the ported
  third-party output. It will reclassify a chunk of every coverage number downward, which is the
  sixth honest correction of the lane and the one that makes the rest of them trustworthy.

## What a real package actually compiles to, measured

- With `port` emitting region by region — a region the renderer will not take is a refusal
  discovered late, and rendering the package as one tree let a single bad one take the whole package
  with it — all six surveyed packages now emit. Two of them could not emit AT ALL before that, and
  the reason was one defect apiece.

  A CAST IS POSTFIX-HOSTILE. `xs.len() as i64.wrapping_sub(1)` is not `(xs.len() as i64)
  .wrapping_sub(1)`; the target rejects it outright, which is the good failure mode and is how this
  was found. Two whole packages failed to render on it, and the hermetic corpus never had a cast
  with a method on it.

  It needed the cast to be VISIBLE. The pack's `len` form is a text template ending in a conversion,
  and handed to the IR as a flat literal nothing downstream could see that the outermost thing was a
  cast. It is now read from the FORM the pack declares — a template with no trailing conversion is
  left exactly alone — and the length-result rule, which used to strip that conversion by editing
  rendered text, now unwraps the node instead.

- THE MEASUREMENT, which is the point of the exercise. Ported and fed to `rustc`:

  | package | lines | errors |
  |---|---|---|
  | xxhash | 96 | 20 |
  | ksuid | 221 | 66 |
  | uuid | 252 | 57 |
  | xid | 28 | 9 |
  | errors | 173 | 17 |
  | semver | 450 | 57 |

  226 errors, and the taxonomy is one cause: **130 × E0425** (cannot find value in scope) and
  **86 × E0433** (unresolved module or crate) — 216 of 226, 96%. Plus 9 × E0422 (unknown struct) and
  exactly ONE type mismatch.

- ONE CAUSE, and the engine already has the rule for a narrower version of it: what is emitted has
  to be SELF-CONTAINED, which is why a body naming a deferred package variable refuses. That rule
  covers deferred kinds and nothing else. It does not cover a call into another package the emitted
  crate does not have, a stdlib function with no mapping, or a reference to a declaration that
  itself refused — and each of those is emitted as a path that resolves to nothing.

  So the coverage numbers are an UPPER BOUND: `translated` means the transform produced an item, and
  for the hermetic corpus a compile proof backs that up while for a real package nothing does.
  Generalising the self-containment rule — a body that names anything the emitted crate will not
  contain refuses — is the single highest-value change left, and it will move every coverage number
  down. That is the correction that makes the rest of them mean something.

  Only ONE of the 226 is a type mismatch, which is the encouraging half: where the engine can see a
  name, it is getting the types right.

## The unit names its failure type once

- `Box<dyn std::error::Error + Send + Sync>` is the longest thing in most emitted signatures and it
  appears in every fallible one — eight times in a four-hundred-line module. Two reviewers named it
  as the type any author writing it a third time would have aliased, and neither was wrong.

- A unit with a fallible declaration now emits `pub type Result<T> = std::result::Result<T, ..>;`
  and its signatures name it: `pub fn length(s: &str) -> Result<i64>`. An alias is TRANSPARENT, so
  a caller may still write the full type and the two are the same — this changes the spelling and
  not the program.

  PER UNIT, because the engine emits one module per source package and has no crate root of its own;
  a module-scoped `Result` is what a crate organised that way writes anyway. Only where the unit HAS
  a fallible declaration, so a module that never fails does not gain a name it never uses.

- IT NEEDED A KIND OF REGION THE ENGINE DID NOT HAVE: a PRELUDE, which belongs to no declaration.
  Every region until now traced to one, and this one is decided by a property of the whole unit —
  whether anything in it can fail — which no per-declaration rule can see. Synthesised after the
  declaration loop and ordered at position -1, which is what the region ordering's "a declaration
  with no position sorts first" clause was already reserving space for.

- AND THE TURBOFISH WENT WITH IT. The pack's form for the source's failure constructor is explicit
  about the type, and its reason said why: the mapping fires in any position and an inferring
  conversion only works where the destination is known. Inside `Err(..)` of a function whose return
  type names the failure, it IS known — so the pack now declares a second form for exactly that
  position, and `Err(Box::<dyn std::error::Error + Send + Sync>::from(ERR_EMPTY))` is
  `Err(ERR_EMPTY.into())`.

  Both forms are the pack's, and the shorter one is applied by matching the general form's own
  template rather than by editing rendered text — so a pack that changes one changes both, and a
  pack that declares no second form gets the first everywhere, exactly as before.

## An index parameter is the target's index type

- The reviewers' first provenance tell, four reviews running: `i64` for every integer, including
  lengths and indices, with `as usize` inserted wherever the target's real types collide with that
  choice. `pub fn swap(values: &mut [i64], i: i64, j: i64) { values.swap(i as usize, j as usize) }`
  was the whole argument in one line.

- A parameter used for NOTHING BUT indexing is a `usize`. Same proof the loop counter uses, one
  scope out: a name whose every read is an index operand never has its signed value observed. Gated
  by the SAME pack idiom, so a pack that drops it gets the conversions back in both places or in
  neither.

  Stricter, never different: no value the source accepts there is one the target does not. A
  NEGATIVE argument the source would take and then reject with a bounds check is one the target now
  rejects at the call, which is the same program failing earlier.

- THE RIPPLE IS THE POINT, and it is why this waited. A parameter's value comes from a CALLER, so
  changing its type changes the call. The signature table already carried each parameter's target
  type for the ownership decision, so an argument crossing into an index type converts there — the
  same conversion an index operand makes, at the one other place a value enters that type.

  Without it the callee would have a signature its own callers could not satisfy, which is a worse
  defect than the conversion this removes. That is the shape of every parameter-side idiom: the
  signature, the body and the call site all have to move together, and the signature table is what
  lets them.

- Result: `pub fn swap(values: &mut [i64], i: usize, j: usize) { values.swap(i, j); }`.

- REAL PACKAGES, re-measured after this and the failure alias: xxhash 20, ksuid 66, uuid 60, xid 9,
  errors 22, semver 66 rustc errors. Up slightly from 226 to 243 in total, and the reason is worth
  recording rather than hiding: the alias and the index parameter both changed signatures, and a
  signature that changes while the names it references still do not resolve produces the same
  unresolved-name error at a new place. The 96% cause is unchanged and untouched, and nothing here
  was going to move it.

## Self-containment, and the seventh honest correction

- 216 of 226 compile errors on six real ported packages were unresolved names, and the cause was one
  line: a call into ANY package became `crate::<module>::<name>`, whether or not that package was in
  the model. The engine already had this rule for a narrower case — a body naming a DEFERRED package
  variable refuses, because "what is emitted has to be self-contained" — and it covered deferrals
  and nothing else.

- Now the resolver knows every unit the MODEL has, which is every module the emitted crate will
  contain. A call into one of them is a path; a call into anything else refuses by name, saying that
  the package is not in the snapshot and that the pack has to map it, as it maps the other calls into
  libraries that do not come along.

- THE CORRECTION, and it is the largest of the lane: xxhash 73.5 → 47.1, semver 51.7 → 32.8, ksuid
  49.5 → 38.7, uuid 46.4 → 40.2, errors 47.4 → 36.8, xid 30.8 → 23.1. Every one of those points was
  a declaration counted as translated while emitting a path that resolves to nothing.

- AND THE PROOF IT WAS RIGHT: rustc errors on the emitted packages fell from 243 to 110, and
  **semver's ported output now compiles with zero errors** — the first time a real third-party
  package's ported subset has compiled at all. 62 lines of it, and they are 62 lines a reviewer can
  be handed.

- WHAT REMAINS in the 110 is the same rule one step further in: 78 × E0425 is a body naming a
  declaration of its OWN unit that itself refused. That needs a fixpoint — a declaration is emitted
  only if everything it names is emitted — and it is the next thing.

- The number that matters did not change: nothing here made the engine translate more. It made it
  stop claiming to.

## The fixpoint, and the eighth honest correction

- After the package rule, 78 of the remaining 110 compile errors were the SAME rule one step further
  in: a body naming a declaration of its OWN unit that itself refused. The emitted crate then has a
  call to a function it does not contain — the same defect as naming another package, arrived at
  from the inside.

- Deciding it needs a FIXPOINT, because refusing one declaration may make another refuse, which may
  make another. Starting from "everything is emittable" and SHRINKING is the whole of the design:
  the set only ever loses members, so it converges; and it gets MUTUALLY RECURSIVE functions right,
  because both translate and neither is removed. Starting from empty and growing would also converge
  and would refuse them both, since on the first round neither can see the other.

  Only a REFUSAL removes a name. A declaration nothing captures is deferred or uncaptured, and
  neither means the name is absent for a caller's purposes.

  The strict pipeline needs no iteration and does none: it requires every declaration to translate,
  so it passes every name and behaves exactly as before.

- Then the same gate at the two remaining places a name can enter the crate — a TYPE this unit
  declares and is not emitting, and an IDENT naming a package-scope declaration that refused. Three
  sites, one rule, and the third was the last.

- THE CORRECTION, cumulative over both rounds: xxhash 73.5 → 38.2, ksuid 49.5 → 21.5, uuid 46.4 →
  26.8, errors 47.4 → 15.8, xid 30.8 → 15.4, semver 51.7 → 32.8.

- AND WHAT IT BOUGHT, which is the number that was worth having: rustc errors on the six ported
  packages fell from 243 to 29, across 327 emitted lines. `semver` compiles with ZERO errors and
  `xid` with one. Before this session the engine could not emit a real package at all; it now emits
  six, and one of them builds.

- The coverage numbers halved and they are now worth something. Every point removed was a
  declaration counted as translated while emitting a name that resolves to nothing — which is not a
  translation, it is a claim.

## Two bugs in the doc-rename rule, found by looking at a real package

- Putting `semver`'s ported output in front of a reader — 62 lines that compile with zero errors —
  found two defects in the rename rule added three commits earlier, neither of which the hermetic
  corpus could show.

- `r#true`. The map recorded every CHILD of a declaration as a member, and a declaration's children
  are its whole tree: the initialiser `= true` registered as a name, so a doc comment saying "when
  set to true" came out saying `r#true`. A name is a member only if it is DECLARED as one — a field
  or a method — and that is what it records now.

- `Allowed`. The target's name for a declaration depends on its KIND, and the rule used one casing
  for all of them: a constant came out in a type's casing, `allowed` → `Allowed` where the emitted
  constant is `ALLOWED`. Naming it wrong is worse than not naming it. It now uses the same rule that
  emits the name — shouted for a value, snake for a function, pascal for a type.

- AND THEN THE REAL BOUND, which the first fix only exposed: `allowed` is an ENGLISH WORD, and the
  package has a private constant of that name, so "not allowed in a valid semantic version" became
  "not ALLOWED". The rule now renames only what the source EXPORTS — the source capitalises what it
  exports, so a capitalised word in prose matching an exported name is a reference to it far more
  often than not, and that is the case the rule was built for. An unexported name is lower-case and
  indistinguishable from English.

  What is left is bounded and small, and worth stating: even a false positive changes the CASING of
  a word and never its meaning, because the rename is always the same word in the target's own
  convention.

- The lesson is the one the whole session keeps teaching: a rule that looks obviously right on a
  corpus built to exercise it can be obviously wrong on the first real package it meets. Three
  commits of confidence, and sixty-two lines of real output to correct it.

## "Never written" is a fact about ONE package

- A reviewer reading the ported `semver` found `pub const COERCE_NEW_VERSION: bool = true` under a
  doc comment saying "when set to true, new_version will coerce" and "this is used when
  COERCE_NEW_VERSION is set to false". A `const` cannot be set by anyone, ever. They called it a
  translation that preserved syntax and dropped semantics, and they were right.

- The rule said a package variable NOTHING WRITES is a constant, and "nothing writes it" was a fact
  about the package's OWN code. An EXPORTED package variable is part of the source's API: anything
  that imports the package may assign to it, the package's own documentation frequently tells you
  to, and the engine cannot see any of it. So it is the mutable global the undecided form is about,
  arrived at from outside rather than from within — and it refuses, with that said.

  Unexported, the fact holds: nothing outside the package can name it, so "this package does not
  write it" IS "nothing writes it".

- A SENTINEL is exempt, and the exemption is the sentinel decision rather than a new one. It becomes
  its MESSAGE, and the message is constant however the variable is reassigned: reassignment changes
  which failure value the NAME holds, which is identity — and identity is what the sentinel decision
  already records as lost, with its cost written down.

- The corpus moved with the rule: the package variables that demonstrate the const form are
  unexported now, which also proves the second half — a private source variable does not become
  public API — and an exported one sits in the refusal corpus.

- semver 32.8 → 29.3, and its ported output still compiles with zero errors. The other five are
  unchanged, because none of them has an exported never-written non-sentinel variable.

- This is the fourth time a rule that was obviously right on the hermetic corpus has been obviously
  wrong on the first real package it met, and the third time the correction came from LOOKING at
  emitted output rather than from a count. The corpus proves a rule fires; only a real package
  proves it should.

## Two more the real package proved

- `pub mod v3`. The source spells a module's major version as a trailing PATH SEGMENT —
  `github.com/x/semver/v3` — and the package that path names is still `semver`. Emitting `pub mod
  v3` names the module after a versioning convention the target does not have, where versions live
  in the manifest; it would be wrong again the moment the source went to v4. Recognised strictly: a
  `v` followed by digits and nothing else, and only where a segment precedes it, so a package
  genuinely named `v3` keeps its name. Now `pub mod semver`.

- A THREE-WAY COMPARISON is the target's ordering type. `fn compare_segment(v, o) -> i64` returning
  -1, 0 or 1 is how the source spells one, because its sort and its comparison interfaces are
  defined in terms of a signed integer. The target has a type for exactly this, and it is the type
  ITS sorting and ITS `Ord` are defined in terms of.

  The integer form is not merely unidiomatic there — it is the bug class the type exists to remove:
  every consumer re-derives which way round the sign goes, and one inversion in that chain compiles
  and sorts backwards. A reviewer said so of `compare_segment` before knowing where it came from.

  Every return must be one of the three literals, because those three are the whole range: a
  function that can return anything else is not this shape. A returned VARIABLE could hold anything,
  and proving otherwise is a range analysis the engine does not have — so it does not qualify.

  `fn compare_segment(v: u64, o: u64) -> std::cmp::Ordering` with `Ordering::Less`, `Greater`,
  `Equal`. The ported `semver` still compiles with zero errors and now passes clippy clean.

## A real package that compiles clean and lints clean

- `match x { "x" | "*" | "X" => true, _ => false }` is a membership TEST, and the target has a macro
  that is exactly one. The source spells it as a switch because its switch is the only multi-pattern
  form it has. Recognised from the ARMS AS BUILT rather than from the source shape — what matters is
  what the arms yield, and that is known only after they are translated — and only where the default
  yields `false` and every other yields `true`. The reverse is a NEGATED test and needs its own form;
  inverting it silently would be a different expression wearing this one's shape.

- WITH THAT, the ported `semver` compiles with ZERO rustc errors and passes clippy with ZERO
  warnings. A real third-party package, emitted by an engine that had never seen it, that a Rust
  toolchain has nothing to say about.

  That is not the goal's bar — a reviewer is — but it is the first time any of this has been true,
  and every one of the four rules that got it there came from LOOKING at the output: the tail
  switch, the ordering, the module name, and this.

- THE PATTERN OF THE WHOLE SESSION, stated once so it is not re-derived: the hermetic corpus proves
  a rule FIRES; only a real package proves it should. Four rules that were obviously right on the
  corpus were obviously wrong on the first real package they met — the doc rename twice, the
  package-variable constant, and the cast under a method call — and in each case what found it was
  reading sixty lines of emitted output rather than reading a number.

## MERGE WITH CHANGES — the first non-rejection in seven reviews

- The seventh blind review, on the ported `semver` rather than on the hermetic corpus, returned
  **MERGE WITH CHANGES**. Six reviews before it returned DO NOT MERGE. Nothing about the bar moved;
  the SUBJECT did, from a construct-coverage fixture to a real package.

- And the reviewer wrote, unprompted, the sentence this lane exists to earn:

  > "`is_x` uses `matches!(x, "x" | "*" | "X")`, which is genuinely idiomatic Rust and is NOT what a
  > naive transliteration of Go's switch produces. Either the translator has a switch-to-`matches!`
  > rule, or a human touched that function specifically. It is the only construct in the file that
  > reads as native."

  It has the rule. That is what an engine rule is FOR, and it is the first time a reviewer has said
  so of anything the engine emits.

- THE EVIDENCE LIST HAS CHANGED SHAPE, which is the more useful signal. Of their ten items, four are
  the SOURCE'S OWN PROSE carried faithfully — `errors.Is` in a doc comment, `uint64` in a doc
  comment, the `_e` suffix on method names, the upstream typo in a sentence — and one is the sentinel
  decision with its cost already recorded. Those are not translation defects; a hand port that
  preserved the source's documentation would have every one of them.

  What is left as ENGINE work is three things: `i64` for length constants, the `pub mod` wrapper, and
  item ordering across a package that was several files.

- THE LADDER GOES WITH THE ORDERING. `if v < o { -1 } if v > o { 1 } 0` is how the source spells
  `cmp`, and it spells it that way because it has no such method — the reviewer put it exactly, "the
  ladder is NECESSARY there and dead here". Recognised strictly: two `if`s over the SAME pair in the
  same order, one `<` and one `>`, each returning the matching extreme, and a trailing equal.
  `fn compare_segment(v: u64, o: u64) -> std::cmp::Ordering { v.cmp(&o) }`.

- AND AN UNTYPED CONSTANT TAKES ITS DEFAULT TYPE. `const magic = "xxh"` is `untyped string` in the
  source and matched nothing in the pack, so it emitted `const MAGIC: String` — which does not
  compile. `types.Default` is the source's own answer to "what type does this take when it must have
  one", which is exactly the question a target declaration asks.

- TWO REAL PACKAGES now compile with zero rustc errors AND pass clippy with zero warnings: `semver`
  and `xid`. Total across all six: 23 errors, down from 243 when `port` first ran.

## Holding the bar through the review responses

- Six files crossed 300 during the last stretch. Split along real question boundaries, and the
  boundaries are worth naming because they are the shape the engine has grown into:

  - what a body translation NEEDS versus the DISPATCH that spends it (`body.rs` / `body_stmt.rs`);
  - whether a failing return MAY become `Err` versus what it BECOMES versus what its operands have
    to be (`failure_proof.rs` / `body_failure.rs` / `body_operand.rs`);
  - a PARAMETER's shape, decided by what the caller keeps, versus a RESULT's, decided by what the
    body can prove (`params.rs` / `results.rs`), and the index parameter proved from how the body
    USES it rather than what it produces (`index_params.rs`);
  - measuring what translates versus deciding which translations SURVIVE self-containment
    (`survey.rs` / `reachable.rs`) versus how a refusal is ranked (`survey_cause.rs`);
  - the ladder that is the target's `cmp` written out (`comparison.rs`).

- Every one of those pairs was one file until a rule arrived that needed only half of it. That is
  what the size bar is actually for: it does not make the code smaller, it makes the second question
  visible when it turns up.

## The module is the FILE, and the prose names types as the target does

- The seventh review ranked `pub mod semver { .. }` wrapping the whole output fourth among the
  reasons it reads as translated: "Go's `package semver` header transliterated into a Rust block. A
  Rust author writing `semver.rs` never adds this; a translator that maps Go package → Rust module
  always does." Right — and it was an artifact of the FACADE, which assembled every unit into one
  stream because that was all it had ever needed to do.

  `port` now emits one FILE per unit, which is what a crate laid out this way is. No wrapper, no
  block, no header. The module is the file.

  The hermetic pipeline keeps its single-stream assembly, and should: it exists so a golden can be
  diffed and a compile proof can run on one input, and neither wants a directory.

- AND THE PROSE NAMES TYPES AS THE TARGET DOES. The reviewer found "the maximum value of a `uint64`"
  in a doc comment beside a signature that correctly said `u64`, and said the code was translated
  and the documentation was not. Exactly right, and it is the same rule as the identifier rename one
  step further: a doc word matching a source TYPE name gets the target's spelling.

  A DELIBERATELY SMALL SET, and the bound is the substance: `uint64`, `int32`, `float64` and their
  kin are unambiguous in English, so a word matching one is naming a type. `string`, `bool`, `byte`,
  `int` and `error` are ABSENT for the opposite reason — each is an ordinary English word, and
  rewriting "returns a string of characters" into "returns a String of characters" would make the
  prose worse to fix nothing.

  The full type map is NOT used for this and must not be: it answers what a declaration's type
  BECOMES, where every entry is correct, and prose is not a declaration.

- NOT DONE, and recorded as unprovable rather than skipped: `ERR_INVALID_SEM_VER` should be
  `ERR_INVALID_SEMVER`, because `SemVer` is one word. The engine cannot know that. `SemVer` and
  `MaxLen` are the same shape — two capitalised runs — and one is a single concept while the other
  is two words. There is no fact in the source that separates them, so the split stays.

## A constant that is a LENGTH, and where the evidence list finally stands

- `const maxVersionLen = 256` is the source's own integer and the type map sends it to the target's
  signed one — right for a value the source typed that way, wrong for a bound on a length. Every
  guard then reads `s.len() as i64 > MAX_VERSION_LEN`: a cast per call site, one chance each to get
  the direction wrong, and because the constant is public the casts leak to every caller. Two
  reviewers called it the most consequential finding in the file.

- Proved from the WHOLE UNIT rather than from the declaration: a constant is a length when
  everything that reads it compares it against one. One read that is anything else — arithmetic, an
  argument, a return — and the signed value IS observed somewhere, so it keeps its type.

  AT LEAST ONE READ IS REQUIRED, and that guard is doing real work rather than being defensive: in
  the surveyed packages every guard that reads those constants currently refuses, so the rule sees
  no reads and correctly declines. "Every read qualifies" over none of them is vacuously true and
  would retype constants on no evidence at all. The rule is latent there and proven in the corpus —
  `pub const MAX_WIDTH: usize = 8;` with `s.len() < MAX_WIDTH`, no cast on either side.

  The constant's declaration and every comparison read the SAME proof, so a guard cannot end up
  comparing two different types.

- WHERE THE EVIDENCE LIST STANDS, after the eighth review — the second consecutive MERGE WITH
  CHANGES, on a FILE rather than a module block. Eight items, and the reviewer marks five of them
  "plausibly human" themselves. Of the three they call conclusive:

  - `errors.Is` and `inc_major_e` in a doc comment. The SOURCE'S OWN PROSE, and their argument is
    that a human would have stopped there — which is right, and it is an argument about the ERROR
    MODEL rather than about the prose. The engine substitutes names; it does not rewrite claims. A
    claim that is false of the emitted code is a consequence of a decision recorded in the pack, and
    the honest fix is that decision.
  - The doc-prefix strip leaving a grammar break. The source's sentence is "ErrInvalidSemVer is
    returned a version is found to be invalid when being parsed" — it is missing a "when" upstream.
    Removing "X is" leaves it no more broken than it was. The strip is the right translation of a
    convention the target inverts, and this example is the source's own defect travelling.
  - `i64` for the limits. Answered above.

- What that leaves as open ENGINE work is one thing: the error model. Everything else on the list is
  either the source's own text carried faithfully, a decision already recorded with its reason, or
  something the reviewer marks as a choice a human porter would make too.

## The error model, which is not an enum — and the identity that came back

- Five reviewers asked for an error enum. MEASURED on `semver`, an enum generated from its seven
  sentinels would cover 16 of 78 failure sites: 43 are built by `fmt.Errorf` with a formatted
  message and have no variant to be, 17 return a binding, 2 construct inline. A `#[non_exhaustive]`
  enum that silently omits four fifths of what a caller can receive is a WORSE API than the boxed
  trait object, not a better one.

  The source's failure set is NOT CLOSED, and an engine may not close it on the author's behalf.
  What the reviewers want is a design decision the source never made — which is exactly the thing
  this engine refuses to invent. What would change it: a package whose every failure IS a declared
  sentinel has a closed set and deserves the enum. Across the surveyed corpora that is none of them,
  and `errors`, `multierror` and `xid` declare no sentinel at all.

- BUT THE COMPLAINT UNDERNEATH IT WAS REAL, and it was one the engine had recorded as a cost:
  `errors.Is(err, ErrSize)` works in the source and the port lost it. The port was strictly less
  capable than the source, and five reviewers said so in different words.

- SO A SENTINEL BECAME A TYPE, reversing a decision this lane made twenty commits ago. The message
  form — `static ERR_SIZE: &str`, the failure built from it at each return — was correct and cheaper
  and lost the one thing that makes a sentinel a sentinel. It is now a unit struct that displays the
  source's message and implements the target's error trait, which is what a Rust author writes for
  exactly this and is what the source's caller is comparing against:

      #[derive(Debug, Clone, Copy, PartialEq, Eq)]
      pub struct ErrEmpty;
      impl std::fmt::Display for ErrEmpty { .. f.write_str("empty") }
      impl std::error::Error for ErrEmpty {}

      return Err(ErrEmpty.into());

- AND THE COMPARISON CAME BACK WITH IT. `err == ErrGone` refused by name for most of this lane, and
  the refusal was right for as long as a sentinel was its message. Now `err.downcast_ref::<ErrGone>()
  .is_some()` — true in exactly the cases the source's is. The refusal corpus that proved the cost
  is now a corpus that proves the capability, at 100%.

  RECORDED, not hidden: the source's `errors.Is` walks an unwrap chain and a downcast does not, so a
  package that wraps its sentinels and tests through the wrapper gets a test that says no where the
  source said yes. No surveyed corpus does it.

- A FAILURE PARAMETER GOT ITS OWN FORM, and the reason is what callers do with it. Every other
  interface parameter is `&impl Trait` — accepts every implementor, monomorphises, no table. A
  failure parameter is `&(dyn std::error::Error + 'static)`, because the question callers ask of one
  is `downcast_ref`, which exists on the trait OBJECT and not on a generic. `&impl Error` accepts
  the value and then cannot answer the only question anyone asks of it.

  `Send + Sync` are deliberately absent: the owned form needs them so a ported failure can cross a
  thread boundary, and a BORROW crossing nothing does not.

## R1c — the four defects that put a name in the output the crate does not have

Every package the engine emits now compiles with **zero rustc errors and zero clippy warnings** under
`--deny=warnings`: semver, xid, xxhash, ksuid, uuid, errors, multierror. Four causes, each one a rule
rather than a repair, and each found by compiling real output rather than by reading it.

**A guard placed below an early return is not a guard.** The front end learned to classify an
identifier that names a PACKAGE (`case *types.PkgName: return "package"`), and the body refused one
the snapshot does not contain. It never fired. `refuse_deferred_reference` opens with a `match` on
the reference kind whose arm list is one entry long and whose fall-through is `return Ok(())` — so
every kind but `package_var` left the function before reaching the new check. The classification was
right, the refusal was right, and the order made the refusal dead code. It is now the first thing the
function does. Cost: xxhash 38.2% → 29.4%, ksuid 21.5% → 18.3%. Every point removed was
`binary.little_endian.uint64(b)` — a path into a crate the output does not have, counted as
translated.

**Two spellings of one claim, one of them checked.** `crate::<module>::<Name>` is a CLAIM that the
emitted crate has that module, true only for a unit of this model. Two functions spelled it.
`resolve_node` checked; `named_path` had a bare fallthrough that spelled it for any non-empty
package. That is how `&impl crate::fmt::State` reached the output — a trait-object position resolves
through `named_path`, so the gate that covered the plain named type and the local one missed the
third path entirely. Both now call one `foreign_path`, so a fourth caller cannot repeat it. `errors`
went from 6 rustc errors to 0 and its coverage from 15.8% to 0.0%, which is the honest number: every
declaration in that package reaches through `fmt`.

**A constant AT a defined type is constructed at it, not assigned to it.** `const Person Domain = 0`
needs no conversion in the source, because an untyped literal takes whatever type the declaration
names. The target's newtype is a distinct type and `pub const PERSON: Domain = 0;` does not
typecheck. This is the same operation the conversion path already performs for `Domain(x)`; it was
missing here only because a constant reaches its type by declaration rather than by call. Nine of
uuid's constants came out ill-typed for exactly that reason. Gated on the type being one the unit
DEFINES and EMITS — a mapped type is a target type the literal already is, and a refused one is not
there to construct.

**The reference a loop needs is not a token to add unconditionally.** Ranging borrowed the sequence
so the loop would not consume it — correct, and wrong for a slice parameter, which arrives borrowed
because the pack's slice idiom decided it is `&[T]`. Borrowing it again yields `&&[T]`, which is not
an iterator. The loop now reads `Body::borrowed`, the set the SIGNATURE already computed, rather than
deriving a second answer that could disagree with it.

**Coverage after:** xxhash 29.4, semver 29.3, uuid 26.8, ksuid 18.3, errors 15.8→0.0, xid 15.4,
multierror 0.0. The number fell again and the output got correct again, which is the trade this
engine exists to make: a declaration that emits a name resolving to nothing was never translated.

**Recorded, not fixed — outside this lane.** `buck2 test //build/port-engine/...` passes 29 and fails
to BUILD 20, on `third-party//:prettyplease-0.2` is not visible to `port-engine-rust-ir`. It fails
identically at HEAD with this branch's changes stashed, so it predates this work. `prettyplease` is a
workspace dependency (root `Cargo.toml`), so reindeer would mark it public; `third-party/BUCK` is
simply stale. The fix is `scripts/ci/regen-third-party.sh`, which needs a reindeer binary this
machine does not have, and which rewrites a generated face owned by another lane. Not hand-edited:
patching a generated file is the same mistake as hand-tuning emitted output.

## R1d — two of the review's findings became rules, two became written reasons

A ninth blind review of the ported `semver` returned DO NOT MERGE with four new items. Two are real
engine rules and are now in. Two are not, and the reason each is not is written here rather than
re-derived next time.

**The `Err` prefix is a workaround for a problem the target does not have.** The source names a
sentinel `ErrEmptyString` because it has no namespacing inside a package; the target has modules, so
`semver::EmptyString` says everything `semver::ErrEmptyString` does. And the prefix costs something
there that it does not cost in the source: the target's failure arm is literally called `Err`, so
`Err(ErrEmptyString)` stutters at every single return. Dropped under three conditions, each one a
case where dropping it would guess or lose: the pack declares a prefix, what remains is not empty,
and no other declaration in the unit already emits that name. Three sites needed the same answer —
the declaration, the return that constructs one, and the downcast that tests identity — so it is
answered once on the resolver. Two of them agreeing on a rename the third missed would not compile.

**A module that names one std module twenty-one times imports it once.** Seven sentinels spelling
`std::fmt::Display`, `std::fmt::Formatter` and `std::fmt::Result` is what the reviewer meant by "what
a code generator emits, not what a person types nine times." The import is derived from the items the
unit ACTUALLY EMITTED, never from what it declared, and that distinction is load-bearing: an unused
type alias is dead code, which the compile proof allows, and an unused import is a warning, which it
denies. A unit whose sentinels all refused must not gain an import for them.

**And the gap that finding uncovered, which is larger than the finding.** There are two emission
paths — the plan-driven assembly and the survey — and only the assembly had a prelude at all. Every
package `port` has ever emitted was missing it: no `Result` alias, so every fallible signature spelled
`Box<dyn std::error::Error + Send + Sync>` in full, which two earlier reviewers named and which the
engine had already fixed on the other path. The prelude is now one per-unit decision both paths ask.
`PortedRegion::position` became signed to hold it, which is what the assembly path had always done
and the survey path could not express.

**Declined: Go's package visibility is wider than Rust's module visibility.** The reviewer ranked this
second and reasoned correctly from what they were shown — a bare `fn` is visible to one module and
its descendants, a lowercase Go identifier to every file in the package, so mapping unexported to
private looks strictly narrowing. It is not, here: this engine emits one module per PACKAGE, never
per file, so an unexported name is private to exactly the scope it was package-visible in. The
finding is an artifact of how the file was framed to the reviewer, not of the output. No rule.

**Declined: `uint64` inside a `Display` string.** `f.write_str("version increment would overflow
uint64")` names a type the target does not have — true, and it is not a translation defect, because
that string is the program's OUTPUT. Rewriting what a program prints is changing the program, not
porting it. The line is clean and worth stating once: the prose type-name rule rewrites DOC COMMENTS,
which describe the code, and never string literals, which the code emits. A caller matching on that
message would break, and this engine's whole purpose is not to produce output that means something
different.

All seven real packages still compile with zero rustc errors and zero clippy warnings, per file:
semver, xid (2 files), xxhash (2 files), ksuid, uuid, errors, multierror.

## R1e — the formatting call, ranked first by packages blocked

Building a string from a template is the most common call in real source after the plain one. It is
how nearly every error message in every real package is made, and it appeared in **six of the seven**
surveyed packages — `fmt.Errorf` fifty times in `semver` alone. Every one of them refused. Measured
and ranked before it was built, exactly as the method says: not by count in one package, but by how
many packages the cause blocks.

**Why it needed its own mechanism, not another row in the function table.** The existing table
substitutes rendered arguments into a template the PACK wrote. A formatting call has to read the
template the SOURCE wrote, translate every verb in it, and establish that what comes out means the
same thing. No table of fixed forms can express that, because the form differs at every call site.

**The verb set is CLOSED, and that is the whole safety argument.** `%v`, `%s` and `%d` all render a
value in its default form, which is what `{}` does; `%q` quotes and escapes, which is `{:?}`.
Everything else refuses BY NAME. `%x` is a base, `%T` is a type name the target has no runtime access
to, `%+v` and `%#v` are source-specific value dumps, and any width, precision or flag is a layout the
target spells differently. Defaulting an unknown verb to the plain placeholder would produce a
program that compiles and prints something else — the one failure this engine exists to prevent.

**`%w` refuses for a different reason, and it is the interesting one.** It is not a rendering at all:
it records the argument as the new error's CAUSE, which is what `errors.Is` and `errors.Unwrap` walk.
Rendering it as `{}` would compile, print the identical text, and silently drop a chain callers
navigate. The failure is invisible in the output and total in the semantics.

**Text assembly was the wrong mechanism and had to go.** The first working version substituted
arguments as TEXT, and every real call still refused: the arguments to a formatting call are field
reads, method calls and indexes, none of which has an unambiguous text spelling. `RustExpr::MacroCall`
carries them as expressions, so precedence is the IR's problem, which is where it belongs.

**Two defects the corpus caught that reading would not have.** The front end records a literal as the
source's own SPELLING — right for a literal that passes straight through, since the emitted tree is
parsed. A template is read and rewritten, so re-emitting the spelling put the source's quotes inside
the target's template: `format!("\"count {}\"", n)`. The literal is now decoded, against a closed
escape set holding only what both languages spell identically; `\a`, `\v` and the octal form refuse.
And a template with no placeholders and no arguments is not a formatting operation at all, whatever
the source spelled — it is the string itself, and invoking the macro for it is a use the target's own
lints name.

**Corpus.** `corpus/formatted` proves all seven shapes: one value, several, the quoted verb, no
values, the escaped percent, the literal brace, and the failure form. It was added only after the
engine handled every one of them.

**What the gates caught, and why that is the point.** Four separate fences fired on this change
before any of it could land: the engine-digest manifest (a new source file the digest did not hash),
the corpus admission set, and two neutrality/architecture fences (a source file no fence reads). Each
one named exactly what was missing. That is four chances to ship a change with no receipt axis
accounting for it, all closed automatically.

Coverage is unchanged on every package, and that is expected: these declarations refuse on several
constructs each, and the formatting call was one of them. What moved is that `fmt.Errorf` and
`fmt.Sprintf` no longer appear in any refusal reason anywhere in the corpus.

**Files split to hold the 100–300 bar:** `rule_format.rs` out of `rule.rs`, and `lower_precedence.rs`
out of `lower_expr.rs` — the second a real seam rather than a size cut: what a node BECOMES and where
the grammar needs it BRACKETED are different questions, and getting the second wrong reassociates
silently rather than failing to compile.

## R1f — two findings a rule about the CODE would never have reached

A ninth blind review verified its provenance claim against upstream rather than inferring it, which
makes its evidence sharper than any before it. Two items were provable and are now in.

**A blank line between declarations.** Eighty-seven lines of items with nothing between them. The
formatter will never fix this — `prettyplease` preserves no blank lines because the tree it renders
has none — so no rule about the code could have reached it. Each region is one declaration's output,
a type and the impls that belong to it, so the break goes BETWEEN regions and never inside one: a
break inside would split a type from its own impl. The reviewer said seven near-identical error
blocks were unscannable without it, and that this is not how authored code is laid out. Both true.

**A read that only RENDERS a value is neutral evidence.** The length-constant proof requires every
read of a candidate to be a comparison against a length — one read that is anything else means the
signed value IS observed somewhere. That was too strict in exactly the case the rule exists for.
`semver` compares each of its bounds against a length once and then names it in the message that
reports the breach; the message read counted as evidence against, so all three stayed signed. A
formatted read observes a value and not its type, and the same non-negative literal renders
identically whichever integer it is. `MAX_CONSTRAINT_LEN`, `MAX_CONSTRAINT_GROUPS` and
`MAX_VERSION_LEN` are now `usize`.

Note what made that reachable: the formatting rule landed one phase earlier. Until the pack could
say which callees render their arguments, "this read only formats the value" was not a fact the
engine had. A reviewer named the signed bound three separate times before the engine could see why.

**Declined again, with the reason sharpened.** The reviewer cited `uint64` twice — once in a doc
comment and once in a `Display` string. The doc comment was already correct; the prose rule had
rewritten it, and the citation was to the string. The string stands: it is the program's OUTPUT, and
rewriting what a program prints is changing the program. The line holds — prose that DESCRIBES the
code is rewritten, text the code EMITS is not.

**Named as the next provable finding, not yet done.** A doc comment that names a declaration the
crate does not emit — `inc_major_e`, `inc_minor_e`, `inc_patch_e`, whose methods refused — is prose
describing an API that is not there, which is the self-containment rule at the prose layer. The same
sentence also names `errors.Is`, a foreign package's function. The engine has both facts already: the
rename map knows the target name and `emitted` knows whether it is there. What stops it today is that
`docs_of` is infallible everywhere it is called, so refusing from inside it is a signature change
across the item faces rather than a rule. Worth doing; worth doing deliberately.

All seven real packages still compile with zero rustc errors and zero clippy warnings.

## R1g — self-containment reaches the prose

Two blind reviews in a row ranked the same sentence as their most decisive evidence that the output
was machine-produced. Not a construct, not a type — a doc comment:

> Errors returned by `inc_major_e`, `inc_minor_e`, and `inc_patch_e` wrap this value and name the
> segment that overflowed, so they can be detected with `errors.Is`.

Three methods the crate does not contain, and one function from a package that did not come along.

**The rule is the one the engine already had, one layer out.** A body that calls a declaration which
refused is refused, because the emitted crate would not contain the name. A doc comment that
describes one is the same defect and reads worse: it documents an API that is not there, in the voice
of somebody who checked. Two shapes, one reason — a word naming a declaration of this unit that is
not being emitted, and a qualified name whose package is not a unit of this model.

**Three false positives it took to bound it, each instructive.** A unit with a member called `con`
refused every declaration whose prose used the word — so the rule is bound to EXPORTED names, which
is the same bound the rename map's own construction already used, arrived at from the other side. A
declaration's own name looked dangling to it, because Go opens a doc with the name and the rewrite
that strips it does not always reach a second mention. And the emitted set holds TOP-LEVEL names
only, so every member looked absent — including the seven of `semver`'s own sentinels sitting in the
output, and `Run` on the hermetic corpus, which broke `port-go` outright. A member is emitted exactly
when its owner is, and `LocalScope` now records which declaration owns each one.

**Cost, and why it is temporary.** xxhash 29.4→26.5, ksuid 18.3→16.1, semver 29.3→27.6, uuid
unchanged. Every one of these refusals is a CASCADE of a method or type that refused for its own
reason — so they recover as those do. It is the self-containment fixpoint extended to prose, not a
standing tax.

**And the string literal, declined twice and now reversed.** "version increment would overflow
uint64" names a type the emitted crate does not have, three lines from its own constant that says
`u64`. The line I drew was that prose DESCRIBES the code and may be rewritten, while text the code
EMITS is the program's output and may not. That line was too clean: the message was not merely
foreign, it had become FALSE, and faithfulness to a sentence that is no longer true is not
faithfulness. The cost is stated rather than hidden — a program that deliberately prints the source
language's type name is changed by this — and it is accepted because the map is tiny, holds only
names that are not English words, and two independent reviewers ranked the untranslated one at the
top of their evidence.

All seven real packages still compile with zero rustc errors and zero clippy warnings.

## R1h — the gate that was only watching one corpus

A third blind review, and the shape of its evidence had changed. Gone from the list: `errors.Is` in
prose, the `_e` method suffix, `uint64` in a message, `i64` for a length, the `Err` prefix stutter,
nine `std::fmt::` paths, and eighty-seven lines with no break in them. What remains is almost
entirely the SOURCE's own design showing through a faithful port — six sentinels because the source
declares six, character-set constants because the source carries them, a comparator helper because
the source needs one, `is_x(x)` because that is what the source calls it. An engine that "fixed"
those would be rewriting the program rather than porting it.

**Then it named something that was true and was mine.** The six error types are constructed and never
compared: zero `downcast_ref` sites across all seven ported packages. The mechanism exists and the
sentinel corpus proves it — but only the sentinel corpus, and looking there showed the declaration
emitting `pub struct Gone` while the comparison asked about `ErrGone`. Two spellings of one name,
from the two sites, which is exactly the disagreement `sentinel_type_name` was written to prevent —
reintroduced by the one call site that cased the name itself instead of asking.

**The gate could not have caught it, so the gate was wrong.** The compile proof ran the strict
pipeline over ONE corpus. The other four were rendered by tests that checked what they SAID and never
checked that it was a program. `every_corpus_compiles.rs` now assembles each committed fixture into a
crate and type-checks it — as a crate, with one module per unit, because that is the layout the
output claims: `crate::shapes::Point` resolves only where the unit modules are siblings at a root, so
compiling the files separately would prove less than the output asserts.

**It found a second defect on its first run.** `let mut total = 0;` followed by
`total.wrapping_add(i)` does not compile: the target cannot infer a width from a method call that
exists on all of them. The source's untyped constant takes a DEFAULT type — its own `int`, which this
pack maps to a 64-bit one — and the target infers from use instead. So the annotation is kept for a
bare numeric literal and dropped for everything else, where the value already has a type and the two
languages agree. A third case had to be split out: a binding with no recorded type at all, which is
what a body-scoped constant is, and where there is nothing to annotate with.

**And a stale assertion, which is its own lesson.** The sentinel refusal test asserted
`downcast_ref::<ErrGone>()` — the pre-prefix-drop spelling. It passed while the declaration and the
comparison disagreed, because it was checking against the same wrong answer one of them gave. A test
that pins a spelling has to be updated when the spelling is decided elsewhere, or it stops being a
check and becomes a second opinion holding the old view.

Three defects, one gate. All seven real packages still compile with zero rustc errors and zero clippy
warnings; all five corpora now do too.

## R1i — the error model, answered with a measurement instead of a preference

Three independent blind reviewers led with the same proposal: the unit's sentinels should be one
`#[non_exhaustive] enum` rather than several types behind a boxed trait object. It was declined
before; it is now declined with numbers, written into the pack beside the decision, because an
engine that cannot say why it did not do the obvious thing gets asked again by every reader.

**An enum cannot type a parameter that accepts an ARBITRARY failure.** The source's error is an open
interface, and **16 parameters across the seven surveyed packages take one** — 10 in `errors`, 4 in
`multierror`, two packages that exist for no other purpose. A per-unit enum accepts only what that
unit declares, so those signatures become untypeable. Not a worse API: an impossible one.

**An enum built from a unit's sentinels does not cover its failures.** `semver` declares 7 sentinels
and has 78 failure sites. The enum reaches **16**. Forty-three are formatted messages with no variant
to be.

**And the variant that would cover them buys nothing.** A `Message(String)` arm is the box under
another name for four fifths of the cases, and the enum must be `#[non_exhaustive]` regardless — so
the exhaustive `match` that was the whole argument for it never arrives.

**What the reviewers were actually seeing is real, and is a different thing.** In a PARTIAL port the
sentinel types are constructed and never compared — zero `downcast_ref` sites — because every
function that compares them refused for its own reason. The comparison IS emitted; `corpus-sentinel`
proves it. It appears as those functions land.

The general shape here is worth keeping: an enum is a better Rust API for a program somebody is
writing fresh, and a DIFFERENT API from the one being ported. Choosing it would be redesigning the
package rather than porting it, and this engine ports.

## R1j — the finding three reviews led with, and the two decisions I had conflated

Three consecutive blind reviews opened with the same thing: seven near-identical `struct` + `Display`
+ `Error` blocks where a Rust author writes one enum. I declined it twice by measurement, and the
measurement was sound but it answered a question nobody had asked.

**The conflation.** `target_type` — what a fallible function RETURNS — has to stay the open boxed
error, because the source's error is an open interface and 16 parameters across the corpus accept an
arbitrary failure. That measurement holds. But it says nothing about how a unit's OWN declared
sentinels are SPELLED, and spelling them one type each was a separate choice that never had to follow
from it. Two decisions, one of which I had been defending with the other's evidence.

Grouped, the unit's sentinels become `pub enum Error` with a variant each, one `Display` whose arms
are built from the same list the variants are, and one `Error` impl. It preserves everything the
separate types carried — distinct identity, the message each holds, the comparison a caller makes —
and it still boxes at the boundary, so an arbitrary failure crosses exactly as before. Six blocks of
`semver` became one; the hermetic corpus's single sentinel became a one-variant enum, which is the
same rule and reads the same way.

**Named `Error`, not after the package.** The target addresses it through the module already:
`semver::Error` says everything `semver::SemverError` does. Falls back to one type per sentinel where
the unit already declares that name, because a collision is worse than the boilerplate.

**`#[non_exhaustive]`, and the cost is stated.** An exhaustive `match` was part of what the reviewers
wanted from an enum, and this withholds it. Taken anyway: the source's sentinel list grows without
ceremony there, and in the target an added variant would break every downstream match. A library that
cannot add a failure without a major version is a worse outcome than one whose callers write a
wildcard arm.

**Three things the change surfaced.**
- The enum was carrying a variant for a sentinel whose own declaration had REFUSED — a failure case
  the type declared and no return could ever construct. Variants are now filtered to what is
  emitted, and the enum is built on the first sentinel that survives rather than the first declared.
- A sentinel emitting nothing of its own was skipping the dangling-prose check entirely, because the
  grouping branch returned before the docs were read. The check now runs first, for its refusal
  rather than for its value.
- The identity test was `matches!(err.downcast_ref::<Error>(), Some(&Error::Gone))` and rendered as
  `downcast_ref::< Error > ()` — the target's formatter prints MACRO bodies as raw tokens. Switched
  to the equality, which says the same thing, needs a derive the type already has, and formats.

**Four files split along real seams** to hold the 100–300 bar: the grouped enum out of `items_static`
(one item from a whole list, where everything there builds one item from one declaration); the
sentinel lowering out of `lower.rs` (both spellings of one decision in one place); the item
VOCABULARY out of the item list; and a unit's own facts out of the resolver that asks questions of
them.

All seven real packages still compile with zero rustc errors and zero clippy warnings, and coverage
is unchanged — this is a change in how the same declarations are spelled, not in how many of them
the engine can reach.

## R1k — MERGE WITH CHANGES, and three findings from the review that said it

The fourth blind review since the enum landed returned **MERGE WITH CHANGES** — the first non-refusal
of the session. Its remaining evidence is almost all the source's own design, with one exception it
was right about and two it named that the engine could fix.

**A blank line between ITEMS, not only between regions.** The formatter emits none, because the tree
it renders has none — a syntax tree records what the items ARE, not how far apart a reader wants them.
So a type, its `Display` impl and its `Error` impl arrived as one unbroken block. Per item rather
than per region: a region is one declaration's whole output and one declaration emits several.

**A `match` used as an argument, bound first instead.** The hermetic formatter breaks
`f.write_str(match self { .. })` across ten lines with a trailing comma after the block, where the
formatter most authors run collapses it — and a reviewer read that, correctly, as output nobody had
formatted. `let message = match self { .. };` then `f.write_str(message)` says exactly the same thing
and survives both formatters. The engine's formatter is fixed by the determinism contract, so where
its output and the common one differ, the fix is a shape both agree on rather than a different
formatter.

**And the alias, which took two tries.** The reviewer's point was narrow and correct: `pub type
Result<T>` is a fixed shape wearing a type parameter — the failure slot cannot be anything else.
Making it `Result<T, E = Box<dyn std::error::Error + Send + Sync>>` fixes that and is 92 characters,
which the formatter breaks across four lines. Worse than what it replaced.

So the failure type gets a NAME: `pub type BoxError = Box<dyn std::error::Error + Send + Sync>;` and
`pub type Result<T, E = BoxError> = std::result::Result<T, E>;`. Both fit on a line, both say more
than the one did, and `BoxError` is what real Rust that does this calls it. Worth stating as a
general shape: when a decision is right and its spelling does not fit, the answer is usually a name.

**One defect it surfaced.** The assembly registered a region once per ITEM, which was invisible while
every unit-level region held exactly one. Two aliases put the prelude region in the order twice, and
the whole region rendered twice — every name in it defined twice. Registered once per region now.

All seven real packages compile with zero rustc errors and zero clippy warnings, all five corpora
compile as crates, and coverage is unchanged.

## R1l — one module importing `std::fmt` and spelling everything else out

Three reviewers in a row named the same inconsistency, the last one ranking it sixth of seven pieces
of evidence: the emitted module imports `std::fmt` and then writes `std::error::Error`,
`std::result::Result` and `std::cmp::Ordering` out at every use. Their words — the source language's
package-qualified reference model applied to the target's paths. Correct, and entirely the engine's.

**Producers emit the short form; the import follows what is NAMED.** No rewriting pass: every place
these paths come from is the engine's own — the pack's type map, the pack's failure type, the
sentinel lowering's `impl` — so they emit `Ordering` and `StdError` directly, and a unit gains the
import only where its emitted types actually name one. The failure mode is safe by construction: a
missed import does not compile, which the compile proof catches, where a spurious one is a denied
warning.

**`StdError`, and why the table is keyed by the NAME.** A unit that declares failures emits its own
type called `Error`, so importing the trait under the path's own last segment would collide with it.
`use std::error::Error as StdError` is what real Rust writes in exactly this situation — and because
the local name differs from the path, it cannot be derived from the path. The table is keyed by what
the emitted code SAYS.

**Asked of the types, not of the text.** A structural walk over the item tree collects every type
spelling, and the match is on whole identifiers — `MyOrdering` does not name `Ordering`. A text scan
would have matched it and emitted an import nothing uses, which is a build failure.

**Imports are a BLOCK.** The item-separation rule put a blank line between every pair, including
consecutive `use` lines, which nobody writes. The rule now knows that much about what it separates.

**Three defects the change surfaced, all of them ordering or parsing.**
- `use std::error::Error as StdError` refused to render, by name, because a rename is not part of a
  path and the lowering parsed one. Right failure, wrong parser: it parses the whole `use` item now.
- The assembly built the prelude and the imports from a LIST of both, and a list evaluates its
  elements before the first is placed — so the import scan looked at a unit that did not have its
  aliases yet. Every unit with a prelude and no sentinel came out naming `StdError` with no import
  for it. Built one after the other now, and the order is the reason rather than a preference.
- A region was registered once per ITEM, invisible while every unit-level region held exactly one.

All seven real packages compile with zero rustc errors and zero clippy warnings; all five corpora
compile as crates.

## R1m — where a package variable is written, which is not the same question as whether

The largest single refusal cause across the corpus is a package-level `var` something writes: 22 of
them, and the reason the engine gave was a paragraph about concurrency policy — what synchronizes a
mutable global, and why none of `static`, `static mut`, `Mutex`, `RwLock` or an atomic is a default
the engine may pick. That reason is right for some of them and wrong for the rest, and the engine
could not tell which because it only knew THAT a write existed.

**Walking per declaration instead of per file.** `go/types` omits `init` from package scope, so the
package initialiser can only be recognised in the syntax: named `init`, no receiver, no parameters,
no results — all four, because a method called `init` is an ordinary method. The write analysis now
records where each write was, and a variable every write to which is in the initialiser carries its
own flag.

**Measured: 12 of the 22 are that shape.** Those have no synchronization question at all — computed
once before anything runs, never changed after. What they lack is not a decision but a FACT: the
initialising expression, which lives in the `init` body the front end still does not index. So the
engine can see THAT such a variable is computed and not WITH WHAT, and the refusal now says exactly
that instead of describing a concurrency trade nobody is making.

**And it names the cost of the form it wants, before anyone reaches for it.** A lazily-initialised
global computes at first use where the source computed before `main`. For a compiled pattern that is
invisible; for an initialiser with side effects it is a different program, and nothing here proves
which. Also measured and recorded: 6 of those 12 are compiled regular expressions whose type does
not come along either, so finishing this rule would unblock about five declarations across seven
packages — worth writing down, and not worth doing before the causes that block more.

**Three gates fired on the regeneration, each correctly.** The closed FLAG vocabulary refused
`init_written` until the Rust side declared it, which is what a closed vocabulary is for. The pack
load's deferral set is asserted exactly and caught the new form. And the upstream-drift pair went
`Unchanged` because I regenerated it under a different module id — the unit stopped being one the
plan selects, so nothing was emitted at all. The pair is the invariant that proves a moved upstream
is Explained by exactly the snapshot axis, and it did its job by failing.

## R1n — the doc fingerprint, and the error model answered from the goal instead of the corpus

Two reviews in a row now return **MERGE WITH CHANGES**. Both put the same two things at the top, and
both are worth a definite answer.

**The doc fingerprint was the engine's, and it is fixed.** Three independent reviewers counted six
docs in one file opening `Returned when …` and named the uniformity as proof the prose had been
mechanically de-prefixed rather than written. They were right about the mechanism and right that it
was ours. The source's convention makes a doc open with the identifier — `ErrEmpty is returned when
the input has no content` — and stripping the name and its copula leaves the narration behind. The
target documents a type by saying what it MEANS, not by narrating who returns it, so the narration
goes with the name it belonged to: `The input has no content.`

The bare `returned` is in the list and is safe BECAUSE of where the rule runs. The name and copula
are already gone, so what remains is a predicate of the declaration, and `returned …` in that
position is always narration rather than a subject. It is also what rescues a sentence upstream got
wrong: `ErrInvalidSemVer is returned a version is found to be invalid` is missing a word in the
SOURCE — verified against it — and dropping the narration leaves the grammatical remainder instead
of carrying the break through. Three reviewers cited that broken sentence as their single most
decisive tell; it now reads as English and the engine invented nothing to make it so.

**The error model, answered from the goal rather than from the corpus.** The proposal is always the
same: default `Result<T, E>` to the unit's own `Error` instead of the boxed one. My earlier answer —
16 parameters across the corpus accept an arbitrary failure — defends the PARAMETER type and does
not actually defend the RESULT default, and I had been using one to argue the other.

The real answer has two parts. First, one default cannot serve every function: 43 of `semver`'s 78
failure sites build a formatted message, which no variant of a sentinel enum can be. Narrowing per
function is possible and is a fixpoint over the call graph — a function is narrow only if every
failure it returns or propagates is narrow — and the engine already has that machinery.

Second, and decisive: **the goal is to keep repos ported as upstream MOVES.** A per-function narrowed
error type makes the emitted public API change shape whenever upstream adds a `fmt.Errorf` to a
function that previously only returned sentinels — a change the source treats as non-breaking and the
target would not. The boxed default is stable under exactly the upstream drift this engine exists to
absorb. That is a reason from the mandate rather than from one corpus, and it is the one to keep.

Recorded as considered and declined: dropping `Copy` from the failure enum for forward compatibility.
The reviewer's argument is that `#[non_exhaustive]` promises additive change and `Copy` blocks adding
context later. True in general, and not here — the enum's variants come from the source's sentinels,
which carry no data by construction, so the field that would break `Copy` has nowhere to come from.

## R1o — the engine was emitting methods that compile and panic

Reviews were converging on `semver`, so I read a different package. `uuid` opened with this:

```rust
impl Domain {
    pub fn string(&self) -> String { todo!() }
}
```

A method that compiles, passes every gate that reads the output as Rust, and aborts at the caller
where the source computed something. That is the one failure this engine exists to prevent, dressed
as success — and worse than a refusal, because a refusal says so.

**Why nothing caught it.** The pack has two rungs for a struct — one that stubs bodies and one that
translates them — and the corpus that proves method bodies declares STRUCTS. The rung for DEFINED
TYPES had no such pair: `build_newtype` hardcoded the stub, so every method of every defined type in
every package was a `todo!()`. The one gate that watches for stubs reads the golden, and the golden's
stub-rung units declare structs with no methods at all.

**The fix is not another rung, it is that no rung may emit one.** A body the engine cannot write is a
refusal, at both sites — the method and the free function — and `RustExpr::Todo` is gone from the IR
so nothing can produce one again. Four fixture tests were asserting the stub behaviour; they now give
their fixtures bodies, which is what they were about.

**Then the translation exposed two more, both real.**

*Named results.* `func (t Time) UnixTime() (sec, nsec int64)` names its results, and those names are
BINDINGS the body assigns to before returning. The target has no such thing. The engine translated
the assignments and never bound the names — a body reading variables that do not exist, which is the
dangling-name defect self-containment refuses everywhere else, arrived at from the signature instead
of from a call. Eleven of them in one package, invisible until the rung stopped stubbing.

*An opaque newtype.* The source's `type Version byte` is transparent — it compares against an untyped
constant, formats as a number, does arithmetic. The target's newtype has none of its underlying
type's operators, and that opacity is exactly why the newtype is the faithful shape. So `if v > 15`
became `if self > 15` on a `&Version`, and `%d` of one became `format!("{}", self)` on a type with no
`Display`. Both refuse by name; the rule they want is `self.0`, and unwrapping without proving the
underlying type is numeric would reach inside a newtype declared to stop exactly that.

**And a termination bug I wrote and the corpus found in minutes.** The newtype check first asked
whether the type was EMITTED. The emittability fixpoint only ever shrinks, and that is what makes it
terminate; a refusal that consults it stops firing as the set loses members, un-refusing a
declaration and growing the set again. Whether a type is a newtype is a fact about the SOURCE, so it
asks the source.

Coverage: uuid 26.8% → 13.4%. Every point removed was a declaration counted as translated while
emitting a method that panics or a body naming variables that do not exist.

## R1p — a write the analysis could not see, twice

Reading `uuid` again after the stub fix turned up `const POOL: [u8; 256] = [0; 256];`. A randomness
pool, emitted as a constant — which in the target is materialised fresh at every use and cannot be
written at all. The write analysis said nobody writes it. Two reasons, both real:

**An assignment target is rarely a bare name.** `pool[i] = x` writes `pool`; `cfg.field = x` writes
`cfg`; `*p = x` writes what `p` points at. The analysis matched only `*ast.Ident` and saw none of
them, so a package array something fills element by element read as never written — and a
never-written variable becomes a constant. Assignment targets are now walked down to the variable
they reach.

**And a slice is the same licence as an address.** The analysis already counted `&x` as a write,
because taking the address hands out a licence to write through it and the write may be anywhere.
`pool[:]` hands out exactly that, spelled differently: `io.ReadFull(rander, pool[:])` fills the array
through a mutable view without ever assigning to the name. That is how this one got through.

The correctness argument is the one already written beside the address case, and it is worth
repeating because it decided this: being conservative here costs a synchronization policy, and being
wrong costs a program that silently stops sharing state.

**No fixture moved.** Every committed corpus was regenerated and none changed — the hermetic corpora
have neither an indexed package-var assignment nor a slice of a package array. That is the gap that
let this live: the corpus only contains what the engine already handles, so a construct it handles
WRONGLY is invisible there until a real package shows it. Ratcheting against real third-party source
is the method for exactly this reason, and it earned its keep twice in one phase.

`POOL` and `NODE_ID` now refuse by name as written package variables. `ZERO_ID` and `XVALUES` stay
constants, which is correct — nothing writes them. All seven real packages still compile with zero
rustc errors and zero clippy warnings.

## R1q — named results, and the type a method hangs off

Two of the constructs the last phase refused by name are now rules. Both were named there with the
rule they wanted; this is that rule.

**Named results are BINDINGS, so they are bound.** The source lets a signature name its results, and
those names are variables: zero-initialised at entry, assigned during the body, returned by a bare
`return`. The target has no such thing. Binding them at the top of the body — at the zero value the
source gives them — makes the rest need no special case at all: an assignment to one is an ordinary
assignment, and `return sec, nsec` is an ordinary return. Eleven declarations across the corpus.

`mut` only where the body actually assigns, because a binding declared mutable and never written is
a warning the compile proof denies — and the whole reason to bind these is that the body writes them.

**A method's body did not know what `self` IS.** The receiver is not a child of the method
declaration and the front end records no type for it, so the newtype-opacity check saw every method
on a defined type as untyped and let `self - G1582NS100` through. The signature is the only place
that knows the owning type, so it tells the body. Everything that asks about the receiver now gets
the same answer the signature gave.

**Coverage: uuid 13.4% → 11.3%.** Down again, and again correctly — binding the results made those
eleven bodies translatable enough to reach the operations on the receiver, which then refused for a
reason the engine could finally see. This is the pattern the whole session has followed: a rule that
unblocks a construct reveals the next one, and the number goes down before it goes up.

**Where the artifact stands.** Eight blind reviews of the ported `semver` since the enum landed:
three MERGE WITH CHANGES, then a DO NOT MERGE that blocks on the boxed `Result` default and on
missing error variants for limits the SOURCE does not declare either. The verdict oscillates on the
same declined decision, which is the honest signal that the artifact has reached a plateau: what
remains on the evidence lists is the source's own design showing through a faithful port — charset
constants, a comparator helper that Go needed and Rust does not, payload-free sentinels — plus one
decision this engine has measured and declined twice, for a reason taken from the mandate rather than
from any corpus.

## R1r — the ranked top cause was pack data all along

Twenty refusals said the same thing: `X is in package Y, which this snapshot does not contain — the
pack has to map the call`. Ranked first among causes that are not cascades, and pure pack work with
no engine change. I had been avoiding it because each mapping is a semantic decision — which is
exactly what a pack is for.

Nineteen distinct callees. Three map, nine cannot, and saying WHICH is most of the value.

**Mapped, where the two agree on every input.** `math/bits.RotateLeft64` → `rotate_left`, because
both rotate and both reduce the distance modulo 64. `os.Getenv` → `env::var(..).unwrap_or_default()`,
because the source cannot tell unset from empty and the target's default for unset is empty.
`strings.LastIndex` → `rfind(..).map_or(-1, ..)`, because both count BYTES — the thing that usually
differs between two languages' index functions — so only the absent case needed spelling.

**And a new table for the ones that CANNOT be mapped, each with the reason.** This is the distinction
that was missing: a call the pack has not reached refuses saying a mapping is owed, and a call the
pack has LOOKED at refuses saying a mapping would be wrong. Every entry is a case where the target
has something that resembles the source's call and differs on input nobody would think to test:

- `strconv.ParseUint` — the target accepts a leading `+` and the source rejects it, and the source
  takes a base and a bit size of which only one pair is the target's `parse` at all. A version parser
  is exactly where that matters: invisible on valid input, and it changes which strings are accepted.
- `strings.EqualFold` — Unicode simple case FOLDING against ASCII folding or case MAPPING. They
  differ on the Kelvin sign, final sigma, and the Turkish dotless i.
- `errors.Is` / `errors.As` — they walk a cause chain this port does not build, since the wrapping
  verb refuses for that reason. Right today; silently wrong the day upstream wraps something.
- `encoding/hex.*`, `crypto/md5.New`, `crypto/sha1.New` — absent from the target's standard library.
  Which crate supplies one is a dependency decision about the ported crate, not the engine's.
- `bytes.Compare` — the values correspond and the TYPES do not, and which to emit is a decision about
  the call sites.

**One mapping paid for the whole phase: xxhash 26.5% → 55.9%.** The rotation is the inner loop of the
hash, and every function that used it had been refusing.

**Two defects it exposed, both mine.**

The mapping template `{0}.rotate_left({1} as u32)` rendered as `x.rotate_left(1` — the cast detector
takes the text after the LAST ` as ` for a type, and here that was `u32)`. It only ever meant to
recognise a cast of the WHOLE expression, which is the case that needs bracketing, so it now requires
the target to be an identifier. Worth noting how it surfaced: as a LATE refusal at render, after the
self-containment fixpoint had already run — so `rol31` vanished from the output while its callers
stayed, leaving exactly the dangling reference that fixpoint exists to prevent. A render refusal is
invisible to the transform, and nothing today reconciles them.

Then `1 as u32` on a literal is a cast the target's own lints call unnecessary, and they are right.
The source counts a rotation distance in its own `int` and the target's method takes a `u32`: for a
literal the target infers it, and for anything else a conversion between two widths is needed that
the source never asked for. So the mapping holds for a literal distance and refuses by name for the
rest, through a new `int_literal_last` argument shape — the same mechanism `panic` already used to
say "faithful for this shape and silently wrong for the others".

## R1s — a comment that named the source language, and the review that verified the arithmetic

Reviews had only ever seen `semver`, the thinnest package. `xxhash` is now the most complete at 52.9%
and exercises entirely different constructs, so it went for review instead. The verdict was **MERGE
WITH CHANGES**, and the reviewer did something none of the others had: they checked the numbers.

> All five primes are numerically exact. `round` and `merge_round` match the algorithm — operand
> order, rotate amount, and the `*PRIME1 + PRIME4` tail. Critically, **every** arithmetic op uses
> `wrapping_add`/`wrapping_mul`. Go's `+`/`*` wrap silently, so a literal transliteration would panic
> in debug builds; someone caught that.

That is the engine's central claim, checked by someone who did not know it was a claim: the output
does not mean something different.

**And their single most decisive provenance finding was ours.** A doc comment, in rustdoc, on a crate
that denies `unsafe`:

> The consts are used when possible in Go code to avoid MOVs but we need a contiguous array for the
> assembly code.

Prose naming the source language, describing its compiler's register allocation and an assembly
backend that does not exist here. A doc that documents a program which was not ported. It refuses
now, for the same reason prose naming an absent API refuses: the engine can see the sentence is false
about what it documents and cannot write a true one.

**Why the language's own name is safe to look for, which is the whole subtlety.** `Go` is an English
word and `multierror` declares a METHOD called `Go` whose doc opens `Go calls the given function in a
new goroutine` — a naive match refuses it on its own name. The check runs on the REWRITTEN prose, and
the source's convention opens a doc with the identifier, so that word is already gone by the time
this looks. Verified: the method survives, and `goroutine` catches what the sentence is really about.
The English verb is lower-case, which the list is not.

**Checked and NOT ours:** the reviewer wanted the primes in hex, since the spec states them that way
and they had to run a calculator. Upstream writes them in decimal. Carried faithfully.

**Still owed, recorded from this review:** `MARSHALED_SIZE: i64` is a byte count that wants `usize` —
the length-constant proof needs a comparison against a length, and the function that would supply one
refuses. `MAGIC: &str` holds binary framing that wants `&[u8; 4]`; the source's `string` is bytes and
the target's is UTF-8, which is a type-map decision the pack has not made. And the eight `rolN`
wrappers exist because the source needed them — faithful, and the reviewer is right that a Rust
author would not write them.

## R1t — the derivation the source folded away

Both `xxhash` reviews put a bare folded literal at or near the top of their evidence, and the second
one said exactly what was wrong with it:

> `76` is precisely `len("xxh\x06") + 8*5 + 32` — the Go source states it as that expression; here the
> expression was evaluated away and only the literal survived. That is a translator constant-folding,
> not a human choosing a number.

They were right, and it is not the engine's folding: `go/types` evaluates a constant before the
extractor ever sees it, so `76` is all that arrived. The initialiser EXPRESSION is in the syntax
though, and the extractor already indexed one for variables. It indexes constants now too, and the
declaration carries both — the value, which is always correct, and the derivation, which is what the
author wrote.

**Why preferring the derivation is safe here and would not be in a body.** Both spellings are the
SAME constant, proven so by the source's own evaluator. A derivation the engine cannot translate is
therefore not a degraded answer, it is the same answer written differently — so the fallback costs
nothing but the author's spelling. That is what makes this reasonable rather than reckless.

Emitted as an EXPRESSION rather than as text, through the item shape that already held a constant's
value as a tree. `const MARSHALED_SIZE: usize = MAGIC.len() + 8 * 5 + 32;`

**Numeric only, and the string case is why.** The target has no `+` on strings at all, so
`"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-" + NUM` parses, type-checks nowhere, and is a
crate that does not build. Both reviewers had also named that concatenation as a fingerprint — and
here the folded value is the only thing the target can say, so it keeps it.

**And the length conversion had to come off the PART that has it.** `unsigned_bound` stripped the
mapped length call's cast only when the whole bound was one call; a bound that is COMPUTED —
`len(magic) + 8*5 + 32` — kept `MAGIC.len() as i64` inside a `usize` constant. It walks the source's
arithmetic now, asking the same question of each side.

**A gap it exposed: a constant reference was never self-containment-checked.** The refusal that
guards package-scope reads grew out of the variable deferral and only ever looked at variables. A
constant's derivation reads other constants, which is what made it visible: `byteLength =
timestampLengthInBytes + payloadLengthInBytes` named two that had refused. Constants are checked now
— except the ones the pack MAPS, which is what a predeclared constant is: `true` is classified as a
constant reference like any other and is not this unit's to emit, and asking the unit for it refused
every declaration that mentions a boolean.

## R1u — types that cannot be mapped, and what the doc refusals actually cost

Re-ranked the causes. The top one is now the CASCADE at 25 — declarations refusing because something
they name refused — and under it, unmapped types at 17. Most of those 17 are the cascade again; only
four are genuinely foreign. So the work was two things: name those four, and find out what is really
at the root of the cascade.

**Four types the pack now refuses BY NAME**, mirroring the unmappable calls table and for the same
distinction — a type the pack has not reached says a mapping is owed, one it has looked at says a
mapping would be wrong:

- `sync.Mutex` — the two guard different things. The source's mutex guards a critical SECTION and
  sits beside the fields it protects, with nothing in the type system tying them together; the
  target's OWNS what it protects, and holding the lock is the only way to reach the data. Mapping the
  type alone emits a lock guarding nothing, which compiles and protects nothing. What it wants is the
  struct reshaped so the mutex wraps the fields it stands next to — and which fields those are is
  something the source records only by adjacency.
- `time.Time` — the source's instant is a wall clock reading with a monotonic reading attached and a
  location beside it. The target splits those three ways and no single mapping keeps all of them.
- `fmt.State`, `math/rand.Source64` — a formatter interface with no counterpart, and randomness the
  target's standard library does not have at all.

**And a measurement worth having, on the engine's own rules.** Chasing `xid.ID` — the central type of
its package — turned up that it refuses because a METHOD's documentation says *"behaves just like
`bytes.Compare`"*. An explanatory aside, not a claim about this crate's API, and the prose refusal
cannot tell those apart.

So both halves of the doc refusal were disabled and measured. Together they cost **xxhash 3.0,
semver 1.7, ksuid 2.1 points** — real, bounded, and much less than feared. They are kept: prose
naming what the crate does not have is false about it, two independent reviewers ranked exactly this
among their most decisive evidence, and the alternative is documentation written in the voice of
somebody who checked.

What the measurement did change is a warning worth recording: **a doc refusal MASKS the code reason
underneath it.** `xid.ID` reports the doc as its cause and still refuses with both halves off, so
something else is wrong there too and the survey will not say what until the doc reason is gone.
Anyone chasing a cascade root should disable these two first and re-read the reasons.

## R1v — an accumulator is one expression, not four statements

Both `xxhash` reviews named the same shape as a transliteration of the source's statement style:

```rust
fn round(mut acc: u64, input: u64) -> u64 {
    acc = acc.wrapping_add(input.wrapping_mul(PRIME2));
    acc = rol31(acc);
    acc = acc.wrapping_mul(PRIME1);
    acc
}
```

Four statements holding one computation, and a `mut` in the signature that exists only to allow the
rewriting. The target spells that computation as itself:

```rust
fn round(acc: u64, input: u64) -> u64 {
    rol31(acc.wrapping_add(input.wrapping_mul(PRIME2))).wrapping_mul(PRIME1)
}
```

**Every condition on the fold is load-bearing**, and each is a way the substitution could be wrong:

- every statement but the last assigns to ONE name, and that name is a parameter — so there is a
  single chain and nothing else in the body to reorder around;
- each assigned value mentions the name EXACTLY once, counting the implicit read a
  read-modify-write performs. `acc += x` means `acc = acc + x` and the implicit read IS the chain's
  link; `acc += acc` would read it twice and is not a chain;
- the last statement returns that name and nothing else.

`merge_round` next to it does not fold, and that is the rule working: it assigns to two names, so
there is no single chain, and it keeps its statements.

**Recognised on the SOURCE and spent in two places, because both have to agree.** The body folds and
the signature drops the `mut`, from one fact — a disagreement is either a mutable binding nothing
writes, which the target warns about, or a write to an immutable one, which does not compile.

**Substitution refuses shapes it was not promised.** It rebuilds only the expression forms a chain
can be made of, and returns nothing for anything else rather than moving a subexpression somewhere
its evaluation order is not the source's. The recogniser already proved the shape; the substituter
checks anyway.

Coverage is unchanged across all seven packages, and that is the point: this changes how the same
translated declarations are SPELLED, and the compile proof holds at zero rustc errors and zero
clippy warnings.

## R1w — a string that is not text

The last `xxhash` review put `const MAGIC: &str = "xxh\x06";` second in its evidence, and the trap it
described is concrete rather than stylistic:

> The trailing byte is a format-version tag, so someone will eventually bump it. Rust `str` literals
> only permit `\xNN` up to `\x7F`. Bump this to `\x80` and the literal stops compiling; the natural
> "fix" is `"xxh\u{80}"`, which is TWO UTF-8 bytes, silently turning `MAGIC.len()` from 4 into 5 and
> shifting every offset in the marshaled layout.

That is this engine's own failure mode described by somebody who did not know what they were looking
at: a change that compiles and means something different. The source's string is a byte string and
the target's is guaranteed UTF-8, so the ordinary mapping is right only for the strings that are
text.

**Recognised, not guessed.** A byte the source had to write as an ESCAPE because it cannot be typed
is exactly the evidence that the value is data rather than prose. The common whitespace escapes —
newline, tab, carriage return — are text and stay text, so a message with a line break in it is
untouched. The byte COUNT is taken from the decoded bytes rather than the spelling, because an escape
is one byte written as four characters, and a wrong count would be the very shift the reviewer warned
about.

`const MAGIC: &[u8; 4] = b"xxh\x06";` — and `MARSHALED_SIZE` still derives from it, because
`<[u8]>::len` is const just as `str::len` is.

**The decoder refuses what it cannot decode.** Only escapes both languages spell identically; a value
with anything else yields nothing and the constant stays a string. A length the engine guessed would
be a wire format the engine guessed.

Coverage unchanged across all seven packages. `items.rs` crossed the 300-line bar and split along the
seam that was already there: which ITEM a declaration becomes, versus what goes on the right of the
`=`.

## R1x — the chain with a longer neck, and a bug I wrote and caught by reading

All three `xxhash` reviews named `merge_round` as statement-for-statement source style, and the last
one wrote out what it should be. It now is:

```rust
fn merge_round(acc: u64, val: u64) -> u64 {
    (acc ^ round(0, val)).wrapping_mul(PRIME1).wrapping_add(PRIME4)
}
```

**The invariant is a property of each ASSIGNMENT, not of each name.** The first version required
every statement to assign the same name, which folded `round` and not this. What actually makes a
sequence foldable is that the value a statement produces is read exactly ONCE before that name is
assigned again — read twice and the fold evaluates it twice, read never and the fold drops what it
did. `val = f(val); acc ^= val; acc = g(acc); return acc` satisfies that on two names, which is the
same chain with a longer neck rather than a different shape.

**A read-modify-write is that same link spelled shorter.** `acc ^= v` means `acc = acc ^ v`, and the
implicit read is what the chain hands forward. Rebuilding it explicitly is how the fold sees the link
at all — without it the body kept its statements while the signature had already dropped the `mut`,
which is precisely the disagreement deciding this from one fact is supposed to prevent.

**And a correctness bug I wrote, found by reading the output.** Substituting each held name over the
RUNNING result lets a name inside something already substituted be substituted again: `acc = g(acc)`
after `val = f(val)` emitted `round(0, round(0, val))` — the source's call applied twice. A program
that compiles and computes something else, in the engine that exists to prevent exactly that. The
substitution now takes only the names the ORIGINAL value mentions, each once.

Worth recording as method rather than as trivia: the compile proof would not have caught it. Both
spellings type-check. It was caught by reading eight lines of emitted output, which is the one check
that sees meaning rather than form.

**Every name the chain consumes drops its `mut`,** not only the one returned — a `mut` left on a name
the fold substituted away is a mutability nothing uses, which the target warns about and the proof
denies. Verified by exit code rather than by grepping for the word "error".

## R1y — the wrapper that only shortens the source's spelling

Three reviews called the eight `rolN` wrappers the single largest thing to change, and the argument
is one this engine already accepted once. `func rol31(x uint64) uint64 { return
bits.RotateLeft64(x, 31) }` is not an abstraction — it is a shorthand, written because the source
spells the rotation as a free function and eight call sites reading `bits.RotateLeft64(acc, 31)`
would be unreadable. The target spells the same rotation as a method on the value, so the shorthand
has nothing left to shorten. Exactly the argument for dropping the source's `Err` prefix from a
sentinel: both are conventions answering a problem the target does not have.

**Four conditions, and the reasoning only holds where all four do.** Unexported, so the wrapper is
not API and removing it is invisible outside. Body is one call and nothing else, so there is no logic
to lose. That call is FOREIGN and the pack MAPS it, which is what makes it a spelling the target
already has — a wrapper around a local function is somebody's abstraction and stays. Every parameter
read exactly once, so substituting the arguments neither drops one nor evaluates one twice.

**Two correctness bugs, both mine, both caught by reading the output rather than by any gate.**

The fold substituted each held name over the RUNNING result, so a name inside something already
substituted was substituted again: `round(0, round(0, val))` — the source's call applied twice. Fixed
by taking only the names the ORIGINAL value mentions.

Then, worse: a chain link whose translated value is OPAQUE mentions nothing, so the substitution loop
did nothing and the statement's value silently overwrote the chain. `acc = rol31(acc)` after
`acc = acc.wrapping_add(..)` emitted `acc.rotate_left(31).wrapping_mul(PRIME1)` and lost the addition
entirely. A link must READ the name it chains from — the recogniser proved it does in the source — so
a translated value that does not is opaque and the fold aborts.

Both compile. Both type-check. Neither would have been caught by anything except reading eight lines
of emitted output, which is the only check this engine has that sees meaning rather than form.

**And the coupling that has now bitten three times is gone.** The recogniser reads the SOURCE and the
fold reads the TRANSLATION, and they disagree whenever a value arrives as opaque target text — which
left a body keeping its statements while the signature had already dropped the `mut`. The signature
is now built from what the fold DID rather than from what the recogniser predicted, which meant
building the body first in both places that build one.

`MAGIC` is `[u8; 4]` by value now rather than a reference to one: a constant is materialised at every
use anyway, so the reference bought an indirection and nothing else.

## R1z — a mapped call is a tree, not text

The wrapper inlining landed and `round` still would not fold, for a reason worth naming: a call the
pack answers for arrived as target TEXT. `structured()` produced `RustExpr::Literal("acc.rotate_left(31)")`
— a string wearing an expression's type — and text is opaque to everything downstream. The
accumulator fold could not substitute into it, so it correctly aborted and the body kept its
statements.

Most mapping forms are one shape: `{0}.method({1})`, a receiver and a name and arguments. That is a
tree, and building it costs nothing:

```rust
fn round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(PRIME2)).rotate_left(31).wrapping_mul(PRIME1)
}
```

**Anything else stays the text substitution it always was.** A form with a cast, a turbofish, or a
construction is not this shape, and pretending otherwise would build a tree that renders differently
from the form the pack wrote. Every argument must be a bare placeholder in order, and every argument
the call has must be consumed — a form that reorders, repeats, or drops one is doing something this
shape cannot express.

This is the same lesson as the formatting call two phases back: **text is the wrong currency between
rules.** A rule that emits text ends every rule downstream of it, silently, by producing something
nothing else can read. Both times the symptom was a later rule quietly declining to fire, and both
times the fix was a tree.

**And the wrappers themselves are gone.** Inlining every call left eight declarations with no
callers; they emit nothing now. Dropped only where nothing takes the function as a VALUE — the
source can write `f := rol31`, and that use has nowhere to go once the declaration does. The callee
child of a call is not such a use, which is exactly the case that inlines.

The emitted `xxhash` is now:

```rust
fn round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(PRIME2)).rotate_left(31).wrapping_mul(PRIME1)
}

fn merge_round(acc: u64, val: u64) -> u64 {
    (acc ^ round(0, val)).wrapping_mul(PRIME1).wrapping_add(PRIME4)
}
```

## R2a — the attribute whose ABSENCE changes the program

Three reviews asked for `#[inline]` on the hash primitives, and the third named the reason precisely:
the absence is what you expect from somebody porting rather than somebody writing the target's hash
crate. That is a style observation with a semantic argument underneath it, and the argument is the
one that made this worth doing.

**This is the rare case where NOT emitting something changes the ported program.** The source's
compiler inlines a small function by a cost heuristic with no annotation; the target's does not,
across codegen units, for a non-generic private one. So the source's helper is inlined and the port's
is a call — a performance difference the TRANSLATION introduced rather than one the author chose.
Emitting the attribute restores a decision the source already made.

**One expression is the bound, and the narrowness is the argument.** That shape is what the source's
own heuristic would certainly have inlined, so the attribute recovers a fact rather than inventing a
judgement. `inline` and not `inline(always)`: the stronger form overrides the target's optimiser,
which is a decision about a particular program that no translation can make. A PUBLIC function is
left alone — whether to promise inlining across a crate boundary is the ported library's contract,
and that belongs to whoever ports it.

The attribute is parsed rather than pasted, through a carrier item, so one the pack cannot spell
refuses at the IR rather than in the emitted file.

`xxhash`'s two primitives now read:

```rust
#[inline]
fn round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(PRIME2)).rotate_left(31).wrapping_mul(PRIME1)
}
```

**Declined, and worth writing down because it looks like a defect.** The same review called
`MARSHALED_SIZE = MAGIC.len() + 8 * 5 + 32` its most decisive tell, on the grounds that `MAGIC` is
`[u8; 4]` and the length is already in the type. True — and `.len()` on a fixed array is const,
correct, and stays correct if the magic ever changes length, which folding it to `4` would not. The
author wrote the derivation; the engine keeps it. What the reviewer is really seeing is that the
CONSTANT exists at all, which is the source's wire format, not the engine's spelling of it.

## R2b — the sentence is the unit, not the declaration

Four reviews in, the findings had converged on things that are the SOURCE's design — a wire-format
constant, an error type that exists because a Go interface demands one. Picking at those was picking
cosmetics, so I went looking for why the module looks PARTIAL instead, which is what a reviewer
actually reacts to when a 28-line file has no public API.

`xxhash`'s central type refused. The reason:

> its documentation names `hash.Hash64.`

The source's doc is three sentences. The first says `Digest implements hash.Hash64.` — false in the
port, because that interface did not come along. The other two say a zero-valued `Digest` is not
ready to receive writes and to call `Reset` first, which are true in the port and are the useful
part. The engine was throwing away the type, and with it everything that names it, over one sentence.

**So the sentence is the unit.** A sentence is what carries a claim; dropping the block loses
documentation that is still true, and keeping it emits a claim the crate does not honour. All three
prose rules — the foreign reference, the unemitted sibling, the source language itself — drop the
sentence and keep the declaration.

**This is prose surgery, and the engine already performs it.** The opening rewrite strips the
source's leading identifier; the narration rewrite strips `is returned when`; the type-name rewrite
replaces `uint64` with `u64`. This is the same category with a stronger warrant: those change how a
TRUE sentence reads, and this removes a FALSE one.

**And it corrects a measurement I had already made and trusted.** Two phases ago I disabled both
halves, measured the cost at "xxhash 3.0, semver 1.7, ksuid 2.1 points", and kept them on that basis.
The number was right and the conclusion was wrong: a coverage delta cannot show that the points lost
were a package's CENTRAL TYPE, and that everything downstream of it was already gone for other
reasons and so never appeared in the delta. Cost measured as a scalar hid cost that was structural.

xxhash 52.9 → 58.8, semver 22.4 → 24.1, ksuid 15.1 → 17.2. And `Digest` now refuses for an honest
reason — an `unproven_owned` pointer disposition, which is the "analysis follows calls" gap the plan
already names — rather than for a sentence.

**One bug fixed on the way.** The word split treated `.` as an identifier character, so a sentence's
final period was absorbed: `hash.Hash64.` became the member name, which is nobody's identifier. It
matched anyway, which is why it went unnoticed.

## R2c — chasing one cascade root to ground

`xxhash` is the most complete package and its central type `Digest` refused, taking everything that
names it with it. Four steps, each one the honest next reason rather than a guess:

**A doc sentence** — fixed by R2b, the sentence rather than the declaration.

**An unproven pointer disposition.** `MarshalBinary`'s receiver was `effect_unknown` because the
method calls `appendUint64(b, d.v1)`, and `appendUint64` passes its own value parameter to a foreign
call the analysis cannot look inside. What it stopped at could not have mattered: `d.v1` is a
`uint64`, the call receives a COPY, and nothing it does can reach `d`. The TYPE decides and not the
expression — `d.mem[:d.n]` roots at `d` too and is a slice, which aliases the receiver and is still
asked about. `carriesEffect` answers for the whole type, conservatively on anything unrecognised,
because being wrong that way costs a refusal and being wrong the other way costs a borrow chosen on
a fact that was never true.

**A type standing where an expression would.** `make([]byte, 0, marshaledSize)` names what to
allocate, and walking that name as an expression recorded `[]byte` as an unsupported node — which
refused every declaration that allocates anything.

**And the allocation itself.** Its own pack table rather than a row in the function map, because the
first argument is a type and the meaning changes with the arity — neither of which a form keyed by
callee identity can express. The distinction that matters is between the two exact shapes:
`make([]T, 0, n)` has NO elements and room for n, and `make([]T, n)` has n ZERO elements. The same
number in two different roles, which is exactly the shape of mistake that compiles and means
something else. Everything else refuses by name, including the map and channel forms: a map's target
is a decision about which map, and a channel's is a decision about the ported program's concurrency.

`Digest` now refuses on `append`, which is the next honest thing and a harder one — the source
returns a new sequence and the target mutates in place, so the assignment `b = append(b, x...)` is a
shape change rather than an expression mapping.

**What this phase is really about.** Coverage moved 52.9 → 58.8 in R2b and has not moved since, and
that is not failure: each step replaced a wrong or vague reason with the true next one. A cascade
root is only reachable one layer at a time, and the engine now says which layer it is at.

## R2d — `append` is a statement, and the spread was never recorded

`Digest` refused on `append`, which is the source's most common sequence operation and the shape the
plan lists under "AssignStmt forms". Two things were wrong, and the first was a FACT the front end
did not have.

**The spread was never recorded.** `append(b, xs...)` adds the ELEMENTS of `xs`; `append(b, x)` adds
`x` itself. Nothing in the tree distinguished them — the ellipsis was not carried at all — so the two
arrived identical and any rule would have translated both the same way and one of them wrongly. The
call now carries the flag.

**And `append` is a STATEMENT, not an expression.** The source's returns a new sequence and the
target's `extend` mutates in place and returns nothing, so there is no expression correspondence to
write; the assignment as a whole is what translates.

**The same name on both sides is what carries across, and that is the whole of the condition.** The
source leaves whether the result shares the argument's storage to the capacity at run time, and that
question has one answer only when the result replaces the original — nothing else can observe the
difference. `c = append(b, ..)` refuses by name: both target answers, extending `b` and aliasing it
or cloning it, are a different program on one of the two run-time paths. Several plain elements at
once refuse too, because the target spells that as a temporary sequence, which allocates where the
source did not.

`Digest` now waits on `appendUint64`, which reads through `binary.LittleEndian` — a foreign package
the pack has not mapped. Four layers of this cascade have been peeled and each one named the next
honestly.

## R2e — the third of three stacked gaps

Ranked by PACKAGES blocked rather than by count — which is the method's criterion and not the one I
had been reading — the top decidable cause is the package-scope `var`: 4 packages and 42 refusals
across its two variants, plus 14 more that read one. It is also the first item in the plan's blocker
list, which names three stacked gaps: no initializer recorded, no package-var write analysis, `init`
bodies never indexed.

Two were closed earlier this session — the write analysis learned where a write happens and that an
indexed assignment or a slice hands out the same licence an address does, and constants got their
initialising expression. **The third is closed now.** `go/types` omits `init` from package scope, so
the extractor indexes those bodies itself and records what the initialiser assigns.

Only where it assigns the variable EXACTLY ONCE, and only a plain `=`. A variable the initialiser
writes twice has no single expression that is its value, and a read-modify-write reads a value from
before the initialiser ran — which is the zero, and a different question.

**What this changes is the refusal, not the coverage, and that is the honest result.** The reason
used to say the missing thing was a FACT the front end could not see. It is not any more: the fact is
there, and what remains is the FORM. The target's lazily-initialised global computes at first use
where the source computed before `main` — invisible for a value the expression merely computes, and
a different program for one whose computation has effects. Deciding it needs a purity proof, and the
reason now says so rather than implying a decision is being weighed.

A variable the initialiser fills in a LOOP still lacks the fact and is named separately: there is no
single expression that is its value, and `xid`'s decoding table is one.

## R2f — a rule I wrote, measured, and reverted

`xxhash` emits a private `SliceHeader { s: String, cap: i64 }`. In the source it mirrors the
runtime's slice layout so a string can be reinterpreted as a byte slice through an unsafe pointer;
both functions that do that refused, and what was left is a struct whose fields could never mean what
they meant, in a crate that denies `unsafe`. A reader meeting it has no way to know it is residue.

So I wrote the mirror of self-containment: that rule refuses a declaration which names something
absent, and this would drop an unexported one that nothing present names. Bounded to unexported
names, because an exported one is API and a caller outside decides whether it is used.

**It was wrong, and the measurement said so immediately.** It removed `SliceHeader` — and also
`PRIMES`, `MARSHALED_SIZE`, and `merge_round`, which is the function a reviewer had singled out as
the part that reads well. Every one of them is genuinely unreferenced in the emitted crate, and that
is the flaw: in a PARTIAL port almost nothing anchors the reference graph, because the exported
declarations that would anchor it are the ones that refused. Reachability then measures what the
refusals happened to orphan rather than what the source means, and deletes accordingly.

**What the case actually needs is narrower and is recorded rather than built.** `unsafe.Pointer` is
the source's escape from its own type system, and a declaration existing to be reinterpreted through
it describes the SOURCE RUNTIME's memory layout — which the target does not share. Such a type cannot
be ported at all, and refusing it BY NAME for that reason is right where dropping it for being
unreferenced is not. It needs the front end to record which references sit inside that escape hatch,
which it does not today.

Worth keeping as a general caution: a rule justified by "nothing uses it" is measuring the engine's
own refusals, not the source. The engine's coverage is the wrong thing to reason from.

## R2g — a blind review of current output, and the three rules it produced

Ran a blind review on the emitted `xxhash` module — the measurement the standing goal asks for and
the one that had gone stalest. Verdict **DO NOT MERGE**, and identified as "mechanically translated,
source language Go" with five tells ranked by decisiveness. No correctness finding: the reviewer
verified all five xxHash64 primes against the specification and confirmed every operation wraps.

Three of the five tells were provable and became rules. All three change SPELLING, not coverage —
which stayed exactly where it was (xxhash 58.8%, semver 24.1%, ksuid 17.2%, xid 11.5%, uuid 11.3%,
errors and multierror 0.0%).

**Tell 5 → `bit_pattern_constants`.** The primes were emitted in decimal, and the reviewer said
plainly that they had to run a script to check them, "exactly the wrong property for a hash crate's
magic numbers". A count belongs in decimal and a bit pattern in hexadecimal, where it can be checked
against whatever defines it.

The interesting part is the discriminator, because the obvious one is WRONG. Magnitude looks like it
separates them and does not: measured across the corpus, seven package constants exceed the 32-bit
line, and two of them — uuid's `g1582` and `g1582ns100`, the ticks between 1582 and the Unix epoch —
are counts. What separates them is the TYPE. The counts are typed at the source's counting integer;
the five that are patterns are typed at a fixed-width UNSIGNED one, which is what an author reaches
for when the bits are the point. The rule fires on exactly those five, corpus-wide, and every count
stayed decimal. The emitted hex now matches the reviewer's own specification table character for
character.

**Tell 4 → `retain_used`, and the boundary that makes it legitimate.** The unit emitted
`pub type BoxError` and `pub type Result<T, E = BoxError>` when nothing in it was fallible; the
reviewer read them as a design baked in before anything needed it, "the signature of translating a
language where every function returns error". The cause: `unit_can_fail` asks the SOURCE, and a unit
whose every fallible function REFUSED still answers yes.

This is the same rule shape that was reverted one phase ago in R2f, and the difference is the whole
point. A prelude alias is the ENGINE'S OWN INTRODUCTION — nothing upstream asked for it and no
caller of the source can be relying on it — so the engine may withdraw its own offer when its own
output does not use it. It may not delete the author's declaration for going unused, because in a
partial port a source declaration is unreferenced mostly because whatever would have referred to it
refused. The engine's coverage is the wrong thing to reason from; the engine's own introductions are
the one thing it IS.

The file this landed in already carried the principle for imports: *"asked of the output rather than
of the declarations because an import nothing uses is a denied warning"*. Both now live in
`emitted_names.rs` — rules that ask the output, not the model that produced it — and both fail SAFE,
keeping the name when the output cannot be inspected.

One bug worth recording, caught by re-measuring rather than by any gate: the first cut matched the
alias name by splitting on non-identifier characters, so `fmt::Result` in a `Display` impl counted as
a use of the crate's `Result` and kept the alias alive in three packages that never used it. A
qualified name is a different name; the alias is introduced unqualified and can only be referred to
that way inside its own module.

**Tell 6 → the same rule.** `use std::error::Error as StdError` renamed a type with nothing to
collide with; it exists only to spell `BoxError`, and went with it.

**Tell 1 is not fixed and is the decisive one.** `SliceHeader` remains. R2f establishes what it needs
— a refusal by name on the ground that `unsafe.Pointer` is the source's escape from its own type
system, so a type reached only through it describes the source runtime's memory layout, which the
target does not share. The reviewer reached the same conclusion independently and by the same
reasoning: "a Rust author would never invent it, because `as_bytes()` already solves the problem for
free." That needs the front end to record which references sit inside that escape hatch.

**Tell 2** — the comment "Store the primes in an array as well.", carried over with its justification
(the source's hand-written assembly needs a contiguous array) left behind — has no provable rule yet.
It is not the existing prose refusal's shape: the sentence names nothing about the source language.

**An honest consequence of `retain_used` worth stating.** No package in the corpus now emits the
failure aliases, because no package currently emits a single fallible function — every one of them
refuses. The error model defended three times over is, at this moment, entirely unexercised by the
output. That is not an argument against it; it is a statement of what the corpus currently proves,
which is nothing either way.

**Refusal ranking after this phase**, by packages blocked rather than by count in any one: *a failing
return's operand must be PROVABLY a failure* now blocks FIVE of seven packages (semver 3, ksuid 2,
uuid 2, errors 1, multierror 1) and is the single largest cause in the corpus. It is what stands
between `errors` and `multierror` and any coverage at all.

**Housekeeping forced by the above.** Three files crossed the 300-line ceiling and were split along
real seams rather than at convenient offsets: `emitted_names.rs` (rules that ask the output),
`value_rules.rs` (what a source VALUE becomes, as pack data), `load_values.rs` (their loaders). Each
is registered in its crate's `sources.rs` and in the neutrality fence — the fence caught
`load_values.rs` before I did, which is the gate working. `core/kernel/src/lib.rs` at 520 lines
remains a DECLARED exemption: it refuses submodule declarations so that "the kernel is exactly this
file" is a property of the build rather than of a scan. `core/transform/src/apply.rs` is at 309 and
was already at 308 before this phase — a real breach, not yet fixed, recorded rather than hidden.

Verification: 11 crates' tests green by exit code; clippy `-D warnings` green over the port-engine
crates; `delta` Green/Unchanged; golden byte-identical; engine source bytes moved, so `engine_digest`
moves with them; `rustc` and `clippy-driver` both green with `--deny=warnings` over all five packages
that emit anything, compiled under `#![forbid(unsafe_code)]`. The pre-existing clippy failure in
`ci/facade/cloud-name-ratchet` is outside this lane and untouched by this branch (last changed by
PR #2102).

## R2h — the largest refusal in the corpus was a lie

Ranking causes by packages blocked put this at the top: *"`composite` needs `type`, which the front
end did not record"* — 13 sites across FOUR packages, the largest single cause by both measures.

It was false. The front end records a type on every composite literal in every snapshot — checked
directly rather than inferred. What actually happened is that `resolve()` failed on that type, and
the call site replaced the resolver's own error with a fabricated `MissingDatum` naming a datum that
was present. Anyone following that refusal went to the extractor, where there was nothing to find.

Fixed by propagating the resolver's real error, which is one character of code and reclassifies all
thirteen. Every one of them turned out to be a CASCADE: the package's central type refused, and each
of these was a literal constructing it. `semver.Version` 8→9 sites, `xid.ID` 4→5, `uuid.UUID` 9.

This is worth stating as a standing rule, because the engine's whole contract rests on it: **a
refusal that misdescribes what is missing is worse than no refusal, because it is acted on.** The
goal says the engine must refuse by name, *saying what is missing* — a fabricated cause satisfies the
first half and inverts the second. Nothing failed here; every gate was green throughout. The only
thing that could have caught it was reading the refusal and checking whether it was true.

**What the corrected ranking shows.** The dominant structure in the corpus is not any one construct.
It is a CASCADE: uuid.UUID 9 sites, semver.Version 9, uuid.Variant 6, xid.ID 5, ksuid.KSUID 3,
uuid.Domain 3 — every one of them "declared in this unit and not emitted, because it refused". Six
central types account for 35 refusals, and each is a package's principal export.

**And the cascade has a single mechanical cause, now located.** `items_types.rs` builds a struct with
`methods: inherent_methods(declaration, resolver, body)?`. The `?` is the whole story: ONE method the
engine cannot translate refuses the ENTIRE TYPE, the type is then not emitted, and every declaration
that mentions it refuses in turn. A package's coverage is therefore capped by its single hardest
method, which is why these numbers look the way they do.

Both languages separate a type from its methods — Go declares them outside the struct, Rust puts them
in their own `impl` block — so a method that cannot be translated does not require refusing the type
whose shape it never affected. Dropping one is already safe by machinery that exists: self-containment
refuses any call to something not emitted, so a caller of a dropped method refuses by name. The
discipline it must not break is that the METHOD's own refusal has to keep being reported by name;
swallowing it would trade a loud cascade for a silent hole, which is the worse failure.

That is the next change, and it is structural: the per-method refusals have to reach the survey as
their own entries rather than as one error for the declaration. `DispositionLog` is the precedent for
threading that through the resolver.

## R2i — breaking the type/method cascade: built, measured, not landed

R2h located the cascade's mechanism. This phase built the fix, measured it, and did NOT land it,
because it regresses two properties the engine already holds. What follows is the complete design and
the ranked worklist it produced, so the next attempt is execution rather than rediscovery.

**The change.** `items_types.rs` builds a struct with `methods: inherent_methods(declaration, …)?`.
Replacing that `?` with a per-method match — keep what translates, record what does not — takes four
edits:

1. `dropped.rs`: a `DropLog` with interior mutability, the `DispositionLog` precedent, holding
   `{owner, name, reason}` per dropped method.
2. `signature.rs`: `inherent_methods` matches per method instead of collecting into `Result`.
3. `survey.rs`: after the declaration's own outcome, every logged drop is pushed as its own refusal
   entry, named `Owner::Method`, kind `method`.
4. `reachable.rs`: the fixpoint must PARTITION. `shrink` removes a name when `round.refused` is
   non-empty, and a drop arrives in that same list — read naively it evicts the type that was just
   successfully emitted, rebuilding the cascade through a harder-to-see door, since the type would
   then be emitted and simultaneously unnameable. The type keeps its name; each SURVIVING method
   earns a qualified one, `Owner::Method`, in the same set. A body calling a dropped method then
   names something the crate does not contain, which is the rule that already governs every other
   reference, and the set still only shrinks — so the fixpoint's termination argument is untouched.

**It works, and the measurement is unambiguous.** Every package's principal export emits for the
first time: `semver::Version`, `uuid::Uuid`, `xid::Id`, `ksuid::Ksuid`. `errors` and `multierror` go
from emitting NOTHING to emitting something. Dropped methods are reported by name — `method
Version::IncMajor`, `Version::UnmarshalText`, `Version::Scan` — so nothing goes missing quietly.

Coverage percentages are NOT comparable across this change and should not be quoted as a gain: the
denominator moves, because a type with N untranslatable methods used to be one refusal and becomes
one translation plus N. Absolute translated counts rose in six of seven packages.

**Why it did not land — two regressions, both real.**

*The compile proof.* The emitted crates stop compiling: six of seven fail `clippy-driver
--deny=warnings`, where before the change five of five passed. Nothing new broke. The defects were
always in the engine and were masked by the fact that almost nothing reached them, which is precisely
the failure mode the standing method warns about — a corpus that only contains what the engine
already handles proves nothing about what it does not.

*Two refusal invariants.* `an_escaping_receiver_is_refused_with_its_reason` and
`a_failure_the_engine_cannot_prove_is_refused_by_its_operand` both fail. They are not wrong: the
refusal corpora still refuse, but now against the METHOD rather than the declaration, and those tests
are the specification of which. Changing what "refused" means for a refusal corpus is a decision that
deserves its own phase and its own justification, not a side effect of an unrelated fix.

**The worklist this produced, ranked by errors and by packages.** These are pre-existing engine gaps,
newly visible:

1. **A method call on an `Option<T>` receiver — 44 of semver's 53 errors, one cause.** A
   pointer-typed receiver maps to `Option<T>` and the call lands on the option rather than on what it
   holds: `no method named 'major' found for enum Option<T>`.
2. **`impl Box<dyn StdError + Send + Sync> for X` — 3 packages** (uuid, errors, multierror). The
   source interface has two target spellings and the engine uses one for both: as a VALUE it is a
   boxed trait object, in TRAIT POSITION it is the trait itself. The value spelling in trait position
   is not a trait at all.
3. **Indexing a newtype — 3 packages.** `cannot index into a value of type &Ksuid` / `&Id` /
   `&Collection`. The source's named array type indexes directly; the target's newtype needs its
   field first.
4. **A string slice where an owned string is required — semver.** `&self.original[..1]` is `&str` and
   the signature says `String`. The source's slice of a string yields a string; the target's yields a
   borrow.
5. **The method self-containment check is inert on a `self` receiver.** Written this phase and
   verified not to fire for `self.is_nil()`, because the receiver node carries no recorded type — the
   owner has to come from the enclosing declaration in that case, not from the receiver.
6. `E0063` missing struct field (ksuid) and `E0425` unresolved name (errors), one each, not yet
   diagnosed.

**The one structural conclusion worth keeping.** The engine's compile proof runs against the HERMETIC
corpus only; real-repo output has never been compile-checked by any gate. That is why every gate
stayed green through a change that stopped six crates from compiling, and it is the gap that let all
six defects above sit unseen. The engine's own libraries may not invoke a compiler (the ADR-0638 D3
firewall, which the existing proof documents carefully), so the answer is not to make the engine
compile its output — it is that the real-repo compile proof must become a gate that runs, and that
each defect class above must become a static rule that REFUSES rather than emits.

The diff is 283 lines and was reconstructible from this entry alone; it was kept out of history rather
than committed and reverted, so no bisect lands on a state that does not build.

## R2j — the decisive tell, refused by name

Four blind reviews running, `SliceHeader` has been the single most decisive evidence that the output
was mechanically translated. R2f established what the case needs and why the obvious instrument was
wrong; R2g confirmed an independent reviewer reaches the same conclusion by the same reasoning. This
phase built it.

**The rule.** `unsafe.Pointer` is the source's escape from its own type system. A type whose EVERY
reference in its own package sits inside that hatch is not describing a value — it is describing how
the source runtime lays one out in memory, and the target does not share that layout. So there is
nothing for the fields to mean, and the type is refused BY NAME rather than ported into a crate that
denies `unsafe`, where a reader has no way to identify it as residue.

EVERY reference, not merely one: a type used both ways is a real type that also happens to be
reinterpreted somewhere, and refusing that would be refusing the author's work on the strength of a
single use.

**The front end had to learn to see it.** Checked before assuming: the string `unsafe` does not occur
anywhere in the xxhash snapshot. Both uses of `sliceHeader` sit under a `StarExpr` that the walker
records as unsupported without descending, so the reference and the hatch were both lost before any
rule could look. `unsafeuse.go` adds a package-wide pass that counts, per type of this package, how
many of its USES (never its definition) sit inside an expression mentioning the `unsafe` package —
resolved through the type-checker, so a local variable named `unsafe` is not the hatch. Equal counts
and non-zero means the flag.

**Measured over the whole corpus: one hit, zero false positives.** `xxhash.sliceHeader` and nothing
else across seven packages and 336 declarations. The emitted xxhash module no longer contains
`SliceHeader`, and the survey reports the refusal against the type's own name with the pack's reason.

**The cost, stated plainly.** xxhash coverage falls 58.8% → 55.9%. Refusing a declaration always
does. It is the right trade and the direction the standing goal points: the type that came out was
`struct SliceHeader { s: String, cap: i64 }`, which compiles, means nothing, and is exactly the
"output that compiles and means something different" the engine exists to prevent.

**Shape notes for the next rule of this kind.** The reason is pack data keyed by the FACT rather than
by the type's name — a property that makes a type unportable holds for every type that has it in any
package, and keying by name would have made the rule a list of one repo's types. It went in as a
sibling map inside the existing `unmappable_types` rule, which already means "no faithful target
form, and the pack says why", so it cost one field rather than a whole new table.

The refusal is asked BEFORE anything is built, and that ordering is the point: nothing about such a
type's translation is wrong. Every field maps, the struct renders, the result compiles. What is wrong
is that the thing it describes does not exist on this side, so a faithful-looking translation is the
worst available outcome.

**One process note.** Three crates appeared to hang for 90 seconds after this change and did not —
the extractor and pack edits invalidated the build cache, and a cold compile of those test targets
outran the window I gave it. Worth remembering before diagnosing a fixpoint loop that is not there:
after a front-end or pack change, the first `cargo test` is a cold build.

Verification: Go tests green; 11 crates' Rust tests green by exit code; clippy `-D warnings` green;
`delta` Green/Unchanged; golden byte-identical; `clippy-driver --deny=warnings` green over all five
packages that emit, under `#![forbid(unsafe_code)]`.

## R2k — the rule was right, the call path was not

A blind review of the current output (xxhash + semver, three files, reviewer unaware of provenance)
moved the verdict from DO NOT MERGE to **MERGE WITH CHANGES**, and `SliceHeader` — the most decisive
tell in four consecutive reviews — is gone from the list entirely. R2j worked.

The new number-one tell was `errors.Is`, named in a Rust doc comment on a public enum variant. The
reviewer called it "the single most decisive line in the corpus": a Go standard-library function left
in prose the port is supposed to own.

**The rule that forbids it was already written and already correct.** `docs_refuse` drops any
sentence naming `pkg.Ident` where `pkg` is not a unit of this model — the prose mirror of the code
rule that refuses reading through a package the snapshot does not contain. It would have caught this
sentence on the first pass.

It never ran. `docs_of` dropped the sentences and then called `docs_from_block` to format them; the
grouped failure enum builds a variant per sentinel and calls `docs_from_block` DIRECTLY, so every
variant's prose skipped the check. The doc comment above that function says it was factored precisely
so "two spellings of a doc rewrite would drift exactly as two spellings of a name did" — and they
drifted anyway, because what was factored out was the REWRITING and not the REFUSING. One path
remembered to refuse first and the other did not.

Fixed by moving the drop inside `docs_from_block`, where every path goes through it and the bypass
cannot recur. The only thing the check needed from the declaration was its own name, so it takes a
name now instead — which is exactly why it was possible to forget: it never really needed the
declaration at all.

**The general lesson, and it is not about documentation.** A rule that lives in the caller is a rule
that is optional. This engine's whole contract is that what it cannot prove it refuses, and a refusal
sited one level above the thing it guards is one new call site away from silence. Worth checking
wherever else a check and the operation it guards are separated: the check belongs inside the
narrowest function that every path must go through.

The emitted variant now reads "Incrementing a version segment would exceed the maximum value of a
u64." — true, complete, and no longer describing an error design this port has not made.

**What the review ranks next, all still open.** In its order: `xxhsum::contains`, a hand-rolled
linear search over a slice where the target has had `.iter().any()` since 1.0 and which the reviewer
called "the single most recognizable Go helper in existence"; `MAGIC`/`MARSHALED_SIZE` as an
undocumented wire format; the `inc_major_e`/`inc_minor_e` naming convention, which is Go's workaround
for having no `Result`; `compare_segment`, a wrapper around `u64::cmp` that exists only in a language
without an ordering trait; and `NUM`/`ALLOWED` as character tables where the target has `char`
predicates. The `PRIMES` comment persists at rank 7.

**Separately measured this phase, not yet built.** `errors` sits at 0% because its central types
refuse, and they refuse for one reason: a method whose SOLE result is the source's error type is
treated as fallible, so `Cause() error` becomes `Result<(), E>` and `return w.cause` must be PROVEN a
failure, which a stored field cannot be. The mapping is wrong before the proof is: `Ok(())` would
mean "there is no cause" and `Err(e)` would mean "the cause is e", which is the failure channel used
to carry data.

A sole `error` result is ambiguous in the source between the failure channel and an error returned as
data, and the discriminator is measurable. Counting across the corpus: 45 functions have a sole error
result; 13 never return the absent value on any path and 32 do. The split is exactly right — every
`Cause`, `Unwrap` and `StackTrace` getter falls on the data side along with the `New`/`Errorf`
constructors, and every `Unmarshal*`, `Scan` and `validate*` falls on the channel side. Ten of the 13
are in `errors`. The target form for the data side is `Option<E>`, where absent is `None` and no
proof is required of anything.

Verification: 11 crates' tests green by exit code; clippy `-D warnings` green; `delta` Green/Unchanged;
golden byte-identical; `clippy-driver --deny=warnings` green over all five packages that emit, under
`#![forbid(unsafe_code)]`. Coverage unchanged, as expected for a prose rule.

## R2l — the error model has a nullability hole, and it is upstream of three separate symptoms

Chasing why `errors` sits at 0% led to one root cause that also explains two findings recorded
elsewhere as if they were unrelated. Written down before building, because the fix is one coherent
change and slicing it leaves the pieces disagreeing with each other.

**The hole.** The source's `error` is an INTERFACE VALUE and is nullable — `nil` is a legal value of
it in every position. The pack maps it, in every position that STORES the value, to
`Box<dyn StdError + Send + Sync>`, which is not. The comment at that branch is careful and correct
about why the form is owned and boxed (a failure outlives the call that produced it), and it names
"a field, a result and a composite element" as sharing that property. What it does not address is
that the source value can be absent and the target type cannot say so.

So a struct field holding a source error is typed as though it always holds one. That is a nullable
value mapped to a non-nullable type, which is the class of defect this engine exists to prevent, and
it is currently invisible because the declarations that would expose it refuse first.

**Symptom one — `errors` at 0%.** Its central types (`withMessage`, `withStack`, `fundamental`,
`stack`) refuse, and 8 of the package's 19 refusals are cascades from them. The trigger is
`func (w *withMessage) Cause() error { return w.cause }`: a sole `error` result is read as the
failure channel, so the method becomes `-> Result<(), E>` and `return w.cause` has to be PROVEN a
failure, which a stored field can never be. The refusal is correct given the mapping. The mapping is
what is wrong: `Ok(())` would mean "there is no cause" and `Err(e)` would mean "the cause is e" —
the failure channel carrying data.

**Symptom two — the sole-result ambiguity, now measured.** A sole `error` result means two different
things in the source and the discriminator is mechanical. Across the corpus, 45 functions have one:
**13 never return the absent value on any path, 32 do.** The split lands exactly where the semantics
say it should — every `Cause`, `Unwrap` and `StackTrace` getter on the data side together with the
`New`/`Errorf` constructors, and every `Unmarshal*`, `Scan` and `validate*` on the channel side. Ten
of the 13 are in `errors`. A validator that returns `nil` on success is a channel and stays
`Result<(), E>`; a getter that never returns `nil` literally is handing back a value.

**Symptom three — the `impl Box<dyn StdError + Send + Sync> for X` defect from R2i.** Recorded there
as its own item across three packages. It is the same root: one source type with several target
spellings selected by POSITION, and the engine holding fewer spellings than there are positions. As
a bound or an impl target the answer is the trait; as a value known present it is the boxed object;
as a value that may be absent it is the option of that.

**Why it is one phase and not three.** The three spellings have to agree. Give the getter an
`Option<E>` result while its backing field stays non-optional and the body cannot typecheck; fix the
field alone and every existing result disagrees with it. The existing `trait_object_forms` table is
the right shape to extend — it already maps POSITION to target form for interface types, with
`param`, `trait` and `supertrait` declared and reasoned — and the failure convention needs the same
treatment rather than a single `target_type` that answers for every storing position at once.

**The one thing already verified about the shape.** `trait_object_forms` proves the pack can express
position-dependent spellings and that the engine consults them; this is an extension of a mechanism
that exists and is justified, not a new concept.

Nothing was built this phase. The measurement is the deliverable: 13/32, the discriminator, and the
fact that three symptoms recorded separately are one cause.

## R2m — the nullability hole closed, and a recorded decision overturned

R2l diagnosed it; this builds the part that is provable. The source's `error` is a nullable interface
value and the pack mapped it, in every position that stores one, to a target type that cannot be
absent. Two changes, both narrow:

**A stored failure is optional.** A FIELD of the failure type now takes `Option<Box<dyn StdError +
Send + Sync>>`. Seeded from `rust-skills/rules/type-option-nullable.md` (MIT,
d525d2c8ff47f5f08d038319f89cacf9e9f1ee60): use `Option<T>` for a value that might not exist, so
absence cannot masquerade as a normal value. One line moved in the hermetic golden —
`Report.cause` — which is the corpus proving it had the case all along.

**A getter of a stored failure lends it.** A sole failure result whose EVERY return reads a field of
the receiver becomes `Option<&(dyn StdError + Send + Sync)>`, and the read becomes `as_deref`. This
is the target's own shape for it: `std::error::Error::source` has exactly that signature.

**The overturn, stated plainly.** The refusal fixture's own documentation recorded the opposite
decision — that a getter's optional form would be *"reading intent from a shape, not proving it"*, so
the engine refused and named the missing proof. That objection was right about the rule I first
wrote. My initial discriminator was "no return hands back the absent value", which cannot tell a
getter from a validator that delegates: `func Validate() error { return doCheck() }` never returns
`nil` literally either, and turning that into `Option<E>` would change a fallible operation's API on
a shape rather than a proof.

So the rule was narrowed to what is actually proven: every return reads a field of the RECEIVER —
the same proof `borrows_from_receiver` already makes for strings, and already accepted as sound. The
claim is not that the function is semantically a getter. It is that the value handed back is a stored
field that may be absent, and an optional borrow says exactly that, where `Result` would claim an
operation succeeded or failed and `Ok(())` would have to mean "there is no cause".

**The fence was preserved rather than deleted, which is the part that mattered.** With `Cause`
translating, the fixture no longer refused anything, and the honest response to a failing invariant
is not to weaken it. A `Check() (int, error)` case was ADDED, returning the same possibly-absent
field in a trailing position beside a companion result. There the operand is the CHANNEL, `Err(..)`
is unconditional, and no alternative spelling exists — so it still refuses, still by OPERAND, which
is what keeps it distinguishable from `Wrapped` returning a declared constructor.

**Two things the engine's own gates caught, both real.** The compile proof rejected the first
attempt: `Option<Box<dyn Error>>` is not `Clone`, so a getter could not hand its field back at all —
which is what forced the borrowed form rather than a preference for it. And the fixture's content
address rejected the edit until its digest was recomputed, which is the snapshot layer doing its job
on a hand-edited fixture.

**Licensing.** The new idiom was written before its provenance was found, and the loader refused it
for missing `seed_source`. The seed was then located rather than invented:
`rust-skills/rules/anti-clone-excessive.md` — *don't clone when borrowing works* — which here is not
merely cheaper but the only spelling that exists, since the target's boxed failure is not clonable.

**Coverage is unchanged everywhere, and that is expected.** `errors` stays at 0%: with the error
model corrected, `withMessage` now refuses on `fmt.Fprintf` in its `Format` method — which is the
R2i type/method cascade, one hard method sinking its whole type. The two findings compose exactly as
predicted, and `errors` needs both.

## R2n — what the transpiler literature says this engine is missing

Researched rather than recalled, and mapped onto measured gaps. Three findings change what should be
built next; one confirms an existing decision was right.

**1. Ownership belongs in a LATTICE, solved by fixpoint — not decided per site.**
C2Rust's ownership analysis assigns each pointer a permission from `READ < WRITE < MOVE` and solves
for a consistent assignment; Laertes and the CAV'23 ownership-guided work extend it with lifetimes.
This engine instead answers per site from flags (`mutated`, `escapes`, `effect_unknown`, `rebound`)
plus a pack table of dispositions. That works, and it is why R2m had to be discovered BY HAND: "a
stored failure may be absent" is a nullability qualifier, and the engine has no place to put one, so
it became a bespoke rule for one type in one position.

The generalization is mechanical: every type occurrence carries a qualifier — present/absent-capable
crossed with owned/borrowed/shared — and the qualifiers are solved as a monotone fixpoint over
constraints generated from the source. The engine already runs one monotone fixpoint (emittability)
and knows why that shape terminates, so this is a second instance of a mechanism it has, not a new
concept. It would have produced R2m automatically, and it is the same machinery the R2i worklist
needs for its largest single defect (44 of semver's 53 errors: a method called on an `Option<T>`
receiver, which is a qualifier mismatch at a call site and nothing else).

**2. Translation validation is the missing THIRD evidence source, and it has a formal shape.**
The literature's term for what this engine lacks: rather than proving a translator always correct,
each individual run is followed by a validation phase establishing that the target refines the
source, via a control mapping from target locations to source locations and a data abstraction
mapping between their variables. Alive2 does this for LLVM IR.

The engine currently has two correctness arguments and neither is refinement. Determinism (the
six-axis receipt) proves the same input yields the same bytes. Refusal proves the engine declined
what it could not justify. Neither says the emitted Rust MEANS what the Go meant. The honest form
here is a differential oracle over generated inputs, and the engine is the only component that can
emit its harness, because it alone knows the Go↔Rust symbol correspondence and which declarations
refused. Every assertion in that harness must still originate outside the engine.

The literature is also clear about the limits, which match the constraint already in force: these
validators cannot handle unbounded loops, external calls, or complex arithmetic for an SMT solver.
Value-level comparison is likewise blind to aliasing and mutation visibility, concurrency, allocation
behaviour, and part of the panic/error boundary. So the division of labour is not a compromise, it
is the design: **what a differential run cannot witness, the engine must refuse rather than guess.**
That gives a usable admission test for any new rule — if its correctness can be witnessed neither by
the compiler nor by a differential run, it needs a written proof in pack data, not a plausible reason.

**3. Being syntax-directed is the thing to keep avoiding.** C2Rust is explicitly syntax-based: it
rewrites C pointers to `*mut` and keeps unsafe semantics, and the whole Laertes/C2SaferRust line
exists to lift that output afterwards. This engine's refusal-first, zero-`unsafe` stance is the
opposite bet, and the research supports it — a syntax-directed port produces code that compiles and
requires a second research programme to make safe. R2j is this engine's version of the same lesson
in miniature: the type whose meaning was the source's memory layout had no target spelling at all,
and emitting it would have been syntax-directed translation of something with no semantics to carry.

**4. Statically checking ownership is undecidable, so incompleteness is expected.** Rust's own borrow
checker is deliberately incomplete. That is the licence for this engine to refuse and the reason
refusal is not an admission of weakness — but it also means "the engine refuses" can never be
evidence that the emitted part is right, which is finding 2's whole point.

**Ranked, by goal impact times provability.** (i) Qualifier lattice, because it generalizes R2m,
unblocks R2i's largest defect, and replaces hand-discovery with inference. (ii) The differential
oracle plus mutation testing to give it teeth, because it is the only evidence that originates
outside the engine and can witness meaning. (iii) SSA and def-use chains, which would replace the
read-count and "read exactly once" approximations behind the accumulator fold — the area that has
produced more of this session's real bugs than any other.

Sources: c2rust.com/manual (ownership analysis), CAV'23 ownership-guided C-to-Rust,
arxiv 2501.14257 (C2SaferRust), Pnueli et al. translation validation, Alive2.

## R2o — two position rules the cascade break needs, landed ahead of it

R2n said the engine holds fewer target spellings than the source has positions. Both of these are
that finding, made concrete, and both were found by re-applying the R2i cascade break and reading
what stopped compiling. They land alone because they are correct alone.

**A failure type in TRAIT position is the trait.** The failure convention answers for VALUES, and its
branch ran before the table that maps position to form for interface types — so an observed
satisfaction of the source's error interface emitted
`impl Box<dyn StdError + Send + Sync> for Fundamental`, which names a struct where a trait belongs.
Three packages failed on it. Letting the trait and supertrait positions fall through to
`trait_object_forms` — which already declares `trait: "{0}"` — gives `impl StdError for Fundamental`.

**An index or slice through a NEWTYPE reaches through the wrapper.** `type ID [12]byte` admits
`id[:]` because in the source the name and the array are the same thing; the target's newtype wraps
it, so the same expression needs the field first. Emitting the source's spelling gave `cannot index
into a value of type &Id` in five packages.

The interesting part is what the body can actually know. The front end records a type on an
expression only where one is needed, and a receiver carries none — so the identifier says nothing.
What the body does know is which declaration it is inside, and the scope now maps a unit's named
types (those whose underlying is not a struct or interface, which are exactly the ones emitted as
newtypes) so the receiver case can be answered. An index through any other binding of a newtype is a
shape the corpus does not have and arrives unchanged rather than being guessed at.

**Both are latent right now, and saying so matters.** Coverage is unchanged across all seven
packages, because the declarations these rules govern only get emitted once the type/method cascade
is broken. They are prerequisites, verified by the compile proof under the break and landed without
it because neither depends on it.

**Where the cascade break stands, measured this phase.** Re-applied on top of R2j–R2m it is
materially better than in R2i, because the error-model fix landed in between: `errors` goes 0 → 7
translated (0% → 22.6%), `multierror` 0 → 1, `uuid` 11 → 29, `semver` 14 → 24, `ksuid` 16 → 25.
Every package gains in absolute translated count.

With these two rules applied, **`xxhash` and `xid` compile clean under the break**; four packages do
not. The remaining tail, ranked:

1. **A method called through an absent-capable receiver — 39 of semver's 51 errors, one cause.** A
   source pointer maps to an option and the call lands on the option rather than on what it holds.
   This is precisely the qualifier mismatch R2n named, and the honest answer is the engine's own
   contract: refuse the call by name unless absence is disproved.
2. `E0277` and `E0407` in three packages: a trait impl whose method set is not the target trait's.
   The source's `Error() string` becomes `fn error(&self) -> String` inside `impl StdError`, which
   has no such method — the correspondence between a source interface's methods and a target trait's
   is a pack decision that does not exist yet.
3. `E0308`, `E0063`, `E0425`, `E0369`, `E0606`, one or two each, not yet diagnosed.

One consequence worth keeping: under the break `multierror` went from 1 translated to 0, and that is
the contract working rather than a regression. Its type refused because the trait impl it would need
does not compile, so the engine now declines instead of emitting something broken.

**An operational incident, recorded because it nearly corrupted the evidence.** The worktree was
deleted mid-session by something outside it. Every commit survived on the branch, but the recreated
worktree lacked the root `Cargo.toml` — which is NOT tracked on this branch, only in the main
checkout. Cargo therefore walked up and resolved the workspace to the main repository, which is on
another lane's branch: for several commands the builds and tests were compiling somebody else's code
while reporting success for mine. It surfaced only because a deliberate `missing field` error failed
to appear when it should have.

Two lessons. A verification that cannot fail is not a verification, and the tell was a gate that went
green when it had no right to. And `cargo metadata --format-version 1 | workspace_root` is the
one-line check that says which tree is actually being built — worth running after any change to the
working directory. The lane now builds with a generated minimal workspace listing only the
port-engine crates, kept untracked, and with its own `CARGO_TARGET_DIR` so it no longer shares the
other lane's artifacts.

## R2p — four qualifier rules, and the cascade break down to fourteen errors

Working method this phase: re-apply the R2i cascade break as scratch state, read what stops
compiling, turn each cause into a rule, and land the rules alone. The break itself still does not
land, because it cannot until every package compiles.

**Where the break now stands.** Under it, `xxhash`, `uuid`, `xid` compile clean and `semver`, `ksuid`,
`errors` carry **fourteen errors between them**, down from roughly eighty-seven when R2i measured it.
Coverage under the break: `errors` 0 → 6 translated, `uuid` 11 → 26, `ksuid` 16 → 21, `semver` 14 → 19.

**The rules, each measured by what it removed.**

*A method call through an absent-capable receiver refuses* — took semver from 51 errors to 11, the
single largest cause in the corpus. The source's pointer admits its absent value, so the target holds
it as an option; the source spells `c.con.Major()` and calling a method on an absent pointer is legal
there, where the target has no such method on an option at all. Neither invention is faithful:
unwrapping claims a value the source never promised and panics where the source ran, and mapping over
the option silently skips a call the source made. Both are decisions about what the program DOES when
the pointer is absent, which the source states nowhere.

The check asks the RESOLVER what the occurrence resolved to rather than reading the node's kind,
because a source pointer does not always become an option — the ownership rules give some of them a
borrow, which has no absent case. Guessing from syntax would have refused calls that translate
correctly today. This is R2n's qualifier mismatch, handled the way the engine's contract says: refuse
and name the missing proof.

*A satisfaction the target trait cannot take the methods of is not emitted* — cleared `uuid`
entirely. The pack maps the source's error interface to the target's error trait, which is right for
a BOUND, but the two do not take the same methods: the source's is satisfied by one method returning
the message, and the target's takes the message from its display trait instead. The method is not
lost — it is left unclaimed so it stays in the inherent block — and the drop is recorded, because
trading a loud failure for a silent hole is the worse outcome.

*A dropped method refuses its callers* — semver 11 → 7. This is a hole the cascade break itself opens
and R2i recorded as written-but-inert: the earlier version asked the receiver node for its type, and
a receiver carries none, so it never fired for a single `self.method()` call in the corpus. It takes
the owner from the enclosing method's receiver type instead.

*A newtype parameter is indexed through its wrapper* — ksuid 11 → 3, extending R2o from the receiver
case. The body cannot learn this from the identifier, so the parameter set is threaded from the
signature exactly as the borrowed and usize parameter sets already are.

**What is landed here is only the rules.** All seven packages compile, every gate is green, and
coverage moves only slightly — `uuid` 11.3% → 11.1% and `xid` 11.5% → 11.1%, because two calls that
used to be emitted now refuse and say why. That is the correct direction: they did not compile under
the break and would not have been right without it.

**The remaining fourteen, ranked for whoever takes it next.** `errors` has eight: two method calls on
an option the resolver did not catch (the receiver's type is not recorded on those nodes), a cast of
a reference to an integer, an equality on an option, and two derive bounds — `Hash` and `Eq` derived
on a struct holding a field that implements neither, which is a provable rule since the engine knows
every field's type. `semver` has two type mismatches. `ksuid` has two empty composite literals of a
newtype, `Uint128 {}`, which should be the type's zero rather than a struct literal with no fields.

## R2q — the cascade break LANDS

Located in R2h, built and reverted in R2i, prerequisites landed in R2o and R2p. It lands here with the
compile proof green over every package that emits.

**The change, once more, because it is the important one.** `items_types.rs` built a struct with
`inherent_methods(declaration, resolver, body)?`. That `?` meant one method the engine could not
translate refused the ENTIRE TYPE, the type was then not emitted, and every declaration mentioning it
refused in turn. A package's coverage was capped by its single hardest method. Both languages declare
a type and its methods separately, and a type's shape owes nothing to any method body — so a method
that cannot be written is now dropped, and the type stands.

**Absolute translated declarations across the corpus: 63 → 87.** Per package: `uuid` 11 → 26,
`ksuid` 16 → 19, `semver` 14 → 17, `xid` 3 → 4, `errors` 0 → 2, `xxhash` 19 unchanged. Coverage
PERCENTAGES fall in places and must not be quoted as a regression: a type with N untranslatable
methods used to be one refusal and is now one translation plus N, so the denominator moved.

**Six rules were needed to make it compile, each measured by what it removed.**

*A dropped method refuses its callers* — the hole the break itself opens, and R2i recorded it as
written-but-inert because it asked the receiver node for a type receivers do not carry.

*A composite literal that loses its operands refuses.* This one is the reason the phase was worth
doing carefully. `makeUint128` returns `uint128{low, high}`, the elements are positional, the struct
path looked for KEYED children, found none, and emitted `Uint128 {}` — **both operands silently
gone**. It surfaced only because the target newtype has a field to be missing; had the type been
empty it would have compiled and meant something different, which is the exact failure this engine
exists to prevent. Two rules came out of it: a composite of a local newtype builds a literal of what
it WRAPS, and — the general one — a literal the source gave operands may never be emitted with none.

*A derive is only earned if every field earns it.* `derives_for` carried a written assumption that a
field naming another emitted type "cannot block anything, because every emitted struct gets the same
list". A NEWTYPE breaks it: one over a slice earns no total equality, and a struct holding it derived
`Eq` and did not compile. The check now follows references into this unit's own declarations, with a
visited set for the cycle a pointer can make.

*A promotion through an absent-capable field refuses.* A promoted method's body is SYNTHESISED rather
than translated, so nothing on the call path ever sees it and the receiver rule could not fire. The
source panics at the embedding when the pointer is absent, which is not a behaviour to reproduce
deliberately.

*Concatenation is not symmetric.* The source adds two strings and gets a third; the target's `+`
takes a borrow on the right. *A returned string SLICE must be owned* where the result is an owned
string — and constructed rather than converted, because a slice renders with its own leading borrow
and a method call binds tighter than it does: `&s[..1].to_owned()` is a reference to an owned string.
*A conversion FROM a newtype reaches through the wrapper*, the same helper the index path uses.

**The two refusal invariants, decided rather than deleted.** R2i deferred this and it had to be
settled: `an_escaping_receiver_is_refused_with_its_reason` and
`a_failure_the_engine_cannot_prove_is_refused_by_its_operand` both failed, because their corpora now
drop a method where they used to refuse a declaration.

Investigating rather than editing the tests found a real defect behind them. The plan-driven path used
a THROWAWAY drop log — so on the path that produces the golden and feeds the receipt, a dropped method
vanished with no report at all. The survey reported drops and `apply` did not. That is precisely the
silent hole the drop mechanism was written to avoid, reintroduced one layer down.

So `TransformOutput` now carries `dropped`, for the reason the dispositions travel with the IR and a
sharper one: a drop is invisible in the output BY CONSTRUCTION — what it leaves is a type with one
fewer method, which reads exactly like a type that never had it. Only this channel can say otherwise.
The refusal corpora then refuse again, on the same reasons, named at the method rather than at the
declaration — which is finer, not weaker. The two transform-level unit tests were updated to assert
the property they were always about: the undeclared decision is refused and its reason reported, never
invented.

Verification: 11 crates' tests green by exit code; clippy `-D warnings` green; `delta` Green/Unchanged;
golden byte-identical; `clippy-driver --deny=warnings` green over all six packages that emit, compiled
under `#![forbid(unsafe_code)]`.

**What is left.** `multierror` alone still emits nothing, blocked by `TypeSwitchStmt` and `sync.Mutex`
rather than by anything this phase touched.

## R2r — a regression found by ranking refusals, and the seven rules behind it

Ranking refusal causes after R2q put an unfamiliar one near the top: *"renderer refused: `0` is not a
valid target identifier"* — 7 sites across **six of seven packages**, the widest spread in the corpus.
It was mine, introduced in R2p.

`self.0` is a TUPLE INDEX, not a field named `0`. The IR had only `Field`, whose lowering parses its
name as an identifier, and `0` is not one — so every declaration reaching a newtype unwrap refused.
`xid::Id`, the package's principal export, was not emitted at all.

**The dangerous part is how it hid.** R2p's compile proof was green, and I reported the newtype index
rule as working. It was green because the affected types stopped being emitted — a rule whose bug
produces a REFUSAL rather than bad output is invisible to a proof that only compiles what survives.
Refusal is this engine's safety mechanism and it doubles as a place for defects to hide: **a compile
proof cannot see a rule that refuses everything it touches.** The refusal histogram can, which is why
ranking causes is not just prioritisation — it is a check.

Fixed by giving the IR a `TupleIndex` node that says what it means. **Translated declarations across
the corpus: 87 → 99.**

That unlocked five further causes, each becoming a rule:

*A returned slice becomes the owned sequence* where the result is one — the sequence twin of the
string rule, and the failure result comes off first, because a fallible function returning a sequence
has two results and one value.

*A slice in POSTFIX position is a place, not a borrowed value.* Three attempts converged here, which
is worth recording. `&x[..].to_vec()` is a reference to a vector, because the method binds to what is
borrowed. Bracketing it into `(&x[..]).to_vec()` compiles and trips the target's own lint for
borrowing what the compiler borrows anyway. The right answer is that a slice under a postfix operator
renders as `x[..]` and autoref does the rest — one place in the lowering, not a special case at each
call site. `lower_postfix_base` already existed for exactly this class and already documented the
`Cast` instance of it; `Slice` simply was not in the list.

*A conversion that changes nothing is not written* — reaching through a newtype already produced the
underlying type, and the source's conversion was a no-op there too.

*The source interface's MESSAGE METHOD maps to the target's display method.* This is the same
correspondence R2p refused to emit as a trait impl, from the other side: the interface is not
implementable because the two traits take different methods, and the CALL is mappable for exactly
that reason — the message is available even though the method is not. Recognised by the receiver's
TYPE, never by the method's name, so a corpus type with its own `Error` method is not rewritten.

*A receiver's type is derived where the front end records none.* Two rules that ask what a receiver IS
were both silently inert on an index into a sequence, which carries no recorded type. The body can
reconstruct the one case the corpus has — an index whose base is a newtype over a sequence has the
sequence's element type — and deliberately no more, because a general expression-typer is the front
end's job.

**One ordering bug, and the rule it produced.** Mapping the message method BEFORE checking the
receiver turned `self.cause.Error()` into `self.cause.to_string()` on an option — a different method
that does not exist either. What the receiver IS has to be settled before what the call BECOMES.

**All seven packages now compile clean under `#![forbid(unsafe_code)]`** with
`clippy-driver --deny=warnings`, including `multierror`, which had emitted nothing all session.

A blind review of the current `uuid` and `ksuid` output ran alongside this and returned DO NOT MERGE
on API grounds, with every data table independently verified correct — 256 `XVALUES` entries, the
ksuid alphabet byte-sorted, `MAX_STRING_ENCODED` decoding to exactly 2^160-1. Its findings are
recorded for the next phase: `i64` where `usize` is required so length constants cannot be used as
array lengths; `Uint128([u64; 2])` reimplementing a type the target has natively; scalar newtypes with
public fields and a derived `Default` that yields invalid variants; and `pub fn error(&self) -> String`
sitting beside a real `Display` impl, which is the method-definition half of the correspondence this
phase mapped for calls.

## R2s — the source's error satisfaction becomes a display impl

The blind review's second-most-decisive tell was `pub fn error(&self) -> String` sitting in the same
file as a real `Display` impl for a different type — Go's `Error() string` transcribed method for
method, beside the template that shows what the target actually wants. R2p had refused to emit the
satisfaction as a trait impl and left the method inherent, which is what produced that pair.

**Both halves are now answered, and they are one answer.** The source's interface is satisfied by one
method returning the message; the target's error trait declares no such method and takes the message
from its display trait. So:

- the CALL maps — `err.Error()` becomes the target's display method (R2r);
- the SATISFACTION becomes a display impl built from that very method, plus the error impl that
  follows from it (here).

What the type had in the source it now has here: it can be printed, boxed as an error, and accepted
by `?`. Refusing the satisfaction was right and still is — the two traits do not take the same
methods — but the method was never the problem.

**Where the target spellings live.** `MessageImpl` is its own IR item rather than something the
transform assembles, because the display trait, the formatter and the write method are the renderer's
to know and are already spelled once for the sentinel enum. That file's own doc says the point of
spelling them in one place is that they cannot diverge; a second spelling would have been the drift
it warns about.

The tail decides how the message is written: a formatting call is handed to the formatter directly,
because writing its result would allocate a string only to copy it — which the target's own lint
objects to and this engine is held to. Anything else is written as the string it is.

**Two consequences, both found by the compile proof rather than reasoned out.**

The import scan asks whether a unit emits a sentinel, to decide whether it needs the formatting
module. A message impl renders the same three names for the same reason, so it belongs in that same
question rather than a second one — `multierror` failed on a missing import until it was.

And the error impl REQUIRES debug, which a type holding the failure type could not earn: the derive
rules block on the interface kind, correctly for interfaces in general, because a bare interface has
no target form. The failure interface is the one that does — a boxed trait object, which is debug —
so the pack now names which derives survive it. One derive, not a lifted block: the boxed form is
neither clonable nor defaultable nor comparable.

That check had to be asked at EVERY node of a type, not just its root. `[]error` is the shape that
found it: the failure sits inside a sequence as often as it stands alone, and a check that looked
only at the outermost type answered for neither. Where the derive survives, the walk does not descend
into the interface at all — its target form is opaque and already known to carry it.

**All seven packages compile clean.** Translated moves 99 → 97, and the direction is right: two
message methods stopped being counted as declarations of their own because they are now the body of
an impl. The golden gained one line, `#[derive(Debug)]` on the hermetic corpus's `Report`, which
holds a boxed failure and had been denied a derive it earns.

## R2t — repairing a mass deletion this lane committed by accident

Preparing to open a pull request surfaced it: the 30 commits of this session showed **669,689
deletions across 2450 files**. That is not what this lane did.

`git show --shortstat` per commit found the whole of it in one place. R2m — the stored-failure
nullability change, six real files — carried a deletion of **655,263 lines**: `ci/`, `intelligence/`,
`specs/`, `tenancy/`, `.grok/`, `tools/`, `infra/`, `flags/`, and the root `Cargo.toml`. R2o added
13,901 more of the same.

**The cause.** Something outside this session deletes the worktree directory
`.claude/worktrees/wise-floating-journal` while work is in progress; it happened three times, and the
first coincided exactly with R2m's `git add -A && git commit`. The deletion was underway DURING the
commit, so `add -A` faithfully staged 2354 files as removed and the commit recorded it. Every gate
after that ran on the surviving tree and passed, because the engine's own crates were never touched.

**The repair.** Restored every deleted path from `0903e9c4f`, the last commit before the damage, plus
`Cargo.lock`. Verified rather than assumed: outside `build/port-engine` the tree is now byte-identical
to that commit except this file's own additions, and inside it the session's work is untouched — one
file added, fifty-one modified, **zero deleted**. The engine builds against the restored workspace.

**Three things worth keeping.**

The root `Cargo.toml` was among the casualties, and its absence is what sent me down a wrong path
earlier: I concluded the branch had no workspace manifest and generated a minimal local one to build
against. The manifest was never absent — it had been destroyed by this commit. A missing file is
evidence of something, and "this branch never had it" was the wrong inference.

`git add -A` is the mechanism that turned an external accident into a committed one. It stages what
it finds, and what it found was a directory being emptied. A lane that stages by explicit path could
not have recorded this.

And nothing in the gate suite could see it. Tests, clippy, the delta, the golden and the compile proof
all pass on a tree with `ci/` and `specs/` deleted, because none of them reads those trees. The check
that caught it was preparing a PR and looking at the diffstat — which is to say, a human-shaped
question about the whole change rather than a machine-shaped question about the part under work.

## R2u — a length is a length wherever it is used, and its neighbours are too

A blind review named the signed length constants twice: once as blocking (`BYTE_LENGTH: i64` cannot
size an array, so `[u8; 20]` hard-codes a literal beside a constant that is decorative) and once as
evidence of translation (`self.0[TIMESTAMP_LENGTH_IN_BYTES as usize..]` — "a Rust author would have
written `usize` and never needed the cast").

The rule that proves a constant is a length already existed and is careful: a constant is one when
every read of it compares it against a length, with rendering, length arithmetic, and being passed AS
a length all counted as neutral. It had two holes and neither was in its reasoning.

**An index or a slice bound is positive evidence, and stronger than a comparison.** A comparison says
the value is measured against a length; an index says the value IS one, because the position it sits
in indexes a sequence and in the target that position has exactly one type. `ksuid`'s timestamp length
is never compared to anything — only sliced with — so the rule had no evidence at all and left it
signed.

**A switch on a length makes every case label a comparison against one.** `switch len(b) { case
byteLength: ... }` is `len(b) == byteLength` written as a table, and the rule saw nothing because a
case label is not a binary node.

**Then the interesting part.** Proving one constant broke the build: `byteLength =
timestampLengthInBytes + payloadLengthInBytes` became `usize + i64`. The source types all three the
same and says no more about them — so nothing on the source side could have decided it.

What supplies the missing constraint is the TARGET. An index type and a signed integer do not add, so
if one operand must be the index type, its partner must be too, and so must the sum. That is a fact
about the language being emitted rather than a guess about the one being read, and it turns the rule
from one-way derivation into UNIFICATION: a declaration and every int constant its value names form a
group, and if any member is proven, all of them are. Run to a fixpoint, growing only.

This is worth stating generally. The engine's rules have mostly been inferences about the SOURCE —
what the Go says, what its conventions mean. This one is an inference from the TARGET's type system
about what the source left underdetermined, and it is sound for a reason the others are not: where
Go's untyped constants make two things interchangeable and Rust's types do not, the target's
constraint is information the source could not have carried.

**And one end of a proven fact must not keep guessing.** With the constant retyped, the index path
still cast it — `TIMESTAMP_LENGTH_IN_BYTES as usize` on something already `usize`, which the target's
own lint rejects. The declaration and the use site now read the same proof, which is the third time
this session that two ends of one decision had to be made to consult one answer.

`ksuid`'s signed integers drop from 12 to 9, the three that are lengths become `usize`, the cast
disappears, and all seven packages still compile clean. What remains signed in `ksuid` are the wire
tag bytes and the epoch — none of which is a length, and the reviewer's separate point that they want
`u8` and a `repr` is a different rule about a different fact.

## R2v — a Result with no failure case is not a faithful port

The blind review flagged it as blocking and as evidence of translation, and the same shape appears in
**every one of the seven corpus packages** — eighteen sites, the widest spread any single cause has
had:

```
pub fn marshal_binary(&self) -> Result<Vec<u8>> { Ok(self.0[..].to_vec()) }
```

An infallible operation with a fallible signature and a boxed error. Every caller writes `?` or an
unwrap on something that has no failure mode, and the crate's own error enum appears in a signature
it can never be constructed for.

**The source's reason for the shape does not survive the port.** `MarshalBinary() ([]byte, error)` has
that signature because an interface requires it, not because the function can fail — and the target
has no such interface to satisfy. Carrying it over is not faithfulness to "this cannot fail"; it is
the source's interface obligation restated in a language that does not have the obligation.

**The proof is the body's, and it is cheap.** A fallible signature whose every return gives the ABSENT
value to the failure result cannot produce a failure. Requires a body and at least one return, for the
same reason `never_absent_pointer` does: a signature-only declaration proves nothing, and a body that
falls off the end says nothing about the results beside the failure.

Bounded to signatures with TWO OR MORE results. A sole failure result is a different question, already
answered by `sole_failure_role` in R2m — dropping it would leave the function returning nothing at all.

**Both ends again.** The signature drops the result and the body drops the operand: each return still
carries the source's trailing `nil`, and with the result gone that operand is a second value nobody
declared. The source states "I did not fail" by returning the absent value; the target states it by
the ABSENCE of a `Result`. Four times now a decision has had to be made to reach the signature and the
body from one answer, and this is the cheapest kind of that bug to create — the two are computed in
different files.

`Result<` disappears entirely from four of the seven emitted packages, because the only ones they had
were the impossible ones. All seven still compile clean.

## R2w — grouped literals, and a finding declined with its reason

**Grouping.** Two blind reviews independently picked the same pair out of the emitted output —
`10000000` and `1000000` on nearby lines, indistinguishable at a glance and different by a factor of
ten. The target groups digits and the source does not, so a value carried across unchanged is correct
and unreadable. Thirteen literals across four packages now group from the right.

Two things the rule has to know. It applies only where the literal STAYS decimal — a value the type
says is a bit pattern is spelled in hexadecimal by `bit_pattern_constants` and grouped by its own
convention, and running both would produce a hex literal grouped in threes. And the threshold is five
digits rather than four, because a four-digit number is read at a glance and the common one is a year:
`2026_` reads worse than `2026`.

It also had to be applied in two places, which is the fifth instance of that shape this session. A
constant declared as a literal carries its value as an attribute; a constant declared as an
EXPRESSION carries its numbers through the body path. `EPOCH * 86400` came out ungrouped beside
neighbours that were grouped, until both paths asked.

**A finding declined, and why.** The same review called `pub struct Uuid(pub [u8; 16])` blocking:
"lets any caller do `u.0[6] = 0xFF` and produce a UUID whose version nibble is garbage". The reasoning
is sound for a crate somebody sits down and writes. It is the wrong change for this engine to make.

The source declares `type UUID [16]byte`. In Go the named type IS the array, and any holder of one can
index and assign to it — `u[6] = 0xFF` is exactly as legal there. The public field is therefore a
FAITHFUL port of the access the source grants. Making it private would add an invariant the source
does not enforce, and the engine would be improving on the program it was given rather than porting
it. That is a decision for whoever owns the ported crate, not for a translator; and an engine that
starts inventing invariants has no principle left for deciding which ones.

Recorded rather than fixed, because the reviewer's premise — that a caller can corrupt the value — is
true of the source too, and a port that hides it would be describing a program that does not exist.
The same answer covers `Ksuid`, `Domain`, `Version` and `Variant`.

The neighbouring finding on the SAME line is different and is not declined: `CompressedSet`'s doc says
"An immutable data type" over a public growable field. That prose is false about the target type
whatever the source allowed, and belongs to the doc rules that already drop a sentence naming
something the crate does not contain — a sentence naming a property the type does not have is the
same defect one step further in. Not yet built; recorded here as the next doc rule.

## R2x — the byte-order package, and four gaps it uncovered

Worked the standing goal's first instruction: build the histogram, rank by packages blocked, read the
top cause and check it is TRUE before acting. The top was `encoding/binary` — **48 call sites across
four of seven packages**, the largest by both measures. The claim held: the package is not in the
snapshot and the target has direct equivalents.

**Why nothing could answer it before.** These are METHODS on a package-level value —
`binary.BigEndian.Uint32(b)` — and the front end deliberately records no callee identity for a method,
because a method is not nameable by package path. So every rule keyed on identity was structurally
unable to see them.

What it does now record is the package's IMPORT PATH, added here. That is the sound key: an import may
be aliased, so `binary.BigEndian` and `bin.BigEndian` are one call written two ways, and keying on the
local name would have been right by convention rather than by construction.

**Built as a tree, not a template.** The pack names the pieces — order suffixes, the integer type each
method yields, the fit and the write methods — and the engine assembles the shape. This engine has
twice been bitten by a rule emitting target TEXT, which is opaque to every rule downstream. A read
becomes `u16::from_be_bytes(x[7..9].try_into().unwrap())`; the unwrap is not an invention, because
the source's own read panics on a short slice.

Verified against upstream rather than assumed: Go's `binary.BigEndian.Uint16(id[7:9])` and the
emitted `u16::from_be_bytes(self.0[7..9].try_into().unwrap())` agree on bounds and order.

**Four gaps it uncovered, each fixed.** This is the pattern the goal warns about — a rule that unlocks
code reveals what was behind it:

*A newtype argument to a MAPPED call reaches through its wrapper.* `len(id)` on a named array becomes
`id.0.len()`: the target's length belongs to the array, and the newtype has none. Only for mapped
callees — a call to this unit's own function takes the newtype, which is what its signature declares.

*A bare return in a function with NAMED results returns those results.* The source binds them at entry
and `return` hands back whatever they hold; the engine emitted `return;` from a function with a result
type.

*A call that WRITES INTO an argument assigns to it.* The source spells the write as a call and the
target as a mutation, so a binding appearing only as such an argument was never observed assigned,
came out immutable, and did not compile. Which callees write is pack data — the same table that maps
them.

*Sequence ownership is per RESULT POSITION.* A function returning `([]byte, uint64)` renders as a
tuple of which only the first element is a sequence the target owns; one answer for the whole
signature said no and left a borrow where the type said otherwise.

**Translated 97 → 100, and `encoding/binary` leaves the histogram entirely.** All seven packages
compile clean. The new top is the failing-return proof at 12 sites across four packages.

An extractor change means every snapshot fixture was regenerated, per the standing verification list.

## R2y — a sentinel whose message is formatted is still a sentinel

Next round of the histogram. Top cause was the failing-return proof — 12 sites, four packages — and
reading an actual site rather than the summary is what found the real problem. `ksuid.Parse` refuses
on `return Nil, errStrSize`, and `errStrSize` is a package-level error value: exactly the thing the
proof already accepts, since a sentinel is a declared constructor's value under a name.

It was not recognised. Two reasons, both in the same place: the pack admitted only the plain
constructor, and the recogniser required the call to carry exactly ONE literal and nothing else.
`fmt.Errorf("Valid KSUIDs are %v bytes", byteLength)` fails both.

**Ten of the corpus's seventeen sentinels are built that way.** Every one was invisible, and every
`return Nil, errStrSize` in four packages refused for want of the proof the sentinel itself would have
supplied.

**The rule, and where its boundary is.** A sentinel built by a FORMATTING constructor is still a
sentinel when every operand beside the template is a package CONSTANT: the message is one string,
fixed before anything runs, and it is simply not spelled as one literal. A NON-CONSTANT operand
disqualifies it, and that boundary is the whole rule — a message depending on a runtime value is a
different string per call, which is a formatted error rather than a sentinel, and the difference is
exactly what a caller comparing against the value relies on.

The verbs are translated through the same table the inline formatting calls use, so a sentinel's
message and a formatted message cannot disagree about what `%v` becomes.

**One shape change, and why the old one could not carry this.** The enum's display bound the message
first and wrote it once — which works only while every message is a literal, because a plain `&str`
and a formatted `String` do not share a type. Each arm now WRITES instead of yielding, so both sit in
one match: `Self::ShortBuffer => f.write_str("…")` beside
`Self::Size => write!(f, "Valid KSUIDs are {} bytes", BYTE_LENGTH)`.

The binding it replaces had a recorded reason — a `match` used as a CALL ARGUMENT formats across ten
lines, which a reviewer read as output nobody had formatted. That reason does not apply here: this
match is the function's tail expression, not an argument, and the formatter leaves it alone. Checked
in the emitted golden rather than assumed.

**Translated 100 → 107**, all seven packages still compiling clean: ksuid 23 → 27, semver 18 → 21.

## R2z — the fourth condition, tested on a real repository for the first time

The standing goal defines done as four things. Three are measured every phase. The fourth — **"it
stays ported": re-porting after upstream moves classifies `Explained`** — had only ever been
exercised by a hermetic fixture pair built to exhibit it. A fixture that exists to demonstrate a
property cannot be evidence for it; that is the engine's own rule about evidence originating outside
itself, applied to the one condition nobody had applied it to.

It is now tested against `cespare/xxhash` at two real commits.

**What had to be built.** `port_snapshot` emitted bytes and no receipt, so there was no way to compare
two ports of one package — the classifier existed and had nothing to classify. The arbitrary-snapshot
path now carries a six-axis receipt, and a `drift <before> <before-pin> <after> <after-pin>` command
ports both and asks the kernel.

The PIN is an argument rather than something read from the snapshot, and that is not a shortcut: the
front end records what the source says, and a Go package does not know its own revision. A caller who
extracted it does. (Recording it at extraction would be better and is what `os/` was faulted for
lacking — noted for later.)

**The result, and why the first answer was not the interesting one.** Across
`998dce2 "Add initial support for custom seeds"` the classification is **`Unchanged`** — and that is
TRUE rather than broken. The commit adds one declaration, `NewWithSeed`, which refuses; the emitted
bytes are byte-identical either side, verified by digest. Upstream moved and the output did not.

So the test was run again across `3686901 "Consolidate array of primes"`, which changes code the
engine does emit:

```
drift=Explained regions_changed=6 axes_moved=[Pin,Snapshot]
```

Six regions changed, accounted for by the two axes that describe upstream identity, with engine,
rulepack, toolchain and formatter all held. That is the fourth condition, on a real repository.

**The counter-tests matter more than the result.** A classifier that says `Explained` for everything
is worthless, so all three discriminating cases were run:

- same snapshot, DIFFERENT pin → `Unchanged`. A moved pin does not manufacture an explanation for a
  change that did not happen.
- EMPTY pin → `IncompleteReceipt`, exit 1. The unusable axis refuses to buy an explanation, which is
  the false-Green defence working on a real package rather than on the fixture written for it.
- planted defect → `Unexplained`, Red. Unchanged.

**What this does not yet prove.** The corpus is still static snapshots for every other purpose, and
nothing runs this in CI. `drift` is a command, not a gate — and the goal's own lesson is that a check
you retype by hand is one you eventually skip. Making it a gate needs the pin recorded at extraction
so a snapshot is self-describing; that is the next piece of this thread rather than a detail.

## R3a — the no-unsafe promise becomes a gate, and a refusal that had gone stale

**The promise was being kept by habit.** Every one of the engine's fourteen crates carries
`#![forbid(unsafe_code)]`, and the only occurrence of the word in engine sources is the Rust keyword
table in `naming.rs` — checked, not assumed. But the EMITTED output was a different story: the
engine's own compile proof compiled it without that attribute, and the attribute appeared only in the
ad-hoc harness I typed by hand each phase.

So the guarantee held because somebody kept remembering it, which is the same class of failure the
drift command has: a check you retype is one you eventually skip. The compile proof now prepends
`#![forbid(unsafe_code)]` to the assembled tree. `forbid` cannot be lifted by an inner `allow`, so one
unsafe block anywhere in the emitted output fails the gate.

It is not hypothetical. R2j refuses a type whose meaning is the source's memory layout precisely
BECAUSE the target denies unsafe — that refusal and this attribute are one decision, and only one of
them was enforced.

**A refusal that had gone stale.** The histogram's top cause — 12 sites, four packages — was the
engine's own declared reason for not emitting an error-interface satisfaction, and it ended: *"The
METHOD is not lost: it stays in the type's inherent block under its own name."* That stopped being
true at R2s, which turned the method into a display impl plus the error impl that follows from it.

The reason now says what happens: the satisfaction is not emitted in the SHAPE the source wrote it,
the conformance is not lost, and the entry marks a translation rather than a gap. It stays reported,
because a reader comparing the two files should be able to see that `Error() string` has no
counterpart under that name and where it went instead.

This is the second time a refusal has outlived what it described. The first (R2h) was the largest
cause in the corpus and pointed at the wrong component entirely. The lesson generalises past
documentation: **a reason is code that nobody compiles.** Every rule that changes behaviour has to be
asked whether some other rule's stated reason was describing the behaviour it just changed.

## R3b — the corpus was measuring the wrong thing, and the front end cannot load the right thing

Two claims were put to this engine: that it must survive JSON/YAML tag reflection, interface-based
composition and concurrent mutex models; and that "best practice" Rust is unreachable without an
explicit account of how ownership maps to the borrow checker. Both are correct, and the measurements
below are worse than the coverage number suggested.

**The ratchet corpus does not contain the destination's problems.** Counted, not assumed:

| | Go files | struct tags | interfaces | mutexes | channels |
|---|---|---|---|---|---|
| siderolabs/talos | 2077 | **5674** | **358** | 98 | 514 |
| k8s.io/apimachinery | 349 | 782 | 104 | 23 | 160 |
| the whole 7-package corpus | ~60 | **3** | **0** | 5 | 2 |

Zero declared interfaces. Three struct tags. `apimachinery` alone calls `reflect.DeepEqual` 208 times
and `reflect.TypeOf` 108. The corpus is near-pure functions over bytes, which is exactly why per-site
ownership rules and no lifetime inference have carried it this far. **23% on this corpus is not 23% of
the problem.**

**Measured against the three domains, on real self-contained packages:**

| package | domain | coverage |
|---|---|---|
| `go-yaml/yaml` | YAML tags, 218 reflect calls | 45.7% |
| `mitchellh/mapstructure` | 52 tags, 427 reflect calls | **6.1%** |
| `patrickmn/go-cache` | 8 mutexes | **0.0%** |

The yaml figure flatters: that package is largely a transliterated C state machine, so what ported is
the mechanical parser core rather than the tag layer. On the concurrency package the engine emits
**nothing at all**.

**And the front end cannot load a repository with third-party dependencies.** `extract.go` resolves
imports with `importer.ForCompiler(fset, "source", nil)` plus an intra-corpus importer — so a package
type-checks only if every non-stdlib import it has lives inside the corpus directory. Every one of
the seven corpus packages is dependency-free. That is not a coincidence; it is a selection effect,
and a sharper version of the goal's own warning: the SCRATCH corpus contains only what the front end
can load, which is why it looks like a set of dependency-free byte utilities.

Confirmed by trying: `gjson`, `chi`, `memberlist` and `apimachinery` all fail to extract, before any
translation question is reached. Vendoring does not help, because the importer never consults the
module graph. The fix is `golang.org/x/tools/go/packages`, which resolves modules and vendor
directories properly; that is a front-end change with a full fixture regeneration behind it.

**Why per-site ownership cannot reach the three domains.** Each one is a whole-program ownership
question, not a local one:

- **Tags.** `json:"name,omitempty"` is a serialization contract, and `omitempty` on a non-pointer
  field decides whether the target field is `Option<T>` or carries a default. The tag constrains the
  TYPE, which constrains every construction of it.
- **Interfaces.** Go's interface is always dynamic. The target's choice between `&dyn Trait`,
  `Box<dyn Trait>` and a generic parameter depends on whether values are stored heterogeneously
  anywhere in the program — a question no single site can answer.
- **Mutexes.** Go's mutex sits BESIDE what it guards; the target's `Mutex<T>` OWNS it. Deciding which
  fields a mutex guards means analysing what is touched between lock and unlock. The pack currently
  refuses `sync.Mutex` and says why, which is honest and is also why `go-cache` is at zero.

This is the R2n finding arriving from a second direction: the engine decides ownership per site from
local facts, and all three domains need a solver over the whole program. C2Rust's permission lattice
and Polonius's Datalog borrow model are the shapes to copy.

**What this means for the four completion conditions.** They are measured against a corpus that
cannot exercise the destination, so meeting all four today would not mean the engine can port Talos.
The corpus has to become representative before the conditions mean what they say — and the front end
has to be able to load a real repository before the corpus can become representative. That ordering
is the plan: `go/packages` first, then the phased oracle repos, then the ownership solver.

## R3c — the front end can load a real repository

R3b found that the front end resolved imports with the standard-library importer plus an intra-corpus
one, so a package type-checked only if every non-stdlib import lived inside the corpus. Every corpus
package was dependency-free, which looked like a corpus choice and was actually a limit of the loader.

**Fixed without adding a dependency.** The extractor's own module doc says it is deliberately
dependency-free and names `golang.org/x/tools/go/packages` as the thing it never uses; licensing
policy fail-closes on any new extractor dependency until provenance is recorded. So the Go tool is
ASKED instead of reimplemented — `go list -deps -json ./...` reports every non-standard package's
import path and source directory, and those are merged UNDER the corpus's own map. The tool already
understands the module graph, vendoring, workspaces and replace directives, and reimplementing that is
how a front end acquires a subtly different idea of what a package is than the compiler has.

The walk also now skips what the Go tool itself skips — `vendor`, `testdata`, and any directory
beginning with `_` or `.`. `chi` failed on an `_examples` program whose imports are not the library's.

**Result.** `gjson` and `chi` extract for the first time; all seven existing corpus packages still do.
First measurement against them:

| package | phase | declarations | translated | coverage |
|---|---|---|---|---|
| `tidwall/gjson` | core language | 124 | 15 | 12.1% |
| `go-chi/chi` | interfaces, closures | 189 | 33 | 17.5% |

The top blocker across both is `FuncLit` — 30 sites. Go's closure has no translation at all, which is
the honest reason a middleware library ports at 17%: chi IS closures. `TypeAssertExpr`, `IncDecStmt`
and partial-clause `for` follow, none of which the old corpus contained.

`memberlist` still does not load, and for a different reason worth keeping separate: it type-checks
its way into `golang.org/x/sys/unix`, a transitive syscall dependency that is hard to type-check
standalone at any release. That is a dependency-quality problem rather than a loader one.

**On the corpus being universal rather than destination-shaped.** R3b framed the gap as the corpus
being unrepresentative of k8s and Talos. That framing was wrong and is corrected here: aiming at those
would make this a k8s porter rather than a Go porter. The criterion is coverage of the LANGUAGE
surface — closures, interfaces, type assertions, channels, select, tags, reflection — ranked by how
common each is across the ecosystem, with k8s and Talos as consumers of the result rather than its
specification.

## R3d — the Go release was an input that changed nothing observable

Asked directly whether the engine checks every Go release from 1.21 to latest. It did not, and the
answer had two parts, of which the second is the dangerous one.

**We type-check at ONE release.** `defaultRelease = 21`, overridable by `-go-release`, and no workflow
runs a matrix. That is a coverage gap.

**And the release was invisible.** Proven rather than reasoned: extracting `xxhash` at go1.21 and at
go1.24 produced **byte-identical snapshots and the same digest**. Go 1.22 rescoped the loop variable —
same syntax, different program — so the engine could have emitted a different program with every
receipt axis holding. That is precisely the false Green the six-axis receipt exists to refuse, hiding
in the one input nobody had recorded.

The extractor's own `buildConfig` doc had already stated the invariant this violated: *"Every field is
an input, so two configurations of one corpus are two snapshots rather than one snapshot with a hidden
variable."* The release was the hidden variable.

**Fixed on both sides.** The snapshot now carries `build_config` (`linux/amd64 go1.24`, canonicalised
by the configuration's own `describe`), and the preimage covers it — mirrored byte-for-byte in the Go
encoder and the Rust one, as every other field is. The model seam gained a DEFAULTED accessor so a
snapshot written before the field still admits: it has nothing to say about its configuration rather
than a false claim about it.

**The matrix, measured.** Every release from 1.21 through 1.26 now yields a distinct digest for one
corpus. 1.27 fails, and correctly: the installed toolchain is go1.26.6 and refuses to type-check
against a release it does not know. So the engine can check any release its own toolchain understands,
and the ceiling moves when the toolchain does.

**Two recorded lessons re-learned, both cheaply this time because they were written down.**

The fixture regeneration lost a hand edit. R2m added a `Check` method to
`fixture-snapshot-unproven-v1.json` by editing the GENERATED JSON, and regenerating from the corpus
wiped it. The right home was always the Go source; it is now in `corpus-unproven/stored/stored.go`,
where regeneration reproduces it. **Editing a generated artifact is a change that survives only until
the next generation.**

And the drift pair broke exactly as the drain says it broke once before: regenerated under the module
id `...-upstream` instead of the plain one, the plan stopped selecting the unit and the delta went
`Unchanged` where the test demands `Explained`. The entry describing that was read and applied rather
than rediscovered — which is what the record is for.

## R3e — `continue` is a decision about the enclosing loop, not about `continue`

The front end refused every `continue` in the language, with this reason:

> `break` maps directly. `continue` does NOT, because a three-clause loop lowers to a `while` whose
> post-statement a `continue` would skip — a different program.

The hazard is real and the refusal was in the wrong place. It holds for exactly one loop shape: one
whose post-statement the target has to spell as the last statement of the body. A `continue` inside
`for cond {}`, inside `for range xs {}`, or inside a three-clause loop that spends its post-statement
building a range, skips nothing. The extractor cannot tell those apart without deciding which target
loop the enclosing `for` becomes — which is the transform's decision, not the extractor's.

So the fact moved to where the decision is. The front end now RECORDS `continue` (and `incdec`, which
was reaching the transform as an `unsupported` node identified by a string label); `body_loops`
refuses by name when, and only when, a post-statement would be jumped over:

> a `for` with a post-statement contains a `continue`, and the target's `continue` jumps to the test
> rather than to the post-statement — spelling the post-statement at the end of the body would skip
> it on exactly those paths

That unlocked the two loop shapes that had none. `for ; cond; post {}` — 50 sites in gjson alone,
because a hand-optimised parser reuses one cursor across loops — becomes `while cond { body; post }`.
`for init; cond; post {}` that is not the canonical ascending-integer form falls back to the same
form wrapped in a block: `{ init; while cond { body; post } }`. The block is not decoration. Go scopes
the init clause to the loop; emitting the binding as a sibling of the loop widens that scope to the
rest of the enclosing body, where it can shadow a name the source left readable there.

`counted_range` also stopped reading `ATTR_SOURCE_NODE == "IncDecStmt"` to recognise its post clause.
That was a string label from the front end standing in for a structure; now that the increment is a
real node kind the check reads the kind and the operator, and a `--` with an ascending `<` test no
longer looks like an increment.

Measured: gjson 12.1% → 16.1%. No package regressed.

### The blank identifier is a hole, not a name

Found by probe, not by the corpus. `_ = len(b)` emitted `item = b.len() as i64;` — an undeclared
name on the left of an assignment. `to_snake_case` has a fallback that names an empty result `item`,
and the blank reached it as if it were an identifier.

`_ = expr` EVALUATES AND DISCARDS; there is no place being assigned. It is now `RustStmt::Discard`,
which renders `let _ = expr;`. Not a `Let` whose name is `_` — the first attempt was exactly that,
and the renderer refused it correctly, because the target's blank is a PATTERN and `parse_ident`
does not accept a keyword. That refusal is the closed vocabulary doing its job.

### Two things that were tracked and should not have been

`gosrc/extractor/extractor` — a 7.4 MB arm64 Mach-O binary, dropped there by `go build ./...` and
committed in R3c and R3d without anyone noticing. It is derived from the `.go` files beside it.
Untracked, with a `.gitignore` that says why.

`regen_fixtures.sh` lived only in scratch and had the worktree path hardcoded to a DIFFERENT lane, so
running it regenerated the wrong tree's fixtures — and it omitted two of the eleven corpora entirely.
It is now `build/port-engine/regen-fixtures.sh`, path-relative, complete, and carrying the two
lessons that have each cost a phase: a hand edit to a generated fixture is lost at the next
regeneration, and the drift pair must share ONE module id or the classification comes back
`Unchanged` instead of `Explained` — a green result for a broken test.

This mattered immediately: the committed fixtures were produced by the older extractor and still
recorded `IncDecStmt` as `unsupported`, so `port-go` refused a region the transform now handles. That
failure reads as a transform regression and is not one.

### DOCTRINE.md

The engine had no doctrine document — only this file, which is append-only chronology and answers
"what happened", not "what is still true". Architecture questions were being re-litigated from
transcripts. `build/port-engine/DOCTRINE.md` now holds the ownership mapping law, the `native-strict`
output profile and why the compatibility lane is forbidden, the constructs refused on purpose, the
measured open gaps, and the external design proposals already rejected with their reasons.

Its §9 records the largest one, found by probe:

    pub fn mutate(s: &mut [i64]) { s[0] = 9; }   // signature correct
    pub fn callee() -> i64 {
        let a = vec![1, 2, 3];
        mutate(a);                                // NOT `&mut a`, and `a` is not `mut`
        a[0]
    }
    port=ok translated=2 refused=0

Ownership is decided per declaration and never reconciled at call sites, and the engine reports
success. Relatedly there is no aliasing model at all: Go's `b := a` shares backing storage and the
engine emits a move — which fails safe only because rustc rejects the result. The safety is
accidental, not designed.

## R3f — a pack form is a tree, not text, in the two places it still was not

Ranked by PACKAGES blocked rather than by count in one, the top cause in the corpus was not closures.
It was this, 34 sites across 8 packages:

> an argument to `len` is a compound expression, and the pack answers for that call with a TEXT
> template — substituting one would need parentheses the template cannot ask for

The refusal was accurate, and it named a principle the engine had already adopted and then failed to
finish applying. `body_mapped::structured_method` builds a tree for a form shaped `{0}.method(..)`.
`len`'s form is `{0}.len() as i64`, which is not that shape — so it fell through to text
substitution, and text substitution has no way to bracket a compound operand.

A trailing conversion is a WRAPPER around the shape underneath it, not a reason to give up on
building one. `structured_form` now peels a trailing cast and builds the method call inside it, using
the same guard `body_call::structured` already used on rendered text: the conversion target must be
an identifier, so `{0}.rotate_left({1} as u32)` — whose ` as ` sits inside — is left alone.

`make` had the identical defect in `body_alloc`, in two arms. Both now recognise the pack's form
structurally instead of substituting into it:

- `Vec::with_capacity({0})` → `structured_call`, a path callee and positional arguments.
- `vec![{1}; {0}]` → `structured_repeat`, and a new `RustExpr::VecRepeat`.

`structured_repeat` reads the placeholder INDICES out of the form rather than assuming them. This is
the one form in the pack whose operands are not in call order — the source's `make([]T, n)` gives the
count first and the target's literal gives the value first — and reading the indices means a pack
that writes the form the other way round gets the other tree instead of silently getting this one.

Both cause classes are now absent from the corpus: zero text-template refusals across all twelve
packages.

### What this measured about the method, which matters more than the rule

Clearing the largest cause across 8 packages moved total translated declarations by ONE.

The survey reports a declaration's FIRST refusal. Removing that cause does not translate the
declaration; it surfaces the declaration's next blocker. Coverage moves only where the cleared cause
was the LAST one. So the histogram ranks CAUSES correctly and says nothing about how close any
declaration is to translating — a cause with 34 sites across 8 packages can be worth one declaration,
and a cause with 6 sites can be worth twenty.

The histogram stays the right tool for choosing which rule to write, because a cause blocking 8
packages is a property of Go and a cause blocking one is a property of a package. But "coverage did
not move" is not evidence a rule was wrong, and neither is "coverage moved" evidence it was right.
The check that a rule did what it claimed is that its cause is GONE from the histogram — which is
what was verified here, and what R2p failed to verify when a broken rule looked green because the
declarations it damaged had stopped being emitted.

## R3g — `translated` was never the same claim as `compiles`, and nothing was checking

`survey` reports how many declarations translated. The engine's compile proof runs on the HERMETIC
fixture corpus. Nothing had ever compiled the output for a real repository, so every coverage number
in this file up to here was a claim about the engine's own confidence rather than about Rust.

Compiled the emitted output for nine real packages. Six compile. Three do not: gjson (60 errors),
chi (14), ksuid (15).

`build/port-engine/compile-corpus.sh` makes it repeatable. A rule is not finished when its refusal
leaves the histogram; it is finished when the output still compiles.

(One false reading on the way: `rustc -o /dev/null` fails with "couldn't create a temp dir", because
rustc writes its temporaries beside the output path. That looks exactly like a compile error and is
not one — it made six packages appear broken. The script writes into a real directory and says so.)

### A switch with no tag is a condition chain

The refusal said it "needs a rule for how an empty case list becomes an `else`". The source compares
each case expression against `true` and takes the FIRST that holds, which is what a chain of
`else if` does. Empty case list is `default`.

Order is the whole content of the construct, so it is preserved — with the one exception the source
permits and the target does not. Go allows `default` to be written ANYWHERE among the cases and still
be the fallback; the target's `else` can only be last. The default is lifted out and emitted last.
Leaving it in place would make every case after it unreachable.

Several expressions in one case hold when ANY does, so they join with `||` and the body is emitted
once, which is what the source does.

+5 declarations; ksuid 24.1% → 27.6%. The emitted chains are the code a person would write:

    fn base62_value(digit: u8) -> u8 {
        if digit >= b'0' && digit <= b'9' {
            digit.wrapping_sub(b'0')
        } else if digit >= b'A' && digit <= b'Z' {
            OFFSET_UPPERCASE.wrapping_add(digit.wrapping_sub(b'A'))
        } else {
            OFFSET_LOWERCASE.wrapping_add(digit.wrapping_sub(b'a'))
        }
    }

### A rune literal resolves to a TYPE before it resolves to a spelling

Those `b'0'`s are the second rule, and without it that function did not compile.

The source's `'0'` is an untyped constant and its meaning comes from context: in `b >= '0'` where
`b` is a byte it IS a byte, and in `r == '0'` where `r` is a rune it is a character. The target
spells those `b'0'` and `'0'` and has no untyped constant to defer the choice to.

The literal passed through as source text. The comment on that path claimed the divergent lexical
forms — "a rune literal, an imaginary literal" — would fail the parse, "which is the correct
outcome". For a rune it does not: `'0'` is well-formed Rust of the WRONG TYPE. It reached `rustc` as
eleven type errors in one package, which is the outcome the pass-through was documented not to have.

The front end now records the type the Go type-checker resolved the literal to, and the spelling is
chosen from the type the PACK maps it to — `u8` gets a byte literal, `char` gets a character literal,
anything else refuses by name. A rune outside ASCII, or one written with a `\u` escape, has no byte
form and refuses rather than being truncated.

### The target's newtype carries none of its underlying type's operators

Caught by the new compile check, and it is the pattern worth naming: the tagless-switch rule made
`Version::String` translate for the first time, and its output did not compile. Coverage went up and
correctness went down in the same commit.

    if self > 15 { return format!("BAD_VERSION_{}", self); }

`Version` is `type Version byte`. In the source a defined type and its underlying are one thing, so
`v > 15` compares bytes and `%d` prints one. In the target the newtype is a struct: it has no `>`
and no `Display`.

`unwrapped_base` already existed and already handled the receiver — it was simply never applied to
an operator's operands or to a format call's arguments. Both now reach through. EQUALITY is excluded
deliberately: the emitted newtype derives `PartialEq`, so `a == b` already means what the source
meant, and reaching through there would churn every existing comparison to say the same thing in
more characters.

gjson 60 → 20 errors, ksuid 15 → 8, uuid back to compiling.

## R3h — the snapshot was a property of the machine that took it

Found by a subagent reading the extractor for an unrelated reason, then verified here rather than
taken on trust.

`buildConfig.context()` copies `build.Default` and overrides GOOS, GOARCH, build tags, cgo, compiler
and release tags. It does not override `ToolTags`, which `go/build` computes AT INIT FROM THE HOST.
On this machine that is:

    [goexperiment.regabiwrappers goexperiment.regabiargs goexperiment.greenteagc
     goexperiment.randomizedheapbase64 arm64.v8.0]

Those are not decoration. `arm64.v8.0` and `amd64.v1` each select different files in the standard
library and in `x/sys/cpu`, so the same commit at the same declared `linux/amd64` extracted to a
DIFFERENT snapshot depending on the architecture of the machine running the extractor. Two
identities for one input, which is exactly what the receipt exists to prevent — the same failure as
the Go release in R3d, one field away from it.

The function's own doc had already stated the principle it was violating: excluding cgo
deterministically "is a refusal the snapshot can state; excluding it because the host happened to
lack a C compiler is not."

Tool tags are now DECLARED, from the architecture the caller asked for, through a closed table; an
architecture with no entry gets none rather than the host's. They are stated in `build_config`, so
they are covered by the preimage and therefore by `snapshot_digest`:

    linux/amd64   go1.21 tooltags=amd64.v1
    linux/arm64   go1.21 tooltags=arm64.v8.0
    linux/riscv64 go1.21 tooltags=riscv64.rva20u64   — three distinct digests

The `goexperiment.*` tags are deliberately NOT reproduced. They are properties of the toolchain that
compiled the extractor rather than of the configuration being described, and a source file selected
because of how this binary was built is not a file the snapshot can account for.

## R3i — the arm order the source permits and the target does not, in the case R3g missed

Eleven subagents diagnosed the corpus and reviewed the output. Both reviewers — one blind, one told
it was ported — returned DO_NOT_MERGE with twenty findings each, and converged independently on the
same defects. The first one is the engine's whole law:

    switch data[i] {
    default:
        return i, false
    case '"', '\\', '/', 'b', 'f', 'n', 'r', 't':
    case 'u':

`gjson.validstring`, and the `default` is written FIRST. The source takes the first case that
MATCHES and `default` is the fallback wherever it sits. The target takes the first arm in ORDER, so
the emitted `match` put the wildcard in front and made every real arm dead — and the arms it
shadowed were the ones that ACCEPT a valid escape. The emitted function rejected every string the
source accepts, and it compiled while doing it.

R3g fixed exactly this for the switch with NO tag and did not fix it for the tagged one. That is the
failure mode already at the top of the standing goal's list: *both ends of one decision must read
ONE answer*. Two files, one rule, and only one of them learned it. The wildcard is now lifted to
last in both, which is safe precisely because it matches everything: no arm it passes over could
have been reached through it, and every arm it passed over was unreachable while it sat in front.

### A source string is BYTES, and the target's is not

The largest remaining compile error in gjson, 13 sites: `the type str cannot be indexed by usize`.

The source's string is a sequence of bytes that may hold anything, and `s[i]` yields one of them.
The target's is guaranteed UTF-8 and is not indexable at all. `s[i]` now goes through `as_bytes()`,
which is the same read of the same byte and cannot fail where the source's could not.

SLICING a string now REFUSES, and this is the more important half. The engine was emitting
`&s[a..b]`, which PANICS when either bound falls inside a multi-byte character — and the source
cannot fail there at all. That is a program which agrees with the source on every ASCII input and
aborts on the first one that is not: invisible until production, and precisely the defect this
engine exists to prevent. `&s.as_bytes()[a..b]` is faithful and is deliberately NOT substituted,
because it yields a byte slice where the source yielded a string, and every destination expecting a
string would then be wrong. That is a decision about the ported program's STRING TYPE, not a
rewrite of one expression, and it is named as such in the refusal.

gjson: 20 compile errors → 6, and 20 translated declarations → 18. Coverage went DOWN and the rule
is correct. Refusing is not losing.

### Nine of chi's fourteen errors were the measuring instrument

`port` emits one module per source package, with a `// <name>.rs` banner. `compile-corpus.sh`
concatenated them into a single file, so `ContextKey` — which exists in both `chi` and
`chi/middleware` — collided with itself, along with eight conflicting impls. Reading those as an
engine defect was wrong; the engine had emitted them apart. The script now writes one file per
module and a crate root that declares them, and chi drops to 5.

Worth stating plainly because it happened twice in one day: a compile check is only evidence about
the engine after it is evidence about itself. The first version of this script also reported six
packages broken because `rustc -o /dev/null` cannot create its temporary directory.

## R3j — the ranking instrument was hiding the largest cause in the corpus

`survey_cause.rs` exists to answer one question, and its own doc states the rule:

> the reason is carried verbatim and the declaration's own name is not part of it: a cause that
> carried the site would count once per site and rank a rule nobody needs above one that blocks six
> packages.

`UndecidedForm` had no arm. It fell through to `other => other.to_string()`, which renders the whole
message INCLUDING the interpolated declaration name — so one undecided form read as eighteen causes
of one site each, and sorting by count put every one of them below the noise floor.

With the arm added, package-scope variable state is **57 sites across the corpus** in three rows:

    6pkg 19  `written_package_var` is a form the pack has not decided
    5pkg 19  `init_written_package_var` is a form the pack has not decided
    5pkg 19  `exported_package_var` is a form the pack has not decided

That is larger than `FuncLit`, which has been the headline blocker for three phases. In `google/uuid`
alone, package-scope mutable state and its cascades account for 23 of 98 refusals, and everything in
`node.go`, `time.go`, `version1.go`, `version4.go` and `version7.go` sits downstream of it.

Every rule chosen from this histogram since it was built was chosen with this cause invisible. The
rules were not wrong; the ordering was.

### A refusal that misdescribes what is missing is worse than no refusal

`google/uuid`'s `Nil` is declared `Nil UUID` with no initialiser and is never assigned anywhere in
the package — verified by grep, not assumed. The engine refused it as a `written_package_var`.

The DECISION is right and was made for a recorded reason: an exported package variable is part of
the source's API, a consumer writes `pkg.Setting = false` because the documentation says to, and a
reviewer reading a real ported package caught the engine deleting exactly that feature by making one
a constant. So the rule asks whether anyone COULD write it rather than whether this package does.

What was wrong is that the refusal reused the WRITTEN form's id, so its first clause told the reader
that a variable nothing assigns to is assigned to. It is now `exported_package_var`, with its own
reason naming the actual missing decision — including the condition under which it becomes
decidable, which is a port that covers every importer.

Found by a subagent, verified here before being acted on. Both halves of its report were useful and
only one of them was right: the rule is not a false refusal, the NAME on it was.

## R3k — four of the five goal repositories compile

`uuid`, `gjson`, `chi` and `multierror` now emit Rust that `rustc` accepts. `memberlist` still does
not extract. Eight of the nine measured packages compile; only `ksuid` remains, with 8 errors.

Five rules, each found by compiling rather than by the histogram, and each universal.

### `var x T` INITIALISES

    let mut i: i64;
    if i == s.len() as i64 {          // E0381: used binding `i` isn't initialized

The front end recorded a `var` with no initialiser as a binding with no value, and `RustStmt::Let`
documented that as "a binding the body fills in later, which the source spells as a `var` with no
initializer". That is what the TARGET's bare `let x: T;` means and it is not what the source wrote:
Go guarantees the zero value, and reads it on every path. The target then refuses to read the name
on any path that does not assign first.

It is now recorded the same way a composite literal's omitted field already was — a `zero` node
carrying the type — so one rule answers both.

### An untyped constant resolves at the OUTERMOST node of its expression

    if f < -9_007_199_254_740_991 || f > 9_007_199_254_740_991.0

The positive literal took the float spelling and the negated one did not. `go/types` records the
literal inside `-9007199254740991` as `untyped int` and puts the conversion to the context's type on
the UNARY, so a translator reading only the literal sees a type it cannot map. The unary's resolved
type is now pushed down to its operand, and `isUntyped` refuses to treat `untyped ...` as a target
type anywhere — it is the source saying "this takes its type from where it is used", and anything
that spells it is guessing.

### Concatenation onto a literal is a formatting call

    "chi context value " + &self.name          // E0369

The target's `+` on strings takes an OWNED left operand and reuses its allocation. A literal is a
borrowed `&'static str` and owns nothing, so this is not an operation the target has. The source has
one string type and cannot express the difference. It becomes `format!("{}{}", ..)`, which allocates
once for the result — which is what the source's concatenation does. `String::from("a") + &b`
allocates twice and reads like a workaround because it is one.

### A sequence constant is an ARRAY, and this was a hole in an "exact" test

    const DEFAULT_COMPRESSIBLE_CONTENT_TYPES: Vec<String> = vec![ "text/html", .. ];

Two defects in one line, and the second is the interesting one. `prove_constant` documents itself as
CLOSED and exact, admitting "a COMPOSITE literal ... because a struct, tuple and array constructor
are all const in the target". True of all three — and the source's slice was becoming none of them.
The pack maps it to the growable sequence, which is right for a value a body builds and allocates,
so it cannot stand in a constant.

The length is known: it is how many elements the source wrote. So a package-scope sequence of
literals now declares `[&str; 15]` and emits an array literal, built from the SAME element
translation a body uses so the two cannot differ. The elements are borrowed strings for exactly the
reason a scalar string constant already was — a literal is static storage, and owning it would
allocate per element where nothing may allocate at all.

`RustType::Array` and `RustExpr::ArrayLiteral` are separate from the growable forms on purpose: one
is a constant expression in the target and the other is not, and a shape that blurred them would
produce this same error somewhere else.

## R3l — memberlist was three stacked loader defects, and two of them were R3d again

`hashicorp/memberlist` — the Phase 3 "ultimate proof" repository — did not extract, and the drain
recorded the cause as "type-checks into `golang.org/x/sys/unix`, a transitive syscall dependency".
That was where the first defect surfaced, not what it was. Diagnosed by a subagent and verified
here; all three fixed, and memberlist now extracts **186 declarations, 78 translated (41.9%)**.

### Defect 1 — one release applied to other people's modules

    conf := types.Config{Importer: c, GoVersion: c.cfg.goVersion()}

Every package the importer resolved — corpus, vendored dependency, anything — was type-checked at
the CORPUS's declared release. A package's language version comes from the `go` directive of the
module that OWNS it. `x/sys` declares `go 1.25` and uses `for range n` over an int, so checking it at
go1.21 fails.

Failing false is the visible half. The other direction is the serious one: a dependency whose module
declares go1.22, checked at go1.21, gets the pre-1.22 loop-variable scoping — the same syntax and a
different program. That is R3d, reintroduced once per dependency, three phases after R3d was fixed
for the corpus.

### Defect 2 — the standard library was resolved for the host

    fallback: importer.ForCompiler(fset, "source", nil)

`go/importer`'s source compiler is `srcimporter` over `build.Default` and takes no build context. So
the corpus and its dependencies were file-selected for the DECLARED `linux/amd64` while the standard
library they import was type-checked for the host — darwin/arm64 here — and `x/sys/unix` failed on
`syscall.Setresuid`, which exists only on Linux. Another instance of R3h: the declared configuration
is what makes a snapshot independent of the machine, and half the type-check was ignoring it.

Fixed by reading what the COMPILER reads. `go list -deps -json -export` produces export data for the
declared target, at the release each module declares, from the real toolchain; the importer is
`ForCompiler(fset, "gc", lookup)` over those files. No new dependency — the ban on adding
`golang.org/x/tools` to this module holds.

### Defect 3 — the resolver was asked the host's question

`go list` ran with no environment, so the DEPENDENCY SET was the host's. `miekg/dns` reaches
`x/net/ipv4` on linux and not on darwin, so the map handed to the importer was missing packages the
declared build imports, and the failure read as "cannot find package in GOROOT". It now runs with
the declared GOOS, GOARCH and `CGO_ENABLED=0`, which is the same configuration the walk selects
files with.

### Two things the fix exposed

**`unsafe` is not a package.** It is built into the type-checker, and `types.Unsafe` is the only
value of it that compares equal to itself. The resolver lists it like any other import, so it was
being type-checked from GOROOT's source — producing a SECOND `unsafe` whose `Pointer` is a different
type from the real one, after which `unsafe.Pointer(&sliceHeader{..})` stopped being legal. That is
correct Go failing to type-check, in `xxhash`, which had extracted fine for twenty phases.

**The corpus's own release is a ceiling, and it is now refused BY NAME.** memberlist declares
`go 1.25`; at the default 21 it used to fail as a syntax error six imports deep inside a vendored
file, naming neither the corpus, nor the release, nor the mismatch. It now says so:

    corpus module github.com/hashicorp/memberlist declares go1.25 and extraction is configured for
    go1.21: the configured release is a ceiling, and a corpus is not silently checked below the
    release its own module requires

That refusal immediately caught `gjson` and `chi`, which declare `go 1.23` and had been extracted at
go1.21 for two phases — every measurement of them before this was taken at the wrong release.

### The weaker importer is recorded rather than hidden

Not every corpus can be built. The fixtures in this repository deliberately cannot, so `go list
-export` produces nothing for them and source is the only answer. Source is a WEAKER answer — it
resolves the standard library for the host — so taking it is recorded in `build_config`, which the
preimage covers:

    linux/amd64 go1.21 tooltags=amd64.v1 imports=source   — the fixtures
    linux/amd64 go1.25 tooltags=amd64.v1                  — memberlist

A degradation that changes what was checked has to change the identity of what was produced, or the
receipt is certifying two different things under one digest.

## R3m — all five goal repositories compile

`uuid`, `gjson`, `chi`, `go-multierror` and `memberlist` now emit Rust that `rustc` accepts under
`#![forbid(unsafe_code)]`. Three rules, all from the compile check, all universal.

### A bare integer literal has no type until something gives it one

    2.wrapping_add(label.len() as i64)      // E0689: ambiguous numeric type

Every other position in the target infers a literal's type from its neighbours. A method call is not
one of those positions — the receiver has to be typed before the method can be found. The type comes
from the OPERATION, which is the type the source gave the whole expression, so the literal is spelled
at the type it already had rather than at one chosen here.

### A scrutinee and its patterns must be the same type, and the target breaks that two ways

The source guarantees they agree: a switch compares its tag against each case with `==`, and it would
not compile if they differed. The target has two ways to lose that, and they need opposite fixes.

A NEWTYPE scrutinee with literal cases:

    switch vsn { case 0: ... }              // `type encryptionVersion uint8`

There the defined type and its underlying are one thing, so this compares numbers. Here the newtype
is a struct and `0` is not one. The SCRUTINEE reaches through the wrapper, because a pattern has no
field access and cannot.

A BORROWED RECEIVER with constant cases:

    func (t NodeStateType) metricsString() string { switch t { case StateAlive: ... } }

A method on a value receiver still takes `&self` here, so `match self` has type `&T` while every
constant the source names has type `T`. The scrutinee is DEREFERENCED — the opposite direction — and
not the patterns wrapped, because `&CONST` is not a pattern the target accepts either. Nothing
moves: a constant pattern binds nothing, so the place is only read.

`RustExpr::Deref` is its own node rather than a `UnaryOp`. Those are arithmetic and logical
operators on a value; this reaches a place through a reference, and neither should have to carry the
other's precedence rule.

### Where the corpus stands

    uuid         125 decls   29 translated  23.2%   compiles
    gjson        124          18            14.5%   compiles
    chi          189          33            17.5%   compiles
    multierror    14           1             7.1%   compiles
    memberlist   186          78            41.9%   compiles

Nine of ten measured packages compile; only `ksuid` remains, at 7 errors.

The compile gate is now met for every repository the goal names. The REVIEW gates are not: both
reviewers returned DO_NOT_MERGE, and their structural findings — closures with no translation at
all, `Uint128` reimplementing `u128`, `NullUuid` where `Option` belongs, Go's named results
surviving as dead locals — are untouched. Compiling was the first of five conditions, not the last.

## R3n — a named result the body never touches needs no binding

Both reviewers found this independently, one of them without being told the code was generated:

    fn validstring(data: &[u8], mut i: i64) -> (i64, bool) {
        let outi: i64 = 0;
        let ok: bool = false;
        while i < data.len() as i64 {

Six times in `gjson`, and every one dead. The source names its results to document the signature —
`func validstring(data []byte, i int) (outi int, ok bool)` — and then every `return` in the body
states its values explicitly.

The binding exists for a real reason and the reason is conditional: a named result IS a variable in
the source, zero-initialised at entry and assignable, and translating an assignment to one without
binding it emits a body naming a variable that does not exist. So it is bound when the body mentions
it, and when there is a bare `return` — which hands back every named result at once and therefore
needs all of them, whether or not any is named anywhere else.

Zero remain across the corpus. `validstring` now reads as the function a person would write.

## R3o — a closure is an ownership question wearing a syntax costume

`FuncLit` was the loudest cause in the corpus for three phases — 41 sites, 4 packages — and its
refusal said only "source construct `FuncLit` has no translation yet". It is gone. Every closure now
either translates or refuses BY NAME with the decision that is missing.

### What the front end had to answer

Which identifiers inside a literal are CAPTURES is a scoping question. A name resolving to a variable
declared outside the literal is one; the same spelling shadowed inside it is not; the same spelling
bound at package scope is neither. Only `types.Info` separates those three, and the transform
receives names rather than objects — so answering it there would mean growing Go's scope rules inside
the component that exists to keep them out.

So the front end emits a `closure` node with its signature, its body, and one `capture` member per
variable it reaches out of its own scope for, each carrying whether the literal WRITES it. Package
scope is excluded: Go closes over a package variable the same way it closes over a local, but the
target reaches a package name directly from anywhere, so recording them would put the whole package
in every capture list.

Measured on chi: 67 literals — 5 capture nothing, 29 capture one, 26 capture two, 7 capture three.

### What translates, and why the rest cannot yet

A literal that captures NOTHING owns nothing, so it needs no ownership decision at all and becomes a
plain closure with inferred parameter types.

Everything else refuses. The reason is worth stating precisely, because it is not "closures are
hard": the target INFERS borrow-versus-mutable-borrow for a closure that does not outlive its scope,
so the non-escaping case needs no analysis either. What needs an answer is the ESCAPING case, and
the proof it needs is whether the callee RETAINS the value it is given. For a callee outside the
corpus that is unknowable, and guessing it produces either a borrow-checker error or — worse — a
`move` that silently stops the sharing the source performs.

The refusal says that, and deliberately does NOT name the captured variables. They are the site, and
a cause that carries its site counts once per site — which is how one undecided form read as
eighteen rows in R3j and hid the largest blocker in the corpus. Two rows per package now, split only
on whether any capture is written, because that is a different missing decision.

### The engine-identity gate earned its place

`body_closure.rs` was a new file and the whole-engine digest test failed immediately:

    an engine source exists that `engine_digest` does not hash — regenerate the crate manifests,
    because a change to that file would alter output with no receipt axis to account for it

That is the R3d class — an input that changes the output and moves no digest — caught by a gate
rather than by a person, on the first run after the file appeared.

## R3p — a byte-order read takes the FRONT of the slice, and the engine required the whole of it

Second round of review gates, on output that had changed substantially. Still DO_NOT_MERGE, and the
findings are a different KIND: the first round was mostly shape, this one found defects by RUNNING
the code. The most important:

    fn u64(b: &[u8]) -> u64 { u64::from_le_bytes(b.try_into().unwrap()) }
    fn consume_uint64(b: &[u8]) -> (Vec<u8>, u64) { let x = u64(b); (b[8..].to_vec(), x) }

`binary.LittleEndian.Uint64(b)` reads the first eight bytes of `b` and IGNORES the rest; it panics
only when there are fewer than eight. `b.try_into()` requires EXACTLY eight and panics on a longer
slice too. Those are different functions.

`consumeUint64` in `cespare/xxhash` reads eight bytes out of a seventy-six byte buffer and returns
the remaining sixty-eight — so the emitted version panicked on every call it exists to serve. It
compiles, and no fixture could ever have caught it: the difference is invisible on a slice of
exactly eight, which is the only input a hermetic corpus supplies.

The comment sitting on that code said:

> The source's read PANICS when the slice is short, and so does the fit — so the unwrap is the
> source's own behaviour restated, not a failure mode this engine introduced.

Half true, and the missing half is the defect. Slicing to the width first restores the source's
exact condition — panic when short, ignore the rest when long. The width is a property of the
TARGET language rather than a decision, so it is code and not pack data, and a type with no known
width refuses by name.

### Two bugs the fix uncovered, both about places

A source already exactly N bytes wide needs no second slice: `self.0[8..10]` is two bytes by
construction. Only LITERAL bounds count as proof of that — a computed bound has a length nobody
here knows, and treating it as exact is the same mistake one level down.

And `lower_slice_place` lowered a slice's BASE as a value, which borrows it. Invisible until a
slice's base is itself a slice: `&x[a..b]` gains `[..n]`, and the borrow — which binds looser than
everything — swallows the method chain that follows. `&x[a..b][..n].try_into()` is a reference to
the conversion rather than a conversion of the slice, and the type error names neither. The base of
a slice is a place.

### An untyped constant takes the type its uses agree on

    const offsetUppercase = 10        // used only as `offsetUppercase + (digit - 'A')`

An untyped constant has no type until it is used and takes a different one at each use. The target
must state one at the declaration, and `types.Default` — the source's own answer to "what type when
it must have one" — is right with no other evidence and wrong when every use says otherwise.
ksuid's offsets defaulted to `int`, and every use of them was byte arithmetic.

Only unanimity counts, and only for a constant whose initialiser is a LITERAL. Retyping one whose
initialiser is an expression over other constants types only that constant:
`const g1582ns100 = g1582 * 10000000` would be declared at its own agreed type while `g1582` keeps
the type ITS uses agree on, and the target rejects the multiplication. Making the whole constant
graph agree is a transitive problem and a different rule. That regression appeared and was caught by
the compile check within one cycle.

ksuid: 8 compile errors → 3. Every other package still compiles.

## R3q — the gate was measuring a weaker claim than the one the engine is held to

`compile-corpus.sh` ran `rustc` without `-D warnings` and without clippy. The goal says both, each
denying warnings, and the difference is not academic: `pub const K: PrivateType` is a WARNING, so
the table said "compiles" for a crate that under the real policy does not build. Turning both on
was the single most productive change of the phase — every finding below came from it, and each one
had also been named independently by a reviewer.

### A type reached by an exported declaration must be as visible as it

    pub const REQUEST_ID_KEY: CtxKeyRequestId = CtxKeyRequestId(0);   // type is private

Go lets an exported declaration have an unexported type — `var RequestIDKey ctxKeyRequestID` is
idiomatic, and a consumer holds the value without being able to name the type. The target has no
such asymmetry.

The TYPE is widened, not the declaration hidden. Hiding it would delete an exported name from the
ported API, which is the source's contract; widening keeps every consumer able to do exactly what
the source let them do and nothing more. Computed to a fixpoint, because a `pub struct` whose field
type is private is the identical diagnostic one level down.

### A parameter written THROUGH A CALL is still written

    func withSequenceNumber(id KSUID, n uint16) KSUID { binary.BigEndian.PutUint16(id[len(id)-2:], n); return id }

The source spells a write into a value as a CALL and the target spells it as a mutation of the
receiver, so a parameter that only ever appears as such an argument was never seen assigned — and
came out immutable, and did not compile. The walk that answers this already existed for named
results; the parameters simply never asked it.

### The length of an ARRAY is a constant

Go defines `len(a)` on an array type as a constant expression: the length is part of the type. So
folding it is the source's own rule rather than an optimisation applied to it — and it removes a
borrow the source does not have. `PutUint16(id[len(id)-2:], n)` writes into a slice of `id` whose
bound READS `id`; the source allows the overlap and the target does not. With the length folded
there is no read.

Only an ARRAY. A slice's length is a run-time property, and the source draws the same line.

### Four idiom rules, each named by a reviewer and by clippy

- `x >= A && x <= B` is `(A..=B).contains(&x)`, and `x < A || x > B` is its negation. Recognised
  from the BUILT operands, requiring the same subject on both sides, constant bounds, and a subject
  where reading twice is the same as reading once — the source evaluates it twice and the range
  evaluates it once, which differs whenever a call has effects. A negative bound is a unary negation
  rather than a literal, and requiring a bare literal silently declined the widest test in `gjson`.
- A byte range that IS a standard class becomes the predicate for it: `is_ascii_digit`,
  `is_ascii_lowercase`, `is_ascii_uppercase`. Matched on the exact delimiting bytes — a range that
  merely overlaps a class is not it.
- A sequence of byte literals is a byte string, keyed on the ELEMENTS rather than the declared type,
  because the source spells a fixed-size sequence two ways that arrive as different kinds.
- Tabs in doc comments become four spaces. The source indents with tabs because its formatter does.

### Where the range rule went, and why it is not pack data

It is code. The pack answers questions with more than one defensible answer; this one has exactly
one — the two forms are the same predicate over the same values, and a pack that said otherwise
would be wrong rather than different.

There is also a governance reason, and it is the better one. The pack's idiom table REQUIRES
`seed_source`, `seed_license` and `seed_commit`, because `specs/k8s-port/licensing.json` fail-closes
on a rust-skills-derived rule that cannot be re-checked. This rule is not derived from a seed — it
comes from the target's own lint, `clippy::manual_range_contains` — and the only way to satisfy that
schema would have been to invent a commit hash. The policy is fail-closed on purpose, so the rule
went where it needs no such claim rather than being dressed up to pass.

### Result

Nine of ten packages pass `rustc` AND `clippy-driver` with `--deny=warnings` under
`#![forbid(unsafe_code)]`. All five repositories the goal names are among them. `semver` remains, on
`len_without_is_empty`.

## R3r — an interface parameter's form depends on where the method is DECLARED

The pack answered "a parameter of interface type is `&impl Trait`" with a good argument: it accepts
every implementor exactly as the source's does, monomorphises rather than dispatching, needs no
allocation, and avoids the `&dyn` at every boundary that reviewers had named as a port artifact.

The argument holds everywhere except the one place it was applied without qualification.

    // memberlist/queue.go — declared ON the Broadcast interface
    Invalidates(b Broadcast) bool

`impl Trait` in a trait method's argument position makes the trait NOT DYN-COMPATIBLE. So
`&impl Broadcast` made `Box<dyn Broadcast>` impossible — and `[]Broadcast`, a slice of the interface,
is the one data structure the interface exists to hold. It also gave each implementor its own
generic parameter, so two broadcast types could never be compared, which is precisely what
invalidation does.

A trait's declared method and every impl of it now take a `trait_method_param` position, which the
pack answers `&dyn {0}`. The monomorphisation argument keeps everything it can: a free function or
an inherent method still gets `&impl`, because nothing there needs a vtable.

The position is THREADED rather than inferred from whether a body is in hand. An impl method has a
body and still needs the trait's answer — inferring it from what happened to be available is how a
signature and its implementation come to disagree, which this file has now paid for six times.

    fn invalidates(&self, b: &dyn Broadcast) -> bool;

Nine of ten packages still pass `rustc` and `clippy-driver` under `--deny=warnings`; all five the
goal names are among them.

## R3s — the source's `String()` IS the target's `Display`

Third round of review gates, on a bundle of exactly the five repositories the goal names, shaped as
a crate a reviewer would recognise, compiling clean under clippy. Both still DO_NOT_MERGE. Four of
their blocking findings were engine defects; the rest are the Go source's own design, which this
engine does not fix.

### The Stringer

`fmt.Stringer` and `std::fmt::Display` are the same contract — one method, no arguments, renders the
receiver as text — and each language's printing facilities go through its own one. Emitting
`String() string` as an inherent `fn string(&self) -> String` keeps the method and loses the
contract: the ported type cannot then be printed, interpolated, or `to_string()`d by anything
generic, which is most of what a caller wants it for. Both gates named `Version::string()` and
`Address::string()` across all three rounds.

The body machinery already existed. The failure interface's message method becomes a display impl by
the same route, and the renderer already knows to hand a formatting call to the formatter rather
than allocate a string to copy it. So this is the same construction with a different trigger, and
the method is CLAIMED so it is not also emitted inherently — both would compile, the inherent one
would win path resolution, and the duplicate would be invisible until someone deleted it.

REFUSED where the body returns early. The method's body yields the TEXT and the impl's body must
yield a formatting RESULT, so only the tail can be rewritten into a write; a `return` of a string in
the middle would return that string from `fmt`. Such a method stays inherent, because reshaping it
means rewriting every exit and that is a rule about control flow rather than about the trait.

Two clippy failures came with it and both were fixed at their source rather than at the impl:
concatenation onto a literal now puts the literal in the TEMPLATE (`format!("prefix {}", x)` rather
than `format!("{}{}", "prefix ", x)` — `clippy::write_literal`, and it reads as slot-filling), and
`write_str` no longer adds a borrow to a tail that is already one.

### A comment described behaviour the code did not have

    ErrInvalidUUIDFormat  ->  InvalidUuidformat

Beside `InvalidBracketedFormat` that reads as a typo, which is what a reviewer called it. The rule
is that the LAST letter of an uppercase run belongs to the next word when a lower-case letter
follows it: `UUIDFormat` is `UUID` and `Format`, and the `F` opens the second.

That rule was already written in the comment above the code, claiming `HTTPServer` became
`HttpServer`. It did not — it became `Httpserver`. A comment describing behaviour the code does not
have is worse than none, because it stops the next reader from checking.

### The failure alias yields to a name the unit declares

`tidwall/gjson` declares `type Result struct`, and the pack's failure alias is also `Result`. Two
public items of one name in one module is a redefinition if both are emitted and a silent shadow of
whichever loses if they are not. The unit's own type is the SOURCE'S CONTRACT and the alias is this
engine's convenience, so the alias yields — and a unit that needs both refuses at the point it needs
the alias, rather than getting one under an invented second name that no reader of the source could
predict.

Latent rather than live today: gjson's `Result` does not yet translate. It was found by a reviewer
reading an earlier bundle, and it is a trap that would have sprung the moment that type started
emitting.

### What was declined, and why

Several blocking findings are the Go source faithfully carried across: `validate_key` returns a
stringly-typed error thirty lines from a typed enum because that is what the Go does;
`pkcs7decode` trusts its padding byte because that is what the Go does. An engine that improves its
input produces something that is no longer a port of it, and the difference between the two is not
visible to anyone reading only the output. Those stay.

All ten packages still pass `rustc` and `clippy-driver` under `--deny=warnings`.

## R3t — a `?` in a new file re-opened a cascade the engine had already closed

The Display rule landed and `uuid` fell from 29 translated declarations to 15. `google/uuid.UUID`
itself refused, and nine declarations refused BECAUSE it had:

    unmapped type `named github.com/google/uuid.UUID — declared in this unit and not emitted,
    because it refused. Naming it would name a type the crate does not contain`

`display_impl` translated the `String()` method with `?`. A method whose body does not translate is
an ordinary event — it stays inherent and the type still emits — but the `?` let it out of the
function that builds the type's impls, so the whole declaration refused and took everything naming
it with it.

That is R2h exactly: one untranslatable method refusing its entire type. It took six supporting
rules to close the first time, and one `?` in a new file to re-open. The histogram caught it
immediately, which is the only reason it was one cycle rather than a phase.

### `x as i64 < y` does not parse

    error: `<` is interpreted as a start of generic arguments for `i64`, not a comparison

A GRAMMAR rule, not a precedence one — which is why the precedence table, where a cast binds
tightest and so is never bracketed, is right and still not enough. It cost
`gjson::string_less_insensitive`, whose refusal blamed a missing comma and pointed nowhere near the
cause. A cast on the left of a comparison is now bracketed.

### `is_empty` beside a public `len`

DERIVED, not invented: `is_empty` is `len() == 0` and nothing else, so it adds no meaning the source
lacked — it adds the spelling every Rust caller reaches for first, which the target's own lint
requires. The source has no such convention, and that is exactly why nothing carried it across:
Go's `len(c)` is a builtin over the value, and the target's is a method on the type, which brings
the type's obligations with it.

Ten of ten packages now pass `rustc` and `clippy-driver` under `--deny=warnings`.

## R3u — the receiver an interface binds, and two values that were never emitted

Both review gates named the same three things. All three were engine decisions, not the Go source.

### `&mut self` does not merely demand mutability — it forbids sharing

The pack's fallback for a trait method receiver was EXCLUSIVE, argued as the choice that forbids
nothing:

> a shared receiver makes a mutating method unimplementable, whereas an exclusive one only demands
> mutability from implementations that do not need it, which is a compile-time inconvenience rather
> than a lost capability

The premise is false, and both gates found the case that shows it. `memberlist.Delegate` carries its
own documentation across — "All the methods must be thread-safe, as they can and generally will be
called concurrently" — above five methods taking `&mut self`, which is the one signature that
guarantees they cannot be. That is a lost capability, not an inconvenience.

Neither default forbids nothing; they forbid opposite things. What decides it is what an interface
value IS in the source: a copyable, aliasable handle that may be called from several goroutines at
once, whose implementations mutate under their OWN synchronization.
`TransmitLimitedQueue.GetBroadcasts` takes `q.mu.Lock()` before it touches anything. That is shared
access with interior mutability, and the target spells it `&self`.

So the fallback is now SHARED. It remains only a fallback: where an implementor is observed to
mutate, the front end still escalates per method, and that answer still wins.

`Send + Sync` was NOT added, though a reviewer asked for it. It is a bound the source's interface
does not state, and an interface is not thread-safe because its documentation says implementations
should be.

### `R#FALSE` is not an identifier

`gjson`'s `Type` emitted 0, 2, 3, 5 — a public numbered type with two values missing and no hole
where a reader could see them. The Go-aware gate called it a silent fidelity break, correctly.

`to_screaming_snake` was `to_snake_case(..).to_ascii_uppercase()`. The snake form ESCAPES a target
keyword, so `False` became `r#false`, and uppercasing the escape produced `R#FALSE` — which the
renderer refused, by name, for a reason that named the symptom. No escape was needed at all: every
target keyword is lower-case, so a SCREAMING name can never be one. The escape belongs to the
binding form and not to the case conversion, and it is now split out.

Both constants emit. `Type` is 0 through 5.

### A parameter the callee borrows takes a borrow

    parse_int(self.str.clone())     // callee takes `&str`

Newly reachable once those constants emitted, and it is doctrine §9 made visible: the argument was
built for a VALUE position, where a field read of a non-copying type clones because reading one
moves — and the clone handed to a borrowing parameter is both the wrong type and an allocation the
source never performed.

The mechanism was already there for the other direction: the signature table records each
parameter's destination so an argument site can read the SAME decision. Only the borrowing case had
no arm. It reads the target's SPELLING rather than its shape, because a borrowed parameter arrives
both ways — the pack's slice and string idioms produce a path already carrying the `&`, and a
pointer disposition produces a structured reference — and two tests for one question is how the two
ends come to disagree.

The `.clone()` is UNDONE rather than borrowed: `&x.clone()` borrows a temporary that dies at the end
of the statement. And a parameter the enclosing signature already borrows is not borrowed again,
which is the same answer the range loop reads for the same question.

Ten of ten packages pass `rustc` and `clippy-driver` under `--deny=warnings`. The five goal
repositories translate 162 declarations between them.

## R3v — a use at the DEFAULT type is not evidence of a type

    PROTOCOL_VERSION_MIN: u8 = 1
    PROTOCOL_VERSION2_COMPATIBLE: u8 = 2
    PROTOCOL_VERSION_MAX: i64 = 5

The min and max of one inclusive range, at two different integer types, so the range check the Go
performs is not expressible at all. Both gates named it.

In the source all three are untyped except the first. `ProtocolVersionMax` is used as `uint8` in
`[]uint8{ProtocolVersionMin, ProtocolVersionMax, ..}` and in `conf.ProtocolVersion > ProtocolVersionMax`
— and as `int` inside `fmt.Errorf("...%d...", .., ProtocolVersionMax)`, because an `interface{}`
parameter constrains nothing and Go then applies the constant's DEFAULT type.

R3p's rule counted that as a use, saw two answers, found no unanimity, and fell back to the default
— which is the answer the disagreeing use had already supplied. A use at the default type carries no
information: it is what the language does when the context says nothing.

Excluding it can only change the answer when some OTHER use constrains the value, and that use is
the one carrying information. Where every use is the default there is no agreement to find and the
default still stands. All three constants are now `u8`.

### And the value has to be spelled at the type it now has

`const pushPullScaleThreshold = 32` is a float in the source because every use of it is one, and the
rule above now sees that. The target states a constant's type at its declaration, and
`const T: f64 = 32` does not typecheck — a literal with no point is an integer there. Only a value
that is entirely digits gets one; anything else already carries a point, an exponent or a sign it
spelled for itself.

Found by the compile gate one cycle after the rule that caused it, which is what the gate is for.

## R3w — a borrow position does not need the copy a value position asks for

Both gates called the clones gratuitous, and they were — but not because the clone rule is wrong.
A field read of a non-copying type CLONES because reading one moves in the target, and that is
correct wherever the value is kept. Four places asked for a value and keep nothing:

- the receiver of the emptiness idiom — `self.name.clone().is_empty()` allocated a string to ask
  whether it was empty and dropped it again;
- a FORMAT operand, which the macro takes by reference;
- a MATCH scrutinee, which is read and never consumed when the patterns are constants —
  `match self.r#type.clone()` allocated on every call to ask which variant it was;
- and the operand of a concatenation being rewritten INTO a format call, which is a format operand
  by the time it lands.

The method-call receiver already had this right and had had it for phases; the other four had never
been asked. `unwrapped_in` now takes the position, and each site states which one it is.

Clones across the five goal repositories: uuid 0, gjson 0, chi 0, multierror 0, memberlist 1 — and
the last one is real, an owned string returned out of a field.

## R3x — a printable type is not an error

Round 4. The blind gate holds at MERGE_WITH_CHANGES and its top blocker was made by R3s:

> Four types that are not errors implement `std::error::Error`: `gjson::Type` (a JSON value tag),
> `chi::ContextKey`, `chi::middleware::ContextKey`, and `memberlist::MockAddress` (a network
> address). This looks like Go's `String()` method being mistaken for `Error()`.

It was exactly that. `RustItem::MessageImpl` renders a display impl AND the error trait, because it
was built for the source's error interface where both are true. Reusing it for the stringer carried
the second one along — and the consequence is not cosmetic: those types then coerce into a boxed
error, satisfy `?` in any failing function, and are documented as failures. None of which the source
says.

The variant now carries `is_failure`, and the two constructions differ by exactly that.

Which is where the interesting part is. The flag was set BACKWARDS on the first attempt — the two
call sites were distinguished by their order in the file, and `display_impl` had been inserted above
`message_impl` rather than below it. The corpus said so immediately: `Type`, `ContextKey` and
`MockAddress` kept their error impls and `Error`, `Chain` and `NoPingResponseError` lost theirs. A
check that reads the OUTPUT catches an inverted boolean; a check that reads the diff does not.

    uuid: Error, UrnPrefixError, InvalidLengthError    multierror: Chain
    memberlist: Error, NoPingResponseError            gjson, chi: none

### `UUIDs` is an acronym and its plural, not two words

R3s's rule — the last letter of an uppercase run belongs to the next word when a lower-case letter
follows — got `UUIDFormat` right and `UUIDs` wrong, producing `UuiDs`, which a reviewer called a
mangled public type name. One letter is never a word. The rule now requires at least TWO lower-case
letters to follow before the run is broken.

## R3y — two meaning changes, and both were the engine's

The Go-aware gate held at DO_NOT_MERGE and led with two findings it classified as MEANING CHANGES
rather than style. It was right about both, and both were mine.

### Unexported types published as `pub`

R3q widened any type an exported declaration REACHED, so that a `pub` item whose type is private
would not trip `private_interfaces`. The walk descended into struct FIELDS, and a private field of a
public struct leaks nothing — so nine of memberlist's unexported wire structs (`ping`, `ackResp`,
`alive`, `messageType`, `encryptionVersion`) became public API the source does not have.

> Map Go exportedness mechanically: leading-lowercase Go identifier → private, leading-uppercase →
> `pub`.

Which is what `visibility` always did. The promotion now walks only the PUBLIC SURFACE — a
declaration's own type, and its EXPORTED members — because that is the only thing the target's rule
is about. A satisfaction is skipped too: it records that a type was seen implementing an interface,
which says nothing about who may name the type.

### The blank identifier is not a name

    let (n, item) = parse_int(&self.str);      // Go: n, _ := parseInt(t.Str)
    fn notify_msg(&self, arg0: &[u8]);         // Go: NotifyMsg([]byte)
    fn pkcs7decode(buf: &[u8], arg1: usize)    // Go: pkcs7decode(buf []byte, _ int)

Three fabricated names for three values the source deliberately refused to name. `item` came from
`to_snake_case`'s empty-name fallback; `arg0` and `arg1` from a rule that argued it invented nothing
because "the position is already its identity". A name is not a position: `arg0` appears in the
emitted documentation, and every downstream implementor of a trait method has to write it out.

All three are now `_`, which is what the source wrote.

That change cost two declarations before it gained them back, and the reason is the same one the
blank binding hit in R3e: `parse_ident` refuses `_`, correctly, because it is not an identifier. A
parameter is a PATTERN, and the blank is one — so it is spelled directly rather than parsed as a
name. The same mistake in the same shape, one layer along, three phases apart.

## R3z — the owned string literal was text, and that cost the display impl its one optimisation

Two more of the Go-aware gate's findings, and the second is a case of the engine's own principle
paying off exactly as it was written down.

### A block comment documents the block

The source hangs one comment on a `const (..)` group and Go readers understand it covers the block.
The target has no block to hang it on, and the extractor gave the comment to EVERY member:

    /// The list of available message types. WARNING: ONLY APPEND TO THIS LIST! ...
    const PING_MSG: MessageType = MessageType(0);
    /// The list of available message types. WARNING: ONLY APPEND TO THIS LIST! ...
    const INDIRECT_PING_MSG: MessageType = MessageType(1);

Fifteen times, and a reviewer counted them. It goes to the FIRST member now and to nothing else.
Dropping it instead would lose the only place the block's meaning is written down, and "ONLY APPEND
TO THIS LIST" is exactly what a port must not lose.

### A match of literals needs no allocation

    f.write_str(&match *self { NULL => "Null".to_owned(), FALSE => "False".to_owned(), .. })

One allocation per call, for text that never changes, immediately written and dropped. Borrowed arms
make the match a `&'static str`, which `write_str` takes directly.

The rule that recognises this looks for a method call in every arm, and found none — because the
owned literal was built as `RustExpr::Literal(format!("{text}.to_owned()"))`. TEXT, which renders
identically and is opaque to everything downstream. That is the exact defect R3f went and fixed in
two other places, with the principle written down as "a mapped call is a tree, not text", and it was
still here in a third.

Built as a tree, the rule fires:

    f.write_str(match *self { NULL => "Null", FALSE => "False", .. })

The borrowing is done in the RENDERER rather than the transform, because it is a property of this
destination: the same match returned from an ordinary method still owes its caller an owned string.

## R4a — an empty error impl ASSERTS there is no cause

The Go-aware gate classified this as a meaning change, and it is one:

    impl StdError for Chain {}

The target's error trait gives `source` a default that returns nothing, so an impl that says
nothing about it asserts the failure has no cause. `hashicorp/go-multierror`'s `chain` exists for
the opposite purpose — its own doc says it "implements the interfaces necessary for errors.Is/As/
Unwrap to work in a deterministic way" — and it declares `Unwrap`.

There is no general translation to reach for. `chain.Unwrap` returns `e[1:]`, a RE-SLICE OF THE
RECEIVER, and the target's `source` returns a BORROW: there is no reference to a slice that does not
exist yet. A cause held in a FIELD would translate; this one is not held anywhere.

So the type keeps its `Display` and does not get the error trait, and the cause method is RECORDED
as dropped so the survey says why.

### Recorded, not raised — and the third time this file has had to learn it

The first attempt returned `Err`. `message_impl` is called through `.transpose()` from the function
that builds a type's impls, so the error refused the whole declaration and took `multierror`'s only
translated type with it: 1 → 0.

That is the type/method cascade R2h closed, R3t re-opened with a `?` in `display_impl`, and this
re-opened again one function away. Three times, in three files, each time by reaching for `?` where
the code wanted a decision. The pattern is specific enough to name: **inside the impl builders, a
failure to translate one member is a fact about that member, and the only correct shapes are
`Ok(None)` or a recorded drop.**

## R4b — the range rewrite answered the opposite for NaN

Round 5. The blind gate holds at MERGE_WITH_CHANGES with ONE blocking finding, down from four. The
Go-aware gate found this, and it is the worst kind of defect this engine can produce — a silent
change of meaning, introduced by a rule added for tidiness.

    Go   (gjson.go:2810):  if f < -9007199254740991 || f > 9007199254740991 { return 0, false }
    port (R3q):            if !(-9_007_199_254_740_991.0..=9_007_199_254_740_991.0).contains(&f)

Verified by running both:

    go_form(NaN)   = false
    port_form(NaN) = true

Every comparison against NaN is false, so the source's disjunction is false and the function does
NOT return early — it returns the value. The negated `contains` is also false, and the `!` turns
that into true, so the port returns `(0, false)`. `gjson.safe_int(NaN)` is a different function in
the port, on an input a JSON parser certainly receives.

The POSITIVE form has no such hazard: `x >= A && x <= B` and `contains` are both false for NaN and
agree everywhere else. Only the negation inverts the disagreement into an answer. So the negated
rewrite now declines a float bound, read from the spelling — a whole number resolved to a float is
spelled with a point precisely so the target reads it as one.

### And then clippy demanded the wrong answer back

`clippy::manual_range_contains` fires on the source's own form, and `--deny=warnings` makes that a
build failure. The lint is wrong here and the engine says so, in the one shape the doctrine permits:

    #[allow(
        clippy::manual_range_contains,
        reason = "the range form answers the opposite for NaN: every comparison against NaN is
                  false, so the source's disjunction is false and the negated contains is true"
    )]

This is the first `allow` the engine emits, and the conditions it was reserved for are exactly met:
it is generated, it carries a machine-readable justification, and it is attached to the one function
whose rewrite was declined rather than to a crate or a module. `DOCTRINE.md` §4 said an allowance
must be "globally approved by policy, or generated with a machine-readable semantic justification".
This is the second.

The first attempt at it refused the whole declaration, because the attribute was emitted without its
`#[..]` wrapper and the renderer parses attributes rather than pasting them. That refusal was
correct and immediate — the closed vocabulary catching a malformed construction before it reached a
file.

## R4c — a struct tag is a wire contract, and the engine could not see it

The Go-aware gate's third lead finding, and the one the standing goal names outright — the engine
"must specifically survive complex JSON/YAML tag reflection".

    type ping struct {
        SeqNo      uint32
        SourceAddr []byte `codec:",omitempty"`
    }

`memberlist`'s ten message structs are encoded by go-codec as msgpack MAPS KEYED BY FIELD NAME. The
port renamed every field to snake_case — `SeqNo` to `seq_no`, `SourceAddr` to `source_addr` — which
changes every key on the wire. A gossip protocol that no longer speaks to its own other half, and it
compiles.

The tag was not recorded AT ALL. The front end read the field's name, type, flags and doc, and
dropped the one attribute that says what the field is called when it leaves the process. Six tags
across the corpus, invisible.

Recorded rather than interpreted: WHICH library reads a given tag, and what the target's counterpart
is, are decisions about the ported program. That the tag EXISTS is a fact, and the transform cannot
refuse what it cannot see.

And it refuses. The type's field NAME is half of the contract — a serialization library keys by it
unless the tag overrides it — so the target's naming convention silently rewrites the format. The
engine has no serialization decision to spend, so the struct refuses by name and says which field
carries the contract it cannot keep.

    field `SourceAddr` carries the source tag `codec:",omitempty"`, which names its identity in a
    serialized form — and the target's field naming would change that identity while still
    compiling. Preserving it is a decision about which serialization the ported program uses, and
    the pack declares none

memberlist: 78 → 76 translated. Two declarations, for a whole class of silent wire-format change.

## R4d — a local that only walks a sequence is the target's index type

The most-cited surviving Go shape, in both gates and across four rounds. The Go-aware gate counted
it: 119 conversions across the five packages, and named `gjson::string_less_insensitive` the worst
case.

    while i < a.len() as i64 && i < b.len() as i64 {
        if a.as_bytes()[i as usize] >= b'A' && a.as_bytes()[i as usize] <= b'Z' {

The source types a cursor `int`, the pack maps that to a signed 64-bit integer, and every use then
converts. The engine already decided this for a LOOP COUNTER and for a PARAMETER used only to
index — a local was the one place nobody asked.

A binding qualifies when EVERY read of it is an index operand, a slice bound, a comparison against a
length or a literal, or its own increment. The disqualifying use is the point: `parse_int`'s `n`
accumulates a value the caller receives, so it stays the source's integer, while `i` beside it walks
the string and becomes an index. A rule that could not tell them apart would have to leave both.

Three ends had to agree, and each was a separate place that had never been asked:

- the BINDING states `usize`;
- its INITIALISER is built at the index type, or `let l: usize = key.len() as i64` states one type
  and supplies another;
- a COMPARISON against a length drops the conversion the length mapping adds — `compares_lengths`
  knew about length CONSTANTS and not about proven cursors, so the binding changed and the
  comparison did not.

All three read the same set, which is what stops them disagreeing.

    fn string_less_insensitive(a: &str, b: &str) -> bool {
        let mut i: usize = 0;
        while i < a.len() && i < b.len() {
            if a.as_bytes()[i] >= b'A' && a.as_bytes()[i] <= b'Z' {

No conversions at all, in the function the reviewer picked out. Casts across the five: 37 → 31, and
the ones left are on values the source's own signatures hand back as integers — `validstring`
RETURNS its cursor, so it is an `int` to its caller and changing it would change the signature.

## R4e — the gate was suppressing a lint class that contained real defects

Round 6. **Both gates now say MERGE_WITH_CHANGES.** The Go-aware gate came off DO_NOT_MERGE for the
first time in six rounds, and its summary is the one worth recording:

> No meaning was changed anywhere — every constant table, bit value, and branch I checked matches
> the Go exactly

The blind gate had ZERO blocking findings, and said why:

> I found no correctness defect that the port introduced on its own: the hex-value table verifies
> byte-for-byte against the ASCII digit set, the JSON scanners and `parse_int`/`parse_uint`
> reproduce the source's wrapping and bounds exactly, and the panics in `pkcs7decode`,
> `encrypt_overhead` and `Chain::fmt` are the original's behaviour rather than new hazards

That last clause is the fidelity boundary the round-6 instructions made explicit, and it removed
three blockers that no translation change could ever have removed.

### `-A unused` was hiding three translator artifacts

`compile-corpus.sh` passed `-A dead_code -A unused`. Dead code IS expected — the engine emits only
what it can prove, so a translated helper whose only caller refused is unused through no fault of its
own. `unused` is a GROUP, and it contains `unused_imports` and `unused_variables`, neither of which
is expected and both of which the port was producing. A reviewer found them by running the compiler
without the flag this script was passing.

An expected warning class is one thing; a blanket group that happens to contain it is another.

Two defects fell out immediately:

- `use std::error::Error as StdError` was emitted into every unit with any message impl — and a
  display impl built from the source's STRINGER renders `fmt::Display` and nothing else. `gjson` and
  `chi` imported the error trait and never named it. The import now follows `is_failure`, the same
  fact the impl itself follows.
- `for j in 0..4` bound a name the body never mentions. The source counts because it has no other
  way to repeat four times; the target has `for _ in 0..4`.

### A value that can write itself

    f.write_str(&self.0[0].to_string())

An allocation made purely to copy into the formatter and drop. The source had no choice — its method
must RETURN a string — and the target's formatter takes the value directly:

    fmt::Display::fmt(&self.0[0], f)

Ten of ten packages pass under the tightened gate.

## R4f — every exit of a display body writes, so an early return is no longer a refusal

Both gates named it, and the Go-aware one named the tell precisely:

> `Version::string()` and `Address::string()` are Go `String() string` methods left as inherent Rust
> methods ... The port is inconsistent about how it renders Go's `String()`. Ten types get a real
> `impl fmt::Display` — including `MockAddress` in this same file — while two are left inherent. Two
> public types in one file, same shape, opposite mappings.

The discriminator was an early `return`. R3s refused those, arguing that "reshaping it means
rewriting every exit and that is a rule about control flow rather than about the trait". Rewriting
every exit is what a PERSON does, and it is two lines once the write is a node rather than a case in
the renderer.

`RustExpr::FormatterWrite` is that node. How a value reaches the formatter — a formatting call goes
directly, a match of literals borrows its arms, a value that can write itself does, anything else is
written as the string it is — was a `match` on the LAST STATEMENT inside the renderer, which is
exactly why only the tail could ever be rewritten. It is now one node, decided in one place, and the
transform wraps the tail AND every `return` in it.

    impl fmt::Display for Version {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.0 > 15 {
                return write!(f, "BAD_VERSION_{}", self.0);
            }
            write!(f, "VERSION_{}", self.0)
        }
    }

Zero inherent `string()` methods remain across the five goal repositories.

The first attempt emitted `f.write_str(&write!(f, ..))` — double-wrapped, because the renderer's old
tail decision was still there beside the new node. Two spellings of one decision, for the length of
one build.

## R4g — a dropped sentence takes its pronouns with it

    /// It's used as
    /// a pointer so it fits in an interface{} without allocation.
    pub struct ContextKey { name: String }

A reviewer read that and said it had no subject. It did not: the sentence before it —
"contextKey is a value for use with context.WithValue." — names `context.WithValue`, which the
emitted crate does not contain, so the dangling-reference rule dropped it. The rule is right, and
its INTERACTION with what follows was not: removing a sentence strands every pronoun that referred
to it.

A sentence whose subject was dropped is now dropped too, and the rule carries forward — three
sentences where the first goes and the second refers to it lose both, and the third survives only if
it does not refer to the second. `ContextKey` ends up with no documentation rather than documentation
about nothing, which is the honest outcome: every sentence the source wrote about it referred to
something that did not come along.

The pronoun list is CLOSED, and the cost of being wrong runs one way only. A sentence wrongly held
to lean on its predecessor is dropped with it and the documentation is shorter; one wrongly kept is
prose referring to something no longer there.

## R4h — a method whose every exit is a literal hands back a borrow of it

    pub fn network(&self) -> String { "mock".to_owned() }

The source has to return a `string` because that is the only string it has. The target then
allocates a copy of text that never changes, on every call, and most callers drop it immediately.
A `&'static str` is the same text with a lifetime the caller can keep just as long.

INHERENT METHODS ONLY, which is why the rule runs where inherent methods are assembled rather than
over every emitted function: a method implementing a trait owes that trait its signature, and
changing one end of a contract is not an improvement.

EVERY exit, and only literals. A method with one computed branch has no static text to hand back,
and rewriting the branches that do would leave arms that no longer agree.

    pub fn network(&self) -> &'static str { "mock" }

## R4i — a derive is a claim about a type, so ask the package that DECLARES it

Five rules landed together, four of them small. The fifth found a hole under all of them.

`Copy` joined the derive list, blocked by the kinds that cannot be copied — interface, slice, map,
pointer, string, chan, func. Twenty-nine of the fifty-six derived types in the corpus earn it, and
`clone_on_copy` stays silent, so no `.clone()` survived on a type that no longer needs one.

Then the fixture corpus refused to compile:

    error[E0204]: the trait `Copy` cannot be implemented for this type
        pub min: crate::shapes::Point,   // this field does not implement `Copy`

`Bounds` holds two `shapes.Point`, and `Point` holds a `String`. The recursion that answers "does
this field block the derive" already followed named references — but through a table holding ONE
unit, keyed by bare name. A reference to another package missed it, and the miss was read as
"blocks nothing". The engine claimed a trait it had never checked.

The same table makes a worse mistake it had not yet been caught making. This corpus declares
`Counter` in both `handoff` and `pointers`. A bare-name lookup answers a reference to one with the
OTHER's fields — not a missing answer but a confidently wrong one. `TypeRef::package` exists for
exactly this, and its doc already said so: it "is what makes a named type ADDRESSABLE. Without it,
a reference to another package's type is indistinguishable from a local one."

So the table is now built across the whole model and keyed by (declaring package, name), and a
reference resolves against the package that declares it. An empty package means the referring unit
declares it itself.

An unresolved reference now BLOCKS rather than continuing. Nothing in the model declares it — an
interface literal, a type from outside the corpus — so the engine cannot prove the trait, and it
does not claim it. Only `named` references participate: a basic carries a name too, `int`, `string`,
that no declaration answers, and blocking on those would have blocked everything.

Blocking alone was measured first and REJECTED as the whole fix. It is correct and it is lossy:
`Bounds` earned nothing at all, when `Point` legitimately earns `Debug, Clone, Default, PartialEq,
Eq, Hash` and only `Copy` was ever wrong. Resolution restores exactly those six. Across the thirteen
corpus packages no emitted struct is left without a derive line, so the conservatism costs nothing
measurable and buys back a compile failure and a silent collision.

The four that landed beside it:

- A write borrows its value. `f.write_str(&self.addr.clone())` allocates a copy to hand to something
  that only reads it.
- A length of zero is emptiness. `len(x) == 0` is `x.is_empty()`, and `> 0` is `!x.is_empty()`.
- A type is promoted to `pub` only on behalf of a declaration that is actually EMITTED. chi's
  `RouteCtxKey` refuses as `exported_package_var` and was still widening `contextKey` to `pub` —
  a type made public on behalf of a reader that does not exist.

## R4j — an escape denotes a number, and the number is the same in both languages

The refusal histogram named chi's top cause, and reading it as the method requires showed it was
not a refusal at all:

    17  renderer refused: `b'\033'` is not a valid target literal: cannot parse string into token stream

`\033` is how the source spells ESC. The rune path copied the source's spelling into a target byte
literal — `b{value}` — and the target has no octal escape, so `syn` could not parse it. Seventeen
declarations of a real package were blocked on a literal whose value was never in doubt.

The source's escape vocabulary is WIDER: `\a`, `\b`, `\f`, `\v` and three-digit octal have no target
spelling. They cannot be carried across as text. They can be COMPUTED, because an escape denotes a
number and the number is the same in both. `body_escape.rs` decodes one and the caller re-spells it:
`'\033'` becomes `b'\x1b'`, and a `char` gets `'\u{..}'`.

The escapes the two languages SHARE keep the source's spelling. `'\n'` stays `'\n'` — respelling by
code point is for the escapes that need it, and making it the rule for all of them would be a worse
output for no gain. That case is a test rather than a comment.

chi went 33 → 50 translated, 17.5% → 26.5%. Then the compile gate went red with 18 errors, which is
the whole point of running it after every rule:

    error: can be more succinctly written as a byte str    x17

`clippy::byte_char_slices`. The byte-string rule refused any element needing an escape, on the
grounds that "getting one wrong changes a byte" — and chi's colour tables all BEGIN with the escape
byte, so every one of them emitted the long form. Decoding removes that risk rather than avoiding
it: the byte is read once as a value and re-spelled from the number.

    const N_BLACK: [u8; 5] = *b"\x1b[30m";

A byte the string can hold literally is written as itself; a quote, a backslash, or anything outside
the printable range is written as its own `\xHH`.

TWO RULES, ONE FINDING, and the second only existed because the first landed. A rule is not finished
when its cause leaves the histogram.

## R4k — the minimum supported release is go1.22, and it is not the same number as the extraction release

Go 1.22 gave each loop iteration its own variables. A closure created in a loop captures one shared
variable under the old rule and a fresh one under the new: the same syntax, a different program, and
nothing downstream can see which was meant. Supporting 1.21 would mean implementing BOTH capture
rules and selecting per module. Making 1.22 the floor deletes the choice instead of answering it.

`regen-fixtures.sh` had been failing since the ceiling guard landed — the fixture module declared
go1.24 while extraction ran at the configured go1.21 — so the fixtures had not been regenerable for
some time and nothing said so. The module now declares 1.22.

The two numbers are DIFFERENT THINGS and the constant was conflating them. `defaultRelease` is the
engine's floor. The release a corpus is extracted at is the one ITS OWN module declares — gjson and
chi say go1.23, memberlist says go1.25 — and it is a receipt axis, so it is passed per corpus rather
than defaulted. Raising the floor to 22 made that visible by refusing three corpora that had been
extracting by accident.

Moving the floor changed `build_config` and nothing else: the golden is byte-identical at 1.22.

## R4l — the manifest is only the whole engine if something regenerates it

Adding `body_escape.rs` turned `the_manifest_is_the_whole_engine` red, correctly: a source nobody
lists changes emitted bytes with no axis movement. The step was documented as required after every
source change and had no tool, so it was done by hand or not at all. `regen-sources.sh` does it, and
its first run reproduced all thirteen existing manifests byte-for-byte while adding the one missing
line — which is the evidence that it matches what was there.

## R4m — ownership is a property of the DESTINATION, and one destination is visible

The largest cause in the corpus by declarations — 35 across five packages — was the closure that
captures. The refusal was true: which of the target's four answers a capture takes is decided by
where the literal GOES, and for most destinations the engine cannot see that.

For one it can. A literal among a `return`'s operands OUTLIVES the frame it is written in, so its
captures cannot be borrowed from that frame. That is a SOURCE fact — the front end sees the literal
sitting in return position and the transform never can — so the extractor records it as
`destination`, and the transform decides what to do about it. Named `destination` rather than
`escapes` because an `escapes` FLAG already exists and means the ownership analysis's own answer.

Owning is not automatically faithful, and this is the half that matters. Go's closure shares the
variable's STORAGE: where anything reassigns a capture — the enclosing body, or a second literal
over the same variable — the source has ONE value and `move` would make several, which stop
agreeing at the first write. So the extractor also records `reassigned`, and a returned literal
whose capture is reassigned KEEPS REFUSING, naming the capture that forced it.

    n := 0
    f := func() { print(n) }
    n = 5
    return f            // Go prints 5; `move` captures 0. Refused.

chi's closure cause went 26 → 12. Coverage did not move, and that is the finding worth recording:
those declarations hit their NEXT blocker — `net/http.HandlerFunc`, `TypeAssertExpr`. A corpus
declaration has several blockers and the survey reports the first, so clearing a cause moves the
HISTOGRAM while coverage stays flat until a declaration's LAST blocker falls.

Checked and DISPROVED along the way: that foreign standard-library types are what gate the goal
repositories. They are 8–19% of refusals, not the majority. The blockers are ordinary language
constructs.

## R4n — what the source's `range` means depends on what is ranged

`for i, v := range xs` refused. The reason given was that binding the index "needs a rule for
whether the element is a copy or a reference", and the real question is a different one: the source's
`range` binds different things by TYPE. Over a sequence the first name is an INDEX; over a map it is
a KEY; over a string it is a BYTE OFFSET with decoded runes for values. The loop's shape does not
say which — the ranged expression's type does, and it was never consulted.

Sequences translate:

    for i, v := range xs      ->  for (i, v) in xs.iter().enumerate()
    for i := range xs         ->  for i in 0..xs.len()

`enumerate` yields the index first and the item second, which is the order the source binds them in,
so the pattern is a transcription rather than a reordering.

A MAP refuses, for the reason its literal already does: the source's map has no order, the target's
has one, and iterating it would make an order observable that the source never promised. The same
decision in two places, and it should be made once.

The binding stopped being a `String`. A single name is an identifier and a pair is a PATTERN, and
the renderer parses what it is handed — so `ForBinding` is `Blank`, `Name` or `Indexed`, and the
blank is a variant rather than the name `_`. That is the third time the blank has been spelled as a
name and reached `parse_ident`; modelling it closes the trap rather than avoiding it again.

## R4o — `copy` writes, so its destination must be a PLACE, and the engine was handing it a clone

The source's `copy(dst, src)` moves `min(len(dst), len(src))` elements and answers with that count.
The target has no call that does it, so it becomes a length and a slice assignment — which is why
this is a block rather than a call, and why it needs two proofs the call form would not.

The first was expected. Each argument is named TWICE in the target, once for its length and once
for the slice, where the source names it once — so an argument that DOES something when evaluated
would do it twice. Stated as what it excludes rather than what it admits: a call, an assignment, an
increment. An admitting list is the version that goes wrong, and it did — the first one forgot
literals and refused four of `uuid`'s copies over a literal slice bound.

The second was not expected and is the reason this entry exists. The rule emitted:

    self.uuid.clone()[..][..copied].copy_from_slice(&data[..copied]);

That compiles. It copies into a CLONE, drops it, and leaves `self.uuid` untouched — a copy that
does nothing, which is precisely the class of failure this engine exists to prevent. The `.clone()`
is not in the source and was never visible to a check on the source's shape: OWNERSHIP puts it
there, downstream, after the argument is translated.

So the destination is checked on the EMITTED form, and the property is that it names STORAGE: a
path, a field of one, an index or slice of either. A call is not a place. Every `copy` in the
corpus refuses on it today, which is the correct answer to what is currently emitted and not the
end of the matter — the engine cloning a destination it is about to WRITE is a defect upstream of
this rule, and this is the first thing to name it.

Also fixed here: a range's lower bound is now optional. `..copied` is what the target spells for a
prefix; `0..copied` compiles to the same thing and reads as something a person did not write. The
first version passed an empty literal instead, which the renderer refused — and that refusal was
counted as a translation failure in `uuid` before it was traced back to here.

## R4p — review round 8: the blind gate went BACKWARDS, and it was right to

    round 7   blind MERGE_WITH_CHANGES (1 blocker)    go-aware MERGE_WITH_CHANGES (5 blockers)
    round 8   blind DO_NOT_MERGE       (9 blockers)   go-aware MERGE_WITH_CHANGES (9 blockers)

The blind verdict got WORSE while the engine got better, and the reason is the instrument rather
than the output. This reviewer wrote its own `Cargo.toml` and ran `cargo build`. The corpus gate
passes `-A dead_code`. Without that flag the bundle emits 121 dead-code warnings on 121 items:

> This is the type-and-constant skeleton of four Go packages with the function bodies dropped —
> not a library.

That is the coverage number restated as a review finding, and it is the single most important thing
either gate said. `uuid` translates 29 of 125 declarations; what emits is the types, the constants
and a handful of leaf functions. A reviewer handed that does not see a partial port, it sees a crate
whose public types cannot be constructed and whose modules export nothing that works. No amount of
idiom work moves the blind gate while that holds. COVERAGE IS THE GATE NOW.

`-A dead_code` is defensible for a library — a public API has no internal callers — and it hid the
one fact that mattered. The gate should report the count rather than suppress it.

Two harness defects, both mine, both already paid for once: the bundle ships no `Cargo.toml`, so
the reviewer had to synthesize one; and it has no tests, which the reviewer counted against it.

### Findings classified

CONFIRMED ENGINE DEFECTS, not idiom:

- Ownership inserts `.clone()` on a destination that is then WRITTEN. Found independently by R4o.
  This is the same defect the blind reviewer's dead-code finding hides: the write is lost.
- Go's `int` emitted as `i64` for every length, size and offset, then cast back with `as usize` at
  every index — 54 cast lints under pedantic. `LABEL_MAX_SIZE: usize` sits five lines from
  `META_MAX_SIZE: i64`, both byte counts, which is the tell that this is transliteration rather
  than a decision. A value derived from a length and used only as an index is a `usize`, and the
  engine already types loop cursors that way; the rule exists and does not reach these.
- `pkcs7decode` computes a negative length in `i64` and indexes with `n as usize`, giving a slice
  bound of ~1.8e19. Go panics here too, so the CLASS is preserved — but the engine introduced the
  cast, and a wrapped bound is not the source's bound.

DECLINED, with reasons:

- `safe_int` admits NaN. Both reviewers found it; it is upstream's behaviour, faithfully carried.
  Go's `int64(NaN)` is spec-undefined and the target saturates to 0, so the VALUE differs — worth
  recording as exposure, not worth inventing a fix the source does not contain. Fixing it here
  would be redesign, which §10 already rejects.
- `Delegate: Send + Sync`. Raised in round 7 and again now. The source states it in prose and not
  in its type, and the engine may not promote a comment to a bound.
- `RouteParams` parallel arrays. The Go-aware reviewer checked this one and withdrew it: chi's
  ordering is significant and duplicate keys are legal, so a map would change behaviour. Recorded
  because a reviewer that argues itself OUT of a finding is the instrument working.

ACCEPTED AS THE NEXT RULES, in the order the histogram supports them:

- `iota` const blocks are CLOSED sets and become enums, not open newtypes. Five instances. The
  reviewer separated the closed cases from the genuinely open ones — `MessageType` and `Version`
  are decoded from wire bytes and must stay newtypes — which is exactly the distinction the pack's
  sentinel reason already draws for error enums, and the same evidence test applies.
- `(T, bool)` becomes `Option<T>` where the `T` is a zero-value throwaway. Three private functions
  in gjson; no API-compatibility argument against it.
- A trait method returning an owned `Vec`/`String` where the source returned a borrow.

## R4q — a shift the engine can size is arithmetic, and a refusal without a reason is not a refusal

Two findings, one of which the survey had been hiding for a long time.

The corpus refuses `<<` and `>>`, and the reason written beside the operator table is good: the
source defines a shift at or beyond the operand's width as ZERO and panics on a negative count,
while the target panics on the first in a debug build and masks the count in a release one — three
behaviours where the source has two. That reason NEVER REACHED THE READER. The refusal said:

    binary operator `>>` has no direct translation

which names the operator and nothing a reader can act on. Condition 4 of the standing goal calls a
vague refusal illegitimate, and this is what one looks like: the analysis existed, and it existed
where only somebody already reading that file would find it. The message now carries it, and `&^`
carries its own.

The disagreement also has a subset where there is nothing to disagree about. A count that is a
LITERAL below the operand's width leaves neither language a case to handle differently, so the plain
operator is exact. A literal is never negative here either — the source spells a negative count as a
negation OF a literal, which is a different node. `usize` and `isize` are measured at 16, the
smallest width the target permits, because their real width belongs to the machine and not to this
snapshot.

    const TIME_DELTA: i64 = 1 << 6;
    (b1 << 4 | b2, b1 != 255 && b2 != 255)

## R4r — the declaration shouted and the body whispered

Letting `xtob` translate made `uuid` stop compiling:

    error[E0425]: cannot find value `xvalues` in this scope
       const XVALUES: [u8; 256] = [ ... ];

A package-scope `var` is emitted as a value with the target's CONSTANT casing — that is what
`items_static` spells, unconditionally — and a reference to one was cased as a local. Both ends of
one decision reading different answers, which is the first failure mode the standing goal lists and
the sixth time it has been paid for.

It was invisible until now for a reason worth keeping: the only function that referred to `xvalues`
refused for an unrelated reason, so the mismatch had nothing to compile. A rule that raises coverage
does not only add code, it EXPOSES code that was already wrong.

The front end classifies a package-scope variable distinctly from a local, so the fix reads that
classification rather than guessing — and a local shadowing the name keeps its own casing.

## R4s — the concurrency trio becomes visible, and says what it is waiting on

Until now the front end did not model Go's concurrency at all. `go`, `select` and `ch <- v` reached
the model as `unsupported`, which meant the survey could not name them and no rule could be written
against them however good the analysis was. They are modelled now: 14 goroutines, 21 selects, 20
sends, 44 select arms across the corpus.

The `default` arm is recorded distinctly, because a select WITH one never blocks and one without
always may — two programs, not a detail of one. A goroutine records the CALL it starts rather than
folding it into whatever the target spawns, because the source evaluates that call's arguments
before the goroutine begins and folding would lose the ordering.

`go` is also a DESTINATION, the second one the engine can see. A literal a goroutine starts outlives
the frame that wrote it exactly as a returned one does, and more strictly: a return needs its
captures OWNED, a spawn needs them owned AND sendable. The ownership half of R4m therefore applies
unchanged, and the sendability is the part still missing.

None of this emits anything, and that is the honest state. What each of the three now does is REFUSE
BY NAME with what it is waiting on, replacing "source construct `GoStmt` has no translation yet":

- a goroutine waits on COLOUR before it waits on the spawn. The source's goroutine blocks anywhere;
  the target's task blocks only at an `.await`. So every callee reachable from a spawn that blocks
  has to become an async function — a property of the whole call graph, not of the statement. A
  spawn whose body blocks the executor compiles and stalls every other task, which is worse than
  emitting nothing.
- a send waits on the pack naming a channel. The source's is multi-producer AND multi-consumer,
  closable, and unbuffered by default, where a send does not complete until a receiver takes the
  value. Each target candidate keeps some of that and drops the rest, and which one it becomes
  decides what the ported program does when a peer is slow.
- a select waits on two things the shape does not give. The source chooses UNIFORMLY AT RANDOM
  among ready arms and the target's macro is biased unless told otherwise, which changes which arm
  starves; and an arm not chosen must not have consumed anything, which holds only for
  cancellation-safe operations and is a property of each arm's callee.

Both closed vocabularies caught the new kinds and the new attribute before any of this ran, which
is the fail-closed admission doing its job twice in one change.

## R4t — the ceiling is not missing syntax, and here is the measurement that says so

Pointer dereference and type assertion were the two largest unmodelled constructs — 127 and 112
occurrences. Both are modelled now. Neither moved coverage by a single declaration.

That is not a failure of the two rules; it is the answer to a question that had been guessed at for
several phases. Across the corpus:

    functions and methods                                 642
    containing NO unmodelled source construct             566
    blocked by exactly one unmodelled construct            69

566 of 642 functions contain nothing the front end cannot see, and coverage is 15–41%. So the
ceiling is NOT missing syntax support. Modelling every remaining Go AST node would unlock at most 69
declarations, and the measurement above overstates even that: 35 of the type assertions are out of a
BARE INTERFACE, which refuses on a decision already made — the source's `interface{}` carries its
own type at runtime and the target has no counterpart — so those declarations would refuse again one
step later, on the type instead of the assertion.

What the refusals are actually made of, over 700 of them:

    124  cascade — refused only because something it names refused
     58  mutable package state
     54  interface and dynamic type
     36  a foreign package not in the snapshot
     29  string representation
     25  closures and ownership
     22  control effects (defer, panic, recover)
     16  a failing return's operand not provably a failure
     12  concurrency

The 124 cascade are free: they come back when their roots do. Of the roots, the largest groups are
not rules waiting to be written. `written_package_var` needs a choice among mutex, reader/writer
lock and atomic, which is a decision about the ported program's CONCURRENCY. `exported_package_var`
needs to know whether an importer assigns to it, which no single package can answer.
`init_written_package_var` needs a purity proof. The bare interface needs the compatibility lane
that §2 forbids. Each is a DECISION, and the doctrine is explicit that the engine may not make one
on the author's behalf.

Both constructs still earn their place. `deref` is exact for a write — a write through a pointer
copies nothing in either language — and for a read the target refuses to compile rather than doing
something else, which is the one direction this engine can take on trust. And `assert` now says
WHICH of two different things is missing, where before both said "no translation yet": out of the
failure type it is a downcast the target has and the source's comma-ok is its `Option` exactly; out
of a bare interface it is waiting on a decision that has already been made the other way.

## R4u — a `continue` that must not skip the step, and the case that was never a pattern

Two findings, and the second only existed because the first landed.

### The step block

`for ; i < len(data); i++ { ... continue ... }` refused. The reason was right — the source's
`continue` jumps to the POST clause and the target's jumps to the TEST, so a post-statement spelled
at the end of the body is skipped on exactly those paths and the loop counts differently — and it
was not the whole answer, because the target has the construct that says it exactly.

A LABELLED BLOCK around one iteration turns `continue` into `break 'step`, which leaves the block
and lands on the post-statement: the same place the source goes.

    while i < data.len() as i64 {
        'step: {
            match data[i as usize] {
                b' ' | b'\t' | b'\n' | b'\r' => break 'step,
                ...
            }
        }
        i += 1;
    }

A `break` written in the same body is relabelled too, because inside the block a bare one leaves
the BLOCK — one iteration — rather than the loop. The loop is labelled only when that happens: an
unused label is a warning, and the emitted crate is held to deny-warnings. The walk does not
descend into a nested loop, whose jumps belong to it.

gjson 19 → 22 translated.

### The case that was never a pattern

Which made `validcomma` translate, and it was wrong:

    case ' ', '\t', '\n', '\r':  continue
    case ',':                    return i, true
    case end:                    return i, true      // `end` is a PARAMETER
    default:                     return i, false

became

    end => { return (i, true); }

`end =>` is not a comparison. It is an irrefutable BINDING that shadows the parameter and matches
every byte, so the `default` arm is unreachable and the function returns success for everything it
should have rejected. It compiles.

A Go case is a value to compare against — the IR's own doc said so — and only some comparisons are
patterns. A literal is one; a name the source declares `const` is one, because the target emits it
as a `const`. Anything read at run time is not, and becomes a GUARD:

    _ if data[i as usize] == end => { return (i, true); }

The guard names the subject again, so the subject must be nameable twice without doing anything —
the same predicate `copy` needed, reused rather than restated. `case a, b:` is an OR of the
comparisons.

THE COMPILE GATE CAUGHT THIS, unreachable-pattern under deny-warnings, before any of it was
believed. That is the second time in this session a rule that raised coverage exposed a defect that
had been sitting behind a refusal, and the first time the defect was one the engine would otherwise
have shipped.

## R4v — `return f(x)` was never claiming a failure

Sixteen declarations across seven packages asked the engine to prove a `call` was "certainly a
failure" and it could not. They were never claiming one.

    func (i *KSUID) Set(s string) error { return i.UnmarshalText([]byte(s)) }

The source hands its caller whatever `UnmarshalText` answered — success or failure — and the
target's `Result` is that same answer. Nothing is being wrapped, so there is nothing to prove about
the operand: `Err(..)` never appears. It is a method that exists to delegate, and that is common.

Two things are read off the model rather than assumed. The operand is a CALL, because a name that
merely holds a failure is a value being wrapped rather than an answer being passed on. And the
CALLEE's own results are the failure alone — which the model already carries on the call's SELECTOR,
because the front end records a callee's whole signature there. No signature table had to grow a
results column, which was the shape this rule was expected to need.

The third condition the caller had already proved: this is reached only where the return's operands
are the failure and nothing beside it, so the enclosing function results in exactly what the callee
does. Equal shapes are the whole requirement.

A CONSTRUCTOR is not a forward, and missing that broke `memberlist`:

    return errors.New("key size must be 16, 24 or 32 bytes")

is a call whose result is the failure type, and it is the source saying FAIL WITH THIS — the callee
had no chance to succeed, so there is no answer to pass on. The pack already names which callees
those are and `is_certainly_a_failure` already reads it; asking the same question here is what keeps
two rules from disagreeing about one call. Without it `validate_key` returned a boxed error where
its signature promised a `Result`, and the compile gate said so.

The cause dropped from 16 to 12 and COVERAGE DID NOT MOVE. That is the sixth rule this session with
that shape, and the pattern is now the finding rather than a disappointment: the declarations this
unblocked call callees that refuse, so they refuse again one step later as CASCADE. Cascade is 134
declarations across all ten packages — the largest single category — and none of it resolves until
the roots do.

## R4w — `(T, bool)` is an option, and the gate could not tell success from absence

### The divergence that earns itself

Go has no sum type, so a function that may or may not produce a value returns both — the value and
a flag saying whether to look at it. The value it hands back when the flag is false is not an
answer; it is what the source had to put in a slot that could not be empty.

    fn safe_int(f: f64) -> Option<i64> {
        if f < -9_007_199_254_740_991.0 || f > 9_007_199_254_740_991.0 {
            return None;
        }
        Some(f as i64)
    }

THE PROOF IS THAT THE FAILING VALUE CARRIES NOTHING, and it is what separates this from a function
returning two real results. `gjson`'s `validstring` also answers `(int, bool)`, and on failure its
int is THE OFFSET IT STOPPED AT — information a caller can use and `None` would discard. So every
failing return must hand back a literal, and one computed value disqualifies the declaration.

The predicate accepts four of the corpus's twenty-six `(T, bool)` functions: `parseUint`,
`parseInt`, `safeInt`, `keyFloat`. Those are exactly the three a reviewer named plus one, and it
rejects `validstring` for exactly the reason the reviewer gave. Written before the emission was
wired, and measured against the reviewer's own list, which is what made it trustworthy.

First attempt accepted ZERO: `true` and `false` are PREDECLARED CONSTANTS in the source, not
literals, so they arrive as identifiers. Measuring before wiring is what caught it.

### The gate could not tell success from absence

Then the compile gate said `yaml` compiles, and `yaml` had emitted nothing at all.

A package whose port fails writes no modules, and AN EMPTY CRATE COMPILES PERFECTLY. Three of the
thirteen — `errors`, `gocache`, `yaml` — were passing that way, and every "13/13 compile" in this
session counted them. The real figure was 10. A gate that cannot tell success from absence measures
nothing, and this is the third instrument defect this session after `-A dead_code` and the
hand-assembled review bundle.

### Why those three emitted nothing

    snapshot admit digest mismatch: claimed `sha256:4158b9c2...`, computed `sha256:f958b64a...`

The extractor appends ` imports=source` to `build_config` when a corpus needed the weaker importer.
It did so in a `defer`, which runs after the return value is set — and therefore AFTER the line that
computes the digest over it. The snapshot shipped a `build_config` its own digest had never seen, so
every corpus needing the source importer failed admission with a mismatch it could not have caused.

Recorded before the digest now. And the three came back:

    errors    3 of 39 translated, compiles
    gocache   0 of 23 translated, emits nothing
    yaml      209 of 461 translated — 45.3%, the HIGHEST of any package in the corpus

`yaml` was the engine's best result and nothing could see it. It also carries eight compile errors
that had never been checked, and they are real: `nil` emitted as a bare identifier, a defined type
compared against an integer literal — `self.kind.clone() == 0`, which the source permits because a
defined type carries its underlying type's operations — and a conversion between two newtypes
without reaching through either. Those are the next work, and they exist because a gate that was
lying stopped lying.

## R4x — the defects the lying gate had been hiding

`yaml` came back from the digest fix carrying 207 translated declarations and eight compile errors
that had never been checked. All eight were real, and all eight are now fixed.

### `nil` is a question, not a name

    self.alias.clone() == nil

`nil` reached the output as a bare identifier, which names nothing. The source asks whether a
pointer holds anything and the target asks the OPTION that pointer became: `self.alias.is_none()`,
and `is_some` for `!=`.

ONLY A POINTER. The source's other nil-able kinds do not survive the question — a nil SLICE and an
empty one are different values there, `append` and equality both see the difference, and the
target's growable sequence has no state that is not a sequence. Answering with emptiness would be a
different question, so a slice or a map compared against nil refuses by name. That refusal took
`Node::IsZero` out of the emitted set, which is the correct answer for a function whose meaning the
target cannot carry.

### A conversion between two defined types wants the value, not the other wrapper

    YamlMappingStyleT(self.style)   // self.style is a YamlStyleT

The source allows converting between defined types wherever they share an underlying type, and
spells it as a conversion. The target's two newtypes are unrelated, so the constructor wants the
UNDERLYING value: `YamlMappingStyleT(self.style.0)`.

FIRST ATTEMPT PUT THIS IN `unwraps_newtype`, which the INDEX path shares — reaching through every
operand that happens to be a newtype is right for a conversion and wrong for an index, and it broke
`uuid`. The question belongs to the caller that needs the answer, so it is asked in `convert`.

### Reading through a wrapper does not need a copy of it

    YamlMappingStyleT(self.style.clone().0)

The operand was built for a position that OWNS, and this position reads through it. `unowned` --
written for Display exits in R4h -- already strips exactly that, so it is reused rather than
written a second time.

`yaml` compiles. TWELVE of thirteen packages now do, and `gocache` is the last: it translates 0 of
23 declarations, which is a coverage result and not a defect.

`yaml` is now the corpus's largest emitted crate by a wide margin -- 207 declarations against
`memberlist`'s 76 -- and every one of its defects existed because a gate that could not tell
compiling from empty had been reporting it green.

## R4y — a directory of Rust files is not a crate

The review bundle shipped no `Cargo.toml`. A reviewer handed it had to write one before anything
could be built, and counted its absence against the CODE — which was the harness's fault and not
the engine's, and is the third finding this script has produced about itself after the name
collision and the module inception.

It writes a manifest now, `unsafe_code = "forbid"` declared there rather than only in the crate
root, and the bundle builds with plain `cargo build`.

## R4z — buck2 is red on a stale generated file, and the fix is not in this lane

    Tests finished: Pass 29. Build failure 20.
    `third-party//:prettyplease-0.2` is not visible to `root//build/port-engine/core/rust-ir`

All twenty failures chain through one edge. `prettyplease` is a declared WORKSPACE dependency —
`Cargo.toml:475`, and `core/rust-ir/Cargo.toml` names it — and `third-party/BUCK` carries no PUBLIC
alias for it, where `quote` and `syn` each have one:

    alias(name = "quote", actual = ":quote-1", visibility = ["PUBLIC"])

So the generated file is STALE relative to Cargo rather than wrong. `third-party/BUCK` says
`@generated by reindeer — do not edit by hand` and names its own regeneration,
`scripts/ci/regen-third-party.sh`.

PRE-EXISTING: `prettyplease` was already in `rust-ir`'s buck deps at this session's first commit.
Not run here, and that is a scope decision rather than an omission — the regeneration rewrites an
eighteen-thousand-line file shared by the whole repository, this lane is scoped to
`build/port-engine/`, and ADR-0716 makes cargo the CI merge path with buck2 a local smoke. The
cargo suite is green and covers the same crates.

## R5a — review round 9, and the scaffolding my own rule was emitting

    round 8   blind DO_NOT_MERGE (9)   go-aware MERGE_WITH_CHANGES (9)
    round 9   blind DO_NOT_MERGE (10)  go-aware MERGE_WITH_CHANGES (9)

The Go-aware reviewer scored the previous round's six complaints as "roughly two fixed, three still
live, one too coarse", which is the first time either gate has acknowledged movement — and the
"too coarse" is worth more than the credit.

### What a reviewer caught that I had built

> `'step: { ... break 'step; }` is a no-op — the block ends immediately before `i += 1`, so
> `break 'step` and falling off the end are identical.

True, and it was my own rule producing it. R4u wraps an iteration in a labelled block so a
`continue` lands on the post-statement instead of jumping past it. That block earns its noise only
where there is something to SKIP. Where every step-break is already the last thing on its path,
leaving the block and falling out of it are the same, and a reader gets scaffolding instead of
logic.

    while i < data.len() as i64 {
        match data[i as usize] {
            b' ' | b'\t' | b'\n' | b'\r' => {}
            b':' => return (i.wrapping_add(1), true),
            _ => return (i, false),
        }
        i += 1;
    }

The block survives where it is doing work — `if digit { break 'step } break 'counted` genuinely
skips the loop-break — and three of six sites kept it.

### Where the two reviewers DISAGREE, and the Go-aware one is right

The blind reviewer verified a panic in `validnumber` — `i -= 1` at `i == 0` indexes at
`usize::MAX` — and called it a blocking defect. The Go-aware reviewer checked the upstream and
recorded the opposite: "validnumber's opening `i -= 1` is upstream's, and callers pass `i+1`". It
is a precondition of the source function, faithfully carried. The engine is not permitted to invent
a bounds check the source does not have; doing so is the redesign §10 rejects.

RECORDED RATHER THAN ACTED ON, because a reviewer who does not know the code is a port cannot tell
a carried precondition from an introduced one — which is exactly why the second gate exists.

### Declined, with the reason

The blanket "iota blocks should be enums" from round 8 is TOO COARSE and the Go-aware reviewer
withdrew part of it on inspection: `MethodTyp` is a BITMASK — `1, 2, 4 ... 1024`, ORed into `mALL`
— and an enum would be wrong; `Version` is genuinely open, its own `Display` returning
`BAD_VERSION_{n}` for `n > 15`. `NodeEventType`, `NodeTyp`, `EncryptionVersion` and `gjson::Type`
remain fair. A rule that turned every iota block into an enum would have made the port worse in two
places, and only reading the uses tells them apart.

### Still live, and named for the next phase

- `(T, bool)` in eight more functions. The reviewer traced the upstream call graph and found the
  failure index discarded by every caller — a WHOLE-PROGRAM fact. `spells_an_option` is local and
  conservative: it requires the failing return to hand back a literal, and these hand back the
  offset they stopped at. Extending it needs the call-graph proof, and for a PRIVATE function that
  proof is available inside the corpus.
- `wrapping_*` applied where nothing can overflow. The reviewer's sharpest point: three of the
  wrapping calls in `parse_uint`/`parse_int` are LOAD-BEARING because gjson relies on Go's wrap,
  and twenty-one on slice indices are not — "the signal that this wrap is deliberate has been
  destroyed by applying it everywhere".
- Go's `int` as `i64` for offsets, then `as usize` at the index. Both gates, twice each now.

## R5b — an index position is not only where a bare name stands

Both gates named the same thing twice: Go's `int` emitted as `i64` for offsets and lengths, then
cast back with `as usize` at every index. The blind reviewer traced a nineteen-digit slice index to
exactly that cast.

The engine already infers the target's index type — for loop counters, for parameters used only as
indices, for length constants. It was asking the question too narrowly in three places:

- an index OPERAND counted only where it was a bare name, so `buf[n-1]` read `n` as arithmetic
  rather than as an index;
- a SLICE BOUND was not an index position at all in one of the two counters, so `buf[:n]` did not
  count;
- a name UPDATING ITSELF counted as an ordinary read. `n = n - x` and `i++` do not observe a value's
  sign; they are the name moving, not a reader looking at it.

    fn pkcs7decode(buf: &[u8], _: usize) -> Vec<u8> {
        let mut n: usize = buf.len();
        let last = buf[n.wrapping_sub(1)];
        n = n.wrapping_sub(last as usize);
        buf[..n].to_vec()

    memberlist   46 `as usize` -> 1,  26 `as i64` -> 1
    yaml          all -> 0
    chi           all -> 0

Two ends had to be taught the same answer, which is the fourth time this session:

- the INDEX SITE stopped casting only for a bare proven name, so the moment the operand became
  `i + 1` it cast a `usize` to a `usize`. It now asks the same question of arithmetic over proven
  operands, which is what the operand actually is.
- an ASSIGNMENT's value carries conversions written for the SOURCE's integer width, and a place the
  engine proved is an index takes an index-typed value. `n = n.wrapping_sub(last as i64)` where `n`
  is a `usize` is those two ends disagreeing.

TWO SEPARATE COUNTERS were asking this question — `count_reads` for parameters and
`count_cursor_reads` for locals — and both had to be widened. That duplication is the smell it looks
like; they answer the same question about the same thing and should be one.

`gjson` is unmoved at 31 casts and that is understood rather than unexplained: its `valid*` family
takes `i` as a parameter AND RETURNS IT, so `i` cannot be an index type until the RESULT is one too,
and the result's type is part of a signature its callers read. That is the whole-program half of the
same rule.

## R5c — a slice of a borrowed parameter is the parameter's memory

    fn pkcs7decode(buf: &[u8], _: usize) -> &[u8] {
        let mut n: usize = buf.len();
        let last = buf[n.wrapping_sub(1)];
        n = n.wrapping_sub(last as usize);
        &buf[..n]
    }

The source's `return buf[:n]` reslices what it was given and copies nothing. The port allocated,
because a `[]byte` result maps to an owned sequence — on the decrypt path of every packet
`memberlist` receives, of a payload bounded at forty megabytes. Both gates named it.

EXACTLY ONE BORROWED PARAMETER is the condition, and it is what keeps this inside the doctrine's
refusal to infer lifetimes: with one reference input the target ELIDES the result's lifetime and
there is nothing left to infer. Two would need the engine to say which input the result comes from,
and it does not know. Every return must also be a slice of that same parameter — a body handing back
a built value on one path owns the result there, and no signature is right for both.

The two ends had to be told once more. `results` decided `&[T]` and `returns_owned_sequence` went on
adding the owning conversion, so the first emit was a `.to_vec()` under a signature that said `&[u8]`
— the fifth instance of that failure mode this session, and the reason the second end now READS the
first's answer instead of deriving its own.

Composed as a borrow of `[element]`, the way the pack already composes a borrowed sequence PARAMETER
under `borrowed_sequence_is_a_slice`, rather than by naming a container the source never had.

## R5d — what the `(T, bool)` extension actually needs, measured

The Go-aware gate's largest surviving blocker is eight more functions that should answer `Option<T>`.
`spells_an_option` accepts four and rejects these, and the rejection is CORRECT under what it can
see: it requires the failing return to hand back a LITERAL, and `gjson`'s `valid*` family hands back
the offset it stopped at, which is information `None` would discard.

The reviewer's counter-claim is that the offset is never READ, and the call graph says so:

    validpayload   call sites bind `_, ok`          <- the offset IS discarded, at both
    validany       `i, ok = validany(json, i)`      <- reassigned, used on the SUCCESS path
    validstring    `return validstring(data, i)`    <- forwarded whole, both results

So the shape is a fixpoint rather than a local test. `Option<T>` keeps the SUCCESS offset — that is
what `Some(i)` is — and loses only the FAILURE one, and the failure offset flows exclusively into
other failure returns until `validpayload`'s callers drop it with the blank.

WHAT THE RULE NEEDS, stated so it is not re-derived: a candidate's failure value is unobserved when
every call site either binds it to the blank, or forwards the call whole through `return f(..)`, or
binds it and reads it only in its own failing return — and the enclosing function is itself a
candidate whose failure value is unobserved. That is a least-fixpoint over the package call graph.

NOT BUILT HERE, deliberately. It is sound to want and easy to get subtly wrong — the third clause is
a reaching-definitions question, not a syntactic one — and a wrong answer converts a function whose
failure offset a caller does use, which is a silent behaviour change of exactly the kind the local
rule exists to refuse. It is tractable because every one of these functions is PRIVATE, so the
package call graph is the whole call graph; the snapshot already carries it.

## R5e — the mutex analysis was the wrong recommendation, and the measurement says so

I put the `sync.Mutex` lock-scope analysis to the operator three times as the decision worth taking
first, on the argument that it is an ANALYSIS rather than a preference and therefore supplies the
fact the pack says is missing. Measured, that recommendation does not survive.

Every struct in the corpus carrying a `sync` field:

    gocache::cache               1 RWMutex,  5 fields
    multierror::Group            1 Mutex + 1 WaitGroup, 3 fields
    memberlist::awareness        1 RWMutex,  4 fields
    memberlist::Keyring          1 Mutex,    2 fields
    memberlist::TransmitLimited  1 Mutex,    6 fields
    memberlist::NetTransport     1 WaitGroup
    memberlist::Memberlist       SEVEN mutexes, 33 fields

`Memberlist` is the central type of the phase-3 repository and it has SEVEN locks over
thirty-three fields. A single-mutex reshape does not touch it, and a seven-way one has to partition
the fields by which lock guards each — which is the harder half of the problem, not a corollary of
the easy half.

And the small candidates do not unlock either, because their methods do not release the lock the way
the analysis assumes:

    func (k *Keyring) UseKey(key []byte) error {
        k.l.Lock()
        defer k.l.Unlock()

`defer` is not modelled. 83 `DeferStmt` nodes reach the model as `unsupported` across the corpus,
234 `defer` statements in `memberlist` alone. So the mutex reshape would emit a struct nobody can
use: the declaration translates and every method that touches it still refuses.

THE GATEWAY IS `defer`, NOT THE MUTEX, and defer-for-a-lock has an exact and better Rust answer that
needs no decision at all — `mu.Lock(); defer mu.Unlock()` is `let _guard = mu.lock()`, which is RAII
and is what a Rust author writes. That is the "diverge where the target is better" case in its
purest form, and it is mechanism rather than policy.

Recorded as a correction rather than quietly re-ordered: the recommendation was made three times and
was wrong, and the reason it was wrong is that it was never measured against the bodies that would
have to use the reshaped struct.

## R5f — a forward is a forward whatever else the callee answers with

R4v recognised `return f(x)` as a forward only where the callee answered with the failure ALONE.
`uuid` writes it the other way round more often:

    func NewDCEPerson() (UUID, error) {
        return NewDCESecurity(Person, uint32(os.Getuid()))
    }

One operand standing for TWO results. The engine asked whether the callee's result tuple was exactly
one failure, found two, and fell through to demanding a proof that the operand was "certainly a
failure" — of a call that was never claiming to be one.

The last result is what matters and the rest come along. Arity needs no checking: a source that
expanded a call into a different number of results would not compile, so the shapes agree by
construction and the call IS this return.

Cause 20 -> 15 declarations across eight packages. Coverage unmoved, for the reason it has been
unmoved six times now — the callees these forward to refuse, so the caller refuses again as cascade.
Cascade is 277 declarations across all thirteen packages and is the whole remaining shape of the
problem.

## R5g — the mutex and `defer` are one build, not two

R5e corrected the recommendation from the mutex to `defer`. Measured, that correction was also only
half right, and the two are MUTUALLY DEPENDENT:

    memberlist   24 `defer func`, 9 `defer intv.Unlock`, 9 `defer e.mu.Unlock`
    gocache      2 `defer c.mu.RUnlock`, 1 `defer c.mu.Unlock`
    multierror   1 `defer g.mutex.Unlock`, 1 `defer g.wg.Done`

A `defer` releasing a lock needs a lock to release — the mutex's target form — and the mutex reshape
needs `defer` because that is how every one of these methods unlocks. Neither is a gateway to the
other; they are one unit of work, and building either alone emits something whose counterpart still
refuses.

What that unit would reach, and what it would not:

    Keyring, TransmitLimitedQueue, awareness, gocache::cache   one mutex each -- reachable
    memberlist::Memberlist                                     SEVEN mutexes over 33 fields -- not

The rest of `defer` — 44 deferred CLOSURES — needs the closure-ownership question and Go's defer
semantics on the panic path, which is a third thing again.

STATED PLAINLY: phase 3 as specified — `memberlist` as a working binary with goroutines, channels
and `select` — needs the mutex reshape including a seven-way field partition, `defer` with its panic
path, `WaitGroup`, the closure-ownership rule for spawned literals, a channel decision, `select`, and
timers. That is a body of work measured in weeks, not in rules. The engine's refusals name every
piece of it by now, which is what they are for.

## R5h — a refusal whose own text says nothing was lost is not a refusal

Chasing the error-interface satisfaction cause — 18 declarations across seven packages, the largest
rule-shaped root left — found that it is not a gap at all. The reason ends:

> The conformance is NOT lost ... The method becomes a display impl for the type.

And it does. `uuid` reports four of these and emits three `impl StdError` alongside the `Display`
each was folded into:

    impl fmt::Display for InvalidLengthError { ... write!(f, "invalid UUID length: {}", self.len) }
    impl StdError for InvalidLengthError {}

So the source's `Error() string` IS translated — into the pair of traits the target uses for the
same job — and what is recorded as refused is the SATISFACTION RECORD, which correctly does not
become a trait impl of its own.

That makes it a defect in the MEASUREMENT rather than in the engine, and the fourth instrument
defect this session after `-A dead_code`, the review bundle's missing manifest, and a gate that
called an empty crate compiled. Corpus-wide:

    translated  468
    refused     973   <- of which at least 18 are folded rather than lost

Coverage UNDERSTATES what the engine translates, and every coverage figure quoted in this session
carries that error. The fix is a category — a declaration whose meaning is emitted somewhere else is
neither translated-here nor refused, and `deferred` already exists for a related idea — but a survey
restructure is not something to start at the end of a long session, so this records the size and the
shape instead.

WHAT IT DOES NOT CHANGE: 18 of 973 is not the reason the blind gate says the crate is a skeleton.
The cascade is 277 and the decisions are still the roots. This corrects a number, not a conclusion.

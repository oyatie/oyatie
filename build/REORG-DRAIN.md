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

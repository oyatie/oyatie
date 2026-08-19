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

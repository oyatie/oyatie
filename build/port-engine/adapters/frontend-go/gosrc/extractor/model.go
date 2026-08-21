package main

// Snapshot model: the wire shape this extractor emits, and the closed vocabularies it may use.
//
// The vocabularies are named HERE and mirrored in the Rust front end, which refuses anything it
// has never heard of. Two lists that must agree are a hazard; two lists where one of them REFUSES
// on disagreement is a check.

const producerBootstrapGo = "bootstrap-go-packages-go-types"

// schemaVersion is the snapshot envelope version this extractor emits.
//
//	v0 — unit identity only
//	v1 — declaration tree, with types as flat spellings
//	v2 — declaration tree, with types as TREES
//
// v1 is not merely superseded, it is UNACCEPTABLE to the current engine: a v1 artifact cannot
// answer the questions v2 asks, and decoding one by treating each spelling as an opaque name
// would reintroduce exactly the flat-table resolution v2 exists to replace.
const schemaVersion = 2

// ---------------------------------------------------------------------------------
// Envelope
// ---------------------------------------------------------------------------------

type snapshot struct {
	SchemaVersion int    `json:"schema_version"`
	Language      string `json:"language"`
	// BuildConfig is the configuration the corpus was type-checked FOR, canonicalised.
	//
	// It is an input that changes what is extracted, and until it was recorded here it was the one
	// input that changed nothing observable: two extractions of one corpus at Go 1.21 and Go 1.24
	// produced byte-identical snapshots and the same digest. Go 1.22 rescoped the loop variable --
	// same syntax, different program -- so the engine could emit a different program with every
	// receipt axis holding, which is the exact failure the receipt exists to prevent.
	BuildConfig    string    `json:"build_config"`
	SnapshotDigest string    `json:"snapshot_digest"`
	Packages       []pkgNode `json:"packages"`
}

type pkgNode struct {
	UnitID       string `json:"unit_id"`
	Producer     string `json:"producer"`
	Declarations []node `json:"declarations"`
}

// node is one node of the declaration tree, and it is deliberately UNIFORM: a constant, a
// struct field, a function parameter and an interface method are all the same shape, and
// what distinguishes them is `kind` — a value, not a field name and not a variant.
//
// The engine's seam types are language-neutral (`LanguagePair` is data, so a second
// language pair is a second directory of rule data over one engine). A snapshot shaped as
// `fields`/`methods`/`params`/`results` would have pushed Go's declaration taxonomy into
// that seam and made the neutral API answer questions only Go asks. Here `kind`,
// `type`, and `flags` are opaque slugs the engine compares and never interprets; the rule
// pack's `captures` are what give them meaning.
type node struct {
	Kind  string    `json:"kind"`
	Name  string    `json:"name"`
	Type  *typeNode `json:"type,omitempty"`
	Flags []string  `json:"flags,omitempty"`
	// Attrs carries key->value facts that do not fit a set: a constant's value, and whatever a
	// later front end needs to record. Kept separate from Flags because the two answer different
	// questions — Flags is membership, Attrs is a value — and collapsing them would mean encoding
	// "exported" as "exported=1" and losing the distinction between an absent key and an empty one.
	Attrs    map[string]string `json:"attrs,omitempty"`
	Children []node            `json:"children,omitempty"`
}

// typeNode is one node of a type TREE. Same uniform shape as `node`: `kind` is a value, so a
// second source language needs a second rule pack rather than a second seam.
//
// `Package` is what makes a named type addressable. Without it a reference to another package's
// type is indistinguishable from a local one, and two packages declaring the same name are
// indistinguishable from each other — so a resolver silently picks one.
type typeNode struct {
	Kind    string      `json:"kind"`
	Name    string      `json:"name,omitempty"`
	Package string      `json:"package,omitempty"`
	Args    []*typeNode `json:"args,omitempty"`
}

// Type kinds. Part of the snapshot contract: the rule pack answers for each one.
const (
	typeBasic = "basic"
	typeNamed = "named"
	// typeNamedInterface is a named type whose underlying type is an interface. It carries the
	// same identity as typeNamed and is a separate kind because the target holds the two
	// differently: a struct is a value, and a trait has no size.
	typeNamedInterface = "named_interface"
	typePointer        = "pointer"
	typeSlice          = "slice"
	typeArray          = "array"
	typeMap            = "map"
	typeChan           = "chan"
	typeFunc           = "func"
	typeInterface      = "interface"
	typeStruct         = "struct"
	typeTuple          = "tuple"
	typeParam          = "type_param"
	// typeUnsupported keeps the model FAITHFUL where the translator is not: a type shape with no
	// node of its own is recorded as present and refused by name downstream, rather than dropped
	// into a spelling nobody can act on.
	typeUnsupported = "unsupported"
)

// Declaration kinds. These strings are the vocabulary the rule pack's `captures` select
// on, so they are part of the snapshot contract rather than an internal detail.
const (
	kindConst = "const"
	kindVar   = "var"
	// kindPackageInit is the package's `init` work, as one declaration carrying every body in
	// file order. Not in package scope and so invisible to the scope walk, which is how it used to
	// reach the model not at all. Distinct from kindInit, which is a `for` loop's init CLAUSE —
	// the two share a source keyword and nothing else.
	kindPackageInit = "package_init"
	kindFunc        = "func"
	kindStruct      = "struct"
	kindInterface   = "interface"
	kindAlias       = "alias"
	kindNamed       = "named"

	kindField  = "field"
	kindMethod = "method"
	// kindImplements is an observed interface satisfaction, hung on the concrete type that
	// satisfies. See satisfy.go for why it is observed rather than derived.
	kindImplements = "implements"
	// kindForeignSatisfaction is an observed satisfaction whose concrete type this corpus does
	// not declare, so there is nowhere to emit the impl. Recorded rather than dropped, and given
	// its own kind rather than the generic unsupported one: a kind broad enough to cover this
	// would be broad enough to swallow any package-scope construct the front end cannot model.
	kindForeignSatisfaction = "foreign_satisfaction"
	// kindEmbeds is an interface an interface embeds, which the target spells as a supertrait.
	kindEmbeds = "embeds"
	// kindPromoted is a method a type gains through EMBEDDING rather than declaration. The
	// target has no promotion, so what is implicit in the source becomes a forwarding method.
	kindPromoted = "promoted"

	kindParam  = "param"
	kindResult = "result"

	// Body vocabulary. A function body is a `body` node whose children are statements, and a
	// statement's children are expressions — the same uniform node all the way down.
	kindBody    = "body"
	kindClosure = "closure"
	kindCapture = "capture"
	kindBlock   = "block"
	kindReturn  = "return"
	// kindGo is a goroutine: `go f(x)`. Its child is the call.
	kindGo = "go"
	// kindSend is `ch <- v`. Children are the channel and the value, in that order.
	kindSend = "send"
	// kindSelect is `select { case ... }`. Children are its arms.
	kindSelect = "select"
	// kindCommClause is one arm of a select: a communication and a body, or just a body when the
	// arm is `default`.
	kindCommClause = "comm_clause"
	// kindComm holds an arm's communicating statement.
	kindComm = "comm"
	kindIf      = "if"
	kindThen    = "then"
	kindElse    = "else"
	kindLet     = "let"
	// kindLetTuple is a destructuring bind — `v, err := f()`. Its children are the names it
	// binds, in order, followed by the single expression they come from.
	kindLetTuple = "let_tuple"
	// kindBind is one name a destructuring bind introduces.
	kindBind = "bind"
	// kindValue is the expression a destructuring bind takes its values from.
	kindValue = "value"
	// kindAssignTuple is a PARALLEL assignment — `a, b = b, a` and `x, err = f()`. Its children
	// are the places it assigns, in order, then the values it assigns from. Distinct from
	// kindLetTuple because those are places rather than new names: nothing is introduced.
	kindAssignTuple = "assign_tuple"
	// kindPlace is one place a parallel assignment writes to.
	kindPlace    = "place"
	kindExprStmt = "expr_stmt"

	kindLiteral  = "literal"
	kindIdent    = "ident"
	kindBinary   = "binary"
	kindUnary    = "unary"
	kindParen    = "paren"
	kindSelector = "selector"
	kindCall     = "call"
	// kindConvert is a type CONVERSION, which the source spells exactly like a call. Its own
	// kind because the target has three forms for it and none is a function call.
	kindConvert = "convert"
	kindIndex   = "index"
	// kindSlice is `s[lo:hi]`. The bounds are children in a fixed order and an ABSENT bound is
	// recorded as an empty node rather than omitted, because `s[:hi]` and `s[lo:]` would
	// otherwise be the same two-child shape meaning different things.
	kindSlice = "slice"
	// kindAbsent is a bound a slice expression left out.
	kindAbsent    = "absent"
	kindComposite = "composite"
	kindKeyed     = "keyed"
	kindZero      = "zero"

	// kindType is a TYPE standing where an expression would. A few of the source's builtins take
	// one -- `make([]byte, 0, n)` names what to allocate -- and walking it as an expression
	// recorded the type syntax as an unsupported node.
	kindType = "type"

	kindAssign   = "assign"
	kindFor      = "for"
	kindCond     = "cond"
	kindPost     = "post"
	kindInit     = "init"
	kindRange    = "range"
	kindSwitch   = "switch"
	kindCase     = "case"
	kindBreak    = "break"
	kindContinue = "continue"
	kindIncDec   = "incdec"

	// kindUnsupported is how the extractor stays FAITHFUL while the engine stays fail-closed.
	// The snapshot is a model of the source, so a construct the translator cannot yet handle is
	// recorded as present rather than omitted; the transform then refuses it BY NAME. Dropping it
	// here instead would make an untranslatable function look like an empty one.
	kindUnsupported = "unsupported"
)

// Flags. Sorted on emit so the set has one spelling.
const (
	flagExported = "exported"
	flagVariadic = "variadic"
	flagEmbedded = "embedded"
	// flagPointerReceiver records that a method is bound through a pointer receiver. The engine
	// refuses to translate one rather than guess an aliasing mode, so this flag has to be
	// extracted for that refusal to be possible at all — dropping it would silently turn a
	// mutating method into a read-only one.
	flagPointerReceiver = "pointer_receiver"

	// flagReread records that the body reads this binding MORE THAN ONCE.
	//
	// The source copies a value on every read and the target moves it, so a second read of a
	// non-copying binding is a use after move. Recorded as a count-based FACT rather than as a
	// decision: whether the type copies is the pack's answer, and the two halves belong on
	// different sides of the seam.
	flagReread = "reread"
	// flagInferred marks a binding whose type the SOURCE did not write. The type is recorded
	// either way because the engine needs it -- whether the binding has a drop to delay decides
	// whether the block that scopes it is necessary -- and the flag is what says the target must
	// not ANNOTATE it, since an annotation the source never had is noise on every short
	// declaration in every body.
	flagInferred = "inferred"

	// flagUnread records that the body never mentions the parameter at all.
	//
	// Ordinary in the source and a WARNING in the target, which is a difference the port has to
	// answer for: an unused parameter is how the source satisfies an interface it does not need
	// every argument of. The target says the same thing with a leading underscore, which keeps the
	// signature identical — a parameter's name is not part of a function's type — and states the
	// intent the source could only leave implicit.
	//
	// Only claimed where a body exists. An interface method has none, and "not read" would then
	// mean "not looked at" rather than "not used".
	flagUnread = "unread"

	// flagRebound records that the body assigns to the binding's OWN name. Distinct from
	// flagMutated, which on a parameter means the body writes through the pointer and is a claim
	// about the caller's value: rebinding the callee's copy is the opposite claim, and one flag
	// carrying both would make every rebound parameter demand an exclusive borrow.
	flagRebound = "rebound"

	// flagSpread records that a call passes its last argument's ELEMENTS rather than the argument
	// itself -- what the source writes `f(xs...)`. Nothing else in the tree distinguishes the two,
	// and they mean different things to the same callee.
	// flagReassigned records that the enclosing body writes this captured variable somewhere --
	// not necessarily inside the literal. See the comment at its only producer: it is what decides
	// whether a target closure may own its captures instead of sharing them.
	flagReassigned = "reassigned"

	flagSpread = "spread"

	// flagInitWritten records that EVERY write to this package variable is in the package
	// initialiser. Such a variable is computed once before anything runs and never changes after,
	// which is not the mutable global the write flag alone describes -- and the two need different
	// target forms, so they need different facts. go/types omits `init` from package scope, so this
	// can only be observed by walking the declarations.
	flagInitWritten = "init_written"

	// flagUnsafeLayoutOnly records that EVERY reference to this type sits inside the source's
	// `unsafe.Pointer` escape hatch, so what it describes is the source runtime's memory layout
	// rather than a value. The target does not share that layout, and in a crate that denies
	// `unsafe` the ported struct could never mean what it meant. See unsafeuse.go.
	flagUnsafeLayoutOnly = "unsafe_layout_only"

	// Ownership facts, observed intra-procedurally. See ownershipFacts for what each means and
	// why the third one exists.
	flagMutated       = "mutated"
	flagEscapes       = "escapes"
	flagEffectUnknown = "effect_unknown"
)

// Attribute keys.
const (
	// Receiver modes a trait method can bind, carried by attrReceiver. Derived from the observed
	// implementors rather than declared, because a source interface does not say.
	receiverShared    = "shared"
	receiverExclusive = "exclusive"

	// attrDestination names where a function literal GOES, when that is a position outliving the frame
	// it is written in. Absent when it is not. The value is the destination, not a decision about
	// ownership: what the target does with it belongs to the transform.
	attrDefault = "default"
	attrDestination = "destination"
	// destinationReturn is the destination of a literal among a `return`'s operands.
	destinationReturn = "return"
	// destinationGo is the destination of a literal a goroutine starts.
	destinationGo = "go"
	// attrOp is a binary or unary operator, spelled as Go source.
	attrOp = "op"
	// attrGoNode names the Go AST node an `unsupported` placeholder stands for, so a refusal can
	// say what it refused rather than only that it refused.
	attrGoNode = "go_node"
	// attrReadCount is how many times the enclosing body reads this binding.
	//
	// Present only where that is more than one. A reader can MOVE the value when nothing reads it
	// afterwards, and comparing this total against the reads inside one construction is how the
	// last read is found without a liveness pass.
	attrReadCount = "read_count"
	// attrRef records what an identifier resolves to — a parameter, a constant, a function, a
	// local. Rust cases each of those differently, and an identifier alone cannot say which it is:
	// rendering a reference to `MaxRetries` as `max_retries` would be a dangling name, not a
	// style choice. go/types knows the answer, so it is recorded here rather than guessed there.
	attrRef = "ref"
	// attrPackagePath holds a package identifier's IMPORT PATH, so a rule keying on a
	// package-qualified call keys on the identity rather than on a local alias.
	attrPackagePath = "package_path"
	// attrDoc is the declaration's documentation block, newline-separated. Recorded because the
	// target emits it: dropping it here is a silent loss of everything the source explained about
	// itself, and no downstream check looks for prose that is simply absent.
	attrDoc = "doc"
	// attrReceiver is the receiver a TRAIT method binds, derived from the observed implementors.
	// A Go interface says nothing about it, so this is the one answer the source cannot give
	// directly and the corpus can.
	attrReceiver = "receiver"
	// attrStructTag is a struct field's raw Go tag. See decls.go: a tag names the field's wire
	// identity, so a port that drops it and cases the name changes the format while compiling.
	attrStructTag = "struct_tag"
	// attrVia is the dotted FIELD PATH a promoted method is reached through. The target has no
	// method promotion, so the forwarding method has to name the field it forwards to.
	attrVia = "via"
	// attrCallee is the package-qualified IDENTITY of what a call resolves to. Recorded because
	// a rule keyed on the callee's spelling would answer for anything that shares its name.
	// EMPTY for a method, which has no package-path name — see attrCalleeKind.
	attrCallee = "callee"
	// attrCalleeKind distinguishes a call through a RECEIVER from a call to a free function. The
	// source spells `value.Method()` and `package.Function()` identically and the target does not,
	// so deciding by syntax emits a method call on a package name.
	attrCalleeKind = "callee_kind"
	// calleeKindMethod is the one value attrCalleeKind takes; its absence means a free function.
	calleeKindMethod = "method"
	// attrInterface is the package-qualified identity of the interface a satisfaction satisfies.
	// Structured rather than folded into attrGoNode's sentence: one concrete type may satisfy
	// several interfaces, and those facts are only distinguishable if the interface is a field.
	attrInterface = "interface"
	// attrBundle marks a satisfaction whose interface declares no method of its own and embeds at
	// least one. The source satisfies such an interface structurally, so the target says it once
	// with a blanket impl; without this the emitted crate carries both that impl and a per-type
	// one, which is a coherence conflict rather than a redundancy.
	attrBundle = "bundle"
	// attrSite records HOW an interface satisfaction was observed. A declared assertion is
	// compile-checked by Go; a flow-derived one is this extractor's inference. An impl emitted
	// from either looks identical, so a reviewer needs the distinction recorded rather than
	// reconstructed.
	attrSite = "site"
	// attrValue is a constant's value, spelled as Go source. It is deliberately the SOURCE
	// spelling rather than a normalized number: the engine emits Rust that must parse, and a
	// Go literal that is not also a valid Rust literal has to fail loudly at the syn parse
	// rather than be silently rounded into something that compiles and means something else.
	attrValue = "value"
)

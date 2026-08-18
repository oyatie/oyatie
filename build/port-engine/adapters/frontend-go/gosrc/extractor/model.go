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
	SchemaVersion  int       `json:"schema_version"`
	Language       string    `json:"language"`
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
	kindConst     = "const"
	kindVar       = "var"
	kindFunc      = "func"
	kindStruct    = "struct"
	kindInterface = "interface"
	kindAlias     = "alias"
	kindNamed     = "named"

	kindField  = "field"
	kindMethod = "method"
	// kindImplements is an observed interface satisfaction, hung on the concrete type that
	// satisfies. See satisfy.go for why it is observed rather than derived.
	kindImplements = "implements"
	// kindEmbeds is an interface an interface embeds, which the target spells as a supertrait.
	kindEmbeds = "embeds"
	// kindPromoted is a method a type gains through EMBEDDING rather than declaration. The
	// target has no promotion, so what is implicit in the source becomes a forwarding method.
	kindPromoted = "promoted"

	kindParam  = "param"
	kindResult = "result"

	// Body vocabulary. A function body is a `body` node whose children are statements, and a
	// statement's children are expressions — the same uniform node all the way down.
	kindBody     = "body"
	kindBlock    = "block"
	kindReturn   = "return"
	kindIf       = "if"
	kindThen     = "then"
	kindElse     = "else"
	kindLet      = "let"
	kindExprStmt = "expr_stmt"

	kindLiteral   = "literal"
	kindIdent     = "ident"
	kindBinary    = "binary"
	kindUnary     = "unary"
	kindParen     = "paren"
	kindSelector  = "selector"
	kindCall      = "call"
	kindIndex     = "index"
	kindComposite = "composite"
	kindKeyed     = "keyed"
	kindZero      = "zero"

	kindAssign = "assign"
	kindFor    = "for"
	kindCond   = "cond"
	kindPost   = "post"
	kindInit   = "init"
	kindRange  = "range"
	kindSwitch = "switch"
	kindCase   = "case"
	kindBreak  = "break"

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

	// attrOp is a binary or unary operator, spelled as Go source.
	attrOp = "op"
	// attrGoNode names the Go AST node an `unsupported` placeholder stands for, so a refusal can
	// say what it refused rather than only that it refused.
	attrGoNode = "go_node"
	// attrRef records what an identifier resolves to — a parameter, a constant, a function, a
	// local. Rust cases each of those differently, and an identifier alone cannot say which it is:
	// rendering a reference to `MaxRetries` as `max_retries` would be a dangling name, not a
	// style choice. go/types knows the answer, so it is recorded here rather than guessed there.
	attrRef = "ref"
	// attrDoc is the declaration's documentation block, newline-separated. Recorded because the
	// target emits it: dropping it here is a silent loss of everything the source explained about
	// itself, and no downstream check looks for prose that is simply absent.
	attrDoc = "doc"
	// attrReceiver is the receiver a TRAIT method binds, derived from the observed implementors.
	// A Go interface says nothing about it, so this is the one answer the source cannot give
	// directly and the corpus can.
	attrReceiver = "receiver"
	// attrVia is the dotted FIELD PATH a promoted method is reached through. The target has no
	// method promotion, so the forwarding method has to name the field it forwards to.
	attrVia = "via"
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

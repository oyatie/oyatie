//! Closed vocabularies the Go snapshot may use.
//!
//! CLOSED, and closed HERE rather than in `port-engine-api`, on purpose. The neutral seam treats
//! `kind`, `flags` and attribute keys as opaque slugs because a second language pair must not need
//! a second seam. This adapter is the Go half, so this is exactly where Go's taxonomy is allowed
//! to be named — and where an extractor emitting something the engine has never heard of gets
//! refused instead of translated into silence.

/// Canonical bootstrap extractor identity (ADR-0638 D3).
pub const PRODUCER_BOOTSTRAP_GO: &str = "bootstrap-go-packages-go-types";

/// Owned Rust front-end producer identity (authorized only after W2 equivalence).
pub const PRODUCER_OWNED_RUST: &str = "owned-rust-go-front-end";

/// Envelope version carrying unit identity only.
pub const SCHEMA_VERSION_IDENTITY_ONLY: u32 = 0;

/// Envelope version carrying the declaration tree with types as flat spellings.
///
/// NOT ACCEPTED. A v1 artifact cannot answer the questions v2 asks — it has no type structure and
/// no package qualification — and decoding one by treating each spelling as an opaque name would
/// reintroduce exactly the flat-table resolution v2 replaced. Refusing it names the fix
/// (regenerate) instead of half-answering.
pub const SCHEMA_VERSION_FLAT_TYPES: u32 = 1;

/// Envelope version carrying the declaration tree with types as TREES.
pub const SCHEMA_VERSION_DECLARATIONS: u32 = 2;

/// The closed type-kind vocabulary. Closed for the same reason the declaration kinds are: a kind
/// the engine has never heard of is a kind no rule will ever answer for, and accepting it would
/// let a type resolve to nothing without anyone being told.
pub const KNOWN_TYPE_KINDS: &[&str] = &[
    "array",
    "basic",
    "chan",
    "func",
    "interface",
    "map",
    "named",
    // A named type whose underlying type is an interface. Same identity as `named`, separate kind
    // because the target holds the two differently: a struct is a value and a trait has no size,
    // so a trait reaches a position as a reference, a box or a generic parameter — and which of
    // those is an ownership decision the pack makes rather than a spelling the engine infers.
    "named_interface",
    "pointer",
    "slice",
    "struct",
    "tuple",
    "type_param",
    // A type shape with no node of its own is RECORDED rather than dropped, and refused by name
    // downstream. Dropping it would make an untranslatable type look like an absent one.
    "unsupported",
];

/// Declaration kinds this Go adapter admits, at package scope.
///
/// CLOSED, and the closure lives here rather than in `port-engine-api` on purpose. The neutral
/// seam treats `kind` as an opaque slug because a second language pair must not need a second
/// seam. This adapter is the Go half, so this is exactly where Go's declaration taxonomy is
/// allowed to be named — and where an extractor that emits a kind the engine has never heard of
/// gets refused instead of translated into silence.
pub const KNOWN_DECLARATION_KINDS: &[&str] = &[
    "alias",
    "const",
    // An observed interface satisfaction whose concrete type this corpus does not declare, so
    // there is nowhere to emit the impl. Admitted because the front end RECORDS it rather than
    // dropping it — a satisfaction the engine cannot host must be distinguishable from one that
    // does not exist — and deferred by the pack, which is where the decision belongs.
    "foreign_satisfaction",
    "func",
    // The package's `init` work. The source keeps it out of package scope — it is not addressable
    // and several may exist — so it reached the model nowhere until it was collected explicitly. A
    // construct that is neither translated nor refused is the one outcome this engine cannot answer
    // for, so it is admitted here and deferred by the pack.
    "package_init",
    "interface",
    "named",
    "struct",
    "var",
];

/// Kinds whose own `name` is NOT a name the enclosing package binds.
///
/// The namespace rule — one name means one thing — is a fact about Go's package scope and stays
/// exactly as strict for everything that enters it. A foreign satisfaction does not enter it: the
/// name is a type ANOTHER package declares, and the entry is an observation ABOUT that type rather
/// than a declaration OF it. `os.File` satisfying both `io.Reader` and `io.Writer` is two facts
/// about one foreign type, and reading them as two declarations of one name rejected an entire
/// snapshot over something Go permits everywhere.
///
/// Closed, and here rather than as an exemption at the call site: a kind that stops binding a name
/// has to say so where the reason is written down.
pub const NON_BINDING_DECLARATION_KINDS: &[&str] = &["foreign_satisfaction"];

/// The kinds whose children form a NAMESPACE — a scope in which one name means one thing.
///
/// Package scope, a struct's fields, a signature's parameters. Below these the tree is SYNTAX, and
/// syntax repeats names freely: `c.total + other.total` has two sibling nodes called `total` and
/// both are correct.
pub const NAMESPACE_KINDS: &[&str] = &[
    "alias",
    "const",
    "func",
    "interface",
    "method",
    "named",
    "struct",
    "var",
];

/// Declaration kinds admitted below package scope, as children of a declaration.
///
/// One flat list rather than a per-level one. The body vocabulary nests arbitrarily — a `binary`
/// inside a `return` inside a `then` inside an `if` — so a level-indexed grammar would have to
/// enumerate the nesting rules of the source language here, in the adapter that is supposed to
/// carry only its taxonomy. The precision this gives up is that a `param` could nominally contain
/// a `field`; the precision it keeps is the one that matters, which is that no PACKAGE-scope kind
/// can appear as a member.
pub const KNOWN_MEMBER_KINDS: &[&str] = &[
    "assign",
    "binary",
    "break",
    "call",
    "capture",
    "case",
    // A function literal. Its captures are recorded as `capture` members rather than left to be
    // rediscovered, because which identifiers inside a literal are captures is a SCOPING question
    // and the transform receives names rather than objects.
    "closure",
    "continue",
    "composite",
    "convert",
    "for",
    // An interface an interface EMBEDS. The target has no embedding, so this becomes a
    // supertrait — which is a requirement rather than a copy of the method set, and is why it is
    // recorded as a relation rather than flattened into the outer interface's methods.
    "embeds",
    // A method a type gains through EMBEDDING rather than declaration. The target has no
    // promotion, so what is implicit in the source becomes a forwarding method — and recording it
    // is what closes `census/interfaces.md` §11 item 7, where 2,747 types have method sets larger
    // than the census could measure.
    "promoted",
    // An observed interface satisfaction, hung on the concrete type that satisfies it. It is a
    // MEMBER kind rather than a declaration kind because the impl belongs to the type: emitting it
    // anywhere else would need the orphan rule answered by the front end, which is a target
    // question the source cannot see.
    "implements",
    "incdec",
    "index",
    "init",
    "keyed",
    "over",
    "patterns",
    "post",
    "range",
    "selector",
    "slice",
    "absent",
    "switch",
    "tag",
    "block",
    "body",
    "cond",
    "else",
    "expr_stmt",
    "field",
    "ident",
    "if",
    // A destructuring bind and its two child kinds. Recorded rather than refused because it is
    // the shape every fallible call in the source has, and a rule cannot fire on a shape the
    // snapshot never carries.
    "bind",
    "let",
    "let_tuple",
    // A PARALLEL assignment and the place kind it carries. Distinct from a destructuring bind
    // because these are places rather than new names: nothing is introduced, and what the source
    // guarantees is that every operand on both sides is evaluated BEFORE any of them is assigned.
    "assign_tuple",
    "place",
    "literal",
    "method",
    "paren",
    "param",
    "result",
    "return",
    "then",
    // A TYPE standing where an expression would. A few of the source's builtins take one —
    // `make([]byte, 0, n)` names what to allocate — and walking it as an expression recorded the
    // type syntax as an unsupported node, which refused every declaration that allocates.
    "type",
    "unary",
    "value",
    // A field a struct literal LEFT OUT, carrying the type whose zero fills it. Recorded
    // rather than omitted because the target needs every field named, and an absent entry
    // would be indistinguishable from a field the front end failed to see.
    "zero",
    // `unsupported` is how the snapshot stays a faithful model of the source while the engine
    // stays fail-closed: a construct the translator cannot handle is RECORDED as present, and
    // refused by name at transform. Omitting it would make an untranslatable function
    // indistinguishable from an empty one.
    "unsupported",
];

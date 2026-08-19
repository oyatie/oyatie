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
    "case",
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
    "literal",
    "method",
    "paren",
    "param",
    "result",
    "return",
    "then",
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

/// The closed flag vocabulary. Same argument as [`KNOWN_DECLARATION_KINDS`]: a flag the engine
/// does not know is a flag nothing will ever select on, and accepting it would let a misspelled
/// `exported` silently unexport a declaration.
pub const KNOWN_FLAGS: &[&str] = &[
    "effect_unknown",
    "embedded",
    "escapes",
    "exported",
    "mutated",
    "pointer_receiver",
    // The body assigns to the binding's OWN name. Distinct from `mutated`, which on a parameter
    // means the body writes THROUGH the pointer and is a claim about the caller's value; rebinding
    // the callee's copy is the opposite claim, and one flag carrying both would make every
    // rebound parameter demand an exclusive borrow.
    "rebound",
    // The body reads this binding more than once. The source copies on read and the target moves,
    // so a second read of a non-copying binding is a use after move.
    "reread",
    // The body never mentions the parameter. Ordinary in the source and a warning in the target,
    // which the leading underscore answers without changing the signature.
    "unread",
    "variadic",
];

/// The closed attribute-key vocabulary, closed for the same reason as the flags.
pub const KNOWN_ATTR_KEYS: &[&str] = &[
    ATTR_CALLEE,
    ATTR_CALLEE_KIND,
    ATTR_DOC,
    ATTR_GO_NODE,
    ATTR_INTERFACE,
    ATTR_LIT_KIND,
    ATTR_OP,
    ATTR_RANGE_KEY,
    ATTR_RANGE_VALUE,
    ATTR_RECEIVER,
    ATTR_REF,
    ATTR_SITE,
    ATTR_VALUE,
    ATTR_VIA,
];

/// Attribute key holding the dotted FIELD PATH a promoted method is reached through.
///
/// The target has no method promotion, so a forwarding method has to name the field it forwards to.
pub const ATTR_VIA: &str = "via";

/// Attribute keys holding the names a `range` loop binds.
///
/// Admitted late, and found by surveying a real package rather than by reading the extractor: the
/// fixture corpus reaches the range loop through a shape that binds only the value, so the key
/// attribute was emitted and never decoded. A closed vocabulary is only a check if the thing it
/// closes over is exercised.
pub const ATTR_RANGE_KEY: &str = "key";
/// See [`ATTR_RANGE_KEY`].
pub const ATTR_RANGE_VALUE: &str = "value";

/// Attribute key holding the package-qualified IDENTITY of what a call resolves to.
///
/// The identity rather than the spelling, because a rule keyed on text would answer for a local
/// variable that shares a package's name.
pub const ATTR_CALLEE: &str = "callee";

/// Attribute key distinguishing a call through a RECEIVER from a call to a free function.
///
/// The source spells `value.Method()` and `package.Function()` identically; the target does not.
/// Only the type-checker can tell which name is a package, so the front end records it rather than
/// leaving the transform to guess from syntax.
pub const ATTR_CALLEE_KIND: &str = "callee_kind";

/// Attribute key holding the receiver a TRAIT method binds, derived from its observed
/// implementors.
///
/// A source interface says nothing about receiver mode, so this is the one answer the declaration
/// cannot give and the corpus can. Its absence means nothing was observed to implement the
/// interface, and the pack's declared decision answers instead.
pub const ATTR_RECEIVER: &str = "receiver";

/// Attribute key recording HOW an interface satisfaction was observed.
///
/// A declared assertion is compile-checked by the source language; a flow-derived one is the front
/// end's inference. An impl emitted from either looks identical, so the distinction is recorded
/// rather than left to be reconstructed.
pub const ATTR_SITE: &str = "site";

/// Attribute key holding a constant's or literal's value, spelled as source.
pub const ATTR_VALUE: &str = "value";

/// Attribute key holding a binary or unary operator, spelled as source.
pub const ATTR_OP: &str = "op";

/// Attribute key naming the source AST node an `unsupported` placeholder stands for, so a refusal
/// can say WHAT it refused rather than only that it refused.
pub const ATTR_GO_NODE: &str = "go_node";

/// Attribute key naming the INTERFACE a foreign satisfaction satisfies.
///
/// Structured rather than folded into [`ATTR_GO_NODE`]'s sentence. One concrete type may satisfy
/// several interfaces, and while the identity lived only in prose those facts differed nowhere a
/// rule could read — `os.File` satisfying `io.Reader` and `os.File` satisfying `io.Writer` were
/// the same entry twice.
pub const ATTR_INTERFACE: &str = "interface";

/// Attribute key classifying what an identifier resolves to — a constant, a function, a local.
/// The target cases each differently, and the identifier alone cannot say which it is.
pub const ATTR_REF: &str = "ref";

/// Attribute key recording a literal's lexical class.
pub const ATTR_LIT_KIND: &str = "lit_kind";

/// Attribute key carrying a declaration's documentation block, newline-separated.
///
/// Recorded because the target emits it. Dropping documentation is a SILENT loss — coverage
/// proves every declaration was translated, not that everything about a declaration survived —
/// and no downstream check looks for prose that is simply absent.
pub const ATTR_DOC: &str = "doc";

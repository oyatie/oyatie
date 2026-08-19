//! The FLAGS and ATTRIBUTE KEYS a Go snapshot may carry, closed the same way the kinds are.
//!
//! Split from `vocabulary.rs` because they answer a different question. A KIND says what a node
//! IS; a flag and an attribute say what was OBSERVED about it — that a binding is written again,
//! that a parameter is variadic, that an identifier resolves to a constant. The engine's
//! decisions are made from these facts, so an extractor emitting one the engine has never heard
//! of has to be refused rather than have its observation dropped in silence.

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
    "inferred",
    // EVERY write to this package variable is in the package initialiser. Distinct from `rebound`,
    // which says only that some write exists: a variable the initialiser alone writes is computed
    // once before anything runs and never changes after, so it has no synchronization question at
    // all — and the two need different target forms, so they cannot share one flag.
    "init_written",
    "reread",
    // The body never mentions the parameter. Ordinary in the source and a warning in the target,
    // which the leading underscore answers without changing the signature.
    "unread",
    "variadic",
];

/// The closed attribute-key vocabulary, closed for the same reason as the flags.
pub const KNOWN_ATTR_KEYS: &[&str] = &[
    ATTR_BUNDLE,
    ATTR_CALLEE,
    ATTR_CALLEE_KIND,
    ATTR_DOC,
    ATTR_GO_NODE,
    ATTR_INTERFACE,
    ATTR_LIT_KIND,
    ATTR_OP,
    ATTR_RANGE_KEY,
    ATTR_READ_COUNT,
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
/// Attribute key marking a satisfaction whose interface is a pure SUPERTRAIT BUNDLE.
///
/// The interface declares no method of its own and embeds at least one, so the source satisfies it
/// structurally — every type with the embedded method sets has it. The target says that once with a
/// blanket impl, and a per-type impl beside one is a coherence conflict rather than a redundancy.
pub const ATTR_BUNDLE: &str = "bundle";

pub const ATTR_SITE: &str = "site";

/// Attribute key holding a constant's or literal's value, spelled as source.
pub const ATTR_VALUE: &str = "value";

/// Attribute key holding how many times the enclosing body reads a binding.
///
/// Present only where that is more than one. A read can MOVE the value when nothing reads it
/// afterwards, and comparing this total against the reads inside one construction is how the last
/// read is found without a liveness pass.
pub const ATTR_READ_COUNT: &str = "read_count";

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

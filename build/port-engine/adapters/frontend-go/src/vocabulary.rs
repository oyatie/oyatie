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

/// Envelope version carrying the declaration tree.
pub const SCHEMA_VERSION_DECLARATIONS: u32 = 1;

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
    "func",
    "interface",
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
    "binary",
    "block",
    "body",
    "cond",
    "else",
    "expr_stmt",
    "field",
    "ident",
    "if",
    "let",
    "literal",
    "method",
    "paren",
    "param",
    "result",
    "return",
    "then",
    "unary",
    // `unsupported` is how the snapshot stays a faithful model of the source while the engine
    // stays fail-closed: a construct the translator cannot handle is RECORDED as present, and
    // refused by name at transform. Omitting it would make an untranslatable function
    // indistinguishable from an empty one.
    "unsupported",
];

/// The closed flag vocabulary. Same argument as [`KNOWN_DECLARATION_KINDS`]: a flag the engine
/// does not know is a flag nothing will ever select on, and accepting it would let a misspelled
/// `exported` silently unexport a declaration.
pub const KNOWN_FLAGS: &[&str] = &["embedded", "exported", "pointer_receiver", "variadic"];

/// The closed attribute-key vocabulary, closed for the same reason as the flags.
pub const KNOWN_ATTR_KEYS: &[&str] = &[ATTR_GO_NODE, ATTR_LIT_KIND, ATTR_OP, ATTR_REF, ATTR_VALUE];

/// Attribute key holding a constant's or literal's value, spelled as source.
pub const ATTR_VALUE: &str = "value";

/// Attribute key holding a binary or unary operator, spelled as source.
pub const ATTR_OP: &str = "op";

/// Attribute key naming the source AST node an `unsupported` placeholder stands for, so a refusal
/// can say WHAT it refused rather than only that it refused.
pub const ATTR_GO_NODE: &str = "go_node";

/// Attribute key classifying what an identifier resolves to — a constant, a function, a local.
/// The target cases each differently, and the identifier alone cannot say which it is.
pub const ATTR_REF: &str = "ref";

/// Attribute key recording a literal's lexical class.
pub const ATTR_LIT_KIND: &str = "lit_kind";

//! The strings the pack and the front end agreed on.
//!
//! Opaque to this face: it compares them and never interprets them. `struct` reaches here as a
//! kind some rule chose to capture, not as a concept.

/// Unit-level construction: emit one empty region named for the unit.
pub const CONSTRUCTION_PASS_THROUGH: &str = "pass_through";
/// Unit-level construction: emit a minimal empty fn region for fixture coupling.
pub const CONSTRUCTION_EMPTY_CANARY: &str = "empty_canary";

/// Declaration-level construction: a constant with a declared type and value.
pub const CONSTRUCTION_RUST_CONST: &str = "rust_const";
/// Declaration-level construction: a transparent type alias.
pub const CONSTRUCTION_RUST_TYPE_ALIAS: &str = "rust_type_alias";
/// Declaration-level construction: a single-field tuple struct over the underlying type.
pub const CONSTRUCTION_RUST_NEWTYPE: &str = "rust_newtype";
/// Declaration-level construction: a struct with fields, plus an `impl` block for its methods.
pub const CONSTRUCTION_RUST_STRUCT: &str = "rust_struct";
/// Declaration-level construction: a trait with one signature per method.
pub const CONSTRUCTION_RUST_TRAIT: &str = "rust_trait";
/// Declaration-level construction: a free function, signature only.
pub const CONSTRUCTION_RUST_FN: &str = "rust_fn";
/// Declaration-level construction: a free function with its body translated.
///
/// A SEPARATE construction rather than a smarter [`CONSTRUCTION_RUST_FN`]. One construction that
/// translated a body when it could and fell back to a stub when it could not would degrade
/// silently: the emitted crate would compile either way, and nothing downstream could tell a
/// translated function from an abandoned one. Two constructions make the pack say which it wants,
/// and asking for a body it cannot have is a refusal.
pub const CONSTRUCTION_RUST_FN_BODY: &str = "rust_fn_body";

/// Precondition: the planned unit must exist in the source model.
pub const PRECONDITION_UNIT_PRESENT: &str = "unit_present";

/// Attribute key a construction reads a declared value from.
pub const ATTR_VALUE: &str = "value";
/// Attribute key holding a binary or unary operator.
pub const ATTR_OP: &str = "op";
/// Attribute key naming the source construct an `unsupported` node stands for.
pub const ATTR_SOURCE_NODE: &str = "go_node";
/// Attribute key classifying what an identifier resolves to.
pub const ATTR_REF: &str = "ref";
/// Attribute key carrying the source declaration's documentation block.
pub const ATTR_DOC: &str = "doc";

/// Flag marking a declaration as part of the source's public surface.
pub const FLAG_EXPORTED: &str = "exported";
/// Flag marking a variadic signature.
pub const FLAG_VARIADIC: &str = "variadic";
/// Flag marking a method bound through a pointer receiver.
pub const FLAG_POINTER_RECEIVER: &str = "pointer_receiver";
/// Ownership fact: the body provably assigns through this pointer.
pub const FLAG_MUTATED: &str = "mutated";
/// Ownership fact: this pointer provably outlives the call.
pub const FLAG_ESCAPES: &str = "escapes";
/// Ownership fact: this pointer reached a call the front end did not analyse, so the other two
/// facts being absent means UNPROVEN rather than false.
pub const FLAG_EFFECT_UNKNOWN: &str = "effect_unknown";

/// Child kinds a construction reads. Opaque here: these are the strings the pack and the front end
/// agreed on, and this face compares them without interpreting them.
pub(crate) const CHILD_FIELD: &str = "field";
pub(crate) const CHILD_METHOD: &str = "method";
pub(crate) const CHILD_PARAM: &str = "param";
pub(crate) const CHILD_RESULT: &str = "result";
pub(crate) const CHILD_BODY: &str = "body";

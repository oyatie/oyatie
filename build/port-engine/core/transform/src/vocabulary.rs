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
/// The construction emitting a `static` for a package variable nothing writes.
pub const CONSTRUCTION_RUST_STATIC: &str = "rust_static";
/// The pack form id for a package variable something writes.
pub const FORM_WRITTEN_PACKAGE_VAR: &str = "written_package_var";
/// Declaration-level construction: a transparent type alias.
pub const CONSTRUCTION_RUST_TYPE_ALIAS: &str = "rust_type_alias";
/// Declaration-level construction: a single-field tuple struct over the underlying type.
pub const CONSTRUCTION_RUST_NEWTYPE: &str = "rust_newtype";
/// Declaration-level construction: a struct with fields, plus an `impl` block whose methods
/// are stubbed.
pub const CONSTRUCTION_RUST_STRUCT: &str = "rust_struct";
/// Declaration-level construction: a struct whose methods carry their translated bodies.
pub const CONSTRUCTION_RUST_STRUCT_BODY: &str = "rust_struct_body";
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
/// The declaration kind of a free function.
pub const KIND_FUNC: &str = "func";
/// The prefix the source's explicit constructor convention uses.
pub const CONSTRUCTOR_PREFIX: &str = "New";
/// The type kind of a fixed-length array, whose `name` is its LENGTH rather than an identity.
pub const TYPE_ARRAY: &str = "array";
/// The type kind of a pointer.
pub const TYPE_POINTER: &str = "pointer";
/// The node kind of a literal.
pub const KIND_IDENT: &str = "ident";
pub const KIND_LITERAL: &str = "literal";
/// The node kind of a composite literal.
pub const KIND_COMPOSITE: &str = "composite";
/// The node kind of one keyed element of a composite literal.
pub const KIND_KEYED: &str = "keyed";
/// The node kind of a type's zero value, which the source supplies where a value is omitted.
pub const KIND_ZERO: &str = "zero";
/// The disposition describing an OWNED pointer, and how one is constructed.
///
/// Named here so the address of a fresh composite is built from the same rule that says what an
/// owned pointer is, rather than from a second rule free to disagree with it.
pub const DISPOSITION_OWNED_POINTER: &str = "escaping_owned";
/// The idiom that spells an emptiness test as a method rather than a comparison.
pub const IDIOM_EMPTY_STRING: &str = "empty_string_comparison";
/// The idiom that borrows a sequence as a slice rather than as its owned container.
pub const IDIOM_BORROWED_SLICE: &str = "borrowed_sequence_is_a_slice";
/// The idiom that spells a type as `Self` inside its own impl block.
pub const IDIOM_SELF_IN_IMPL: &str = "self_inside_own_impl";
/// The idiom that a counter used only as an index needs neither of the two conversions.
pub const IDIOM_INDEX_COUNTER: &str = "index_counter_is_usize";
/// The idiom that a loop counting to reach each element is an iterator over the sequence.
pub const IDIOM_INDEX_LOOP: &str = "index_loop_is_an_iterator";
/// The idiom that a parallel assignment exchanging two elements is the sequence's own swap.
pub const IDIOM_SWAP: &str = "parallel_exchange_is_a_swap";
/// The idiom that a three-way comparison is the target's ordering type.
pub const IDIOM_ORDERING: &str = "three_way_comparison_is_an_ordering";
/// The idiom that a match yielding only booleans is a membership test.
pub const IDIOM_MATCHES: &str = "boolean_match_is_matches";
/// The idiom that a counter used only as an index is a , so neither conversion is needed.
/// The argument shape a conditional mapping may require: a source string literal.
pub const ARGUMENT_STRING_LITERAL: &str = "string_literal";
/// The node kind of a unary expression.
pub const KIND_UNARY: &str = "unary";
/// The node kind of a call.
pub const KIND_CALL: &str = "call";
/// The node kind of a return statement.
pub const KIND_RETURN: &str = "return";
/// The node kind of an index expression.
pub const KIND_INDEX: &str = "index";
/// The node kind of a field selector.
pub const KIND_SELECTOR: &str = "selector";
/// The declaration kind of a package-level variable.
pub const KIND_VAR: &str = "var";
/// The source spelling of the address-of operator.
pub const OPERATOR_ADDRESS_OF: &str = "&";
/// Attribute key holding a literal's lexical class, as the source spells it.
pub const ATTR_LIT_KIND: &str = "lit_kind";
/// The lexical class of a string literal.
pub const LIT_KIND_STRING: &str = "STRING";
/// The source type name whose ownership decides a string literal's target form.
pub const SOURCE_STRING: &str = "string";
/// The source type name of its own default integer.
pub const SOURCE_INT: &str = "int";
/// The target's unsized view of a string, which is what a borrowed one is.
pub const TARGET_STR: &str = "str";
/// Attribute key naming the source construct an `unsupported` node stands for.
pub const ATTR_SOURCE_NODE: &str = "go_node";
/// The named POSITIONS a type can appear in, which is what decides the form a trait takes there.
///
/// A borrowed trait object is right for a parameter and impossible for a value a function returns,
/// so the pack answers per position rather than once.
pub const POSITION_PARAM: &str = "param";
/// See [`POSITION_PARAM`].
pub const POSITION_RESULT: &str = "result";
/// See [`POSITION_PARAM`].
pub const POSITION_FIELD: &str = "field";
/// The position a trait appears in as another trait's REQUIREMENT.
pub const POSITION_SUPERTRAIT: &str = "supertrait";
/// The position an `impl Trait for Type` names its trait in — the one place a trait appears as
/// itself rather than as something holding it, and still declared rather than special-cased.
pub const POSITION_TRAIT: &str = "trait";

/// Type kind: a named type whose underlying type is an interface.
///
/// Distinguished from a plain named type because the target holds the two differently — a struct
/// is a value and a trait has no size.
pub const TYPE_NAMED_INTERFACE: &str = "named_interface";
/// The type kind of the source's BARE interface, which names no methods.
pub const TYPE_INTERFACE: &str = "interface";

/// Attribute key classifying what an identifier resolves to.
pub const ATTR_REF: &str = "ref";
/// The [`ATTR_REF`] value for an identifier naming a constant.
pub const REF_CONST: &str = "const";
/// The [`ATTR_REF`] value for an identifier naming a PACKAGE, which is not a value at all.
pub const REF_PACKAGE: &str = "package";
/// Attribute key holding the package-qualified IDENTITY of what a call resolves to.
pub const ATTR_CALLEE: &str = "callee";
/// Attribute key distinguishing a call through a RECEIVER from a call to a free function.
pub const ATTR_CALLEE_KIND: &str = "callee_kind";
/// The one value [`ATTR_CALLEE_KIND`] takes; its absence means a free function.
pub const CALLEE_KIND_METHOD: &str = "method";
/// Attribute key holding the receiver a TRAIT method binds, derived from its observed
/// implementors. Absent means nothing was observed to implement the interface, and the pack's
/// declared decision answers instead.
pub const ATTR_RECEIVER: &str = "receiver";
/// Attribute key holding the dotted FIELD PATH a promoted method is reached through.
pub const ATTR_VIA: &str = "via";
/// Attribute key recording HOW an interface satisfaction was observed.
pub const ATTR_SITE: &str = "site";
/// Attribute key carrying the source declaration's documentation block.
pub const ATTR_DOC: &str = "doc";

/// Flag marking a declaration as part of the source's public surface.
pub const FLAG_EXPORTED: &str = "exported";
/// Flag marking a variadic signature.
pub const FLAG_VARIADIC: &str = "variadic";
/// Flag marking a method bound through a pointer receiver.
pub const FLAG_POINTER_RECEIVER: &str = "pointer_receiver";
/// The body assigns to this binding's OWN name, so the target needs `mut` on it.
///
/// Distinct from [`FLAG_MUTATED`]: on a parameter that flag means the body writes THROUGH the
/// pointer, which is a claim about the CALLER's value and drives an exclusive borrow. Rebinding
/// the callee's copy is the opposite claim — the caller sees nothing — so one flag carrying both
/// would demand a borrow for every parameter a body happens to reassign.
pub const FLAG_REBOUND: &str = "rebound";
/// Flag marking a binding whose type the source did not write, so the target must not annotate it.
pub const FLAG_INFERRED: &str = "inferred";
/// The body never mentions this parameter.
///
/// Ordinary in the source — it is how a function satisfies an interface it does not need every
/// argument of — and a WARNING in the target. The leading underscore says the same thing the
/// source left implicit, and a parameter's name is not part of a function's type, so the signature
/// is unchanged.
pub const FLAG_UNREAD: &str = "unread";
/// The body reads this binding MORE THAN ONCE.
///
/// The source copies a value on every read and the target moves it, so a second read of a
/// non-copying binding is a use after move. A binding read once is left alone: moving it is both
/// correct and what someone writing the target would put.
pub const FLAG_REREAD: &str = "reread";
/// How many times the enclosing body reads a binding, present only where that is more than one.
pub const ATTR_READ_COUNT: &str = "read_count";
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
/// An OBSERVED interface satisfaction, carrying the trait's full method set.
///
/// The method set rides on the node rather than being looked up from the interface's own
/// declaration, because the interface routinely lives in another unit and the impl is emitted
/// where the type is — a cross-unit lookup is a reference the model does not carry.
pub(crate) const CHILD_IMPLEMENTS: &str = "implements";
/// An interface an interface EMBEDS, which the target spells as a supertrait.
pub(crate) const CHILD_EMBEDS: &str = "embeds";
/// A method a type gains through EMBEDDING rather than declaration.
pub(crate) const CHILD_PROMOTED: &str = "promoted";
/// One name a destructuring bind introduces.
pub(crate) const CHILD_BIND: &str = "bind";
/// The child role of one place a parallel assignment writes to.
pub(crate) const CHILD_PLACE: &str = "place";
/// Attribute key marking a satisfaction whose interface is a pure supertrait bundle.
pub(crate) const ATTR_BUNDLE: &str = "bundle";
/// The expression a destructuring bind takes its values from.
pub(crate) const CHILD_VALUE: &str = "value";

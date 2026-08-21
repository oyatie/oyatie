//! Items: what an emitted region contains.
//!
//! Visibility, doc comments and a method's receiver are VALUES here rather than string prefixes.
//! That is the difference that fixed two real defects: a `"pub "` prefix concatenated into a trait
//! body produced `pub fn` on a trait method, which `syn` parses and `rustc` rejects; and a
//! receiver rendered as the literal `&self` made an interface's mutating method unimplementable
//! while the concrete path refused the same guess.

use crate::expr::RustExpr;
use crate::item_parts::{Receiver, RustField, RustFn, StructShape, Visibility};
use crate::stmt::RustStmt;
use crate::ty::RustType;

/// Whether an item is part of the emitted crate's public surface.
/// A top-level item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustItem {
    /// `const NAME: T = value;`
    Const {
        /// Documentation carried over from the source.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// Whether the constant is public.
        vis: Visibility,
        /// Its name, already cased for the target.
        name: String, // data_class: INTERNAL_ONLY
        /// Its type.
        ty: RustType,
        /// Its value, carried as a source spelling.
        value: String, // data_class: INTERNAL_ONLY
    },
    /// NOTHING, for a declaration whose output belongs to a unit-level item instead.
    ///
    /// Distinct from an absent item: the declaration WAS translated, and what it became is in the
    /// crate — a variant of the unit's failure enum, built once from the whole list rather than
    /// once per sentinel. Saying so with a value keeps the accounting honest, where returning an
    /// empty list would make a translated declaration indistinguishable from one nothing captured.
    Nothing,
    /// An IMPORT the module makes, so a path it names repeatedly is named once.
    ///
    /// A module that spells `std::fmt::Display`, `std::fmt::Formatter` and `std::fmt::Result` in
    /// every one of its seven sentinels names one std module twenty-one times. A person writing
    /// that writes `use std::fmt;` and then `fmt::Display` — which is what a reviewer meant by
    /// calling the qualified form "what a code generator emits, not what a person types nine
    /// times".
    ///
    /// Emitted only from what the module ACTUALLY contains, never from what it declared: an import
    /// nothing uses is a denied warning, where an unused type alias is only dead code.
    Use {
        /// The path imported, without the `use` or the semicolon.
        path: String, // data_class: INTERNAL_ONLY
    },
    /// The unit's sentinel failures as ONE enum: the type, its messages, and its `Error` impl.
    ///
    /// One item because they are one concept, exactly as the single-sentinel form is — and because
    /// the `Display` arm for each variant must be built from the same list the variants are, or a
    /// message drifts from the sentinel it belongs to.
    ///
    /// Grouped rather than one type each because the source's per-sentinel declaration answers a
    /// namespacing problem the target does not have. What that costs and what it preserves is the
    /// pack's to say; this face only spells it.
    SentinelEnum {
        /// Documentation for the type itself, which no source declaration owns.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// Whether the enum is public.
        vis: Visibility,
        /// Its type name.
        name: String, // data_class: INTERNAL_ONLY
        /// Whether a caller outside the crate may match it exhaustively.
        exhaustive: bool,
        /// Each sentinel, in the order the source declares them.
        variants: Vec<SentinelVariant>,
    },
    /// A SENTINEL failure: a unit struct that is an error, and can be compared against.
    ///
    /// Three items that are one concept — the type, its message, and its `Error` impl — so they are
    /// one IR item. Emitting them separately would let a message drift from the type it belongs to.
    /// The target's rendering of a type that the source satisfied its error interface with.
    ///
    /// The source's interface is satisfied by ONE method returning the message; the target's error
    /// trait declares no such method and takes the message from its display trait. So the method
    /// becomes a display impl, and the error impl follows from it — which is what makes the ported
    /// type usable everywhere the source's was, instead of carrying an inherent method nothing calls
    /// and no trait knows about.
    ///
    /// Held as its own item rather than assembled by the transform because the target spellings for
    /// this — the display trait, the formatter, the write method — are the renderer's to know, and
    /// they are already spelled once for the sentinel enum. Two spellings of one decision drift.
    MessageImpl {
        /// The source's documentation for the method.
        docs: Vec<String>,
        /// The type the impl is for.
        self_ty: RustType,
        /// The method's translated body, whose tail is the message.
        body: Vec<crate::stmt::RustStmt>,
        /// Whether the type is a FAILURE, and so also implements the target's error trait.
        ///
        /// The two constructions render identically and mean different things. A type satisfying
        /// the source's error interface is an error and gets both impls; a type with a `String`
        /// method is merely printable, and giving it the error trait makes it coerce into a boxed
        /// error, satisfy `?` in any failing function, and appear in the documentation as a failure
        /// — none of which the source says. A JSON value tag, a context key and a network address
        /// all became errors that way.
        is_failure: bool,
    },
    SentinelError {
        /// Documentation carried over from the source.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// Whether the sentinel is public.
        vis: Visibility,
        /// Its type name.
        name: String, // data_class: INTERNAL_ONLY
        /// The message it displays, as a source literal.
        message: String, // data_class: INTERNAL_ONLY
    },
    /// `const NAME: T = value;` — a package-level immutable value.
    ///
    /// Distinct from [`RustItem::Const`] only in that its value is an EXPRESSION rather than a
    /// source spelling, which is what a composite or a zero needs. Both emit `const`.
    ///
    /// It emitted `static` at first, and the argument was that the source's package variable has an
    /// ADDRESS while a const is materialised afresh at every use. That argument is sound and
    /// protects nothing: taking the address of a package variable is `&x` of an existing binding,
    /// which the engine REFUSES everywhere, so no emitted code can observe the difference. Two
    /// reviewers independently read the `const`/`static` split as a fingerprint of the source's own
    /// `const`/`var` split — which is what it was, since the source cannot make a `const` of a
    /// struct and can of an integer, a limitation that carries no meaning worth porting.
    ///
    /// The day `&<package variable>` translates, this is a `static` again, and the two decisions are
    /// linked rather than independent.
    PackageValue {
        /// Documentation carried over from the source.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// Whether the static is public.
        vis: Visibility,
        /// Its name, already cased for the target.
        name: String, // data_class: INTERNAL_ONLY
        /// Its type.
        ty: RustType,
        /// Its value, which must be a constant expression.
        value: RustExpr,
    },
    /// `type Name = T;` — transparent, for a source alias.
    TypeAlias {
        /// Documentation carried over from the source.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// Whether the alias is public.
        vis: Visibility,
        /// Its name.
        name: String, // data_class: INTERNAL_ONLY
        /// Type parameters, for an alias that takes one.
        ///
        /// Empty for the source's own aliases, which are always concrete. The failure alias takes
        /// one — a `Result` names the success type its user supplies — and it is the only thing
        /// this crate emits that does.
        generics: Vec<String>, // data_class: INTERNAL_ONLY
        /// What it aliases.
        ty: RustType,
    },
    /// A struct, with any inherent methods that belong to it.
    Struct {
        /// Documentation carried over from the source.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// Whether the struct is public.
        vis: Visibility,
        /// Its name.
        name: String, // data_class: INTERNAL_ONLY
        /// Its data shape.
        shape: StructShape,
        /// The traits derived on it, in the order the pack declares them.
        ///
        /// Order is the pack's, not sorted: a derive list is read by people, and `Debug, Clone`
        /// reads the way the pack lists it rather than the way the alphabet does.
        derives: Vec<String>, // data_class: INTERNAL_ONLY
        /// Methods emitted into an `impl` block beside it.
        methods: Vec<RustFn>,
    },
    /// A trait, from a source interface.
    Trait {
        /// Documentation carried over from the source.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// Whether the trait is public.
        vis: Visibility,
        /// Its name.
        name: String, // data_class: INTERNAL_ONLY
        /// Traits this one REQUIRES, from a source interface's embedded interfaces.
        ///
        /// A requirement rather than a copy of the method set: an implementor must implement these
        /// too, which is what the source means by embedding and what a flattened method list would
        /// silently weaken.
        supertraits: Vec<RustType>,
        /// Its required methods, as signatures.
        methods: Vec<RustFn>,
    },
    /// `impl<T: A + B> Trait for T {}` — a supertrait bundle satisfied by anything that qualifies.
    ///
    /// The shape a source interface takes when it embeds other interfaces and declares no method of
    /// its own, which is the great majority of them. The source satisfies such an interface
    /// STRUCTURALLY: a type that has both method sets has it, with nothing to declare. The target
    /// is nominal, so the same statement needs a blanket impl — and the alternative, one hand-written
    /// empty impl per type, is both more code and strictly weaker, because a type the engine never
    /// saw asserted would not have the trait the source says it has.
    BlanketImpl {
        /// The trait implemented for everything that meets the bounds.
        name: String, // data_class: INTERNAL_ONLY
        /// The bounds, which are the trait's own supertraits.
        bounds: Vec<RustType>,
    },
    /// `impl Trait for Type { .. }`, from an OBSERVED interface satisfaction.
    ///
    /// The trait is a path rather than a name because the interface a type satisfies is routinely
    /// declared in another unit, and the impl is emitted where the type is.
    TraitImpl {
        /// Documentation carried over from the source, plus how the satisfaction was observed.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// The trait being implemented, as a path.
        trait_path: RustType,
        /// The type implementing it.
        self_ty: RustType,
        /// The trait's required methods, each with a body.
        methods: Vec<RustFn>,
    },
    /// `impl Type { .. }` — an inherent impl block standing on its own.
    ///
    /// Separate from [`RustItem::Struct`]'s methods because the engine emits one region per source
    /// declaration, and a package-level constructor is a declaration of its own. Folding it into
    /// the type's item would make one declaration's output depend on another's; the target allows
    /// several inherent impls for a type, so nothing is given up by keeping them apart.
    InherentImpl {
        /// Documentation carried over from the source declaration.
        docs: Vec<String>, // data_class: INTERNAL_ONLY
        /// The type the block is on.
        self_ty: RustType,
        /// The associated functions it carries.
        methods: Vec<RustFn>,
    },
    /// A free function.
    Function(RustFn),
}

/// One sentinel, as a variant of the unit's failure enum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentinelVariant {
    /// Documentation carried over from the source declaration.
    pub docs: Vec<String>, // data_class: INTERNAL_ONLY
    /// Its variant name, already cased for the target.
    pub name: String, // data_class: INTERNAL_ONLY
    /// The message it carries, as a target string literal spelling.
    pub message: String, // data_class: INTERNAL_ONLY
    /// The values the message interpolates, empty when it is a plain literal.
    ///
    /// A source sentinel is sometimes built by a FORMATTING constructor over constants —
    /// `fmt.Errorf("Valid KSUIDs are %v bytes", byteLength)`. Its message is still fixed at compile
    /// time, so it is still a sentinel; it is simply not spelled as one literal. Carrying the values
    /// separately is what lets the display arm write them rather than pretend they are not there.
    pub arguments: Vec<RustExpr>, // data_class: INTERNAL_ONLY
}

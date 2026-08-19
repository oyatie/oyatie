//! Items: what an emitted region contains.
//!
//! Visibility, doc comments and a method's receiver are VALUES here rather than string prefixes.
//! That is the difference that fixed two real defects: a `"pub "` prefix concatenated into a trait
//! body produced `pub fn` on a trait method, which `syn` parses and `rustc` rejects; and a
//! receiver rendered as the literal `&self` made an interface's mutating method unimplementable
//! while the concrete path refused the same guess.

use crate::expr::RustExpr;
use crate::stmt::RustStmt;
use crate::ty::RustType;

/// Whether an item is part of the emitted crate's public surface.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum Visibility {
    /// No qualifier. This is also the ONLY legal choice for a trait item, which is as public as
    /// its trait and may not say so.
    #[default]
    Inherited,
    /// `pub`.
    Public,
}

/// How a method takes its receiver.
///
/// An explicit decision with no default. The source's receiver mode is a fact the front end knows
/// and the transform must not guess: `&self` silently drops the mutation a pointer receiver
/// exists to permit, and `&mut self` claims one the source may never perform.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Receiver {
    /// `&self`
    Shared,
    /// `&mut self`
    Exclusive,
    /// `self`
    Owned,
}

impl Receiver {
    /// The receiver's spelling.
    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Shared => "&self",
            Self::Exclusive => "&mut self",
            Self::Owned => "self",
        }
    }
}

/// One parameter of a function or method.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustParam {
    /// The parameter's name, already cased for the target.
    pub name: String, // data_class: INTERNAL_ONLY
    /// Whether the body never mentions this parameter.
    ///
    /// Emitted with a leading underscore. The source treats an unused parameter as ordinary and
    /// the target warns on it, so the underscore is what says the omission is deliberate — and it
    /// changes nothing a caller can see, because a parameter's name is not part of the type.
    pub unread: bool,
    /// Whether the body assigns to this parameter's own binding.
    ///
    /// The source makes every parameter a mutable local copy and the target makes none of them, so
    /// this is observed rather than defaulted. It says nothing about the CALLER's value — a
    /// rebound parameter is still passed by value, and the caller sees nothing.
    pub rebound: bool,
    /// Its type.
    pub ty: RustType,
}

/// One field of a struct.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustField {
    /// Documentation carried over from the source.
    pub docs: Vec<String>, // data_class: INTERNAL_ONLY
    /// Whether the field is public.
    pub vis: Visibility,
    /// The field's name, already cased for the target.
    pub name: String, // data_class: INTERNAL_ONLY
    /// Its type.
    pub ty: RustType,
}

/// A function, a method, or a trait method's signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustFn {
    /// Documentation carried over from the source.
    pub docs: Vec<String>, // data_class: INTERNAL_ONLY
    /// Whether the function is public. Always [`Visibility::Inherited`] for a trait item.
    pub vis: Visibility,
    /// The function's name, already cased for the target.
    pub name: String, // data_class: INTERNAL_ONLY
    /// The receiver, when this is a method.
    pub receiver: Option<Receiver>,
    /// Parameters, in order — which is semantic.
    pub params: Vec<RustParam>,
    /// The return type. `None` and the unit type both render without an arrow.
    pub ret: Option<RustType>,
    /// The body. `None` is a signature with no body, which is what a trait item is.
    pub body: Option<Vec<RustStmt>>,
}

/// The shape of a struct's data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructShape {
    /// `struct Name;`
    Unit,
    /// `struct Name(T);` — a newtype, which is how a distinct source type stays distinct.
    Tuple(Vec<RustField>),
    /// `struct Name { .. }`
    Named(Vec<RustField>),
}

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

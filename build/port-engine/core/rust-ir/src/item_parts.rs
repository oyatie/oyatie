//! The pieces an item is BUILT FROM: visibility, a receiver, a parameter, a field, a signature.
//!
//! Split from `item.rs` so that file holds one thing — the closed set of items this IR can emit.
//! These are the vocabulary that set is written in, and they are stable where the item list grows
//! with every construct the engine learns.

use crate::stmt::RustStmt;
use crate::ty::RustType;

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
    /// Attributes the function carries, without the brackets a reader would write.
    ///
    /// The pack decides which and why; this face only spells them. Empty for a function that wants
    /// none, which is most of them.
    pub attrs: Vec<String>, // data_class: INTERNAL_ONLY
    /// The body. `None` is a signature with no body, which is what a trait item is.
    pub body: Option<Vec<RustStmt>>,
    /// Whether the body may YIELD to an executor, which the target spells on the signature.
    ///
    /// A property of the function rather than of any statement in it: `.await` is legal only inside
    /// an `async` body, so a rule that emits one has to say so here as well, and the two are the
    /// same decision read in two places.
    pub is_async: bool,
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

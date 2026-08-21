//! A source type, as a tree.
//!
//! Until now a type reached the engine as a flat SPELLING — `int`, `[]byte`,
//! `map[string][]*Foo` — and the pack answered by matching that spelling against a table. That
//! works exactly as long as every type in the corpus is either primitive or has its own table
//! entry, and it fails in three ways at once on a real corpus: a composite type needs an entry per
//! shape rather than per constructor, a type from another package resolves to nothing because the
//! table is keyed by unqualified text, and two packages that each declare a `Point` collide.
//!
//! So a type is a tree here, and it is the same UNIFORM NODE the declaration tree uses: `kind` is
//! a value rather than a variant, so a second source language needs a second rule pack and not a
//! second seam. The engine compares these strings and never interprets them — `slice` is not a
//! sequence to the engine, it is a key the pack answers for.

/// One node of a source type.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeRef {
    /// What kind of type this is, as an opaque slug: `basic`, `named`, `pointer`, `slice`, and so
    /// on. The pack decides what each means. // data_class: INTERNAL_ONLY
    pub kind: String,
    /// The type's own name, for the kinds that have one. Empty otherwise.
    pub name: String, // data_class: INTERNAL_ONLY
    /// The package that declares this type, for the kinds that have one.
    ///
    /// This is what makes a named type ADDRESSABLE. Without it, a reference to another package's
    /// type is indistinguishable from a local one, and two packages declaring the same name are
    /// indistinguishable from each other — so the resolution silently picks one.
    pub package: String, // data_class: INTERNAL_ONLY
    /// Type arguments, in significant order: an element type, a key and a value, a parameter list.
    pub args: Vec<TypeRef>,
}

impl TypeRef {
    /// A type of `kind` with no name, package, or arguments.
    pub fn of(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            ..Self::default()
        }
    }

    /// A primitive named type: `int`, `bool`, `string`.
    pub fn basic(name: impl Into<String>) -> Self {
        Self {
            kind: "basic".to_owned(),
            name: name.into(),
            ..Self::default()
        }
    }

    /// A named type declared by a package.
    pub fn named(package: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            kind: "named".to_owned(),
            name: name.into(),
            package: package.into(),
            ..Self::default()
        }
    }

    /// A composite over its arguments: a slice, a map, a pointer.
    pub fn composite(kind: impl Into<String>, args: Vec<Self>) -> Self {
        Self {
            kind: kind.into(),
            args,
            ..Self::default()
        }
    }

    /// `true` when this node carries no information at all — the shape a declaration without a
    /// type has.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kind.is_empty() && self.name.is_empty() && self.args.is_empty()
    }

    /// The type's fully-qualified identity, for the kinds that have one: `package.Name`.
    ///
    /// Empty when the type has no name. A bare name is deliberately NOT returned for an unpackaged
    /// type: a qualified key and an unqualified one must not collide in the same table, because
    /// that is the collision this whole type exists to prevent.
    #[must_use]
    pub fn qualified(&self) -> String {
        match (self.package.is_empty(), self.name.is_empty()) {
            (_, true) => String::new(),
            (true, false) => self.name.clone(),
            (false, false) => format!("{}.{}", self.package, self.name),
        }
    }

    /// A readable rendering, for refusal messages. Never used to make a decision.
    #[must_use]
    pub fn describe(&self) -> String {
        let mut out = self.kind.clone();
        let qualified = self.qualified();
        if !qualified.is_empty() {
            out.push(' ');
            out.push_str(&qualified);
        }
        if !self.args.is_empty() {
            let rendered: Vec<String> = self.args.iter().map(Self::describe).collect();
            out.push('<');
            out.push_str(&rendered.join(", "));
            out.push('>');
        }
        out
    }
}

//! Statements: the things a body is a sequence of.
//!
//! Split from `expr.rs` because the two are different shapes with different rules. An expression
//! has a value and can stand anywhere one is wanted; a statement has a position, and the LAST one
//! in a block decides the block's value. Keeping them in one file made a reader hold both sets of
//! rules at once to find either.

use crate::expr::RustExpr;
use crate::ops::BinaryOp;
use crate::ty::RustType;

/// A statement in an emitted body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustStmt {
    /// `let (a, b) = value;` — a destructuring bind.
    LetTuple {
        /// The names bound, in order.
        names: Vec<TupleBind>, // data_class: INTERNAL_ONLY
        /// What they are bound from.
        value: RustExpr,
    },
    /// `(<place>, <place>) = <value>;` — the target's destructuring assignment.
    ///
    /// Distinct from a sequence of [`RustStmt::Assign`] because the ORDER is the whole content of
    /// the source construct: `a[i], a[j] = a[j], a[i]` is a swap only because every operand on both
    /// sides is evaluated before any of them is assigned. Two separate assignments would write the
    /// first place and then read it back, which is a different program.
    AssignTuple {
        /// The places written, in order.
        places: Vec<RustExpr>,
        /// What they are assigned from — one expression per place, or a single tuple-valued one.
        values: Vec<RustExpr>,
    },
    /// `let [mut] <name>[: <ty>] [= <value>];`
    Let {
        /// The bound name, already cased for the target.
        name: String, // data_class: INTERNAL_ONLY
        /// Whether the binding is written again.
        ///
        /// A VALUE rather than a default, because the source makes every binding mutable and the
        /// target makes none of them: assuming either way is wrong for half the bindings in any
        /// real body.
        mutable: bool,
        /// The declared type, when the source declared one.
        ty: Option<RustType>,
        /// What it is bound to. `None` is a binding the body fills in later, which the source
        /// spells as a `var` with no initializer.
        value: Option<RustExpr>,
    },
    /// An expression evaluated for effect: `<expr>;`
    Semi(RustExpr),
    /// A trailing expression, with no semicolon — the value of the enclosing block.
    Tail(RustExpr),
    /// An early `return <expr>;`
    Return(Option<RustExpr>),
    /// `<target> [op]= <value>;`
    Assign {
        /// What is assigned to: a path, a field, an index.
        target: RustExpr,
        /// The operator of a read-modify-write, or `None` for a plain assignment.
        ///
        /// Carried here rather than desugared into `target = target op value`, because the place
        /// expression is evaluated ONCE by both languages and rewriting it would evaluate an index
        /// or a call inside the place twice.
        op: Option<BinaryOp>,
        /// The new value.
        value: RustExpr,
    },
    /// `while <cond> { .. }`
    While {
        /// The loop condition.
        cond: RustExpr,
        /// The loop body.
        body: Vec<RustStmt>,
    },
    /// `loop { .. }`
    Loop(Vec<RustStmt>),
    /// `for <binding> in <iter> { .. }`
    ForIn {
        /// The bound name, already cased for the target.
        binding: String, // data_class: INTERNAL_ONLY
        /// What is iterated.
        iter: RustExpr,
        /// The loop body.
        body: Vec<RustStmt>,
    },
    /// `break;`
    Break,
    /// `let _ = <expr>;` — evaluate the operand and keep nothing.
    ///
    /// Not a [`RustStmt::Let`] whose name happens to be `_`. The target's blank is a PATTERN and
    /// not an identifier, so spelling it as a name reaches the renderer as one and is refused
    /// there; and it is not a [`RustStmt::Semi`] either, because that would drop an operand whose
    /// value the target warns about discarding.
    Discard(RustExpr),
    /// `continue;`
    ///
    /// Legal only where the enclosing loop advances WITHOUT a statement at the end of its body.
    /// The construction that would break this — a source loop whose post-statement the target has
    /// to spell as the body's last statement — refuses in the transform rather than emitting a
    /// `continue` that skips it, because that compiles and loops a different number of times.
    Continue,
    /// `{ .. }` — a bare block, entered for its SCOPE rather than for a value.
    ///
    /// The source's `for init; cond; post` scopes `init` to the loop. Emitting the binding as a
    /// sibling of the loop would widen that scope to the rest of the enclosing block, where it can
    /// shadow a name the source left visible — so the block is the scope, spelled.
    Block(Vec<RustStmt>),
}

/// One name a destructuring bind introduces.
///
/// Mutability is PER NAME rather than per statement. The source binds each name independently, and
/// in `v, err := f()` it is routinely `err` that is written again while `v` is not — so a single
/// flag for the pair would have to be the disjunction of both, and every value binding would come
/// out mutable to serve the failure beside it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TupleBind {
    /// The bound name, already cased for the target.
    pub name: String, // data_class: INTERNAL_ONLY
    /// Whether the body writes this name after binding it.
    pub mutable: bool,
}

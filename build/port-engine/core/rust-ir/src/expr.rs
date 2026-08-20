//! Statements and expressions, with precedence decided by the TREE.
//!
//! The previous IR built bodies with `format!` and parenthesised every binary expression
//! unconditionally, because a text builder cannot see its own nesting and a wrong precedence
//! table produces arithmetic that compiles and computes something else. A tree can see it: a
//! child is parenthesised exactly when its own precedence binds looser than the position it sits
//! in, so `a + b * c` emits without parentheses and `(a + b) * c` keeps the one it needs.
//!
//! The precedence table itself lives in [`crate::ops`].

use crate::ops::{BinaryOp, Precedence, UnaryOp};
use crate::stmt::RustStmt;
use crate::ty::RustType;



/// An expression in an emitted body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustExpr {
    /// A literal, carried as its source spelling.
    ///
    /// Pass-through is safe only because the emitted tree is parsed and then compiled: a source
    /// literal with no valid target spelling fails the parse, which is the correct outcome. No
    /// attempt is made to normalise numbers, because a rounded literal compiles and means
    /// something else.
    /// `&base[lo..hi]` — a borrowed subrange, with either bound optional.
    ///
    /// BORROWED, not owned. The source's slice expression produces a VIEW over the same backing
    /// array and does not copy, so an owned target would be a different program with different
    /// costs and different aliasing.
    Slice {
        /// What is being sliced.
        base: Box<RustExpr>,
        /// The lower bound, or the start.
        low: Option<Box<RustExpr>>,
        /// The upper bound, or the end.
        high: Option<Box<RustExpr>>,
    },
    /// `expr as T` — a truncating conversion.
    ///
    /// A NODE rather than text, so a later rule can see the cast and remove it. A cast rendered
    /// into a string is invisible to integer right-sizing, which is the rule most likely to want it
    /// gone.
    Cast {
        /// What is being converted.
        expr: Box<RustExpr>,
        /// What it is converted to.
        ty: RustType,
    },
    /// `expr?` — propagate a failure to the caller.
    ///
    /// An OPERATOR rather than a call, which is the whole point of recognising the source's
    /// propagation idiom: a two-statement bind-and-check becomes one expression that cannot be
    /// forgotten, where the source's version is a convention a caller may ignore.
    Try(Box<RustExpr>),
    Literal(String), // data_class: INTERNAL_ONLY
    /// A path or identifier, already cased for the target.
    Path(String), // data_class: INTERNAL_ONLY
    /// A binary operation.
    Binary {
        /// The operator.
        op: BinaryOp,
        /// Left operand.
        lhs: Box<RustExpr>,
        /// Right operand.
        rhs: Box<RustExpr>,
    },
    /// A prefix operation.
    Unary {
        /// The operator.
        op: UnaryOp,
        /// The operand.
        operand: Box<RustExpr>,
    },
    /// `if <cond> { .. } else ..`
    If {
        /// The condition.
        cond: Box<RustExpr>,
        /// The taken branch.
        then: Vec<RustStmt>,
        /// An `else` branch, which is itself a block or a further `if`.
        otherwise: Option<Box<RustExpr>>,
    },
    /// A braced block.
    Block(Vec<RustStmt>),
    /// A tuple, which is how a multi-value result leaves a function.
    Tuple(Vec<RustExpr>),
    /// `<base>.<name>` — a field access.
    /// A TUPLE INDEX — `x.0`, the one field of a newtype.
    ///
    /// Not a [`Self::Field`] with a numeric name. The target spells the two the same way and means
    /// different things by them: a field is an identifier and `0` is not a valid one, so lowering a
    /// tuple index as a field refuses the whole declaration. It did, in six packages at once, and
    /// the failure was invisible because what it produced was a REFUSAL rather than bad output —
    /// the compile proof went green because the type was no longer emitted at all.
    TupleIndex {
        /// What the index reads from.
        base: Box<RustExpr>,
        /// Which element, counted from zero.
        index: usize,
    },
    Field {
        /// What the field is read from.
        base: Box<RustExpr>,
        /// The field's name, already cased for the target.
        name: String, // data_class: INTERNAL_ONLY
    },
    /// `<callee>(<args>)`
    /// A MACRO call with a template and its arguments.
    ///
    /// Its own node rather than assembled text, because the arguments are ordinary expressions and
    /// text assembly cannot carry one: a field read, a method call, or anything needing parentheses
    /// has no unambiguous spelling to substitute. That limit is what made every real formatting
    /// call in the corpus refuse.
    ///
    /// The template is carried as an already-translated target template. Whether its placeholders
    /// correspond to the arguments is settled before the node is built, because the macro checks it
    /// at compile time and a mismatch would be a render failure rather than a refusal by name.
    MacroCall {
        /// The macro's name, without the `!`.
        name: String, // data_class: INTERNAL_ONLY
        /// The target template, placeholders already translated.
        template: String, // data_class: INTERNAL_ONLY
        /// The values the template consumes, in order.
        args: Vec<RustExpr>,
    },
    Call {
        /// The function being called.
        callee: Box<RustExpr>,
        /// Its arguments, in order.
        args: Vec<RustExpr>,
    },
    /// `<receiver>.<method>(<args>)`
    MethodCall {
        /// What the method is called on.
        receiver: Box<RustExpr>,
        /// The method's name, already cased for the target.
        method: String, // data_class: INTERNAL_ONLY
        /// Its arguments, in order.
        args: Vec<RustExpr>,
    },
    /// `<base>[<index>]`
    Index {
        /// What is indexed.
        base: Box<RustExpr>,
        /// The index.
        index: Box<RustExpr>,
    },
    /// `<path> { <field>: <value>, .. }`
    StructLiteral {
        /// The struct's path.
        path: String, // data_class: INTERNAL_ONLY
        /// Its fields, in declared order.
        fields: Vec<(String, RustExpr)>,
    },
    /// `<start>..<end>`
    Range {
        /// Inclusive lower bound.
        start: Box<RustExpr>,
        /// Exclusive upper bound.
        end: Box<RustExpr>,
    },
    /// `&<inner>` or `&mut <inner>`
    Reference {
        /// `true` for `&mut`.
        mutable: bool,
        /// What is referenced.
        inner: Box<RustExpr>,
    },
    /// `self`
    SelfValue,
    /// `match <scrutinee> { <patterns> => { .. }, .. }`
    Match {
        /// What is matched on.
        scrutinee: Box<RustExpr>,
        /// The arms, in order. An arm with no patterns is the wildcard.
        arms: Vec<MatchArm>,
    },
}

/// One arm of a match.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchArm {
    /// The patterns this arm accepts, as literal expressions. EMPTY is the wildcard arm.
    ///
    /// Patterns are expressions because the source's switch cases are expressions: a Go case is a
    /// value to compare against, not a destructuring pattern. Modelling them as full patterns
    /// would be inventing a capability the source does not have.
    pub patterns: Vec<RustExpr>,
    /// The arm's body.
    pub body: Vec<RustStmt>,
}

impl RustExpr {
    /// The precedence this expression presents to its parent.
    ///
    /// Anything atomic — a literal, a path, a parenthesised or braced form — binds tightest, so it
    /// never needs wrapping.
    #[must_use]
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::Binary { op, .. } => op.precedence(),
            Self::Unary { .. } => UnaryOp::precedence(),
            // An `if` is not an operand in any position this IR builds, but giving it the loosest
            // precedence means that if it ever becomes one it is parenthesised rather than
            // silently reassociated.
            Self::If { .. } => Precedence::LOOSEST,
            // A reference binds like a prefix operator; a range binds looser than any of them.
            Self::Reference { .. } => UnaryOp::precedence(),
            Self::Range { .. } => Precedence::LOOSEST,
            // Postfix forms bind tightest of all: `a.b`, `f(x)`, `v[i]` never need wrapping, and
            // their own base is bracketed by the lowering when it is not itself atomic.
            Self::Literal(_)
            | Self::Path(_)
            | Self::Block(_)
            | Self::Tuple(_)
            | Self::Field { .. }
            | Self::TupleIndex { .. }
            | Self::Call { .. }
            | Self::MethodCall { .. }
            | Self::MacroCall { .. }
            | Self::Index { .. }
            | Self::StructLiteral { .. }
            | Self::Try(_)
            | Self::Slice { .. }
            | Self::Cast { .. }
            | Self::SelfValue
            | Self::Match { .. } => Precedence::ATOMIC,
        }
    }

    /// Whether this expression needs parentheses when it appears as an operand.
    ///
    /// `side_binds_tighter` distinguishes the two operands of a left-associative operator: the
    /// right-hand side of `a - (b - c)` needs its parentheses and the left-hand side of
    /// `(a - b) - c` does not, and the difference is associativity rather than precedence.
    #[must_use]
    pub fn needs_parens_under(&self, parent: BinaryOp, is_right_operand: bool) -> bool {
        let own = self.precedence();
        let parent_precedence = parent.precedence();
        if own < parent_precedence {
            return true;
        }
        if own == parent_precedence {
            return is_right_operand || parent.is_non_associative();
        }
        false
    }
}

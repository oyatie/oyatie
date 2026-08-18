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

/// A statement in an emitted body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustStmt {
    /// `let <name> = <value>;`
    Let {
        /// The bound name, already cased for the target.
        name: String, // data_class: INTERNAL_ONLY
        /// What it is bound to.
        value: RustExpr,
    },
    /// An expression evaluated for effect: `<expr>;`
    Semi(RustExpr),
    /// A trailing expression, with no semicolon — the value of the enclosing block.
    Tail(RustExpr),
    /// An early `return <expr>;`
    Return(Option<RustExpr>),
    /// `<target> = <value>;`
    Assign {
        /// What is assigned to: a path, a field, an index.
        target: RustExpr,
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
}

/// An expression in an emitted body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustExpr {
    /// A literal, carried as its source spelling.
    ///
    /// Pass-through is safe only because the emitted tree is parsed and then compiled: a source
    /// literal with no valid target spelling fails the parse, which is the correct outcome. No
    /// attempt is made to normalise numbers, because a rounded literal compiles and means
    /// something else.
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
    Field {
        /// What the field is read from.
        base: Box<RustExpr>,
        /// The field's name, already cased for the target.
        name: String, // data_class: INTERNAL_ONLY
    },
    /// `<callee>(<args>)`
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
    /// `todo!()` — an unimplemented body, emitted only where a construction asked for a stub.
    Todo,
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
            | Self::Call { .. }
            | Self::MethodCall { .. }
            | Self::Index { .. }
            | Self::StructLiteral { .. }
            | Self::SelfValue
            | Self::Match { .. }
            | Self::Todo => Precedence::ATOMIC,
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

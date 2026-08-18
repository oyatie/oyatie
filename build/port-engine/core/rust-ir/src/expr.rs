//! Statements and expressions, with precedence decided by the TREE.
//!
//! The previous IR built bodies with `format!` and parenthesised every binary expression
//! unconditionally, because a text builder cannot see its own nesting and a wrong precedence
//! table produces arithmetic that compiles and computes something else. A tree can see it: a
//! child is parenthesised exactly when its own precedence binds looser than the position it sits
//! in, so `a + b * c` emits without parentheses and `(a + b) * c` keeps the one it needs.

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
    /// `todo!()` — an unimplemented body, emitted only where a construction asked for a stub.
    Todo,
}

/// Binary operators, grouped by the precedence the target language gives them.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BinaryOp {
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    /// `&`
    BitAnd,
    /// `^`
    BitXor,
    /// `|`
    BitOr,
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `&&`
    And,
    /// `||`
    Or,
}

/// Where each operator sits in the target's precedence order. Higher binds tighter.
///
/// Taken from the Rust reference's expression-precedence table rather than from the source
/// language's, because this is what the EMITTED text will be parsed as. Getting it from the wrong
/// language is precisely the defect the unconditional parentheses were avoiding.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Precedence(u8);

/// Comparison operators are non-associative in Rust: `a == b == c` does not parse, so a comparison
/// nested inside a comparison always needs parentheses regardless of level.
const COMPARISON: Precedence = Precedence(3);

impl BinaryOp {
    /// The operator's precedence level.
    #[must_use]
    pub const fn precedence(self) -> Precedence {
        match self {
            Self::Mul | Self::Div | Self::Rem => Precedence(9),
            Self::Add | Self::Sub => Precedence(8),
            Self::Shl | Self::Shr => Precedence(7),
            Self::BitAnd => Precedence(6),
            Self::BitXor => Precedence(5),
            Self::BitOr => Precedence(4),
            Self::Eq | Self::Ne | Self::Lt | Self::Le | Self::Gt | Self::Ge => COMPARISON,
            Self::And => Precedence(2),
            Self::Or => Precedence(1),
        }
    }

    /// The operator's spelling.
    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Shl => "<<",
            Self::Shr => ">>",
            Self::BitAnd => "&",
            Self::BitXor => "^",
            Self::BitOr => "|",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::And => "&&",
            Self::Or => "||",
        }
    }

    /// `true` when this operator may not be nested inside another of the same precedence without
    /// parentheses.
    #[must_use]
    pub const fn is_non_associative(self) -> bool {
        matches!(self.precedence().0, 3)
    }
}

/// Prefix operators.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UnaryOp {
    /// Arithmetic negation.
    Neg,
    /// Logical or bitwise NOT — one operator in the target, distinguished by operand type.
    Not,
}

impl UnaryOp {
    /// The operator's spelling.
    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Neg => "-",
            Self::Not => "!",
        }
    }

    /// Prefix operators bind tighter than every binary operator.
    #[must_use]
    pub const fn precedence() -> Precedence {
        Precedence(11)
    }
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
            Self::If { .. } => Precedence(0),
            Self::Literal(_) | Self::Path(_) | Self::Block(_) | Self::Tuple(_) | Self::Todo => {
                Precedence(u8::MAX)
            }
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

//! The target's operator tables, and the precedence they carry.
//!
//! Taken from the TARGET language's precedence order, never the source's. The two disagree — Go
//! gives `&&` and `||` different levels from Rust's, and puts the bitwise operators in a different
//! band entirely — and the emitted text is parsed as Rust, so a table borrowed from the source
//! would reassociate arithmetic that compiles either way.

use crate::expr::RustExpr;

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

impl Precedence {
    /// Binds looser than every operator, so an expression carrying it is parenthesised in any
    /// operand position rather than silently reassociated.
    pub const LOOSEST: Self = Self(0);
    /// Binds tighter than every operator: a literal, a path, or any already-bracketed form.
    pub const ATOMIC: Self = Self(u8::MAX);
}

/// Comparison operators are non-associative in Rust: `a == b == c` does not parse, so a comparison
/// nested inside a comparison always needs parentheses regardless of level.
const COMPARISON: Precedence = Precedence(3);

impl BinaryOp {
    /// Whether this operator COMPARES, which is the set whose left operand a cast must be bracketed
    /// under. `<` and `<=` are the ones the grammar actually trips on — the target reads `T <` as
    /// generic arguments — and the rest are included because a reader cannot be asked to remember
    /// which two of six comparisons need brackets and which four do not.
    #[must_use]
    pub const fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Eq | Self::Ne | Self::Lt | Self::Le | Self::Gt | Self::Ge
        )
    }

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

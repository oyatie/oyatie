//! Operator and identifier tables.
//!
//! Every table here is TOTAL over what it admits and returns `None` otherwise, so an operator with
//! no faithful target form reaches a refusal instead of a plausible substitute.

use port_engine_api::Declaration;
use port_engine_rust_ir::{BinaryOp, RustExpr, UnaryOp};

use crate::body::Body;
use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_screaming_snake, to_snake_case};
use crate::resolve::Resolver;
use crate::vocabulary::{ATTR_OP, ATTR_REF, CHILD_RESULT, SOURCE_STRING};

/// Case an identifier by what it REFERS to.
///
/// A reference to a constant must render in the target's constant casing or it names nothing at
/// all, so the front end's classification is used rather than one default applied to everything.
pub(crate) fn reference(node: &Declaration) -> String {
    match node.attr(ATTR_REF) {
        Some("const") => to_screaming_snake(&node.name),
        Some("type") => to_pascal_case(&node.name),
        _ => to_snake_case(&node.name),
    }
}

/// `true` when this identifier is the enclosing method's receiver, whose target spelling is not
/// its name.
pub(crate) fn is_receiver(node: &Declaration) -> bool {
    node.attr(ATTR_REF) == Some("receiver")
}

pub(crate) fn binary_operator(spelling: &str) -> Option<BinaryOp> {
    Some(match spelling {
        "+" => BinaryOp::Add,
        "-" => BinaryOp::Sub,
        "*" => BinaryOp::Mul,
        "/" => BinaryOp::Div,
        "%" => BinaryOp::Rem,
        "==" => BinaryOp::Eq,
        "!=" => BinaryOp::Ne,
        "<" => BinaryOp::Lt,
        "<=" => BinaryOp::Le,
        ">" => BinaryOp::Gt,
        ">=" => BinaryOp::Ge,
        "&&" => BinaryOp::And,
        "||" => BinaryOp::Or,
        "&" => BinaryOp::BitAnd,
        "|" => BinaryOp::BitOr,
        "^" => BinaryOp::BitXor,
        "<<" => BinaryOp::Shl,
        ">>" => BinaryOp::Shr,
        // `&^` (AND NOT) has no single-operator target form. It is spellable as `& !`, but the
        // operand widths differ between the languages and a silent rewrite of a bit operation is
        // exactly the class of change nobody reviews.
        _ => return None,
    })
}

pub(crate) fn unary_operator(spelling: &str) -> Option<UnaryOp> {
    Some(match spelling {
        "-" => UnaryOp::Neg,
        // Logical NOT and bitwise NOT are both `!` in the target, distinguished by operand type
        // rather than by spelling.
        "!" | "^" => UnaryOp::Not,
        // `&` and `*` are references and dereferences. Both are aliasing decisions, which
        // docs/programs/k8s-port/census/ownership-escape.md exists to work out.
        _ => return None,
    })
}

/// Whether this signature's single result is the OWNED target for a source string.
///
/// Single result only. Several results leave as a tuple, and which member a literal lands in is a
/// question about position inside the tuple that this does not answer — so it says no rather than
/// answering for the wrong member.
pub(crate) fn returns_owned_string(declaration: &Declaration, resolver: &Resolver<'_>) -> bool {
    let results = declaration.children_of_kind(CHILD_RESULT);
    let [result] = results.as_slice() else {
        return false;
    };
    if result.type_ref.name != SOURCE_STRING {
        return false;
    }
    resolver.owns_strings()
}

/// Own a bare string literal being RETURNED, where the signature says the result is owned.
///
/// A source string literal has type `string`, which the pack maps to an owned `String`; the
/// target's literal is a borrowed `&'static str` and does not fit. This is the case the compile
/// proof caught — `fn describe() -> String { "globals" }`.
///
/// Applied only in return position, and only when the single result resolves to the owned target,
/// because that is the one place the destination is in hand. An earlier attempt owned EVERY string
/// literal: it compiled, and it produced `s == "".to_owned()` and `from("empty".to_owned())` —
/// correct output that no reader would accept. A borrowed literal in a borrowed position is
/// already right, and only the owned positions were ever wrong.
///
/// Everywhere else the destination is a parameter or a comparison operand, which is the same
/// question unary `&` is blocked on and needs the signature table rather than a guess.
///
/// `.to_owned()` rather than `.to_string()`: both allocate, and `to_owned` names a borrow-to-owned
/// conversion rather than a formatting call. That spelling is an IDIOM decision living in code for
/// now and belongs in pack data with the rest of the idiom rules.
pub(crate) fn own_returned_string(expr: RustExpr, cx: &Body<'_>) -> RustExpr {
    match cx.result_is_owned_string {
        true => own_string(expr),
        false => expr,
    }
}

/// Own a bare string literal whose destination is the owned string target.
///
/// The destination comes from the signature table, and the comparison is spelling EQUALITY against
/// what the pack maps `string` to — never a pattern match on the spelling, because `Option<Box<T>>`
/// and `String` are strings today and deciding by their shape would break the moment a form is
/// spelled differently for the same decision.
pub(crate) fn own_string_for(expr: RustExpr, wanted: &str, cx: &Body<'_>) -> RustExpr {
    match cx.resolver.owned_string_target().is_some_and(|owned| owned == wanted) {
        true => own_string(expr),
        false => expr,
    }
}

fn own_string(expr: RustExpr) -> RustExpr {
    let RustExpr::Literal(text) = &expr else {
        return expr;
    };
    if !text.starts_with('"') {
        return expr;
    }
    RustExpr::Literal(format!("{text}.to_owned()"))
}

/// Refuse a body that reads a declaration the pack DEFERS.
///
/// What the engine emits has to be self-contained. A deferred declaration is not emitted, so a
/// body naming it produces a crate with a dangling name — which the compile proof catches and
/// which no amount of parsing would.
///
/// The mapping from a reference KIND to the declaration kind it names is spelled here because it
/// is the one place both vocabularies meet: the front end classifies an identifier by what the
/// type-checker says it resolves to, and the pack defers by declaration kind.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the declaration and the dependency it cannot have.
pub(crate) fn refuse_deferred_reference(
    node: &Declaration,
    cx: &Body<'_>,
) -> Result<(), TransformError> {
    let kind = match node.attr(ATTR_REF) {
        Some("package_var") => "var",
        _ => return Ok(()),
    };
    if !cx.resolver.deferred.contains(kind) {
        return Ok(());
    }
    Err(TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "reads `{}`, a package-scope `{kind}` the pack defers; what is emitted has to be \
             self-contained, so this refuses rather than naming a declaration that is not there",
            node.name
        ),
    })
}

/// Why a unary operator was refused, in the operator's own terms.
///
/// `&` and `*` are NOT missing a spelling — the target has both. What is missing is the
/// DESTINATION: `&x` yields a pointer, and which target form that pointer takes is the same
/// ownership decision the pack already answers for a `*T` type position. The answer depends on the
/// position the value flows into, and the body translator does not know it.
///
/// Measured rather than assumed, over the seven surveyed third-party corpora: of 33 `&` sites, 11
/// are `f(&x)` — where the destination is the CALLEE's parameter — 7 are `x := &T{..}`, 4 are
/// `return &T{..}`, 3 are `x = &T{..}`, 3 are `return &x`, and the rest are unsupported operands.
/// Every one of them resolves against a signature the engine has already translated, so what
/// unblocks this is a signature table, not a rule. Choosing a form without one would be the guess
/// this engine exists to refuse.
pub(crate) fn unary_refusal(spelling: &str) -> String {
    match spelling {
        "&" | "*" => format!(
            "unary `{spelling}` in a non-argument position: an ARGUMENT is answered by the \
             signature table, which gives the parameter's disposition and the construction that \
             feeds it. A `let`, a `return` or an assignment is not — the local's or the result's \
             target type has to reach the expression walk first"
        ),
        other => format!("unary operator `{other}` has no target form"),
    }
}

pub(crate) fn operator_of<'a>(
    node: &'a Declaration,
    cx: &Body<'_>,
) -> Result<&'a str, TransformError> {
    node.attr(ATTR_OP)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: node.kind.clone(),
            name: cx.owner.to_owned(),
            datum: ATTR_OP,
        })
}

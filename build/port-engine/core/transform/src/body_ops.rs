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
use crate::vocabulary::{
    ATTR_OP, ATTR_REF, CHILD_RESULT, REF_CONST, REF_PACKAGE, SOURCE_STRING,
};

/// Case an identifier by what it REFERS to.
///
/// A reference to a constant must render in the target's constant casing or it names nothing at
/// all, so the front end's classification is used rather than one default applied to everything.
///
/// A PREDECLARED constant is the exception, and it is not a casing question: `true` is a
/// universe-scope constant of the source, so it arrives classified as a constant, and casing it
/// would emit `TRUE` — a name nothing declares. The pack's constant map is consulted first, which
/// is also where a predeclared constant the pack does not list refuses instead of being invented.
pub(crate) fn reference(node: &Declaration, resolver: &Resolver<'_>) -> String {
    match node.attr(ATTR_REF) {
        Some(REF_CONST) => match resolver.constant_map.get(&node.name) {
            Some(spelling) => spelling.clone(),
            None => to_screaming_snake(&node.name),
        },
        Some("type") => to_pascal_case(&node.name),
        _ => to_snake_case(&node.name),
    }
}

/// `true` when this identifier is the enclosing method's receiver, whose target spelling is not
/// its name.
pub(crate) fn is_receiver(node: &Declaration) -> bool {
    node.attr(ATTR_REF) == Some("receiver")
}

/// Whether this binary node compares a LENGTH CONSTANT against a length.
///
/// Both sides are then the target's index type, so the conversion the length call's mapping adds is
/// what is wrong — the same question the loop counter asks, at the one other place two values of
/// that type meet. The constant and the comparison read the SAME proof, so a guard cannot end up
/// comparing two different types.
pub(crate) fn compares_lengths(node: &Declaration, cx: &Body<'_>) -> bool {
    let [left, right] = node.children.as_slice() else {
        return false;
    };
    [(left, right), (right, left)].iter().any(|(a, b)| {
        a.kind == crate::vocabulary::KIND_IDENT
            && cx.resolver.scope.length_constants.contains(&a.name)
            && b.kind == crate::vocabulary::KIND_CALL
    })
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
        // `<<` and `>>` are DELIBERATELY ABSENT, and this is the one place the absence is
        // visible. The source defines a shift at or beyond the operand width as ZERO and panics on
        // a negative count; the target panics on the first in a debug build and masks the count in
        // a release one. Three behaviours where the source has two, none of them matching — output
        // that compiles and means something different, which is the failure this engine exists to
        // prevent.
        //
        // Emitting the plain operator was a knowing gap while the wrapping policy was written, and
        // a reviewer reading the emitted crate found it by executing it: a public `shift(n, by)`
        // aborts for `by >= 64` and for any negative `by`, where the source returns zero and
        // panics respectively. A gap that emits is not a gap, it is a defect.
        //
        // Refused by name until the pack declares the form. `checked_shl(..).unwrap_or(0)` is the
        // zero half and says nothing about the negative half, and `census/` sizes no numeric
        // family at all — which the standing brief already says, and which is now the second time
        // it has cost something.
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
        true => own_string(expr, cx),
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
        true => own_string(expr, cx),
        false => expr,
    }
}

fn own_string(expr: RustExpr, cx: &Body<'_>) -> RustExpr {
    // A SLICE of a string is a string in the source and a borrow in the target. `v.original[0:1]`
    // is a value the source hands back, and the target's `&s[..1]` borrows what `s` owns — so a
    // result position that wants an owned string has to own it. This owns the SLICE rather than
    // what it slices, which is the whole point: the substring is the value, not the original.
    if matches!(expr, RustExpr::Slice { .. })
        && let Some(owned) = cx.resolver.owned_string_target()
    {
        // CONSTRUCTED rather than converted with a method. A slice renders with its own leading
        // borrow, so a method call on it binds tighter than the borrow does — `&s[..1].to_owned()`
        // is a reference to an owned string, which is not what the signature asked for. Naming the
        // owned type takes the precedence question away entirely.
        return RustExpr::Call {
            callee: Box::new(RustExpr::Path(format!("{owned}::from"))),
            args: vec![expr],
        };
    }
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
    // A PACKAGE NAME is not a value, and the emitted crate has a module for one only if the model
    // has that unit. `binary.LittleEndian.Uint64(b)` came out as `binary.little_endian.uint64(b)`,
    // a path into a crate the output does not have — the same self-containment defect as a call
    // into a foreign package, reached through a selector rather than through a callee identity.
    if node.attr(ATTR_REF) == Some(REF_PACKAGE) && !cx.resolver.units.contains(&node.name) {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "reads through package `{}`, which this snapshot does not contain — the emitted \
                 crate has no module for it, and the pack has to map what is reached through it",
                node.name
            ),
        });
    }

    let kind = match node.attr(ATTR_REF) {
        Some("package_var") => "var",
        // A CONSTANT is a package-scope name too, and reading one the crate does not contain is the
        // same dangling reference. It was never checked because the deferral this function grew out
        // of was about variables — and a constant's DERIVATION reads other constants, which is what
        // made it visible: `byteLength = timestampLengthInBytes + payloadLengthInBytes` named two
        // that had refused.
        // ...unless the pack MAPS it, which is what a predeclared constant is. `true` is classified
        // as a constant reference like any other, and it is not this unit's to emit — asking the
        // unit for it refused every declaration that mentions a boolean.
        Some(REF_CONST) if !cx.resolver.constant_map.contains_key(&node.name) => "const",
        _ => return Ok(()),
    };
    // NOT EMITTED, whichever way: the pack defers the kind, or the declaration itself refused.
    // Both leave the crate without the name, and a body that uses one is not self-contained.
    if !cx.resolver.deferred.contains(kind) && cx.resolver.emitted.contains(&node.name) {
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

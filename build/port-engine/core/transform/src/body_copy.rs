//! The source's `copy` builtin, which the target spells as two operations.
//!
//! `copy(dst, src)` copies `min(len(dst), len(src))` elements and answers with that count. The
//! target has no single call that does it: `copy_from_slice` requires the two slices to be the
//! same length and says nothing about how many it moved. So the count is computed first and both
//! sides are cut to it, which is the same rule the source applies and the reason this is a BLOCK
//! rather than a call.
//!
//! Two things make it faithful, and both are checked rather than assumed:
//!
//! - Each argument is named TWICE in the target — once for its length and once for the slice — and
//!   the source names it once. That is only safe for an expression that can be evaluated twice
//!   without doing anything, so anything else refuses.
//! - The count is bound to a name, and a name binds over the expressions that follow it. An
//!   argument mentioning that name would read the binding rather than its own value.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt};

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::vocabulary::KIND_CALL;

/// The source's assignment statement.
const KIND_ASSIGN: &str = "assign";
/// The source's `x++` / `x--`.
const KIND_INCDEC: &str = "incdec";

/// What the copied count is bound to.
const COUNT: &str = "copied";

/// Translate the source's `copy`, or say this is not one.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the declaration and the shape that is not answered.
pub(crate) fn slice_copy(
    node: &Declaration,
    callee: &str,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    if callee != "copy" {
        return Ok(None);
    }
    let refuse = |why: &str| {
        Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "`copy` copies the smaller of its two lengths and answers with it, which the \
                 target spells as a length and a slice assignment rather than as a call — and {why}"
            ),
        })
    };
    let [_, destination, source] = node.children.as_slice() else {
        return refuse("this call does not have the two arguments that form takes").map(Some);
    };
    // A STRING source is legal for the source's `copy` and is a different operation: its bytes are
    // not the target's, and the element types do not line up.
    for argument in [destination, source] {
        if !repeatable(argument) {
            return refuse(
                "an argument here is not an expression the target can name twice without \
                 re-running it — the length and the slice each name it once, and the source \
                 evaluates it once",
            )
            .map(Some);
        }
    }

    let (destination, source) = (expression(destination, cx)?, expression(source, cx)?);
    // THE DESTINATION IS WRITTEN, so it has to be a place — something that denotes storage the
    // write reaches. What the source spelled is not enough to know that: ownership may put a
    // `.clone()` in the emitted form, and `self.uuid.clone()[..copied].copy_from_slice(..)`
    // compiles, writes into a temporary, and drops it. That is a copy that does nothing, and it is
    // the exact failure this engine exists to prevent, so it is checked on the EMITTED form.
    if !is_place(&destination) {
        return refuse(
            "the destination does not name storage in the target — the emitted form builds a              value, and a copy into a value writes into something nothing can read afterwards",
        )
        .map(Some);
    }
    for rendered in [&destination, &source] {
        if mentions(rendered, COUNT) {
            return refuse(
                "an argument mentions the name the count is bound to, which would read the count \
                 instead of the argument",
            )
            .map(Some);
        }
    }

    let length_of = |value: &RustExpr| RustExpr::MethodCall {
        receiver: Box::new(value.clone()),
        method: "len".to_owned(),
        args: Vec::new(),
    };
    let prefix = |value: &RustExpr| RustExpr::Index {
        base: Box::new(value.clone()),
        index: Box::new(RustExpr::Range {
            // NO LOWER BOUND: a prefix starts where the slice does, and `..copied` says so.
            start: None,
            end: Box::new(RustExpr::Path(COUNT.to_owned())),
            inclusive: false,
        }),
    };

    Ok(Some(RustExpr::Block(vec![
        // `let copied = dst.len().min(src.len());`
        RustStmt::Let {
            name: COUNT.to_owned(),
            mutable: false,
            ty: None,
            value: Some(RustExpr::MethodCall {
                receiver: Box::new(length_of(&destination)),
                method: "min".to_owned(),
                args: vec![length_of(&source)],
            }),
        },
        // `dst[..copied].copy_from_slice(&src[..copied]);`
        RustStmt::Semi(RustExpr::MethodCall {
            receiver: Box::new(prefix(&destination)),
            method: "copy_from_slice".to_owned(),
            args: vec![RustExpr::Reference {
                mutable: false,
                inner: Box::new(prefix(&source)),
            }],
        }),
        // The count, because the source's `copy` answers with it. A caller that ignores the answer
        // ignores this the same way.
        RustStmt::Tail(RustExpr::Path(COUNT.to_owned())),
    ])))
}

/// Whether naming this expression twice is the same as naming it once.
///
/// Stated as what it EXCLUDES rather than what it admits. Naming a value, a field, an index, a
/// slice bound or a literal a second time reads the same thing again; the hazards are the nodes
/// that DO something — a call, an assignment, an increment — and listing those is both shorter and
/// safer than enumerating everything harmless, where a form left off the list refuses a copy that
/// was fine. `uuid` slices with a literal bound, and an admitting list that forgot literals
/// refused four of them.
fn repeatable(node: &Declaration) -> bool {
    !matches!(node.kind.as_str(), KIND_CALL | KIND_ASSIGN | KIND_INCDEC)
        && node.children.iter().all(repeatable)
}

/// Whether this emitted expression denotes STORAGE rather than a value.
///
/// A name, a field of one, and an index or slice of either are places: writing through them reaches
/// something that outlives the statement. A call is not — it produces a temporary — and `.clone()`
/// is the call that matters here, because ownership inserts it and a copy into a clone is a copy
/// into nothing.
fn is_place(value: &RustExpr) -> bool {
    match value {
        RustExpr::Path(_) => true,
        RustExpr::Field { base, .. } => is_place(base),
        RustExpr::Index { base, .. } => is_place(base),
        RustExpr::Reference { inner, .. } => is_place(inner),
        _ => false,
    }
}

/// The same slice, without an index that selects the whole of it.
///
/// The source writes `u[:]` to turn an array into a slice, and the target's own indexing does that
/// where it is needed — so carrying the source's full-range index across leaves `node[..][..n]`,
/// which is the same slice named twice.
fn without_full_range(value: RustExpr) -> RustExpr {
    match &value {
        RustExpr::Index { base, index } => match index.as_ref() {
            RustExpr::Range {
                start: None,
                end,
                inclusive: false,
            } if matches!(end.as_ref(), RustExpr::Literal(text) if text.is_empty()) => {
                (**base).clone()
            }
            _ => value,
        },
        _ => value,
    }
}

/// Whether rendered target text names this identifier.
///
/// Compared on WORD BOUNDARIES: `copied` and `copied_bytes` are different names, and a substring
/// test would confuse them.
fn mentions(value: &RustExpr, name: &str) -> bool {
    let text = format!("{value:?}");
    text.match_indices(name).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + name.len()..].chars().next();
        let boundary = |c: Option<char>| !c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        boundary(before) && boundary(after)
    })
}

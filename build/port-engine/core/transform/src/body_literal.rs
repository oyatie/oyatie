//! COMPOSITE literals: a struct's fields, or a sequence's elements.
//!
//! One module because the source spells both with the same production and tells them apart by the
//! type behind it — a struct names its fields, a sequence gives its elements in order. Separated
//! from the expression walk because each carries a question the walk does not: which fields a
//! literal LEFT OUT, and what the target spells for a sequence of a given kind.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_call::render_operand;
use crate::body_expr::expression;
use crate::body_parts::one_child;
use crate::body_place::moves_on_read;
use crate::error::TransformError;
use crate::naming::to_snake_case;
use std::collections::BTreeMap;

use crate::vocabulary::{ATTR_READ_COUNT, FLAG_REREAD};
/// rather than something inferred from a spelling here.
/// A field's value, cloned when the binding it reads is read AGAIN.
///
/// A struct literal's field takes OWNERSHIP of what it is given. The source copies a value on
/// every read and the target moves it, so `T{a: x, b: x}` is two copies in the source and a use
/// after move in the target — which is exactly what the compile proof caught.
///
/// Only here, and only for a binding the front end counted more than one read of. A read is not
/// automatically a move: `x.len()` borrows, and cloning for a length would be the needless
/// allocation a reviewer flags. Which argument positions take ownership is a question for the
/// signature table; a struct field always does.
///
/// # Errors
/// [`TransformError`] from translating the value.
fn owned_read(
    node: &Declaration,
    cx: &Body<'_>,
    is_last_read: bool,
) -> Result<RustExpr, TransformError> {
    let value = expression(node, cx)?;

    // A BORROWED parameter reaching a field that owns. The source shared its string and its slice
    // with the caller, so nothing there had to say this; the target's field owns, so the borrow
    // has to become one. `to_owned` rather than `clone`, because `clone` on a `&str` yields a
    // `&str` and the field wants the owned form.
    if node.kind == "ident" && cx.borrowed.contains(&node.name) {
        return Ok(RustExpr::MethodCall {
            receiver: Box::new(value),
            method: "to_owned".to_owned(),
            args: Vec::new(),
        });
    }

    if is_last_read
        || node.kind != "ident"
        || !node.has_flag(FLAG_REREAD)
        || !moves_on_read(&node.type_ref, cx)
    {
        return Ok(value);
    }
    Ok(RustExpr::MethodCall {
        receiver: Box::new(value),
        method: "clone".to_owned(),
        args: Vec::new(),
    })
}

/// A struct literal, with every field named.
///
/// Go zero-fills the fields a literal omits; the target has no such rule and rejects an incomplete
/// literal outright. The front end therefore emits one entry per DECLARED field, and the omitted
/// ones arrive as `zero` nodes carrying the field's type — so the target's zero is a pack answer
pub(crate) fn composite(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    // A SEQUENCE literal is a different construction from a struct one, and the type's kind is
    // what says which. A struct names its fields; a sequence gives its elements in order.
    if let Some(rendered) = sequence_literal(node, cx)? {
        return Ok(rendered);
    }

    // The resolver's OWN error, not a substituted one. This used to report a missing datum --
    // "`composite` needs `type`, which the front end did not record" -- for every literal whose type
    // the resolver could not map, and the type was recorded in every one of them. It was the largest
    // single refusal in the corpus by both count and package spread, and it named the wrong
    // component: a reader following it went to the front end, where there was nothing to find.
    // A refusal that misdescribes what is missing is worse than no refusal, because it is acted on.
    // A composite of a local NEWTYPE is the wrapper around a literal of what it WRAPS. The source
    // writes `uint128{high, low}` because there the name and the array are one thing; the target's
    // newtype has one unnamed field holding the array.
    //
    // This is the case that produced a silent loss rather than a compile error: the literal's
    // elements are positional, the struct path looks for KEYED children, found none, and emitted
    // `Uint128 {}` — both operands gone. It was caught only because the target type has a field to
    // be missing. See the arity guard below, which is what makes the class impossible rather than
    // this one instance fixed.
    if let Some(underlying) = cx.resolver.scope.newtypes.get(&node.type_ref.name).cloned() {
        let inner = Declaration {
            type_ref: underlying,
            ..node.clone()
        };
        if let Some(rendered) = sequence_literal(&inner, cx)? {
            return Ok(RustExpr::Call {
                callee: Box::new(RustExpr::Path(
                    cx.resolver.resolve(&node.type_ref, cx.owner)?.spelling(),
                )),
                args: vec![rendered],
            });
        }
    }

    // An EMPTY literal of a local NEWTYPE is the wrapper around what it wraps, zeroed. The source
    // writes `uint128{}` and means two zero words; the target's newtype has one unnamed field, so a
    // struct literal with no fields names none of it and does not compile. The type's own zero is
    // the answer for the same reason `[20]byte{}` is twenty zero bytes rather than an empty array.
    if node.children.is_empty()
        && let Some(underlying) = cx.resolver.scope.newtypes.get(&node.type_ref.name).cloned()
        && let Some(zero) = cx.resolver.zero_value(&underlying)
    {
        return Ok(RustExpr::Call {
            callee: Box::new(RustExpr::Path(
                cx.resolver.resolve(&node.type_ref, cx.owner)?.spelling(),
            )),
            args: vec![RustExpr::Literal(zero)],
        });
    }

    let path = cx.resolver.resolve(&node.type_ref, cx.owner)?;

    let keyed = node.children_of_kind("keyed");
    // Where a literal holds EVERY read of a binding, its final read can move — nothing follows it.
    // Counted rather than tracked: liveness would say which read is last on every path, and this
    // says it for the one construction that can answer without.
    let contained = fully_contained_reads(&keyed);
    let mut fields = Vec::with_capacity(keyed.len());
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for entry in &keyed {
        let value = one_child(entry, cx, "keyed")?;
        let occurrence = *seen
            .entry(value.name.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        let is_last = contained
            .get(&value.name)
            .is_some_and(|total| occurrence == *total);
        fields.push((to_snake_case(&entry.name), owned_read(value, cx, is_last)?));
    }
    // A literal the source gave OPERANDS may not come out with none. Nothing else in this function
    // can tell the difference between a struct with no fields and a struct whose fields were all
    // dropped, and the second is a silent loss of every value the source wrote — which is the exact
    // failure this engine exists to prevent, and which reached the output once.
    if fields.is_empty() && !node.children.is_empty() {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "a composite literal of `{}` carries {} operand(s) in the source and none of them \
                 survived translation — the target literal would construct the type while dropping \
                 every value the source put in it",
                node.type_ref.describe(),
                node.children.len()
            ),
        });
    }
    Ok(RustExpr::StructLiteral {
        path: path.spelling(),
        fields,
    })
}

/// A slice, array or map literal, when the pack says what its kind becomes.
///
/// EMPTY is answered by the type's ZERO rather than by an empty construction. `[20]byte{}` is not
/// an empty array — it is twenty zero bytes, and the target spells that `[0u8; 20]`, which is
/// exactly the type's zero value. Answering it any other way would need the engine to invent a
/// length it already has.
///
/// A kind the pack declares no constructor for refuses through the ordinary path: a map literal is
/// the case, because the source's map has no order and the target's ordered map imposes one, which
/// makes the entry order observable where it was not.
///
/// # Errors
/// [`TransformError`] from translating an element.
fn sequence_literal(node: &Declaration, cx: &Body<'_>) -> Result<Option<RustExpr>, TransformError> {
    let kind = node.type_ref.kind.as_str();
    if !cx.resolver.is_sequence_literal(kind) {
        return Ok(None);
    }
    if node.children.is_empty() {
        return zero_value(node, cx).map(Some);
    }

    let mut elements = Vec::with_capacity(node.children.len());
    for child in &node.children {
        elements.push(render_element(&expression(child, cx)?, cx)?);
    }
    let Some(rendered) = cx.resolver.sequence_form(kind, &elements) else {
        return Ok(None);
    };
    Ok(Some(RustExpr::Literal(rendered)))
}

/// One element of a sequence literal, as target text for the constructor to join.
///
/// Only forms whose text is unambiguous are admitted, for the same reason the mapped-call template
/// admits only those: a constructor is textual joining, and an element that reassociates would
/// need brackets this cannot see the need for.
fn render_element(expr: &RustExpr, cx: &Body<'_>) -> Result<String, TransformError> {
    render_operand(expr).ok_or_else(|| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: "an element of a sequence literal is a compound expression, and the pack builds \
                 that literal from a TEXT template — substituting one would need parentheses the \
                 template cannot ask for"
            .to_owned(),
    })
}

/// The target's zero for a source type the literal left out.
///
/// Refuses BY NAME rather than reaching for `Default::default()`. That would compile for the types
/// the corpus has and would silently be a different program for the ones it does not: a type whose
/// `Default` impl is not its zero value, or one that has no `Default` at all and only fails much
/// later, in the emitted crate, with the source declaration nowhere in the message.
pub(crate) fn zero_value(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    cx.resolver
        .zero_value(&node.type_ref)
        .map(RustExpr::Literal)
        .ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "a struct literal omits field `{}` of type `{}`, and the pack declares no zero \
                 value for it — Go fills the field with that type's zero and the target must \
                 spell it out",
                node.name,
                node.type_ref.describe()
            ),
        })
}

/// Bindings whose EVERY read in the body happens inside this literal, and how many reads that is.
///
/// A binding read three times in the body and twice here has a third read elsewhere, so neither
/// read here is the last one and both must clone. A binding read twice in the body and twice here
/// is fully contained, so the second is the last and can take the value.
///
/// The front end records the body-wide count; this counts the occurrences here. Nothing else is
/// needed, which is why this is counting rather than liveness — and why it answers for exactly one
/// construction rather than for the body.
fn fully_contained_reads(entries: &[&Declaration]) -> BTreeMap<String, usize> {
    let mut here: BTreeMap<String, usize> = BTreeMap::new();
    for entry in entries {
        let Some(value) = entry.children.first() else {
            continue;
        };
        if value.kind != "ident" || !value.has_flag(FLAG_REREAD) {
            continue;
        }
        *here.entry(value.name.clone()).or_insert(0) += 1;
    }
    here.retain(|name, count| {
        entries
            .iter()
            .filter_map(|entry| entry.children.first())
            .find(|value| &value.name == name)
            .and_then(|value| value.attr(ATTR_READ_COUNT))
            .and_then(|total| total.parse::<usize>().ok())
            .is_some_and(|total| total == *count)
    });
    here
}

/// A rune literal, spelled for the type the source RESOLVED it to.
///
/// The source's `'0'` is an untyped constant, and what it means is decided by its context: in
/// `b >= '0'` where `b` is a byte it IS a byte, and in `r == '0'` where `r` is a rune it is a
/// character. The target spells those two differently — `b'0'` and `'0'` — and it has no untyped
/// constant to defer the choice to.
///
/// Passing the source spelling through emitted a character in both cases. That is not the harmless
/// outcome the pass-through was documented to have: a character where a byte is wanted is not a
/// parse failure, it is a well-formed expression of the wrong type, and it reached `rustc` as
/// eleven type errors in one package.
///
/// The spelling is chosen from the type the PACK resolves the literal to, not from the source's
/// name for it, so a pack that maps the source's byte somewhere else gets that answer here too.
/// Anything the target cannot spell as a literal of the wanted type refuses BY NAME.
pub(crate) fn typed_literal(
    node: &Declaration,
    cx: &Body<'_>,
    value: &str,
) -> Option<Result<RustExpr, TransformError>> {
    match node.attr(crate::vocabulary::ATTR_LIT_KIND) {
        Some("CHAR") => rune_literal(node, cx, value),
        Some("INT") => whole_number_literal(node, cx, value),
        _ => None,
    }
}

/// A whole-number literal the source RESOLVED to a floating-point type.
///
/// The source's `9007199254740991` is an untyped constant and takes its type from context, so in
/// `f < -9007199254740991` where `f` is a float it IS a float. The target has no untyped constant:
/// a literal with no decimal point is an integer there, and comparing one to a float does not
/// compile. The decimal point is the whole of the fix and it changes no value.
///
/// Only the FLOAT case is handled. An integer literal that resolved to an integer type is left
/// exactly alone — its spelling is already right, and touching it would churn every number in the
/// corpus to say the same thing.
fn whole_number_literal(
    node: &Declaration,
    cx: &Body<'_>,
    value: &str,
) -> Option<Result<RustExpr, TransformError>> {
    if node.type_ref.is_empty() {
        return None;
    }
    let resolved = match cx.resolver.resolve(&node.type_ref, cx.owner) {
        Ok(path) => path.spelling(),
        Err(_) => return None,
    };
    if resolved != "f32" && resolved != "f64" {
        return None;
    }
    // Grouped FIRST, then pointed. The grouping rule reads digits and would decline a value that
    // already carries a decimal point, so the order is not interchangeable.
    let grouped = crate::items_value::readable_literal(value, cx.resolver)
        .unwrap_or_else(|| value.to_owned());
    Some(Ok(RustExpr::Literal(format!("{grouped}.0"))))
}

fn rune_literal(
    node: &Declaration,
    cx: &Body<'_>,
    value: &str,
) -> Option<Result<RustExpr, TransformError>> {
    if node.attr(crate::vocabulary::ATTR_LIT_KIND) != Some("CHAR") {
        return None;
    }
    // An untyped literal the front end could not resolve keeps the old behaviour rather than
    // refusing: the type is recorded where the type checker had one, and its absence means this is
    // not a context that resolved it.
    if node.type_ref.is_empty() {
        return None;
    }
    let resolved = match cx.resolver.resolve(&node.type_ref, cx.owner) {
        Ok(path) => path.spelling(),
        Err(error) => return Some(Err(error)),
    };
    let refuse = |why: &str| {
        Some(Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "the source's rune literal `{value}` resolves to `{resolved}` in the target, and \
                 {why}"
            ),
        }))
    };
    match resolved.as_str() {
        // A CHARACTER, which is what the source spelled and, for the escapes the two languages
        // share, what the target spells the same way. The rest are respelled by CODE POINT.
        "char" => match crate::body_escape::shared_escape(value) {
            true => Some(Ok(RustExpr::Literal(value.to_owned()))),
            false => match crate::body_escape::rune_code_point(value) {
                Some(code) => Some(Ok(RustExpr::Literal(format!("'\\u{{{code:x}}}'")))),
                None => refuse("the target has no form for that escape and its value is not known"),
            },
        },
        "u8" => {
            // A rune outside ASCII is not one byte in either language.
            if !value.is_ascii() {
                return refuse("a rune outside ASCII is not one byte");
            }
            if crate::body_escape::shared_escape(value) {
                return Some(Ok(RustExpr::Literal(format!("b{value}"))));
            }
            match crate::body_escape::rune_code_point(value) {
                // The target's byte literal admits the whole octet through `\xHH`, so an escape it
                // does not share is spelled as the byte it denotes.
                Some(code) if code <= u32::from(u8::MAX) => {
                    Some(Ok(RustExpr::Literal(format!("b'\\x{code:02x}'"))))
                }
                Some(_) => refuse("a code point above one byte has no byte form"),
                None => refuse("the target has no form for that escape and its value is not known"),
            }
        }
        _ => refuse("the target has no literal of that type for a rune"),
    }
}

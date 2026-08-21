//! A call that builds a string from a TEMPLATE, which needs the source's template read.
//!
//! Split from `body_call.rs` because the mechanism is different in kind. A mapped call substitutes
//! rendered arguments into a template the PACK wrote, which is a text operation on a form the pack
//! controls. This one has to read the template the SOURCE wrote, translate every verb in it, and
//! establish that what comes out means the same thing — and the pack cannot write that down as a
//! form because the form is different at every call site.
//!
//! Worth its own face: building a string from a template is the most common call in real source
//! after the plain one, appearing in six of the seven surveyed packages, and every one of those
//! calls refused before this existed.
//!
//! Everything here refuses BY NAME. A verb outside the pack's closed set, a template that is not a
//! literal, an argument count that does not match — each one is a case where guessing produces a
//! program that compiles and prints something else.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::error::TransformError;
use crate::vocabulary::{ATTR_CALLEE, ATTR_VALUE, KIND_LITERAL};

/// Translate a formatting call, or say this is not one.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the declaration and what about the template it cannot do.
pub(crate) fn formatted_call(
    node: &Declaration,
    args: &[RustExpr],
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let format = cx.resolver.format_calls;
    let Some(identity) = node.attr(ATTR_CALLEE) else {
        return Ok(None);
    };
    let Some(mapping) = format.functions.get(identity) else {
        return Ok(None);
    };

    // The TEMPLATE is the first argument, and it has to be a literal. The target's formatter is a
    // macro that parses its template at compile time, so a template computed at run time cannot
    // reach it at all. Read from the SOURCE node rather than from the rendered argument, because
    // what matters is whether the source wrote a literal there.
    let template_node = node
        .children
        .get(1)
        .ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("`{identity}` was called with no template at all"),
        })?;
    if template_node.kind != KIND_LITERAL {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "the template passed to `{identity}` is not a literal, and the target's formatter \
                 is a macro that parses its template at compile time — a computed template cannot \
                 reach it. {}",
                format.literal_only_reason
            ),
        });
    }
    let raw = template_node
        .attr(ATTR_VALUE)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "format template".to_owned(),
            name: cx.owner.to_owned(),
            datum: ATTR_VALUE,
        })?;

    let content = literal_content(raw, cx, identity)?;
    let (translated, placeholders) = translate_template(&content, cx, identity)?;
    let operands = &args[1..];
    if placeholders != operands.len() {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "the template passed to `{identity}` has {placeholders} placeholders and the call \
                 passes {} arguments; the target's formatter checks that correspondence at compile \
                 time and the source does not, so a mismatch here is a defect the source hid",
                operands.len()
            ),
        });
    }

    // STRUCTURED, not assembled as text. The arguments here are ordinary expressions — a field
    // read, a method call, an index — and none of those has an unambiguous text spelling to
    // substitute. Text assembly is what made every one of these calls refuse.
    //
    // A template with NOTHING in it is not a formatting operation at all, whatever the source
    // spelled: it is the string itself, and the target owns one by saying so. Invoking the macro
    // for it would be a use of the macro that does no formatting, which the target's own lints
    // name — and which is the kind of tell that reads as generated rather than written.
    let built = match placeholders == 0 && operands.is_empty() {
        true => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Literal(quoted(&translated))),
            method: "to_owned".to_owned(),
            args: Vec::new(),
        },
        false => RustExpr::MacroCall {
            name: format.macro_name.clone(),
            template: translated,
            args: operands.to_vec(),
        },
    };
    Ok(Some(match mapping.wrapper.is_empty() {
        // Nothing wraps it: the string IS the result.
        true => built,
        false => RustExpr::Call {
            callee: Box::new(RustExpr::Path(mapping.wrapper.clone())),
            args: vec![built],
        },
    }))
}

/// The source's template as the target's, and how many values it consumes.
///
/// Walks it once. A `%` begins a verb, and the verb is looked up in the pack's CLOSED set — so a
/// width, a precision, a flag, or a verb nobody decided refuses by name rather than rendering as
/// the plain placeholder. A literal brace is doubled, because the target's template gives braces
/// the meaning the source gives `%`.
fn translate_template(
    raw: &str,
    cx: &Body<'_>,
    identity: &str,
) -> Result<(String, usize), TransformError> {
    let format = cx.resolver.format_calls;
    let bytes: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut placeholders = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        let ch = bytes[index];
        if ch == '{' || ch == '}' {
            out.push(ch);
            out.push(ch);
            index += 1;
            continue;
        }
        if ch != '%' {
            out.push(ch);
            index += 1;
            continue;
        }
        // `%%` is the source's escape for a literal percent, which the target writes plainly.
        if bytes.get(index + 1) == Some(&'%') {
            out.push('%');
            index += 2;
            continue;
        }
        let verb: String = bytes
            .get(index..index + 2)
            .ok_or_else(|| TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: format!("the template passed to `{identity}` ends in a bare `%`"),
            })?
            .iter()
            .collect();
        if !format.wrap_verb.is_empty() && verb == format.wrap_verb {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: format!(
                    "the template passed to `{identity}` uses `{verb}`. {}",
                    format.wrap_verb_reason
                ),
            });
        }
        let Some(placeholder) = format.verbs.get(&verb) else {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: format!(
                    "the template passed to `{identity}` uses `{verb}`, which the pack's verb set \
                     does not contain. {}",
                    format.verbs_reason
                ),
            });
        };
        out.push_str(placeholder);
        placeholders += 1;
        index += 2;
    }
    Ok((out, placeholders))
}

/// The CHARACTERS a source string literal stands for, with its delimiters and escapes resolved.
///
/// The front end records a literal as the source's own SPELLING, which is right for a literal that
/// passes straight through — the emitted tree is parsed, so a lexical form the target does not have
/// fails there. A template is different: it is read and rewritten, so its actual characters are
/// needed, and re-emitting the spelling put the source's own quotes inside the target's template.
///
/// The escape set is CLOSED, and it holds only what both languages spell the same way. The source's
/// `\a`, `\v`, and its octal form have no target counterpart, so they refuse by name rather than
/// being passed through to mean something else. A raw literal refuses too: its content is
/// uninterpreted in the source, and deciding what that becomes in the target is a decision nobody
/// has made.
fn literal_content(raw: &str, cx: &Body<'_>, identity: &str) -> Result<String, TransformError> {
    let refuse = |what: &str| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!("the template passed to `{identity}` {what}"),
    };
    let Some(inner) = raw
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return Err(refuse(
            "is not a quoted string literal — a raw literal's content is uninterpreted in the \
             source, and what that becomes in the target is a decision nobody has made",
        ));
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let escape = chars
            .next()
            .ok_or_else(|| refuse("ends in a bare escape"))?;
        out.push(match escape {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '\\' => '\\',
            '"' => '"',
            '\'' => '\'',
            '0' => '\0',
            other => {
                return Err(refuse(&format!(
                    "uses the escape `\\{other}`, which the two languages do not spell the same \
                     way — passing it through would put a different character in the message"
                )));
            }
        });
    }
    Ok(out)
}

/// A target string literal for a value, escaping what the target gives meaning to.
fn quoted(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

//! What SHAPE a pack mapping's form has, and the tree it becomes.
//!
//! Split from `body_call.rs` because that file dispatches a call and this reads the pack's answer.
//! The distinction that matters here is text versus tree: a form the engine can build structurally
//! stays readable to every rule downstream, and one it cannot is target text that ends them.

use port_engine_rust_ir::RustExpr;

/// A mapping form as a TREE, when the engine can build one.
///
/// A trailing conversion is a WRAPPER around the shape underneath it, not a reason to give up on
/// building one. `{0}.len() as i64` is a method call inside a cast; reading it as neither left the
/// whole form as text substitution, and text substitution cannot take a compound argument because
/// it has no way to ask for the parentheses one needs. That refusal was the single largest cause in
/// the corpus — 34 sites across 8 packages — and every one of them was a `len` of something that
/// was not a bare name.
///
/// Only a cast of the WHOLE form counts. A form whose ` as ` sits inside it —
/// `{0}.rotate_left({1} as u32)` — is not one, and is left alone by the same check
/// [`crate::body_call`] uses on the rendered text: the conversion target has to be an identifier,
/// and `u32)` is not.
pub(crate) fn structured_form(form: &str, args: &[RustExpr]) -> Option<RustExpr> {
    let Some((inner, target)) = form.rsplit_once(" as ") else {
        return structured_method(form, args);
    };
    if target.is_empty()
        || !target
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return structured_method(form, args);
    }
    Some(RustExpr::Cast {
        expr: Box::new(structured_method(inner, args)?),
        ty: port_engine_rust_ir::RustType::path(target),
    })
}

/// A mapping form as a METHOD CALL, when that is the shape it has.
///
/// `{0}.rotate_left({1})` is a receiver, a name, and arguments — a tree the rest of the engine can
/// read. Anything else stays the text substitution it always was: a form with a cast, a turbofish,
/// or a construction is not this shape, and pretending otherwise would build a tree that renders
/// differently from the form the pack wrote.
///
/// Every argument must be a bare placeholder, in order from `{1}`. A form that reorders or repeats
/// them is doing something this shape cannot express.
pub(crate) fn structured_method(form: &str, args: &[RustExpr]) -> Option<RustExpr> {
    let rest = form.strip_prefix("{0}.")?;
    let (method, tail) = rest.split_once('(')?;
    let inside = tail.strip_suffix(')')?;
    if !method
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        || method.is_empty()
    {
        return None;
    }
    let receiver = args.first()?.clone();
    let mut taken = Vec::new();
    if !inside.is_empty() {
        for (offset, placeholder) in inside.split(", ").enumerate() {
            if placeholder != format!("{{{}}}", offset + 1) {
                return None;
            }
            taken.push(args.get(offset + 1)?.clone());
        }
    }
    // Every argument the call has must be consumed, or the form is dropping one.
    (taken.len() + 1 == args.len()).then(|| RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: method.to_owned(),
        args: taken,
    })
}

/// A form as a PATH CALL: `Vec::with_capacity({0})`.
///
/// The counterpart of [`structured_method`] for the forms whose callee is a path rather than a
/// receiver. Same reason for existing: a form built as a tree stays readable to every rule
/// downstream and can take a compound argument, and one built by substituting into text can do
/// neither, because text has no way to ask for the parentheses an operand needs.
pub(crate) fn structured_call(form: &str, args: &[RustExpr]) -> Option<RustExpr> {
    let (callee, tail) = form.split_once('(')?;
    let inside = tail.strip_suffix(')')?;
    if callee.is_empty()
        || !callee
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ':')
    {
        return None;
    }
    let mut taken = Vec::new();
    if !inside.is_empty() {
        for (offset, placeholder) in inside.split(", ").enumerate() {
            if placeholder != format!("{{{offset}}}") {
                return None;
            }
            taken.push(args.get(offset)?.clone());
        }
    }
    (taken.len() == args.len()).then(|| RustExpr::Call {
        callee: Box::new(RustExpr::Path(callee.to_owned())),
        args: taken,
    })
}

/// A form as a REPEATED SEQUENCE: `vec![{1}; {0}]`.
///
/// The placeholder ORDER is read from the form rather than assumed, because this is the one form in
/// the pack whose operands are not in call order: the source's `make([]T, n)` gives the count first
/// and the target's literal gives the value first. Reading the indices means a pack that writes the
/// form the other way round gets the other tree, instead of getting this one silently.
pub(crate) fn structured_repeat(form: &str, args: &[RustExpr]) -> Option<RustExpr> {
    let inside = form.strip_prefix("vec![")?.strip_suffix(']')?;
    let (value, count) = inside.split_once("; ")?;
    let index = |placeholder: &str| -> Option<usize> {
        placeholder
            .strip_prefix('{')?
            .strip_suffix('}')?
            .parse()
            .ok()
    };
    Some(RustExpr::VecRepeat {
        value: Box::new(args.get(index(value)?)?.clone()),
        count: Box::new(counted(args.get(index(count)?)?.clone())),
    })
}

/// A repeat COUNT in the target's index type.
///
/// `vec![v; n]` takes a `usize`, and the source's count is its own integer — so the mapping that
/// renders `len` puts ` as i64` on the end and the count no longer fits. Stripping that cast gets
/// the `usize` the length already was, rather than casting it back through a second conversion.
///
/// A count that is not a length is converted instead. The source's `make` takes its own integer
/// there too, and a negative one aborts in both languages: the source panics on a negative make and
/// the target's allocation fails on the value it wraps to.
fn counted(count: RustExpr) -> RustExpr {
    match count {
        RustExpr::Cast { expr, ref ty } if *ty == port_engine_rust_ir::RustType::path("i64") => {
            *expr
        }
        RustExpr::Literal(_) => count,
        other => RustExpr::Cast {
            expr: Box::new(other),
            ty: port_engine_rust_ir::RustType::path("usize"),
        },
    }
}

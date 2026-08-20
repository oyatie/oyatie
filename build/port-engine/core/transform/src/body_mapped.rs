//! What SHAPE a pack mapping's form has, and the tree it becomes.
//!
//! Split from `body_call.rs` because that file dispatches a call and this reads the pack's answer.
//! The distinction that matters here is text versus tree: a form the engine can build structurally
//! stays readable to every rule downstream, and one it cannot is target text that ends them.

use port_engine_rust_ir::RustExpr;

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

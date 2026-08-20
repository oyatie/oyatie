//! Folding a recognised chain into the expression it computes.
//!
//! Split from the recogniser because they read different things and can disagree: that one reads the
//! SOURCE and this reads the TRANSLATION. A value the source said reads a name may arrive as opaque
//! target text, and this aborts where that happens — which is why the signature is built from what
//! this DID rather than from what the recogniser predicted.

use std::collections::BTreeMap;

use port_engine_rust_ir::{RustExpr, RustStmt};

/// The chain FOLDED into the single expression it computes.
///
/// Substitutes each step forward into the next, which is exactly what the sequence means: every
/// value depends on the one before it and on nothing else that the body changes. The conditions
/// above are what make the substitution sound — one mention per step, so nothing is dropped and
/// nothing is evaluated twice.
///
/// Returns `None` where the translated statements are not the shape the source promised, which
/// cannot happen for a body the recogniser accepted and is checked anyway rather than assumed.
pub(crate) fn fold(statements: Vec<RustStmt>, name: &str) -> Option<Vec<RustStmt>> {
    let (last, leading) = statements.split_last()?;
    // What each name HOLDS, as the chain is walked forward. A statement's value is rewritten with
    // everything already known before it is recorded, so by the last one the returned name holds the
    // whole computation and every other name has been consumed into it.
    let mut held: std::collections::BTreeMap<String, RustExpr> = std::collections::BTreeMap::new();
    for statement in leading {
        let RustStmt::Assign {
            target: RustExpr::Path(target),
            op,
            value,
        } = statement
        else {
            return None;
        };
        // Only the names the ORIGINAL value mentions, and each exactly once. Substituting over the
        // running result instead lets a name inside something already substituted be substituted
        // AGAIN — `acc = g(acc)` after `val = f(val)` produced `f(f(val))`, applying the source's
        // call twice. A program that compiles and computes something else, in the engine that
        // exists to prevent exactly that.
        // A chain LINK must read the name it chains from. The recogniser proved it does in the
        // SOURCE; if the translated value does not, the translation is OPAQUE — a call the pack
        // answers for arrives as target TEXT rather than as a tree, and nothing can be substituted
        // into text. Overwriting the chain with it silently dropped every statement before:
        // `acc = rol31(acc)` after `acc = acc.wrapping_add(..)` emitted
        // `acc.rotate_left(31).wrapping_mul(PRIME1)` and lost the addition. A program that compiles
        // and computes something else, which is the one failure this engine exists to prevent.
        //
        // A read-modify-write is exempt: its read is implicit and the value need not mention the
        // name at all.
        if op.is_none() && held.contains_key(target) && mentions_expr(value, target) == 0 {
            return None;
        }
        let mut resolved = value.clone();
        // A substitution that FAILS aborts the fold. Skipping it silently drops the statement whose
        // value could not be placed — `acc = rol31(acc)` after `acc = acc.wrapping_add(..)` emitted
        // `acc.rotate_left(31).wrapping_mul(PRIME1)`, losing the addition entirely. A program that
        // compiles and computes something else, which is the one failure this engine exists to
        // prevent, and the second time this fold produced one.
        //
        // It fails whenever a value is OPAQUE: a call the pack answers for arrives as target text
        // rather than as a tree, and nothing can be substituted into text. Those bodies keep their
        // statements, which is correct and is what the abort delivers.
        for bound in held.keys().filter(|bound| mentions_expr(value, bound) > 0) {
            resolved = substituted(&resolved, bound, &held[bound])?;
        }
        // A READ-MODIFY-WRITE is the same link spelled shorter: `acc ^= v` means `acc = acc ^ v`,
        // and the implicit read is what the chain hands forward. Rebuilding it explicitly is how the
        // fold sees that link at all — without this the body kept its statements while the signature
        // had already dropped the `mut`, which is the disagreement one fact is supposed to prevent.
        if let Some(operator) = op {
            resolved = RustExpr::Binary {
                op: *operator,
                // The parameter's OWN value where nothing has been assigned to it yet: the first
                // link of a chain reads the argument the caller passed, not something the body
                // computed.
                lhs: Box::new(
                    held.get(target)
                        .cloned()
                        .unwrap_or_else(|| RustExpr::Path(target.clone())),
                ),
                rhs: Box::new(resolved),
            };
        }
        held.insert(target.clone(), resolved);
    }
    let accumulated = held.get(name).cloned();
    let returns_name = matches!(last, RustStmt::Return(Some(RustExpr::Path(returned))) if returned == name)
        || matches!(last, RustStmt::Tail(RustExpr::Path(returned)) if returned == name);
    if !returns_name {
        return None;
    }
    Some(vec![RustStmt::Tail(accumulated?)])
}

/// One expression with the accumulated name replaced by what it holds.
///
/// The name appears exactly once — the recogniser proved it on the source — so this replaces the
/// first occurrence it finds and reports failure if there was none, rather than replacing blindly.
fn substituted(within: &RustExpr, name: &str, value: &RustExpr) -> Option<RustExpr> {
    if let RustExpr::Path(path) = within
        && path == name
    {
        return Some(value.clone());
    }
    let mut replaced = false;
    let mut once = |inner: &RustExpr| -> RustExpr {
        if replaced {
            return inner.clone();
        }
        match substituted(inner, name, value) {
            Some(new) => {
                replaced = true;
                new
            }
            None => inner.clone(),
        }
    };
    let rebuilt = match within {
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        } => RustExpr::MethodCall {
            receiver: Box::new(once(receiver)),
            method: method.clone(),
            args: args.iter().map(&mut once).collect(),
        },
        RustExpr::Call { callee, args } => RustExpr::Call {
            callee: callee.clone(),
            args: args.iter().map(&mut once).collect(),
        },
        RustExpr::Binary { op, lhs, rhs } => RustExpr::Binary {
            op: *op,
            lhs: Box::new(once(lhs)),
            rhs: Box::new(once(rhs)),
        },
        RustExpr::Cast { expr, ty } => RustExpr::Cast {
            expr: Box::new(once(expr)),
            ty: ty.clone(),
        },
        RustExpr::Unary { op, operand } => RustExpr::Unary {
            op: *op,
            operand: Box::new(once(operand)),
        },
        // Any other shape is one the recogniser did not promise, and guessing at it would move an
        // expression somewhere its evaluation order is not the source's.
        _ => return None,
    };
    replaced.then_some(rebuilt)
}

/// How many times this target expression names the binding.
///
/// Asked of the value BEFORE any substitution, so a name that only appears because something was
/// substituted in is not treated as one the statement mentioned.
fn mentions_expr(expr: &RustExpr, name: &str) -> usize {
    match expr {
        RustExpr::Path(path) => usize::from(path == name),
        RustExpr::MethodCall { receiver, args, .. } => {
            mentions_expr(receiver, name)
                + args.iter().map(|arg| mentions_expr(arg, name)).sum::<usize>()
        }
        RustExpr::Call { callee, args } => {
            mentions_expr(callee, name)
                + args.iter().map(|arg| mentions_expr(arg, name)).sum::<usize>()
        }
        RustExpr::Binary { lhs, rhs, .. } => mentions_expr(lhs, name) + mentions_expr(rhs, name),
        RustExpr::Unary { operand, .. } => mentions_expr(operand, name),
        RustExpr::Cast { expr, .. } => mentions_expr(expr, name),
        _ => 0,
    }
}

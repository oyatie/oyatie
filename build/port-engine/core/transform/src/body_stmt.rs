//! One source statement into one target statement.
//!
//! Split from `body.rs` because the two answer different questions: that file says what a body
//! translation NEEDS in order to answer anything — the pack's tables, what the signature decided,
//! what the enclosing loop proved — and this one is the dispatch that spends it.
//!
//! Every arm here is one construct, and a construct with no arm refuses by name rather than
//! reaching a default. A statement translator whose fall-through emits something is a translator
//! that turns an untranslatable body into a green one.

use std::collections::BTreeSet;

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt, TupleBind};

use crate::body::{Body, TailPosition, translate};
use crate::body_cond::conditional;
use crate::body_expr::{Position, expression, in_position};
use crate::body_failure::translated_return;
use crate::body_loops::{counted_loop, range_loop, switch};
use crate::body_ops::binary_operator;
use crate::body_parts::unsupported_source;
use crate::body_parts::{branch, named_child, one_child, two_children};
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::vocabulary::{
    ATTR_OP, CHILD_BIND, CHILD_PLACE, CHILD_VALUE, FLAG_INFERRED, FLAG_MUTATED, IDIOM_SWAP,
};

pub(crate) fn statement(
    node: &Declaration,
    cx: &Body<'_>,
    is_last: bool,
) -> Result<RustStmt, TransformError> {
    match node.kind.as_str() {
        "return" => translated_return(node, cx, is_last),
        "block" => Ok(RustStmt::Semi(RustExpr::Block(translate(
            &node.children,
            cx,
            TailPosition::No,
        )?))),
        "if" => Ok(RustStmt::Semi(conditional(node, cx)?)),
        // A `cond` node reaching statement position is an init clause's own statement, already
        // handled by `conditional`. Reaching here would mean the tree is shaped differently than
        // the front end claims, which is a defect rather than a construct.
        "let" => Ok(RustStmt::Let {
            name: to_snake_case(&node.name),
            // MUTABLE only when the body writes it again, which the front end observed. The source
            // makes every binding mutable and the target makes none of them, so a default in
            // either direction is wrong for half the bindings in any real body.
            mutable: node.has_flag(FLAG_MUTATED),
            // A DECLARED type is carried through. The source often declares one where the target
            // could infer it, and dropping it would change what the binding IS wherever the two
            // languages default differently — an untyped integer literal being the common case.
            //
            // An INFERRED one is not. The type is recorded on every binding because the engine
            // needs it, and annotating one the source never wrote puts a type on every short
            // declaration in every body — noise the author did not write and the target does not
            // need, since it infers exactly what the source inferred.
            ty: match node.type_ref.is_empty() || node.has_flag(FLAG_INFERRED) {
                true => None,
                false => Some(cx.resolver.resolve_in(
                    &node.type_ref,
                    cx.owner,
                    crate::vocabulary::POSITION_PARAM,
                )?),
            },
            value: match node.children.first() {
                Some(child) => Some(expression(child, cx)?),
                None => None,
            },
        }),
        "expr_stmt" => Ok(RustStmt::Semi(expression(
            one_child(node, cx, "expr_stmt")?,
            cx,
        )?)),
        "assign" => {
            let (target, value) = two_children(node, cx, "assign")?;
            // A read-modify-write carries the operator it applies; a plain assignment carries
            // none. The operator is refused by name when the target has no form for it, which is
            // the same answer the binary expression gives for the same spelling.
            let place = in_position(target, cx, Position::Place)?;
            let Some(spelling) = node.attr(ATTR_OP) else {
                return Ok(RustStmt::Assign {
                    target: place,
                    op: None,
                    value: expression(value, cx)?,
                });
            };

            // A read-modify-write on integers carries the same overflow question a binary
            // operation does, and the target has no `wrapping_mul_assign` to answer it with. So
            // the compound form EXPANDS to `place = place.wrapping_mul(value)` — which reads the
            // place twice, and is only sound where reading it twice is the same as reading it
            // once. Where it is not, this refuses rather than calling something twice that the
            // source called once.
            if let Some(method) = cx.resolver.wrapping_method(node, spelling) {
                if !reads_once(&place) {
                    return Err(TransformError::Unsupported {
                        name: cx.owner.to_owned(),
                        detail: format!(
                            "`{spelling}=` on an integer needs the wrapping form, which reads the \
                             assigned place twice, and this place is not one that can be read \
                             twice safely"
                        ),
                    });
                }
                return Ok(RustStmt::Assign {
                    target: place.clone(),
                    op: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(place),
                        method: method.to_owned(),
                        args: vec![expression(value, cx)?],
                    },
                });
            }

            let op = binary_operator(spelling).ok_or_else(|| TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: format!("assignment operator `{spelling}=` has no target form"),
            })?;
            Ok(RustStmt::Assign {
                target: place,
                op: Some(op),
                value: expression(value, cx)?,
            })
        }
        "let_tuple" => {
            // A destructuring bind that is NOT the failure check — the propagation matcher runs
            // first and consumes those, so anything reaching here binds several values from one
            // expression and means exactly what the target's tuple binding means.
            let binds = node.children_of_kind(CHILD_BIND);
            let value = named_child(node, CHILD_VALUE, cx, "let_tuple")?;
            Ok(RustStmt::LetTuple {
                names: binds
                    .iter()
                    .map(|bound| TupleBind {
                        name: to_snake_case(&bound.name),
                        // Observed by the front end, exactly as a single binding's is. Assuming
                        // either way is wrong for half the bindings in any real body.
                        mutable: bound.has_flag(FLAG_MUTATED),
                    })
                    .collect(),
                value: expression(one_child(value, cx, "let_tuple")?, cx)?,
            })
        }
        "assign_tuple" => {
            // A PARALLEL assignment. The source evaluates every operand on both sides before
            // assigning any of them, which is what makes `a[i], a[j] = a[j], a[i]` a swap rather
            // than two writes — and the target's destructuring assignment has the same rule, so
            // the construct carries across whole. Two separate assignments would not: the first
            // would be written and then read back by the second.
            let mut places = Vec::new();
            for place in node.children_of_kind(CHILD_PLACE) {
                places.push(expression(one_child(place, cx, "assign_tuple")?, cx)?);
            }
            let mut values = Vec::new();
            for value in node.children_of_kind(CHILD_VALUE) {
                values.push(expression(one_child(value, cx, "assign_tuple")?, cx)?);
            }
            // An EXCHANGE is what the target's sequence has a method for. The parallel form is
            // already faithful; this is the spelling, and it is recognised from the source nodes
            // rather than from the rendered places so the match cannot depend on how they printed.
            if cx.resolver.idiom_method(IDIOM_SWAP).is_some()
                && let Some(swapped) = crate::body_swap::exchange(node, cx)?
            {
                return Ok(RustStmt::Semi(swapped));
            }
            match values.len() == 1 || values.len() == places.len() {
                true => Ok(RustStmt::AssignTuple { places, values }),
                // Neither shape the target has: one expression yielding the whole tuple, or one
                // per place. Anything else would need a rule for how the values pair up, and
                // pairing them by position when the counts disagree is a guess.
                false => Err(TransformError::Unsupported {
                    name: cx.owner.to_owned(),
                    detail: format!(
                        "a parallel assignment writes {} places from {} values, and the target \
                         spells only one value per place or one expression yielding them all",
                        places.len(),
                        values.len()
                    ),
                }),
            }
        }
        "break" => Ok(RustStmt::Break),
        "for" => counted_loop(node, cx),
        "range" => range_loop(node, cx),
        // A switch in TAIL position is the body's value, and its arms are too — so an arm yields
        // rather than returning, and the match itself is the tail rather than a statement.
        "switch" => {
            let tail = match is_last {
                true => TailPosition::Yes,
                false => TailPosition::No,
            };
            let matched = switch(node, cx, tail)?;
            Ok(match is_last {
                true => RustStmt::Tail(matched),
                false => RustStmt::Semi(matched),
            })
        }
        "unsupported" => Err(unsupported_source(node, cx)),
        other => Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("statement kind `{other}` has no translation"),
        }),
    }
}

/// Whether reading this place twice is the same as reading it once.
///
/// The expanded compound assignment reads the assigned place on both sides, and the source read it
/// once. A path, a field of one, and an index by one are all pure reads; anything reached through
/// a CALL is not, and doubling it would run the caller's code twice.
fn reads_once(place: &RustExpr) -> bool {
    match place {
        RustExpr::Path(_) | RustExpr::SelfValue | RustExpr::Literal(_) => true,
        RustExpr::Field { base, .. } => reads_once(base),
        RustExpr::Index { base, index } => reads_once(base) && reads_once(index),
        _ => false,
    }
}

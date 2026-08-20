//! Loop and switch statements.
//!
//! Every shape here is matched STRUCTURALLY and refused otherwise. A loop that merely resembles a
//! translated one is a different loop, and getting it wrong changes how many times a body runs —
//! which is the class of defect no golden and no parse check would ever surface.

use port_engine_api::Declaration;
use port_engine_rust_ir::{MatchArm, RustExpr, RustStmt};

use crate::body::{Body, TailPosition, translate};
use crate::body_parts::{branch, named_child, one_child, two_children};
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::vocabulary::{KIND_IDENT, 
    ATTR_OP, IDIOM_INDEX_COUNTER, IDIOM_INDEX_LOOP, IDIOM_MATCHES,
};

/// A three-clause or condition-only `for`.
///
/// The CANONICAL ascending-integer form becomes a range loop, because that is what it means and
/// what a reader expects. Everything else with a post-statement REFUSES rather than becoming a
/// `while`: a `while` runs its post-statement only on the paths that reach the end of the body, so
/// a `continue` — or any early exit added later — silently skips it. That is a different program,
/// and it is different in a way no test of the current corpus would catch.
pub(crate) fn counted_loop(node: &Declaration, cx: &Body<'_>) -> Result<RustStmt, TransformError> {
    let body = branch(node, "then", cx)?;
    let condition = node.children_of_kind("cond").first().copied();
    let init = node.children_of_kind("init").first().copied();
    let post = node.children_of_kind("post").first().copied();

    match (init, condition, post) {
        (None, None, None) => Ok(RustStmt::Loop(translate(
            &body.children,
            cx,
            TailPosition::No,
        )?)),
        (None, Some(cond), None) => Ok(RustStmt::While {
            cond: expression(one_child(cond, cx, "cond")?, cx)?,
            body: translate(&body.children, cx, TailPosition::No)?,
        }),
        // An init clause with no post is a SCOPE and a loop. The binding belongs to the loop in the
        // source, so it is wrapped in a block rather than emitted beside the loop, where it would
        // stay visible to the rest of the enclosing body and could shadow a name the source left
        // readable there.
        (Some(init), condition, None) => {
            let inner = match condition {
                None => RustStmt::Loop(translate(&body.children, cx, TailPosition::No)?),
                Some(cond) => RustStmt::While {
                    cond: expression(one_child(cond, cx, "cond")?, cx)?,
                    body: translate(&body.children, cx, TailPosition::No)?,
                },
            };
            Ok(RustStmt::Block(vec![
                crate::body_stmt::statement(one_child(init, cx, "init")?, cx, false)?,
                inner,
            ]))
        }
        // A post-statement. The CANONICAL ascending-integer form spends it building a range, and
        // everything else has to spell it as the last statement of the body — which is correct only
        // when no path can jump over it.
        (init, Some(cond), Some(post)) => {
            if let Some(init) = init
                && let Ok(ranged) = counted_range(init, cond, post, body, cx)
            {
                return Ok(ranged);
            }
            while_with_post(init, cond, post, body, cx)
        }
        // A post-statement and no condition: `for ; ; i++`. The target's `loop` has nowhere to put
        // the post-statement that a `continue` could not skip, and unlike the condition form there
        // is no test to hang it after.
        _ => Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "a `for` with a post-statement and no condition has no direct target form"
                .to_owned(),
        }),
    }
}

/// `for [init]; cond; post { .. }` as a `while` whose body ends with the post-statement.
///
/// REFUSED when any path inside the body can reach the next iteration without running the post
/// statement. The target's `continue` jumps to the loop's test; the source's jumps to its POST
/// clause, and the two differ by exactly one statement — so a body containing one would run a
/// counter's increment on some iterations and not others. That compiles, and it loops a different
/// number of times, which is the class of defect this engine exists to prevent.
fn while_with_post(
    init: Option<&Declaration>,
    cond: &Declaration,
    post: &Declaration,
    body: &Declaration,
    cx: &Body<'_>,
) -> Result<RustStmt, TransformError> {
    if continues_this_loop(body) {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "a `for` with a post-statement contains a `continue`, and the target's \
                     `continue` jumps to the test rather than to the post-statement — spelling the \
                     post-statement at the end of the body would skip it on exactly those paths"
                .to_owned(),
        });
    }

    let mut translated = translate(&body.children, cx, TailPosition::No)?;
    translated.push(crate::body_stmt::statement(
        one_child(post, cx, "post")?,
        cx,
        false,
    )?);
    let looped = RustStmt::While {
        cond: expression(one_child(cond, cx, "cond")?, cx)?,
        body: translated,
    };

    match init {
        None => Ok(looped),
        Some(init) => Ok(RustStmt::Block(vec![
            crate::body_stmt::statement(one_child(init, cx, "init")?, cx, false)?,
            looped,
        ])),
    }
}

/// Whether a `continue` inside this body targets THIS loop.
///
/// The walk stops at every nested loop, because a `continue` written inside one targets that loop
/// and not this one. It does NOT stop at a `switch`: the source's switch is not a loop, so a
/// `continue` written inside a case continues the loop that encloses the switch.
fn continues_this_loop(node: &Declaration) -> bool {
    node.children.iter().any(|child| match child.kind.as_str() {
        "continue" => true,
        "for" | "range" => false,
        _ => continues_this_loop(child),
    })
}

/// Recognise `for i := A; i < B; i++` and emit `for i in A..B`.
///
/// Matched STRUCTURALLY and refused otherwise. A loop that merely resembles this one — a different
/// variable in the condition, a decrement, a bound that changes inside the body — is a different
/// loop, and emitting a range for it would silently change how many times the body runs.
fn counted_range(
    init: &Declaration,
    cond: &Declaration,
    post: &Declaration,
    body: &Declaration,
    cx: &Body<'_>,
) -> Result<RustStmt, TransformError> {
    let init = one_child(init, cx, "init")?;
    let cond = one_child(cond, cx, "cond")?;
    let post = one_child(post, cx, "post")?;

    let refuse = |why: &str| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "`for` loop is not the canonical ascending-integer form ({why}), and a `while` would \
             run its post-statement only on the paths that reach the end of the body"
        ),
    };

    if init.kind != "let" {
        return Err(refuse("the init clause does not bind a new name"));
    }
    let counter = &init.name;
    if cond.kind != "binary" || cond.attr(ATTR_OP) != Some("<") {
        return Err(refuse("the condition is not `<`"));
    }
    let (lhs, rhs) = two_children(cond, cx, "cond")?;
    if lhs.kind != "ident" || &lhs.name != counter {
        return Err(refuse("the condition does not test the counter"));
    }
    // The post clause has to be `i++`. A DECREMENT with an ascending `<` test is a loop that
    // either runs forever or not at all, and a range would run it a sensible number of times —
    // which is the kind of "fix" that makes emitted code mean something the source does not.
    if post.kind != "incdec" || post.attr(ATTR_OP) != Some("++") {
        return Err(refuse("the post clause is not an increment"));
    }

    // The counter is a `usize` when the loop uses it for NOTHING BUT indexing, and then the range
    // needs no conversion and neither does the index. See the pack's `index_counter_is_usize`.
    let indexes_only = cx.resolver.idiom_method(IDIOM_INDEX_COUNTER).is_some()
        && crate::counters::indexes_only(body, counter);
    // The counter exists only to reach each element, and the target reaches them directly. Same
    // elements, same order, same number of times — what goes is the index. See the pack's
    // `index_loop_is_an_iterator` for what it costs, which is one invented loop-local name.
    if cx.resolver.idiom_method(IDIOM_INDEX_LOOP).is_some()
        && let Some(sequence) =
            crate::counters::walked_sequence(rhs, body, counter, cx.resolver.length_functions)
        && let Some(element) = crate::counters::element_name(&sequence, body)
        && crate::counters::elements_copy(body, counter, cx)
    {
        return Ok(RustStmt::ForIn {
            binding: to_snake_case(&element),
            // Borrow the sequence and COPY each element, which is exactly what the source's index
            // read does: it takes a copy and leaves the sequence usable. Consuming the sequence
            // would end its life at the loop, and handing out references would give the body a
            // reference where the source gave it a value.
            iter: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Path(to_snake_case(&sequence))),
                    method: "iter".to_owned(),
                    args: Vec::new(),
                }),
                method: "copied".to_owned(),
                args: Vec::new(),
            },
            body: translate(
                &body.children,
                &cx.with_element(counter, &sequence, &to_snake_case(&element)),
                TailPosition::No,
            )?,
        });
    }

    let widened;
    let inner: &Body<'_> = match indexes_only {
        true => {
            widened = cx.with_usize_counter(counter);
            &widened
        }
        false => cx,
    };
    let end = match indexes_only {
        true => crate::counters::unsigned_bound(rhs, inner)?,
        false => expression(rhs, inner)?,
    };

    Ok(RustStmt::ForIn {
        // THE BLANK where the body never mentions it. See `counters::reads_name`: the source has no
        // way to repeat a fixed number of times except by counting, and a name nobody reads is a
        // denied warning here.
        binding: match crate::counters::reads_name(body, counter) {
            true => to_snake_case(counter),
            false => "_".to_owned(),
        },
        iter: RustExpr::Range {
            start: Box::new(expression(one_child(init, cx, "let")?, inner)?),
            end: Box::new(end),
            // EXCLUSIVE, because the loop this came from tests `<`.
            inclusive: false,
        },
        body: translate(&body.children, inner, TailPosition::No)?,
    })
}

/// A `range` loop over a sequence.
///
/// Only the value-only form translates. `for i, v := range xs` binds two names, and the target's
/// equivalent — `.iter().enumerate()` — changes what `v` IS from a copy to a reference; that is a
/// rule about element ownership rather than a loop shape, so it refuses here.
pub(crate) fn range_loop(node: &Declaration, cx: &Body<'_>) -> Result<RustStmt, TransformError> {
    let over = named_child(node, "over", cx, "range")?;
    let body = branch(node, "then", cx)?;

    let key = node.attr("key").unwrap_or_default();
    let value = node.attr("value").unwrap_or_default();

    let binding = match (key.is_empty() || key == "_", value.is_empty()) {
        (true, false) => value,
        _ => {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: "only `for _, v := range xs` translates: binding the index too needs a \
                         rule for whether the element is a copy or a reference"
                    .to_owned(),
            });
        }
    };

    // Iterate by REFERENCE. The source's range copies the element and leaves the sequence usable;
    // consuming it here would end the sequence's life at the loop.
    //
    // Unless the sequence expression ALREADY denotes one. A slice parameter arrives borrowed —
    // that is what the pack's slice idiom decided it is — and borrowing it again yields `&&[T]`,
    // which is not an iterator. The reference is what the loop NEEDS, not a token to add
    // unconditionally, so it is added only where the expression does not already carry it.
    let sequence = one_child(over, cx, "over")?;
    let rendered = expression(sequence, cx)?;
    // Already borrowed when the sequence IS a parameter the signature borrows — the same answer the
    // signature gave, read rather than derived a second time.
    let already_borrowed =
        sequence.kind == KIND_IDENT && cx.borrowed.contains(&to_snake_case(&sequence.name));
    let iter = match already_borrowed {
        true => rendered,
        false => RustExpr::Reference {
            mutable: false,
            inner: Box::new(rendered),
        },
    };

    Ok(RustStmt::ForIn {
        binding: to_snake_case(binding),
        iter,
        body: translate(&body.children, cx, TailPosition::No)?,
    })
}

/// An expression switch becomes a `match`.
///
/// The target's `match` does not fall through and neither does the source's switch, so the two
/// agree on the one semantic that usually differs between languages here.
pub(crate) fn switch(
    node: &Declaration,
    cx: &Body<'_>,
    tail: TailPosition,
) -> Result<RustExpr, TransformError> {
    let cases = node.children_of_kind("case");
    let Some(tag) = node.children_of_kind("tag").first().copied() else {
        return condition_chain(&cases, cx, tail);
    };

    let mut arms = Vec::with_capacity(cases.len());
    // The WILDCARD's position, so it can be moved. The source lets `default:` be written anywhere
    // among the cases and still be the fallback; the target takes the first arm that matches, so a
    // wildcard left in place makes every arm after it unreachable. gjson writes it first, and the
    // arms it shadowed were the ones that accept a valid escape — so the emitted function rejected
    // every string the source accepts, and did it while compiling.
    let mut wildcard: Option<usize> = None;
    for case in cases {
        let patterns_node = named_child(case, "patterns", cx, "switch")?;
        let body = branch(case, "then", cx)?;
        let patterns = patterns_node
            .children
            .iter()
            .map(|pattern| expression(pattern, cx))
            .collect::<Result<Vec<_>, _>>()?;
        if patterns.is_empty() {
            if wildcard.is_some() {
                return Err(TransformError::Unsupported {
                    name: cx.owner.to_owned(),
                    detail: "a switch has two `default` cases, and which one is the fallback is \
                             not a question the target can be asked"
                        .to_owned(),
                });
            }
            wildcard = Some(arms.len());
        }
        arms.push(MatchArm {
            patterns,
            // AN ARM OF A TAIL MATCH IS ITSELF IN TAIL POSITION. The source's switch is a statement
            // and every arm has to `return` out of the function; the target's is an expression, and
            // an arm that returns where it could simply yield is the shape clippy's
            // `needless_return` names — which is what it named, on the first real package this was
            // pointed at. The top level of a body already got this right; the arms did not.
            body: translate(&body.children, cx, tail)?,
        });
    }

    // LAST, wherever the source wrote it. Moving it is safe precisely because it matches
    // everything: no arm it passes over can have been reachable through it, and every arm it passes
    // over was unreachable while it sat in front of them.
    if let Some(index) = wildcard
        && index + 1 != arms.len()
    {
        let fallback = arms.remove(index);
        arms.push(fallback);
    }

    let tag_node = one_child(tag, cx, "tag")?;
    let tag_expr = match_scrutinee(tag_node, &arms, cx)?;

    // A MEMBERSHIP TEST, which the target has a macro for. Recognised after the arms are built,
    // from what they yield rather than from the source shape, so it cannot fire on a match that
    // merely looks like one.
    if cx.resolver.idiom_method(IDIOM_MATCHES).is_some()
        && let Some(test) = crate::body_swap::membership(&arms, &tag_expr)
    {
        return Ok(test);
    }

    // A `match` must be exhaustive and a Go switch need not be. Adding the arm silently would
    // invent a branch the source does not have, so the absence is a refusal — with the fix named.
    if wildcard.is_none() {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "switch has no `default`, and the target's match must be exhaustive — adding \
                     the missing arm here would invent a branch the source does not have"
                .to_owned(),
        });
    }

    Ok(RustExpr::Match {
        scrutinee: Box::new(tag_expr),
        arms,
    })
}

/// A switch with NO TAG, which is a chain of conditions rather than a match.
///
/// The source compares each case expression against `true` and takes the FIRST that holds, which is
/// exactly what a chain of `else if` does. It is not a `match`: there is no scrutinee, and the
/// target's match arms are patterns rather than tests.
///
/// Order is the whole content of the construct, so it is preserved literally — with one exception
/// the source permits and the target does not. The source allows `default` to be written ANYWHERE
/// among the cases and still be the fallback; the target's `else` can only be last. So the default
/// is lifted out and emitted last, and the remaining cases keep their relative order. Leaving it in
/// place would make every case after it unreachable.
fn condition_chain(
    cases: &[&Declaration],
    cx: &Body<'_>,
    tail: TailPosition,
) -> Result<RustExpr, TransformError> {
    let mut tested = Vec::new();
    let mut fallback = None;
    for case in cases {
        let patterns = named_child(case, "patterns", cx, "switch")?;
        let body = branch(case, "then", cx)?;
        // No patterns is the source's `default`. A switch may have only one, so a second is a shape
        // the front end did not record the way this expects rather than a choice to make.
        if patterns.children.is_empty() {
            if fallback.is_some() {
                return Err(TransformError::Unsupported {
                    name: cx.owner.to_owned(),
                    detail: "a switch with no tag has two `default` cases, and which one is the \
                             fallback is not a question the target can be asked"
                        .to_owned(),
                });
            }
            fallback = Some(translate(&body.children, cx, tail)?);
            continue;
        }
        // SEVERAL expressions in one case hold when ANY of them does — the source compares each
        // against `true` in turn. Joined here rather than expanded into separate branches so the
        // body is emitted once, which is what the source does.
        let mut condition: Option<RustExpr> = None;
        for pattern in &patterns.children {
            let next = expression(pattern, cx)?;
            condition = Some(match condition {
                None => next,
                Some(previous) => RustExpr::Binary {
                    op: port_engine_rust_ir::BinaryOp::Or,
                    lhs: Box::new(previous),
                    rhs: Box::new(next),
                },
            });
        }
        let Some(condition) = condition else {
            unreachable!("the pattern list was checked non-empty above");
        };
        tested.push((condition, translate(&body.children, cx, tail)?));
    }

    // Built from the BACK, because each `else` holds the chain that follows it.
    let mut chain = fallback.map(RustExpr::Block);
    for (condition, body) in tested.into_iter().rev() {
        chain = Some(RustExpr::If {
            cond: Box::new(condition),
            then: body,
            // The target's `else` takes a block or another `if`, never a bare expression, and both
            // of those are exactly what this carries.
            otherwise: chain.map(Box::new),
        });
    }

    chain.ok_or_else(|| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: "a switch with no tag has no cases at all, and the target has no form for a \
                 conditional with nothing to test"
            .to_owned(),
    })
}

/// The scrutinee of a `match`, at the type its PATTERNS have.
///
/// The source guarantees these agree — a switch compares its tag against each case with `==`, and
/// the source would not compile if they were different types. The target breaks that agreement in
/// two ways the source has no counterpart for, and each needs the opposite correction.
///
/// A NEWTYPE scrutinee whose cases are literals. `switch vsn { case 0: }` on a `type
/// encryptionVersion uint8` compares numbers, because there the defined type and its underlying are
/// one thing. The target's newtype is a struct and `0` is not one, so the scrutinee reaches through
/// the wrapper — the patterns cannot, since a pattern has no field access.
///
/// A BORROWED RECEIVER whose cases are constants. A method on a value receiver still takes `&self`
/// here, so `match self` has type `&T` while every constant the source names has type `T`. The
/// scrutinee is dereferenced rather than the patterns being wrapped, because `&CONST` is not a
/// pattern the target accepts either. Nothing moves: a constant pattern binds nothing, so the place
/// is only read.
fn match_scrutinee(
    tag: &Declaration,
    arms: &[MatchArm],
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    let patterns = || arms.iter().flat_map(|arm| arm.patterns.iter());
    let mut any = false;
    let all_literals = patterns().all(|pattern| {
        any = true;
        matches!(pattern, RustExpr::Literal(_))
    }) && any;
    if all_literals && crate::body_index::unwraps_newtype(tag, cx) {
        return crate::body_index::unwrapped_base(tag, cx);
    }

    // A PLACE. A match reads its scrutinee and constant patterns bind nothing, so nothing is
    // consumed — and a value position would clone a field read of a non-copying type for a copy no
    // arm keeps. `match self.r#type.clone()` allocated on every call to ask which variant it was.
    let translated = crate::body_expr::in_position(tag, cx, crate::body_expr::Position::Place)?;
    let all_paths = patterns().all(|pattern| matches!(pattern, RustExpr::Path(_))) && any;
    if all_paths
        && crate::body_ops::is_receiver(tag)
        && cx
            .receiver_type
            .is_some_and(|owner| cx.resolver.scope.newtypes.contains_key(owner))
    {
        return Ok(RustExpr::Deref(Box::new(translated)));
    }
    Ok(translated)
}

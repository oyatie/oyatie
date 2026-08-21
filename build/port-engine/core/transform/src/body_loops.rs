//! Loop and switch statements.
//!
//! Every shape here is matched STRUCTURALLY and refused otherwise. A loop that merely resembles a
//! translated one is a different loop, and getting it wrong changes how many times a body runs —
//! which is the class of defect no golden and no parse check would ever surface.

use port_engine_api::Declaration;
use port_engine_rust_ir::{ForBinding, MatchArm, RustExpr, RustStmt};

use crate::body::{Body, TailPosition, translate};
use crate::body_expr::expression;
use crate::body_parts::{branch, named_child, one_child, two_children};
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::vocabulary::{
    ATTR_OP, IDIOM_INDEX_COUNTER, IDIOM_INDEX_LOOP, IDIOM_MATCHES, KIND_IDENT,
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
            label: None,
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
                    label: None,
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
    // A `continue` here does not mean what the target's `continue` means. The source's jumps to the
    // POST clause; the target's jumps to the test, skipping the post-statement this form has to
    // spell at the end of the body — so the loop would count differently, which compiles.
    //
    // The target has the construct that says it exactly. A LABELLED BLOCK around the body turns
    // `continue` into `break 'step`, which leaves the block and lands on the post-statement: the
    // same place the source goes. A `break` written in the same body has to be relabelled too,
    // because inside the block a bare one would leave the BLOCK — one iteration — rather than the
    // loop.
    let stepped = continues_this_loop(body);
    // Only where a `break` would end up INSIDE the step block, because there a bare one leaves the
    // block rather than the loop. An unused label is a warning, and the emitted crate is held to
    // deny-warnings.
    let labelled = stepped && breaks_this_loop(body);
    let mut translated = translate(&body.children, cx, TailPosition::No)?;
    if stepped {
        relabel(&mut translated, labelled);
        // THE BLOCK IS ONLY WORTH ITS NOISE IF IT SKIPS SOMETHING. `break 'step` leaves the block
        // and lands on the post-statement; falling off the end of the block lands there too. So
        // where every step-break is already the last thing on its path, the block and the breaks
        // say exactly what plain fall-through says, and a reviewer reading `'step: { .. }` around a
        // match whose arms all diverge is reading scaffolding rather than logic.
        //
        // A reviewer named this on the first package it appeared in, and was right.
        match skips_nothing(&translated) {
            true => drop_step_breaks(&mut translated),
            false => {
                translated = vec![RustStmt::Labelled {
                    label: STEP.to_owned(),
                    body: translated,
                }];
            }
        }
    }
    translated.push(crate::body_stmt::statement(
        one_child(post, cx, "post")?,
        cx,
        false,
    )?);
    let looped = RustStmt::While {
        cond: expression(one_child(cond, cx, "cond")?, cx)?,
        body: translated,
        label: labelled.then(|| LOOP.to_owned()),
    };

    match init {
        None => Ok(looped),
        Some(init) => Ok(RustStmt::Block(vec![
            crate::body_stmt::statement(one_child(init, cx, "init")?, cx, false)?,
            looped,
        ])),
    }
}

/// Whether every `break 'step` in these statements is already the last thing on its path.
///
/// The block exists to skip the rest of an iteration. Where there is no rest to skip -- the break
/// is the final statement of the block, or the final statement of a branch that is itself final --
/// leaving the block and falling out of it are the same, and the label is scaffolding.
fn skips_nothing(statements: &[RustStmt]) -> bool {
    let Some((last, earlier)) = statements.split_last() else {
        return true;
    };
    // A break anywhere but the end HAS something after it, which is the whole reason for the block.
    if earlier.iter().any(mentions_step) {
        return false;
    }
    tail_skips_nothing(last)
}

/// The same question of one statement standing in tail position.
fn tail_skips_nothing(statement: &RustStmt) -> bool {
    match statement {
        RustStmt::Break(Some(label)) => label == STEP,
        RustStmt::Block(body) | RustStmt::Labelled { body, .. } => skips_nothing(body),
        RustStmt::Semi(expr) | RustStmt::Tail(expr) => match expr {
            RustExpr::Match { arms, .. } => arms.iter().all(|arm| skips_nothing(&arm.body)),
            RustExpr::If {
                then, otherwise, ..
            } => {
                skips_nothing(then)
                    && otherwise
                        .as_deref()
                        .is_none_or(|other| tail_skips_nothing(&RustStmt::Semi(other.clone())))
            }
            RustExpr::Block(body) => skips_nothing(body),
            _ => true,
        },
        // A nested loop owns its own jumps, so nothing inside it is this block's business.
        RustStmt::While { .. } | RustStmt::Loop(_) | RustStmt::ForIn { .. } => true,
        _ => true,
    }
}

/// Whether a `break 'step` appears anywhere in this statement, including inside a nested loop --
/// where it would still be this block's, because a label crosses a loop boundary.
fn mentions_step(statement: &RustStmt) -> bool {
    match statement {
        RustStmt::Break(Some(label)) => label == STEP,
        RustStmt::Block(body)
        | RustStmt::Labelled { body, .. }
        | RustStmt::While { body, .. }
        | RustStmt::Loop(body)
        | RustStmt::ForIn { body, .. } => body.iter().any(mentions_step),
        RustStmt::Semi(expr) | RustStmt::Tail(expr) => mentions_step_in(expr),
        _ => false,
    }
}

/// The same, through the expressions that carry statements.
fn mentions_step_in(expr: &RustExpr) -> bool {
    match expr {
        RustExpr::Block(body) => body.iter().any(mentions_step),
        RustExpr::If {
            then, otherwise, ..
        } => then.iter().any(mentions_step) || otherwise.as_deref().is_some_and(mentions_step_in),
        RustExpr::Match { arms, .. } => arms.iter().any(|arm| arm.body.iter().any(mentions_step)),
        _ => false,
    }
}

/// Remove every `break 'step`, which falling out of the block now does instead.
fn drop_step_breaks(statements: &mut Vec<RustStmt>) {
    statements
        .retain(|statement| !matches!(statement, RustStmt::Break(Some(label)) if label == STEP));
    for statement in statements {
        match statement {
            RustStmt::Block(body) | RustStmt::Labelled { body, .. } => drop_step_breaks(body),
            RustStmt::Semi(expr) | RustStmt::Tail(expr) => drop_step_breaks_in(expr),
            _ => {}
        }
    }
}

/// The same, through the expressions that carry statements.
fn drop_step_breaks_in(expr: &mut RustExpr) {
    match expr {
        RustExpr::Block(body) => drop_step_breaks(body),
        RustExpr::If {
            then, otherwise, ..
        } => {
            drop_step_breaks(then);
            if let Some(other) = otherwise.as_deref_mut() {
                drop_step_breaks_in(other);
            }
        }
        RustExpr::Match { arms, .. } => {
            for arm in arms {
                drop_step_breaks(&mut arm.body);
            }
        }
        _ => {}
    }
}

/// The label on the block one iteration of a stepped loop runs in.
const STEP: &str = "step";
/// The label on a stepped loop itself, for a `break` that has to leave more than the step.
const LOOP: &str = "counted";

/// Whether a `break` inside this body targets THIS loop.
///
/// The same walk `continues_this_loop` does and for the same reason: a `break` written inside a
/// nested loop or a `switch` arm belongs to that construct. The source's `switch` DOES capture a
/// `break`, unlike its `continue`, so the walk stops at one.
fn breaks_this_loop(node: &Declaration) -> bool {
    node.children.iter().any(|child| match child.kind.as_str() {
        "break" => true,
        "for" | "range" | "switch" | "select" => false,
        _ => breaks_this_loop(child),
    })
}

/// Point this body's jumps at the labels a stepped loop introduces.
///
/// `continue` becomes `break 'step`, which leaves one iteration's block and lands on the
/// post-statement — where the source's `continue` goes. A `break` becomes `break 'counted` when the
/// loop carries that label, because inside the step block a bare one would leave the block instead.
///
/// The walk does NOT descend into a nested loop: a jump written there belongs to it, and rewriting
/// one would move it to the wrong construct.
fn relabel(statements: &mut [RustStmt], labelled: bool) {
    for statement in statements {
        match statement {
            RustStmt::Continue => *statement = RustStmt::Break(Some(STEP.to_owned())),
            RustStmt::Break(target @ None) if labelled => *target = Some(LOOP.to_owned()),
            RustStmt::Block(body) | RustStmt::Labelled { body, .. } => relabel(body, labelled),
            RustStmt::Semi(expr) | RustStmt::Tail(expr) => relabel_expression(expr, labelled),
            // A nested loop owns its own jumps.
            RustStmt::While { .. } | RustStmt::Loop(_) | RustStmt::ForIn { .. } => {}
            _ => {}
        }
    }
}

/// The same, through the expressions that carry statements.
fn relabel_expression(expr: &mut RustExpr, labelled: bool) {
    match expr {
        RustExpr::Block(body) => relabel(body, labelled),
        RustExpr::If {
            then, otherwise, ..
        } => {
            relabel(then, labelled);
            if let Some(otherwise) = otherwise.as_deref_mut() {
                relabel_expression(otherwise, labelled);
            }
        }
        RustExpr::Match { arms, .. } => {
            for arm in arms {
                relabel(&mut arm.body, labelled);
            }
        }
        _ => {}
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
            binding: ForBinding::Name(to_snake_case(&element)),
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
            true => ForBinding::Name(to_snake_case(counter)),
            false => ForBinding::Blank,
        },
        iter: RustExpr::Range {
            start: Some(Box::new(expression(one_child(init, cx, "let")?, inner)?)),
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

    let sequence_kind = one_child(over, cx, "over")?.type_ref.kind.clone();
    // WHAT THE SOURCE'S `range` MEANS depends on what is ranged, and the forms do not agree: over a
    // sequence the first name is an INDEX, over a map it is a KEY, over a string it is a BYTE
    // OFFSET with runes for values. Only the sequences are answered here, and the type is what says
    // which this is — the loop's shape alone does not.
    let indexable = matches!(sequence_kind.as_str(), "slice" | "array");
    let binding = match (key.is_empty() || key == "_", value.is_empty()) {
        (true, false) => ForBinding::Name(to_snake_case(value)),
        // `for i, v := range xs` over a sequence. The target's `enumerate` yields the index first
        // and the item second, which is the order the source binds them in, so the pattern is a
        // direct transcription rather than a reordering.
        (false, false) if indexable => ForBinding::Indexed {
            index: to_snake_case(key),
            item: to_snake_case(value),
        },
        // `for i := range xs`. The source binds the INDEX and reads elements itself; there is no
        // item to yield, so this is a range over the positions rather than over the sequence.
        (false, true) if indexable => ForBinding::Name(to_snake_case(key)),
        _ => {
            // A MAP is refused for the reason its literal already is: the source's map has no
            // order and the target's has one, so iterating it makes an order observable that the
            // source never promised. That is a decision about which map the port uses, not a loop
            // shape, and it is the same decision in both places.
            let why = match sequence_kind.as_str() {
                "map" => {
                    "ranging a map binds its KEYS in an order the source does not define, and                           the target's ordered map would make that order observable — the same                           decision its literal refuses, and it belongs with that one"
                }
                "basic" => {
                    "ranging the source's string binds BYTE OFFSETS with decoded runes for                             values, and the target's string iterators yield one or the other but                             not the pair"
                }
                _ => {
                    "binding the index needs to know what is ranged, and the type of this                       expression does not say — only a sequence has indices to bind"
                }
            };
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: why.to_owned(),
            });
        }
    };
    // The POSITIONS, not the elements: `for i := range xs` never names an item.
    let over_positions = !value.is_empty() || key.is_empty() || key == "_";

    // Iterate by REFERENCE. The source's range copies the element and leaves the sequence usable;
    // consuming it here would end the sequence's life at the loop.
    //
    // Unless the sequence expression ALREADY denotes one. A slice parameter arrives borrowed —
    // that is what the pack's slice idiom decided it is — and borrowing it again yields `&&[T]`,
    // which is not an iterator. The reference is what the loop NEEDS, not a token to add
    // unconditionally, so it is added only where the expression does not already carry it.
    let sequence = one_child(over, cx, "over")?;
    let rendered = expression(sequence, cx)?;
    // Kept UNBORROWED for the positional form: `0..xs.len()` asks the sequence its length and never
    // iterates it, so the reference the iterating forms need would be a borrow of nothing.
    let rendered_len = rendered.clone();
    // Already borrowed when the sequence IS a parameter the signature borrows — the same answer the
    // signature gave, read rather than derived a second time.
    let already_borrowed =
        sequence.kind == KIND_IDENT && cx.borrowed.contains(&to_snake_case(&sequence.name));
    // A COPYABLE ELEMENT IS COPIED, which is what the source's range does. Borrowing the sequence
    // and stopping there hands the body a REFERENCE where the source handed it a value, and every
    // use downstream then has one type too many -- `out.send(value)` wanted an `i64` and was given
    // an `&i64`. `.copied()` is the same read the counted loop already does for the same reason.
    //
    // Only where the element IS copyable. A sequence of owned values yields references because that
    // is all it can yield without consuming the sequence, and the body works with those.
    // COPYABLE IN THE TARGET, which the pack decides and `moves_on_read` already asks. The source's
    // `string` is a basic type there and a `String` here, and `String` is not copyable -- so a test
    // that read the source's own classification called `[]string` copyable and broke `chi` and
    // `xxhash`. One answer, asked in the one place that has it.
    let copies = sequence
        .type_ref
        .args
        .first()
        .is_some_and(|element| !crate::body_place::moves_on_read(element, cx));
    let borrowed = match already_borrowed {
        true => rendered,
        false => RustExpr::Reference {
            mutable: false,
            inner: Box::new(rendered),
        },
    };
    let iter = match copies {
        false => borrowed,
        true => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(borrowed),
                method: "iter".to_owned(),
                args: Vec::new(),
            }),
            method: "copied".to_owned(),
            args: Vec::new(),
        },
    };

    let iter = match (&binding, over_positions) {
        // `xs.iter().enumerate()`
        (ForBinding::Indexed { .. }, _) => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(iter),
                method: "iter".to_owned(),
                args: Vec::new(),
            }),
            method: "enumerate".to_owned(),
            args: Vec::new(),
        },
        // `0..xs.len()` — the positions, because nothing here names an item.
        (ForBinding::Name(_), false) => RustExpr::Range {
            start: Some(Box::new(RustExpr::Literal("0".to_owned()))),
            end: Box::new(RustExpr::MethodCall {
                receiver: Box::new(rendered_len),
                method: "len".to_owned(),
                args: Vec::new(),
            }),
            inclusive: false,
        },
        // Named or blank, this form iterates the items themselves.
        (ForBinding::Name(_) | ForBinding::Blank, true) => iter,
        (ForBinding::Blank, false) => iter,
    };

    // THE INDEX `enumerate` HANDS BACK IS ALREADY THE TARGET'S INDEX TYPE, and the body has to know
    // that or it converts it again — `points[i as usize]` where `i` came from `enumerate`, which is
    // a `usize` casting to a `usize`. The counted loop already tells its body this; the ranged one
    // did not, so the two ends of one decision disagreed the moment the ranged form started
    // producing an index at all.
    let widened;
    let inner: &Body<'_> = match &binding {
        ForBinding::Indexed { index, .. } => {
            widened = cx.with_usize_counter(index);
            &widened
        }
        _ => cx,
    };

    Ok(RustStmt::ForIn {
        body: translate(&body.children, inner, TailPosition::No)?,
        binding,
        iter,
    })
}

/// Whether a source case value is something the target's PATTERNS accept.
///
/// A literal is. A name the source declares `const` is, because the target emits one as a `const`
/// and its patterns accept those. Everything else -- a parameter, a local, a field, a call -- is a
/// value read at run time, and naming it in a pattern binds rather than compares.
fn is_pattern(node: &Declaration) -> bool {
    node.kind == crate::vocabulary::KIND_LITERAL
        || node.attr(crate::vocabulary::ATTR_REF) == Some(crate::vocabulary::REF_CONST)
}

/// The guard a non-constant case needs: the subject compared against each of its values.
///
/// The subject is named again rather than bound, which is why it must be an expression that can be
/// named twice without doing anything. A call there would run once in the source and once per
/// guarded arm here.
fn guard_for(
    comparisons: &[&Declaration],
    tag: &Declaration,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    let subject = one_child(tag, cx, "tag")?;
    if !crate::body_copy::repeatable(subject) {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "a switch compares its subject against a value that is not a constant, which \
                     the target spells as a guard — and the guard has to name the subject again, \
                     where this one does something when it is evaluated"
                .to_owned(),
        });
    }
    let mut guard: Option<RustExpr> = None;
    for value in comparisons {
        let test = RustExpr::Binary {
            op: port_engine_rust_ir::BinaryOp::Eq,
            lhs: Box::new(expression(subject, cx)?),
            rhs: Box::new(expression(value, cx)?),
        };
        guard = Some(match guard {
            None => test,
            // `case a, b:` is a match on EITHER, which is an or of the comparisons.
            Some(built) => RustExpr::Binary {
                op: port_engine_rust_ir::BinaryOp::Or,
                lhs: Box::new(built),
                rhs: Box::new(test),
            },
        });
    }
    guard.ok_or_else(|| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: "a switch case compares against nothing".to_owned(),
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
        // A CASE IS A COMPARISON, and only some comparisons are patterns. A literal is one, and so
        // is a name the source declares `const` — the target emits it as a `const`, which its
        // patterns accept. Anything else is a value read at run time: `case end:` where `end` is a
        // parameter compares against it, and `end =>` in the target BINDS it, shadowing the
        // parameter and matching everything. gjson's `validcomma` returned success for every byte
        // it should have rejected, and it compiled.
        //
        // So a non-constant case becomes a GUARD instead, which is what the source meant.
        let comparisons: Vec<&Declaration> = patterns_node.children.iter().collect();
        let constant = comparisons.iter().all(|node| is_pattern(node));
        let (patterns, guard) = match constant {
            true => (
                comparisons
                    .iter()
                    .map(|pattern| expression(pattern, cx))
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            ),
            false => (Vec::new(), Some(guard_for(&comparisons, tag, cx)?)),
        };
        if patterns.is_empty() && guard.is_none() {
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
            guard,
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

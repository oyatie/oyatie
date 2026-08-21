//! Which END of a channel a site holds, and what the target spells for it.
//!
//! The source's channel is ONE value carrying both ends. The target's is a pair: a sender that
//! clones and a receiver that does not. So `chan T` has no single target type, and asking for one
//! is the wrong question — the right one is which end THIS site holds, which the body answers.
//!
//! A parameter the body only sends on is a sender. One it only receives from is a receiver. One it
//! does both on has no single end, and refuses rather than being given the half that happens to be
//! spelled first.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt, SelectArm};

use crate::body::Body;
use crate::error::TransformError;

/// Which end of a channel a body holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum End {
    /// Only sent on.
    Sender,
    /// Only received from.
    Receiver,
}

/// Which end each of this declaration's channel PARAMETERS holds, by position.
pub(crate) fn parameter_ends(declaration: &Declaration) -> std::collections::BTreeMap<usize, End> {
    let Some(body) = declaration
        .children_of_kind(crate::vocabulary::CHILD_BODY)
        .first()
        .copied()
    else {
        return std::collections::BTreeMap::new();
    };
    declaration
        .children_of_kind(crate::vocabulary::CHILD_PARAM)
        .into_iter()
        .enumerate()
        .filter(|(_, param)| param.type_ref.kind == crate::vocabulary::TYPE_CHANNEL)
        .filter_map(|(index, param)| Some((index, end_held(body, &param.name)?)))
        .collect()
}

/// The end this body holds of the channel named `name`, or `None` where it holds neither or both.
pub(crate) fn end_held(body: &Declaration, name: &str) -> Option<End> {
    let mut sends = false;
    let mut receives = false;
    walk(body, name, &mut sends, &mut receives);
    match (sends, receives) {
        (true, false) => Some(End::Sender),
        (false, true) => Some(End::Receiver),
        _ => None,
    }
}

/// The same, for a body that may only HAND THE CHANNEL ONWARD.
///
/// `func Start(out chan int64) { go Produce(out) }` neither sends nor receives; it gives the
/// channel to something that does. The end it holds is therefore the end its callee holds, which is
/// a fact about the callee and reaches here through the signature table.
///
/// A name passed to two callees wanting DIFFERENT ends holds neither, and says so — the source's
/// one value could be both and the target's cannot.
pub(crate) fn end_held_or_passed(
    body: &Declaration,
    name: &str,
    signatures: &crate::signature_table::SignatureTable,
) -> Option<End> {
    if let Some(direct) = end_held(body, name) {
        return Some(direct);
    }
    let mut passed = None;
    handed_onward(body, name, signatures, &mut passed);
    passed.flatten()
}

/// Walk for calls that are given this name, collecting the end each callee holds of it.
///
/// `Some(None)` records a CONFLICT and is sticky: once two callees have disagreed, nothing later
/// resolves it.
fn handed_onward(
    node: &Declaration,
    name: &str,
    signatures: &crate::signature_table::SignatureTable,
    found: &mut Option<Option<End>>,
) {
    if node.kind == crate::vocabulary::KIND_CALL {
        let callee = node
            .attr(crate::vocabulary::ATTR_CALLEE)
            .unwrap_or_default();
        for (index, argument) in node.children.iter().skip(1).enumerate() {
            if !is_name(argument, name) {
                continue;
            }
            let end = signatures.channel_end(callee, index);
            *found = Some(match (*found, end) {
                (None, end) => end,
                (Some(seen), end) if seen == end => seen,
                _ => None,
            });
        }
    }
    for child in &node.children {
        handed_onward(child, name, signatures, found);
    }
}

/// Count the directions this body uses the name in.
fn walk(node: &Declaration, name: &str, sends: &mut bool, receives: &mut bool) {
    // `ch <- v`. The channel is the FIRST child and the value the second, so a name standing in the
    // value slot is being sent, not sent ON.
    if node.kind == crate::vocabulary::KIND_SEND
        && node
            .children
            .first()
            .is_some_and(|channel| is_name(channel, name))
    {
        *sends = true;
    }
    // `<-ch`, which the front end records as a unary operator because that is what the source
    // spells. The arrow is the only unary the target has no operator for at all.
    if node.kind == crate::vocabulary::KIND_UNARY
        && node.attr(crate::vocabulary::ATTR_OP) == Some(RECEIVE_OPERATOR)
        && node
            .children
            .first()
            .is_some_and(|channel| is_name(channel, name))
    {
        *receives = true;
    }
    for child in &node.children {
        walk(child, name, sends, receives);
    }
}

/// The source's receive operator, which it spells as a prefix arrow.
pub(crate) const RECEIVE_OPERATOR: &str = "<-";

fn is_name(node: &Declaration, name: &str) -> bool {
    node.kind == crate::vocabulary::KIND_IDENT && node.name == name
}

/// Whether this declaration's body performs a channel operation, and so may YIELD.
///
/// The target spells that on the signature — `.await` is legal only inside an `async` body — so the
/// signature and the statement that awaits have to read one answer, and this is it.
///
/// A `go` statement does NOT make its enclosing function suspend: the spawn hands the work to the
/// executor and returns, which is the whole of what the source's `go` says.
pub(crate) fn suspends(declaration: &Declaration) -> bool {
    declaration
        .children_of_kind(crate::vocabulary::CHILD_BODY)
        .first()
        .copied()
        .is_some_and(communicates)
}

/// Whether this subtree sends on or receives from any channel.
fn communicates(node: &Declaration) -> bool {
    if node.kind == crate::vocabulary::KIND_SEND {
        return true;
    }
    if node.kind == crate::vocabulary::KIND_UNARY
        && node.attr(crate::vocabulary::ATTR_OP) == Some(RECEIVE_OPERATOR)
    {
        return true;
    }
    // A nested function literal has its own body and its own colour.
    node.children
        .iter()
        .filter(|child| child.kind != "closure")
        .any(communicates)
}

/// A send, built from the pack's form as a TREE rather than as text.
///
/// The form is read for its shape and not substituted into: a mapped call is a tree, and text that
/// happens to parse is how a downstream rule stops being able to see what it is looking at. What
/// the pack decides here is WHICH methods and whether the failure aborts; the arrangement of them
/// is fixed by what the target's sender is.
///
/// # Errors
/// [`TransformError::Unsupported`] where the pack's form is not one this builder recognises.
pub(crate) fn sent(
    channel: RustExpr,
    value: RustExpr,
    form: &str,
    cx: &Body<'_>,
) -> Result<RustExpr, TransformError> {
    let Some((send, rest)) = form.split_once("({1})") else {
        return Err(unrecognised(form, cx));
    };
    let Some(method) = send.strip_prefix("{0}.") else {
        return Err(unrecognised(form, cx));
    };
    let sent = RustExpr::Await(Box::new(RustExpr::MethodCall {
        receiver: Box::new(channel),
        method: method.to_owned(),
        args: vec![value],
    }));
    // WHAT HAPPENS WHEN THE FAR END IS GONE. The source aborts; the target hands back a failure the
    // caller must do something with, and doing nothing with it is a warning this engine is held to.
    let Some(aborts) = rest.strip_prefix(".await.") else {
        return Err(unrecognised(form, cx));
    };
    let Some((abort, message)) = aborts.split_once('(') else {
        return Err(unrecognised(form, cx));
    };
    Ok(RustExpr::MethodCall {
        receiver: Box::new(sent),
        method: abort.to_owned(),
        args: vec![RustExpr::Literal(message.trim_end_matches(')').to_owned())],
    })
}

/// The pack wrote a send form this builder cannot take apart.
fn unrecognised(form: &str, cx: &Body<'_>) -> TransformError {
    TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "the pack's channel send form `{form}` is not one this engine builds: it expects the \
             channel, the send, the await and what to do when the far end is gone, and reads the \
             names out of the form rather than assuming them"
        ),
    }
}

/// `select { case v := <-ch: .. }` as the target's waiting macro.
///
/// EVERY ARM IS A RECEIVE THAT BINDS. The source admits a send arm and a bare receive too, and this
/// answers neither yet -- they refuse by name rather than being folded into the shape that happens
/// to be written.
///
/// A `default` arm refuses, and the reason is not that the target lacks one. The target's `else`
/// runs when every branch is DISABLED; the source's `default` runs when no arm is READY. A select
/// over an open channel with nothing in it has no ready arm and no disabled branch, so the source
/// takes its default and the target waits -- the same program doing two different things.
///
/// # Errors
/// [`TransformError::Unsupported`] for an arm shape this does not answer, or where the pack names
/// no select.
pub(crate) fn selected(node: &Declaration, cx: &Body<'_>) -> Result<RustStmt, TransformError> {
    let forms = cx.resolver.channel;
    if forms.select.is_empty() || forms.zero_on_close.is_empty() {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "a `select` has no target form, because the pack names no waiting macro"
                .to_owned(),
        });
    }
    let refuse = |why: &str| TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!("a `select` arm {why}"),
    };
    let mut arms = Vec::new();
    for clause in node.children_of_kind("comm_clause") {
        if clause.attr(crate::vocabulary::ATTR_DEFAULT).is_some() {
            return Err(refuse(
                "is the source's `default`, which runs when no arm is READY. The target's `else` \
                 runs when every branch is DISABLED, and a select over an open channel with \
                 nothing in it has neither a ready arm nor a disabled branch -- so the source takes \
                 its default there and the target waits",
            ));
        }
        let communication = clause
            .children_of_kind("comm")
            .first()
            .copied()
            .ok_or_else(|| refuse("has no communication"))?;
        let bound = communication
            .children
            .first()
            .filter(|statement| statement.kind == "let" && !statement.name.is_empty())
            .ok_or_else(|| {
                refuse("is not a receive that binds, which is the only arm shape answered yet")
            })?;
        let received = bound
            .children
            .first()
            .filter(|value| {
                value.kind == crate::vocabulary::KIND_UNARY
                    && value.attr(crate::vocabulary::ATTR_OP) == Some(RECEIVE_OPERATOR)
            })
            .ok_or_else(|| refuse("binds something that is not a receive"))?;
        let channel = received
            .children
            .first()
            .ok_or_else(|| refuse("receives from nothing"))?;
        let name = crate::naming::to_snake_case(&bound.name);
        let body = crate::body_parts::branch(clause, "body", cx)?;
        // THE ZERO ON CLOSE, bound over the arm's own name. The source's receive on a closed
        // channel hands back the element's zero and keeps going; the target's hands back nothing.
        // Shadowing rather than a second name, because the source has one name here and so should
        // the reader.
        let mut statements = vec![RustStmt::Let {
            name: name.clone(),
            mutable: false,
            ty: None,
            value: Some(zero_on_close(
                RustExpr::Path(name.clone()),
                &forms.zero_on_close,
            )),
        }];
        statements.extend(crate::body::translate(
            &body.children,
            cx,
            crate::body::TailPosition::No,
        )?);
        arms.push(SelectArm {
            binding: name,
            future: receiving(
                crate::body_expr::in_position(channel, cx, crate::body_expr::Position::Place)?,
                &forms.receive,
            )?,
            body: statements,
        });
    }
    if arms.is_empty() {
        return Err(refuse("list is empty, which waits forever on nothing"));
    }
    Ok(RustStmt::Select {
        path: forms.select.clone(),
        arms,
    })
}

/// The receive itself, read out of the pack's form rather than assumed.
fn receiving(channel: RustExpr, form: &str) -> Result<RustExpr, TransformError> {
    let method = form
        .strip_prefix("{0}.")
        .and_then(|rest| rest.strip_suffix("().await"))
        .unwrap_or("recv");
    Ok(RustExpr::MethodCall {
        receiver: Box::new(channel),
        method: method.to_owned(),
        args: Vec::new(),
    })
}

/// What a receive yields once the far end is gone.
fn zero_on_close(received: RustExpr, form: &str) -> RustExpr {
    let method = form
        .strip_prefix("{0}.")
        .and_then(|rest| rest.strip_suffix("()"))
        .unwrap_or("unwrap_or_default");
    RustExpr::MethodCall {
        receiver: Box::new(received),
        method: method.to_owned(),
        args: Vec::new(),
    }
}

/// `go f(x)` as the target's spawn.
///
/// The source's goroutine runs the call concurrently and its caller carries on. The target's spawn
/// says the same thing, and adds two requirements the source does not have — which is why this is a
/// rule rather than a spelling.
///
/// OWNED. What a spawned body names must outlive the frame that spawned it, so the block takes
/// ownership. The parameters that flow into it are therefore not borrowed by the enclosing
/// signature; see `params::spawned_arguments`, which reads the same statement this does.
///
/// AWAITED where the callee suspends. A spawned call to an async function is a future that does
/// nothing until it is driven, and dropping it on the floor is a goroutine that never runs. The
/// signature table says which callees those are, because only their own bodies do.
///
/// # Errors
/// [`TransformError::Unsupported`] where the pack names no spawn, or the statement is not a call.
pub(crate) fn spawned(node: &Declaration, cx: &Body<'_>) -> Result<RustStmt, TransformError> {
    let forms = cx.resolver.channel;
    if forms.spawn.is_empty() {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "a goroutine has no target form, because the pack names no spawn".to_owned(),
        });
    }
    let started = node
        .children
        .first()
        .filter(|child| child.kind == crate::vocabulary::KIND_CALL)
        .ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: "a goroutine starts something that is not a call, which the source allows and \
                     this does not answer yet"
                .to_owned(),
        })?;
    let callee = started
        .attr(crate::vocabulary::ATTR_CALLEE)
        .unwrap_or_default();
    let mut call = crate::body_expr::expression(started, cx)?;
    if cx.resolver.signatures.suspends(callee) {
        call = RustExpr::Await(Box::new(call));
    }
    let Some(path) = forms.spawn.split_once('(').map(|(head, _)| head) else {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "the pack's spawn form `{}` is not one this engine builds: it expects a call \
                 taking the body to run",
                forms.spawn
            ),
        });
    };
    Ok(RustStmt::Semi(RustExpr::Call {
        callee: Box::new(RustExpr::Path(path.to_owned())),
        args: vec![RustExpr::AsyncBlock(vec![RustStmt::Semi(call)])],
    }))
}

/// Every name a `go` statement in this body hands to the task it starts.
///
/// A spawned body OWNS what it names, so a parameter reaching one cannot also be borrowed by the
/// signature that receives it -- the borrow would not outlive the frame, which is exactly what the
/// target refuses. Read off the same statement `spawned` reads.
pub(crate) fn spawned_arguments(declaration: &Declaration) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    if let Some(body) = declaration
        .children_of_kind(crate::vocabulary::CHILD_BODY)
        .first()
    {
        collect_spawned(body, &mut names);
    }
    names
}

/// Walk for `go` statements, collecting the names their calls are given.
fn collect_spawned(node: &Declaration, into: &mut std::collections::BTreeSet<String>) {
    if node.kind == "go" {
        for argument in node
            .children
            .first()
            .map(|call| call.children.as_slice())
            .unwrap_or_default()
        {
            names_in(argument, into);
        }
    }
    for child in &node.children {
        collect_spawned(child, into);
    }
}

/// Every identifier this subtree names.
fn names_in(node: &Declaration, into: &mut std::collections::BTreeSet<String>) {
    if node.kind == crate::vocabulary::KIND_IDENT && !node.name.is_empty() {
        into.insert(node.name.clone());
    }
    for child in &node.children {
        names_in(child, into);
    }
}

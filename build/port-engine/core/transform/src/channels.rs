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
use port_engine_rust_ir::RustExpr;

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
        args: vec![RustExpr::Literal(
            message.trim_end_matches(')').to_owned(),
        )],
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

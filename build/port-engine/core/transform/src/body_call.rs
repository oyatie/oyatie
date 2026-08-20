//! A CALL, and the arguments it hands over.
//!
//! Separated from the expression walk because an argument is the one expression translated for a
//! DESTINATION rather than on its own terms. The signature table holds every parameter the engine
//! has translated, so `f(&x)` can take the construction that parameter's disposition declares and
//! a bare string literal can be owned when the parameter holds an owned string — both the same
//! decision the parameter position already made, applied at the other end.
//!
//! What a call resolves to was already the trickiest part of the front end: the source spells
//! `f(x)`, `value.Method()` and `package.Function()` with one production, and deciding by shape is
//! how a cross-package call became a method call on a binding that did not exist.

use port_engine_api::{Declaration, FunctionMapping, PointerConstruction};
use port_engine_rust_ir::RustExpr;

use crate::body::{Body};
use crate::body_parts::{one_child};
use crate::body_argument::argument;
use crate::body_expr::{Position, expression, in_position};
use crate::body_ops::{operator_of, own_string_for, reference};
use crate::error::TransformError;
use crate::naming::{module_path, to_snake_case};
use crate::vocabulary::{
    ARGUMENT_INT_LITERAL_LAST, ARGUMENT_STRING_LITERAL, ATTR_CALLEE, ATTR_CALLEE_KIND, ATTR_LIT_KIND, CALLEE_KIND_METHOD,
    KIND_LITERAL, KIND_UNARY, LIT_KIND_INT, LIT_KIND_STRING, OPERATOR_ADDRESS_OF,
};

/// A call, which is a method call when its callee is a field access.
///
/// The source spells both as one form and distinguishes them by what the callee resolves to; the
/// target spells them differently. A field access in callee position is a method call — a plain
/// field holding a function is a shape the corpus does not have and that refuses rather than being
/// silently rewritten into a method.
pub(crate) fn call(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let callee = node
        .children
        .first()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: "call".to_owned(),
            name: cx.owner.to_owned(),
            datum: "callee",
        })?;
    // Each argument is translated FOR ITS DESTINATION. The callee identity is what the signature
    // table is keyed by, and a call carrying none — a method — gets no destinations at all, which
    // the argument path reads as "cannot say" rather than as "no conversion needed".
    let callee_id = node.attr(ATTR_CALLEE).unwrap_or_default();
    // A VARIADIC callee needs its trailing arguments collected into a sequence, and which of the
    // target's sequence forms that is has not been decided — nor what `f(xs...)` does when the
    // caller forwards a slice it already has. Refused here rather than at the declaration: the
    // signature is an ordinary slice and ports fine, and only a CALL needs the answer.
    if cx.resolver.signatures.is_variadic(callee_id) {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "`{callee_id}` is variadic, and the target has no variadic call: the trailing \
                 arguments need collecting into a sequence, and the pack declares neither which \
                 sequence form nor what a forwarded slice becomes"
            ),
        });
    }
    // A SHORTHAND first, before the arguments are even translated: a call to one of these IS the
    // call it wraps, so what gets translated is that call with these arguments in it. Translating
    // the arguments here and again inside would decide their destinations against the wrong
    // signature — the wrapper's, rather than the wrapped callee's.
    if let Some(shorthand) = cx.resolver.signatures.eta(callee_id)
        && let Some(direct) = crate::eta::inlined(shorthand, &node.children[1..])
    {
        return expression(&direct, cx);
    }

    if let Some(built) = crate::body_alloc::allocation(node, callee_id, cx)? {
        return Ok(built);
    }

    let args = node.children[1..]
        .iter()
        .enumerate()
        .map(|(index, arg)| argument(arg, callee_id, index, cx))
        .collect::<Result<Vec<_>, _>>()?;
    // A MAPPED callee's form describes what the UNDERLYING value supports, so a newtype argument
    // reaches through its wrapper. `len(id)` on a named array becomes `id.0.len()`: the target's
    // length is the array's, and the newtype has none. Only for mapped callees — a call to one of
    // this unit's own functions takes the newtype itself, which is what its signature declares.
    let args = match cx.resolver.function_map.contains_key(callee_id) {
        true => node.children[1..]
            .iter()
            .map(|arg| crate::body_index::unwrapped_base(arg, cx))
            .collect::<Result<Vec<_>, _>>()?,
        false => args,
    };

    // The pack answers for the callee FIRST, by identity. A call it answers for is one the target
    // has no name of its own for — a builtin, or something from a standard library that does not
    // come along — and emitting the source's spelling would name nothing.
    // A FORMATTING call first: it also has a pack answer keyed by identity, but the answer needs
    // the source's own template read rather than substituted into, so it cannot go through the
    // table below.
    // A FORMAT operand reaches through a newtype wrapper for the same reason a mapped callee's
    // argument does: the target's formatter asks the value for a trait its underlying type
    // implements and its wrapper does not. `%d` on a `type Version byte` prints the byte, and the
    // wrapper has no `Display` at all — which reached `rustc` rather than being refused here.
    if !args.is_empty() {
        let mut operands = args.clone();
        for (offset, operand) in operands.iter_mut().enumerate().skip(1) {
            *operand = crate::body_index::unwrapped_base(&node.children[offset + 1], cx)?;
        }
        if let Some(rendered) = crate::body_format::formatted_call(node, &operands, cx)? {
            return Ok(rendered);
        }
    }
    // THE LENGTH OF AN ARRAY IS A CONSTANT. The source defines `len(a)` on an array type as a
    // constant expression — the length is part of the type — so folding it is the source's own
    // rule rather than an optimisation applied to it.
    //
    // It also removes a borrow the source does not have. `PutUint16(id[len(id)-2:], n)` writes into
    // a slice of `id` whose bound READS `id`, which the source allows and the target does not: the
    // read and the mutable borrow overlap. With the length folded there is no read, and the
    // emitted code is what someone would write.
    if let Some(folded) = array_length(node, cx) {
        return Ok(folded);
    }
    if let Some(rendered) = mapped_call(node, &args, cx)? {
        return Ok(rendered);
    }

    // A call through a RECEIVER, as the type-checker saw it — not as the syntax looked. The
    // source spells `value.Method()` and `package.Function()` the same way, and deciding by shape
    // emitted a method call on a package name.
    if node.attr(ATTR_CALLEE_KIND) == Some(CALLEE_KIND_METHOD) {
        let receiver_node = one_child(callee, cx, "selector")?;
        // A BYTE-ORDER call, which is a method on another package's value and so has no callee
        // identity for the ordinary map to key on. Answered before the receiver checks, because the
        // receiver here is a package's value rather than a value of this program — asking whether it
        // may be absent is asking about the wrong thing.
        if let Some(rendered) = crate::body_bytes::byte_order_call(callee, &args, cx)? {
            return Ok(rendered);
        }
        // ABSENCE FIRST. A receiver that may hold nothing has no methods of what it holds, and that
        // includes the mapped one below — mapping before checking turned `self.cause.Error()` into
        // `self.cause.to_string()` on an option, which is a different method that does not exist
        // either. The order is the rule: what the receiver IS has to be settled before what the
        // call becomes.
        refuse_absent_capable_receiver(receiver_node, &callee.name, cx)?;
        // The source interface's MESSAGE METHOD, which the target takes from a different trait. The
        // call is mappable even though the interface is not implementable — see the pack's
        // `message_method_reason`.
        if let Some(rendered) = message_method_call(receiver_node, &callee.name, cx)? {
            return Ok(rendered);
        }
        refuse_dropped_method(receiver_node, &callee.name, cx)?;
        return Ok(RustExpr::MethodCall {
            // The receiver of a method call is a PLACE, not a value: `x.m()` borrows `x` rather
            // than reading it, so cloning here would call the method on a temporary.
            receiver: Box::new(in_position(receiver_node, cx, Position::Place)?),
            method: to_snake_case(&callee.name),
            args,
        });
    }

    // A free function, named by the path its identity resolves to. A local one keeps its bare
    // name; one from another unit is reached through that unit's emitted module, the same way a
    // type from another unit is.
    let path = cx
        .resolver
        .function_path(node.attr(ATTR_CALLEE), cx.owner)?;
    Ok(RustExpr::Call {
        callee: Box::new(RustExpr::Path(path)),
        args,
    })
}

/// A call the pack answers for by the callee's IDENTITY, rendered from its declared template.
///
/// Arity is checked rather than assumed: a template that expects an argument the call does not have
/// would leave its own placeholder in the output, which parses as nothing and would be discovered
/// far from its cause.
fn mapped_call(
    node: &Declaration,
    args: &[RustExpr],
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let Some(identity) = node.attr(ATTR_CALLEE) else {
        return Ok(None);
    };
    let Some(mapping) = cx.resolver.function_map.get(identity) else {
        return Ok(None);
    };
    refuse_wrong_argument_shape(node, identity, mapping, cx)?;

    // STRUCTURED where the form is one the target spells as a method call, which is most of them.
    // A mapped call used to arrive as target TEXT, and text is opaque to everything downstream: the
    // accumulator fold cannot substitute into it, so a body whose chain passes through one kept its
    // statements. The tree costs nothing and every later rule can see through it.
    if let Some(built) = crate::body_mapped::structured_form(&mapping.form, args) {
        return Ok(Some(built));
    }

    let mut rendered = mapping.form.clone();
    for (index, arg) in args.iter().enumerate() {
        let operand = render_operand(arg).ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "an argument to `{identity}` is a compound expression, and the pack answers for \
                 that call with a TEXT template — substituting one would need parentheses the \
                 template cannot ask for"
            ),
        })?;
        rendered = rendered.replace(&format!("{{{index}}}"), &operand);
    }
    if rendered.contains('{') {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "the pack's template for `{identity}` expects more arguments than the call has"
            ),
        });
    }
    Ok(Some(structured(&rendered, &mapping.form)))
}

/// A rendered mapping as an EXPRESSION, with a trailing conversion read as one.
///
/// The pack's forms are text templates, and one of them ends in a conversion: `{0}.len() as i64`.
/// Handed to the IR as a flat literal, nothing downstream can see that the outermost thing is a
/// cast — and `xs.len() as i64` with a method on it renders as `xs.len() as i64.wrapping_sub(1)`,
/// which the target REJECTS outright. Two whole packages failed to render on exactly that, and the
/// hermetic corpus never had a cast with a method on it.
///
/// Read from the FORM the pack declares rather than by sniffing the rendered text, so a template
/// with no trailing conversion is left exactly alone and a pack that changes one changes this.
fn structured(rendered: &str, form: &str) -> RustExpr {
    let Some((_, target)) = form.rsplit_once(" as ") else {
        return RustExpr::Literal(rendered.to_owned());
    };
    // A TRAILING cast, and nothing else. A form whose ` as ` sits INSIDE it —
    // `{0}.rotate_left({1} as u32)` — is not a cast of the whole call, and reading one there took
    // `u32)` for a type and cut the rendered text at the paren: `x.rotate_left(31`, which does not
    // parse. The whole point of recognising the cast is that a cast is postfix-hostile and needs
    // bracketing, and only a cast of the WHOLE expression does.
    if !target
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return RustExpr::Literal(rendered.to_owned());
    }
    match rendered
        .strip_suffix(&format!(" as {target}"))
        .filter(|inner| !inner.is_empty())
    {
        Some(inner) => RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(inner.to_owned())),
            ty: port_engine_rust_ir::RustType::path(target),
        },
        None => RustExpr::Literal(rendered.to_owned()),
    }
}

/// Refuse a mapped call whose argument is not the shape the mapping declares.
///
/// Some mappings hold for any argument and some do not. `panic` is the case this exists for: Go's
/// `panic(v)` aborts carrying `v` and Rust's `panic!` aborts carrying a formatted string, so where
/// `v` is a STRING LITERAL the two are the same abort with the same message and the same payload
/// type — and where it is an error or an arbitrary value the payload TYPE is lost, which a caller
/// that recovers and type-asserts on it would see as a different program.
///
/// The condition is pack data and its vocabulary is CLOSED: a shape the engine has never heard of
/// refuses rather than being read as "no condition", because a condition nobody checks is a
/// condition that is not there.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the call, the shape required, and the shape found.
fn refuse_wrong_argument_shape(
    node: &Declaration,
    identity: &str,
    mapping: &FunctionMapping,
    cx: &Body<'_>,
) -> Result<(), TransformError> {
    let Some(required) = mapping.requires_argument.as_deref() else {
        return Ok(());
    };
    let argument = node.children.get(1);
    let holds = match required {
        ARGUMENT_STRING_LITERAL => argument.is_some_and(|arg| {
            arg.kind == KIND_LITERAL && arg.attr(ATTR_LIT_KIND) == Some(LIT_KIND_STRING)
        }),
        // A COUNT, and the target infers its own width for a literal one. Where the source passes a
        // variable instead, the width has to be converted and the source's `int` is not the
        // target's — so the mapping holds for the literal and refuses for the rest rather than
        // carrying a cast that is wrong for one of them.
        ARGUMENT_INT_LITERAL_LAST => node.children.last().is_some_and(|arg| {
            arg.kind == KIND_LITERAL && arg.attr(ATTR_LIT_KIND) == Some(LIT_KIND_INT)
        }),
        unknown => {
            return Err(TransformError::Unsupported {
                name: cx.owner.to_owned(),
                detail: format!(
                    "the pack requires argument shape `{unknown}` for `{identity}`, which is not \
                     a shape this engine knows how to check"
                ),
            });
        }
    };
    if holds {
        return Ok(());
    }
    Err(TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "`{identity}` is answered by the pack only for a `{required}` argument, and this call \
             passes `{}` — {}",
            argument.map_or("nothing", |arg| arg.kind.as_str()),
            mapping.reason
        ),
    })
}

/// An argument, as target text for a template to interpolate.
///
/// Only the forms whose text is unambiguous are admitted. A template is textual substitution, and
/// substituting a compound expression into one would need parentheses this cannot see the need for —
/// so anything else refuses rather than producing text that reassociates.
pub(crate) fn render_operand(arg: &RustExpr) -> Option<String> {
    match arg {
        RustExpr::Literal(text) | RustExpr::Path(text) => Some(text.clone()),
        _ => None,
    }
}

/// Refuse a method call whose receiver may hold NOTHING in the target.
///
/// The source's pointer admits its absent value, so a FIELD of pointer type is emitted as an option
/// — that mapping is right and is not what is in question here. What is in question is the call: the
/// source spells `c.con.Major()` and calling a method on an absent pointer is legal there, where the
/// target has no method of that name on an option at all. Emitted as written it does not compile,
/// and 39 of one package's 51 compile errors were this one shape.
///
/// Neither answer the engine could invent is faithful. Unwrapping claims a value the source never
/// promised is present and panics where the source ran; mapping over the option silently skips a
/// call the source made. Both are decisions about what the program DOES when the pointer is absent,
/// which the source states nowhere, so this refuses and says which proof is missing.
///
/// Narrow on purpose: only a FIELD read, because that is the position the engine is known to emit as
/// an option. A pointer PARAMETER is bound by the ownership rules to a borrow and has no absent
/// case, and refusing one would refuse a call that translates correctly today.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the receiver, the method, and what would have to be proved.
fn refuse_absent_capable_receiver(
    receiver: &Declaration,
    method: &str,
    cx: &Body<'_>,
) -> Result<(), TransformError> {
    let Some(type_ref) = receiver_type_ref(receiver, cx) else {
        return Ok(());
    };
    // ASKED of the resolver, and of EVERY receiver rather than only the pointer-typed ones. What
    // makes a receiver unusable is that its target type is absent-capable, and a pointer is not
    // the only way there: a stored failure is an option too, because the source's error interface
    // admits its absent value. Gating on the source's kind missed exactly that. A source pointer,
    // meanwhile, does not always become an option -- the ownership rules give some a borrow, which
    // has no absent case -- so the question is only ever what THIS occurrence resolved to.
    let position = match receiver.kind == crate::vocabulary::KIND_SELECTOR {
        true => crate::vocabulary::POSITION_FIELD,
        false => crate::vocabulary::POSITION_PARAM,
    };
    let Ok(resolved) = cx.resolver.resolve_in(&type_ref, cx.owner, position) else {
        return Ok(());
    };
    if !resolved.spelling().starts_with("Option<") {
        return Ok(());
    }
    Err(TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "`{}` is of pointer type and resolves to a value that may be ABSENT, \
             and `{method}` is a method of what it holds rather than of the option. The source \
             permits this call when the pointer is absent and says nothing about what the program \
             does then; unwrapping would panic where the source ran, and skipping the call would \
             drop work the source performed. What is missing is a proof that this field is never \
             absent at the call",
            receiver.name
        ),
    })
}

/// Refuse a call to a method the receiver's own type is not emitting.
///
/// Breaking the type/method cascade made this necessary: a method the engine cannot translate is now
/// dropped from its type's `impl` while the type itself is emitted, so a call to one names something
/// the emitted crate does not contain. Every other kind of reference is already governed by that
/// rule; a method call was not, and unlike a missing function it fails at the call site with nothing
/// to say why.
///
/// The owner comes from the RECEIVER TYPE when the receiver is the enclosing method's own — a
/// receiver carries no recorded type, so an earlier version of this check asked the receiver node
/// and was silently inert for every `self.method()` call in the corpus.
///
/// Silent where the type is not this unit's or was not recorded: a check that cannot see the type
/// cannot claim the method is absent, and refusing on absence of evidence would refuse every call
/// the type-checker happened not to annotate.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the type and the method, when the type is emitted and the
/// method is not.
fn refuse_dropped_method(
    receiver: &Declaration,
    method: &str,
    cx: &Body<'_>,
) -> Result<(), TransformError> {
    let owner = match crate::body_ops::is_receiver(receiver) {
        true => cx.receiver_type.unwrap_or_default(),
        false => receiver
            .type_ref
            .name
            .rsplit('.')
            .next()
            .unwrap_or_default(),
    };
    if owner.is_empty() || !cx.resolver.emitted.contains(owner) {
        return Ok(());
    }
    if cx.resolver.emitted.contains(&format!("{owner}::{method}")) {
        return Ok(());
    }
    Err(TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "`{owner}` is emitted but its method `{method}` is not — that method refused, so a call \
             to it would name something the crate does not contain. What is emitted has to be \
             self-contained"
        ),
    })
}

/// A call to the source interface's message method, as the target's own.
///
/// Recognised by the receiver's TYPE being the failure interface, never by the method's name alone:
/// a type of this corpus may declare a method with the same name meaning something else, and
/// rewriting that would be answering for a call the pack never spoke about.
fn message_method_call(
    receiver: &Declaration,
    method: &str,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let Some(convention) = cx.resolver.failure else {
        return Ok(None);
    };
    if convention.message_method.is_empty()
        || method != convention.message_method_source
    {
        return Ok(None);
    }
    if !receiver_type_ref(receiver, cx).is_some_and(|found| cx.resolver.is_failure_type(&found)) {
        return Ok(None);
    }
    Ok(Some(RustExpr::MethodCall {
        receiver: Box::new(in_position(receiver, cx, Position::Place)?),
        method: convention.message_method.clone(),
        args: Vec::new(),
    }))
}

/// The source TYPE a receiver expression has, where the body can work it out.
///
/// The front end records a type on an expression only where one is needed, so an index into a
/// sequence carries none — and the two rules that ask what a receiver IS were both silently inert on
/// exactly that shape. What the body can reconstruct is the one case the corpus has: an index whose
/// base is a value of one of this unit's newtypes, whose underlying is a sequence, has the sequence's
/// ELEMENT type.
///
/// Deliberately not more than that. A general expression-typer is the front end's job, and guessing
/// here would answer for shapes nobody measured.
fn receiver_type_ref(receiver: &Declaration, cx: &Body<'_>) -> Option<port_engine_api::TypeRef> {
    if !receiver.type_ref.name.is_empty() || !receiver.type_ref.kind.is_empty() {
        return Some(receiver.type_ref.clone());
    }
    if receiver.kind != crate::vocabulary::KIND_INDEX {
        return None;
    }
    let base = receiver.children.first()?;
    let owner = match crate::body_ops::is_receiver(base) {
        true => cx.receiver_type?,
        false => base.type_ref.name.as_str(),
    };
    let underlying = cx.resolver.scope.newtypes.get(owner)?;
    underlying.args.first().cloned()
}

/// `len(a)` where `a`'s type is an ARRAY, folded to the length the type states.
///
/// Only an array. A slice's length is a run-time property and folding it would be a different
/// program; the source draws the same line, which is why `len` of an array is a constant expression
/// there and `len` of a slice is not.
///
/// Reaches through a NEWTYPE, because the source's named array IS the array: `type KSUID [20]byte`
/// makes `len(id)` twenty for the same reason `len([20]byte{})` is.
fn array_length(node: &Declaration, cx: &Body<'_>) -> Option<RustExpr> {
    let identity = node.attr(ATTR_CALLEE)?;
    if !cx.resolver.length_functions.contains(identity) {
        return None;
    }
    let [_, argument] = node.children.as_slice() else {
        return None;
    };
    let mut declared = argument.type_ref.clone();
    // A RECEIVER carries no type, so the newtype it belongs to is what the body knows instead.
    if declared.is_empty() && crate::body_ops::is_receiver(argument) {
        declared = cx
            .resolver
            .scope
            .newtypes
            .get(cx.receiver_type?)
            .cloned()?;
    }
    if let Some(underlying) = cx.resolver.scope.newtypes.get(&declared.name) {
        declared = underlying.clone();
    }
    if declared.kind != "array" {
        return None;
    }
    declared.name.parse::<usize>().ok().map(|length| {
        RustExpr::Literal(
            crate::items_value::readable_literal(&length.to_string(), cx.resolver)
                .unwrap_or_else(|| length.to_string()),
        )
    })
}

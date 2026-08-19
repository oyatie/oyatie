//! Function bodies: source statements into IR nodes.
//!
//! The supported subset is small ON PURPOSE and everything outside it refuses BY NAME. A
//! translator that guesses at a construct it does not understand emits code that compiles and is
//! wrong, which the receipt then certifies as reproducible.
//!
//! Nothing here builds text. Operator precedence is the IR's problem now, which is why the
//! operator tables in [`crate::body_ops`] map to typed operators rather than to spellings: a
//! spelling has to be parenthesised defensively, a typed operator carries its own binding power.

use std::collections::BTreeSet;

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt, TupleBind};

use crate::body_cond::conditional;
use crate::body_expr::{Position, expression, in_position};
use crate::body_ops::{binary_operator, returns_owned_string};
use crate::body_failure::{propagate, propagate_into_success, translated_return};
use crate::body_loops::{counted_loop, range_loop, switch};
use crate::body_parts::{branch, named_child, one_child, two_children, unsupported_source};
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::resolve::Resolver;
use crate::vocabulary::{
    ATTR_OP, ATTR_SOURCE_NODE, CHILD_BIND, CHILD_PLACE, CHILD_VALUE, FLAG_MUTATED,
};

/// What one body translation needs in order to answer a question about the TARGET.
///
/// Threaded rather than ambient. An earlier version kept the copy-type set in a thread-local to
/// avoid the plumbing, and the moment a second pack table arrived the shortcut stopped paying for
/// itself — these tables are not properties of the process, they are properties of the rule pack,
/// and a body translated under a different pack must see different answers.
pub(crate) struct Body<'a> {
    /// The declaration being translated. Every refusal names it, which is the whole reason it is
    /// carried down rather than reconstructed at the top.
    pub(crate) owner: &'a str,
    /// The pack's answers: type mapping, copy types, zero values, ownership.
    pub(crate) resolver: &'a Resolver<'a>,
    /// Whether this function can FAIL — whether its results end in the failure type.
    ///
    /// A property of the signature that only the body can spend: the same `return x, y` is two
    /// different target constructions depending on it, and nothing inside a return says which.
    pub(crate) fallible: bool,
    /// Parameter names the signature BORROWS.
    ///
    /// The transform decided which ones those are when it built the signature, so this is the same
    /// answer rather than a second derivation. A borrowed value reaching a position that OWNS —
    /// a struct literal's field — has to be owned there, and the source did not have to say so
    /// because its string and its slice were already shared.
    pub(crate) borrowed: BTreeSet<String>,
    /// Whether the single result resolves to the OWNED target for a source string.
    ///
    /// A property of the signature that only the body can spend, exactly like `fallible`: a bare
    /// string literal being returned is a `&'static str` in the target and a `string` in the
    /// source, and nothing inside the `return` says which the destination wants.
    pub(crate) result_is_owned_string: bool,
    /// The sequence a counted loop WALKS, and the name its element took.
    ///
    /// Set inside a loop the iterator idiom rewrote, so `xs[i]` inside it renders as the element
    /// rather than as an index into a counter that no longer exists. Scoped to that loop: a nested
    /// loop over a different sequence gets its own answer, and nothing outside is affected.
    pub(crate) walked: Option<Walked>,
    /// The type this body is a method OF, and `None` for a free function.
    ///
    /// The receiver is not a child of the method declaration and carries no type, so a body has no
    /// way to learn what `self` IS except by being told — which matters because the target's newtype
    /// carries none of its underlying type's operators and the source's defined type carries all of
    /// them.
    pub(crate) receiver_type: Option<&'a str>,
    /// Counter names proven to be used for NOTHING BUT indexing, which are `usize` in the target.
    ///
    /// A property of the enclosing LOOP that only the operands inside it can spend: the range
    /// builds the counter and the index reads it, and neither one alone can see that the signed
    /// value is never observed. Both read this rather than each deciding, because they must agree
    /// or the loop does not compile.
    /// What the SIGNATURE decided about the results, which only the body can spend.
    ///
    /// One value rather than three because they are one thing and they grow together: every result
    /// idiom this engine adds is another fact the return operand cannot see for itself. The return
    /// operand is `&T{..}`, or `r.field`, or `len(x)` whatever the destination wants — and the
    /// signature and the body must agree or the emitted function does not compile.
    pub(crate) results: crate::returns::ResultFacts,
    pub(crate) usize_counters: BTreeSet<String>,
}

impl<'a> Body<'a> {
    pub(crate) fn new(
        owner: &'a str,
        resolver: &'a Resolver<'a>,
        fallible: bool,
        result_is_owned_string: bool,
        borrowed: BTreeSet<String>,
        results: crate::returns::ResultFacts,
    ) -> Self {
        Self {
            owner,
            resolver,
            fallible,
            result_is_owned_string,
            borrowed,
            results,
            usize_counters: BTreeSet::new(),
            walked: None,
            receiver_type: None,
        }
    }

}

/// The one sequence a rewritten loop walks, and what its element is called.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Walked {
    /// The source name of the counter the loop no longer has.
    pub(crate) counter: String,
    /// The source name of the sequence being walked.
    pub(crate) sequence: String,
    /// The target name the element took.
    pub(crate) element: String,
}

/// Whose signature this body has to satisfy.
///
/// A body normally answers to the declaration it belongs to, and the two are built together so they
/// cannot disagree. A TRAIT IMPL splits them: the signature comes from the trait's method and the
/// body from the type's own, and a body built for its own signature would then be wrong for the one
/// it is spliced into. So the splicer says which, rather than the body guessing.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ResultShape {
    /// The body's own declaration decides, which is every case but a trait impl.
    Own,
    /// The signature comes from elsewhere and fixes the shape, so no result idiom applies.
    Inherited,
}

/// Translate a function body's statements.
///
/// A trailing `return` becomes a TAIL EXPRESSION. That is a target-language idiom rather than a
/// change of meaning — `return x;` as the last statement of a function and `x` are the same
/// program — and it is owned here for the same reason identifier casing is: this face renders
/// Rust, so Rust's conventions are its business.
///
/// # Errors
/// [`TransformError::Unsupported`] for any construct outside the translated subset.
pub(crate) fn statements(
    nodes: &[Declaration],
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    result: ResultShape,
    // The type a METHOD hangs off, and `None` for a free function. Nothing in the method
    // declaration says it — the receiver is not a child and carries no type — so it is threaded
    // from the signature, which is the only place that knows.
    receiver_type: Option<&str>,
) -> Result<Vec<RustStmt>, TransformError> {
    // The names a signature's results carry are BINDINGS the body may assign to before returning,
    // and the target binds nothing from a signature — so they are bound here, ahead of everything
    // the body does, exactly as the source binds them at entry.
    let mut bound = crate::body_wider::named_result_bindings(declaration, resolver)?;
    let fallible = crate::failure::is_fallible(declaration, resolver.failure);
    let mut translated = translate(
        nodes,
        &Body::new(
            &declaration.name,
            resolver,
            fallible,
            returns_owned_string(declaration, resolver),
            crate::params::borrowed_parameters(declaration, resolver),
            crate::returns::ResultFacts::of(declaration, resolver, result),
        )
        // A parameter the signature made a `usize` is one for the body too, and it reaches the
        // index through the same set a proven loop counter does — so one place decides, and the
        // signature and the body cannot disagree about whether a conversion is needed.
        .with_usize_parameters(crate::index_params::index_parameters(declaration, resolver))
        .with_receiver_type(receiver_type),
        TailPosition::Yes,
    )?;
    bound.append(&mut translated);
    Ok(bound)
}

/// Whether the last statement of this sequence is in TAIL position — the position whose value is
/// the enclosing block's value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum TailPosition {
    Yes,
    No,
}

pub(crate) fn translate(
    nodes: &[Declaration],
    cx: &Body<'_>,
    tail: TailPosition,
) -> Result<Vec<RustStmt>, TransformError> {
    let mut out = Vec::with_capacity(nodes.len());
    let mut index = 0;
    while index < nodes.len() {
        // A body that PROPAGATES AND SUCCEEDS is the call itself. `check(s)?; Ok(())` runs the
        // call, hands its failure out, and reports success — which is what the call already does.
        // The extra shape exists because the SOURCE could not say it in one statement, and the
        // target can; a reviewer reading `check(s)?; Ok(())` in a two-line function said so.
        if let Some(forwarded) = crate::body_forward::forwarded_call(nodes, index, cx, tail)? {
            out.push(forwarded);
            index += 3;
            continue;
        }
        // The propagation idiom spans TWO statements, so it is matched here rather than inside
        // `statement`: a bind alone says nothing, and the check that follows is what decides
        // whether the pair is an operator or two ordinary statements.
        if let Some(found) = crate::failure::propagation(nodes, index, cx.resolver.failure) {
            out.push(propagate(&found, cx)?);
            index += 2;
            continue;
        }
        // The UNCHECKED propagation, which spans two statements for the same reason: `err := f()`
        // alone says nothing, and it is the return that follows which decides whether the pair is
        // an operator and a success or two ordinary statements.
        if cx.fallible
            && let Some(found) =
                crate::failure::tail_propagation(nodes, index, cx.resolver.failure)
        {
            out.extend(propagate_into_success(&found, cx)?);
            index += 2;
            continue;
        }
        // THE COMPARISON LADDER, which is the target's `cmp` written out because the source has no
        // such method. Whole-body, so it is matched once before any statement is.
        if index == 0
            && cx.results.is_an_ordering
            && let Some((left, right)) = crate::comparison::comparison_ladder_of(nodes, cx)
        {
            out.push(RustStmt::Tail(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Path(crate::naming::to_snake_case(&left))),
                method: "cmp".to_owned(),
                args: vec![RustExpr::Reference {
                    mutable: false,
                    inner: Box::new(RustExpr::Path(crate::naming::to_snake_case(&right))),
                }],
            }));
            break;
        }
        // A CHOICE the source had to spell as a mutation: `x := 0; if c { x = a } else { x = b }`.
        // Two statements for one construct, matched here for the same reason the propagation pair
        // is — the binding alone says nothing, and it is the `if` that follows which decides.
        if let Some(chosen) = crate::body_choice::choice(nodes, index, cx)? {
            index += chosen.statements;
            out.push(chosen.statement);
            continue;
        }
        // An `if` INIT CLAUSE whose binding COPIES needs no block. The block exists to scope the
        // name, and scoping only matters where it can be observed — a copy type has no drop to
        // delay, so the only thing left to observe is shadowing, which is checked. What remains
        // reads as a binding and an `if`, which is what someone would have written.
        if let Some(hoisted) = crate::body_cond::hoisted_init(nodes, index, cx)? {
            out.extend(hoisted);
            index += 1;
            continue;
        }
        let is_tail = tail == TailPosition::Yes && index + 1 == nodes.len();
        out.push(crate::body_stmt::statement(&nodes[index], cx, is_tail)?);
        index += 1;
    }
    Ok(out)
}

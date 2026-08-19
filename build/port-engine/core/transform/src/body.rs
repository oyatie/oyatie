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
use crate::body_failure::{propagate, translated_return};
use crate::body_loops::{counted_loop, range_loop, switch};
use crate::error::TransformError;
use crate::naming::to_snake_case;
use crate::resolve::Resolver;
use crate::vocabulary::{ATTR_OP, ATTR_SOURCE_NODE, CHILD_BIND, CHILD_VALUE, FLAG_MUTATED};

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
}

impl<'a> Body<'a> {
    pub(crate) fn new(
        owner: &'a str,
        resolver: &'a Resolver<'a>,
        fallible: bool,
        result_is_owned_string: bool,
        borrowed: BTreeSet<String>,
    ) -> Self {
        Self {
            owner,
            resolver,
            fallible,
            result_is_owned_string,
            borrowed,
        }
    }
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
) -> Result<Vec<RustStmt>, TransformError> {
    let fallible = crate::failure::is_fallible(declaration, resolver.failure);
    translate(
        nodes,
        &Body::new(
            &declaration.name,
            resolver,
            fallible,
            returns_owned_string(declaration, resolver),
            crate::params::borrowed_parameters(declaration, resolver),
        ),
        TailPosition::Yes,
    )
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
        // The propagation idiom spans TWO statements, so it is matched here rather than inside
        // `statement`: a bind alone says nothing, and the check that follows is what decides
        // whether the pair is an operator or two ordinary statements.
        if let Some(found) = crate::failure::propagation(nodes, index, cx.resolver.failure) {
            out.push(propagate(&found, cx)?);
            index += 2;
            continue;
        }
        let is_tail = tail == TailPosition::Yes && index + 1 == nodes.len();
        out.push(statement(&nodes[index], cx, is_tail)?);
        index += 1;
    }
    Ok(out)
}

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
            // A declared type is carried through. The source often declares one where the target
            // could infer it, and dropping it would change what the binding IS wherever the two
            // languages default differently — an untyped integer literal being the common case.
            ty: match node.type_ref.is_empty() {
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
        "break" => Ok(RustStmt::Break),
        "for" => counted_loop(node, cx),
        "range" => range_loop(node, cx),
        "switch" => Ok(RustStmt::Semi(switch(node, cx)?)),
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

/// A named child holding a statement list.
pub(crate) fn branch<'a>(
    node: &'a Declaration,
    kind: &str,
    cx: &Body<'_>,
) -> Result<&'a Declaration, TransformError> {
    node.children_of_kind(kind)
        .first()
        .copied()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: node.kind.clone(),
            name: cx.owner.to_owned(),
            datum: "body",
        })
}

/// The one child of a given kind, named in the refusal when it is absent.
pub(crate) fn named_child<'a>(
    node: &'a Declaration,
    kind: &'static str,
    cx: &Body<'_>,
    construction: &str,
) -> Result<&'a Declaration, TransformError> {
    node.children_of_kind(kind)
        .first()
        .copied()
        .ok_or_else(|| TransformError::MissingDatum {
            construction: construction.to_owned(),
            name: cx.owner.to_owned(),
            datum: kind,
        })
}

pub(crate) fn unsupported_source(node: &Declaration, cx: &Body<'_>) -> TransformError {
    let source_node = node
        .attr(ATTR_SOURCE_NODE)
        .unwrap_or("an unnamed construct");
    TransformError::Unsupported {
        name: cx.owner.to_owned(),
        detail: format!(
            "source construct `{source_node}` has no translation yet — a rule for it belongs in \
             the pack, and the analysis in docs/programs/k8s-port/census/"
        ),
    }
}

pub(crate) fn one_child<'a>(
    node: &'a Declaration,
    cx: &Body<'_>,
    what: &str,
) -> Result<&'a Declaration, TransformError> {
    node.children
        .first()
        .ok_or_else(|| TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("`{what}` node carries no operand"),
        })
}

pub(crate) fn two_children<'a>(
    node: &'a Declaration,
    cx: &Body<'_>,
    what: &str,
) -> Result<(&'a Declaration, &'a Declaration), TransformError> {
    match node.children.as_slice() {
        [lhs, rhs] => Ok((lhs, rhs)),
        other => Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!("`{what}` node needs two operands, got {}", other.len()),
        }),
    }
}

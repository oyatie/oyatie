//! Constants that are LENGTHS, proved from what the unit compares them against.
//!
//! `const maxVersionLen = 256` is the source's own integer, and the type map sends it to the
//! target's signed one — right for a value the source typed that way, and wrong for a bound on a
//! length. Every guard then reads `s.len() as i64 > MAX_VERSION_LEN`, a cast per call site and one
//! chance each to get the direction wrong; and because the constant is public, the casts leak to
//! every caller. A reviewer reading a real ported package called it the most consequential finding
//! in the file.
//!
//! Proved rather than guessed, and from the whole unit rather than from the declaration: a constant
//! is a length when everything that reads it compares it against one. One read that is anything
//! else — arithmetic, an argument, a return — and the signed value IS observed somewhere, so it
//! keeps its type.
//!
//! At least one read is required. A constant nothing reads has no evidence either way, and "every
//! read qualifies" over none of them is vacuously true, which would retype every unused constant
//! in the corpus on no evidence at all.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::Declaration;

use crate::vocabulary::{ATTR_CALLEE, KIND_CALL, KIND_IDENT, KIND_LITERAL, REF_CONST, SOURCE_INT};

/// Every constant of this unit that is a length.
pub(crate) fn length_constants(
    declarations: &[Declaration],
    lengths: &BTreeSet<String>,
    renders: &BTreeSet<String>,
    takes_length: &BTreeSet<String>,
) -> BTreeSet<String> {
    let candidates: BTreeSet<String> = declarations
        .iter()
        .filter(|declaration| {
            declaration.kind == "const" && declaration.type_ref.name == SOURCE_INT
        })
        .map(|declaration| declaration.name.clone())
        .collect();
    if candidates.is_empty() {
        return BTreeSet::new();
    }

    let mut reads: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for declaration in declarations {
        count_reads(
            declaration,
            &candidates,
            lengths,
            renders,
            takes_length,
            &mut reads,
        );
    }
    let proven: BTreeSet<String> = reads
        .into_iter()
        .filter(|(_, (total, against_length))| *total > 0 && total == against_length)
        .map(|(name, _)| name)
        .collect();
    derived_lengths(declarations, &candidates, proven)
}

/// The proven set, closed over the constants DERIVED from it.
///
/// `byteLength = timestampLengthInBytes + payloadLengthInBytes` is a length because what it is built
/// from are lengths, and nothing else about it needs proving. Without this the rule retypes one
/// constant and leaves its neighbour signed, and the sum of the two no longer typechecks — which is
/// exactly what happened the first time the index rule proved `timestampLengthInBytes` on its own.
///
/// A FIXPOINT, because a derived constant can feed another. The set only grows and every round adds
/// at least one member or stops, so it terminates for the same reason the emittability fixpoint does
/// — read in the other direction.
///
/// Every operand must be a member or a literal. One operand that is neither and the value is
/// something else, which is the same bar `is_length_arithmetic` sets for a read.
fn derived_lengths(
    declarations: &[Declaration],
    candidates: &BTreeSet<String>,
    mut proven: BTreeSet<String>,
) -> BTreeSet<String> {
    loop {
        let mut grew = false;
        for declaration in declarations {
            if declaration.kind != "const"
                || declaration.type_ref.name != SOURCE_INT
                || proven.contains(&declaration.name)
            {
                continue;
            }
            let Some(value) = declaration.children.first() else {
                continue;
            };
            // UNIFICATION, not one-way derivation. `byteLength = timestampLengthInBytes +
            // payloadLengthInBytes` proves nothing on its own about the second operand — the source
            // types all three the same and says no more. What supplies the missing constraint is the
            // TARGET: an index type and a signed integer do not add, so if one operand must be the
            // index type its partner must be too, and so must the sum. That is a fact about the
            // language being emitted, not a guess about the one being read.
            //
            // So a group is: the declaration and every int constant its value names. If any member
            // is proven, all of them are. This is what makes the rule safe to apply at all — proving
            // one constant and leaving its neighbour signed does not typecheck, and did not.
            let mut group = names_in(value, candidates);
            if group.is_empty() {
                continue;
            }
            group.insert(declaration.name.clone());
            if !built_from(value, candidates) || !group.iter().any(|name| proven.contains(name)) {
                continue;
            }
            for name in group {
                grew |= proven.insert(name);
            }
        }
        if !grew {
            return proven;
        }
    }
}

/// Whether every leaf of this expression is one of the unit's int constants or a literal.
///
/// The bar is the same one `is_length_arithmetic` sets for a read: one operand that is neither and
/// the value is something else, and nothing about it can be concluded.
fn built_from(node: &Declaration, candidates: &BTreeSet<String>) -> bool {
    match node.kind.as_str() {
        KIND_LITERAL => true,
        KIND_IDENT => candidates.contains(&node.name),
        "binary" | "paren" => node
            .children
            .iter()
            .all(|child| built_from(child, candidates)),
        _ => false,
    }
}

/// The unit's int constants this expression names.
fn names_in(node: &Declaration, candidates: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if node.kind == KIND_IDENT && candidates.contains(&node.name) {
        out.insert(node.name.clone());
    }
    for child in &node.children {
        out.extend(names_in(child, candidates));
    }
    out
}

/// Count every read of a candidate, and how many of those are compared against a length.
fn count_reads(
    node: &Declaration,
    candidates: &BTreeSet<String>,
    lengths: &BTreeSet<String>,
    renders: &BTreeSet<String>,
    takes_length: &BTreeSet<String>,
    into: &mut BTreeMap<String, (usize, usize)>,
) {
    // A read PASSED AS a length is neutral, for the same reason a rendered one is: it observes the
    // value and says nothing about its type. Passing a constant as an allocation's size is treating
    // it as a length -- the opposite of evidence against -- and counting it against was enough on
    // its own to keep `xxhash`'s marshaled size signed, though the unit also compares it to one.
    if node.kind == KIND_CALL
        && node
            .attr(ATTR_CALLEE)
            .is_some_and(|callee| takes_length.contains(callee))
    {
        for child in &node.children {
            count_reads(
                child,
                &BTreeSet::new(),
                lengths,
                renders,
                takes_length,
                into,
            );
        }
        return;
    }
    // A read that only RENDERS the value is NEUTRAL: it neither proves the constant is a length
    // nor disproves it. Formatting a bound into a message reads its value and not its type, and
    // the same non-negative literal renders identically whichever integer it is — so counting it
    // as evidence against was rejecting exactly the constants this rule exists for. Every one of
    // `semver`'s bounds is compared against a length once and named in the message that reports
    // the breach, and a reviewer called the resulting signed type the most consequential finding
    // in the file three separate times before this could see why.
    if node.kind == KIND_CALL
        && node
            .attr(ATTR_CALLEE)
            .is_some_and(|callee| renders.contains(callee))
    {
        // The TEMPLATE and the values are all rendered, so none of this subtree is evidence —
        // except a nested call, whose own arguments are its own business. Descending with the
        // candidate set emptied says exactly that: nothing here counts as a read.
        for child in &node.children {
            count_reads(
                child,
                &BTreeSet::new(),
                lengths,
                renders,
                takes_length,
                into,
            );
        }
        return;
    }
    // LENGTH ARITHMETIC is neutral, for the third time in the same family and for the same reason:
    // it observes the value and says nothing about its type. A length times a count is a length, and
    // a length plus a number is a length — so `byteLength * len(ksuids)` and `1 + byteLength` are
    // not evidence that the signed value is observed. Counting them against was enough on its own to
    // keep `ksuid`'s byte length signed, though the unit compares it to a length as well.
    if is_length_arithmetic(node, candidates, lengths) {
        for child in &node.children {
            count_reads(
                child,
                &BTreeSet::new(),
                lengths,
                renders,
                takes_length,
                into,
            );
        }
        return;
    }
    if node.kind == KIND_IDENT
        && node.attr(crate::vocabulary::ATTR_REF) == Some(REF_CONST)
        && candidates.contains(&node.name)
    {
        into.entry(node.name.clone()).or_default().0 += 1;
    }
    // A SWITCH ON A LENGTH makes every case label a comparison against one. `switch len(b) { case
    // byteLength: ... }` is the same guard as `len(b) == byteLength` written as a table, and the
    // rule saw nothing because a case label is not a binary node. That single shape is why `ksuid`'s
    // byte length stayed signed after the bound rule below fixed its neighbour.
    if node.kind == "switch"
        && node
            .children
            .iter()
            .find(|child| child.kind == "tag")
            .and_then(|tag| tag.children.first())
            .is_some_and(|tagged| is_length(tagged, lengths))
    {
        for label in node
            .children
            .iter()
            .filter(|child| child.kind == "case")
            .flat_map(|case| case.children.iter())
            .filter(|part| part.kind == "patterns")
            .flat_map(|patterns| patterns.children.iter())
        {
            if label.kind == KIND_IDENT && candidates.contains(&label.name) {
                into.entry(label.name.clone()).or_default().1 += 1;
            }
        }
    }
    // An INDEX or a SLICE BOUND is positive evidence, and of a stronger kind than a comparison:
    // a comparison says the value is measured against a length, while this says the value IS one —
    // the position it sits in indexes a sequence, and in the target that position has exactly one
    // type. `ksuid` proves the point: its byte length and timestamp length are never compared to
    // anything, only sliced with, so the comparison rule had no evidence and left them signed, and
    // every use then carried a cast the source never wrote.
    //
    // The BASE is not a bound and is skipped: `xs[n]` says nothing about `xs`.
    if matches!(node.kind.as_str(), "index" | "slice") {
        for bound in node.children.iter().skip(1) {
            if bound.kind == KIND_IDENT && candidates.contains(&bound.name) {
                into.entry(bound.name.clone()).or_default().1 += 1;
            }
        }
    }
    // A COMPARISON with a candidate on one side and a length on the other. Both orders, because
    // `n > max` and `max < n` are the same guard written by different people.
    if node.kind == "binary"
        && let [left, right] = node.children.as_slice()
    {
        for (candidate, other) in [(left, right), (right, left)] {
            if candidate.kind == KIND_IDENT
                && candidates.contains(&candidate.name)
                && is_length(other, lengths)
            {
                into.entry(candidate.name.clone()).or_default().1 += 1;
            }
        }
    }
    for child in &node.children {
        count_reads(child, candidates, lengths, renders, takes_length, into);
    }
}

/// Whether this operand is a call the pack declares yields a length.
fn is_length(operand: &Declaration, lengths: &BTreeSet<String>) -> bool {
    operand.kind == KIND_CALL
        && operand
            .attr(ATTR_CALLEE)
            .is_some_and(|callee| lengths.contains(callee))
}

/// Whether this node is ARITHMETIC over things that are all lengths.
///
/// A length times a count is a length; a length plus a number is a length. What makes this safe to
/// treat as neutral rather than as evidence either way is that every operand has to be one of three
/// things the source already types as a length or a plain number — a candidate, a call whose value
/// is a length, or a literal. One operand that is none of those and the whole expression is
/// something else, and the read inside it counts.
///
/// Comparison operators are deliberately absent: a comparison against a length is POSITIVE evidence
/// and is counted as such above, and folding it in here would lose the only proof the rule has.
fn is_length_arithmetic(
    node: &Declaration,
    candidates: &BTreeSet<String>,
    lengths: &BTreeSet<String>,
) -> bool {
    if node.kind != "binary" {
        return false;
    }
    let arithmetic = matches!(
        node.attr(crate::vocabulary::ATTR_OP),
        Some("+" | "-" | "*" | "/" | "%")
    );
    arithmetic
        && node
            .children
            .iter()
            .all(|operand| is_lengthlike(operand, candidates, lengths))
}

/// Whether this operand is a length, a plain number, or arithmetic over them.
fn is_lengthlike(
    operand: &Declaration,
    candidates: &BTreeSet<String>,
    lengths: &BTreeSet<String>,
) -> bool {
    if operand.kind == KIND_LITERAL {
        return true;
    }
    if operand.kind == KIND_IDENT && candidates.contains(&operand.name) {
        return true;
    }
    if is_length(operand, lengths) {
        return true;
    }
    is_length_arithmetic(operand, candidates, lengths)
}

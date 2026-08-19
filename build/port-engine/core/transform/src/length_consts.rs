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
        count_reads(declaration, &candidates, lengths, renders, takes_length, &mut reads);
    }
    reads
        .into_iter()
        .filter(|(_, (total, against_length))| *total > 0 && total == against_length)
        .map(|(name, _)| name)
        .collect()
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
            count_reads(child, &BTreeSet::new(), lengths, renders, takes_length, into);
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
        && node.attr(ATTR_CALLEE).is_some_and(|callee| renders.contains(callee))
    {
        // The TEMPLATE and the values are all rendered, so none of this subtree is evidence —
        // except a nested call, whose own arguments are its own business. Descending with the
        // candidate set emptied says exactly that: nothing here counts as a read.
        for child in &node.children {
            count_reads(child, &BTreeSet::new(), lengths, renders, takes_length, into);
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
            count_reads(child, &BTreeSet::new(), lengths, renders, takes_length, into);
        }
        return;
    }
    if node.kind == KIND_IDENT
        && node.attr(crate::vocabulary::ATTR_REF) == Some(REF_CONST)
        && candidates.contains(&node.name)
    {
        into.entry(node.name.clone()).or_default().0 += 1;
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

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

use crate::vocabulary::{ATTR_CALLEE, KIND_CALL, KIND_IDENT, REF_CONST, SOURCE_INT};

/// Every constant of this unit that is a length.
pub(crate) fn length_constants(
    declarations: &[Declaration],
    lengths: &BTreeSet<String>,
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
        count_reads(declaration, &candidates, lengths, &mut reads);
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
    into: &mut BTreeMap<String, (usize, usize)>,
) {
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
        count_reads(child, candidates, lengths, into);
    }
}

/// Whether this operand is a call the pack declares yields a length.
fn is_length(operand: &Declaration, lengths: &BTreeSet<String>) -> bool {
    operand.kind == KIND_CALL
        && operand
            .attr(ATTR_CALLEE)
            .is_some_and(|callee| lengths.contains(callee))
}

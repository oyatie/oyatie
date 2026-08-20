//! Wire shapes for what a source VALUE becomes, lifted into the port's own types.
//!
//! Split from the main loader for one reason: it had grown past the length this repository allows a
//! file to be, and these five entries are the group that comes out cleanly -- each is a total
//! function from one optional wire rule to the decision it carries, with no dependence on any other
//! part of the pack. A pack that omits one gets the default, which every consumer reads as "the
//! pack does not answer for this" rather than as an answer.

use port_engine_api::{
    Allocation, BinaryString, BitPatternConstants, FormatCalls, FormatFunction, SequenceAppend,
};

use crate::rule_format::{
    AllocationRule, BinaryStringRule, BitPatternConstantsRule, FormatCallsRule, SequenceAppendRule,
};

/// How a sequence GROWS.
///
/// The source spells appending as an assignment -- `x = append(x, v)` -- and the target spells it as
/// a mutation, so the two forms differ in what they say about the name on the left. The pack names
/// both target forms because appending one element and appending many are different methods.
///
/// The default stands for "the pack does not answer for this", which every consumer reads as a
/// reason to leave the source's form alone rather than as an answer.
pub(crate) fn sequence_append(rule: Option<SequenceAppendRule>) -> SequenceAppend {
    rule.map(|rule| SequenceAppend {
        extend: rule.extend,
        push: rule.push,
        reason: rule.reason,
    })
    .unwrap_or_default()
}

/// What the source's allocating builtin becomes.
///
/// Two answers, not one, and the difference is the whole point: `make([]T, 0, n)` reserves room for
/// `n` and holds nothing, while `make([]T, n)` holds `n` zeroed elements. They are one spelling in
/// the source and two entirely different values, and a pack that answered with a single form would
/// make the shorter one silently allocate the longer one's contents.
///
/// The default stands for "the pack does not answer for this", which every consumer reads as a
/// reason to leave the source's form alone rather than as an answer.
pub(crate) fn allocation(rule: Option<AllocationRule>) -> Allocation {
    rule.map(|rule| Allocation {
        empty_with_capacity: rule.empty_with_capacity,
        empty_with_capacity_reason: rule.empty_with_capacity_reason,
        filled: rule.filled,
        filled_reason: rule.filled_reason,
        owned_from_slice: rule.owned_from_slice,
        owned_from_slice_reason: rule.owned_from_slice_reason,
        reason: rule.reason,
    })
    .unwrap_or_default()
}

/// What a source STRING becomes when its content is not text.
///
/// The source's string is a byte string and the target's is guaranteed UTF-8, so the ordinary
/// mapping holds only for the ones that hold text. A framing prefix typed as text is one escape
/// away from a wire-format break, which is why this is a decision the pack makes rather than a
/// default the loader supplies.
///
/// The default stands for "the pack does not answer for this", which every consumer reads as a
/// reason to leave the source's form alone rather than as an answer.
pub(crate) fn binary_string(rule: Option<BinaryStringRule>) -> BinaryString {
    rule.map(|rule| BinaryString {
        target_type: rule.target_type,
        literal_form: rule.literal_form,
        reason: rule.reason,
    })
    .unwrap_or_default()
}

/// Which integer constants the target spells as BIT PATTERNS.
///
/// A count and a mask wear the same syntax and are read differently: a count is a quantity and
/// belongs in decimal, a multiplier or seed is read as its bits and belongs in hexadecimal, where a
/// reviewer can check it against whatever specification defines it. The TYPE decides, because
/// magnitude cannot -- the corpus holds constants above the 32-bit line of both kinds.
///
/// The default stands for "the pack does not answer for this", which every consumer reads as a
/// reason to leave the source's form alone rather than as an answer.
pub(crate) fn bit_pattern_constants(rule: Option<BitPatternConstantsRule>) -> BitPatternConstants {
    rule.map(|rule| BitPatternConstants {
        widths: rule.widths,
        min_value: rule.min_value,
        reason: rule.reason,
    })
    .unwrap_or_default()
}

/// How a formatted string is built, and which of the source's verbs survive.
///
/// The only entry here that is not a single decision: it carries the target macro, the callees that
/// reach it, the verbs the pack can translate, and the reason attached to each. Verbs are enumerated
/// rather than passed through because a verb the target has no equivalent for must refuse -- a
/// format string that renders differently is exactly the silent divergence this engine exists to
/// prevent.
///
/// The default stands for "the pack does not answer for this", which every consumer reads as a
/// reason to leave the source's form alone rather than as an answer.
pub(crate) fn format_calls(rule: Option<FormatCallsRule>) -> FormatCalls {
    rule.map(|rule| FormatCalls {
        macro_name: rule.r#macro,
        macro_reason: rule.macro_reason,
        functions: rule
            .functions
            .into_iter()
            .map(|(identity, entry)| {
                (
                    identity,
                    FormatFunction {
                        wrapper: entry.wrapper,
                        reason: entry.reason,
                    },
                )
            })
            .collect(),
        wrapper_reason: rule.wrapper_reason,
        verbs: rule.verbs,
        verbs_reason: rule.verbs_reason,
        wrap_verb: rule.wrap_verb,
        wrap_verb_reason: rule.wrap_verb_reason,
        literal_only_reason: rule.literal_only_reason,
        brace_reason: rule.brace_reason,
    })
    .unwrap_or_default()
}

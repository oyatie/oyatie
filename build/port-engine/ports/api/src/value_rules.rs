//! What a source VALUE becomes, as pack data.
//!
//! Grouped apart from the conventions that govern a whole unit because these answer a different
//! question: not how the target is organised, but what one expression or one literal turns into.
//! Each is a DECISION the source cannot make for the target -- a byte string is not a UTF-8 one, a
//! bit pattern is not a count, a growing sequence is not an assignment -- and each carries the
//! reason it was decided that way, so the answer and its justification cannot drift apart.

use std::collections::BTreeMap;

/// How the source's `append` becomes the target's.
///
/// A STATEMENT rule, not an expression one: the source's returns a new sequence and the target's
/// mutates in place and returns nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SequenceAppend {
    /// Adding the ELEMENTS of `{1}` to `{0}`.
    pub extend: String, // data_class: INTERNAL_ONLY
    /// Adding `{1}` itself to `{0}`.
    pub push: String, // data_class: INTERNAL_ONLY
    /// Why only the same-name shape carries across, and what the spread decides.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// How the source's ALLOCATING builtin becomes the target's.
///
/// Its own shape rather than a row in the function map because the builtin's first argument is a
/// TYPE and its meaning changes with its arity — neither of which a form keyed by callee identity
/// can express.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Allocation {
    /// A sequence with no elements and room for `{0}`.
    pub empty_with_capacity: String, // data_class: INTERNAL_ONLY
    /// Why the zero length is required rather than incidental.
    pub empty_with_capacity_reason: String, // data_class: INTERNAL_ONLY
    /// A sequence of `{0}` elements, each the zero `{1}`.
    pub filled: String, // data_class: INTERNAL_ONLY
    /// Why an unknown zero refuses.
    pub filled_reason: String, // data_class: INTERNAL_ONLY
    /// Why the map and channel shapes are absent.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// What a source STRING becomes when its content is not text.
///
/// The source's string is a byte string and the target's is guaranteed UTF-8, so the ordinary
/// mapping is right only for the ones that hold text. This says what the rest become.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BinaryString {
    /// The target type, with `{0}` for the byte count.
    pub target_type: String, // data_class: INTERNAL_ONLY
    /// The target literal, with `{0}` for the source's own literal spelling.
    pub literal_form: String, // data_class: INTERNAL_ONLY
    /// Why a byte array rather than the target's string, and how binary is recognised.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// Which integer constants the target spells as a BIT PATTERN rather than as a number.
///
/// The two are different kinds of value wearing the same syntax. A count is read as a quantity and
/// belongs in decimal; a mask, a seed, or a multiplier is read as its bits and belongs in
/// hexadecimal, where a reviewer can check it against the specification that defines it. The source
/// writes both in whatever base its author chose, and that choice does not survive the front end —
/// so the target must decide from the TYPE, which is the one place the distinction is recorded.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BitPatternConstants {
    /// Source type names whose values are bit patterns, with the target width of each in bits.
    pub widths: BTreeMap<String, u32>, // data_class: INTERNAL_ONLY
    /// Below this magnitude a value stays decimal, however it is typed.
    pub min_value: u128, // data_class: INTERNAL_ONLY
    /// Why the type decides this and magnitude alone cannot.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// What the pack does with a formatted string, for one source callee.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormatFunction {
    /// The target callee that receives the formatted string. Empty when the string IS the result.
    pub wrapper: String, // data_class: INTERNAL_ONLY
    /// Why this call becomes this form, and what it does not preserve.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// How the pack answers for a call that FORMATS: a template plus arguments.
///
/// Separate from [`FunctionMapping`] because the mechanism is different, not just the spelling. A
/// mapped call substitutes rendered arguments into a template the pack wrote; a formatted call has
/// to read the SOURCE's template, translate every verb in it, and check that what is left means the
/// same thing. A text substitution cannot do that, and a table of forms cannot express it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FormatCalls {
    /// The target macro that builds a string from a template.
    pub macro_name: String, // data_class: INTERNAL_ONLY
    /// Why that macro is the target's spelling for this operation.
    pub macro_reason: String, // data_class: INTERNAL_ONLY
    /// Callee identity to what RECEIVES the formatted string, empty when the string is the result.
    pub functions: BTreeMap<String, FormatFunction>, // data_class: INTERNAL_ONLY
    /// Why the receiver is a callee path rather than a template with a hole in it.
    pub wrapper_reason: String, // data_class: INTERNAL_ONLY
    /// The CLOSED set of source verbs and the target placeholder each becomes.
    ///
    /// Closed on purpose: a verb absent from here refuses by name. Defaulting an unknown verb to the
    /// plain placeholder would produce a program that compiles and prints something else.
    pub verbs: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    /// Why the verb set is closed, and what the ones left out would have cost.
    pub verbs_reason: String, // data_class: INTERNAL_ONLY
    /// The verb that records a CAUSE rather than rendering anything. Empty if the source has none.
    pub wrap_verb: String, // data_class: INTERNAL_ONLY
    /// Why a wrapping verb cannot become a rendering.
    pub wrap_verb_reason: String, // data_class: INTERNAL_ONLY
    /// Why the template has to be a literal.
    pub literal_only_reason: String, // data_class: INTERNAL_ONLY
    /// Why a literal brace in the template has to be doubled.
    pub brace_reason: String, // data_class: INTERNAL_ONLY
}

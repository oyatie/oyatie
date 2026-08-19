//! Wire shapes for the pack's FORMAT calls.
//!
//! Split from `rule.rs` because a formatting call is answered by a different mechanism than every
//! other call: the pack names what receives the formatted string and which verbs it knows, and the
//! engine reads the SOURCE's own template rather than substituting into one the pack wrote.

use serde::Deserialize;

/// Wire shape for what one source callee does with a formatted string.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FormatFunctionRule {
    #[serde(default)]
    pub(crate) wrapper: String,
    #[serde(default)]
    pub(crate) reason: String,
}

/// Wire shape for the pack's FORMAT calls.
///
/// Its own rule rather than a corner of the function map because the mechanism differs: this one
/// reads the source's own template and translates it, where a function mapping substitutes into a
/// template the pack wrote.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FormatCallsRule {
    #[serde(default)]
    pub(crate) r#macro: String,
    #[serde(default)]
    pub(crate) macro_reason: String,
    #[serde(default)]
    pub(crate) functions: std::collections::BTreeMap<String, FormatFunctionRule>,
    #[serde(default)]
    pub(crate) wrapper_reason: String,
    #[serde(default)]
    pub(crate) verbs: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) verbs_reason: String,
    #[serde(default)]
    pub(crate) wrap_verb: String,
    #[serde(default)]
    pub(crate) wrap_verb_reason: String,
    #[serde(default)]
    pub(crate) literal_only_reason: String,
    #[serde(default)]
    pub(crate) brace_reason: String,
}

/// Wire shape for the library paths the pack names by their short form.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TargetImportsRule {
    #[serde(default)]
    pub(crate) paths: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) reason: String,
}

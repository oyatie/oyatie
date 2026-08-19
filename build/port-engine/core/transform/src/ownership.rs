//! Applying the ownership decision to a receiver or a pointer parameter.
//!
//! The decision itself lives in `port-engine-analysis`; this reads the facts off a declaration,
//! asks for a decision, and records what came back so a reviewer can audit the inference rather
//! than trust it.

use std::cell::RefCell;

use port_engine_analysis::{Decision, decide, receiver_form};
use port_engine_api::{Declaration, OwnershipFacts};

use port_engine_api::PointerConstruction;

use crate::error::TransformError;
use crate::vocabulary::{FLAG_EFFECT_UNKNOWN, FLAG_ESCAPES, FLAG_MUTATED, FLAG_POINTER_RECEIVER};

/// One recorded decision, with the site it was made for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispositionRecord {
    /// Where the decision applies, as `unit::item` or `unit::item(param)`.
    pub site: String,
    /// The rule that matched.
    pub rule_id: String,
    /// The form chosen for this position.
    pub form: String,
    /// Whether the decision rests on facts the front end could not prove.
    pub unproven: bool,
    /// Why these facts deserve this form.
    pub reason: String,
}

/// A per-run log of every ownership decision.
///
/// Collected rather than printed: a disposition is an inference over facts a reader cannot see
/// from the emitted code, so the record has to be an ARTIFACT they can diff. Emitting it as a
/// comment in the output would put it where a rule change is hardest to review — inline, mixed
/// with the translation it explains.
#[derive(Debug, Default)]
pub struct DispositionLog {
    records: RefCell<Vec<DispositionRecord>>,
}

impl DispositionLog {
    /// An empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn record(&self, record: DispositionRecord) {
        self.records.borrow_mut().push(record);
    }

    /// Every decision made, in the order they were made.
    #[must_use]
    pub fn records(&self) -> Vec<DispositionRecord> {
        self.records.borrow().clone()
    }
}

/// Read the ownership facts a front end recorded on a declaration.
pub(crate) fn facts_of(declaration: &Declaration) -> OwnershipFacts {
    OwnershipFacts {
        mutated: declaration.has_flag(FLAG_MUTATED),
        escapes: declaration.has_flag(FLAG_ESCAPES),
        effect_unknown: declaration.has_flag(FLAG_EFFECT_UNKNOWN),
    }
}

/// Whether a method binds its receiver through a pointer.
pub(crate) fn binds_by_pointer(declaration: &Declaration) -> bool {
    declaration.has_flag(FLAG_POINTER_RECEIVER)
}

/// Decide, record, and return the receiver form for a method.
///
/// # Errors
/// [`TransformError::Ownership`] when no rule accepts the facts, or when the matched rule declines
/// the receiver position — which is how "this pointer escapes, so it cannot be a borrow of self"
/// arrives as a refusal rather than as a borrow that does not hold.
pub(crate) fn receiver_for(
    method: &Declaration,
    site: &str,
    context: &OwnershipContext<'_>,
) -> Result<String, TransformError> {
    let facts = facts_of(method);
    let decision = decision_for(site, facts, context)?;
    let form = receiver_form(site, &decision)
        .map_err(|err| TransformError::Ownership {
            detail: err.to_string(),
        })?
        .to_owned();
    context.log.record(DispositionRecord {
        site: site.to_owned(),
        rule_id: decision.rule_id.clone(),
        form: form.clone(),
        unproven: decision.rests_on_unproven_facts(),
        reason: decision.reason.clone(),
    });
    Ok(form)
}

/// Decide, record, and return the target type for a pointer parameter.
///
/// # Errors
/// [`TransformError::Ownership`] when no rule accepts the facts.
pub(crate) fn parameter_target(
    parameter: &Declaration,
    pointee: &str,
    site: &str,
    context: &OwnershipContext<'_>,
) -> Result<String, TransformError> {
    let facts = facts_of(parameter);
    let decision = decision_for(site, facts, context)?;
    let target = decision.target.replace("{0}", pointee);
    context.log.record(DispositionRecord {
        site: site.to_owned(),
        rule_id: decision.rule_id.clone(),
        form: target.clone(),
        unproven: decision.rests_on_unproven_facts(),
        reason: decision.reason.clone(),
    });
    Ok(target)
}

fn decision_for(
    site: &str,
    facts: OwnershipFacts,
    context: &OwnershipContext<'_>,
) -> Result<Decision, TransformError> {
    decide(site, facts, context.rules).map_err(|err| TransformError::Ownership {
        detail: err.to_string(),
    })
}

/// Which disposition was chosen at a site, and how an argument reaches it.
impl OwnershipContext<'_> {
    /// The disposition id chosen at `site`, if one was.
    ///
    /// Read back from the LOG rather than recomputed, so the argument site and the parameter site
    /// are answered by the same decision rather than by two evaluations that could diverge. Linear
    /// in the log, which is affordable because the signature table is built once.
    pub(crate) fn decided_for(&self, site: &str) -> Option<String> {
        self.log
            .records()
            .into_iter()
            .rev()
            .find(|record| record.site == site)
            .map(|record| record.rule_id)
    }

    /// How an argument reaches a parameter holding the disposition `id`.
    pub(crate) fn construction_for(&self, id: &str) -> Option<&PointerConstruction> {
        self.rules
            .iter()
            .find(|rule| rule.id == id)
            .map(|rule| &rule.construction)
    }
}

/// What the ownership pass needs: the pack's rules and somewhere to record what it decided.
pub struct OwnershipContext<'a> {
    pub(crate) rules: &'a [port_engine_api::PointerDisposition],
    pub(crate) log: &'a DispositionLog,
}

impl<'a> OwnershipContext<'a> {
    /// A context over `rules`, logging into `log`.
    #[must_use]
    pub const fn new(
        rules: &'a [port_engine_api::PointerDisposition],
        log: &'a DispositionLog,
    ) -> Self {
        Self { rules, log }
    }
}

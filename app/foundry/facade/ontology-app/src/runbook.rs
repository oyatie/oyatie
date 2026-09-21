//! The operator response that travels with each objective.
//!
//! An objective states when the service is failing its users; a runbook states
//! what to do about it. Keeping them in one IR means a payload consumer has
//! both, with no second artifact to fetch and no link to rot, and means an
//! objective cannot be declared without an answer to "and then what".

/// What an operator needs at 3am, in the order they need it.
pub struct Runbook {
    /// What the operator sees, in their terms rather than the metric's.
    pub symptom: &'static str, // data_class: INTERNAL_ONLY
    /// The one observation that splits the likely causes apart.
    pub first_check: &'static str, // data_class: INTERNAL_ONLY
    /// The action that restores service, which is rarely the action that
    /// explains the failure.
    pub mitigation: &'static str, // data_class: INTERNAL_ONLY
    /// Who owns it once the mitigation has not worked.
    pub escalation: &'static str, // data_class: INTERNAL_ONLY
}

/// A runbook bound to the objective it answers.
pub struct RunbookEntry {
    pub objective: &'static str, // data_class: INTERNAL_ONLY
    pub runbook: Runbook,
}

pub const RUNBOOKS: &[RunbookEntry] = &[
    RunbookEntry {
        objective: "ontology-projection-freshness",
        runbook: Runbook {
            symptom: "Reads return entities that are correct but stale: a write acknowledged by submit is not yet visible to a read.",
            first_check: "GET /statusz and compare the log head against the applied ordinal. A gap that is growing is a stalled fold; a gap that is constant and small is normal catch-up.",
            mitigation: "A stalled fold is usually a poisoned entry the fold will not pass. /statusz names the ordinal an operator starts from; rebuild the projection from the durable log rather than editing the entry.",
            escalation: "foundry, with the ordinal from /statusz and whether the gap grew, held, or closed while observed.",
        },
    },
    RunbookEntry {
        objective: "ontology-submit-availability",
        runbook: Runbook {
            symptom: "Writes are refused. Callers see a refusal rather than a timeout, so the surface is answering.",
            first_check: "Separate authorization from storage: a policy denial refuses every caller identically, while a storage fault refuses regardless of identity and is not a conflict.",
            mitigation: "If the authorizer is unreachable the surface denies by default and that is intended; restore the authorizer rather than bypassing it. If the log is unwritable, restore write access to it before retrying.",
            escalation: "foundry, naming which of the two the first check ruled out.",
        },
    },
    RunbookEntry {
        objective: "ontology-read-availability",
        runbook: Runbook {
            symptom: "Reads are refused. A surface refusing everything still answers, so this is not reachability.",
            first_check: "An absent credential, a policy denial and an unusable revision pin are all counted here. Check whether refusals share one caller, one pin, or all callers.",
            mitigation: "A single bad revision pin is the caller's to correct. Refusals across all callers point at the authorizer or at a projection the process could not read; prefer restoring the projection over serving unauthorized reads.",
            escalation: "foundry, with whether refusals were caller-scoped or global.",
        },
    },
    RunbookEntry {
        objective: "ontology-invocation-latency",
        runbook: Runbook {
            symptom: "The surface answers correctly but too slowly; callers may time out before it does.",
            first_check: "Compare read latency against projection freshness. A slow read with a healthy fold is query shape; a slow read with a growing gap is the fold competing for the same store.",
            mitigation: "Query shape is addressed by the caller's filter or page size, not by the service. Fold contention is addressed by letting catch-up finish before restoring full read load.",
            escalation: "foundry, with the slow operation and whether the freshness gap moved with it.",
        },
    },
    RunbookEntry {
        objective: "ontology-denial-trail-completeness",
        runbook: Runbook {
            symptom: "Authorization decisions are being made without a matching trail entry, so a denial cannot later be explained.",
            first_check: "This is an evidence failure, not a serving failure: the surface is healthy and refusing correctly. Confirm whether the trail sink is unreachable or is accepting and dropping.",
            mitigation: "Do not disable the trail to restore throughput. A denial that is not recorded is indistinguishable from one that never happened, and the trail is what makes a refusal auditable.",
            escalation: "foundry and trust-and-safety together, since the gap is a compliance surface rather than an availability one.",
        },
    },
];

/// The runbook answering `objective`, if it is declared.
pub fn runbook_for(objective: &str) -> Option<&'static Runbook> {
    RUNBOOKS
        .iter()
        .find(|entry| entry.objective == objective)
        .map(|entry| &entry.runbook)
}

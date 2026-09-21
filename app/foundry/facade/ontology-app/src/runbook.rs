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
            symptom: "A tenant's durable projection store is behind its log head, so a write the writer accepted is not yet readable.",
            first_check: "Read foundry_projection_lag and foundry_projection_unreadable. The objective scores a tenant zero when its log head or store could not be READ, which is a different fault from one that is merely behind, and only the second is catch-up.",
            mitigation: "A store that is behind is repaired by letting catch-up finish; readiness refuses until it does, so the refusal is the mechanism working. An unreadable store is a storage fault and catch-up will not fix it. Do not chase a poisoned entry here: a poison advances the fold and is excluded from this objective by design.",
            escalation: "foundry, naming which of lag or unreadability the first check found, and whether foundry_projection_contended was sustained.",
        },
    },
    RunbookEntry {
        objective: "ontology-submit-availability",
        runbook: Runbook {
            symptom: "Submissions are refused. The surface is answering, so this is not reachability.",
            first_check: "A policy refusal counts here deliberately, so separate it from a writer fault by scope: policy is identity-dependent and refuses one caller or one tenant, since every grant is scoped to a role and to principal.tenant == resource.tenant. A writer fault refuses regardless of who asks.",
            mitigation: "An identity-scoped refusal is usually a correct denial; confirm the caller's grant before treating it as an outage. A refusal that ignores identity points at the writer or the log it appends to. The policy engine is compiled into this binary, so there is no authorizer to reconnect: a policy bundle rejected at boot fails the process rather than degrading it.",
            escalation: "foundry, with whether refusals were identity-scoped or universal.",
        },
    },
    RunbookEntry {
        objective: "ontology-read-availability",
        runbook: Runbook {
            symptom: "Reads are refused. A surface refusing everything still answers, so this is not reachability.",
            first_check: "Four refusal classes count here: an absent credential, a policy denial, an unusable revision pin, and a log the process could not read. Check whether refusals share one caller, one pin, or all callers.",
            mitigation: "A bad revision pin is the caller's to correct. Refusals across all callers point at a log or projection the process cannot read. Note that /statusz is an authorized read and counts against this ratio, so polling it during an incident makes the number worse rather than better.",
            escalation: "foundry, with whether refusals were caller-scoped, pin-scoped, or global.",
        },
    },
    RunbookEntry {
        objective: "ontology-invocation-latency",
        runbook: Runbook {
            symptom: "Accepted single-Action invocations are answering, but outside the 250 ms budget. This is the WRITE path: credential, policy decision, tenant lock, append and fold.",
            first_check: "Only accepted invocations are timed, so a rise here is not refusals getting slower. Compare against projection freshness: the fold is inside this budget, so a tenant whose store is behind or contended pays for it in this objective.",
            mitigation: "Contention on a tenant lock is relieved by letting an in-flight write finish rather than by retrying into the same lock. A slow fold is the projection store's problem and shows up in freshness first. Migration runs are deliberately not timed against this budget, so a run in progress is not the cause.",
            escalation: "foundry, with whether projection freshness moved with the latency.",
        },
    },
    RunbookEntry {
        objective: "ontology-denial-trail-completeness",
        runbook: Runbook {
            symptom: "The WRITER refused an Action the policy decision point had already allowed, and the denial trail did not hold the record. The caller saw a denial the audit did not.",
            first_check: "This is an evidence failure, not a serving failure. Refusals before the writer are not denials and count against availability instead, so confirm the events are writer refusals before treating this as a trail fault. A submission too malformed to describe is also recorded as not held, with a healthy sink.",
            mitigation: "Do not disable the trail to restore throughput. Under an occurrence budget at 99.99%, a window with fewer than ten thousand writer refusals is breached by a single lost record, which is the intended posture for an audit control rather than a signal to loosen it.",
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

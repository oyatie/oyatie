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
            symptom: "foundry_projection_fresh is 0: a tenant the process could read has its durable projection store behind its log head, a tenant's log head or store could not be read, or no tenant could be observed at all.",
            first_check: "GET /statusz: projection_lag is a store behind its log, unreadable_tenants is a log head or store that could not be read, and contended_tenants is a request in flight rather than a fault.",
            mitigation: "Lag is repaired only by a catch-up, which runs at boot and after a migration run and never in the background, so restart the process; a restart that logs boot refused naming the projection store or its catch-up has named the storage fault. A poisoned entry advances the fold and is not this objective's event.",
            escalation: "foundry, with projection_lag, unreadable_tenants and contended_tenants from /statusz and whether foundry_projection_contended stayed non-zero across scrapes.",
        },
    },
    RunbookEntry {
        objective: "ontology-submit-availability",
        runbook: Runbook {
            symptom: "foundry_action_submit_refused_total is rising against foundry_action_submit_served_total: the single-Action and migration run surfaces are answering, but refusing.",
            first_check: "The refusal body's gate and HTTP status. A 503 with gate log is the action log refusing to be written. A 403 with gate authorization for every caller at once is the policy decision point failing every decision, since any error it returns is a denial; the runtime guard's circuit, open after five consecutive runtime faults such as a decision past its 250 ms deadline or a crash, is one such error. A 403 with gate parameters or admission is the writer refusing one submission, which is the denial-trail objective's event; a 401, 400, 409 or a 403 for one caller is that request's credential, body, idempotency key or grant.",
            mitigation: "A refusal at one caller is that caller's request to fix. The policy engine is compiled into this binary, so a universal authorization refusal has no authorizer to reconnect; the circuit retries the engine after a 30 s cooldown. A restart reloads the compiled-in bundle and re-opens the log, and logs boot refused naming the policy seed or the action log if either is the fault.",
            escalation: "foundry, with the gate and HTTP status of the refusals and whether every caller received them.",
        },
    },
    RunbookEntry {
        objective: "ontology-read-availability",
        runbook: Runbook {
            symptom: "foundry_read_refused_total is rising against foundry_read_served_total. A served /statusz poll counts as a good read, so polling dilutes this ratio; only a refused poll counts against it.",
            first_check: "The refusal body's gate and HTTP status. A 401 credential, a 400 surface for a missing revision pin, a 404 surface for an object the durable projection does not hold and a 409 surface for an unretained revision are the caller's request; a 403 authorization for every caller at once is the policy decision point failing every decision; a 503 with gate store or log is storage this process cannot read.",
            mitigation: "Caller-side refusals are the caller's to correct, and a client probing references that do not exist breaches this objective through the 404 path. A universal authorization refusal is the same compiled-in policy decision point as on submit. A 503 is storage: a restart re-opens the log and store and refuses to boot if it cannot.",
            escalation: "foundry, with the gate and HTTP status of the refusals and whether every caller received them.",
        },
    },
    RunbookEntry {
        objective: "ontology-invocation-latency",
        runbook: Runbook {
            symptom: "foundry_action_invocation_answered_within_250ms_total is falling behind foundry_action_invocation_answered_total: accepted single-Action submissions are answering, but past 250 ms. Only accepted submissions on that surface are timed; migration runs are not.",
            first_check: "contended_tenants on /statusz, read as which request held the tenant lock: the clock starts at handler entry and includes the policy decision, waiting for the tenant lock, the log append and the durable SQLite mirror into the projection store; a migration run holds that lock across its whole fixpoint and catch-up and a history or audit read holds it across a full log replay, so a submit that arrives behind either waits with the clock running.",
            mitigation: "Do not run a migration while latency matters: one run holds the tenant lock for its whole duration. The fold itself is in memory; the durable mirror is a SQLite write inside the budget, and the policy decision's own deadline equals the whole budget. The shared tenant lock is what couples this objective to freshness, and foundry_projection_contended measures it.",
            escalation: "foundry, with whether a migration run was in flight and whether foundry_projection_contended was non-zero over the window.",
        },
    },
    RunbookEntry {
        objective: "ontology-denial-trail-completeness",
        runbook: Runbook {
            symptom: "The WRITER refused an Action the policy decision point had already allowed, and the denial trail did not hold the record. The caller saw a denial the audit did not.",
            first_check: "This is an evidence failure, not a serving failure. Refusals before the writer are not denials and count against availability instead. A submission too malformed to describe is also recorded as not held, with a healthy sink.",
            mitigation: "Do not disable the trail to restore throughput. Under an occurrence budget at 99.99%, a window with fewer than ten thousand writer refusals is breached by a single lost record, which is the intended posture for an audit control rather than a signal to loosen it.",
            escalation: "foundry, since the gap is a compliance surface rather than an availability one.",
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

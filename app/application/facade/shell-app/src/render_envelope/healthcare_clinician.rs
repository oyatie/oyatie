use super::builders::{
    approval, community, fact, message, metric, module, node, s, schedule, suggestion, work,
    workflow,
};
use super::product_activity::product_activity_spine;
use super::types::{AccreditationState, OperatorContext, TenantRenderEnvelope};

pub(super) fn healthcare_clinician_envelope() -> TenantRenderEnvelope {
    TenantRenderEnvelope {
        context: OperatorContext::HealthcareClinician,
        tenant_name: s("Harborview Care Network"),
        role_name: s(OperatorContext::HealthcareClinician.role()),
        tenant_class: s("Accredited healthcare tenant · Clinical role scope"),
        accreditation: AccreditationState {
            label: s("Healthcare accredited"),
            healthcare_enabled: true,
            explanation: s(
                "Clinical modules are included because both tenant and user context carry accredited healthcare state.",
            ),
        },
        server_derivation_note: s(
            "Server-derived envelope: clinician receives care schedule, secure messages, patient-safe tasks, and healthcare workflow templates.",
        ),
        product_activity: product_activity_spine(OperatorContext::HealthcareClinician),
        metrics: vec![
            metric("Care tasks", "11", "4 due before noon"),
            metric("Patient schedule", "7", "Next visit in 18 minutes"),
            metric(
                "Secure messages",
                "6",
                "2 require clinician acknowledgement",
            ),
            metric(
                "Compliance posture",
                "Green",
                "No PHI is present in this contract envelope",
            ),
        ],
        modules: crate::shell_capability_registry::permitted_module_cards(
            OperatorContext::HealthcareClinician,
        ),
        daily_tasks: vec![
            work(
                "Prepare visit room 4",
                "Redacted patient context; no PHI in contract envelope data",
                "High",
            ),
            work(
                "Acknowledge lab follow-up",
                "Secure message requires clinician acknowledgement",
                "High",
            ),
            work(
                "Review discharge checklist",
                "Template draft for care-team review",
                "Medium",
            ),
            work(
                "Update shift handoff note",
                "No production chart mutation",
                "Medium",
            ),
        ],
        schedule: vec![
            schedule(
                "09:10",
                "Care team huddle",
                "Shift priorities and safety notes",
            ),
            schedule(
                "09:40",
                "Visit redacted A",
                "No PHI/PII in contract envelope data",
            ),
            schedule(
                "11:20",
                "Discharge planning",
                "Care workflow template review",
            ),
        ],
        messages: vec![
            message(
                "Charge nurse",
                "Secure messenger",
                "Room 4 checklist is ready for acknowledgement.",
            ),
            message(
                "Care coordinator",
                "Secure messenger",
                "Discharge template needs clinician review.",
            ),
            message(
                "Compliance bot",
                "Notice",
                "Placeholders only; no PHI entered.",
            ),
        ],
        community: vec![
            community(
                "Care team community",
                "Shift huddle thread",
                "Handoff blockers and care-team notes stay placeholder-only",
            ),
            community(
                "Compliance circle",
                "Accredited workflow review",
                "Policy owners discussing discharge checklist language",
            ),
            community(
                "Workflow builders",
                "Care coordination template",
                "Clinicians requested a safer review-before-notify pattern",
            ),
        ],
        approvals: vec![
            approval(
                "Discharge checklist draft",
                "Care coordinator",
                "Clinician review required before workflow use",
            ),
            approval(
                "Shift handoff template",
                "Charge nurse",
                "No production chart write",
            ),
            approval(
                "Care-team message",
                "Secure messenger",
                "Acknowledge locally; not yet wired to a live service",
            ),
        ],
        workflow: workflow(
            "Care coordination handoff",
            "Accredited healthcare workflow template with human review and no PHI in contract envelope data.",
            vec![
                node(
                    "trigger",
                    "Visit status",
                    "Clinical",
                    55,
                    82,
                    "Uses a placeholder visit status object visible only in healthcare envelopes.",
                ),
                node(
                    "handoff",
                    "Draft handoff",
                    "Intelligence",
                    250,
                    82,
                    "Drafts a handoff note from permitted placeholder data.",
                ),
                node(
                    "review",
                    "Clinician review",
                    "Human",
                    445,
                    82,
                    "Requires clinician acknowledgement before any downstream action.",
                ),
                node(
                    "team",
                    "Notify care team",
                    "Message",
                    640,
                    82,
                    "Creates a secure-message draft; sending stays local unless a deployment adapter is configured.",
                ),
            ],
        ),
        ontology: vec![
            fact(
                "CareTask",
                "belongs to visit",
                "Clinical role can see care-task placeholders.",
            ),
            fact(
                "Visit",
                "requires handoff",
                "Healthcare accreditation enables care workflow templates.",
            ),
            fact(
                "SecureMessage",
                "references care team",
                "Visible only in accredited healthcare context.",
            ),
        ],
        intelligence: vec![
            suggestion(
                "Explain the checklist",
                "Summarize why each discharge step matters in plain language.",
                "No PHI/PII; placeholders only.",
            ),
            suggestion(
                "Draft handoff",
                "Create a clinician-reviewable handoff draft from permitted task labels.",
                "User must approve; no chart write.",
            ),
            suggestion(
                "Find missing step",
                "Compare this template to the care-team checklist.",
                "Advisory only.",
            ),
        ],
        omitted_capability_note: s(
            "Non-clinical finance/admin controls are absent from this clinician envelope unless separately permitted.",
        ),
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DemoContext {
    TenantAdmin,
    CorporateOffice,
    HealthcareClinician,
}

impl DemoContext {
    pub const ALL: [Self; 3] = [
        Self::TenantAdmin,
        Self::CorporateOffice,
        Self::HealthcareClinician,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::TenantAdmin => "tenant-admin",
            Self::CorporateOffice => "corporate-office",
            Self::HealthcareClinician => "healthcare-clinician",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::TenantAdmin => "Tenant admin",
            Self::CorporateOffice => "Corporate office",
            Self::HealthcareClinician => "Accredited healthcare",
        }
    }

    pub const fn role(self) -> &'static str {
        match self {
            Self::TenantAdmin => "Cloud owner / tenant admin",
            Self::CorporateOffice => "Accounting + HR operations user",
            Self::HealthcareClinician => "Clinician in accredited healthcare tenant",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|context| context.id() == id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TenantRenderEnvelope {
    pub context: DemoContext,
    pub tenant_name: String,
    pub role_name: String,
    pub tenant_class: String,
    pub accreditation: AccreditationState,
    pub server_derivation_note: String,
    pub metrics: Vec<MetricCard>,
    pub modules: Vec<ModuleCard>,
    pub daily_tasks: Vec<WorkItem>,
    pub schedule: Vec<ScheduleItem>,
    pub messages: Vec<MessageItem>,
    pub community: Vec<CommunityItem>,
    pub approvals: Vec<ApprovalItem>,
    pub workflow: WorkflowPreview,
    pub ontology: Vec<OntologyFact>,
    pub intelligence: Vec<IntelligenceSuggestion>,
    pub omitted_capability_note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccreditationState {
    pub label: String,
    pub healthcare_enabled: bool,
    pub explanation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModuleCard {
    pub name: String,
    pub group: String,
    pub description: String,
    pub action_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkItem {
    pub title: String,
    pub detail: String,
    pub priority: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub time: String,
    pub title: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MessageItem {
    pub from: String,
    pub channel: String,
    pub preview: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommunityItem {
    pub space: String,
    pub topic: String,
    pub activity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalItem {
    pub title: String,
    pub requester: String,
    pub risk_note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowPreview {
    pub name: String,
    pub goal: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub x: i32,
    pub y: i32,
    pub explanation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OntologyFact {
    pub entity: String,
    pub relation: String,
    pub access_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntelligenceSuggestion {
    pub title: String,
    pub body: String,
    pub guardrail: String,
}

#[cfg(any(feature = "ssr", test))]
pub fn server_derived_envelope(context: DemoContext) -> TenantRenderEnvelope {
    #[cfg(feature = "ssr")]
    {
        crate::server_mock_catalog::derive_tenant_render_envelope(context)
    }

    #[cfg(not(feature = "ssr"))]
    {
        permitted_envelope_snapshot(context)
    }
}

#[cfg(any(feature = "ssr", test))]
pub fn permitted_envelope_snapshot(context: DemoContext) -> TenantRenderEnvelope {
    match context {
        DemoContext::TenantAdmin => tenant_admin_envelope(),
        DemoContext::CorporateOffice => corporate_office_envelope(),
        DemoContext::HealthcareClinician => healthcare_clinician_envelope(),
    }
}

#[cfg(any(feature = "ssr", test))]
fn tenant_admin_envelope() -> TenantRenderEnvelope {
    TenantRenderEnvelope {
        context: DemoContext::TenantAdmin,
        tenant_name: s("Northwind Industrial Group"),
        role_name: s(DemoContext::TenantAdmin.role()),
        tenant_class: s("Enterprise tenant · US/EU/KR packs enabled"),
        accreditation: AccreditationState {
            label: s("Healthcare not accredited for this tenant"),
            healthcare_enabled: false,
            explanation: s(
                "Healthcare modules are absent from this render envelope because the tenant lacks accredited healthcare state.",
            ),
        },
        server_derivation_note: s(
            "Server-derived envelope: admin can see tenant posture, cloud controls, approvals, service catalog, and workflow governance only.",
        ),
        metrics: vec![
            metric("Tenant posture", "92%", "7 controls need owner attestation"),
            metric(
                "Monthly cloud run-rate",
                "$48.2k",
                "Forecast is 4% under committed budget",
            ),
            metric(
                "Open approvals",
                "14",
                "3 high-risk changes require admin review",
            ),
            metric(
                "Enabled modules",
                "12",
                "Healthcare omitted by accreditation policy",
            ),
        ],
        modules: vec![
            module(
                "Tenant Admin",
                "Control",
                "Users, roles, packs, residency, module enablement",
                "Review posture",
            ),
            module(
                "Cloud Compute",
                "Cloud",
                "VMs, functions, Kubernetes workloads, and runtime tiers",
                "Open compute",
            ),
            module(
                "Cloud Network",
                "Cloud",
                "VPC, DNS, load balancing, ingress posture",
                "Open network",
            ),
            module(
                "FinOps",
                "Operations",
                "Cost allocation, budgets, sustainability views",
                "Review spend",
            ),
            module(
                "Workflow Studio",
                "No-code",
                "Design approvals and operating workflows safely",
                "Open studio",
            ),
            module(
                "Audit Chain",
                "Trust",
                "Sealed evidence and policy event review",
                "Inspect evidence",
            ),
        ],
        daily_tasks: vec![
            work(
                "Approve production network split",
                "Needs residency and rollback confirmation",
                "High",
            ),
            work(
                "Assign owner for KR pack evidence",
                "Compliance pack has two stale attestations",
                "Medium",
            ),
            work(
                "Review new module request",
                "Accounting team requested payroll workflow templates",
                "Medium",
            ),
        ],
        schedule: vec![
            schedule("09:30", "Cloud spend review", "FinOps + accounting owners"),
            schedule("11:00", "Access recertification", "Quarterly admin control"),
            schedule(
                "15:30",
                "Workflow change board",
                "Review no-code automation drafts",
            ),
        ],
        messages: vec![
            message(
                "Ops bot",
                "Messenger",
                "Kubernetes runtime tier drift detected in cell-us-east-2.",
            ),
            message(
                "Finance lead",
                "Mail",
                "Please approve department budget tags before close.",
            ),
            message(
                "Security reviewer",
                "Messenger",
                "Network split requires audit-chain evidence before promotion.",
            ),
        ],
        community: vec![
            community(
                "Governance council",
                "Network split RFC",
                "3 reviewers discussing rollback evidence",
            ),
            community(
                "FinOps circle",
                "Budget tag playbook",
                "New guidance pinned for department owners",
            ),
            community(
                "Workflow builders",
                "Payroll template request",
                "Corporate HR is asking for a reusable no-code flow",
            ),
        ],
        approvals: vec![
            approval(
                "Network hot split",
                "Platform operations",
                "Requires rollback and residency evidence",
            ),
            approval(
                "Enable payroll workflows",
                "Corporate HR",
                "No backend execution in prototype",
            ),
            approval(
                "Increase compute quota",
                "Factory systems",
                "Budget owner review required",
            ),
        ],
        workflow: workflow(
            "Tenant change approval",
            "No-code approval path for risky cloud and tenant configuration changes.",
            vec![
                node(
                    "intake",
                    "Request intake",
                    "Form",
                    55,
                    82,
                    "Captures a tenant change request and maps it to a ChangeRequest ontology object.",
                ),
                node(
                    "policy",
                    "Policy check",
                    "Guardrail",
                    250,
                    82,
                    "Evaluates residency, role, and accreditation gates before any reviewer sees the change.",
                ),
                node(
                    "approval",
                    "Admin approval",
                    "Human",
                    445,
                    82,
                    "Routes high-risk requests to the tenant admin; this demo never executes the change.",
                ),
                node(
                    "evidence",
                    "Evidence note",
                    "Audit",
                    640,
                    82,
                    "Drafts audit-chain evidence text for review, not production emission.",
                ),
            ],
        ),
        ontology: vec![
            fact(
                "Tenant",
                "owns enabled module set",
                "Admin role can inspect module posture.",
            ),
            fact(
                "ChangeRequest",
                "requires approval",
                "High-risk changes route to authorized reviewers.",
            ),
            fact(
                "Budget",
                "constrains cloud action",
                "FinOps module is visible to tenant admins.",
            ),
        ],
        intelligence: vec![
            suggestion(
                "Explain the risk",
                "Summarize why this network split needs rollback evidence.",
                "Advisory only; no change execution.",
            ),
            suggestion(
                "Draft approval note",
                "Create a plain-English reviewer note for non-technical owners.",
                "User must review before saving.",
            ),
            suggestion(
                "Find owner",
                "Suggest likely control owners from visible tenant metadata.",
                "Uses permitted envelope data only.",
            ),
        ],
        omitted_capability_note: s(
            "Healthcare and patient-care modules are not present in this tenant admin envelope; they are not hidden client-side.",
        ),
    }
}

#[cfg(any(feature = "ssr", test))]
fn corporate_office_envelope() -> TenantRenderEnvelope {
    TenantRenderEnvelope {
        context: DemoContext::CorporateOffice,
        tenant_name: s("Northwind Industrial Group"),
        role_name: s(DemoContext::CorporateOffice.role()),
        tenant_class: s("Corporate office role · Accounting + HR module scope"),
        accreditation: AccreditationState {
            label: s("Healthcare not available for this role"),
            healthcare_enabled: false,
            explanation: s(
                "This employee sees corporate work modules, not factory controls or healthcare surfaces.",
            ),
        },
        server_derivation_note: s(
            "Server-derived envelope: same corporate tenant, but the role receives daily work, approvals, mail, messenger, HR, and accounting modules.",
        ),
        metrics: vec![
            metric(
                "Today’s work",
                "9",
                "Tasks across accounting, HR, and approvals",
            ),
            metric(
                "Pending approvals",
                "5",
                "2 expense exceptions need manager review",
            ),
            metric("Unread work mail", "18", "4 tagged as finance close"),
            metric(
                "Scheduled focus",
                "3.5h",
                "Calendar protected around payroll close",
            ),
        ],
        modules: vec![
            module(
                "Work Home",
                "Daily",
                "Tasks, calendar, mail, messenger, and approvals",
                "Open home",
            ),
            module(
                "Accounting",
                "Corporate",
                "Invoices, close tasks, budgets, and exceptions",
                "Review close",
            ),
            module(
                "Human Resources",
                "Corporate",
                "Onboarding, policy acknowledgements, and payroll workflows",
                "Open HR",
            ),
            module(
                "Approvals",
                "Workflow",
                "Plain-language approvals with policy context",
                "Review queue",
            ),
            module(
                "Workflow Studio",
                "No-code",
                "Draft team workflows from templates",
                "Draft workflow",
            ),
        ],
        daily_tasks: vec![
            work(
                "Approve travel exception",
                "Policy allows manager review under $2,500",
                "High",
            ),
            work(
                "Send payroll close reminder",
                "Draft is ready for HR review",
                "Medium",
            ),
            work(
                "Reconcile vendor invoice",
                "Accounting module matched 2 of 3 line items",
                "Medium",
            ),
            work(
                "Acknowledge updated leave policy",
                "Due today for all office roles",
                "Low",
            ),
        ],
        schedule: vec![
            schedule("08:45", "Finance close standup", "Accounting team"),
            schedule("10:30", "New hire onboarding", "HR + manager"),
            schedule("14:00", "Approvals block", "Expense and payroll exceptions"),
        ],
        messages: vec![
            message(
                "Payroll bot",
                "Messenger",
                "Three employees still need bank-detail confirmation.",
            ),
            message(
                "Vendor AP",
                "Mail",
                "Invoice NW-4421 has a tax-code mismatch.",
            ),
            message(
                "HR partner",
                "Messenger",
                "Can you review the onboarding workflow draft?",
            ),
        ],
        community: vec![
            community(
                "Accounting community",
                "Month-end close room",
                "AP and finance leads coordinating exception owners",
            ),
            community(
                "HR policy circle",
                "Leave policy rollout",
                "Managers asked for a plain-language acknowledgement flow",
            ),
            community(
                "Office announcements",
                "New hire cohort",
                "Onboarding mentors sharing first-week checklists",
            ),
        ],
        approvals: vec![
            approval(
                "Travel exception",
                "Sales manager",
                "Over soft policy limit; manager can approve",
            ),
            approval(
                "Vendor tax-code fix",
                "Accounts payable",
                "Accounting-only scope",
            ),
            approval(
                "Onboarding checklist",
                "HR partner",
                "Template draft, no production execution",
            ),
        ],
        workflow: workflow(
            "Onboarding checklist",
            "No-code workflow for corporate office onboarding with approvals and reminders.",
            vec![
                node(
                    "invite",
                    "Invite employee",
                    "Mail",
                    55,
                    82,
                    "Creates a draft work-mail message from HR-approved template text.",
                ),
                node(
                    "tasks",
                    "Assign tasks",
                    "Task",
                    250,
                    82,
                    "Creates visible checklist items in the permitted employee scope.",
                ),
                node(
                    "manager",
                    "Manager review",
                    "Approval",
                    445,
                    82,
                    "Routes exceptions to the manager instead of auto-approving.",
                ),
                node(
                    "summary",
                    "Close summary",
                    "Intelligence",
                    640,
                    82,
                    "Drafts a summary for HR; user review required.",
                ),
            ],
        ),
        ontology: vec![
            fact(
                "Employee",
                "has onboarding tasks",
                "HR/accounting role can see permitted employee workflow state.",
            ),
            fact(
                "Invoice",
                "maps to approval",
                "Accounting module exposes invoice exceptions.",
            ),
            fact(
                "Message",
                "can reference workflow",
                "Messenger/mail surfaces are visible for this role.",
            ),
        ],
        intelligence: vec![
            suggestion(
                "Turn policy into steps",
                "Convert the leave-policy update into a checklist for affected teams.",
                "Draft only; HR approves before sending.",
            ),
            suggestion(
                "Explain invoice mismatch",
                "Summarize the tax-code mismatch without exposing unrelated vendor data.",
                "Uses accounting envelope only.",
            ),
            suggestion(
                "Prioritize my day",
                "Group mail, tasks, and approvals into a close-friendly order.",
                "No state mutation.",
            ),
        ],
        omitted_capability_note: s(
            "Factory controls and healthcare modules are absent from this role-shaped envelope.",
        ),
    }
}

#[cfg(any(feature = "ssr", test))]
fn healthcare_clinician_envelope() -> TenantRenderEnvelope {
    TenantRenderEnvelope {
        context: DemoContext::HealthcareClinician,
        tenant_name: s("Harborview Care Network"),
        role_name: s(DemoContext::HealthcareClinician.role()),
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
                "No PHI is present in this prototype",
            ),
        ],
        modules: vec![
            module(
                "Clinical Home",
                "Healthcare",
                "Care tasks, visits, and secure team messages",
                "Open home",
            ),
            module(
                "Patient Schedule",
                "Healthcare",
                "Visit flow with compliance-safe placeholders",
                "Review schedule",
            ),
            module(
                "Care Workflows",
                "Healthcare",
                "Accredited workflow templates for care coordination",
                "Open workflows",
            ),
            module(
                "Secure Messenger",
                "Healthcare",
                "Team communication with care-context labels",
                "Open messages",
            ),
            module(
                "Workflow Studio",
                "No-code",
                "Draft safe care coordination workflows",
                "Draft care flow",
            ),
        ],
        daily_tasks: vec![
            work(
                "Prepare visit room 4",
                "Placeholder patient context; no PHI in prototype",
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
            schedule("09:40", "Visit placeholder A", "No PHI/PII in demo"),
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
                "Prototype uses placeholders only; no PHI entered.",
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
                "Acknowledge only in mock UI",
            ),
        ],
        workflow: workflow(
            "Care coordination handoff",
            "Accredited healthcare workflow template with human review and no PHI in the prototype.",
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
                    "Creates a secure-message draft; this prototype never sends it.",
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

#[cfg(any(feature = "ssr", test))]
fn workflow(name: &str, goal: &str, nodes: Vec<WorkflowNode>) -> WorkflowPreview {
    let edges = nodes
        .windows(2)
        .map(|pair| WorkflowEdge {
            from: pair[0].id.clone(),
            to: pair[1].id.clone(),
            label: s("then"),
        })
        .collect();

    WorkflowPreview {
        name: s(name),
        goal: s(goal),
        nodes,
        edges,
    }
}

#[cfg(any(feature = "ssr", test))]
fn metric(label: &str, value: &str, detail: &str) -> MetricCard {
    MetricCard {
        label: s(label),
        value: s(value),
        detail: s(detail),
    }
}

#[cfg(any(feature = "ssr", test))]
fn module(name: &str, group: &str, description: &str, action_label: &str) -> ModuleCard {
    ModuleCard {
        name: s(name),
        group: s(group),
        description: s(description),
        action_label: s(action_label),
    }
}

#[cfg(any(feature = "ssr", test))]
fn work(title: &str, detail: &str, priority: &str) -> WorkItem {
    WorkItem {
        title: s(title),
        detail: s(detail),
        priority: s(priority),
    }
}

#[cfg(any(feature = "ssr", test))]
fn schedule(time: &str, title: &str, detail: &str) -> ScheduleItem {
    ScheduleItem {
        time: s(time),
        title: s(title),
        detail: s(detail),
    }
}

#[cfg(any(feature = "ssr", test))]
fn message(from: &str, channel: &str, preview: &str) -> MessageItem {
    MessageItem {
        from: s(from),
        channel: s(channel),
        preview: s(preview),
    }
}

#[cfg(any(feature = "ssr", test))]
fn community(space: &str, topic: &str, activity: &str) -> CommunityItem {
    CommunityItem {
        space: s(space),
        topic: s(topic),
        activity: s(activity),
    }
}

#[cfg(any(feature = "ssr", test))]
fn approval(title: &str, requester: &str, risk_note: &str) -> ApprovalItem {
    ApprovalItem {
        title: s(title),
        requester: s(requester),
        risk_note: s(risk_note),
    }
}

#[cfg(any(feature = "ssr", test))]
fn node(id: &str, label: &str, kind: &str, x: i32, y: i32, explanation: &str) -> WorkflowNode {
    WorkflowNode {
        id: s(id),
        label: s(label),
        kind: s(kind),
        x,
        y,
        explanation: s(explanation),
    }
}

#[cfg(any(feature = "ssr", test))]
fn fact(entity: &str, relation: &str, access_reason: &str) -> OntologyFact {
    OntologyFact {
        entity: s(entity),
        relation: s(relation),
        access_reason: s(access_reason),
    }
}

#[cfg(any(feature = "ssr", test))]
fn suggestion(title: &str, body: &str, guardrail: &str) -> IntelligenceSuggestion {
    IntelligenceSuggestion {
        title: s(title),
        body: s(body),
        guardrail: s(guardrail),
    }
}

#[cfg(any(feature = "ssr", test))]
fn s(value: &str) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::{DemoContext, server_derived_envelope};

    #[test]
    fn healthcare_modules_are_absent_from_unaccredited_contexts() {
        for context in [DemoContext::TenantAdmin, DemoContext::CorporateOffice] {
            let envelope = server_derived_envelope(context);
            let module_names = envelope
                .modules
                .iter()
                .map(|module| module.name.as_str())
                .collect::<Vec<_>>()
                .join("|");

            assert!(!envelope.accreditation.healthcare_enabled);
            assert!(!module_names.contains("Patient"));
            assert!(!module_names.contains("Clinical"));
            assert!(!module_names.contains("Care Workflows"));
        }
    }

    #[test]
    fn accredited_healthcare_context_receives_care_modules() {
        let envelope = server_derived_envelope(DemoContext::HealthcareClinician);
        let module_names = envelope
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>()
            .join("|");

        assert!(envelope.accreditation.healthcare_enabled);
        assert!(module_names.contains("Clinical Home"));
        assert!(module_names.contains("Patient Schedule"));
        assert!(module_names.contains("Care Workflows"));
    }

    #[test]
    fn every_context_has_daily_dashboard_primitives() {
        for context in DemoContext::ALL {
            let envelope = server_derived_envelope(context);

            assert!(!envelope.daily_tasks.is_empty(), "{context:?} tasks");
            assert!(!envelope.schedule.is_empty(), "{context:?} schedule");
            assert!(!envelope.messages.is_empty(), "{context:?} messages");
            assert!(!envelope.community.is_empty(), "{context:?} community");
            assert!(!envelope.approvals.is_empty(), "{context:?} approvals");
            assert!(!envelope.modules.is_empty(), "{context:?} modules");
            assert!(
                envelope.workflow.nodes.len() >= 4,
                "{context:?} workflow nodes"
            );
        }
    }

    #[test]
    fn permitted_envelope_snapshot_is_local_and_cloneable_for_island_state() {
        let envelope = server_derived_envelope(DemoContext::TenantAdmin);
        let cloned_for_island = envelope.clone();

        assert_eq!(cloned_for_island.context, DemoContext::TenantAdmin);
        assert_eq!(cloned_for_island.modules, envelope.modules);
        assert_eq!(cloned_for_island.workflow.nodes, envelope.workflow.nodes);
    }
}

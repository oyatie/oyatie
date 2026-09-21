use super::builders::{
    approval, community, fact, message, metric, module, node, s, schedule, suggestion, work,
    workflow,
};
use super::product_activity::product_activity_spine;
use super::types::{AccreditationState, OperatorContext, TenantRenderEnvelope};

pub(super) fn corporate_office_envelope() -> TenantRenderEnvelope {
    TenantRenderEnvelope {
        context: OperatorContext::CorporateOffice,
        tenant_name: s("Northwind Industrial Group"),
        role_name: s(OperatorContext::CorporateOffice.role()),
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
        product_activity: product_activity_spine(OperatorContext::CorporateOffice),
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
        modules: crate::shell_capability_registry::permitted_module_cards(
            OperatorContext::CorporateOffice,
        ),
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
            "Factory controls and healthcare-regulated surfaces are absent from this role-shaped envelope.",
        ),
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorContext {
    TenantAdmin,
    CorporateOffice,
    HealthcareClinician,
}

impl OperatorContext {
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
    pub context: OperatorContext,
    pub tenant_name: String,
    pub role_name: String,
    pub tenant_class: String,
    pub accreditation: AccreditationState,
    pub server_derivation_note: String,
    pub product_activity: ProductActivitySpine,
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
pub struct ProductActivitySpine {
    pub active_route: String,
    pub active_context: String,
    pub status_label: String,
    pub evidence_id: String,
    pub steps: Vec<ProductActivityStep>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProductActivityStep {
    pub route_key: String,
    pub label: String,
    pub surface: String,
    pub detail: String,
    pub target: String,
    pub state: String,
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
pub fn server_derived_envelope(context: OperatorContext) -> TenantRenderEnvelope {
    permitted_envelope_snapshot(context)
}

#[cfg(any(feature = "ssr", test))]
pub fn permitted_envelope_snapshot(context: OperatorContext) -> TenantRenderEnvelope {
    match context {
        OperatorContext::TenantAdmin => tenant_admin_envelope(),
        OperatorContext::CorporateOffice => corporate_office_envelope(),
        OperatorContext::HealthcareClinician => healthcare_clinician_envelope(),
    }
}

#[cfg(any(feature = "ssr", test))]
fn tenant_admin_envelope() -> TenantRenderEnvelope {
    TenantRenderEnvelope {
        context: OperatorContext::TenantAdmin,
        tenant_name: s("Northwind Industrial Group"),
        role_name: s(OperatorContext::TenantAdmin.role()),
        tenant_class: s("Enterprise tenant · US/EU/KR packs enabled"),
        accreditation: AccreditationState {
            label: s("Healthcare not accredited for this tenant"),
            healthcare_enabled: false,
            explanation: s(
                "Healthcare-regulated surfaces are absent from this render envelope because the tenant lacks accredited healthcare state.",
            ),
        },
        server_derivation_note: s(
            "Server-derived envelope: admin can see tenant posture, cloud controls, approvals, service catalog, and workflow governance only.",
        ),
        product_activity: product_activity_spine(OperatorContext::TenantAdmin),
        metrics: vec![
            metric("Close progress", "73%", "+12 vs Mar · payroll run"),
            metric("Cycle time", "5.4d", "+1.4d vs target"),
            metric("Cost of delay", "₩2,180,000", "+₩410k"),
            metric("Open approvals", "8", "2 overdue"),
            metric("Compliance", "4/6", "2 review"),
        ],
        modules: crate::shell_capability_registry::permitted_module_cards(
            OperatorContext::TenantAdmin,
        ),
        daily_tasks: vec![
            work(
                "2026-04 급여 마감 — 박서준 직원 4대보험 변동 확인 필요",
                "건강보험료 등급 상승(변동액 +₩47,200/월). 마감 전 확인 후 승인.",
                "High",
            ),
            work(
                "연차 사용 승인 — 김지영 (5/13 – 5/17, 5일)",
                "$60 잔여 13.5일 → 8.5일. 팀 백업 확인됨.",
                "Medium",
            ),
            work(
                "원천징수이행상황신고서 — 2026-04 제출 준비 완료",
                "118명 직원 검증 완료. 홈택스 전송 대기.",
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
                "구매 승인 경로 단축 가능 — Stripe 청구서 (₩4,820,000)",
                "Procurement",
                "정책상 < ₩5M는 1단계 승인 가능. 현재 3단계로 라우팅 중.",
            ),
            approval(
                "Payroll workflow template request",
                "Corporate HR",
                "No production execution; draft routes to Mail and Community.",
            ),
            approval(
                "Increase compute quota",
                "Factory systems",
                "Budget owner review required before runtime tier change.",
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
                    "Routes high-risk requests to the tenant admin; execution stays disabled until live integration.",
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
                "병목: 공계 승인 단계 (2.1d)",
                "과거 12회 사이클 평균 0.6d. 최유나 매니저에 위임 권한 자동 부여 시 1.4d 단축.",
                "Read-only suggestion; never auto-executes.",
            ),
            suggestion(
                "원천세 신고 자동 제출 가능",
                "118명 검증 완료. 사업자등록번호 1건 확인 후 자동 전송.",
                "User must review before HomeTax send.",
            ),
            suggestion(
                "$53 위험: 윤태민",
                "이번 주 49.5h 예상. 백업 가능 인력 2명 추천.",
                "Uses permitted roster data only.",
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

#[cfg(any(feature = "ssr", test))]
fn healthcare_clinician_envelope() -> TenantRenderEnvelope {
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
fn product_activity_spine(context: OperatorContext) -> ProductActivitySpine {
    let (active_context, status_label) = match context {
        OperatorContext::TenantAdmin => (
            "Tenant admin · Northwind · FD-001 finance close",
            "FD-001 product graph active · Oyatie Cloud cell-us-east-2 · local visual state",
        ),
        OperatorContext::CorporateOffice => (
            "Corporate office · Accounting + HR · FD-001 daily work",
            "Corporate work queue active · Workflow, Mail, Messenger, Community remain tenant workloads",
        ),
        OperatorContext::HealthcareClinician => (
            "Accredited healthcare · Harborview · care workflow preview",
            "Healthcare-safe workflow lens active · no PHI/PII or chart write before live integration",
        ),
    };

    ProductActivitySpine {
        active_route: s("fd001"),
        active_context: s(active_context),
        status_label: s(status_label),
        evidence_id: s("REC-FD001-CLOUD-009"),
        steps: vec![
            activity_step(
                "fd001",
                "FD-001 graph",
                "Product substrate",
                "Canonical product graph, service catalog, and tenant workload coverage.",
                "#service-catalog",
                "active",
            ),
            activity_step(
                "workflow",
                "Workflow",
                "Governed runbook",
                "Payroll close DAG, visual rules, simulation overlays, and inspector state.",
                "#workflow-studio",
                "draft",
            ),
            activity_step(
                "messenger",
                "Messenger",
                "Ops room",
                "Operational thread extracts actions and links rollback evidence.",
                "#work-hub",
                "watch",
            ),
            activity_step(
                "mail",
                "Mail",
                "Formal brief",
                "Approval mail carries receipts, signoff checks, and delivery disabled state.",
                "#work-hub",
                "draft",
            ),
            activity_step(
                "community",
                "Community",
                "Council post",
                "Governance audience, policy moderation, and pinned digest stay visible.",
                "#work-hub",
                "ready",
            ),
            activity_step(
                "cloud",
                "Oyatie Cloud",
                "Tenant cell",
                "Cell topology, deployment gates, FinOps, residency, and rollback posture.",
                "#cloud-ops-cockpit",
                "guarded",
            ),
            activity_step(
                "evidence",
                "Evidence",
                "Audit receipt",
                "Immutable local receipt proves what changed without wiring external systems.",
                "#audit-ledger",
                "sealed",
            ),
        ],
    }
}

#[cfg(any(feature = "ssr", test))]
fn activity_step(
    route_key: &str,
    label: &str,
    surface: &str,
    detail: &str,
    target: &str,
    state: &str,
) -> ProductActivityStep {
    ProductActivityStep {
        route_key: s(route_key),
        label: s(label),
        surface: s(surface),
        detail: s(detail),
        target: s(target),
        state: s(state),
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
    use super::{OperatorContext, server_derived_envelope};

    #[test]
    fn regulated_care_surfaces_are_absent_from_unaccredited_contexts() {
        for context in [
            OperatorContext::TenantAdmin,
            OperatorContext::CorporateOffice,
        ] {
            let envelope = server_derived_envelope(context);
            let surface_names = envelope
                .modules
                .iter()
                .map(|module| module.name.as_str())
                .collect::<Vec<_>>()
                .join("|");

            assert!(!envelope.accreditation.healthcare_enabled);
            assert!(!surface_names.contains("Patient"));
            assert!(!surface_names.contains("Clinical"));
            assert!(!surface_names.contains("Care Workflows"));
        }
    }

    #[test]
    fn accredited_healthcare_context_receives_care_modules() {
        let envelope = server_derived_envelope(OperatorContext::HealthcareClinician);
        let surface_names = envelope
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>()
            .join("|");

        assert!(envelope.accreditation.healthcare_enabled);
        assert!(surface_names.contains("Clinical Home"));
        assert!(surface_names.contains("Patient Schedule"));
        assert!(surface_names.contains("Care Workflows"));
    }

    #[test]
    fn every_context_has_daily_dashboard_primitives() {
        for context in OperatorContext::ALL {
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
        let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
        let cloned_for_island = envelope.clone();

        assert_eq!(cloned_for_island.context, OperatorContext::TenantAdmin);
        assert_eq!(cloned_for_island.modules, envelope.modules);
        assert_eq!(cloned_for_island.workflow.nodes, envelope.workflow.nodes);
    }
}

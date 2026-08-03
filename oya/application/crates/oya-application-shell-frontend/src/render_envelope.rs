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
    pub developer_portal: DeveloperPortalFixture,
    pub support_advisory: Option<SupportAdvisoryFixture>,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeveloperPortalFixture {
    pub catalog_entity: DeveloperCatalogEntity,
    pub approved_template: ApprovedServiceTemplate,
    pub admission_sequence: Vec<String>,
    pub provisioning_request: ProvisioningRequestFixture,
    pub provisioning_operation: ProvisioningOperationFixture,
    pub generated_artifacts: Vec<GeneratedArtifactFixture>,
    pub preview_environment: PreviewEnvironmentFixture,
    pub declared_resources: Vec<String>,
    pub resource_facets: Vec<ResourceFacetFixture>,
    pub facet_contract_fail_closed: bool,
    pub cost_preview: CostPreviewFixture,
    pub policy_denial_fixture: PolicyDenialFixture,
    pub audit_event_fixture: Vec<DeveloperPortalAuditEventFixture>,
    pub role_coverage: Vec<DeveloperPortalRoleCoverage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeveloperCatalogEntity {
    pub kind: String,
    pub service_slug: String,
    pub display_name: String,
    pub source_contract: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovedServiceTemplate {
    pub template_id: String,
    pub display_name: String,
    pub owner: String,
    pub version: TemplateVersionFixture,
    pub supported_golden_path_resources: Vec<String>,
    pub parameters: Vec<TemplateParameterFixture>,
    pub policy_hooks: Vec<String>,
    pub quota_dimensions: Vec<String>,
    pub cost_dimensions: Vec<String>,
    pub artifact_generators: Vec<String>,
    pub rollback_behavior: String,
    pub deletion_behavior: String,
    pub evidence_events: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TemplateVersionFixture {
    pub version: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TemplateParameterFixture {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub default_value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvisioningRequestFixture {
    pub request_id: String,
    pub requested_by: String,
    pub parameters: Vec<TemplateParameterFixture>,
    pub approval_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvisioningOperationFixture {
    pub operation_id: String,
    pub state: String,
    pub idempotency_key: String,
    pub ledger_mutation: String,
    pub reconciler_owned: bool,
    pub evidence_receipt: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneratedArtifactFixture {
    pub artifact_kind: String,
    pub path: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreviewEnvironmentFixture {
    pub environment_id: String,
    pub lifecycle_state: String,
    pub url: String,
    pub rollback_behavior: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceFacetFixture {
    pub resource: String,
    pub lifecycle: String,
    pub identity: String,
    pub policy: String,
    pub quota: String,
    pub billing: String,
    pub audit: String,
    pub observability: String,
    pub rollback: String,
    pub reconciliation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CostPreviewFixture {
    pub currency: String,
    pub monthly_minor_units: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyDenialFixture {
    pub decision: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeveloperPortalAuditEventFixture {
    pub event_type: String,
    pub receipt: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeveloperPortalRoleCoverage {
    pub role: String,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SupportAdvisoryFixture {
    pub case_id: String,
    pub tenant_id: String,
    pub resource_ref: String,
    pub service_ref: String,
    pub severity: String,
    pub response_target_label: String,
    pub entitlement_plan: String,
    pub customer_state: String,
    pub incident_refs: Vec<String>,
    pub status_refs: Vec<String>,
    pub trust_evidence_refs: Vec<String>,
    pub governance_posture_refs: Vec<String>,
    pub finops_refs: Vec<String>,
    pub onboarding_refs: Vec<String>,
    pub audit_chain_refs: Vec<String>,
    pub diagnostic_bundle_ref: String,
    pub diagnostic_bundle_evidence_refs: Vec<String>,
    pub advisor_key: String,
    pub advisor_severity: String,
    pub advisor_owner: String,
    pub recommended_action: String,
    pub non_goal_copy: String,
    pub expires_at: String,
    pub support_access_state: String,
    pub support_engineer_view: String,
    pub post_case_actions: Vec<String>,
    pub api_records: Vec<SupportApiRecordFixture>,
    pub security_assertions: Vec<String>,
    pub authority_boundaries: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SupportApiRecordFixture {
    pub record_type: String,
    pub record_id: String,
    pub tenant_id: String,
    pub actor: String,
    pub purpose: String,
    pub data_class: String,
    pub freshness: String,
    pub redaction_policy_id: String,
    pub audit_event_ref: String,
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
        developer_portal: developer_portal_fixture(),
        support_advisory: Some(support_advisory_fixture()),
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
        developer_portal: developer_portal_fixture(),
        support_advisory: None,
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
                "No PHI is present in this transitional dataset",
            ),
        ],
        modules: crate::shell_capability_registry::permitted_module_cards(
            OperatorContext::HealthcareClinician,
        ),
        daily_tasks: vec![
            work(
                "Prepare visit room 4",
                "Placeholder patient context; no PHI in transitional data",
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
                "Visit placeholder A",
                "No PHI/PII in transitional data",
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
            "Accredited healthcare workflow template with human review and no PHI in transitional data.",
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
                    "Creates a secure-message draft; sending stays local until live service integration.",
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
        developer_portal: developer_portal_fixture(),
        support_advisory: None,
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
fn developer_portal_fixture() -> DeveloperPortalFixture {
    let required_parameters = vec![
        TemplateParameterFixture {
            name: s("service_slug"),
            field_type: s("dns-label"),
            required: true,
            default_value: s("orders-api"),
        },
        TemplateParameterFixture {
            name: s("owning_team"),
            field_type: s("team-ref"),
            required: true,
            default_value: s("axis-developer-experience"),
        },
        TemplateParameterFixture {
            name: s("cell"),
            field_type: s("region-cell"),
            required: true,
            default_value: s("cell-us-east-2"),
        },
    ];

    DeveloperPortalFixture {
        catalog_entity: DeveloperCatalogEntity {
            kind: s("ServiceCatalogEntity"),
            service_slug: s("orders-api"),
            display_name: s("Orders API fixture service"),
            source_contract: s("API-001 fixture snapshot · DEVPORTAL-001A"),
        },
        approved_template: ApprovedServiceTemplate {
            template_id: s("svc-rust-axum-leptos"),
            display_name: s("Rust API + Leptos Shell Service"),
            owner: s("platform engineer"),
            version: TemplateVersionFixture {
                version: s("1.0.0-fixture"),
                status: s("approved"),
            },
            supported_golden_path_resources: vec![
                s("service"),
                s("database"),
                s("topic"),
                s("bucket"),
                s("secret reference"),
                s("SLO"),
                s("runbook"),
                s("deploy pipeline"),
                s("preview env"),
            ],
            parameters: required_parameters.clone(),
            policy_hooks: vec![
                s("template allowlist"),
                s("Cedar identity and tenancy decision"),
                s("security reviewer denial fixture"),
                s("tenant admin quota/budget approval"),
            ],
            quota_dimensions: vec![s("cpu_millis"), s("memory_mib"), s("database_gib")],
            cost_dimensions: vec![s("compute"), s("database"), s("preview environment")],
            artifact_generators: vec![
                s("OpenSLO"),
                s("runbook-live-doc-entry"),
                s("progressive-delivery-policy"),
                s("preview-environment-lifecycle"),
                s("backing-resource-descriptor"),
            ],
            rollback_behavior: s("fixture rollback records compensation intent; no provider apply"),
            deletion_behavior: s(
                "fixture deletion records tombstone intent; reconciler owns live cleanup later",
            ),
            evidence_events: vec![
                s("developer_portal.provisioning.requested"),
                s("developer_portal.policy.denied_fixture"),
                s("developer_portal.operation.accepted_fixture"),
            ],
        },
        admission_sequence: vec![
            s("identity/tenant/project binding"),
            s("template allowlist"),
            s("parameter validation"),
            s("Cedar/policy decision"),
            s("quota reservation and cost preview"),
            s("approval if required"),
            s("idempotent operation ledger mutation"),
            s("reconciler-owned actuation"),
            s("generated artifact registration"),
            s("audit/event emission"),
        ],
        provisioning_request: ProvisioningRequestFixture {
            request_id: s("prq-devportal-001a"),
            requested_by: s("application developer"),
            parameters: required_parameters,
            approval_required: true,
        },
        provisioning_operation: ProvisioningOperationFixture {
            operation_id: s("op-devportal-001a"),
            state: s("accepted_fixture"),
            idempotency_key: s("idem-prq-devportal-001a"),
            ledger_mutation: s("operation ledger fixture row only"),
            reconciler_owned: true,
            evidence_receipt: s("REC-DEVPORTAL-001A"),
        },
        generated_artifacts: vec![
            GeneratedArtifactFixture {
                artifact_kind: s("OpenSLO"),
                path: s("fixtures/devportal/orders-api/slo.openslo.yaml"),
                description: s("SLO stub registered as generated artifact fixture"),
            },
            GeneratedArtifactFixture {
                artifact_kind: s("runbook-live-doc-entry"),
                path: s("fixtures/devportal/orders-api/runbook.md"),
                description: s("live-doc runbook entry fixture"),
            },
            GeneratedArtifactFixture {
                artifact_kind: s("progressive-delivery-policy"),
                path: s("fixtures/devportal/orders-api/progressive-delivery.yaml"),
                description: s("deploy pipeline and release policy fixture"),
            },
            GeneratedArtifactFixture {
                artifact_kind: s("preview-environment-lifecycle"),
                path: s("fixtures/devportal/orders-api/preview-environment.json"),
                description: s("preview environment lifecycle fixture"),
            },
            GeneratedArtifactFixture {
                artifact_kind: s("backing-resource-descriptor"),
                path: s("fixtures/devportal/orders-api/resources.database.json"),
                description: s("database resource descriptor fixture"),
            },
        ],
        preview_environment: PreviewEnvironmentFixture {
            environment_id: s("prv-orders-api-001a"),
            lifecycle_state: s("preview-ready-fixture"),
            url: s("https://preview.devportal.local/orders-api"),
            rollback_behavior: s("delete preview fixture row; no cluster action"),
        },
        declared_resources: vec![s("service"), s("database"), s("topic"), s("bucket")],
        resource_facets: vec![
            ResourceFacetFixture {
                resource: s("service"),
                lifecycle: s("fixture-created -> preview-ready -> tombstoned"),
                identity: s("service account reference only"),
                policy: s("Cedar allowlist fixture"),
                quota: s("500m CPU / 512Mi memory"),
                billing: s("compute cost preview line"),
                audit: s("REC-DEVPORTAL-001A"),
                observability: s("OpenSLO + trace fixture"),
                rollback: s("operation compensation marker"),
                reconciliation: s("reconciler-owned, not UI-owned"),
            },
            ResourceFacetFixture {
                resource: s("database"),
                lifecycle: s("fixture-reserved -> schema-preview -> tombstoned"),
                identity: s("database role reference only"),
                policy: s("tenant data residency fixture"),
                quota: s("20Gi storage ceiling"),
                billing: s("storage cost preview line"),
                audit: s("REC-DEVPORTAL-001A-DB"),
                observability: s("connection and migration checks"),
                rollback: s("drop preview schema marker"),
                reconciliation: s("resource controller fixture"),
            },
            ResourceFacetFixture {
                resource: s("topic"),
                lifecycle: s("fixture-reserved -> preview topic -> tombstoned"),
                identity: s("publisher/consumer refs"),
                policy: s("event contract allowlist fixture"),
                quota: s("1k msg/min preview cap"),
                billing: s("event cost preview line"),
                audit: s("REC-DEVPORTAL-001A-TOPIC"),
                observability: s("lag and DLQ fixture"),
                rollback: s("delete topic marker"),
                reconciliation: s("event bus reconciler fixture"),
            },
            ResourceFacetFixture {
                resource: s("bucket"),
                lifecycle: s("fixture-reserved -> preview bucket -> tombstoned"),
                identity: s("object-writer reference only"),
                policy: s("retention and encryption fixture"),
                quota: s("5Gi preview cap"),
                billing: s("object storage cost preview line"),
                audit: s("REC-DEVPORTAL-001A-BUCKET"),
                observability: s("object count fixture"),
                rollback: s("delete bucket marker"),
                reconciliation: s("storage reconciler fixture"),
            },
        ],
        facet_contract_fail_closed: true,
        cost_preview: CostPreviewFixture {
            currency: s("USD"),
            monthly_minor_units: 7420,
        },
        policy_denial_fixture: PolicyDenialFixture {
            decision: s("deny"),
            reason: s("policy denied fixture: unapproved template version or quota over budget"),
        },
        audit_event_fixture: vec![
            DeveloperPortalAuditEventFixture {
                event_type: s("developer_portal.provisioning.requested"),
                receipt: s("REC-DEVPORTAL-001A"),
            },
            DeveloperPortalAuditEventFixture {
                event_type: s("developer_portal.provisioning.policy_denied"),
                receipt: s("REC-DEVPORTAL-001A-DENY"),
            },
            DeveloperPortalAuditEventFixture {
                event_type: s("developer_portal.generated_artifacts.registered"),
                receipt: s("REC-DEVPORTAL-001A-ARTIFACTS"),
            },
        ],
        role_coverage: vec![
            DeveloperPortalRoleCoverage {
                role: s("platform engineer"),
                path: s("publish/deprecate approved templates and inspect generated artifacts"),
            },
            DeveloperPortalRoleCoverage {
                role: s("security reviewer"),
                path: s("inspect Cedar decisions, policy denials, and audit evidence"),
            },
            DeveloperPortalRoleCoverage {
                role: s("tenant admin"),
                path: s(
                    "approve request, set quota/budget boundaries, view ownership and billing impact",
                ),
            },
        ],
    }
}

#[cfg(any(feature = "ssr", test))]
fn support_advisory_fixture() -> SupportAdvisoryFixture {
    let tenant_id = "tn_northwind_prod";

    SupportAdvisoryFixture {
        case_id: s("case-support-001a"),
        tenant_id: s(tenant_id),
        resource_ref: s(
            "orn:oya:us-east-2:acct-northwind-prod:cloud-compute:k8s-cluster/cluster-nw-app-01",
        ),
        service_ref: s("svc-orders-api"),
        severity: s("SEV-2"),
        response_target_label: s("SEV-2 · 4h target label"),
        entitlement_plan: s(
            "Enterprise support plan · response targets only, no 24x7 staffing claim",
        ),
        customer_state: s(
            "case opened -> diagnostic bundle preview attached -> customer communication awaiting SRE/CS handoff",
        ),
        incident_refs: vec![s("sreops_incident_ref=inc-sreops-20260701-17")],
        status_refs: vec![s("status_ref=status-component-orders-api-yellow")],
        trust_evidence_refs: vec![s(
            "trust_evidence_ref=trustcenter-control-soc2-cc7-2-redacted",
        )],
        governance_posture_refs: vec![s(
            "governance_posture_ref=govposture-policy-drift-asset-cluster-nw-app-01",
        )],
        finops_refs: vec![s("finops_ref=budget-anomaly-northwind-compute-20260701")],
        onboarding_refs: vec![s("admin_onboarding_ref=domain-verified-northwind.example")],
        audit_chain_refs: vec![s("audit_event_ref=aud-support-case-created-001")],
        diagnostic_bundle_ref: s("diagnostic_bundle_ref=db-support-case-001a-redacted"),
        diagnostic_bundle_evidence_refs: vec![
            s("resource/service IDs only"),
            s("health/status refs only"),
            s("latest SREOPS incident refs only"),
            s("Trust Center evidence refs only"),
            s("governance posture refs only"),
            s("FinOps budget/anomaly refs only"),
            s("admin/onboarding entitlement refs only"),
            s("audit-chain refs only"),
        ],
        advisor_key: s("budget_anomaly_followup"),
        advisor_severity: s("advisory · medium"),
        advisor_owner: s("customer success + FinOps owner"),
        recommended_action: s(
            "Schedule a budget anomaly follow-up, attach the redacted diagnostic bundle, and route remediation through Workflow for human approval.",
        ),
        non_goal_copy: s(
            "Advisory only: no recommendation engine, no billing mutation, no incident resolution, and no automated remediation in this slice.",
        ),
        expires_at: s("fresh_until=2026-07-08T00:00:00Z"),
        support_access_state: s(
            "approval-gated breakglass · pending security and tenant-admin approval",
        ),
        support_engineer_view: s(
            "support engineer redacted context · handles only, no raw logs/secrets/PII/PHI/financial details",
        ),
        post_case_actions: vec![
            s("post_case_action_item: verify budget anomaly owner accepted follow-up"),
            s("post_case_action_item: refresh stale Trust Center evidence handle"),
            s("post_case_action_item: confirm incident/status handoff remains read-only"),
        ],
        api_records: vec![
            support_api_record(
                "support_case",
                "case-support-001a",
                tenant_id,
                "tenant-admin-platform",
                "open case",
                "customer-support-metadata",
                "fresh",
                "redact-support-v1",
                "aud-support-case-created-001",
            ),
            support_api_record(
                "support_entitlement",
                "ent-northwind-enterprise",
                tenant_id,
                "support-bff",
                "show support plan targets",
                "entitlement-labels",
                "fresh",
                "redact-entitlement-v1",
                "aud-support-entitlement-viewed-001",
            ),
            support_api_record(
                "diagnostic_bundle",
                "db-support-case-001a-redacted",
                tenant_id,
                "tenant-admin-platform",
                "attach diagnostic bundle preview",
                "evidence-ref-handles",
                "fresh",
                "redact-diagnostic-bundle-v1",
                "aud-support-bundle-attached-001",
            ),
            support_api_record(
                "advisor_recommendation",
                "adv-budget-anomaly-followup-001",
                tenant_id,
                "advisor-fixture",
                "recommend follow-up",
                "advisory-metadata",
                "fresh_until=2026-07-08T00:00:00Z",
                "redact-advisor-v1",
                "aud-support-advisor-viewed-001",
            ),
            support_api_record(
                "customer_communication",
                "comm-support-case-001a",
                tenant_id,
                "tenant-admin-platform",
                "customer-visible case update",
                "communication-metadata",
                "fresh",
                "redact-communication-v1",
                "aud-support-communication-created-001",
            ),
            support_api_record(
                "support_access_approval",
                "saa-support-case-001a",
                tenant_id,
                "security-reviewer",
                "approve breakglass support access",
                "approval-handle",
                "pending",
                "redact-support-access-v1",
                "aud-support-access-requested-001",
            ),
            support_api_record(
                "post_case_action_item",
                "pcai-support-case-001a-001",
                tenant_id,
                "customer-success",
                "track post-case follow-up",
                "action-item-metadata",
                "fresh",
                "redact-action-item-v1",
                "aud-support-action-item-created-001",
            ),
        ],
        security_assertions: vec![
            s("tenant_id=tn_northwind_prod"),
            s("other tenant denied"),
            s("raw_log_absent=true"),
            s("raw_pii_absent=true"),
            s("raw_phi_absent=true"),
            s("financial_detail_absent=true"),
            s("exploit_payload_absent=true"),
            s("cross-tenant list/open/count/attach/export/search/support-mode denied"),
        ],
        authority_boundaries: vec![
            s("ADR-0536 D-4/D-5 support and trusted-advisor claim ceiling"),
            s(
                "specs/root-hub-pointers.json#entry_points.agent_operating_contract current authority",
            ),
            s("specs/sre-operations-center-contract.json linked by ref only"),
            s("specs/trust-center-compliance-evidence-portal.json evidence handle only"),
            s("no status publish authority"),
            s("no Trust Center publishability mutation"),
            s("no SREOPS incident resolution authority"),
            s("no live notification delivery or external ticket-provider integration"),
        ],
    }
}

#[cfg(any(feature = "ssr", test))]
#[allow(
    clippy::too_many_arguments,
    reason = "Fixture records intentionally spell out the API contract fields required by SUPPORT-ADVISORY-001A."
)]
fn support_api_record(
    record_type: &str,
    record_id: &str,
    tenant_id: &str,
    actor: &str,
    purpose: &str,
    data_class: &str,
    freshness: &str,
    redaction_policy_id: &str,
    audit_event_ref: &str,
) -> SupportApiRecordFixture {
    SupportApiRecordFixture {
        record_type: s(record_type),
        record_id: s(record_id),
        tenant_id: s(tenant_id),
        actor: s(actor),
        purpose: s(purpose),
        data_class: s(data_class),
        freshness: s(freshness),
        redaction_policy_id: s(redaction_policy_id),
        audit_event_ref: s(audit_event_ref),
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

    #[test]
    fn support_advisory_fixture_is_tenant_admin_only() {
        let tenant_admin = server_derived_envelope(OperatorContext::TenantAdmin);
        assert!(tenant_admin.support_advisory.is_some());
        assert!(
            tenant_admin
                .modules
                .iter()
                .any(|module| module.name == "Customer Support")
        );

        for context in [
            OperatorContext::CorporateOffice,
            OperatorContext::HealthcareClinician,
        ] {
            let envelope = server_derived_envelope(context);
            assert!(
                envelope.support_advisory.is_none(),
                "{context:?} must not receive the tenant-admin support/advisory fixture"
            );
            assert!(
                !envelope
                    .modules
                    .iter()
                    .any(|module| module.name == "Customer Support"),
                "{context:?} must not receive the support navigation module"
            );
        }
    }

    #[test]
    fn developer_portal_fixture_models_approved_template_admission_and_artifacts() {
        let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
        let portal = envelope.developer_portal;

        assert_eq!(portal.catalog_entity.kind, "ServiceCatalogEntity");
        assert_eq!(portal.approved_template.template_id, "svc-rust-axum-leptos");
        assert_eq!(portal.approved_template.version.version, "1.0.0-fixture");
        assert!(
            portal
                .approved_template
                .supported_golden_path_resources
                .iter()
                .any(|resource| resource == "database")
        );
        assert!(
            portal
                .approved_template
                .supported_golden_path_resources
                .iter()
                .any(|resource| resource == "preview env")
        );
        assert!(
            portal
                .approved_template
                .parameters
                .iter()
                .any(|parameter| parameter.name == "service_slug" && parameter.required)
        );
        assert!(
            portal
                .admission_sequence
                .starts_with(&["identity/tenant/project binding".to_string()])
        );
        assert_eq!(portal.provisioning_request.request_id, "prq-devportal-001a");
        assert_eq!(
            portal.provisioning_operation.operation_id,
            "op-devportal-001a"
        );
        assert_eq!(portal.provisioning_operation.state, "accepted_fixture");
        assert_eq!(portal.generated_artifacts.len(), 5);
        assert!(
            portal
                .generated_artifacts
                .iter()
                .any(|artifact| artifact.artifact_kind == "OpenSLO")
        );
        assert!(
            portal
                .generated_artifacts
                .iter()
                .any(|artifact| artifact.artifact_kind == "progressive-delivery-policy")
        );
        assert_eq!(
            portal.preview_environment.lifecycle_state,
            "preview-ready-fixture"
        );
    }

    #[test]
    fn developer_portal_facets_policy_denial_quota_and_audit_are_fail_closed() {
        let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
        let portal = envelope.developer_portal;

        assert!(portal.facet_contract_fail_closed);
        assert_eq!(
            portal.resource_facets.len(),
            portal.declared_resources.len()
        );
        for facets in &portal.resource_facets {
            assert_ne!(facets.lifecycle, "missing");
            assert_ne!(facets.identity, "missing");
            assert_ne!(facets.policy, "missing");
            assert_ne!(facets.quota, "missing");
            assert_ne!(facets.billing, "missing");
            assert_ne!(facets.audit, "missing");
            assert_ne!(facets.observability, "missing");
            assert_ne!(facets.rollback, "missing");
            assert_ne!(facets.reconciliation, "missing");
        }
        assert_eq!(portal.cost_preview.currency, "USD");
        assert!(portal.cost_preview.monthly_minor_units > 0);
        assert_eq!(portal.policy_denial_fixture.decision, "deny");
        assert!(
            portal
                .audit_event_fixture
                .iter()
                .any(|event| event.event_type == "developer_portal.provisioning.requested")
        );
        assert!(
            portal
                .role_coverage
                .iter()
                .any(|role| role.role == "platform engineer")
        );
        assert!(
            portal
                .role_coverage
                .iter()
                .any(|role| role.role == "security reviewer")
        );
        assert!(
            portal
                .role_coverage
                .iter()
                .any(|role| role.role == "tenant admin")
        );
    }
}

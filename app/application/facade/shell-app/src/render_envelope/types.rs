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
    #[serde(default)]
    pub finance_body_projection: Option<FinanceBodyProjection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FinanceBodyProjection {
    pub context: OperatorContext,
    pub body: String,
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
pub struct InboxSampleMetadata {
    pub id: String,
    pub owner: String,
    pub owned_by_viewer: bool,
    pub blocking: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkItem {
    pub title: String,
    pub detail: String,
    pub priority: String,
    pub inbox: InboxSampleMetadata,
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
    pub inbox: InboxSampleMetadata,
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
#[serde(deny_unknown_fields)]
pub struct IntelligenceSuggestion {
    pub id: IntelligenceSampleId,
    pub title: String,
    pub body: String,
    pub guardrail: String,
    pub destination: Option<String>,
    pub channel: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntelligenceSampleId {
    TenantWorkflowDelegation,
    TenantTaxFiling,
    TenantOvertime,
    CorporatePolicyChecklist,
    CorporateInvoiceMismatch,
    CorporateDayPriority,
    ClinicalDischargeChecklist,
    ClinicalHandoff,
    ClinicalChecklistGap,
}

impl IntelligenceSampleId {
    pub const ALL: [Self; 9] = [
        Self::TenantWorkflowDelegation,
        Self::TenantTaxFiling,
        Self::TenantOvertime,
        Self::CorporatePolicyChecklist,
        Self::CorporateInvoiceMismatch,
        Self::CorporateDayPriority,
        Self::ClinicalDischargeChecklist,
        Self::ClinicalHandoff,
        Self::ClinicalChecklistGap,
    ];

    pub fn context(self) -> OperatorContext {
        match self {
            Self::TenantWorkflowDelegation | Self::TenantTaxFiling | Self::TenantOvertime => {
                OperatorContext::TenantAdmin
            }
            Self::CorporatePolicyChecklist
            | Self::CorporateInvoiceMismatch
            | Self::CorporateDayPriority => OperatorContext::CorporateOffice,
            Self::ClinicalDischargeChecklist
            | Self::ClinicalHandoff
            | Self::ClinicalChecklistGap => OperatorContext::HealthcareClinician,
        }
    }

    /// Existing local sample views only; this relation grants no live action.
    pub fn route(self) -> Option<(&'static str, Option<&'static str>)> {
        match self {
            Self::TenantWorkflowDelegation
            | Self::CorporatePolicyChecklist
            | Self::ClinicalDischargeChecklist
            | Self::ClinicalChecklistGap => Some(("workflow-studio", None)),
            Self::TenantTaxFiling => None,
            Self::TenantOvertime => Some(("identity-panel-employees", None)),
            Self::CorporateInvoiceMismatch => Some(("finance-panel-ledger", None)),
            Self::CorporateDayPriority => Some(("work-hub", Some("Mail"))),
            Self::ClinicalHandoff => Some(("work-hub", Some("Messenger"))),
        }
    }

    pub fn id(self) -> String {
        serde_json::to_value(self)
            .expect("sample identity serializes")
            .as_str()
            .expect("sample identity is a string")
            .to_owned()
    }
}

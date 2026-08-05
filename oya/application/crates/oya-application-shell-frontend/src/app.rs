use leptos::prelude::*;

use crate::design_system::audit_evidence_timeline::{
    AuditEvidenceTimeline, EvidencePath, EvidenceRow, RowSeverity, RowState, TimelineVariant,
    row_severity,
};
use crate::design_system::ops_deployment_status_panel::{
    ClusterHealthStatus, DeploymentStatus, OpsDeploymentStatusPanel,
};
#[cfg(any(feature = "ssr", test))]
use crate::render_envelope::server_derived_envelope;
use crate::render_envelope::{
    ApprovalItem, CommunityItem, DeveloperPortalFixture, IntelligenceSuggestion, MessageItem,
    MetricCard, ModuleCard, OntologyFact, OperatorContext, ProductActivitySpine,
    ProductActivityStep, ScheduleItem, SupportAdvisoryFixture, TenantRenderEnvelope, WorkItem,
    WorkflowNode,
};

const OPS_CLUSTER_HEALTH_FIXTURE_JSON: &str = r#"{
    "cluster_id": "cell-us-east-2",
    "observed_at": "2026-07-01T05:00:00Z",
    "health": "yellow",
    "signals": [
        "argocd-app-health degraded",
        "cosign verify ok",
        "traceparent fixture only"
    ]
}"#;

const CLOUD_OBSERVABILITY_AUDIT_READ_ENDPOINT: &str =
    "QUERY /v1/cloud/observability/audit-records:read";
const CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE: &str = "cloud.observability.audit.read";
const CLOUD_OBSERVABILITY_AUDIT_OPERATION_CLASS: &str = "cloud_iam_policy";
const CLOUD_OBSERVABILITY_AUDIT_AUTHORITY_REF: &str =
    "contracts/openapi/cloud/cloud-observability-audit-v1.yaml#readCloudObservabilityAudit";
const CLOUD_OBSERVABILITY_AUDIT_SOURCE_ANCHORS: &str = "G007 goals.json + AUTHZ-006 + specs/root-hub-pointers.json#entry_points.agent_operating_contract";
const CLOUD_OBSERVABILITY_AUDIT_BOUNDARY: &str = "analysis-only-non-mutating";
const CLOUD_OBSERVABILITY_AUDIT_READ_JSON: &str = r#"{
    "data": [
        {
            "id": "caud_tn_northwind_prod_42",
            "tenant_id": "tn_northwind_prod",
            "region": "us-east-2",
            "cell_id": "cell-us-east-2a",
            "topic": "oya.audit.cloud_iam_policy",
            "operation": "iam_policy_changed",
            "record_class": "control_plane_mutation",
            "source_resource_id": "orn:oya:us-east-2:acct-northwind-prod:cloud-iam:policy/policy-tenant-admin",
            "actor": "sp_audit_reader",
            "iam_role": "role_cloud_security_reviewer",
            "occurred_at_epoch_seconds": 1782887880,
            "chain_sequence": 42,
            "previous_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "hash": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "payload_hash": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
            "idempotency_key": "idem/cloud-iam-policy/001",
            "decision": "allow tenant admin policy delta after AUTHZ-006 review",
            "purpose": "security",
            "plane": "audit",
            "data_classes_referenced": [
                { "label": "INTERNAL_ONLY" },
                { "label": "AUDIT" }
            ],
            "signed_export_uri": "oyaaudit://signed-export/northwind/cloud-iam-policy?sig=sig_cloud_iam_policy_001",
            "audit_marker": "AUDIT",
            "schema_version": 1
        }
    ],
    "metadata": {
        "request_id": "req-shell-004-audit-evidence",
        "tenant_id": "tn_northwind_prod",
        "region": "us-east-2",
        "record_count": 1,
        "next_cursor": null,
        "chain_complete": true,
        "high_watermark_sequence": 42
    }
}"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProductSurface {
    Workflow,
    Messenger,
    Mail,
    Community,
}

impl ProductSurface {
    const ALL: [Self; 4] = [Self::Workflow, Self::Messenger, Self::Mail, Self::Community];

    const fn label(self) -> &'static str {
        match self {
            Self::Workflow => "Workflow Studio",
            Self::Messenger => "Messenger",
            Self::Mail => "Mail",
            Self::Community => "Community",
        }
    }

    const fn summary(self) -> &'static str {
        match self {
            Self::Workflow => "Build governed no-code flows",
            Self::Messenger => "Discuss operational threads",
            Self::Mail => "Draft formal work messages",
            Self::Community => "Coordinate role-aware spaces",
        }
    }

    const fn href(self) -> &'static str {
        match self {
            Self::Workflow => "#workflow-studio",
            Self::Messenger | Self::Mail | Self::Community => "#work-hub",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkflowTool {
    Select,
    Connect,
    Simulate,
}

impl WorkflowTool {
    const ALL: [Self; 3] = [Self::Select, Self::Connect, Self::Simulate];

    const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Connect => "Connect",
            Self::Simulate => "Simulate",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalDraft {
    surface: ProductSurface,
    title: String,
    body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HubItem {
    surface: ProductSurface,
    source: String,
    title: String,
    body: String,
    meta: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExecutionRow {
    id: String,
    kind: &'static str,
    state: &'static str,
    title: String,
    body: String,
    owner: String,
    due: String,
    route: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResourceRow {
    kind: &'static str,
    name: &'static str,
    region: &'static str,
    owner: &'static str,
    state: &'static str,
    monthly: &'static str,
    risk: &'static str,
    side_id: &'static str,
    description: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AuditReceipt {
    time: &'static str,
    event: &'static str,
    actor: &'static str,
    receipt: &'static str,
    severity: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DeploymentGate {
    label: &'static str,
    detail: &'static str,
    state: &'static str,
    progress: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CloudObservabilityAuditReadSuccessResponse {
    data: Vec<CloudObservabilityAuditRecord>,
    metadata: CloudObservabilityAuditReadMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CloudObservabilityAuditReadMetadata {
    request_id: String,
    tenant_id: String,
    region: String,
    record_count: u32,
    next_cursor: Option<String>,
    chain_complete: bool,
    high_watermark_sequence: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CloudObservabilityAuditRecord {
    id: String,
    tenant_id: String,
    region: String,
    cell_id: Option<String>,
    topic: String,
    operation: String,
    record_class: String,
    source_resource_id: Option<String>,
    actor: String,
    iam_role: Option<String>,
    occurred_at_epoch_seconds: u64,
    chain_sequence: u64,
    previous_hash: String,
    hash: String,
    payload_hash: String,
    idempotency_key: String,
    decision: String,
    purpose: String,
    plane: String,
    data_classes_referenced: Vec<CloudObservabilityDataClassRef>,
    signed_export_uri: String,
    audit_marker: String,
    schema_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CloudObservabilityDataClassRef {
    label: String,
}

impl CloudObservabilityAuditReadSuccessResponse {
    fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    const fn endpoint(&self) -> &'static str {
        CLOUD_OBSERVABILITY_AUDIT_READ_ENDPOINT
    }

    const fn authz_surface(&self) -> &'static str {
        CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE
    }

    const fn operation_class(&self) -> &'static str {
        CLOUD_OBSERVABILITY_AUDIT_OPERATION_CLASS
    }

    const fn authority_ref(&self) -> &'static str {
        CLOUD_OBSERVABILITY_AUDIT_AUTHORITY_REF
    }

    const fn source_anchors(&self) -> &'static str {
        CLOUD_OBSERVABILITY_AUDIT_SOURCE_ANCHORS
    }

    const fn authority_boundary(&self) -> &'static str {
        CLOUD_OBSERVABILITY_AUDIT_BOUNDARY
    }

    fn high_watermark_sequence_label(&self) -> String {
        self.metadata
            .high_watermark_sequence
            .map(|sequence| sequence.to_string())
            .unwrap_or_else(|| "none".to_owned())
    }

    fn timeline_rows(&self) -> Vec<EvidenceRow> {
        self.data
            .iter()
            .filter(|record| record.operation_class() == self.operation_class())
            .map(|record| record.to_timeline_row(self.metadata.chain_complete))
            .collect()
    }
}

impl CloudObservabilityAuditRecord {
    fn operation_class(&self) -> &str {
        self.topic
            .strip_prefix("oya.audit.")
            .unwrap_or(self.operation.as_str())
    }

    fn to_timeline_row(&self, chain_complete: bool) -> EvidenceRow {
        let state =
            if chain_complete && self.audit_marker == "AUDIT" && !self.signed_export_uri.is_empty()
            {
                RowState::Complete
            } else if self.signed_export_uri.is_empty() {
                RowState::SignatureMissing
            } else {
                RowState::MissingRow
            };
        let source = self
            .source_resource_id
            .clone()
            .unwrap_or_else(|| self.id.clone());
        let classes = self
            .data_classes_referenced
            .iter()
            .map(|data_class| data_class.label.as_str())
            .collect::<Vec<_>>()
            .join("+");
        let cell = self.cell_id.as_deref().unwrap_or("region-wide");
        let iam_role = self.iam_role.as_deref().unwrap_or("role absent");

        EvidenceRow {
            state,
            event_type: format!("{} · {}", self.topic, self.operation),
            timestamp: format!("epoch-seconds:{}", self.occurred_at_epoch_seconds),
            evidence: Some(EvidencePath::SignedExternal {
                digest: self.hash.clone(),
                signature_ref: self.signed_export_uri.clone(),
            }),
            signature_status: format!(
                "signed export {} · seq {} · previous {} · payload {} · tenant {} · region {} · cell {} · actor {} · role {} · purpose {} · plane {} · class {} · data {} · schema {}",
                self.signed_export_uri,
                self.chain_sequence,
                self.previous_hash,
                self.payload_hash,
                self.tenant_id,
                self.region,
                cell,
                self.actor,
                iam_role,
                self.purpose,
                self.plane,
                self.record_class,
                classes,
                self.schema_version
            ),
            linked_change_id: format!("{} · {} · {}", self.idempotency_key, source, self.decision),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BusinessLogicRow {
    id: &'static str,
    name: &'static str,
    english_name: &'static str,
    category: &'static str,
    owner: &'static str,
    cadence: &'static str,
    criticality: &'static str,
    cost: &'static str,
    sla: &'static str,
    state: &'static str,
    state_label: &'static str,
    tasks: &'static str,
    route: &'static str,
    description: &'static str,
}

const BUSINESS_LOGIC_ROWS: [BusinessLogicRow; 7] = [
    BusinessLogicRow {
        id: "BL-001",
        name: "2026-04 payroll close",
        english_name: "Payroll close",
        category: "workforce",
        owner: "Finance + HR",
        cadence: "Monthly",
        criticality: "P0",
        cost: "₩2.18M",
        sla: "5.4/4.0d",
        state: "at-risk",
        state_label: "at risk",
        tasks: "3",
        route: "#payroll-cockpit",
        description: "Insurance delta, payroll reminder mail, and sealed receipt staged into one governed work item.",
    },
    BusinessLogicRow {
        id: "BL-002",
        name: "Withholding return",
        english_name: "HomeTax filing readiness",
        category: "compliance",
        owner: "Tax operations",
        cadence: "Monthly",
        criticality: "P0",
        cost: "₩820k",
        sla: "3.1/4.0d",
        state: "review",
        state_label: "review",
        tasks: "2",
        route: "#filing-readiness",
        description: "Employee validation, HomeTax transport, reviewer attestation, and audit receipt are visible before submission.",
    },
    BusinessLogicRow {
        id: "BL-003",
        name: "Vendor renewal review",
        english_name: "Spend exception workflow",
        category: "finance",
        owner: "CFO office",
        cadence: "Weekly",
        criticality: "P1",
        cost: "₩1.44M",
        sla: "1.2/2.0d",
        state: "attention",
        state_label: "attention",
        tasks: "1",
        route: "#vendors-spend",
        description: "Stripe renewal, budget note, and CFO attestation are linked to FinOps and vendor spend surfaces.",
    },
    BusinessLogicRow {
        id: "BL-004",
        name: "Access recertification",
        english_name: "Quarterly role envelope review",
        category: "trust",
        owner: "Security reviewer",
        cadence: "Quarterly",
        criticality: "P0",
        cost: "₩640k",
        sla: "2.8/3.0d",
        state: "review",
        state_label: "review",
        tasks: "4",
        route: "#policy-access",
        description: "RBAC, sessions, policy evidence, and deployment gates share one recertification spine.",
    },
    BusinessLogicRow {
        id: "BL-005",
        name: "New hire onboarding",
        english_name: "Offer → account → payroll",
        category: "workforce",
        owner: "People ops",
        cadence: "Event",
        criticality: "P1",
        cost: "₩430k",
        sla: "0.8/2.0d",
        state: "on-track",
        state_label: "on track",
        tasks: "2",
        route: "#identity-onboarding",
        description: "Onboarding, documents, payroll setup, and community announcement are staged from one workflow.",
    },
    BusinessLogicRow {
        id: "BL-006",
        name: "Tenant network split",
        english_name: "Cloud change governance",
        category: "cloud",
        owner: "Infrastructure ops",
        cadence: "Event",
        criticality: "P0",
        cost: "₩3.6M",
        sla: "6.1/4.0d",
        state: "blocked",
        state_label: "blocked",
        tasks: "5",
        route: "#cloud-ops-cockpit",
        description: "Network split, rollback runbook, FinOps anomaly, and audit evidence are unified as a governed logic.",
    },
    BusinessLogicRow {
        id: "BL-007",
        name: "Governance council note",
        english_name: "Community evidence broadcast",
        category: "community",
        owner: "Governance",
        cadence: "Ad hoc",
        criticality: "P2",
        cost: "₩90k",
        sla: "0.4/1.0d",
        state: "done",
        state_label: "done",
        tasks: "0",
        route: "#work-hub",
        description: "Council update fans out to Messenger, Mail, Community, and an evidence-spine receipt.",
    },
];

pub fn shell_scope_notice_text() -> &'static str {
    "Operator console scope: panels render transitional in-process data behind the locked shell-BFF contracts pending live service integration; no PHI/PII · shell covers close, workflow, people, mail, messenger, and community."
}

pub fn shell_landmark_label() -> &'static str {
    "Oyatie Operations · Cloud/Tenant Control Center"
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="oya-console-app">
            <a class="skip-link" href="#console-shell">"Skip to dashboard"</a>
            <ShellRail />
            <ShellHeader />
            <main
                id="console-shell"
                class="control-center"
                aria-labelledby="console-title"
                aria-describedby="console-notice"
            >
                <HeroPanel />
                <DashboardIsland />
            </main>
            <UtilityPanels />
            <SidePeek />
        </div>
    }
}

#[component]
fn ShellRail() -> impl IntoView {
    view! {
        <aside class="app-rail" aria-label="Product navigation">
            <div class="rail-brand">
                <span class="rail-mark" aria-hidden="true">"O"</span>
                <div>
                    <strong>"Oyatie"</strong>
                    <span>"Control Center"</span>
                </div>
                <code>"v0.1"</code>
            </div>
            <section class="rail-proof-card" aria-label="FD-001 and Oyatie Cloud shell proof">
                <p>"FD-001 TENANT WORKLOADS"</p>
                <strong>"Service graph on Oyatie Cloud"</strong>
                <span>"Messenger · Mail · Community dogfood the substrate."</span>
                <small data-rail-status="true">"REC-WF-7741 · cell-us-east-2 · local visual routes"</small>
                <div class="rail-proof-actions" aria-label="Persistent shell proof routes">
                    <button type="button" class="is-selected" data-rail-proof-action="service-graph">"Service graph"</button>
                    <button type="button" data-rail-proof-action="cloud">"Cloud"</button>
                    <button type="button" data-rail-proof-action="evidence">"Evidence"</button>
                    <button type="button" data-rail-proof-action="work-hub">"Work hub"</button>
                </div>
                <div class="rail-comms-switcher" aria-label="Built-in Work Hub surface routes">
                    <button type="button" class="is-selected" data-rail-comms-surface="Messenger">"Messenger"</button>
                    <button type="button" data-rail-comms-surface="Mail">"Mail"</button>
                    <button type="button" data-rail-comms-surface="Community">"Community"</button>
                </div>
            </section>
            <p class="rail-group">"Run the company"</p>
            <a class="rail-nav active" href="#console-shell"><span aria-hidden="true">"⌂"</span>"Command center"</a>
            <a class="rail-nav" href="#command-center-workbench"><span aria-hidden="true">"▥"</span>"Action Inbox"<em>"8"</em></a>
            <a class="rail-nav" href="#governance-analytics"><span aria-hidden="true">"↟"</span>"Governance analytics"</a>
            <p class="rail-group">"Operate"</p>
            <a class="rail-nav" href="#business-logics"><span aria-hidden="true">"⌬"</span>"Business Logics"<em>"17"</em></a>
            <a class="rail-nav" href="#tasks-title"><span aria-hidden="true">"☑"</span>"Tasks"<em>"73"</em></a>
            <a class="rail-nav" href="#schedule-title"><span aria-hidden="true">"◷"</span>"Schedule"</a>
            <a class="rail-nav" href="#workflow-studio"><span aria-hidden="true">"⌘"</span>"Workflow Studio"</a>
            <a class="rail-nav" href="#work-hub"><span aria-hidden="true">"✉"</span>"Messenger · Mail · Community"<em>"18"</em></a>
            <a class="rail-nav" href="#cloud-ops-cockpit"><span aria-hidden="true">"◫"</span>"Cloud Ops"</a>
            <p class="rail-group">"Money"</p>
            <a class="rail-nav" href="#payroll-cockpit"><span aria-hidden="true">"₩"</span>"Payroll"</a>
            <a class="rail-nav" href="#ledger-preview"><span aria-hidden="true">"▤"</span>"Ledger"</a>
            <a class="rail-nav" href="#vendors-spend"><span aria-hidden="true">"◇"</span>"Vendors & spend"</a>
            <a class="rail-nav" href="#billing-tax"><span aria-hidden="true">"▧"</span>"Billing & tax"</a>
            <a class="rail-nav" href="#finops-pane"><span aria-hidden="true">"₩"</span>"FinOps"</a>
            <p class="rail-group">"Compliance"</p>
            <a class="rail-nav" href="#filing-readiness"><span aria-hidden="true">"□"</span>"Filing readiness"<em>"2"</em></a>
            <a class="rail-nav" href="#audit-ledger"><span aria-hidden="true">"◱"</span>"Audit ledger"</a>
            <a class="rail-nav" href="#policy-access"><span aria-hidden="true">"⚿"</span>"Policy & access"</a>
            <p class="rail-group">"People"</p>
            <a class="rail-nav" href="#identity-employees"><span aria-hidden="true">"◎"</span>"Employees"</a>
            <a class="rail-nav" href="#leave-time"><span aria-hidden="true">"◫"</span>"Leave & time"</a>
            <a class="rail-nav" href="#identity-workforce-service"><span aria-hidden="true">"⚿"</span>"Auth · Org"</a>
            <p class="rail-group">"Trust"</p>
            <a class="rail-nav" href="#trust-center-portal"><span aria-hidden="true">"◈"</span>"Trust Center"<em>"new"</em></a>
            <a class="rail-nav" href="#resource-inventory"><span aria-hidden="true">"▤"</span>"Resource inventory"</a>
            <a class="rail-nav" href="#modules-title"><span aria-hidden="true">"▦"</span>"Service catalog"</a>
            <a class="rail-nav" href="#evidence-spine"><span aria-hidden="true">"▥"</span>"Evidence spine"</a>
            <a class="rail-nav" href="#deployment-gates"><span aria-hidden="true">"✓"</span>"Deployment gates"</a>
            <a class="rail-nav" href="#ontology-title"><span aria-hidden="true">"◎"</span>"Object graph"</a>
            <a class="rail-nav" href="#intelligence-title"><span aria-hidden="true">"✦"</span>"Copilot rail"</a>
            <div class="workspace-switch">
                <span class="workspace-avatar" aria-hidden="true">"N"</span>
                <div>
                    <strong>"Northwind"</strong>
                    <span>"Enterprise · US/EU/KR"</span>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn ShellHeader() -> impl IntoView {
    view! {
        <header class="app-header" role="banner">
            <div class="top-breadcrumb" aria-label="Breadcrumb">
                <span>"Oyatie Cloud"</span>
                <span class="sep">"/"</span>
                <span>"Operations"</span>
                <span class="sep">"/"</span>
                <strong>"Control Center"</strong>
            </div>
            <div class="header-route-strip" aria-label="FD-001 and Oyatie Cloud quick routes">
                <button type="button" class="is-selected" data-header-route="fd001"><span>"FD-001"</span>"Service graph"</button>
                <button type="button" data-header-route="cloud"><span>"Cloud"</span>"Substrate"</button>
                <button type="button" data-header-route="work-hub"><span>"Comms"</span>"Work hub"</button>
                <div class="header-comms-switcher" aria-label="Built-in communications quick routes">
                    <button type="button" class="is-selected" data-header-comms-surface="Messenger">"Messenger"</button>
                    <button type="button" data-header-comms-surface="Mail">"Mail"</button>
                    <button type="button" data-header-comms-surface="Community">"Community"</button>
                </div>
                <button type="button" data-header-route="evidence"><span>"Audit"</span>"Evidence"</button>
                <small data-header-route-status="true">"REC-WF-7741 · local quick routes"</small>
            </div>
            <button class="command-trigger" type="button" data-command-trigger="true" aria-haspopup="dialog">
                <span aria-hidden="true">"⌕"</span>
                <span>"Search actions, objects, workflows"</span>
                <kbd>"⌘K"</kbd>
            </button>
            <div class="header-actions" aria-label="Shell render status">
                <button type="button" class="header-status">"SSR shell"</button>
                <button type="button" class="header-status muted">"Selective WASM islands"</button>
                <button type="button" class="header-icon" data-header-action="notifications" aria-label="Open notifications">
                    "◔"
                    <span class="header-badge" data-activity-badge="true">"3"</span>
                </button>
                <button type="button" class="header-icon" data-header-action="settings" aria-label="Open settings">"⚙"</button>
            </div>
        </header>
    }
}

#[component]
fn HeroPanel() -> impl IntoView {
    view! {
        <section class="hero-panel" aria-labelledby="console-title">
            <div class="hero-main">
                <div class="page-title-copy">
                    <p class="screen-anchor">"01 / Command Center"</p>
                    <div class="hero-title-row">
                        <h1 id="console-title">"Operations · 2026 May, week 19"</h1>
                        <span class="hero-lens-chip">"● Lens: tenant admin · Finance · 1,000 ppl"</span>
                    </div>
                    <p id="console-notice" class="scope-notice" role="note">
                        "이번 주 운영 현황 — 마감, 신고, 인적자원, 결재 대기 "
                        <span>"This week — close, filings, people, approvals."</span>
                    </p>
                </div>
                <section class="hero-close-strip" aria-label="FD-001 close command proof">
                    <div>
                        <p class="screen-anchor">"FD-001 CLOSE COMMAND"</p>
                        <strong>"April close proves the product workload on Oyatie Cloud"</strong>
                        <span data-hero-status="true">"Ready · REC-CLOSE-2026-04 · cell-us-east-2 · local command only"</span>
                    </div>
                    <div class="hero-close-actions" aria-label="Close package routes">
                        <button type="button" data-hero-action="close-april">"Stage close"</button>
                        <button type="button" data-hero-action="route-ledger">"Ledger"</button>
                        <button type="button" data-hero-action="route-cloud">"Cloud proof"</button>
                        <button type="button" data-hero-action="route-evidence">"Evidence"</button>
                    </div>
                </section>
                <section class="render-architecture-strip" aria-label="SSR shell and selective WASM hydration model">
                    <article class="selected" data-render-arch-card="ssr">
                        <p class="screen-anchor">"SSR SHELL"</p>
                        <strong>"Fast baseline, service graph visible first"</strong>
                        <span>"Navigation, proof copy, tenant posture, and core dashboards render before island hydration."</span>
                        <button type="button" class="is-selected" data-render-arch-action="ssr">"Show shell"</button>
                    </article>
                    <article data-render-arch-card="islands">
                        <p class="screen-anchor">"SELECTIVE WASM"</p>
                        <strong>"Only interactive product surfaces hydrate"</strong>
                        <span>"Workflow Studio, Work Hub, filters, canvas state, and local drafts become browser-only islands."</span>
                        <button type="button" data-render-arch-action="islands">"Show islands"</button>
                    </article>
                    <article data-render-arch-card="boundary">
                        <p class="screen-anchor">"LOCAL BOUNDARY"</p>
                        <strong>"Visually functional, deliberately unwired"</strong>
                        <span data-render-arch-status="true">"No workflow execution, external send, IAM, billing, deploy, or cloud mutation."</span>
                        <button type="button" data-render-arch-action="boundary">"Show evidence"</button>
                    </article>
                </section>
            </div>
            <div class="hero-side">
                <div class="hero-copy page-actions">
                    <button
                        type="button"
                        data-sidepeek-trigger="new-action"
                        data-sidepeek-title="Create governed action"
                        data-sidepeek-id="ACT-LOCAL-DRAFT"
                        data-sidepeek-desc="Local visual-only action draft. Nothing is persisted or sent."
                        data-sidepeek-owner="Current operator session"
                        data-sidepeek-risk="Draft"
                        data-sidepeek-sla="No live SLA"
                    >
                        "New action"
                    </button>
                    <button type="button" data-command-trigger="true">"Search ⌘K"</button>
                    <button type="button" class="primary" data-hero-action="close-april">"Close April →"</button>
                </div>
            </div>
        </section>
    }
}

#[component]
fn UtilityPanels() -> impl IntoView {
    view! {
        <div class="utility-panel-backdrop" data-utility-backdrop hidden></div>

        <section
            class="utility-panel activity-center"
            data-utility-panel="notifications"
            aria-label="Notification and activity center"
            aria-hidden="true"
        >
            <div class="utility-panel-head">
                <div>
                    <p class="screen-anchor">"ACTIVITY CENTER"</p>
                    <h2>"Notifications, approvals, and local events"</h2>
                </div>
                <button type="button" data-utility-close="true" aria-label="Close activity center">"×"</button>
            </div>
            <section class="utility-proof-strip" aria-label="Activity center FD-001 substrate proof">
                <article>
                    <p class="screen-anchor">"FD-001 OPERATIONS SIGNALS"</p>
                    <strong>"Notifications are workload control signals, not inbox noise"</strong>
                    <span data-activity-status="true">"Close, filing, vendor, and audit events are Oyatie Cloud tenant workload previews."</span>
                </article>
                <div class="utility-route-grid" aria-label="Activity center routes">
                    <button type="button" data-utility-route="work-hub"><span>"Comms"</span><strong>"Work Hub"</strong></button>
                    <button type="button" data-utility-route="evidence"><span>"Receipt"</span><strong>"Evidence spine"</strong></button>
                    <button type="button" data-utility-route="cloud"><span>"Substrate"</span><strong>"Cloud cells"</strong></button>
                </div>
            </section>
            <div class="utility-summary">
                    <span><strong data-activity-count="true">"3"</strong><small>"unread"</small></span>
                <span><strong>"12"</strong><small>"today"</small></span>
                <span><strong>"3"</strong><small>"blocking"</small></span>
            </div>
            <div class="utility-filter-row" role="toolbar" aria-label="Activity filters">
                <button type="button" class="active" data-activity-filter="all">"All"</button>
                <button type="button" data-activity-filter="unread">"Unread"</button>
                <button type="button" data-activity-filter="blocking">"Blocking"</button>
                <button type="button" data-activity-action="clear-read">"Clear read"</button>
            </div>
            <ol class="activity-list" data-activity-list="true" aria-live="polite">
                <li data-activity-item="true" data-activity-state="unread" data-activity-severity="blocking">
                    <time>"09:18"</time>
                    <span class="status-chip danger">"blocking"</span>
                    <strong>"4대보험 변동 확인 필요"</strong>
                    <p>"Payroll close cannot seal until Park Seo-jun's insurance delta is reviewed."</p>
                    <button type="button" data-activity-action="mark-read">"Mark read"</button>
                </li>
                <li data-activity-item="true" data-activity-state="unread" data-activity-severity="review">
                    <time>"09:42"</time>
                    <span class="status-chip warning">"review"</span>
                    <strong>"Withholding tax brief ready"</strong>
                    <p>"HomeTax transport is staged locally; reviewer must approve before send."</p>
                    <button type="button" data-activity-action="mark-read">"Mark read"</button>
                </li>
                <li data-activity-item="true" data-activity-state="unread" data-activity-severity="blocking">
                    <time>"10:05"</time>
                    <span class="status-chip danger">"vendor"</span>
                    <strong>"Stripe renewal needs owner"</strong>
                    <p>"Spend approval exceeds one-step threshold and requires CFO attestation."</p>
                    <button type="button" data-activity-action="mark-read">"Mark read"</button>
                </li>
                <li data-activity-item="true" data-activity-state="read" data-activity-severity="info">
                    <time>"10:21"</time>
                    <span class="status-chip success">"sealed"</span>
                    <strong>"Audit receipt staged"</strong>
                    <p>"REC-FIN-2026-05 was added to the local close package preview."</p>
                    <button type="button" data-activity-action="open-audit">"Open audit"</button>
                </li>
            </ol>
        </section>

        <section
            class="utility-panel settings-center"
            data-utility-panel="settings"
            aria-label="Workspace settings"
            aria-hidden="true"
        >
            <div class="utility-panel-head">
                <div>
                    <p class="screen-anchor">"SETTINGS"</p>
                    <h2>"Workspace, profile, appearance, and integrations"</h2>
                </div>
                <button type="button" data-utility-close="true" aria-label="Close settings">"×"</button>
            </div>
            <section class="utility-proof-strip settings-proof" aria-label="Settings FD-001 substrate proof">
                <article>
                    <p class="screen-anchor">"CONTROL PLANE SETTINGS"</p>
                    <strong>"Workspace preferences stay tied to FD-001, policy, and Oyatie Cloud posture"</strong>
                    <span>"Every preference is local visual state; no auth, IAM, billing, integration, mail, or cloud mutation occurs."</span>
                </article>
                <div class="utility-route-grid" aria-label="Settings connected routes">
                    <button type="button" data-utility-route="identity"><span>"Identity"</span><strong>"Role envelope"</strong></button>
                    <button type="button" data-utility-route="policy"><span>"Policy"</span><strong>"Access matrix"</strong></button>
                    <button type="button" data-utility-route="catalog"><span>"Catalog"</span><strong>"Tenant modules"</strong></button>
                </div>
            </section>
            <div class="settings-person-card">
                <span class="workspace-avatar" aria-hidden="true">"최"</span>
                <div><strong>"최유나 · Choi Yu-na"</strong><p>"Tenant admin · Finance owner · PIPA-safe transitional data"</p></div>
            </div>
            <div class="settings-tabs" role="tablist" aria-label="Settings panels">
                <button type="button" class="active" data-settings-tab="profile" role="tab" aria-selected="true">"Profile"</button>
                <button type="button" data-settings-tab="appearance" role="tab" aria-selected="false">"Appearance"</button>
                <button type="button" data-settings-tab="integrations" role="tab" aria-selected="false">"Integrations"</button>
                <button type="button" data-settings-tab="audit" role="tab" aria-selected="false">"Audit"</button>
            </div>
            <article class="settings-panel active" data-settings-panel="profile">
                <dl class="settings-kv">
                    <div><dt>"Workspace"</dt><dd>"Oyatie Corp. · 118 employees"</dd></div>
                    <div><dt>"Role"</dt><dd>"Admin · payroll close approver"</dd></div>
                    <div><dt>"Region pack"</dt><dd>"US/EU/KR · Korean payroll enabled"</dd></div>
                </dl>
                <button type="button" data-settings-action="open-identity">"Open identity profile"</button>
            </article>
            <article class="settings-panel" data-settings-panel="appearance">
                <p>"Adjust local visual density and shell language without changing server state."</p>
                <div class="settings-action-grid">
                    <button type="button" data-settings-action="density-comfortable">"Comfortable"</button>
                    <button type="button" data-settings-action="density-compact">"Compact"</button>
                    <button type="button" data-settings-action="locale-ko">"한국어 우선"</button>
                    <button type="button" data-settings-action="locale-en">"English labels"</button>
                </div>
            </article>
            <article class="settings-panel" data-settings-panel="integrations">
                <ol class="integration-list">
                    <li><strong>"Shinhan Bank"</strong><span class="status-chip success">"verified"</span><small>"Bank transport staged locally; no money movement."</small></li>
                    <li><strong>"HomeTax"</strong><span class="status-chip warning">"review"</span><small>"Filing transport waits for human attestation."</small></li>
                    <li><strong>"Google Workspace"</strong><span class="status-chip">"local"</span><small>"Mail and community previews only."</small></li>
                </ol>
            </article>
            <article class="settings-panel" data-settings-panel="audit">
                <ol class="activity-list compact">
                    <li><time>"09:14"</time><strong>"Settings drawer opened"</strong><p>"Local shell state only."</p></li>
                    <li><time>"09:18"</time><strong>"Density preference staged"</strong><p>"Stored in this browser session."</p></li>
                    <li><time>"09:42"</time><strong>"Identity panel linked"</strong><p>"No auth mutation."</p></li>
                </ol>
            </article>
            <p class="settings-status" data-settings-status="true">"Local settings ready · no backend persistence."</p>
        </section>
    }
}

#[component]
fn SidePeek() -> impl IntoView {
    view! {
        <aside
            class="side-peek"
            data-side-peek="true"
            aria-label="Object quick view"
            aria-hidden="true"
        >
            <div class="side-peek-head">
                <div>
                    <p class="screen-anchor">"OBJECT QUICK VIEW"</p>
                    <h2 data-sidepeek-title-target="true">"Network hot split"</h2>
                </div>
                <button type="button" data-sidepeek-close="true" aria-label="Close object quick view">"×"</button>
            </div>
            <div class="side-peek-body">
                <section class="quick-identity">
                    <span class="workspace-avatar" aria-hidden="true">"N"</span>
                    <div>
                        <strong data-sidepeek-id-target="true">"CHG-NTW-4182"</strong>
                        <p data-sidepeek-desc-target="true">"Tenant network split awaiting residency and rollback evidence."</p>
                    </div>
                </section>
                <dl class="peek-kv">
                    <div><dt>"Owner"</dt><dd data-sidepeek-owner-target="true">"Infrastructure operations"</dd></div>
                    <div><dt>"Risk"</dt><dd><span class="status-chip danger" data-sidepeek-risk-target="true">"High"</span></dd></div>
                    <div><dt>"SLA"</dt><dd data-sidepeek-sla-target="true">"4.0h target · +1.4h over"</dd></div>
                    <div><dt>"Execution"</dt><dd>"Visual-only until live integration"</dd></div>
                </dl>
                <section class="side-peek-proof" aria-label="FD-001 object proof">
                    <p class="screen-anchor">"OBJECT PROOF"</p>
                    <strong>"Selected objects resolve to FD-001 workload evidence on Oyatie Cloud"</strong>
                    <span data-sidepeek-status="true">"Inspector ready · REC-WF-7741 · cell-us-east-2 · local visual state only."</span>
                    <div class="side-peek-route-grid" aria-label="Object proof routes">
                        <button type="button" data-sidepeek-route="workload"><span>"Workload"</span><strong>"FD-001 graph"</strong></button>
                        <button type="button" data-sidepeek-route="cloud"><span>"Cloud"</span><strong>"cell-us-east-2"</strong></button>
                        <button type="button" data-sidepeek-route="evidence"><span>"Receipt"</span><strong>"REC-WF-7741"</strong></button>
                    </div>
                </section>
                <section>
                    <h3>"Evidence trail"</h3>
                    <ol class="peek-timeline">
                        <li><time>"09:18"</time><span>"Policy guardrail matched residency rule for FD-001 tenant workload."</span></li>
                        <li><time>"09:42"</time><span>"Oyatie Cloud rollback plan requested from network owner."</span></li>
                        <li><time>"10:05"</time><span>"Audit-chain receipt REC-WF-7741 drafted locally."</span></li>
                    </ol>
                </section>
                <section class="peek-actions" aria-label="Object actions">
                    <button type="button" data-sidepeek-action="assign-owner">"Assign owner"</button>
                    <button type="button" data-sidepeek-action="draft-note">"Draft note"</button>
                    <button type="button" class="primary" data-sidepeek-action="review-evidence">"Review evidence"</button>
                </section>
            </div>
        </aside>
    }
}

#[component]
pub fn DashboardIsland() -> impl IntoView {
    let initial_envelope = initial_envelope();
    let initial_node_id = initial_envelope
        .as_ref()
        .and_then(|envelope| envelope.workflow.nodes.first())
        .map(|node| node.id.clone())
        .unwrap_or_default();

    let (active_context, set_active_context) = signal(OperatorContext::TenantAdmin);
    let (selected_node_id, set_selected_node_id) = signal(initial_node_id);
    let (envelope, set_envelope) = signal(initial_envelope);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (active_surface, set_active_surface) = signal(ProductSurface::Workflow);
    let (workflow_tool, set_workflow_tool) = signal(WorkflowTool::Select);
    let (draft_node_count, set_draft_node_count) = signal(0_usize);
    let (selected_hub_index, set_selected_hub_index) = signal(0_usize);
    let (draft_body, set_draft_body) = signal(String::new());
    let (local_drafts, set_local_drafts) = signal(Vec::<LocalDraft>::new());

    #[cfg(target_arch = "wasm32")]
    if envelope.get_untracked().is_none() {
        request_render_envelope(
            OperatorContext::TenantAdmin,
            set_envelope,
            set_selected_node_id,
            set_loading,
            set_error,
        );
    }

    view! {
        <div
            class=move || if loading.get() { "dashboard-island loading" } else { "dashboard-island" }
            data-island="render-envelope-dashboard"
            aria-live="polite"
            aria-busy=move || loading.get()
        >
            <section class="context-switcher island-frame" aria-labelledby="context-title">
                <div>
                    <p class="eyebrow">"Context"</p>
                    <h2 id="context-title">"Switch render envelope"</h2>
                    <span class="island-label">"interactive island"</span>
                </div>
                <div class="context-grid" role="list" aria-label="Tenant and role contexts">
                    {OperatorContext::ALL.into_iter().map(|context| view! {
                        <button
                            type="button"
                            class=move || if active_context.get() == context { "context-card selected" } else { "context-card" }
                            aria-pressed=move || active_context.get() == context
                            on:click=move |_| {
                                set_active_context.set(context);
                                request_render_envelope(
                                    context,
                                    set_envelope,
                                    set_selected_node_id,
                                    set_loading,
                                    set_error,
                                );
                                set_active_surface.set(ProductSurface::Workflow);
                                set_workflow_tool.set(WorkflowTool::Select);
                                set_draft_node_count.set(0);
                                set_selected_hub_index.set(0);
                                set_draft_body.set(String::new());
                                set_local_drafts.set(Vec::new());
                            }
                        >
                            <span class="context-icon" aria-hidden="true">{context_icon(context)}</span>
                            <span class="context-label">{context.label()}</span>
                            <span class="context-role">{context.role()}</span>
                        </button>
                    }).collect_view()}
                </div>
            </section>

            {move || error.get().map(|message| view! {
                <p class="fetch-error" role="alert">{message}</p>
            })}

            {move || match envelope.get() {
                Some(envelope) => dashboard_view(
                    envelope,
                    selected_node_id.get(),
                    set_selected_node_id,
                    active_surface,
                    set_active_surface,
                    workflow_tool,
                    set_workflow_tool,
                    draft_node_count,
                    set_draft_node_count,
                    selected_hub_index,
                    set_selected_hub_index,
                    draft_body,
                    set_draft_body,
                    local_drafts,
                    set_local_drafts,
                ).into_any(),
                None => loading_state().into_any(),
            }}
        </div>
    }
}

fn initial_envelope() -> Option<TenantRenderEnvelope> {
    #[cfg(any(feature = "ssr", test))]
    {
        Some(server_derived_envelope(OperatorContext::TenantAdmin))
    }

    #[cfg(not(any(feature = "ssr", test)))]
    {
        None
    }
}

#[cfg(target_arch = "wasm32")]
fn request_render_envelope(
    context: OperatorContext,
    set_envelope: WriteSignal<Option<TenantRenderEnvelope>>,
    set_selected_node_id: WriteSignal<String>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::{JsFuture, spawn_local};

    set_loading.set(true);
    set_error.set(None);

    spawn_local(async move {
        let context_id = context.id().to_string();
        let result = async {
            let window = web_sys::window().ok_or_else(|| "window unavailable".to_string())?;
            let response_value = JsFuture::from(
                window.fetch_with_str(&format!("/api/render-envelope/{context_id}")),
            )
            .await
            .map_err(|_| "render-envelope request failed".to_string())?;
            let response = response_value
                .dyn_into::<web_sys::Response>()
                .map_err(|_| "render-envelope response was invalid".to_string())?;

            if !response.ok() {
                return Err(format!(
                    "render-envelope request returned HTTP {}",
                    response.status()
                ));
            }

            let text_promise = response
                .text()
                .map_err(|_| "render-envelope body was unavailable".to_string())?;
            let text_value = JsFuture::from(text_promise)
                .await
                .map_err(|_| "render-envelope body could not be read".to_string())?;
            let text = text_value
                .as_string()
                .ok_or_else(|| "render-envelope body was not text".to_string())?;

            serde_json::from_str::<TenantRenderEnvelope>(&text)
                .map_err(|error| format!("render-envelope JSON was invalid: {error}"))
        }
        .await;

        match result {
            Ok(envelope) => {
                let node_id = envelope
                    .workflow
                    .nodes
                    .first()
                    .map(|node| node.id.clone())
                    .unwrap_or_default();
                set_selected_node_id.set(node_id);
                set_envelope.set(Some(envelope));
            }
            Err(message) => set_error.set(Some(message)),
        }

        set_loading.set(false);
    });
}

#[cfg(all(not(target_arch = "wasm32"), any(feature = "ssr", test)))]
fn request_render_envelope(
    context: OperatorContext,
    set_envelope: WriteSignal<Option<TenantRenderEnvelope>>,
    set_selected_node_id: WriteSignal<String>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_loading.set(true);
    set_error.set(None);

    let envelope = server_derived_envelope(context);
    let node_id = envelope
        .workflow
        .nodes
        .first()
        .map(|node| node.id.clone())
        .unwrap_or_default();
    set_selected_node_id.set(node_id);
    set_envelope.set(Some(envelope));
    set_loading.set(false);
}

#[cfg(all(not(target_arch = "wasm32"), not(any(feature = "ssr", test))))]
fn request_render_envelope(
    _context: OperatorContext,
    _set_envelope: WriteSignal<Option<TenantRenderEnvelope>>,
    _set_selected_node_id: WriteSignal<String>,
    _set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_error.set(Some(
        "render envelope refresh requires the SSR dev server or the WASM island fetch path"
            .to_string(),
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Leptos shell view composes several reactive signals at the island boundary; refactoring into state bags would obscure the explicit transitional data flow."
)]
fn dashboard_view(
    envelope: TenantRenderEnvelope,
    selected_node_id: String,
    set_selected_node_id: WriteSignal<String>,
    active_surface: ReadSignal<ProductSurface>,
    set_active_surface: WriteSignal<ProductSurface>,
    workflow_tool: ReadSignal<WorkflowTool>,
    set_workflow_tool: WriteSignal<WorkflowTool>,
    draft_node_count: ReadSignal<usize>,
    set_draft_node_count: WriteSignal<usize>,
    selected_hub_index: ReadSignal<usize>,
    set_selected_hub_index: WriteSignal<usize>,
    draft_body: ReadSignal<String>,
    set_draft_body: WriteSignal<String>,
    local_drafts: ReadSignal<Vec<LocalDraft>>,
    set_local_drafts: WriteSignal<Vec<LocalDraft>>,
) -> impl IntoView {
    let display_nodes = workflow_display_nodes(&envelope.workflow.nodes, draft_node_count.get());
    let selected_node = selected_workflow_node(&display_nodes, &selected_node_id)
        .cloned()
        .or_else(|| display_nodes.first().cloned());
    let support_advisory_view = match envelope.support_advisory.clone() {
        Some(fixture) if support_advisory_is_visible(&envelope) => {
            support_advisory_case_console(fixture).into_any()
        }
        _ => ().into_any(),
    };

    view! {
        {surface_command_bar(active_surface, set_active_surface)}

        {product_activity_spine(envelope.product_activity.clone())}

        <section class="envelope-banner" aria-labelledby="envelope-title">
            <div>
                <p class="eyebrow">"Active context"</p>
                <h2 id="envelope-title">{envelope.tenant_name.clone()}</h2>
                <p>{envelope.role_name.clone()}</p>
            </div>
            <div class="envelope-detail">
                <span class="badge">{envelope.tenant_class.clone()}</span>
                <span class=if envelope.accreditation.healthcare_enabled { "badge success" } else { "badge warning" }>
                    {envelope.accreditation.label.clone()}
                </span>
                <p>{envelope.server_derivation_note.clone()}</p>
            </div>
        </section>

        {metric_grid(envelope.metrics.clone())}

        {command_shell_substrate()}

        {substrate_proof_command(envelope.clone())}

        {command_center_workbench(envelope.clone())}

        {tenant_rbac_board(envelope.clone())}

        {identity_workforce_service()}

        {finance_commercial_service()}

        {operator_intelligence_strip(envelope.clone())}

        {tenant_operations_cockpit(envelope.clone())}

        {control_plane_resource_explorer_operation_center()}

        {trust_center_portal()}

        {support_advisory_view}

        {audit_evidence_timeline_panel()}

        {resource_audit_console(envelope.clone())}

        <section class="dashboard-grid" aria-label="Personalized dashboard">
            {daily_execution_console(envelope.clone())}

            <section id="work-hub" class="panel communications-panel" aria-labelledby="messages-title">
                <PanelHeader eyebrow="Messenger · Mail · Community" title={"Work hub".to_string()} />
                {communication_hub(
                    envelope.messages.clone(),
                    envelope.community.clone(),
                    active_surface,
                    set_active_surface,
                    selected_hub_index,
                    set_selected_hub_index,
                    draft_body,
                    set_draft_body,
                    local_drafts,
                    set_local_drafts,
                )}
            </section>

            <section id="service-catalog" class="panel modules-panel catalog-workbench" aria-labelledby="modules-title">
                <div class="panel-header catalog-header">
                    <div>
                        <p class="eyebrow">"Service catalog"</p>
                        <h3 id="modules-title">"Permitted service graph"</h3>
                    </div>
                    <span class="catalog-live-chip">"local · visually interactive"</span>
                </div>
                {service_catalog_panel(envelope.modules.clone(), envelope.omitted_capability_note.clone())}
            </section>
        </section>

        <section class="studio-grid" aria-label="Workflow, ontology, and intelligence">
            {workflow_studio_panel(
                envelope.workflow.name.clone(),
                envelope.workflow.goal.clone(),
                display_nodes,
                selected_node,
                set_selected_node_id,
                workflow_tool,
                set_workflow_tool,
                draft_node_count,
                set_draft_node_count,
                set_active_surface,
            )}

            <section id="ontology-command-console" class="panel ontology-command-shell" aria-labelledby="ontology-title">
                <PanelHeader eyebrow="Ontology" title={"Tenant workload graph".to_string()} />
                {ontology_list(envelope.ontology.clone())}
            </section>

            <section id="intelligence-command-console" class="panel intelligence-command-shell" aria-labelledby="intelligence-title">
                <PanelHeader eyebrow="Intelligence" title={"Governed AI command".to_string()} />
                {suggestion_list(envelope.intelligence.clone())}
            </section>
        </section>
    }
}

fn support_advisory_is_visible(envelope: &TenantRenderEnvelope) -> bool {
    envelope.support_advisory.is_some()
        && envelope
            .modules
            .iter()
            .any(|module| module.name == "Customer Support")
}

fn loading_state() -> impl IntoView {
    view! {
        <section class="panel loading-panel" aria-label="Loading permitted dashboard">
            <p class="eyebrow">"Server render envelope"</p>
            <h2>"Loading permitted dashboard"</h2>
            <p>"Fetching only the modules and workflow state allowed for this tenant and role."</p>
        </section>
    }
}

#[component]
fn PanelHeader(eyebrow: &'static str, title: String) -> impl IntoView {
    view! {
        <div class="panel-header">
            <p class="eyebrow">{eyebrow}</p>
            <h3>{title}</h3>
        </div>
    }
}

fn surface_command_bar(
    active_surface: ReadSignal<ProductSurface>,
    set_active_surface: WriteSignal<ProductSurface>,
) -> impl IntoView {
    view! {
        <nav class="surface-command-bar" aria-label="Open built-in product surface">
            {ProductSurface::ALL.into_iter().map(|surface| view! {
                <a
                    href=surface.href()
                    class=move || {
                        if active_surface.get() == surface {
                            "surface-command active"
                        } else {
                            "surface-command"
                        }
                    }
                    on:click=move |_| set_active_surface.set(surface)
                >
                    <span>{surface.label()}</span>
                    <small>{surface.summary()}</small>
                </a>
            }).collect_view()}
        </nav>
    }
}

fn workflow_display_nodes(nodes: &[WorkflowNode], draft_node_count: usize) -> Vec<WorkflowNode> {
    let mut display_nodes = nodes.to_vec();
    for (id, label, kind, x, y, explanation) in [
        (
            "finance-review",
            "Finance sign-off",
            "Human",
            250,
            190,
            "Adds a finance owner checkpoint before any costly tenant operation proceeds.",
        ),
        (
            "mail-notice",
            "Mail approval brief",
            "Mail",
            445,
            190,
            "Drafts the formal approval mail with policy, cost, and audit receipt context.",
        ),
        (
            "messenger-ops",
            "Messenger ops update",
            "Messenger",
            640,
            190,
            "Posts a local-only operational summary into the governed Ops room preview.",
        ),
        (
            "community-post",
            "Community notice",
            "Community",
            250,
            300,
            "Stages a community update for the governance council with reviewer trace links.",
        ),
        (
            "receipt",
            "Immutable receipt",
            "Audit",
            640,
            300,
            "Seals a visual receipt preview that can be inspected from the evidence spine.",
        ),
    ] {
        if display_nodes.iter().all(|node| node.id != id) {
            display_nodes.push(WorkflowNode {
                id: id.to_string(),
                label: label.to_string(),
                kind: kind.to_string(),
                x,
                y,
                explanation: explanation.to_string(),
            });
        }
    }
    for index in 0..draft_node_count {
        display_nodes.push(WorkflowNode {
            id: format!("draft-block-{}", index + 1),
            label: format!("Draft block {}", index + 1),
            kind: "Local".to_string(),
            x: 110 + ((index as i32 % 4) * 165),
            y: 164 + ((index as i32 / 4) * 74),
            explanation: "Local visual-only block added in the shell. It is not yet wired to a backend or workflow engine.".to_string(),
        });
    }
    display_nodes
}

fn metric_grid(metrics: Vec<MetricCard>) -> impl IntoView {
    view! {
        <section class="metric-grid" aria-label="Dashboard metrics">
            {metrics.into_iter().map(|metric| view! {
                <article class="metric-card">
                    <p>{metric.label}</p>
                    <strong>{metric.value}</strong>
                    <span>{metric.detail}</span>
                </article>
            }).collect_view()}
        </section>
    }
}

fn product_activity_spine(spine: ProductActivitySpine) -> impl IntoView {
    let active_route = spine.active_route.clone();
    let active_label = spine
        .steps
        .iter()
        .find(|step| step.route_key == active_route)
        .map(|step| step.label.clone())
        .unwrap_or_else(|| active_route.clone());
    let route_steps = spine.steps.clone();
    let lane_steps = spine.steps.clone();

    view! {
        <section
            id="product-activity-spine"
            class="product-activity-spine panel"
            aria-labelledby="product-activity-title"
            data-product-activity-spine="true"
        >
            <div class="activity-spine-head">
                <div>
                    <p class="screen-anchor">"PRODUCT ACTIVITY SPINE"</p>
                    <h3 id="product-activity-title">"One operating model for FD-001 tenant workloads on Oyatie Cloud"</h3>
                    <span data-spine-active-context="true">{spine.active_context.clone()}</span>
                </div>
                <div class="activity-spine-proof">
                    <span data-spine-active-route="true">{active_label}</span>
                    <code data-spine-evidence-id="true">{spine.evidence_id.clone()}</code>
                    <strong data-global-activity-status="true">{spine.status_label.clone()}</strong>
                </div>
            </div>
            <div class="activity-spine-grid">
                <aside class="activity-route-column" aria-label="Cross-surface routes">
                    <p class="screen-anchor">"ROUTES"</p>
                    {route_steps.into_iter().map(|step| product_activity_route_button(step, active_route.clone())).collect_view()}
                </aside>
                <div class="activity-flow-lane" aria-label="FD-001 workload path">
                    {lane_steps.into_iter().map(|step| product_activity_step_card(step, active_route.clone())).collect_view()}
                </div>
                <aside class="activity-inspector-card" aria-label="Selected route inspector">
                    <p class="screen-anchor">"INSPECTOR"</p>
                    <h4 data-spine-inspector-title="true">"FD-001 graph · product substrate"</h4>
                    <p data-spine-inspector-body="true">"Service catalog, workflow, Messenger, Mail, Community, cloud posture, and evidence receipts are one cohesive local operating graph."</p>
                    <dl>
                        <div><dt>"Tenant"</dt><dd data-spine-inspector-tenant="true">{spine.active_context.clone()}</dd></div>
                        <div><dt>"Boundary"</dt><dd>"Visual-only · no backend write"</dd></div>
                        <div><dt>"Receipt"</dt><dd>{spine.evidence_id.clone()}</dd></div>
                    </dl>
                    <div class="activity-inspector-actions">
                        <button type="button" data-activity-route="workflow">"Open Workflow"</button>
                        <button type="button" data-activity-route="mail">"Mail brief"</button>
                        <button type="button" data-activity-route="evidence">"Evidence"</button>
                    </div>
                </aside>
            </div>
            <div class="activity-spine-statusbar" aria-label="Current local shell state">
                <span>"SSR shell"</span>
                <span>"Selective WASM islands"</span>
                <span>"Local-only actions"</span>
                <span data-spine-last-action="true">"Ready · route and inspector state will update visually"</span>
            </div>
        </section>
    }
}

fn product_activity_route_button(step: ProductActivityStep, active_route: String) -> impl IntoView {
    let selected = step.route_key == active_route;
    let route_key = step.route_key.clone();
    let target = step.target.clone();
    let label = step.label.clone();
    let label_attr = label.clone();
    let detail = step.detail.clone();
    let state = step.state.clone();
    let surface = step.surface.clone();

    view! {
        <button
            type="button"
            class=if selected { "selected" } else { "" }
            data-activity-route=route_key
            data-activity-target=target
            data-activity-label=label_attr
            data-activity-detail=detail
            data-activity-state=state
        >
            <strong>{label}</strong>
            <span>{surface}</span>
        </button>
    }
}

fn product_activity_step_card(step: ProductActivityStep, active_route: String) -> impl IntoView {
    let selected = step.route_key == active_route;
    let route_key = step.route_key.clone();
    let target = step.target.clone();
    let label = step.label.clone();
    let label_attr = label.clone();
    let detail = step.detail.clone();
    let detail_attr = detail.clone();
    let state = step.state.clone();
    let state_attr = state.clone();
    let surface = step.surface.clone();

    view! {
        <button
            type="button"
            class=if selected { "activity-step-card selected" } else { "activity-step-card" }
            data-activity-route=route_key.clone()
            data-activity-target=target
            data-activity-label=label_attr
            data-activity-detail=detail_attr
            data-activity-state=state_attr
            data-spine-step=route_key
        >
            <span>{surface}</span>
            <strong>{label}</strong>
            <small>{detail}</small>
            <em>{state}</em>
        </button>
    }
}

const COMMAND_SHELL_ROUTES: [(&str, &str, &str, &str); 8] = [
    ("fd001", "FD-001", "Service graph", "#service-catalog"),
    (
        "daily",
        "Action Inbox",
        "Priority work",
        "#command-center-workbench",
    ),
    ("workflow", "Workflow", "Studio IDE", "#workflow-studio"),
    (
        "messenger",
        "Comms",
        "Messenger/Mail/Community",
        "#work-hub",
    ),
    (
        "finance",
        "Finance",
        "Close + ledger",
        "#finance-commercial-service",
    ),
    (
        "identity",
        "Identity",
        "Org + access",
        "#identity-workforce-service",
    ),
    ("cloud", "Cloud", "Substrate cells", "#cloud-ops-cockpit"),
    (
        "evidence",
        "Evidence",
        "Audit receipts",
        "#resource-audit-console",
    ),
];

fn command_shell_substrate() -> impl IntoView {
    view! {
        <section
            id="command-shell-substrate"
            class="command-shell-substrate panel"
            aria-labelledby="command-shell-title"
            data-command-shell-substrate="true"
        >
            <div class="command-shell-copy">
                <p class="screen-anchor">"COMMAND SHELL SUBSTRATE"</p>
                <h3 id="command-shell-title">"Every lower panel inherits the same active route, tenant lens, and local boundary"</h3>
                <span data-command-shell-status="true">"FD-001 graph is active · lower surfaces will keep the route/status/inspector spine synchronized."</span>
            </div>
            <div class="command-shell-context" aria-live="polite">
                <span><small>"Active route"</small><strong data-command-shell-route="true">"FD-001 graph"</strong></span>
                <span><small>"Target"</small><strong data-command-shell-target="true">"#service-catalog"</strong></span>
                <span><small>"Updated"</small><strong data-command-shell-updated="true">"SSR render"</strong></span>
            </div>
            <div class="command-shell-routes" role="toolbar" aria-label="Lower dashboard product routes">
                {COMMAND_SHELL_ROUTES.into_iter().map(|(route, label, detail, target)| view! {
                    <button
                        type="button"
                        class=if route == "fd001" { "selected" } else { "" }
                        data-shell-context-route=route
                        data-shell-context-target=target
                    >
                        <strong>{label}</strong>
                        <span>{detail}</span>
                    </button>
                }).collect_view()}
            </div>
        </section>
    }
}

fn substrate_proof_command(envelope: TenantRenderEnvelope) -> impl IntoView {
    view! {
        <section id="substrate-proof" class="substrate-proof-command panel" data-substrate-proof="true" aria-labelledby="substrate-proof-title">
            <div class="substrate-proof-head">
                <div>
                    <p class="screen-anchor">"OYATIE CLOUD · FD-001 DOGFOOD SUBSTRATE"</p>
                    <h3 id="substrate-proof-title">"Prove production tenancy by running FD-001 as real tenant workloads"</h3>
                    <p>"FD-001 remains the product delivery goal. Oyatie Cloud is the hyperscaler-grade substrate proving those microservices can host production tenants before any external claim."</p>
                </div>
                <div class="substrate-proof-actions">
                    <span class="status-chip success" data-substrate-status="true">"12 workloads · 3 cells · 0 external writes"</span>
                    <button type="button" data-substrate-action="cloud">"Cloud cells"</button>
                    <button type="button" data-substrate-action="workflow">"Workflow proof"</button>
                    <button type="button" data-substrate-action="evidence">"Evidence"</button>
                </div>
            </div>

            <div class="substrate-proof-grid" aria-label="Substrate proof metrics">
                <article class="substrate-proof-card primary">
                    <p class="screen-anchor">"PRODUCT GOAL"</p>
                    <strong>"FD-001 delivery"</strong>
                    <span>"Core, workflow, messenger, mail, community, finance, identity, intelligence, and ontology run as tenant workload previews."</span>
                </article>
                <article class="substrate-proof-card">
                    <p class="screen-anchor">"SUBSTRATE"</p>
                    <strong>"Oyatie Cloud"</strong>
                    <span>"Cellular runtime, policy, FinOps, resource inventory, deployment gates, and rollback evidence."</span>
                </article>
                <article class="substrate-proof-card">
                    <p class="screen-anchor">"TENANT LENS"</p>
                    <strong>{envelope.tenant_name.clone()}</strong>
                    <span>{format!("{} · server-derived envelope · local dogfood only", envelope.role_name)}</span>
                </article>
                <article class="substrate-proof-card warning">
                    <p class="screen-anchor">"READINESS"</p>
                    <strong>"84% proof"</strong>
                    <span>"3 blockers: payroll delta, cloud rollback receipt, PIPA review."</span>
                </article>
            </div>

            <div class="substrate-workload-map" aria-label="FD-001 tenant workload deployment map">
                <div class="substrate-map-column substrate-product-column">
                    <p class="screen-anchor">"FD-001 WORKLOADS"</p>
                    <button type="button" data-substrate-action="workflow"><strong>"Workflow"</strong><span>"approval engine · no-code studio"</span></button>
                    <button type="button" data-substrate-action="messenger"><strong>"Messenger"</strong><span>"ops room thread · evidence extraction"</span></button>
                    <button type="button" data-substrate-action="mail"><strong>"Mail"</strong><span>"formal approval brief"</span></button>
                    <button type="button" data-substrate-action="community"><strong>"Community"</strong><span>"governance council post"</span></button>
                </div>
                <div class="substrate-map-spine" aria-hidden="true">
                    <span>"tenant workload"</span>
                    <i></i>
                    <span>"cell runtime"</span>
                    <i></i>
                    <span>"evidence receipt"</span>
                </div>
                <div class="substrate-map-column substrate-cloud-column">
                    <p class="screen-anchor">"OYATIE CLOUD CELLS"</p>
                    <button type="button" data-substrate-action="cloud"><strong>"cell-us-east-2"</strong><span>"primary · workload dogfood"</span></button>
                    <button type="button" data-substrate-action="finops"><strong>"kr-seoul-1"</strong><span>"localization pack · FinOps watch"</span></button>
                    <button type="button" data-substrate-action="deployment"><strong>"gitops promotion"</strong><span>"Jenkins · ArgoCD · cosign · audit"</span></button>
                    <button type="button" data-substrate-action="evidence"><strong>"evidence spine"</strong><span>"REC-FD001-CLOUD-009"</span></button>
                </div>
            </div>

            <div class="substrate-proof-footer" aria-label="Dogfood proof routes">
                <span>"Proof loop: tenant workload → Oyatie Cloud cell → policy gate → human route → evidence receipt"</span>
                <button type="button" data-substrate-action="finance">"Finance close"</button>
                <button type="button" data-substrate-action="identity">"Identity policy"</button>
                <button type="button" data-substrate-action="catalog">"Service catalog"</button>
            </div>
        </section>
    }
}

fn command_center_workbench(envelope: TenantRenderEnvelope) -> impl IntoView {
    let tasks = envelope.daily_tasks.clone();
    let approvals = envelope.approvals.clone();
    let suggestions = envelope.intelligence.clone();
    let workflow_name = envelope.workflow.name.clone();
    let role_name = envelope.role_name.clone();

    view! {
        <section
            id="command-center-workbench"
            class="command-center-workbench"
            aria-labelledby="command-workbench-title"
        >
            <article class="priority-workbench panel" aria-labelledby="command-workbench-title">
                <div class="workbench-head">
                    <div>
                        <p class="screen-anchor">"ACTION INBOX"</p>
                        <h3 id="command-workbench-title">"Priority queue" <span>"8"</span></h3>
                    </div>
                    <div class="workbench-filters" role="toolbar" aria-label="Action inbox filters">
                        <button type="button" class="active" data-workbench-filter="all">"All"</button>
                        <button type="button" data-workbench-filter="mine">"Mine"</button>
                        <button type="button" data-workbench-filter="blocking">"Blocking"</button>
                    </div>
                </div>

                <div class="workbench-summary-strip" aria-label="Action inbox summary">
                    <span><strong>"3"</strong><small>"blocking"</small></span>
                    <span><strong>"5"</strong><small>"owned by you"</small></span>
                    <span><strong>"4.0h"</strong><small>"SLA pressure"</small></span>
                    <span><strong>"12"</strong><small>"evidence links"</small></span>
                </div>

                {action_inbox_proof_board()}

                <div class="workbench-bulkbar" aria-label="Action inbox bulk operations">
                    <label>
                        <input type="checkbox" data-inbox-select-all="true" aria-label="Select all visible inbox items" />
                        <span><strong data-inbox-selected-count="true">"0"</strong>" selected"</span>
                    </label>
                    <div class="workbench-bulk-actions">
                        <button type="button" data-inbox-bulk="approve" disabled>"Approve"</button>
                        <button type="button" data-inbox-bulk="defer" disabled>"Defer"</button>
                        <button type="button" data-inbox-bulk="mail" disabled>"Mail brief"</button>
                        <button type="button" data-inbox-bulk="evidence" disabled>"Attach evidence"</button>
                    </div>
                    <span class="workbench-status" data-inbox-status="true">"No items selected · local inbox only"</span>
                </div>

                <div class="workbench-list" role="list" aria-label="Operational priority queue">
                    {tasks.into_iter().enumerate().map(|(index, item)| {
                        let title = item.title.clone();
                        let title_attr = title.clone();
                        let detail = item.detail.clone();
                        let detail_attr = detail.clone();
                        let priority = item.priority.clone();
                        let priority_label = priority.clone();
                        let priority_risk = priority.clone();
                        let priority_chip_class = if priority.eq_ignore_ascii_case("high") { "status-chip danger" } else { "status-chip" };
                        let priority_key = if priority.eq_ignore_ascii_case("high") {
                            "blocking"
                        } else if index % 2 == 0 {
                            "mine"
                        } else {
                            "all"
                        };
                        let row_class = if priority_key == "blocking" {
                            "workbench-row blocking"
                        } else {
                            "workbench-row"
                        };
                        let receipt = format!("REC-OYA-2026-05-{index:02}");
                        view! {
                            <article
                                class=row_class
                                data-workbench-row=priority_key
                                data-inbox-row="true"
                            >
                                <label class="inbox-select-cell">
                                    <input type="checkbox" data-inbox-select="true" data-inbox-title=title.clone() aria-label=format!("Select {title}") />
                                </label>
                                <span class="workbench-row-id">{format!("ACT-78{:02}", index + 41)}</span>
                                <span class=priority_chip_class>
                                    {priority_label}
                                </span>
                                <button
                                    type="button"
                                    class="inbox-row-main"
                                    data-sidepeek-trigger="action-inbox"
                                    data-sidepeek-title=title_attr
                                    data-sidepeek-id=receipt
                                    data-sidepeek-desc=detail_attr
                                    data-sidepeek-owner=role_name.clone()
                                    data-sidepeek-risk=priority_risk
                                    data-sidepeek-sla="4.0h target · local data"
                                >
                                    <strong>{title}</strong>
                                    <p>{detail}</p>
                                </button>
                                <time>{if index == 0 { "오늘 18:00" } else if index == 1 { "내일 09:00" } else { "5월 10일" }}</time>
                                <span class="inbox-row-actions">
                                    <button type="button" data-inbox-row-action="workflow">"Flow"</button>
                                    <button type="button" data-inbox-row-action="mail">"Mail"</button>
                                    <button type="button" data-inbox-row-action="audit">"Audit"</button>
                                </span>
                            </article>
                        }
                    }).collect_view()}
                    {approvals.into_iter().enumerate().map(|(index, item)| {
                        let title = item.title.clone();
                        let title_attr = title.clone();
                        let requester = item.requester.clone();
                        let requester_attr = requester.clone();
                        let requester_text = requester.clone();
                        let risk_note = item.risk_note.clone();
                        let risk_attr = risk_note.clone();
                        view! {
                            <article
                                class="workbench-row approval"
                                data-workbench-row="mine"
                                data-inbox-row="true"
                            >
                                <label class="inbox-select-cell">
                                    <input type="checkbox" data-inbox-select="true" data-inbox-title=title.clone() aria-label=format!("Select {title}") />
                                </label>
                                <span class="workbench-row-id">{format!("APR-{}", index + 274)}</span>
                                <span class="status-chip warning">"approval"</span>
                                <button
                                    type="button"
                                    class="inbox-row-main"
                                    data-sidepeek-trigger="approval"
                                    data-sidepeek-title=title_attr
                                    data-sidepeek-id=format!("APR-{}-{}", workflow_name.replace(' ', "-").to_ascii_uppercase(), index + 1)
                                    data-sidepeek-desc=risk_attr
                                    data-sidepeek-owner=requester_attr
                                    data-sidepeek-risk="Review"
                                    data-sidepeek-sla="Reviewer queue · visual only"
                                >
                                    <strong>{title}</strong>
                                    <p>{requester_text}" · "{risk_note}</p>
                                </button>
                                <time>"대기"</time>
                                <span class="inbox-row-actions">
                                    <button type="button" data-inbox-row-action="workflow">"Flow"</button>
                                    <button type="button" data-inbox-row-action="mail">"Mail"</button>
                                    <button type="button" data-inbox-row-action="audit">"Audit"</button>
                                </span>
                            </article>
                        }
                    }).collect_view()}
                </div>
            </article>

            <aside class="governed-copilot panel" aria-labelledby="copilot-workbench-title">
                <div class="copilot-head">
                    <div>
                        <p class="screen-anchor">"COPILOT · GOVERNED"</p>
                        <h3 id="copilot-workbench-title">"Suggested next moves"</h3>
                    </div>
                    <span class="status-chip ai">"PIPA-safe"</span>
                </div>
                <div class="copilot-suggestions" aria-live="polite">
                    {suggestions.into_iter().enumerate().map(|(index, suggestion)| {
                        let title = suggestion.title.clone();
                        let body = suggestion.body.clone();
                        let guardrail = suggestion.guardrail.clone();
                        view! {
                            <article class="copilot-card">
                                <strong>{title}</strong>
                                <p>{body}</p>
                                <small>{guardrail}</small>
                                <div>
                                    <button type="button" data-copilot-action="apply">"Apply delegation"</button>
                                    <button type="button" data-copilot-action="dismiss">"Dismiss"</button>
                                    <button type="button" data-sidepeek-trigger="copilot-trace" data-sidepeek-title="Copilot trace" data-sidepeek-id=format!("AI-TRACE-{index}") data-sidepeek-desc="Shows why a governed local suggestion is visible in this render envelope." data-sidepeek-owner="Governed Copilot" data-sidepeek-risk="Advisory" data-sidepeek-sla="Never auto-executes">"Trace"</button>
                                </div>
                            </article>
                        }
                    }).collect_view()}
                </div>
                <p class="copilot-status" data-copilot-status="true">
                    "Read-only · scoped to roster + run + workflow data · suggestions never auto-execute."
                </p>
            </aside>
        </section>
    }
}

fn action_inbox_proof_board() -> impl IntoView {
    view! {
        <section class="daily-proof-board inbox-proof-board" aria-label="FD-001 Action Inbox and Oyatie Cloud execution proof">
            <div class="daily-proof-grid">
                <article class="daily-proof-card selected" data-daily-proof-card="inbox-fd001">
                    <p class="screen-anchor">"FD-001 ACTION INBOX"</p>
                    <h5>"Priority queue is the product control plane"</h5>
                    <p>
                        "Blocking payroll, vendor, policy, Workflow, Mail, Messenger, Community, and evidence items stay inside the FD-001 tenant workload graph."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="route-daily">"Daily queue"</button>
                        <button type="button" data-daily-proof-action="route-workflow">"Workflow gate"</button>
                    </div>
                </article>
                <article class="daily-proof-card" data-daily-proof-card="inbox-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD ADMISSION"</p>
                    <h5>"Every item can prove tenant readiness"</h5>
                    <p>
                        "Oyatie Cloud substrate checks policy, residency, release gates, FinOps, and audit freshness before any FD-001 workload claims production readiness."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="route-cloud">"Cloud cells"</button>
                        <button type="button" data-daily-proof-action="route-policy">"Policy board"</button>
                    </div>
                </article>
                <article class="daily-proof-card" data-daily-proof-card="inbox-local">
                    <p class="screen-anchor">"LOCAL-ONLY REVIEW"</p>
                    <h5>"Interactive, never wired"</h5>
                    <p>
                        "Operators can select, defer, brief, and attach receipts visually; no approval, auth, workflow execution, mail send, payroll, billing, or cloud mutation runs."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="stage-packet">"Stage packet"</button>
                        <button type="button" data-daily-proof-action="route-audit">"Audit ledger"</button>
                    </div>
                </article>
            </div>
            <div class="daily-proof-footer">
                <span data-daily-proof-status="true">"Action Inbox ready · FD-001 priority work dogfoods Oyatie Cloud locally."</span>
                <div class="daily-proof-routes" aria-label="Action Inbox connected routes">
                    <button type="button" data-daily-proof-action="route-mail">"Reviewer Mail"</button>
                    <button type="button" data-daily-proof-action="route-community">"Community"</button>
                    <button type="button" data-daily-proof-action="route-evidence">"Evidence"</button>
                </div>
            </div>
        </section>
    }
}

fn tenant_rbac_board(envelope: TenantRenderEnvelope) -> impl IntoView {
    view! {
        <section
            class="tenant-rbac-board"
            aria-labelledby="service-board-title"
            id="business-logics"
        >
            <div class="service-board-head">
                <div>
                    <p class="screen-anchor">"TENANT RBAC SERVICE GRAPH"</p>
                    <h3 id="service-board-title">"Corporate operations graph"</h3>
                </div>
                <span class="status-chip success">{format!("{} permitted services", envelope.modules.len())}</span>
            </div>

            {business_logic_os_panel()}
            {governance_ops_command_board(envelope.workflow.name.clone())}

            <article id="payroll-cockpit" class="service-card payroll-card">
                <p class="screen-anchor">"PAYROLL CLOSE"</p>
                <h4>"2026-04 payroll close"</h4>
                <p class="service-card-brief">"FD-001 payroll workload dogfooded on Oyatie Cloud with workflow, Mail, and evidence return paths."</p>
                <div class="service-metric-row">
                    <span><strong>"73%"</strong><small>"close progress"</small></span>
                    <span><strong>"5.4d"</strong><small>"cycle time"</small></span>
                    <span><strong>"₩2.18M"</strong><small>"cost of delay"</small></span>
                </div>
                <ol class="service-checklist">
                    <li><span class="status-chip danger">"blocking"</span>"4대보험 변동 확인 필요"</li>
                    <li><span class="status-chip warning">"review"</span>"Payroll reminder mail draft ready"</li>
                    <li><span class="status-chip success">"sealed"</span>"Receipt REC-PAY-2026-04 staged"</li>
                </ol>
                <div class="service-card-actions">
                    <button type="button" data-service-action="payroll-finance">"Finance cockpit"</button>
                    <button type="button" data-service-action="payroll-workflow">"Workflow gate"</button>
                    <button type="button" data-service-action="payroll-mail">"Mail brief"</button>
                    <button type="button" data-service-action="payroll-evidence">"Evidence"</button>
                </div>
            </article>

            <article id="filing-readiness" class="service-card filing-card">
                <p class="screen-anchor">"FILING READINESS"</p>
                <h4>"Withholding return"</h4>
                <p class="service-card-brief">"Korea localization workload joins the same substrate proof loop: reviewer attestation, transport, and receipt."</p>
                <div class="readiness-bars compact" aria-label="Filing readiness">
                    <span style="--bar: 86%"><em>"Employee validation"</em></span>
                    <span style="--bar: 64%"><em>"HomeTax transport"</em></span>
                    <span style="--bar: 52%"><em>"Reviewer attestation"</em></span>
                </div>
                <div class="service-card-actions">
                    <button type="button" data-sidepeek-trigger="filing" data-sidepeek-title="Withholding return" data-sidepeek-id="FILE-KR-2026-04" data-sidepeek-desc="Filing readiness is staged locally and never submitted before live integration." data-sidepeek-owner="Finance close" data-sidepeek-risk="2 review" data-sidepeek-sla="Due 2026-05-10">"Inspect"</button>
                    <button type="button" data-service-action="filing-billing">"Billing · tax"</button>
                    <button type="button" data-service-action="filing-community">"Council note"</button>
                    <button type="button" data-service-action="filing-evidence">"Receipt"</button>
                </div>
                {filing_readiness_anchor_board()}
            </article>

            <article id="employee-directory" class="service-card employee-card">
                <p class="screen-anchor">"EMPLOYEES"</p>
                <h4>"Employee directory"</h4>
                <p class="service-card-brief">"Identity and workforce data remain role-visible while FD-001 tenant workloads prove policy envelopes."</p>
                <div class="service-people-stats">
                    <span><strong>"118"</strong><small>"employees"</small></span>
                    <span><strong>"109"</strong><small>"active"</small></span>
                    <span><strong>"5"</strong><small>"probation watch"</small></span>
                </div>
                <table class="employee-mini-table">
                    <thead><tr><th>"Name"</th><th>"Role"</th><th>"Team"</th><th>"Status"</th></tr></thead>
                    <tbody>
                        <tr><td>"이재현 Jaehyun Lee"</td><td>"CEO"</td><td>"Office"</td><td>"활성"</td></tr>
                        <tr><td>"최유나 Yuna Choi"</td><td>"CFO"</td><td>"Finance"</td><td>"활성"</td></tr>
                        <tr><td>"박서준 Seojun Park"</td><td>"VP Engineering"</td><td>"Infrastructure"</td><td>"활성"</td></tr>
                        <tr><td>"김지영 Jiyoung Kim"</td><td>"Manager"</td><td>"Infrastructure"</td><td>"수습"</td></tr>
                    </tbody>
                </table>
                <div class="service-card-actions">
                    <button type="button" data-service-action="employee-identity">"Identity service"</button>
                    <button type="button" data-service-action="employee-onboarding">"Onboarding"</button>
                    <button type="button" data-service-action="employee-policy">"Policy"</button>
                    <button type="button" data-service-action="employee-mail">"Mail reviewer"</button>
                </div>
            </article>

            <article id="governance-analytics-summary" class="service-card governance-card">
                <p class="screen-anchor">"GOVERNANCE ANALYTICS"</p>
                <h4>"Policy, receipts, workflow health"</h4>
                <p class="service-card-brief">"Executive posture rolls FD-001 workload proof, cloud cell evidence, and built-in surface routes into one council view."</p>
                <div class="service-graph" aria-hidden="true">
                    <span style="--bar: 78%"></span>
                    <span style="--bar: 48%"></span>
                    <span style="--bar: 92%"></span>
                    <span style="--bar: 61%"></span>
                    <span style="--bar: 72%"></span>
                </div>
                <dl class="service-kv">
                    <div><dt>"Risk"</dt><dd>"3 high-risk approvals"</dd></div>
                    <div><dt>"Evidence"</dt><dd>"12 sealed draft receipts"</dd></div>
                    <div><dt>"Workflow"</dt><dd>{envelope.workflow.name}</dd></div>
                </dl>
                <div class="service-card-actions">
                    <button type="button" data-service-action="governance-command">"Command board"</button>
                    <button type="button" data-service-action="governance-risk">"Risk heatmap"</button>
                    <button type="button" data-service-action="governance-community">"Community"</button>
                    <button type="button" data-service-action="governance-evidence">"Evidence"</button>
                </div>
            </article>
        </section>
    }
}

fn filing_readiness_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board filing-trust-board" aria-label="FD-001 filing readiness and Oyatie Cloud localization proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="filing-fd001">
                    <p class="screen-anchor">"FD-001 LOCALIZATION WORKLOAD"</p>
                    <h5>"Korea filing is tenant workload delivery, not a side widget"</h5>
                    <p>
                        "Withholding return, employee validation, HomeTax transport, billing, Mail, Community, and evidence receipts stay in the FD-001 tenant workload graph."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="stage-filing">"Stage filing"</button>
                        <button type="button" data-trust-proof-action="route-billing">"Billing · tax"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="filing-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD RESIDENCY"</p>
                    <h5>"Substrate proves regional pack posture"</h5>
                    <p>
                        "Oyatie Cloud shows PIPA-aware residency, policy envelope, release gates, audit freshness, and rollback posture before any tax workload readiness claim."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-policy">"PIPA policy"</button>
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud cells"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="filing-local">
                    <p class="screen-anchor">"LOCAL-ONLY FILING RAIL"</p>
                    <h5>"Reviewer-ready, never submitted"</h5>
                    <p>
                        "Operators can inspect readiness, stage a reviewer packet, and route council notes visually; no HomeTax, bank, payroll, billing, mail, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-mail">"Reviewer Mail"</button>
                        <button type="button" data-trust-proof-action="route-evidence">"Receipt spine"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Filing readiness ready · FD-001 localization workload dogfoods Oyatie Cloud with local-only submission controls."
                </span>
                <div class="trust-anchor-routes" aria-label="Filing readiness connected routes">
                    <button type="button" data-trust-proof-action="route-finance">"Finance close"</button>
                    <button type="button" data-trust-proof-action="route-community">"Community note"</button>
                    <button type="button" data-trust-proof-action="route-audit">"Audit ledger"</button>
                    <button type="button" data-trust-proof-action="route-catalog">"Catalog"</button>
                </div>
            </div>
        </div>
    }
}

fn governance_ops_command_board(workflow_name: String) -> impl IntoView {
    view! {
        <section id="governance-analytics" class="governance-ops-command" aria-labelledby="governance-ops-title">
            <div class="governance-command-head">
                <div>
                    <p class="screen-anchor">"00 / GOVERNANCE ANALYTICS"</p>
                    <h4 id="governance-ops-title">"Policy, evidence, risk, and business logic control"</h4>
                    <p>"Bominal-grade executive governance: posture, risk heatmap, control attestations, compliance calendar, decision queue, evidence chain, and routes into every built-in surface."</p>
                </div>
                <div class="governance-command-actions">
                    <span class="status-chip warning" data-governance-status="true">"84 posture · 5 top risks · local only"</span>
                    <button type="button" data-governance-action="run-review">"Run review"</button>
                    <button type="button" data-governance-action="seal-brief" data-governance-route="evidence">"Seal brief"</button>
                    <button type="button" data-governance-action="route-inbox" data-governance-route="inbox">"Open queue"</button>
                </div>
            </div>

            <div class="governance-posture-strip" aria-label="Governance posture overview">
                <article class="gov-posture-score">
                    <span>"Composite posture"</span>
                    <strong>"84"</strong>
                    <em>"A− · +3 vs last quarter · target 90"</em>
                </article>
                <article class="gov-posture-trend">
                    <div><span>"13 week trend"</span><strong>"78 → 84"</strong></div>
                    <svg viewBox="0 0 160 42" aria-hidden="true" class="gov-sparkline">
                        <polyline points="0,34 14,32 28,32 42,29 56,29 70,27 84,29 98,26 112,23 126,23 140,18 154,16" />
                    </svg>
                </article>
                <div class="gov-pillar-grid" aria-label="Governance pillars">
                    <button type="button" data-governance-action="pillar-compliance" data-governance-route="finance"><span>"Compliance"</span><strong>"87"</strong><em>"24 controls · 1 breach"</em></button>
                    <button type="button" data-governance-action="pillar-financial" data-governance-route="finance"><span>"Financial controls"</span><strong>"91"</strong><em>"manual JE 4.2%"</em></button>
                    <button type="button" data-governance-action="pillar-workforce" data-governance-route="identity"><span>"Workforce risk"</span><strong>"71"</strong><em>"§53 watch"</em></button>
                    <button type="button" data-governance-action="pillar-disclosure" data-governance-route="community"><span>"Board + disclosure"</span><strong>"88"</strong><em>"pack ships May 12"</em></button>
                </div>
            </div>

            <div class="governance-command-grid">
                <article class="governance-command-card policy-gate-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"POLICY GATES"</p>
                            <h5>"Decision rights before risky operation"</h5>
                        </div>
                        <span class="status-chip danger">"2 blocks"</span>
                    </div>
                    <div class="policy-gate-list" aria-label="Governance policy gates">
                        <button type="button" class="active" data-governance-action="select-payroll" data-governance-route="workflow"><span>"P0"</span><strong>"Payroll close"</strong><em>"2-person CFO signoff"</em></button>
                        <button type="button" data-governance-action="select-hometax" data-governance-route="finance"><span>"P0"</span><strong>"HomeTax filing"</strong><em>"사업자등록번호 confirmation"</em></button>
                        <button type="button" data-governance-action="select-cloud" data-governance-route="cloud"><span>"P0"</span><strong>"Network split"</strong><em>"rollback evidence required"</em></button>
                        <button type="button" data-governance-action="select-pipa" data-governance-route="identity"><span>"P1"</span><strong>"PIPA boundary"</strong><em>"vendor cannot view employee PII"</em></button>
                    </div>
                </article>

                <article class="governance-command-card decision-queue-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"EXEC APPROVALS"</p>
                            <h5>"Owners, SLA, origin, and route"</h5>
                        </div>
                        <button type="button" data-governance-action="open-inbox" data-governance-route="inbox">"Inbox"</button>
                    </div>
                    <table class="governance-decision-table">
                        <thead><tr><th>"Decision"</th><th>"Owner"</th><th>"SLA"</th><th>"State"</th></tr></thead>
                        <tbody>
                            <tr><td><strong>"Park 4대보험 delta"</strong><small>"REC-PAY-2026-04-PARK"</small></td><td>"CFO"</td><td>"4.0h"</td><td><span class="status-chip danger">"blocking"</span></td></tr>
                            <tr><td><strong>"Stripe approval compression"</strong><small>"policy ≤ ₩5M route"</small></td><td>"AP"</td><td>"1d"</td><td><span class="status-chip warning">"review"</span></td></tr>
                            <tr><td><strong>"Governance council note"</strong><small>"Community + Mail packet"</small></td><td>"Gov"</td><td>"today"</td><td><span class="status-chip success">"ready"</span></td></tr>
                            <tr><td><strong>"Board option pool"</strong><small>"resolution pack May 12"</small></td><td>"CEO"</td><td>"6d"</td><td><span class="status-chip">"scheduled"</span></td></tr>
                        </tbody>
                    </table>
                </article>

                <article class="governance-command-card risk-matrix-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"RISK MATRIX"</p>
                            <h5>"Audit committee 5×5 heatmap"</h5>
                        </div>
                        <span class="status-chip ai">"interactive"</span>
                    </div>
                    <div class="gov-risk-workspace">
                        <div class="gov-risk-heatmap" aria-label="Selectable risk matrix">
                            <span class="tone-minimal"></span><span class="tone-minimal"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span>
                            <span class="tone-minimal"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span>
                            <span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span>
                            <span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-extreme"></span>
                            <span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-extreme"></span><span class="tone-extreme"></span>
                            <button type="button" class="gov-risk-pin selected" style="--x: 78%; --y: 42%" data-governance-risk="RISK-042" data-risk-title="AI agent governance" data-risk-detail="Auto-delegation rollout needs change-management, rollback evidence, and human signoff." data-risk-owner="EMP-104" data-risk-score="4×3 High">"AI"</button>
                            <button type="button" class="gov-risk-pin" style="--x: 58%; --y: 30%" data-governance-risk="RISK-014" data-risk-title="LSA §53 weekly-hour breach" data-risk-detail="Yoon Tae-min projected 49.5h; automatic reassignment lowers residual risk." data-risk-owner="EMP-211" data-risk-score="3×4 Moderate">"53"</button>
                            <button type="button" class="gov-risk-pin" style="--x: 40%; --y: 34%" data-governance-risk="RISK-009" data-risk-title="PIPA retention overrun" data-risk-detail="43 medical certificates expire in 14 days; consent renewal or purge decision required." data-risk-owner="EMP-274" data-risk-score="2×4 Moderate">"PI"</button>
                            <button type="button" class="gov-risk-pin" style="--x: 57%; --y: 54%" data-governance-risk="RISK-031" data-risk-title="JE four-eyes gap" data-risk-detail="7 manual journal entries posted with a single approver; enforcement pending." data-risk-owner="EMP-188" data-risk-score="3×3 Moderate">"JE"</button>
                            <button type="button" class="gov-risk-pin" style="--x: 21%; --y: 20%" data-governance-risk="RISK-018" data-risk-title="Missed board resolution" data-risk-detail="Option pool expansion requires May 12 board resolution and quorum confirmation." data-risk-owner="EMP-274" data-risk-score="1×5 Low">"BD"</button>
                        </div>
                        <aside class="gov-risk-peek" aria-live="polite">
                            <div><span data-risk-peek-id="true">"RISK-042"</span><strong data-risk-peek-score="true">"4×3 High"</strong></div>
                            <h6 data-risk-peek-title="true">"AI agent governance"</h6>
                            <p data-risk-peek-detail="true">"Auto-delegation rollout needs change-management, rollback evidence, and human signoff."</p>
                            <dl><div><dt>"Owner"</dt><dd data-risk-peek-owner="true">"EMP-104"</dd></div><div><dt>"Next review"</dt><dd>"2026-05-08"</dd></div></dl>
                        </aside>
                    </div>
                </article>

                <article class="governance-command-card compliance-calendar-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"COMPLIANCE CALENDAR"</p>
                            <h5>"12-month filing commitments"</h5>
                        </div>
                        <button type="button" data-governance-action="calendar-review" data-governance-route="finance">"Review"</button>
                    </div>
                    <div class="gov-calendar" aria-label="Compliance calendar">
                        <span class="cal-corner">"Family"</span><span class="cal-month now">"May"</span><span class="cal-month">"Jun"</span><span class="cal-month">"Jul"</span><span class="cal-month">"Aug"</span><span class="cal-month">"Sep"</span><span class="cal-month">"Oct"</span><span class="cal-month">"Nov"</span><span class="cal-month">"Dec"</span><span class="cal-month">"Jan"</span><span class="cal-month">"Feb"</span><span class="cal-month">"Mar"</span><span class="cal-month">"Apr"</span>
                        <strong>"Withholding"</strong><button type="button" class="ready" data-gov-calendar-cell="Withholding May ready">"10 ₩38.2M"</button><button type="button" class="pending" data-gov-calendar-cell="Withholding Jun pending">"10 ₩39.1M"</button><button type="button" class="pending" data-gov-calendar-cell="Withholding Jul pending">"10 ₩40.0M"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button>
                        <strong>"4대보험"</strong><button type="button" class="review" data-gov-calendar-cell="Social insurance May review">"10 ₩57.0M"</button><button type="button" class="pending" data-gov-calendar-cell="Social insurance Jun pending">"10 ₩58.2M"</button><button type="button" class="pending" data-gov-calendar-cell="Social insurance Jul pending">"10 ₩59.1M"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button>
                        <strong>"VAT"</strong><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q2 prelim pending">"25 Q2 prelim"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q3 final pending">"25 Q3 final"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q4 prelim pending">"25 Q4 prelim"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q1 final pending">"25 Q1 final"</button>
                    </div>
                </article>

                <article class="governance-command-card evidence-readiness-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"EVIDENCE READINESS"</p>
                            <h5>"Receipts that prove the graph"</h5>
                        </div>
                        <span class="status-chip success">"sealed draft"</span>
                    </div>
                    <div class="evidence-readiness-lanes" aria-label="Evidence readiness lanes">
                        <span style="--bar: 92%"><strong>"Workflow receipts"</strong><em>"11 / 12"</em></span>
                        <span style="--bar: 78%"><strong>"Mail approvals"</strong><em>"7 linked"</em></span>
                        <span style="--bar: 64%"><strong>"Cloud runbooks"</strong><em>"2 waiting"</em></span>
                        <span style="--bar: 86%"><strong>"PIPA audit"</strong><em>"vendor gated"</em></span>
                    </div>
                </article>

                <article class="governance-command-card graph-route-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"ROUTE MATRIX"</p>
                            <h5>"Open connected product surface"</h5>
                        </div>
                    </div>
                    <div class="governance-route-grid" aria-label="Governance route matrix">
                        <button type="button" data-governance-action="route-workflow" data-governance-route="workflow">"Workflow"</button>
                        <button type="button" data-governance-action="route-mail" data-governance-route="mail">"Mail"</button>
                        <button type="button" data-governance-action="route-community" data-governance-route="community">"Community"</button>
                        <button type="button" data-governance-action="route-finance" data-governance-route="finance">"Finance"</button>
                        <button type="button" data-governance-action="route-cloud" data-governance-route="cloud">"Cloud Ops"</button>
                        <button type="button" data-governance-action="route-identity" data-governance-route="identity">"Identity"</button>
                        <button type="button" data-governance-action="route-evidence" data-governance-route="evidence">"Evidence"</button>
                        <button type="button" data-governance-action="route-catalog" data-governance-route="catalog">"Catalog"</button>
                    </div>
                    <dl class="governance-kv">
                        <div><dt>"Graph root"</dt><dd>{workflow_name}</dd></div>
                        <div><dt>"Autonomy ceiling"</dt><dd>"No auto-approval · visual review only"</dd></div>
                    </dl>
                </article>
            </div>

            <div class="governance-lower-deck">
                <article class="governance-command-card control-attestation-card">
                    <div class="governance-card-head"><div><p class="screen-anchor">"CONTROL ATTESTATION"</p><h5>"Controls with overdue evidence"</h5></div><span class="status-chip warning">"3 due"</span></div>
                    <div class="gov-attestation-list">
                        <button type="button" data-governance-action="attest-payroll" data-governance-route="workflow"><strong>"CTRL-PAY-09 · payroll 4-eyes"</strong><span style="--bar: 82%"></span><em>"82% · CFO attestation waiting"</em></button>
                        <button type="button" data-governance-action="attest-pipa" data-governance-route="identity"><strong>"CTRL-PIPA-03 · retention boundary"</strong><span style="--bar: 68%"></span><em>"68% · vendor visibility review"</em></button>
                        <button type="button" data-governance-action="attest-cloud" data-governance-route="cloud"><strong>"CTRL-CLOUD-12 · rollback runbook"</strong><span style="--bar: 74%"></span><em>"74% · regional evidence stale"</em></button>
                    </div>
                </article>
                <article class="governance-command-card audit-chain-card">
                    <div class="governance-card-head"><div><p class="screen-anchor">"AUDIT CHAIN"</p><h5>"Immutable receipt lineage"</h5></div><button type="button" data-governance-action="route-evidence" data-governance-route="evidence">"Evidence"</button></div>
                    <ol class="gov-audit-chain">
                        <li><span>"01"</span><strong>"Workflow run"</strong><em>"hash 5bf7…91"</em></li>
                        <li><span>"02"</span><strong>"Mail approval"</strong><em>"hash a81d…0c"</em></li>
                        <li><span>"03"</span><strong>"Community note"</strong><em>"hash 44c2…bf"</em></li>
                        <li><span>"04"</span><strong>"Board packet"</strong><em>"draft"</em></li>
                    </ol>
                </article>
                <article class="governance-command-card board-cycle-card">
                    <div class="governance-card-head"><div><p class="screen-anchor">"BOARD CYCLE"</p><h5>"Resolution timeline and quorum"</h5></div><span class="status-chip success">"3 / 5 ack"</span></div>
                    <div class="gov-board-timeline" aria-label="Board cycle timeline">
                        <span class="done"><strong>"Draft"</strong><em>"May 04"</em></span>
                        <span class="active"><strong>"Review"</strong><em>"May 07"</em></span>
                        <span><strong>"Send"</strong><em>"May 10"</em></span>
                        <span><strong>"Vote"</strong><em>"May 12"</em></span>
                    </div>
                </article>
            </div>
        </section>
    }
}

fn business_logic_os_panel() -> impl IntoView {
    view! {
        <section class="business-logic-os" aria-label="Business logic operating system">
            <div class="logic-os-kpis" aria-label="Business logic summary">
                <article class="logic-os-kpi"><span>"Active logics"</span><strong>"17"</strong><small>"7 visible in this envelope"</small></article>
                <article class="logic-os-kpi"><span>"P0 critical"</span><strong>"4"</strong><small>"cannot fail silently"</small></article>
                <article class="logic-os-kpi warn"><span>"Need attention"</span><strong>"3"</strong><small>"blocked or at-risk"</small></article>
                <article class="logic-os-kpi accent"><span>"Real cost / month"</span><strong>"₩9.2M"</strong><small>"hard + soft + delay"</small></article>
                <article class="logic-os-kpi"><span>"Annualized"</span><strong>"₩110M"</strong><small>"if cadence holds"</small></article>
            </div>

            <div class="logic-os-toolbar" aria-label="Business logic filters">
                <label class="logic-os-search">
                    <span aria-hidden="true">"⌕"</span>
                    <input data-logic-search="true" type="search" aria-label="Search business logics" placeholder="Search logic, owner, route, evidence..." />
                </label>
                <div class="logic-os-segments" role="toolbar" aria-label="Logic category filters">
                    <button type="button" class="active" data-logic-filter="all">"All"</button>
                    <button type="button" data-logic-filter="workforce">"Workforce"</button>
                    <button type="button" data-logic-filter="finance">"Finance"</button>
                    <button type="button" data-logic-filter="compliance">"Compliance"</button>
                    <button type="button" data-logic-filter="trust">"Trust"</button>
                    <button type="button" data-logic-filter="cloud">"Cloud"</button>
                    <button type="button" data-logic-filter="attention">"Attention"</button>
                </div>
                <span class="logic-os-status" data-logic-status="true">
                    <strong data-logic-visible-count="true">{BUSINESS_LOGIC_ROWS.len()}</strong>
                    " visible · all categories · local only"
                </span>
            </div>

            <div class="logic-os-layout">
                <div class="logic-table-shell" role="region" aria-label="Business logic catalog">
                    <table class="logic-os-table">
                        <thead>
                            <tr>
                                <th>"Health"</th>
                                <th>"Logic"</th>
                                <th>"Category"</th>
                                <th>"Owner"</th>
                                <th>"Cadence"</th>
                                <th>"Crit."</th>
                                <th>"Cost/run"</th>
                                <th>"SLA"</th>
                                <th>"Tasks"</th>
                                <th>"Action"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {BUSINESS_LOGIC_ROWS.into_iter().map(business_logic_row_view).collect_view()}
                        </tbody>
                    </table>
                </div>

                <aside class="logic-os-rail" aria-label="Business logic dependency and evidence rail">
                    <div class="logic-rail-card">
                        <p class="screen-anchor">"DEPENDENCY MAP"</p>
                        <strong>"Payroll anomaly → Workflow → Messenger/Mail → Audit"</strong>
                        <div class="logic-dependency-map" aria-hidden="true">
                            <span>"HR"</span><i></i><span>"Payroll"</span><i></i><span>"Workflow"</span><i></i><span>"Mail"</span><i></i><span>"Audit"</span>
                        </div>
                        <div class="logic-rail-actions">
                            <button type="button" data-logic-graph-action="workflow">"Workflow"</button>
                            <button type="button" data-logic-graph-action="mail">"Mail brief"</button>
                            <button type="button" data-logic-graph-action="catalog">"Catalog"</button>
                            <button type="button" data-logic-graph-action="audit">"Evidence"</button>
                        </div>
                    </div>
                    <div class="logic-rail-card matrix">
                        <p class="screen-anchor">"COST × HEALTH"</p>
                        <div class="logic-matrix" aria-label="Cost by health preview">
                            <span class="dot danger" style="--x: 78%; --y: 20%" title="Tenant network split"></span>
                            <span class="dot warn" style="--x: 54%; --y: 38%" title="Payroll close"></span>
                            <span class="dot warn" style="--x: 42%; --y: 52%" title="Vendor renewal"></span>
                            <span class="dot ok" style="--x: 24%; --y: 70%" title="New hire onboarding"></span>
                            <span class="dot done" style="--x: 12%; --y: 84%" title="Governance council note"></span>
                        </div>
                    </div>
                </aside>
            </div>
        </section>
    }
}

fn business_logic_row_view(row: BusinessLogicRow) -> impl IntoView {
    let state_class = format!("logic-health-dot {}", row.state);
    let criticality_class = format!("crit crit-{}", row.criticality);
    let task_class = if row.tasks == "0" {
        "tasks-cell"
    } else {
        "tasks-cell has-open"
    };
    view! {
        <tr
            class="logic-os-row"
            data-logic-row="true"
            data-logic-category=row.category
            data-logic-state=row.state
        >
            <td><span class=state_class aria-label=row.state_label></span></td>
            <td>
                <button
                    type="button"
                    class="logic-name-button"
                    data-sidepeek-trigger="business-logic"
                    data-sidepeek-title=row.name
                    data-sidepeek-id=row.id
                    data-sidepeek-desc=row.description
                    data-sidepeek-owner=row.owner
                    data-sidepeek-risk=row.state_label
                    data-sidepeek-sla=row.sla
                >
                    {row.name}
                </button>
                <div class="logic-code">{row.id}" · "{row.english_name}</div>
            </td>
            <td><span class="cat-tag">{row.category}</span></td>
            <td><span class="owner-cell"><span class="avatar-xs" aria-hidden="true">{logic_owner_initials(row.owner)}</span>{row.owner}</span></td>
            <td class="mono-cell">{row.cadence}</td>
            <td><span class=criticality_class>{row.criticality}</span></td>
            <td class="cost-cell">{row.cost}</td>
            <td><span class={logic_sla_class(row.state)}>{row.sla}</span></td>
            <td><span class=task_class>{row.tasks}</span></td>
            <td>
                <span class="logic-row-actions">
                    <button type="button" data-logic-action="open" data-logic-target=row.route>"Open"</button>
                    <button type="button" data-logic-action="run">"Run preview"</button>
                </span>
            </td>
        </tr>
    }
}

fn logic_owner_initials(owner: &str) -> &'static str {
    match owner {
        "Finance + HR" => "FH",
        "Tax operations" => "TO",
        "CFO office" => "CF",
        "Security reviewer" => "SR",
        "People ops" => "PO",
        "Infrastructure ops" => "PL",
        "Governance" => "GV",
        _ => "LO",
    }
}

fn logic_sla_class(state: &str) -> &'static str {
    match state {
        "blocked" | "at-risk" => "sla-cell over",
        "attention" | "review" => "sla-cell near",
        _ => "sla-cell ok",
    }
}

fn identity_workforce_service() -> impl IntoView {
    view! {
        <section
            id="identity-workforce-service"
            class="identity-workforce-service panel"
            data-identity-service="true"
            aria-labelledby="identity-service-title"
        >
            <div class="identity-service-head">
                <div>
                    <p class="screen-anchor">"SETTINGS · WORKFORCE"</p>
                    <h3 id="identity-service-title">"Identity, organization profile, onboarding, and employees"</h3>
                </div>
                <div class="identity-service-actions">
                    <span class="status-chip success" data-identity-status="true">"local profile ready"</span>
                    <button type="button" data-identity-action="open-audit">"Audit log"</button>
                </div>
            </div>

            <div class="identity-service-shell">
                <aside class="identity-settings-rail" aria-label="Identity and organization sections">
                    <div class="identity-person">
                        <span aria-hidden="true">"최"</span>
                        <div><strong>"최유나 · Choi Yu-na"</strong><small>"Admin · Oyatie Corp."</small></div>
                    </div>
                    <p class="screen-anchor">"ACCOUNT"</p>
                    <button type="button" class="active" data-identity-tab="auth">"패스키 · MFA"</button>
                    <button type="button" data-identity-tab="sessions">"세션 · 기기"</button>
                    <button type="button" data-identity-tab="roles">"역할 · 권한"</button>
                    <p class="screen-anchor">"WORKSPACE"</p>
                    <button type="button" data-identity-tab="org">"조직 프로필"</button>
                    <button type="button" data-identity-tab="employees">"구성원"</button>
                    <button type="button" data-identity-tab="onboarding">"워크스페이스 설정"</button>
                    <p class="identity-chain">"Oyatie v0.1 · chain 0x4f81 · last sync 09:14 KST"</p>
                </aside>

                <div class="identity-service-main">
                    <div class="identity-tabs" role="tablist" aria-label="Identity service views">
                        <button type="button" class="active" data-identity-tab="auth" role="tab" aria-selected="true">"Auth"</button>
                        <button type="button" data-identity-tab="sessions" role="tab" aria-selected="false">"Sessions"</button>
                        <button type="button" data-identity-tab="roles" role="tab" aria-selected="false">"Roles"</button>
                        <button type="button" data-identity-tab="org" role="tab" aria-selected="false">"Org profile"</button>
                        <button type="button" data-identity-tab="employees" role="tab" aria-selected="false">"Employees"</button>
                        <button type="button" data-identity-tab="onboarding" role="tab" aria-selected="false">"Onboarding"</button>
                    </div>

                    <article id="identity-auth" class="identity-panel active" data-identity-panel="auth" aria-labelledby="auth-panel-title">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"ACCOUNT · PASSKEYS"</p>
                            <h4 id="auth-panel-title">"패스키 · MFA"</h4>
                            <p>"가능한 모든 기기에 패스키를 등록하면 더 빠르고 안전하게 로그인합니다."</p>
                        </div>
                        <div class="auth-grid">
                            <div class="auth-method-list" data-passkey-list="true">
                                <div class="auth-method"><span>"⌘"</span><strong>"MacBook Pro · Touch ID"</strong><small>"passkey · macOS · Apple · PRIMARY"</small><em>"방금 전"</em></div>
                                <div class="auth-method"><span>"◉"</span><strong>"iPhone 15 Pro · Face ID"</strong><small>"passkey · iOS · Apple"</small><em>"12 hours ago"</em></div>
                                <div class="auth-method"><span>"▣"</span><strong>"Authy · personal phone"</strong><small>"totp · added 2025-11-03"</small><em>"3 weeks ago"</em></div>
                                <div class="auth-method"><span>"⌁"</span><strong>"Recovery codes (10)"</strong><small>"recovery · printed · 10 unused"</small><em>"never"</em></div>
                            </div>
                            <aside class="security-score-card">
                                <p class="screen-anchor">"SECURITY SCORE"</p>
                                <strong data-security-score="true">"94/100"</strong>
                                <span class="score-bar" style="--bar: 94%"><em></em></span>
                                <ol>
                                    <li>"✓ 패스키 2개 등록됨"</li>
                                    <li>"✓ TOTP 백업 활성화"</li>
                                    <li>"✓ 복구 코드 미사용"</li>
                                    <li>"○ Apple Watch 패스키 미등록"</li>
                                </ol>
                                <button type="button" data-identity-action="add-passkey">"+ 패스키 추가"</button>
                            </aside>
                        </div>
                        {identity_command_board()}
                    </article>

                    <article id="identity-sessions" class="identity-panel" data-identity-panel="sessions" aria-labelledby="sessions-panel-title">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"ACCOUNT · SESSIONS"</p>
                            <h4 id="sessions-panel-title">"세션 · 기기"</h4>
                            <p>"현재 로그인한 기기, 최근 활동, 의심 신호를 한 화면에서 확인합니다."</p>
                        </div>
                        <div class="identity-session-grid">
                            <article class="session-card primary">
                                <span class="device-glyph" aria-hidden="true">"⌘"</span>
                                <div><strong>"MacBook Pro"</strong><small>"Chrome · Seoul · current session"</small></div>
                                <em>"방금 전"</em>
                            </article>
                            <article class="session-card">
                                <span class="device-glyph" aria-hidden="true">"◉"</span>
                                <div><strong>"iPhone 15 Pro"</strong><small>"Safari · Seoul · passkey verified"</small></div>
                                <em>"12 hours ago"</em>
                            </article>
                            <article class="session-card">
                                <span class="device-glyph" aria-hidden="true">"▣"</span>
                                <div><strong>"Edge on Windows"</strong><small>"Finance office · remembered device"</small></div>
                                <em>"3 days ago"</em>
                            </article>
                        </div>
                        <ol class="identity-audit-log">
                            <li><time>"09:14"</time><strong>"New passkey challenge passed"</strong><span>"MFA guardrail OK"</span></li>
                            <li><time>"08:42"</time><strong>"Payroll role inspected"</strong><span>"No write mutation"</span></li>
                            <li><time>"Yesterday"</time><strong>"Recovery codes viewed"</strong><span>"Admin acknowledgement required"</span></li>
                        </ol>
                        {identity_sessions_anchor_board()}
                    </article>

                    <article id="identity-roles" class="identity-panel" data-identity-panel="roles" aria-labelledby="roles-panel-title">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"ACCOUNT · ACCESS"</p>
                            <h4 id="roles-panel-title">"역할 · 권한"</h4>
                            <p>"워크플로우, 급여, 감사, 직원 정보 접근이 어떤 근거로 허용되는지 표시합니다."</p>
                        </div>
                        <table class="role-matrix-table">
                            <thead><tr><th>"역할"</th><th>"범위"</th><th>"결정"</th><th>"근거"</th></tr></thead>
                            <tbody>
                                <tr><td><strong>"Tenant Admin"</strong><small>"owner"</small></td><td>"Workspace · billing · users"</td><td><span class="status-chip success">"Allow"</span></td><td>"법인 관리자"</td></tr>
                                <tr><td><strong>"Payroll Approver"</strong><small>"finance"</small></td><td>"Payroll close · filing"</td><td><span class="status-chip warning">"Review"</span></td><td>"2-person approval"</td></tr>
                                <tr><td><strong>"Workflow Builder"</strong><small>"studio"</small></td><td>"Draft · simulate"</td><td><span class="status-chip success">"Allow"</span></td><td>"No live execution"</td></tr>
                                <tr><td><strong>"External Vendor"</strong><small>"guest"</small></td><td>"Employee PII"</td><td><span class="status-chip danger">"Deny"</span></td><td>"PIPA boundary"</td></tr>
                            </tbody>
                        </table>
                        {identity_roles_anchor_board()}
                    </article>

                    <article id="identity-org" class="identity-panel" data-identity-panel="org" aria-labelledby="org-panel-title">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"SETTINGS · OVERVIEW"</p>
                            <h4 id="org-panel-title">"조직 프로필"</h4>
                            <p>"모든 핵심 설정을 한 곳에서 관리하고 변경사항은 감사 체인에 자동 기록됩니다."</p>
                        </div>
                        <div class="org-stat-grid">
                            <span><small>"임직원"</small><strong>"118명"</strong><em>"▲ +6 last quarter"</em></span>
                            <span><small>"월 인건비"</small><strong>"₩894,000,000"</strong><em>"▲ +4.2% MoM"</em></span>
                            <span><small>"활성 워크플로우"</small><strong>"42개"</strong><em>"▲ +3 since launch"</em></span>
                            <span><small>"미해결 행동항목"</small><strong>"7건"</strong><em>"▼ −2 this week"</em></span>
                        </div>
                        <div class="org-profile-grid">
                            <dl><dt>"법인명"</dt><dd>"오야티 주식회사"</dd><dt>"사업자등록번호"</dt><dd>"123-45-67890"</dd><dt>"대표자"</dt><dd>"이재현"</dd><dt>"본점"</dt><dd>"서울 강남구 테헤란로 521 12층"</dd></dl>
                            <dl><dt>"주거래"</dt><dd>"신한은행 · 주식회사 오야티"</dd><dt>"출금 항목"</dt><dd>"급여 · 원천세 · 4대보험"</dd><dt>"검증 상태"</dt><dd>"✓ 1원 검증 완료"</dd><dt>"결제 카드"</dt><dd>"신한카드 ****-4081"</dd></dl>
                            <dl><dt>"주기"</dt><dd>"월급"</dd><dt>"지급일"</dt><dd>"매월 25일"</dd><dt>"근태 마감"</dt><dd>"3일 전 (22일)"</dd><dt>"다음 마감"</dt><dd>"2026-05-22 (금)"</dd></dl>
                            <dl><dt>"국민연금"</dt><dd>"12345678901"</dd><dt>"건강보험"</dt><dd>"234567890"</dd><dt>"고용보험"</dt><dd>"EI-2024-0091"</dd><dt>"산재보험"</dt><dd>"WCI-2024-0091 · 0.65%"</dd></dl>
                        </div>
                        {identity_org_anchor_board()}
                    </article>

                    <article id="identity-employees" class="identity-panel" data-identity-panel="employees" aria-labelledby="employees-panel-title">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"SETTINGS · EMPLOYEES"</p>
                            <h4 id="employees-panel-title">"직원 디렉토리"</h4>
                            <p>
                                "FD-001 people, payroll, policy, Mail, and Community workloads stay product-first while "
                                "Oyatie Cloud proves identity data can be hosted as a governed tenant surface."
                            </p>
                        </div>
                        <div class="employee-directory-stats">
                            <span><small>"전체"</small><strong>"118명"</strong></span>
                            <span><small>"활성"</small><strong>"109명"</strong></span>
                            <span><small>"최근 30일 입사"</small><strong>"6명"</strong></span>
                            <span><small>"수습 종료 임박"</small><strong>"5명"</strong></span>
                        </div>
                        {workforce_anchor_board()}
                        <div class="employee-directory-tools">
                            <label><span aria-hidden="true">"⌕"</span><input data-employee-search="true" aria-label="Search employees" placeholder="이름, 직책, 팀, ID 검색..." /></label>
                            <div class="employee-filter-pills" aria-label="Employee filters">
                                <button type="button" class="active" data-employee-filter="all">"활성 109"</button>
                                <button type="button" data-employee-filter="infrastructure">"플랫폼팀"</button>
                                <button type="button" data-employee-filter="finance">"Finance"</button>
                            </div>
                            <button type="button" data-identity-action="add-employee">"+ 직원 추가"</button>
                        </div>
                        <table class="employee-directory-table">
                            <thead><tr><th>"이름"</th><th>"직책"</th><th>"부서 · 팀"</th><th>"매니저"</th><th>"입사일"</th><th>"상태"</th><th>"Action"</th></tr></thead>
                            <tbody>
                                <tr data-employee-row="true" data-employee-team="office"><td><strong>"이재현"</strong><small>"Jaehyun Lee · emp_0000"</small></td><td>"Chief Executive Officer"</td><td>"Office of CEO"</td><td>"—"</td><td>"2021-03-14"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="finance"><td><strong>"최유나"</strong><small>"Yuna Choi · emp_0011"</small></td><td>"Chief Financial Officer"</td><td>"Finance"</td><td>"이재현"</td><td>"2022-04-01"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>"박서준"</strong><small>"Seojun Park · emp_0001"</small></td><td>"VP of Engineering"</td><td>"플랫폼팀"</td><td>"이재현"</td><td>"2021-06-01"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>"김지영"</strong><small>"Jiyoung Kim · emp_0002"</small></td><td>"Engineering Manager"</td><td>"플랫폼팀"</td><td>"박서준"</td><td>"2022-09-12"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>"윤태민"</strong><small>"Taemin Yoon · emp_0003"</small></td><td>"Senior Software Engineer"</td><td>"플랫폼팀"</td><td>"김지영"</td><td>"2026-05-12"</td><td><span class="status-chip warning">"수습"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="product"><td><strong>"임도윤"</strong><small>"Doyun Lim · emp_0004"</small></td><td>"Engineering Manager · Product"</td><td>"프로덕트팀"</td><td>"박서준"</td><td>"2022-11-04"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="product"><td><strong>"강수아"</strong><small>"Sua Kang · emp_0005"</small></td><td>"Software Engineer"</td><td>"프로덕트팀"</td><td>"임도윤"</td><td>"2024-02-19"</td><td><span class="status-chip">"휴직"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="data"><td><strong>"정우진"</strong><small>"Woojin Jung · emp_0006"</small></td><td>"Software Engineer · Frontend"</td><td>"데이터팀"</td><td>"임도윤"</td><td>"2025-08-07"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                            </tbody>
                        </table>
                    </article>

                    <article id="identity-onboarding" class="identity-panel" data-identity-panel="onboarding" aria-labelledby="onboarding-panel-title">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"WORKSPACE SETUP"</p>
                            <h4 id="onboarding-panel-title">"워크스페이스 설정"</h4>
                            <p>
                                "Workspace setup is the tenant-admission path: FD-001 product workloads collect legal, payroll, "
                                "policy, schedule, and evidence facts before Oyatie Cloud hosts the tenant."
                            </p>
                        </div>
                        {onboarding_anchor_board()}
                        <div class="onboarding-flow">
                            <div class="onboarding-progress">
                                <strong data-onboarding-percent="true">"56%"</strong>
                                <span class="score-bar" style="--bar: 56%"><em></em></span>
                                <button type="button" data-identity-action="advance-onboarding">"다음 단계 완료"</button>
                            </div>
                            <article class="setup-current-step">
                                <p class="screen-anchor">"CURRENT STEP"</p>
                                <h5>"급여 캘린더 확인"</h5>
                                <p>"지급일, 근태 마감, 원천세 신고 마감이 이번 달 워크플로우와 일치하는지 검토합니다."</p>
                                <ul>
                                    <li>"✓ 법인 정보 검증 완료"</li>
                                    <li>"✓ 은행 출금 계좌 검증 완료"</li>
                                    <li>"○ 캘린더 승인 대기"</li>
                                </ul>
                            </article>
                            <ol class="onboarding-steps">
                                <li class="done">"1 환영합니다"</li>
                                <li class="done">"2 법인 정보"</li>
                                <li class="done">"3 은행 · 결제"</li>
                                <li class="active">"4 급여 캘린더"</li>
                                <li>"5 4대보험 가입"</li>
                                <li>"6 관할"</li>
                                <li>"7 직원 가져오기"</li>
                                <li>"8 보존 · 정책"</li>
                                <li>"9 검토 · 가동"</li>
                            </ol>
                        </div>
                    </article>
                </div>

                <aside class="identity-context-rail" aria-label="Identity provenance">
                    <p class="screen-anchor">"PROVENANCE"</p>
                    <dl class="service-kv">
                        <div><dt>"Actor"</dt><dd>"Choi Yu-na · Admin"</dd></div>
                        <div><dt>"Scope"</dt><dd>"Auth · org · people"</dd></div>
                        <div><dt>"Receipt"</dt><dd>"REC-ID-2026-05"</dd></div>
                    </dl>
                    <p class="screen-anchor">"LOCAL ONLY"</p>
                    <ol class="notification-stack">
                        <li>"No real auth mutation"</li>
                        <li>"No external HR system write"</li>
                        <li>"Audit preview staged visually"</li>
                    </ol>
                </aside>
            </div>
        </section>
    }
}

fn workforce_anchor_board() -> impl IntoView {
    view! {
        <div class="workforce-anchor-grid" aria-label="FD-001 and Oyatie Cloud workforce proof">
            <article class="workforce-anchor-card selected" data-workforce-card="fd001">
                <p class="screen-anchor">"FD-001 WORKFORCE"</p>
                <h5>"People data powers product delivery"</h5>
                <p>
                    "Employee directory, payroll eligibility, reviewer mail, community announcements, and onboarding "
                    "are FD-001 tenant workloads, not separate HR widgets."
                </p>
                <div class="workforce-anchor-actions">
                    <button type="button" data-workforce-anchor-action="route-payroll">"Payroll impact"</button>
                    <button type="button" data-workforce-anchor-action="route-workflow">"Workflow path"</button>
                </div>
            </article>
            <article class="workforce-anchor-card" data-workforce-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD"</p>
                <h5>"Hosted as a governed tenant surface"</h5>
                <p>
                    "PIPA boundaries, regional pack gates, role envelopes, audit receipts, and evidence routes prove "
                    "the substrate can host real workforce tenants."
                </p>
                <div class="workforce-anchor-actions">
                    <button type="button" data-workforce-anchor-action="route-policy">"Policy envelope"</button>
                    <button type="button" data-workforce-anchor-action="route-audit">"Audit trail"</button>
                </div>
            </article>
            <article class="workforce-anchor-card" data-workforce-card="lifecycle">
                <p class="screen-anchor">"LIFECYCLE OPS"</p>
                <h5>"Interactive, local-only employee command"</h5>
                <p>
                    "Operators can inspect people, stage invites, route leave/time, and brief reviewers while HRIS, "
                    "auth, payroll, and cloud mutations remain disconnected."
                </p>
                <div class="workforce-anchor-actions">
                    <button type="button" data-workforce-anchor-action="stage-invite">"Stage invite"</button>
                    <button type="button" data-workforce-anchor-action="route-leave">"Leave & time"</button>
                </div>
            </article>
        </div>
        <div class="workforce-anchor-footer">
            <span data-workforce-anchor-status="true">"Employees ready · FD-001 workforce workload dogfoods Oyatie Cloud locally."</span>
            <div class="workforce-anchor-routes" aria-label="Workforce connected routes">
                <button type="button" data-workforce-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-workforce-anchor-action="route-community">"Community update"</button>
                <button type="button" data-workforce-anchor-action="route-evidence">"Evidence graph"</button>
                <button type="button" data-workforce-anchor-action="route-cloud">"Cloud cells"</button>
            </div>
        </div>
    }
}

fn onboarding_anchor_board() -> impl IntoView {
    view! {
        <div class="onboarding-anchor-grid" aria-label="FD-001 tenant admission setup proof">
            <article class="onboarding-anchor-card selected" data-onboarding-card="tenant">
                <p class="screen-anchor">"FD-001 TENANT ADMISSION"</p>
                <h5>"Product workload setup path"</h5>
                <p>
                    "Legal profile, payroll calendar, employee import, policy gates, Mail reviewers, Community launch notes, "
                    "and evidence receipts become one tenant setup packet."
                </p>
                <div class="onboarding-anchor-actions">
                    <button type="button" data-onboarding-anchor-action="route-tasks">"Today queue"</button>
                    <button type="button" data-onboarding-anchor-action="import-employees">"Import people"</button>
                </div>
            </article>
            <article class="onboarding-anchor-card" data-onboarding-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD"</p>
                <h5>"Substrate readiness before go-live"</h5>
                <p>
                    "Region pack, PIPA boundary, role envelope, deployment gates, audit freshness, and rollback posture "
                    "prove the tenant can be hosted safely."
                </p>
                <div class="onboarding-anchor-actions">
                    <button type="button" data-onboarding-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-onboarding-anchor-action="route-policy">"Policy gate"</button>
                </div>
            </article>
            <article class="onboarding-anchor-card" data-onboarding-card="launch">
                <p class="screen-anchor">"LAUNCH PACKET"</p>
                <h5>"Interactive, local-only setup"</h5>
                <p>
                    "Operators can advance setup, draft reviewer mail, post a community note, and attach evidence while "
                    "registries, HRIS, payroll, auth, and cloud mutations remain disconnected."
                </p>
                <div class="onboarding-anchor-actions">
                    <button type="button" data-onboarding-anchor-action="advance-setup">"Advance setup"</button>
                    <button type="button" data-onboarding-anchor-action="route-evidence">"Evidence"</button>
                </div>
            </article>
        </div>
        <div class="onboarding-anchor-footer">
            <span data-onboarding-anchor-status="true">
                "Onboarding ready · FD-001 tenant setup dogfoods Oyatie Cloud locally."
            </span>
            <div class="onboarding-anchor-routes" aria-label="Onboarding connected routes">
                <button type="button" data-onboarding-anchor-action="route-payroll">"Payroll calendar"</button>
                <button type="button" data-onboarding-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-onboarding-anchor-action="route-community">"Community launch"</button>
                <button type="button" data-onboarding-anchor-action="route-schedule">"Schedule"</button>
            </div>
        </div>
    }
}

fn identity_sessions_anchor_board() -> impl IntoView {
    view! {
        <div class="identity-anchor-grid identity-sessions-anchor" aria-label="Session tenant proof">
            <article class="identity-anchor-card selected" data-identity-anchor-card="sessions">
                <p class="screen-anchor">"FD-001 SESSION PROOF"</p>
                <h5>"Auth sessions protect product workloads"</h5>
                <p>"Passkey, device, and payroll-role activity are evidence leaves for FD-001 tenant services."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-roles">"Role envelope"</button>
                    <button type="button" data-identity-anchor-action="route-evidence">"Evidence graph"</button>
                </div>
            </article>
            <article class="identity-anchor-card" data-identity-anchor-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD"</p>
                <h5>"Oyatie Cloud tenant session posture"</h5>
                <p>"Device locality, PIPA-safe audit, and session freshness prove the substrate can host workforce tenants."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-identity-anchor-action="route-policy">"PIPA policy"</button>
                </div>
            </article>
        </div>
        <div class="identity-anchor-footer">
            <span data-identity-anchor-status="true">"Sessions ready · local-only identity telemetry."</span>
            <div class="identity-anchor-routes">
                <button type="button" data-identity-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-identity-anchor-action="route-audit">"Audit ledger"</button>
            </div>
        </div>
    }
}

fn identity_roles_anchor_board() -> impl IntoView {
    view! {
        <div class="identity-anchor-grid identity-roles-anchor" aria-label="Role envelope proof">
            <article class="identity-anchor-card selected" data-identity-anchor-card="roles">
                <p class="screen-anchor">"FD-001 ROLE ENVELOPE"</p>
                <h5>"Access controls every product workload"</h5>
                <p>"Payroll, filing, workflow, Mail, Community, and cloud operations share one role envelope."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="review-roles">"Review grants"</button>
                    <button type="button" data-identity-anchor-action="route-workflow">"Workflow gate"</button>
                </div>
            </article>
            <article class="identity-anchor-card" data-identity-anchor-card="pipa">
                <p class="screen-anchor">"OYATIE CLOUD POLICY"</p>
                <h5>"Oyatie Cloud PIPA-safe tenant boundary"</h5>
                <p>"Role decisions stay auditable before any tenant workload can move through cloud admission gates."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-policy">"Policy board"</button>
                    <button type="button" data-identity-anchor-action="route-cloud">"Cloud gate"</button>
                </div>
            </article>
            <article class="identity-anchor-card" data-identity-anchor-card="local">
                <p class="screen-anchor">"LOCAL ONLY"</p>
                <h5>"Interactive access preview"</h5>
                <p>"Grant reviews, denial traces, and reviewer routes update visual state only; no SSO or IAM mutation runs."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-evidence">"Evidence"</button>
                    <button type="button" data-identity-anchor-action="route-community">"Community note"</button>
                </div>
            </article>
        </div>
        <div class="identity-anchor-footer">
            <span data-identity-anchor-status="true">"Roles ready · FD-001 access envelope dogfoods Oyatie Cloud locally."</span>
            <div class="identity-anchor-routes">
                <button type="button" data-identity-anchor-action="route-payroll">"Payroll close"</button>
                <button type="button" data-identity-anchor-action="route-audit">"Audit packet"</button>
            </div>
        </div>
    }
}

fn identity_org_anchor_board() -> impl IntoView {
    view! {
        <div class="identity-anchor-grid identity-org-anchor" aria-label="Organization tenant proof">
            <article class="identity-anchor-card selected" data-identity-anchor-card="org">
                <p class="screen-anchor">"FD-001 ORG PROFILE"</p>
                <h5>"Corporate facts feed every module"</h5>
                <p>"Legal profile, payroll calendar, tax identifiers, billing, and employee facts become shared tenant context."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-onboarding">"Setup packet"</button>
                    <button type="button" data-identity-anchor-action="route-payroll">"Payroll calendar"</button>
                </div>
            </article>
            <article class="identity-anchor-card" data-identity-anchor-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD TENANT"</p>
                <h5>"Oyatie Cloud hosted profile readiness"</h5>
                <p>"Region packs, audit receipts, deployment gates, and evidence spine prove this tenant can be hosted."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-identity-anchor-action="route-evidence">"Evidence spine"</button>
                </div>
            </article>
        </div>
        <div class="identity-anchor-footer">
            <span data-identity-anchor-status="true">"Organization ready · local-only tenant profile preview."</span>
            <div class="identity-anchor-routes">
                <button type="button" data-identity-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-identity-anchor-action="route-community">"Community launch"</button>
            </div>
        </div>
    }
}

fn identity_command_board() -> impl IntoView {
    view! {
        <div class="identity-command-board" aria-label="Identity command center">
            <section class="identity-command-card identity-spine-card">
                <div class="identity-command-card-head">
                    <div>
                        <p class="screen-anchor">"ACCESS SPINE"</p>
                        <h5>"One governed identity path from auth to payroll close"</h5>
                    </div>
                    <span class="status-chip success">"PIPA-safe"</span>
                </div>
                <div class="identity-spine-flow" aria-label="Identity access lineage">
                    <span class="active"><em>"01"</em><strong>"Passkey"</strong><small>"verified"</small></span>
                    <i></i>
                    <span><em>"02"</em><strong>"Session"</strong><small>"current device"</small></span>
                    <i></i>
                    <span class="review"><em>"03"</em><strong>"Role"</strong><small>"payroll review"</small></span>
                    <i></i>
                    <span><em>"04"</em><strong>"Employee"</strong><small>"118 records"</small></span>
                    <i></i>
                    <span class="review"><em>"05"</em><strong>"Workflow"</strong><small>"2-person gate"</small></span>
                    <i></i>
                    <span class="sealed"><em>"06"</em><strong>"Audit"</strong><small>"REC-ID-2026-05"</small></span>
                </div>
                <dl class="identity-command-kv">
                    <div><dt>"Autonomy ceiling"</dt><dd>"No auth mutation · local preview only"</dd></div>
                    <div><dt>"Primary risk"</dt><dd>"Payroll approver role expires before close"</dd></div>
                    <div><dt>"Connected route"</dt><dd>"Workflow → Mail → Evidence Spine"</dd></div>
                </dl>
            </section>

            <section class="identity-command-card identity-risk-card">
                <div class="identity-command-card-head">
                    <div>
                        <p class="screen-anchor">"RISK QUEUE"</p>
                        <h5>"Access work that affects today’s operations"</h5>
                    </div>
                    <button type="button" data-identity-route-action="evidence">"Evidence"</button>
                </div>
                <div class="identity-risk-list" role="list" aria-label="Identity risk queue">
                    <article role="listitem" data-identity-risk-row="review">
                        <span class="status-chip warning">"review"</span>
                        <strong>"Payroll approver recertification"</strong>
                        <p>"CFO role grants payroll close and HomeTax transport; 2-person review due today."</p>
                        <small>"Owner CFO · SLA 4.0h · REC-ID-2026-05"</small>
                    </article>
                    <article role="listitem" data-identity-risk-row="blocking">
                        <span class="status-chip danger">"blocking"</span>
                        <strong>"Vendor guest cannot view employee PII"</strong>
                        <p>"Stripe renewal route needs procurement context without exposing workforce records."</p>
                        <small>"Owner Security reviewer · policy POL-PII-014"</small>
                    </article>
                    <article role="listitem" data-identity-risk-row="sealed">
                        <span class="status-chip success">"sealed"</span>
                        <strong>"Passkey challenge evidence sealed"</strong>
                        <p>"MacBook and iPhone passkey state available to audit, not external auth writes."</p>
                        <small>"Source local island · 09:14 KST"</small>
                    </article>
                </div>
            </section>

            <section class="identity-command-card identity-lifecycle-card">
                <div class="identity-command-card-head">
                    <div>
                        <p class="screen-anchor">"WORKFORCE LIFECYCLE"</p>
                        <h5>"Employees, onboarding, roles, and payroll impact"</h5>
                    </div>
                    <button type="button" data-identity-route-action="employees">"Open people"</button>
                </div>
                <div class="identity-lifecycle-grid" aria-label="Workforce lifecycle state">
                    <span style="--bar: 77%"><strong>"Onboarding"</strong><em>"6 active · 77%"</em></span>
                    <span style="--bar: 64%"><strong>"Role review"</strong><em>"14 grants · 64%"</em></span>
                    <span style="--bar: 48%"><strong>"Session hygiene"</strong><em>"3 stale · 48%"</em></span>
                    <span style="--bar: 83%"><strong>"Payroll readiness"</strong><em>"109 active · 83%"</em></span>
                </div>
            </section>

            <section class="identity-command-card identity-route-card">
                <div class="identity-command-card-head">
                    <div>
                        <p class="screen-anchor">"ROUTE MATRIX"</p>
                        <h5>"Every identity action lands inside the same service graph"</h5>
                    </div>
                </div>
                <div class="identity-route-grid" aria-label="Identity local routes">
                    <button type="button" data-identity-route-action="workflow">"Workflow gate"</button>
                    <button type="button" data-identity-route-action="mail">"Mail reviewer"</button>
                    <button type="button" data-identity-route-action="sessions">"Session audit"</button>
                    <button type="button" data-identity-route-action="onboarding">"Setup checklist"</button>
                    <button type="button" data-identity-route-action="finance">"Payroll close"</button>
                    <button type="button" data-identity-route-action="evidence">"Evidence spine"</button>
                </div>
                <p class="identity-command-note">"Routes change local visual state only; no SSO, HRIS, payroll, or directory backend is wired."</p>
            </section>
        </div>
    }
}

fn finance_commercial_service() -> impl IntoView {
    view! {
        <section
            id="finance-commercial-service"
            class="finance-commercial-service panel"
            data-finance-service="true"
            aria-labelledby="finance-service-title"
        >
            <div class="finance-service-head">
                <div>
                    <p class="screen-anchor">"MONEY · COMMERCIAL OPERATIONS"</p>
                    <h3 id="finance-service-title">"Finance, ledger, vendor spend, billing, and leave-time"</h3>
                </div>
                <div class="finance-service-actions">
                    <span class="status-chip success" data-finance-status="true">"close package ready"</span>
                    <button type="button" data-finance-action="reconcile">"Reconcile"</button>
                    <button type="button" data-finance-action="export-pack">"Export pack"</button>
                </div>
            </div>

            <div class="finance-kpi-strip" aria-label="Finance operations summary">
                <span><small>"Cash runway"</small><strong>"18.4 mo"</strong><em>"₩12.6B available"</em></span>
                <span><small>"Open invoices"</small><strong>"₩482M"</strong><em>"12 invoices · 2 overdue"</em></span>
                <span><small>"Vendor risk"</small><strong>"3 high"</strong><em>"Stripe · AWS · payroll bureau"</em></span>
                <span><small>"Leave liability"</small><strong>"₩53M"</strong><em>"49.5h this week"</em></span>
            </div>

            <div class="finance-service-shell">
                <aside class="finance-ledger-rail" aria-label="Finance sections">
                    <p class="screen-anchor">"FINANCE VIEWS"</p>
                    <button type="button" class="active" data-finance-tab="ledger">"Ledger close"</button>
                    <button type="button" data-finance-tab="vendors">"Vendors & spend"</button>
                    <button type="button" data-finance-tab="billing">"Billing & tax"</button>
                    <button type="button" data-finance-tab="leave">"Leave & time"</button>
                    <div class="finance-close-card">
                        <small>"APRIL CLOSE"</small>
                        <strong>"73%"</strong>
                        <span class="score-bar" style="--bar: 73%"><em></em></span>
                        <p>"Payroll, withholding, and vendor accruals are staged for 2-person review."</p>
                    </div>
                </aside>

                <div class="finance-service-main">
                    <div class="finance-tabs" role="tablist" aria-label="Finance service views">
                        <button type="button" class="active" data-finance-tab="ledger" role="tab" aria-selected="true">"Ledger"</button>
                        <button type="button" data-finance-tab="vendors" role="tab" aria-selected="false">"Vendors"</button>
                        <button type="button" data-finance-tab="billing" role="tab" aria-selected="false">"Billing"</button>
                        <button type="button" data-finance-tab="leave" role="tab" aria-selected="false">"Leave & time"</button>
                    </div>

                    <article id="ledger-preview" class="finance-panel active" data-finance-panel="ledger" aria-labelledby="ledger-panel-title">
                        <div class="finance-panel-copy">
                            <p class="screen-anchor">"LEDGER · CLOSE PACKAGE"</p>
                            <h4 id="ledger-panel-title">"Ledger close cockpit"</h4>
                            <p>"Every payroll, filing, vendor, and billing event resolves into one audit-ready close package."</p>
                        </div>
                        <div class="ledger-layout">
                            <div class="ledger-reconciliation">
                                <span style="--bar: 92%"><em>"Bank feed match · 92%"</em></span>
                                <span style="--bar: 86%"><em>"Payroll accrual · 86%"</em></span>
                                <span style="--bar: 68%"><em>"Vendor accrual · 68%"</em></span>
                                <span style="--bar: 51%"><em>"Tax evidence · 51%"</em></span>
                            </div>
                            <table class="finance-table">
                                <thead><tr><th>"Time"</th><th>"Account"</th><th>"Object"</th><th>"Amount"</th><th>"State"</th></tr></thead>
                                <tbody>
                                    <tr><td>"09:18"</td><td>"Payroll payable"</td><td>"APR payroll close"</td><td>"₩894,000,000"</td><td><span class="status-chip warning">"review"</span></td></tr>
                                    <tr><td>"09:42"</td><td>"Withholding tax"</td><td>"HomeTax draft"</td><td>"₩118,400,000"</td><td><span class="status-chip success">"matched"</span></td></tr>
                                    <tr><td>"10:05"</td><td>"Vendor accrual"</td><td>"Stripe invoice"</td><td>"₩4,820,000"</td><td><span class="status-chip danger">"blocking"</span></td></tr>
                                    <tr><td>"10:21"</td><td>"Leave liability"</td><td>"Yoon Tae-min risk"</td><td>"₩53,000,000"</td><td><span class="status-chip">"advisory"</span></td></tr>
                                </tbody>
                            </table>
                        </div>
                        {finance_command_board()}
                        {ledger_preview_anchor_board()}
                    </article>

                    <article id="vendors-spend" class="finance-panel" data-finance-panel="vendors" aria-labelledby="vendors-panel-title">
                        <div class="finance-panel-copy">
                            <p class="screen-anchor">"PROCUREMENT · VENDORS"</p>
                            <h4 id="vendors-panel-title">"Vendors & spend control"</h4>
                            <p>"Contracts, approvals, owners, and cost-of-delay are tied to the same service graph as workflow and mail."</p>
                        </div>
                        <div class="vendor-toolbar">
                            <label><span aria-hidden="true">"⌕"</span><input data-vendor-search="true" aria-label="Search vendors" placeholder="Search vendor, owner, contract..." /></label>
                            <button type="button" data-finance-action="add-vendor">"+ Vendor"</button>
                        </div>
                        <table class="finance-table vendor-table">
                            <thead><tr><th>"Vendor"</th><th>"Owner"</th><th>"Monthly"</th><th>"Renewal"</th><th>"Risk"</th><th>"Action"</th></tr></thead>
                            <tbody>
                                <tr data-vendor-row="true"><td><strong>"Stripe"</strong><small>"Payments · invoice INV-4281"</small></td><td>"Finance"</td><td>"₩4.82M"</td><td>"2026-05-31"</td><td><span class="status-chip danger">"High"</span></td><td><button type="button" data-finance-action="approve-vendor">"Review"</button></td></tr>
                                <tr data-vendor-row="true"><td><strong>"AWS Korea"</strong><small>"Cloud infrastructure · reserved capacity"</small></td><td>"SRE"</td><td>"₩63.4M"</td><td>"2026-06-12"</td><td><span class="status-chip warning">"Medium"</span></td><td><button type="button" data-finance-action="optimize-spend">"Optimize"</button></td></tr>
                                <tr data-vendor-row="true"><td><strong>"Shinhan Bank"</strong><small>"Payroll and withholding transport"</small></td><td>"CFO"</td><td>"₩1.2M"</td><td>"2027-01-10"</td><td><span class="status-chip success">"Low"</span></td><td><button type="button" data-sidepeek-trigger="bank-vendor" data-sidepeek-title="Shinhan Bank transport" data-sidepeek-id="VEN-SHINHAN" data-sidepeek-desc="Bank transport is staged visually and is not connected to a real payment rail." data-sidepeek-owner="CFO" data-sidepeek-risk="Low" data-sidepeek-sla="Staged only">"Inspect"</button></td></tr>
                            </tbody>
                        </table>
                        {finance_vendors_anchor_board()}
                    </article>

                    <article id="billing-tax" class="finance-panel" data-finance-panel="billing" aria-labelledby="billing-panel-title">
                        <div class="finance-panel-copy">
                            <p class="screen-anchor">"BILLING · TAX"</p>
                            <h4 id="billing-panel-title">"Billing, plans, and filings"</h4>
                            <p>"Customer invoices, plan changes, HomeTax filing readiness, and payment evidence stay visible next to operations."</p>
                        </div>
                        <div class="billing-grid">
                            <article><p class="screen-anchor">"REVENUE"</p><strong>"₩2.31B"</strong><span>"ARR staged · 42 active contracts"</span><button type="button" data-finance-action="send-invoice">"Stage invoice"</button></article>
                            <article><p class="screen-anchor">"TAX FILING"</p><strong>"64%"</strong><span>"HomeTax transport awaiting reviewer"</span><button type="button" data-finance-action="tax-brief">"Draft brief"</button></article>
                            <article><p class="screen-anchor">"PLAN CHANGES"</p><strong>"7"</strong><span>"2 require billing owner review"</span><button type="button" data-sidepeek-trigger="billing-plans" data-sidepeek-title="Plan change queue" data-sidepeek-id="BILL-PLAN-7" data-sidepeek-desc="Plan changes are staged queue items with no payment or billing mutation." data-sidepeek-owner="Revenue ops" data-sidepeek-risk="Review" data-sidepeek-sla="Visual only">"Open queue"</button></article>
                        </div>
                        {finance_billing_anchor_board()}
                    </article>

                    <article id="leave-time" class="finance-panel" data-finance-panel="leave" aria-labelledby="leave-panel-title">
                        <div class="finance-panel-copy">
                            <p class="screen-anchor">"PEOPLE COST · LEAVE"</p>
                            <h4 id="leave-panel-title">"Leave & time liability"</h4>
                            <p>"Leave approvals are connected to workforce, payroll, schedule, and financial liability before close."</p>
                        </div>
                        <div class="leave-layout">
                            <ol class="leave-queue">
                                <li><time>"May 13–17"</time><strong>"김지영 leave request"</strong><span>"5 days · backup confirmed"</span><button type="button" data-finance-action="approve-leave">"Approve locally"</button></li>
                                <li><time>"This week"</time><strong>"윤태민 overtime risk"</strong><span>"49.5h projected · 2 backup engineers recommended"</span><button type="button" data-finance-action="reassign-time">"Reassign"</button></li>
                                <li><time>"May 22"</time><strong>"Payroll cutoff"</strong><span>"Timesheets lock 3 days before payout"</span><button type="button" data-finance-action="lock-timesheets">"Preview lock"</button></li>
                            </ol>
                            <div class="time-heatmap" aria-label="Weekly time utilization">
                                <span style="--bar: 58%">"Mon"</span>
                                <span style="--bar: 72%">"Tue"</span>
                                <span style="--bar: 91%">"Wed"</span>
                                <span style="--bar: 84%">"Thu"</span>
                                <span style="--bar: 49%">"Fri"</span>
                            </div>
                        </div>
                        {finance_leave_anchor_board()}
                    </article>
                </div>

                <aside class="finance-context-rail" aria-label="Finance provenance">
                    <p class="screen-anchor">"CLOSE PROVENANCE"</p>
                    <dl class="service-kv">
                        <div><dt>"Workflow"</dt><dd>"April close package"</dd></div>
                        <div><dt>"Receipts"</dt><dd>"18 staged"</dd></div>
                        <div><dt>"Reviewers"</dt><dd>"CFO · payroll · tax"</dd></div>
                    </dl>
                    <p class="screen-anchor">"LOCAL NOTIFICATIONS"</p>
                    <ol class="notification-stack">
                        <li>"Vendor approval waiting"</li>
                        <li>"Tax brief can be drafted"</li>
                        <li>"No real money moves"</li>
                    </ol>
                </aside>
            </div>
        </section>
    }
}

fn finance_vendors_anchor_board() -> impl IntoView {
    view! {
        <div class="finance-anchor-grid" aria-label="FD-001 vendor spend and Oyatie Cloud tenant proof">
            <article class="finance-anchor-card selected" data-finance-anchor-card="vendors-fd001">
                <p class="screen-anchor">"FD-001 VENDOR WORKLOAD"</p>
                <h5>"Procurement is part of product delivery"</h5>
                <p>
                    "Vendor approvals, spend controls, Workflow tasks, Mail briefs, and Community notes are FD-001 tenant workloads sharing one commercial graph."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="stage-contract">"Stage contract"</button>
                    <button type="button" data-finance-anchor-action="route-workflow">"Workflow gate"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="vendors-cloud">
                <p class="screen-anchor">"OYATIE CLOUD FINOPS"</p>
                <h5>"Cloud substrate proves tenant spend posture"</h5>
                <p>
                    "Oyatie Cloud hosts FD-001 services as real tenant workloads while FinOps, policy, audit, and regional gates stay visible before production claims."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-cloud">"Open FinOps"</button>
                    <button type="button" data-finance-anchor-action="route-policy">"Policy envelope"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="vendors-local">
                <p class="screen-anchor">"LOCAL-ONLY RAIL"</p>
                <h5>"Interactive procurement preview"</h5>
                <p>
                    "Operators can inspect Stripe, AWS Korea, and bank transport paths; no bank, payroll, tax, billing, vendor, or cloud mutation executes."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-audit">"Audit trail"</button>
                    <button type="button" data-finance-anchor-action="route-mail">"Reviewer Mail"</button>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Vendors ready · FD-001 procurement workload dogfoods Oyatie Cloud locally."</span>
            <div class="finance-anchor-routes" aria-label="Vendor connected routes">
                <button type="button" data-finance-anchor-action="route-ledger">"Ledger"</button>
                <button type="button" data-finance-anchor-action="route-billing">"Billing"</button>
                <button type="button" data-finance-anchor-action="route-community">"Community"</button>
                <button type="button" data-finance-anchor-action="route-evidence">"Evidence"</button>
            </div>
        </div>
    }
}

fn finance_billing_anchor_board() -> impl IntoView {
    view! {
        <div class="finance-anchor-grid" aria-label="FD-001 billing tax and Oyatie Cloud tenant proof">
            <article class="finance-anchor-card selected" data-finance-anchor-card="billing-fd001">
                <p class="screen-anchor">"FD-001 REVENUE WORKLOAD"</p>
                <h5>"Billing supports master-plan product delivery"</h5>
                <p>
                    "Invoices, plan changes, tax briefs, customer Mail, and evidence receipts stay inside FD-001 so product delivery remains the master-plan goal."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="stage-invoice">"Stage invoice"</button>
                    <button type="button" data-finance-anchor-action="route-mail">"Mail customer"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="billing-cloud">
                <p class="screen-anchor">"OYATIE CLOUD TENANT"</p>
                <h5>"Revenue systems run as tenant workloads"</h5>
                <p>
                    "Oyatie Cloud proves production hosting through residency, policy, rollback, and audit receipts before any FD-001 billing surface claims readiness."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-cloud">"Cloud proof"</button>
                    <button type="button" data-finance-anchor-action="route-policy">"Tax policy"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="billing-local">
                <p class="screen-anchor">"LOCAL-ONLY CASH CONTROL"</p>
                <h5>"Tax and invoice dry-run only"</h5>
                <p>
                    "Operators can route invoices, HomeTax briefs, and plan reviews visually; no bank, payroll, tax filing, billing send, or cloud mutation executes."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="tax-brief">"Tax brief"</button>
                    <button type="button" data-finance-anchor-action="route-audit">"Audit packet"</button>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Billing ready · FD-001 revenue workload dogfoods Oyatie Cloud locally."</span>
            <div class="finance-anchor-routes" aria-label="Billing connected routes">
                <button type="button" data-finance-anchor-action="route-vendors">"Vendors"</button>
                <button type="button" data-finance-anchor-action="route-leave">"Leave cost"</button>
                <button type="button" data-finance-anchor-action="route-workflow">"Workflow"</button>
                <button type="button" data-finance-anchor-action="route-evidence">"Evidence"</button>
            </div>
        </div>
    }
}

fn finance_leave_anchor_board() -> impl IntoView {
    view! {
        <div class="finance-anchor-grid" aria-label="FD-001 leave time and Oyatie Cloud tenant proof">
            <article class="finance-anchor-card selected" data-finance-anchor-card="leave-fd001">
                <p class="screen-anchor">"FD-001 PEOPLE COST"</p>
                <h5>"Leave and time feed payroll, schedule, and tenant workload delivery"</h5>
                <p>
                    "Leave approvals, overtime risk, payroll cutoff, Workflow routes, Mail, and Community updates are FD-001 tenant workload evidence, not a side module."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="approve-leave">"Approve preview"</button>
                    <button type="button" data-finance-anchor-action="route-workflow">"Workflow route"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="leave-cloud">
                <p class="screen-anchor">"OYATIE CLOUD WORKFORCE"</p>
                <h5>"Substrate hosts workforce-cost tenant surfaces"</h5>
                <p>
                    "Oyatie Cloud proves the workforce substrate with regional policy, audit receipts, and deployment gates before leave or payroll workloads claim readiness."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-finance-anchor-action="route-policy">"PIPA policy"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="leave-local">
                <p class="screen-anchor">"LOCAL-ONLY TIME RAIL"</p>
                <h5>"Interactive liability preview"</h5>
                <p>
                    "Operators can reassign coverage, preview timesheet locks, and brief reviewers; no bank, payroll, tax, billing, HRIS, or cloud mutation executes."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="reassign-time">"Reassign time"</button>
                    <button type="button" data-finance-anchor-action="route-audit">"Audit trail"</button>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Leave/time ready · FD-001 people-cost workload dogfoods Oyatie Cloud locally."</span>
            <div class="finance-anchor-routes" aria-label="Leave time connected routes">
                <button type="button" data-finance-anchor-action="route-ledger">"Ledger"</button>
                <button type="button" data-finance-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-finance-anchor-action="route-community">"Community"</button>
                <button type="button" data-finance-anchor-action="route-evidence">"Evidence"</button>
            </div>
        </div>
    }
}

fn ledger_preview_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board ledger-trust-board" aria-label="FD-001 ledger close and Oyatie Cloud commercial proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="ledger-fd001">
                    <p class="screen-anchor">"FD-001 CLOSE PACKAGE"</p>
                    <h5>"Ledger is the commercial product spine"</h5>
                    <p>
                        "Payroll, filing, vendors, billing, leave/time, Workflow, Mail, Community, and audit receipts resolve into one FD-001 tenant workload close packet."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="stage-close">"Stage close"</button>
                        <button type="button" data-trust-proof-action="route-workflow">"Workflow gate"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD FINOPS"</p>
                    <h5>"Substrate cost and audit prove readiness"</h5>
                    <p>
                        "Oyatie Cloud hosts commercial microservices as tenant workloads while FinOps, resource inventory, release gates, and policy guard the close."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-finops">"FinOps"</button>
                        <button type="button" data-trust-proof-action="route-inventory">"Resources"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-local">
                    <p class="screen-anchor">"LOCAL-ONLY LEDGER CONTROL"</p>
                    <h5>"Dense finance preview, no money movement"</h5>
                    <p>
                        "Operators can stage reconciliations, route reviewers, and attach evidence visually; no bank, payroll, tax, invoice, database, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-mail">"Reviewer Mail"</button>
                        <button type="button" data-trust-proof-action="route-audit">"Audit packet"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Ledger ready · FD-001 commercial close workload dogfoods Oyatie Cloud with local visual controls only."
                </span>
                <div class="trust-anchor-routes" aria-label="Ledger preview connected routes">
                    <button type="button" data-trust-proof-action="route-billing">"Billing"</button>
                    <button type="button" data-trust-proof-action="route-vendors">"Vendors"</button>
                    <button type="button" data-trust-proof-action="route-filing">"Filing"</button>
                    <button type="button" data-trust-proof-action="route-evidence">"Evidence"</button>
                </div>
            </div>
        </div>
    }
}

fn finance_command_board() -> impl IntoView {
    view! {
        <section class="finance-command-board" aria-labelledby="finance-command-board-title">
            <div class="finance-command-head">
                <div>
                    <p class="screen-anchor">"CLOSE COMMAND · OBJECT GRAPH"</p>
                    <h4 id="finance-command-board-title">"April commercial command center"</h4>
                    <p>
                        "A Bominal-grade operating surface: payroll, ledger, tax, vendors, billing, leave, workflow, messages, and evidence stay in one dense local workspace."
                    </p>
                </div>
                <div class="finance-command-actions">
                    <span class="status-chip warning" data-finance-command-status="true">"7 objects · 3 blockers · local dry-run"</span>
                    <button type="button" data-finance-command-action="run-close">"Run close dry-run"</button>
                    <button type="button" data-finance-command-action="attach-proof" data-finance-route="evidence">"Attach proof"</button>
                </div>
            </div>

            <div class="finance-command-grid">
                <article class="finance-command-card finance-close-spine-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"CLOSE SPINE"</p>
                            <h5>"Every commercial object joins the same package"</h5>
                        </div>
                        <span class="status-chip danger">"3 blockers"</span>
                    </div>
                    <div class="finance-spine-flow" aria-label="Commercial close object spine">
                        <button type="button" class="active" data-finance-command-action="open-payroll" data-finance-route="ledger">
                            <span>"Payroll"</span><strong>"₩894M"</strong><em>"review"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-tax" data-finance-route="billing">
                            <span>"Tax"</span><strong>"₩118M"</strong><em>"HomeTax"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-vendor" data-finance-route="vendors">
                            <span>"Vendors"</span><strong>"₩69M"</strong><em>"Stripe · AWS"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-billing" data-finance-route="billing">
                            <span>"Billing"</span><strong>"₩482M"</strong><em>"12 invoices"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-leave" data-finance-route="leave">
                            <span>"Leave"</span><strong>"49.5h"</strong><em>"liability"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-evidence" data-finance-route="evidence">
                            <span>"Evidence"</span><strong>"18"</strong><em>"receipts"</em>
                        </button>
                    </div>
                    <dl class="finance-command-kv">
                        <div><dt>"Critical path"</dt><dd>"Payroll delta → vendor approval → HomeTax filing → CFO signoff"</dd></div>
                        <div><dt>"Autonomy ceiling"</dt><dd>"Visual dry-run only; no banking, payroll, tax, or invoice rail executes."</dd></div>
                    </dl>
                </article>

                <article class="finance-command-card finance-cash-pipeline-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"CASH PIPELINE"</p>
                            <h5>"Invoices, plans, and filing readiness"</h5>
                        </div>
                        <button type="button" data-finance-command-action="open-billing" data-finance-route="billing">"Billing"</button>
                    </div>
                    <div class="finance-cash-lanes" aria-label="Commercial cash pipeline">
                        <button type="button" class="finance-cash-row" data-finance-command-action="stage-invoice" data-finance-route="billing">
                            <span>"Invoice"</span><strong>"Northwind annual plan"</strong><em>"₩184M · due May 27"</em><i class="status-chip success">"ready"</i>
                        </button>
                        <button type="button" class="finance-cash-row" data-finance-command-action="review-plan" data-finance-route="billing">
                            <span>"Plan"</span><strong>"7 contract changes"</strong><em>"2 owner reviews"</em><i class="status-chip warning">"review"</i>
                        </button>
                        <button type="button" class="finance-cash-row" data-finance-command-action="tax-transport" data-finance-route="billing">
                            <span>"Tax"</span><strong>"HomeTax withholding"</strong><em>"118 employees validated"</em><i class="status-chip warning">"draft"</i>
                        </button>
                        <button type="button" class="finance-cash-row" data-finance-command-action="bank-match" data-finance-route="ledger">
                            <span>"Bank"</span><strong>"Shinhan feed match"</strong><em>"92% matched"</em><i class="status-chip success">"matched"</i>
                        </button>
                    </div>
                </article>

                <article class="finance-command-card finance-vendor-risk-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"RISK QUEUE"</p>
                            <h5>"Spend, renewal, and approval compression"</h5>
                        </div>
                        <button type="button" data-finance-command-action="open-vendor" data-finance-route="vendors">"Vendors"</button>
                    </div>
                    <table class="finance-risk-table">
                        <thead><tr><th>"Object"</th><th>"Owner"</th><th>"SLA"</th><th>"Next"</th></tr></thead>
                        <tbody>
                            <tr data-finance-risk-row="true"><td><strong>"Stripe invoice"</strong><small>"approval can collapse to 1-stage"</small></td><td>"AP"</td><td><span class="status-chip danger">"4.0h"</span></td><td><button type="button" data-finance-command-action="route-stripe" data-finance-route="vendors">"Route"</button></td></tr>
                            <tr data-finance-risk-row="true"><td><strong>"AWS reserved capacity"</strong><small>"commit under-run vs kr-seoul gate"</small></td><td>"SRE"</td><td><span class="status-chip warning">"1d"</span></td><td><button type="button" data-finance-command-action="route-aws" data-finance-route="cloud">"FinOps"</button></td></tr>
                            <tr data-finance-risk-row="true"><td><strong>"Payroll bureau"</strong><small>"NHIS tier delta requires reviewer"</small></td><td>"CFO"</td><td><span class="status-chip danger">"today"</span></td><td><button type="button" data-finance-command-action="route-payroll" data-finance-route="workflow">"Workflow"</button></td></tr>
                        </tbody>
                    </table>
                </article>

                <article class="finance-command-card finance-evidence-lane-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"EVIDENCE LANE"</p>
                            <h5>"Reviewer packet and communication routes"</h5>
                        </div>
                        <span class="status-chip success">"sealed draft"</span>
                    </div>
                    <ol class="finance-evidence-lane">
                        <li><span>"REC-PAY-2026-04-PARK"</span><strong>"Payroll delta"</strong><em>"blocking · Finance close"</em></li>
                        <li><span>"REC-TAX-HOMETAX-118"</span><strong>"Withholding transport"</strong><em>"review · CFO desk"</em></li>
                        <li><span>"REC-COMM-GOV-221"</span><strong>"Council note"</strong><em>"ready · Community"</em></li>
                    </ol>
                    <div class="finance-mini-actions">
                        <button type="button" data-finance-command-action="mail-brief" data-finance-route="mail">"Mail brief"</button>
                        <button type="button" data-finance-command-action="messenger-room" data-finance-route="messenger">"Messenger room"</button>
                        <button type="button" data-finance-command-action="council-note" data-finance-route="community">"Community note"</button>
                    </div>
                </article>
            </div>

            <div class="finance-route-matrix" aria-label="Commercial operations route matrix">
                <button type="button" data-finance-command-action="route-ledger" data-finance-route="ledger">"Ledger"</button>
                <button type="button" data-finance-command-action="route-vendors" data-finance-route="vendors">"Vendors"</button>
                <button type="button" data-finance-command-action="route-billing" data-finance-route="billing">"Billing · Tax"</button>
                <button type="button" data-finance-command-action="route-leave" data-finance-route="leave">"Leave · Time"</button>
                <button type="button" data-finance-command-action="route-workflow" data-finance-route="workflow">"Workflow"</button>
                <button type="button" data-finance-command-action="route-mail" data-finance-route="mail">"Mail"</button>
                <button type="button" data-finance-command-action="route-community" data-finance-route="community">"Community"</button>
                <button type="button" data-finance-command-action="route-evidence" data-finance-route="evidence">"Evidence"</button>
            </div>
        </section>
    }
}

fn operator_intelligence_strip(envelope: TenantRenderEnvelope) -> impl IntoView {
    let evidence_count = envelope.approvals.len() + envelope.workflow.nodes.len() + 6;
    let readiness = if envelope.accreditation.healthcare_enabled {
        "Accredited"
    } else {
        "Gated"
    };
    let object_count =
        envelope.modules.len() + envelope.daily_tasks.len() + envelope.community.len() + 7;
    let signal_count =
        envelope.messages.len() + envelope.community.len() + envelope.approvals.len();
    let workflow_name = envelope.workflow.name.clone();
    let workflow_goal = envelope.workflow.goal.clone();

    let evidence_events = [
        (
            "blocking",
            "REC-PAY-2026-04-PARK",
            "Payroll delta needs four-insurance approval",
            "NHIS tier increase detected; owner must approve before April close package seals.",
            "Payroll",
            "Finance close",
            "4.0h",
        ),
        (
            "review",
            "REC-TAX-HOMETAX-118",
            "HomeTax withholding transport waiting",
            "118 employees validated; 사업자등록번호 confirmation remains before send preview.",
            "Tax",
            "CFO desk",
            "1d",
        ),
        (
            "sealed",
            "REC-WF-7741",
            "Workflow receipt staged from Command Center",
            "Tenant change approval produced Messenger, Mail, Community, and audit drafts.",
            "Workflow",
            "Tenant admin",
            "sealed",
        ),
        (
            "review",
            "REC-CLOUD-MESH-4182",
            "Network split rollback evidence requested",
            "us-east-2 mesh split requires regional capacity and rollback runbook attestation.",
            "Cloud Ops",
            "Infrastructure SRE",
            "2.1h",
        ),
        (
            "sealed",
            "REC-COMM-GOV-221",
            "Governance council broadcast prepared",
            "Community note links policy rationale, approval owner, and object graph lineage.",
            "Community",
            "Governance",
            "sealed",
        ),
        (
            "watch",
            "REC-VND-STRIPE-4820",
            "Vendor renewal route can be shortened",
            "Stripe 청구서 approval can move from three-stage to one-stage below policy threshold.",
            "Procurement",
            "AP owner",
            "next run",
        ),
    ];

    let object_nodes = [
        (
            "tenant",
            "Tenant",
            "Northwind Corp.",
            "Authoritative tenant envelope and pack gates",
        ),
        (
            "workflow",
            "Workflow",
            "Tenant change approval",
            "No-code approval path and run preview",
        ),
        (
            "approval",
            "Approval",
            "APR-274",
            "Human reviewer checkpoint before action",
        ),
        (
            "mail",
            "Mail",
            "Finance close brief",
            "Formal approval route draft",
        ),
        (
            "messenger",
            "Messenger",
            "Ops room",
            "Fast coordination thread",
        ),
        (
            "community",
            "Community",
            "Governance council",
            "Role-aware broadcast",
        ),
        (
            "cloud",
            "Cloud cell",
            "us-east-2",
            "Runtime, network, and FinOps posture",
        ),
        (
            "audit",
            "Receipt",
            "REC-WF-7741",
            "Immutable local evidence preview",
        ),
    ];

    view! {
        <section
            id="evidence-spine"
            class="operator-intelligence evidence-intelligence-console panel"
            aria-label="Evidence spine, object graph, and governed copilot intelligence"
        >
            <div class="evidence-console-head">
                <div>
                    <p class="screen-anchor">"EVIDENCE SPINE · OBJECT GRAPH"</p>
                    <h3>"Operational intelligence console"</h3>
                    <p>
                        "Workflow, approvals, Messenger, Mail, Community, cloud operations, and audit receipts are shown as one cohesive local service graph."
                    </p>
                </div>
                <div class="evidence-head-actions" aria-label="Evidence console actions">
                    <span class="status-chip success">"sealed draft"</span>
                    <button type="button" data-evidence-action="run-review">"Run review"</button>
                    <button type="button" data-evidence-action="export">"Export packet"</button>
                </div>
            </div>

            <div class="evidence-kpi-strip" aria-label="Operational intelligence summary">
                <span><strong>{evidence_count}</strong><small>"evidence leaves"</small></span>
                <span><strong>{object_count}</strong><small>"graph objects"</small></span>
                <span><strong>{signal_count}</strong><small>"cross-service signals"</small></span>
                <span><strong>{readiness}</strong><small>"tenant readiness"</small></span>
                <span><strong>"0"</strong><small>"backend writes"</small></span>
            </div>

            <div class="evidence-console-toolbar" aria-label="Evidence ledger filters">
                <label class="evidence-search">
                    <span aria-hidden="true">"⌕"</span>
                    <input data-evidence-search="true" type="search" aria-label="Search evidence ledger" placeholder="Search evidence, owner, object, route..." />
                </label>
                <div class="evidence-filter-pills" role="toolbar" aria-label="Evidence state filters">
                    <button type="button" class="active" data-evidence-filter="all">"All"</button>
                    <button type="button" data-evidence-filter="blocking">"Blocking"</button>
                    <button type="button" data-evidence-filter="review">"Review"</button>
                    <button type="button" data-evidence-filter="sealed">"Sealed"</button>
                    <button type="button" data-evidence-filter="watch">"Watch"</button>
                </div>
                <span class="evidence-console-status" data-evidence-status="true">
                    {format!("{} visible · all states · local evidence only", evidence_events.len())}
                </span>
            </div>

            <div class="evidence-layout">
                <article id="evidence-ledger" class="evidence-ledger-panel" aria-labelledby="evidence-ledger-title">
                    <div class="evidence-panel-head">
                        <div>
                            <p class="screen-anchor">"LEDGER"</p>
                            <h4 id="evidence-ledger-title">"Receipt timeline"</h4>
                        </div>
                        <button type="button" data-evidence-action="attach">"Attach to inbox"</button>
                    </div>
                    <ol class="evidence-event-list">
                        {evidence_events.into_iter().map(|(state, receipt, title, body, source, owner, sla)| {
                            let chip_class = match state {
                                "blocking" => "status-chip danger",
                                "review" => "status-chip warning",
                                "sealed" => "status-chip success",
                                _ => "status-chip",
                            };
                            view! {
                                <li class="evidence-event" data-evidence-event="true" data-evidence-state=state>
                                    <button
                                        type="button"
                                        class="evidence-event-main"
                                        data-evidence-action="open"
                                        data-sidepeek-trigger="evidence"
                                        data-sidepeek-title=title
                                        data-sidepeek-id=receipt
                                        data-sidepeek-desc=body
                                        data-sidepeek-owner=owner
                                        data-sidepeek-risk=state
                                        data-sidepeek-sla=sla
                                    >
                                        <span class=chip_class>{state}</span>
                                        <strong>{title}</strong>
                                        <p>{body}</p>
                                    </button>
                                    <dl>
                                        <div><dt>"Source"</dt><dd>{source}</dd></div>
                                        <div><dt>"Owner"</dt><dd>{owner}</dd></div>
                                        <div><dt>"SLA"</dt><dd>{sla}</dd></div>
                                    </dl>
                                </li>
                            }
                        }).collect_view()}
                    </ol>
                    {evidence_ledger_anchor_board()}
                </article>

                <article id="object-graph" class="object-graph-panel" aria-labelledby="object-graph-title">
                    <div class="evidence-panel-head">
                        <div>
                            <p class="screen-anchor">"OBJECT GRAPH"</p>
                            <h4 id="object-graph-title">"Tenant operation lineage"</h4>
                        </div>
                        <span data-object-status="true">"Tenant selected · 8 linked objects"</span>
                    </div>
                    <div class="object-graph-canvas" aria-label="Selectable object graph preview">
                        {object_nodes.into_iter().enumerate().map(|(index, (key, label, value, desc))| {
                            let node_class = if index == 0 { "object-node active" } else { "object-node" };
                            view! {
                                <button
                                    type="button"
                                    class=node_class
                                    data-object-node=key
                                    data-object-label=label
                                    data-sidepeek-trigger="object-graph"
                                    data-sidepeek-title=label
                                    data-sidepeek-id=format!("OBJ-{}", key.to_ascii_uppercase())
                                    data-sidepeek-desc=desc
                                    data-sidepeek-owner="Object graph"
                                    data-sidepeek-risk="Read-only"
                                    data-sidepeek-sla="Local data"
                                    aria-label=format!("Open object node {label}")
                                >
                                    <span>{label}</span>
                                    <strong>{value}</strong>
                                </button>
                            }
                        }).collect_view()}
                        <svg viewBox="0 0 640 280" aria-hidden="true" class="object-graph-links">
                            <path d="M92 60 C210 40 250 112 324 116 S484 88 552 64" />
                            <path d="M96 118 C190 168 266 158 326 116" />
                            <path d="M324 116 C388 146 452 168 550 166" />
                            <path d="M322 116 C302 198 374 230 548 226" />
                            <path d="M92 222 C190 232 252 204 324 116" />
                        </svg>
                    </div>
                    <div class="object-graph-table" aria-label="Object graph properties">
                        <dl>
                            <div><dt>"Graph root"</dt><dd>{workflow_name.clone()}</dd></div>
                            <div><dt>"Primary output"</dt><dd>"Task · message · evidence draft"</dd></div>
                            <div><dt>"Autonomy ceiling"</dt><dd>"No auto-execution"</dd></div>
                            <div><dt>"Region"</dt><dd>"us-east-2 active · kr-seoul pack gated"</dd></div>
                        </dl>
                    </div>
                    {object_graph_anchor_board()}
                </article>

                <aside class="copilot-rail-panel" aria-labelledby="intel-copilot-title">
                    <div class="evidence-panel-head compact">
                        <div>
                            <p class="screen-anchor">"COPILOT RAIL"</p>
                            <h4 id="intel-copilot-title">"Governed next moves"</h4>
                        </div>
                        <span class="status-chip ai">"PIPA-safe"</span>
                    </div>
                    <div class="intel-action-stack">
                        <button type="button" data-intel-action="audit">
                            <strong>"Draft audit brief"</strong>
                            <span>"Bundle payroll, HomeTax, workflow, and cloud evidence into one reviewer packet."</span>
                        </button>
                        <button type="button" data-intel-action="workflow">
                            <strong>"Simulate critical path"</strong>
                            <span>"Preview CFO escalation and Mail/Messenger/Community outputs before any execution."</span>
                        </button>
                        <button type="button" data-intel-action="mail">
                            <strong>"Compose approval mail"</strong>
                            <span>"Open the built-in Mail surface with receipt links and owner context."</span>
                        </button>
                        <button type="button" data-intel-action="community">
                            <strong>"Post council note"</strong>
                            <span>"Route a governance update to Community without leaving the console."</span>
                        </button>
                    </div>
                    <p class="copilot-rail-status" data-copilot-rail-status="true">
                        "Read-only recommendations; every action changes local visual state only."
                    </p>
                </aside>

                <article class="signal-lineage-panel" aria-labelledby="signal-lineage-title">
                    <div class="evidence-panel-head">
                        <div>
                            <p class="screen-anchor">"CROSS-MODULE SIGNAL"</p>
                            <h4 id="signal-lineage-title">{workflow_name}</h4>
                        </div>
                        <span class="status-chip warning">"review path"</span>
                    </div>
                    <p>{workflow_goal}</p>
                    <ol class="signal-lineage">
                        <li class="root"><span>"Workflow"</span><strong>"Tenant change approval"</strong><em>"root event"</em></li>
                        <li><span>"Messenger"</span><strong>"Ops room update"</strong><em>"drafted"</em></li>
                        <li><span>"Mail"</span><strong>"Finance approval brief"</strong><em>"ready"</em></li>
                        <li><span>"Community"</span><strong>"Governance council note"</strong><em>"review"</em></li>
                        <li><span>"Cloud Ops"</span><strong>"Rollback evidence"</strong><em>"blocking"</em></li>
                        <li><span>"Audit"</span><strong>"Receipt spine"</strong><em>"sealed draft"</em></li>
                    </ol>
                    <div class="lineage-actions">
                        <button type="button" data-intel-action="messenger">"Messenger"</button>
                        <button type="button" data-intel-action="mail">"Mail"</button>
                        <button type="button" data-intel-action="community">"Community"</button>
                        <button type="button" data-intel-action="audit">"Audit ledger"</button>
                    </div>
                </article>
            </div>
        </section>
    }
}

fn evidence_ledger_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board" aria-label="FD-001 evidence ledger and Oyatie Cloud substrate proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="ledger-fd001">
                    <p class="screen-anchor">"FD-001 RECEIPT SPINE"</p>
                    <h5>"Tenant workload delivery remains the master-plan goal"</h5>
                    <p>
                        "Messenger, Mail, Community, Workflow, Finance, and Daily Work receipts stay one FD-001 tenant workload packet rather than disconnected modules."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="seal-receipt">"Seal packet"</button>
                        <button type="button" data-trust-proof-action="route-workflow">"Workflow proof"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD ADMISSION"</p>
                    <h5>"Substrate proves real tenant hosting"</h5>
                    <p>
                        "Every cloud cell, policy grant, release gate, and FinOps signal attaches evidence before FD-001 services can claim production readiness."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud cells"</button>
                        <button type="button" data-trust-proof-action="route-gates">"Release gates"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-local">
                    <p class="screen-anchor">"LOCAL-ONLY RECEIPT VAULT"</p>
                    <h5>"Interactive trust without mutation"</h5>
                    <p>
                        "Operators can inspect, seal, brief, and route receipts visually; no backend write, deploy, billing, mail, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-mail">"Mail brief"</button>
                        <button type="button" data-trust-proof-action="route-community">"Community note"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Evidence ledger ready · FD-001 tenant workload receipts dogfood Oyatie Cloud locally."
                </span>
                <div class="trust-anchor-routes" aria-label="Evidence ledger connected routes">
                    <button type="button" data-trust-proof-action="route-finops">"FinOps"</button>
                    <button type="button" data-trust-proof-action="route-inventory">"Inventory"</button>
                    <button type="button" data-trust-proof-action="route-graph">"Object graph"</button>
                    <button type="button" data-trust-proof-action="route-policy">"Policy"</button>
                </div>
            </div>
        </div>
    }
}

fn object_graph_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board" aria-label="FD-001 object graph and Oyatie Cloud substrate proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="graph-fd001">
                    <p class="screen-anchor">"FD-001 OBJECT MODEL"</p>
                    <h5>"One service graph spans every surface"</h5>
                    <p>
                        "Workflow, approvals, Messenger, Mail, Community, Finance, Daily Work, and audit nodes resolve to a single tenant operation lineage."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="trace-lineage">"Trace lineage"</button>
                        <button type="button" data-trust-proof-action="route-catalog">"Catalog objects"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="graph-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD GRAPH"</p>
                    <h5>"Substrate nodes join product nodes"</h5>
                    <p>
                        "Cells, resources, policies, deployment gates, FinOps, and receipts prove FD-001 microservices can run as production tenant workloads."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud topology"</button>
                        <button type="button" data-trust-proof-action="route-policy">"Policy edge"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="graph-local">
                    <p class="screen-anchor">"LOCAL-ONLY GRAPH OPS"</p>
                    <h5>"Selectable lineage, no side effects"</h5>
                    <p>
                        "Operators can traverse graph edges, stage evidence, and open communications visually; no database, workflow, deploy, or cloud mutation occurs."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-evidence">"Evidence spine"</button>
                        <button type="button" data-trust-proof-action="route-mail">"Reviewer Mail"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Object graph ready · FD-001 services and Oyatie Cloud substrate stay one local visual lineage."
                </span>
                <div class="trust-anchor-routes" aria-label="Object graph connected routes">
                    <button type="button" data-trust-proof-action="route-workflow">"Workflow"</button>
                    <button type="button" data-trust-proof-action="route-inventory">"Resources"</button>
                    <button type="button" data-trust-proof-action="route-community">"Community"</button>
                    <button type="button" data-trust-proof-action="route-finops">"FinOps"</button>
                </div>
            </div>
        </div>
    }
}

fn audit_evidence_timeline_from_durable_source() -> CloudObservabilityAuditReadSuccessResponse {
    CloudObservabilityAuditReadSuccessResponse::from_json(CLOUD_OBSERVABILITY_AUDIT_READ_JSON)
        .expect("static cloud observability audit read JSON must match the durable API contract")
}

fn audit_evidence_timeline_panel() -> impl IntoView {
    let source = audit_evidence_timeline_from_durable_source();
    let rows = source.timeline_rows();
    let chain_complete = source.metadata.chain_complete.to_string();
    let high_watermark_sequence = source.high_watermark_sequence_label();
    let record_count = source.metadata.record_count.to_string();
    let request_id = source.metadata.request_id.clone();
    let tenant_id = source.metadata.tenant_id.clone();
    let region = source.metadata.region.clone();
    let next_cursor = source
        .metadata
        .next_cursor
        .clone()
        .unwrap_or_else(|| "none".to_owned());

    view! {
        <section
            id="audit-evidence-timeline"
            class="audit-evidence-timeline-panel panel"
            aria-labelledby="audit-evidence-timeline-title"
            data-audit-evidence-source="durable-cloud-observability-audit-api"
            data-audit-evidence-endpoint=source.endpoint()
            data-authz-surface=source.authz_surface()
            data-audit-operation-class=source.operation_class()
            data-authority-ref=source.authority_ref()
            data-source-anchors=source.source_anchors()
            data-authority-boundary=source.authority_boundary()
            data-chain-complete=chain_complete
            data-high-watermark-sequence=high_watermark_sequence
        >
            <div class="panel-header audit-evidence-header">
                <div>
                    <p class="eyebrow">"Audit Evidence · durable source"</p>
                    <h3 id="audit-evidence-timeline-title">"Cloud IAM policy evidence timeline"</h3>
                    <p>
                        "Read-only projection from the Cloud Observability audit read API; the UI renders signed digest-chain evidence for one operation class without provider mutation."
                    </p>
                    <p class="scope-notice">
                        "Authority: contracts/openapi/cloud/cloud-observability-audit-v1.yaml · G007/AUTHZ-006 slice · analysis-only-non-mutating."
                    </p>
                </div>
                <span class="status-chip success">"chain complete"</span>
            </div>
            <div class="audit-evidence-source-strip" aria-label="Durable audit source metadata">
                <span><strong>{record_count}</strong><small>"records"</small></span>
                <span><strong>{request_id}</strong><small>"request"</small></span>
                <span><strong>{tenant_id}</strong><small>"tenant"</small></span>
                <span><strong>{region}</strong><small>"region"</small></span>
                <span><strong>{next_cursor}</strong><small>"next cursor"</small></span>
            </div>
            <AuditEvidenceTimeline variant=TimelineVariant::ChangesetProvenance rows=rows />
        </section>
    }
}

fn ops_cluster_health_from_typed_api() -> ClusterHealthStatus {
    ClusterHealthStatus::from_json(OPS_CLUSTER_HEALTH_FIXTURE_JSON)
        .expect("static ops cluster health fixture must match the OpenAPI contract")
}

fn ops_cluster_health_panel_status() -> DeploymentStatus {
    ops_cluster_health_from_typed_api().to_deployment_status()
}

fn ops_cluster_health_panel() -> impl IntoView {
    let api_status = ops_cluster_health_from_typed_api();
    let endpoint = api_status.endpoint();
    let status = api_status.to_deployment_status();

    view! {
        <div
            class="ops-cluster-health-panel"
            data-ops-cluster-health-source="typed-api"
            data-ops-cluster-health-endpoint=endpoint
        >
            <OpsDeploymentStatusPanel status=status />
        </div>
    }
}

fn control_plane_resource_explorer_operation_center() -> impl IntoView {
    view! {
        <section
            id="resource-explorer"
            class="panel resource-explorer-panel"
            data-fixture-kind="resource-explorer-resource"
            aria-labelledby="resource-explorer-title"
        >
            <div class="panel-header">
                <div>
                    <p class="eyebrow">"Control plane · fixture only"</p>
                    <h3 id="resource-explorer-title">"Resource Explorer"</h3>
                    <p>"Read-only tenant-scoped index for one representative resource type; no provider CRUD, billing, quota, Cedar runtime, or production persistence is wired."</p>
                </div>
                <span class="status-chip warning">"fixture-only non-claim"</span>
            </div>
            <p class="scope-notice">"Organization → Account → Project → Region → Cell → Resource Group → Resource"</p>
            <div class="catalog-toolbar" aria-label="Resource Explorer filters">
                <label class="catalog-search">
                    <span aria-hidden="true">"⌕"</span>
                    <input
                        type="search"
                        aria-label="Search ORN"
                        value="orn:oya:us-east-2:acct-northwind-prod:cloud-compute:k8s-cluster/cluster-nw-app-01"
                    />
                </label>
                <span class="catalog-status">"GET /v1/cloud-control-plane/resources · X-Oyatie-API-Version: 2026-07-01 (mock)"</span>
            </div>
            <article class="resource-explorer-card" data-resource-type="K8sCluster">
                <span class="status-chip success">"K8sCluster"</span>
                <strong>"cluster-nw-app-01"</strong>
                <code>"orn:oya:us-east-2:acct-northwind-prod:cloud-compute:k8s-cluster/cluster-nw-app-01"</code>
                <dl>
                    <div><dt>"Organization"</dt><dd>"org-oyatie-northwind"</dd></div>
                    <div><dt>"Account"</dt><dd>"acct-northwind-prod"</dd></div>
                    <div><dt>"Project"</dt><dd>"prj-fd001-control"</dd></div>
                    <div><dt>"Region / Cell"</dt><dd>"us-east-2 / cell-us-east-2a"</dd></div>
                    <div><dt>"Resource group"</dt><dd>"rg-platform-admin"</dd></div>
                    <div><dt>"Owner"</dt><dd>"tenant-admin-platform"</dd></div>
                </dl>
            </article>
            <div class="resource-facet-grid" aria-label="Resource detail facets">
                <span data-resource-facet="lifecycle">"lifecycle · ACTIVE fixture"</span>
                <span data-resource-facet="identity">"identity · workload principal reference only"</span>
                <span data-resource-facet="policy">"policy · Cedar default-deny expectation"</span>
                <span data-resource-facet="quota">"quota · mock CPU/memory ceiling"</span>
                <span data-resource-facet="billing">"billing · mock meter labels only"</span>
                <span data-resource-facet="audit">"audit · REC-CTRL-001A"</span>
                <span data-resource-facet="observability">"observability · trace fixture"</span>
                <span data-resource-facet="rollback">"rollback · compensation pending"</span>
                <span data-resource-facet="reconciliation">"reconciliation · controller-owned later"</span>
            </div>
            <aside class="resource-access-rules" aria-label="Data access rules">
                <p>"Cedar default-deny expectation"</p>
                <p>"Cross-tenant rows are redacted"</p>
                <p>"Support access is constrained"</p>
                <p>"No secret material is exposed"</p>
            </aside>
        </section>
        <section
            id="operation-center"
            class="panel operation-center-panel"
            data-fixture-kind="operation-timeline-entry"
            aria-labelledby="operation-center-title"
        >
            <div class="panel-header">
                <div>
                    <p class="eyebrow">"Operation Center · AIP-151 fixture"</p>
                    <h3 id="operation-center-title">"Operation Center"</h3>
                    <p>"Operation-ledger timeline for create/update/delete attempts; read-only fixture rows until API-001/resource-provider integration."</p>
                </div>
                <span class="status-chip ai">"jsonschema.cloud-control-plane.fixture.operation-timeline-entry"</span>
            </div>
            <ol class="ops-timeline" aria-label="Operation ledger timeline">
                <li>
                    <time>"2026-07-01T09:18:00Z"</time>
                    <strong>"op-20260701-0001 · current state accepted · fixture row"</strong>
                    <span>"client idempotency key idem-ctrlplane-001 · actor tenant-admin-platform · requested mutation create K8sCluster"</span>
                    <span>"timestamps accepted/queued/running fixture · retry classification policy · rollback / compensation pending"</span>
                    <span>"audit-chain pointer REC-CTRL-001A · error / remediation metadata policy reviewer required"</span>
                </li>
                <li>
                    <time>"2026-07-01T09:42:00Z"</time>
                    <strong>"op-20260701-0002 · current state failed · policy denied fixture row"</strong>
                    <span>"client idempotency key idem-ctrlplane-002 · actor support-engineer-redacted · requested mutation update labels"</span>
                    <span>"timestamps accepted/failed fixture · retry classification operator_required · rollback / compensation not-started"</span>
                    <span>"audit-chain pointer REC-CTRL-001B · error / remediation metadata support scope redacted"</span>
                </li>
            </ol>
            <div class="policy-decision-strip" aria-label="Persona user-story paths">
                <article><strong>"tenant admin path"</strong><p>"Search ORN, inspect facets, route reviewer work without live mutation."</p></article>
                <article><strong>"operator path"</strong><p>"Inspect operation retries, rollback state, and remediation metadata."</p></article>
                <article><strong>"developer path"</strong><p>"Use the ORN and operation id from the service catalog/dev portal flow."</p></article>
                <article><strong>"security/compliance reviewer path"</strong><p>"Verify audit, policy, retention, jurisdiction, and no-secret posture."</p></article>
                <article><strong>"support engineer path"</strong><p>"See constrained redacted support context without cross-tenant leakage."</p></article>
            </div>
            <p class="cockpit-note">"ADR-0536 D-4/D-5 · specs/masterplan.json:144-257 · specs/cloud-control-plane-canonical.json · specs/cloud-resource-catalog-target.json · specs/api-contract-ssot-canonical.json · ADR-0393 · CONTROLPLANE-UX-001 spec comment"</p>
        </section>
    }
}

fn support_advisory_case_console(fixture: SupportAdvisoryFixture) -> impl IntoView {
    let SupportAdvisoryFixture {
        case_id,
        tenant_id,
        resource_ref,
        service_ref,
        severity,
        response_target_label,
        entitlement_plan,
        customer_state,
        incident_refs,
        status_refs,
        trust_evidence_refs,
        governance_posture_refs,
        finops_refs,
        onboarding_refs,
        audit_chain_refs,
        diagnostic_bundle_ref,
        diagnostic_bundle_evidence_refs,
        advisor_key,
        advisor_severity,
        advisor_owner,
        recommended_action,
        non_goal_copy,
        expires_at,
        support_access_state,
        support_engineer_view,
        post_case_actions,
        api_records,
        security_assertions,
        authority_boundaries,
    } = fixture;
    let support_access_state_summary = support_access_state.clone();
    let linked_refs = incident_refs
        .into_iter()
        .chain(status_refs)
        .chain(trust_evidence_refs)
        .chain(governance_posture_refs)
        .chain(finops_refs)
        .chain(onboarding_refs)
        .chain(audit_chain_refs);

    view! {
        <section
            id="support-advisory-console"
            class="panel support-advisory-console"
            data-fixture-kind="support-advisory-case"
            aria-labelledby="support-advisory-title"
        >
            <div class="panel-header">
                <div>
                    <p class="eyebrow">"Customer Support + Trusted Advisor · fixture only"</p>
                    <h3 id="support-advisory-title">"Support case diagnostic bundle and advisory"</h3>
                    <p>"Tenant admin opens a case, selects severity, attaches a redacted diagnostic bundle preview, sees a trusted-advisor recommendation, and tracks post-case action items without live ticketing or notification delivery."</p>
                </div>
                <span class="status-chip warning">"fixture-only non-claim"</span>
            </div>
            <div class="catalog-kpi-strip" aria-label="Support entitlement and severity summary">
                <span><strong>{severity}</strong><small>{response_target_label}</small></span>
                <span><strong>"Plan"</strong><small>{entitlement_plan}</small></span>
                <span><strong>"State"</strong><small>{customer_state}</small></span>
                <span><strong>"Access"</strong><small>{support_access_state_summary}</small></span>
            </div>
            <div class="devportal-story-grid">
                <article aria-label="Support case story">
                    <p class="screen-anchor">"support_case"</p>
                    <h5>{case_id}</h5>
                    <dl class="service-kv">
                        <div><dt>"Tenant"</dt><dd>"tenant_id="{tenant_id}</dd></div>
                        <div><dt>"Resource"</dt><dd>{resource_ref}</dd></div>
                        <div><dt>"Service"</dt><dd>{service_ref}</dd></div>
                        <div><dt>"Engineer view"</dt><dd>{support_engineer_view}</dd></div>
                    </dl>
                    <button type="button" data-support-action="attach-bundle">"Attach diagnostic bundle preview"</button>
                    <button type="button" data-support-action="select-severity">"Select severity"</button>
                </article>
                <article aria-label="Diagnostic bundle allowed refs">
                    <p class="screen-anchor">"diagnostic_bundle"</p>
                    <h5>{diagnostic_bundle_ref}</h5>
                    <div class="devportal-resource-pills" aria-label="Support linked refs">
                        {linked_refs.map(|reference| view! { <span class="status-chip">{reference}</span> }).collect_view()}
                    </div>
                    <ul>
                        {diagnostic_bundle_evidence_refs.into_iter().map(|reference| view! { <li>{reference}</li> }).collect_view()}
                    </ul>
                </article>
                <article aria-label="Trusted advisor recommendation">
                    <p class="screen-anchor">"advisor_recommendation"</p>
                    <h5>{advisor_key}</h5>
                    <dl class="service-kv">
                        <div><dt>"Severity"</dt><dd>{advisor_severity}</dd></div>
                        <div><dt>"Owner"</dt><dd>{advisor_owner}</dd></div>
                        <div><dt>"Freshness"</dt><dd>{expires_at}</dd></div>
                    </dl>
                    <p>{recommended_action}</p>
                    <p>{non_goal_copy}</p>
                </article>
            </div>
            <table class="finance-table" aria-label="Support API fixture records">
                <thead><tr><th>"Record"</th><th>"ID"</th><th>"Tenant"</th><th>"Actor"</th><th>"Purpose"</th><th>"Data class"</th><th>"Freshness"</th><th>"Redaction"</th><th>"Audit"</th></tr></thead>
                <tbody>
                    {api_records.into_iter().map(|record| view! {
                        <tr data-support-record=record.record_type.clone()>
                            <th scope="row">{record.record_type.clone()}</th>
                            <td>{record.record_id}</td>
                            <td>{record.tenant_id}</td>
                            <td>{record.actor}</td>
                            <td>{record.purpose}</td>
                            <td>{record.data_class}</td>
                            <td>{record.freshness}</td>
                            <td>{record.redaction_policy_id}</td>
                            <td>{"audit_event_ref="}{record.audit_event_ref}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
            <div class="resource-access-rules" aria-label="Support redaction and authority boundaries">
                <p>"approval-gated breakglass"</p>
                {security_assertions.into_iter().map(|assertion| view! { <p>{assertion}</p> }).collect_view()}
                {authority_boundaries.into_iter().map(|boundary| view! { <p>{boundary}</p> }).collect_view()}
            </div>
            <ol class="ops-timeline" aria-label="Customer communication and post-case action items">
                <li><strong>"customer_communication"</strong><span>"Customer-visible update is staged; external send is disabled."</span></li>
                <li><strong>"support_access_approval"</strong><span>{support_access_state}</span></li>
                {post_case_actions.into_iter().map(|action| view! { <li><strong>"post_case_action_item"</strong><span>{action}</span></li> }).collect_view()}
            </ol>
        </section>
    }
}

fn tenant_operations_cockpit(envelope: TenantRenderEnvelope) -> impl IntoView {
    let healthcare_gate = if envelope.accreditation.healthcare_enabled {
        "Healthcare surfaces enabled"
    } else {
        "Healthcare surfaces gated"
    };
    let visible_modules = envelope.modules.len();

    view! {
        <section id="cloud-ops-cockpit" class="ops-cockpit panel" aria-labelledby="ops-cockpit-title">
            <div class="panel-header cockpit-header">
                <div>
                    <p class="eyebrow">"Operate"</p>
                    <h3 id="ops-cockpit-title">"Cloud, policy, and FinOps cockpit"</h3>
                </div>
                <div class="cockpit-tabs" role="tablist" aria-label="Operations cockpit views">
                    <button type="button" class="active" data-cockpit-tab="topology" role="tab" aria-selected="true">"Topology"</button>
                    <button type="button" data-cockpit-tab="policy" role="tab" aria-selected="false">"Policy"</button>
                    <button type="button" data-cockpit-tab="finops" role="tab" aria-selected="false">"FinOps"</button>
                </div>
            </div>

            <div class="cockpit-panels">
                <article class="cockpit-panel active" data-cockpit-panel="topology" aria-labelledby="cloud-topology-title">
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"CLOUD TOPOLOGY"</p>
                        <h4 id="cloud-topology-title">"Tenant runtime map"</h4>
                    </div>
                    <div class="topology-map" aria-hidden="true">
                        <span class="region primary">"us-east-2"<em>"cell active"</em></span>
                        <span class="region">"eu-west-1"<em>"warm standby"</em></span>
                        <span class="region">"kr-seoul"<em>"pack gated"</em></span>
                        <span class="service compute">"Compute"</span>
                        <span class="service network">"Network"</span>
                        <span class="service storage">"Storage"</span>
                        <span class="service audit">"Audit chain"</span>
                    </div>
                    {cloud_ops_command_matrix()}
                    {ops_cluster_health_panel()}
                    <div class="ops-metrics-strip" aria-label="Cloud operations live posture">
                        <span><small>"Availability"</small><strong>"99.96%"</strong><em>"+0.01 vs SLO"</em></span>
                        <span><small>"Pending rollbacks"</small><strong>"2"</strong><em>"1 network · 1 key"</em></span>
                        <span><small>"Run-rate"</small><strong>"$48.2k"</strong><em>"4% under commit"</em></span>
                        <span><small>"Evidence age"</small><strong>"12m"</strong><em>"fresh"</em></span>
                    </div>
                    <div class="topology-detail-grid">
                        <article>
                            <p class="screen-anchor">"INCIDENT THREAD"</p>
                            <ol class="ops-timeline">
                                <li><time>"09:18"</time><strong>"Mesh split detected"</strong><span>"northwind-prod-mesh · rollback evidence requested"</span></li>
                                <li><time>"09:42"</time><strong>"DNS policy verified"</strong><span>"tenant-control-plane routes stay global"</span></li>
                                <li><time>"10:05"</time><strong>"Audit sidecar healthy"</strong><span>"receipt vault sealed draft attached"</span></li>
                            </ol>
                        </article>
                        <article>
                            <p class="screen-anchor">"RUNBOOK QUEUE"</p>
                            <div class="runbook-list">
                                <button type="button" data-cockpit-action="reconcile-cell">"Reconcile cell evidence"</button>
                                <button type="button" data-cockpit-action="simulate-failover">"Simulate failover"</button>
                                <button type="button" data-cockpit-action="queue-runbook">"Queue rollback runbook"</button>
                            </div>
                        </article>
                        <article>
                            <p class="screen-anchor">"REGIONAL CAPACITY"</p>
                            <div class="capacity-bars">
                                <span style="--bar: 72%"><em>"us-east-2"</em></span>
                                <span style="--bar: 44%"><em>"eu-west-1"</em></span>
                                <span style="--bar: 28%"><em>"kr-seoul"</em></span>
                            </div>
                        </article>
                    </div>
                    <div class="cockpit-actions">
                        <button
                            type="button"
                            data-sidepeek-trigger="topology"
                            data-sidepeek-title="Tenant runtime map"
                            data-sidepeek-id="CELL-US-EAST-2"
                            data-sidepeek-desc="Primary cell running compute, network, storage, and audit-chain staged surfaces."
                            data-sidepeek-owner="Cloud infrastructure"
                            data-sidepeek-risk="Medium"
                            data-sidepeek-sla="99.95% target · local data"
                        >
                            "Inspect cell"
                        </button>
                        <button type="button" data-command-trigger="true">"Search resources"</button>
                        <span class="cockpit-status" data-cockpit-status="true">"Topology ready · local runbooks only."</span>
                    </div>
                </article>

                <article id="policy-access" class="cockpit-panel" data-cockpit-panel="policy" aria-labelledby="policy-access-title">
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"POLICY & ACCESS"</p>
                        <h4 id="policy-access-title">"Policy envelope command board"</h4>
                    </div>
                    <div class="policy-command-grid" aria-label="FD-001 and Oyatie Cloud policy proof">
                        <article class="policy-command-card selected" data-policy-card="fd001">
                            <div>
                                <p class="screen-anchor">"FD-001 TENANT"</p>
                                <h5>"Product delivery stays the goal"</h5>
                                <p>
                                    "Messenger, Mail, Community, Workflow, Ontology, and Intelligence run as tenant workloads; "
                                    "Oyatie Cloud proves they can be hosted without moving the tenant workload north star."
                                </p>
                            </div>
                            <div class="policy-command-actions" aria-label="FD-001 policy routes">
                                <button type="button" data-policy-anchor-action="role-review">"Review role grants"</button>
                                <button type="button" data-policy-anchor-action="open-identity">"Open identity"</button>
                                <button type="button" data-policy-anchor-action="route-evidence">"Evidence spine"</button>
                            </div>
                        </article>
                        <article class="policy-command-card" data-policy-card="substrate">
                            <div>
                                <p class="screen-anchor">"OYATIE CLOUD"</p>
                                <h5>"Dogfood substrate boundary"</h5>
                                <p>
                                    "Cloud controls stay tenant-scoped, PIPA-aware, auditable, and local-only until "
                                    "real FD-001 services are admitted through release gates."
                                </p>
                            </div>
                            <div class="policy-command-actions" aria-label="Oyatie Cloud policy routes">
                                <button type="button" data-policy-anchor-action="route-cloud">"Cloud topology"</button>
                                <button type="button" data-policy-anchor-action="pipa-boundary">"PIPA boundary"</button>
                                <button type="button" data-policy-anchor-action="open-audit">"Audit trail"</button>
                            </div>
                        </article>
                        <article class="policy-command-card" data-policy-card="autonomy">
                            <div>
                                <p class="screen-anchor">"AUTONOMY CEILING"</p>
                                <h5>"Interactive, never wired"</h5>
                                <p>
                                    "Policy can preview allow, gate, deny, rollback, and reviewer paths, but every action is "
                                    "visual state with no cloud, billing, DNS, or workflow mutation."
                                </p>
                            </div>
                            <div class="policy-command-actions" aria-label="Autonomy policy routes">
                                <button type="button" data-policy-anchor-action="autonomy-ceiling">"Show ceiling"</button>
                                <button type="button" data-policy-anchor-action="residency">"Residency pack"</button>
                                <button type="button" data-policy-anchor-action="route-mail">"Mail brief"</button>
                            </div>
                        </article>
                    </div>
                    <table class="policy-table">
                        <thead>
                            <tr><th>"Subject"</th><th>"Scope"</th><th>"Decision"</th><th>"Reason"</th></tr>
                        </thead>
                        <tbody>
                            <tr><td>"Tenant admin"</td><td>"Cloud controls"</td><td><span class="status-chip success">"Allow"</span></td><td>"Owner role"</td></tr>
                            <tr><td>{envelope.role_name.clone()}</td><td>"Healthcare"</td><td><span class="status-chip warning">{healthcare_gate}</span></td><td>"Accreditation"</td></tr>
                            <tr><td>"Workflow builder"</td><td>"Execution"</td><td><span class="status-chip danger">"Deny"</span></td><td>"Autonomy ceiling"</td></tr>
                        </tbody>
                    </table>
                    <div class="policy-evidence-grid">
                        <span><strong>"12"</strong><small>"Cedar rules mirrored"</small></span>
                        <span><strong>"7"</strong><small>"tenant pack grants"</small></span>
                        <span><strong>"3"</strong><small>"human review stops"</small></span>
                    </div>
                    <div class="policy-decision-strip" aria-label="Policy decision proof path">
                        <article class="policy-decision-card" data-policy-card="allow">
                            <span class="status-chip success">"Allow"</span>
                            <strong>"Tenant admin → Cloud controls"</strong>
                            <p>"Owner-scoped controls stay inside the dogfood substrate and attach receipt IDs before promotion."</p>
                            <button type="button" data-policy-anchor-action="route-cloud">"Inspect controls"</button>
                        </article>
                        <article class="policy-decision-card" data-policy-card="gate">
                            <span class="status-chip warning">"Gate"</span>
                            <strong>{format!("{} → regulated data", envelope.role_name.clone())}</strong>
                            <p>{format!("{healthcare_gate} · reviewer evidence and residency pack required before any FD-001 workload placement.")}</p>
                            <button type="button" data-policy-anchor-action="residency">"Review gate"</button>
                        </article>
                        <article class="policy-decision-card" data-policy-card="deny">
                            <span class="status-chip danger">"Deny"</span>
                            <strong>"Workflow builder → execution"</strong>
                            <p>"The autonomy ceiling blocks real execution; visual routing proves the UX without wiring side effects."</p>
                            <button type="button" data-policy-anchor-action="autonomy-ceiling">"Trace denial"</button>
                        </article>
                    </div>
                    <div class="policy-anchor-footer">
                        <span class="cockpit-status" data-policy-anchor-status="true">
                            "Policy board ready · FD-001 workloads dogfood Oyatie Cloud as tenant surfaces."
                        </span>
                        <div class="policy-anchor-routes" aria-label="Connected policy routes">
                            <button type="button" data-policy-anchor-action="route-community">"Community review"</button>
                            <button type="button" data-policy-anchor-action="open-audit">"Audit ledger"</button>
                            <button type="button" data-policy-anchor-action="route-evidence">"Evidence graph"</button>
                        </div>
                    </div>
                </article>

                <article id="finops-pane" class="cockpit-panel" data-cockpit-panel="finops" aria-labelledby="finops-title">
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"FINOPS"</p>
                        <h4 id="finops-title">"Run-rate and sustainability"</h4>
                    </div>
                    <div class="finops-bars" aria-label="FinOps breakdown">
                        <span style="--bar: 72%"><em>"Compute · $21.4k"</em></span>
                        <span style="--bar: 51%"><em>"Network · $9.8k"</em></span>
                        <span style="--bar: 43%"><em>"Storage · $7.2k"</em></span>
                        <span style="--bar: 26%"><em>"Audit · $3.1k"</em></span>
                    </div>
                    <div class="finops-action-grid">
                        <button type="button" data-cockpit-action="open-commit">"Open commit plan"</button>
                        <button type="button" data-cockpit-action="tag-anomaly">"Tag anomaly"</button>
                        <button type="button" data-cockpit-action="draft-budget-note">"Draft budget note"</button>
                    </div>
                    <span class="cockpit-status" data-cockpit-status="true">"FinOps ready · local budget actions only."</span>
                    {finops_anchor_board()}
                    <p class="cockpit-note">{format!("{visible_modules} services visible in this envelope · backend wiring remains disabled")}</p>
                </article>
            </div>
        </section>
    }
}

fn cloud_ops_command_matrix() -> impl IntoView {
    view! {
        <div class="ops-command-matrix" aria-label="Cloud operations command matrix">
            <section class="ops-command-card ops-cell-card">
                <div class="ops-command-card-head">
                    <div>
                        <p class="screen-anchor">"CELL CONTROL"</p>
                        <h5>"Runtime cells, residency, and rollback posture"</h5>
                    </div>
                    <span class="status-chip warning">"2 guardrails"</span>
                </div>
                <div class="ops-cell-grid" role="list" aria-label="Regional cell state">
                    <button type="button" class="active" data-cockpit-action="select-us-east">
                        <strong>"us-east-2"</strong><span>"primary"</span><em>"99.96% · 72% cap"</em>
                    </button>
                    <button type="button" data-cockpit-action="select-eu-west">
                        <strong>"eu-west-1"</strong><span>"standby"</span><em>"warm · 44% cap"</em>
                    </button>
                    <button type="button" data-cockpit-action="select-kr-seoul">
                        <strong>"kr-seoul"</strong><span>"pack gated"</span><em>"residency review"</em>
                    </button>
                </div>
                <dl class="ops-command-kv">
                    <div><dt>"Residency"</dt><dd>"KR pack gated before workload placement"</dd></div>
                    <div><dt>"Rollback"</dt><dd>"Network split runbook needs reviewer evidence"</dd></div>
                    <div><dt>"Audit sidecar"</dt><dd>"Receipt vault sealed draft attached"</dd></div>
                </dl>
            </section>

            <section class="ops-command-card ops-workload-card ops-tenant-plane" data-cloud-workload-plane="true">
                <div class="ops-command-card-head">
                    <div>
                        <p class="screen-anchor">"FD-001 TENANT WORKLOAD PLANE"</p>
                        <h5>"Product microservices hosted on the dogfood substrate"</h5>
                    </div>
                    <button type="button" data-cockpit-action="open-resource-inventory">"Inventory"</button>
                </div>
                <div class="ops-plane-summary" aria-label="FD-001 tenant workload summary">
                    <span><strong>"9"</strong><small>"FD-001 services"</small></span>
                    <span><strong>"3"</strong><small>"cells"</small></span>
                    <span><strong>"0"</strong><small>"live mutations"</small></span>
                </div>
                <div class="ops-workload-list" aria-label="FD-001 microservices running as tenant workloads">
                    <button
                        type="button"
                        class="selected"
                        data-cockpit-workload="workflow"
                        data-workload-title="Workflow runner"
                        data-workload-service="workflow-runner"
                        data-workload-cell="us-east-2"
                        data-workload-state="review"
                        data-workload-route="Workflow → Messenger/Mail/Community → Evidence"
                        data-workload-receipt="REC-FD001-WF-018"
                    >
                        <span>"Workflow"</span>
                        <strong>"workflow-runner"</strong>
                        <em>"us-east-2 · review"</em>
                        <small>"Runs approvals as visual-only tenant workload previews."</small>
                    </button>
                    <button
                        type="button"
                        data-cockpit-workload="comms"
                        data-workload-title="Built-in communications"
                        data-workload-service="messenger-mail-community"
                        data-workload-cell="us-east-2 + kr-seoul"
                        data-workload-state="drafts"
                        data-workload-route="Messenger/Mail/Community handoff bus"
                        data-workload-receipt="REC-COMMS-HANDOFF-006"
                    >
                        <span>"Comms"</span>
                        <strong>"messenger-mail-community"</strong>
                        <em>"multi-surface · drafts"</em>
                        <small>"Local drafts prove FD-001 coordination without delivery."</small>
                    </button>
                    <button
                        type="button"
                        data-cockpit-workload="evidence"
                        data-workload-title="Evidence spine"
                        data-workload-service="audit-vault"
                        data-workload-cell="multi-cell"
                        data-workload-state="sealed"
                        data-workload-route="Audit ledger + object graph"
                        data-workload-receipt="REC-FD001-CLOUD-009"
                    >
                        <span>"Evidence"</span>
                        <strong>"audit-vault"</strong>
                        <em>"multi-cell · sealed"</em>
                        <small>"Receipts bind cloud posture, workflow output, and reviewers."</small>
                    </button>
                    <button
                        type="button"
                        data-cockpit-workload="identity"
                        data-workload-title="Identity envelope"
                        data-workload-service="identity-access"
                        data-workload-cell="kr-seoul gated"
                        data-workload-state="policy"
                        data-workload-route="Identity → Policy → Deployment gates"
                        data-workload-receipt="REC-ID-2026-05"
                    >
                        <span>"Identity"</span>
                        <strong>"identity-access"</strong>
                        <em>"kr pack · policy"</em>
                        <small>"Role and residency controls prove tenant placement."</small>
                    </button>
                </div>
                <div class="ops-workload-detail" aria-label="Selected tenant workload detail">
                    <span class="status-chip warning" data-cockpit-workload-status="true">
                        "Workflow runner selected · review gate open · local-only substrate proof"
                    </span>
                    <dl>
                        <div><dt>"Service"</dt><dd data-workload-detail-service="true">"workflow-runner"</dd></div>
                        <div><dt>"Cell"</dt><dd data-workload-detail-cell="true">"us-east-2"</dd></div>
                        <div><dt>"Route"</dt><dd data-workload-detail-route="true">"Workflow → Messenger/Mail/Community → Evidence"</dd></div>
                        <div><dt>"Receipt"</dt><dd data-workload-detail-receipt="true">"REC-FD001-WF-018"</dd></div>
                    </dl>
                    <div class="ops-workload-routes" aria-label="Selected workload routes">
                        <button type="button" data-cockpit-workload-route="workflow">"Workflow"</button>
                        <button type="button" data-cockpit-workload-route="mail">"Mail brief"</button>
                        <button type="button" data-cockpit-workload-route="community">"Community"</button>
                        <button type="button" data-cockpit-workload-route="evidence">"Evidence"</button>
                        <button type="button" data-cockpit-workload-route="gates">"Gates"</button>
                    </div>
                </div>
            </section>

            <section class="ops-command-card ops-release-card">
                <div class="ops-command-card-head">
                    <div>
                        <p class="screen-anchor">"RELEASE GATES"</p>
                        <h5>"Jenkins, ArgoCD, cosign, and audit evidence"</h5>
                    </div>
                    <button type="button" data-cockpit-action="open-deployment-gates">"Gates"</button>
                </div>
                <div class="ops-release-lanes" aria-label="Cloud release readiness">
                    <span style="--bar: 92%"><strong>"Jenkins parity"</strong><em>"92%"</em></span>
                    <span style="--bar: 74%"><strong>"ArgoCD app"</strong><em>"74%"</em></span>
                    <span style="--bar: 88%"><strong>"Cosign verify"</strong><em>"88%"</em></span>
                    <span style="--bar: 69%"><strong>"Audit emit"</strong><em>"69%"</em></span>
                </div>
            </section>

            <section class="ops-command-card ops-route-card">
                <div class="ops-command-card-head">
                    <div>
                        <p class="screen-anchor">"ROUTES"</p>
                        <h5>"Open the connected product surface without leaving context"</h5>
                    </div>
                </div>
                <div class="ops-route-grid" aria-label="Cloud operations local routes">
                    <button type="button" data-cockpit-action="open-workflow">"Workflow"</button>
                    <button type="button" data-cockpit-action="open-mail">"Mail brief"</button>
                    <button type="button" data-cockpit-action="open-evidence">"Evidence"</button>
                    <button type="button" data-cockpit-action="open-finops">"FinOps"</button>
                </div>
                <p>"All actions are visual-only local state; no cloud, DNS, deploy, or billing operation is executed."</p>
            </section>
        </div>
    }
}

fn finops_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board finops-trust-board" aria-label="FD-001 FinOps and Oyatie Cloud substrate proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="finops-fd001">
                    <p class="screen-anchor">"FD-001 WORKLOAD ECONOMY"</p>
                    <h5>"Product delivery remains the north star"</h5>
                    <p>
                        "Run-rate is shown per FD-001 tenant workload: Workflow, Messenger, Mail, Community, Intelligence, and audit services stay in one delivery envelope."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="stage-budget">"Stage budget"</button>
                        <button type="button" data-trust-proof-action="route-finance">"Finance close"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="finops-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD SUBSTRATE"</p>
                    <h5>"Costs prove real tenant hosting"</h5>
                    <p>
                        "Compute, network, storage, audit, residency, and release gates expose hyperscaler-grade posture before FD-001 workloads claim production readiness."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud topology"</button>
                        <button type="button" data-trust-proof-action="route-policy">"Policy gate"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="finops-local">
                    <p class="screen-anchor">"LOCAL-ONLY FINOPS"</p>
                    <h5>"Interactive budget controls, no spend mutation"</h5>
                    <p>
                        "Operators can stage commitments, tag anomalies, and brief reviewers visually; no billing, procurement, deploy, DNS, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-audit">"Audit receipt"</button>
                        <button type="button" data-trust-proof-action="route-evidence">"Evidence spine"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "FinOps ready · FD-001 microservices dogfood Oyatie Cloud as tenant workloads with local-only controls."
                </span>
                <div class="trust-anchor-routes" aria-label="FinOps connected routes">
                    <button type="button" data-trust-proof-action="route-inventory">"Resources"</button>
                    <button type="button" data-trust-proof-action="route-gates">"Gates"</button>
                    <button type="button" data-trust-proof-action="route-mail">"Reviewer Mail"</button>
                    <button type="button" data-trust-proof-action="route-community">"Community"</button>
                </div>
            </div>
        </div>
    }
}

fn resource_inventory_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board" aria-label="FD-001 resource inventory and Oyatie Cloud substrate proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="resource-fd001">
                    <p class="screen-anchor">"FD-001 SERVICE FLEET"</p>
                    <h5>"Microservices are tenant workloads"</h5>
                    <p>
                        "Tenant admin, workflow runner, audit vault, Mail, Messenger, Community, and Intelligence assets are tracked as one FD-001 product fleet."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="link-resource">"Link resource"</button>
                        <button type="button" data-trust-proof-action="route-catalog">"Service catalog"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="resource-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD INVENTORY"</p>
                    <h5>"Substrate owns residency and release posture"</h5>
                    <p>
                        "Each resource shows cell, owner, cost, risk, policy, deployment gate, and audit receipt so the cloud substrate can prove real tenant hosting."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-finops">"FinOps cost"</button>
                        <button type="button" data-trust-proof-action="route-gates">"Admission gates"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="resource-local">
                    <p class="screen-anchor">"LOCAL-ONLY RESOURCE OPS"</p>
                    <h5>"Inspect without provider mutation"</h5>
                    <p>
                        "Operators can inspect ownership, route evidence, and preview remediation; no cloud provider, database, deploy, billing, or audit mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-audit">"Audit ledger"</button>
                        <button type="button" data-trust-proof-action="trace-lineage">"Trace lineage"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Resource inventory ready · FD-001 workload fleet is hosted-proof on Oyatie Cloud with local visual controls only."
                </span>
                <div class="trust-anchor-routes" aria-label="Resource inventory connected routes">
                    <button type="button" data-trust-proof-action="route-workflow">"Workflow"</button>
                    <button type="button" data-trust-proof-action="route-evidence">"Evidence"</button>
                    <button type="button" data-trust-proof-action="route-mail">"Mail"</button>
                    <button type="button" data-trust-proof-action="route-community">"Community"</button>
                </div>
            </div>
        </div>
    }
}

fn trust_center_portal() -> impl IntoView {
    view! {
        <section
            id="trust-center-portal"
            class="trust-center-portal panel"
            aria-labelledby="trust-center-title"
            data-trust-center-fixture="contract-first-slice"
        >
            <div class="panel-header trust-center-header">
                <div>
                    <p class="eyebrow">"Trust Center · customer evidence"</p>
                    <h3 id="trust-center-title">"Trust Center overview"</h3>
                    <p>
                        "Contracted fixture for /specs/trust-center-compliance-evidence-portal.json: tenant-scoped evidence, reviewer room, status link, SBOM/VEX posture, freshness, and access audit."
                    </p>
                </div>
                <span class="status-chip warning">"first slice · fixture/API contract"</span>
            </div>
            <div class="trust-center-role-strip" aria-label="Trust Center persona paths">
                <article data-trust-role="tenant-admin"><span class="status-chip success">"tenant admin"</span><strong>"Review posture, filter evidence, invite/revoke reviewers"</strong><p>"Can inspect summary, open detail, manage bounded reviewer grants, and view access audit."</p></article>
                <article data-trust-role="security-compliance-reviewer"><span class="status-chip">"reviewer"</span><strong>"Read-only evidence room with expiry"</strong><p>"Reviewer can read approved evidence and safe exports, but cannot administer tenant settings."</p></article>
                <article data-trust-role="oyatie-operator"><span class="status-chip warning">"operator"</span><strong>"Publishability and redaction stay upstream"</strong><p>"Operator-only detail withheld; UI shows handles, claim tiers, audit refs, and customer-safe summaries."</p></article>
            </div>
            <div class="trust-center-kpi-strip" aria-label="Trust Center evidence freshness summary">
                <span><strong>"3"</strong><small>"current evidence"</small></span>
                <span><strong>"2"</strong><small>"warning/stale"</small></span>
                <span><strong>"1"</strong><small>"blocked_missing_evidence · not green"</small></span>
                <span><strong>"target_non_claim"</strong><small>"claim tier: target_non_claim"</small></span>
            </div>
            <div class="trust-center-toolbar" aria-label="Trust Center filters and public status controls">
                <label><span aria-hidden="true">"⌕"</span><input aria-label="Filter trust evidence by class, source, freshness, or claim tier" value="" placeholder="Filter evidence, source, freshness, claim tier..." /></label>
                <div class="trust-filter-pills" role="toolbar" aria-label="Trust Center evidence state filters">
                    <button type="button" class="active" data-trust-filter="all" aria-pressed="true">"All"</button>
                    <button type="button" data-trust-filter="current" aria-pressed="false">"Current"</button>
                    <button type="button" data-trust-filter="stale" aria-pressed="false">"Stale"</button>
                    <button type="button" data-trust-filter="blocked" aria-pressed="false">"Blocked"</button>
                    <button type="button" data-trust-filter="target_non_claim" aria-pressed="false">"Target non-claim"</button>
                </div>
                <a class="trust-status-link" href="https://status.oyatie.example/northwind">"Public status link / embed"</a>
            </div>
            <div class="trust-center-layout">
                <article class="trust-evidence-table-card" aria-labelledby="trust-evidence-table-title">
                    <div class="cockpit-column-head"><p class="screen-anchor">"CONTROLS AND EVIDENCE"</p><h4 id="trust-evidence-table-title">"Evidence table with non-color states"</h4></div>
                    <table class="resource-table trust-evidence-table" aria-label="Trust Center evidence table">
                        <thead><tr><th>"Control"</th><th>"Source"</th><th>"Freshness"</th><th>"Claim tier"</th><th>"Audit ref"</th><th>"Detail"</th></tr></thead>
                        <tbody>
                            <tr data-trust-evidence-state="current"><td><strong>"SOC2 CC7.2 validation"</strong><small>"security_validation_controls"</small></td><td>"security-validation matrix"</td><td><span class="status-chip success" aria-label="freshness state current">"current · source 2026-07-01T07:42Z"</span></td><td>"mechanically_enforced"</td><td><code>"aud-trust-ctrl-cc72-001"</code></td><td><button type="button" aria-label="Open SOC2 CC7.2 evidence detail">"Open detail"</button></td></tr>
                            <tr data-trust-evidence-state="stale"><td><strong>"DR drill floor"</strong><small>"slo_dr_status_incident"</small></td><td>"compliance-pack floors"</td><td><span class="status-chip warning" aria-label="freshness state stale warning">"stale · refresh required"</span></td><td>"spec_ready"</td><td><code>"aud-trust-dr-floor-014"</code></td><td><button type="button" aria-label="Open DR drill stale evidence detail">"Open detail"</button></td></tr>
                            <tr data-trust-evidence-state="blocked_missing_evidence"><td><strong>"Quality kit abuse drill"</strong><small>"cloud_quality_kits"</small></td><td>"quality-kit evidence catalogue"</td><td><span class="status-chip danger" aria-label="freshness state blocked missing evidence">"blocked_missing_evidence · not green"</span></td><td>"target_non_claim"</td><td><code>"gap-trust-qk-abuse-001"</code></td><td><button type="button" aria-label="Open missing quality kit evidence gap detail">"Open detail"</button></td></tr>
                            <tr data-trust-evidence-state="target_non_claim"><td><strong>"External certification display"</strong><small>"claim boundary"</small></td><td>"trust-center spec"</td><td><span class="status-chip" aria-label="target non claim not certified">"target_non_claim · no certification"</span></td><td>"claim tier: target_non_claim"</td><td><code>"nonclaim-trust-cert-000"</code></td><td><button type="button" aria-label="Open non certification boundary detail">"Open detail"</button></td></tr>
                        </tbody>
                    </table>
                </article>
                <aside class="trust-detail-card" aria-labelledby="trust-detail-title">
                    <p class="screen-anchor">"EVIDENCE DETAIL"</p><h4 id="trust-detail-title">"Customer-safe detail pane"</h4>
                    <dl class="service-kv"><div><dt>"source_system"</dt><dd>"security-validation matrix"</dd></div><div><dt>"source_record_ref"</dt><dd>"secval://lane/api-fuzz/latest"</dd></div><div><dt>"freshness_state"</dt><dd>"current"</dd></div><div><dt>"audit_event_ref"</dt><dd>"aud-trust-ctrl-cc72-001"</dd></div><div><dt>"redaction"</dt><dd>"raw scanner output redacted · operator-only detail withheld"</dd></div></dl>
                    <p class="scope-notice">"Cross-tenant route denied; payload tenant assertions are display-only and never authority."</p>
                    <p>"keyboard table/detail/filter/reviewer grant paths labelled"</p>
                </aside>
            </div>
            <div class="trust-lower-grid">
                <article class="trust-sbom-card" aria-labelledby="trust-sbom-title"><p class="screen-anchor">"SBOM / VEX"</p><h4 id="trust-sbom-title">"SBOM/VEX posture"</h4><ul class="evidence-steps"><li><span>"SBOM"</span><strong>"signed SBOM present"</strong></li><li><span>"VEX"</span><strong>"customer-safe VEX summary · exploit details withheld"</strong></li><li><span>"SLA"</span><strong>"2 remediation SLA exceptions expire in 6 days"</strong></li></ul></article>
                <article class="trust-reviewer-card" aria-labelledby="trust-reviewer-title" data-reviewer-grant-flow="create-revoke-expiring"><p class="screen-anchor">"REVIEWER GRANT"</p><h4 id="trust-reviewer-title">"Reviewer invite"</h4><label><span>"Reviewer email"</span><input aria-label="Reviewer email" type="email" value="security.reviewer@example.com" /></label><label><span>"Expiry"</span><input aria-label="Reviewer grant expiry" type="datetime-local" value="2026-07-08T00:00" /></label><div class="trust-card-actions"><button type="button" aria-label="Create expiring reviewer grant">"Invite reviewer"</button><button type="button" aria-label="Revoke reviewer access">"Revoke reviewer access"</button></div><p>"Reviewer cannot administer tenant settings; grant is bounded by purpose, scope, and expiry."</p></article>
                <article class="trust-access-audit-card" aria-labelledby="trust-access-audit-title"><p class="screen-anchor">"ACCESS AUDIT"</p><h4 id="trust-access-audit-title">"Access audit"</h4><ol class="audit-ledger-list compact"><li><time>"07:42"</time><span class="status-chip success">"viewed"</span><strong>"trust_center.evidence_index_viewed"</strong><p>"tenant_admin · aud-trust-index-001"</p></li><li><time>"07:48"</time><span class="status-chip warning">"grant"</span><strong>"trust_center.access_grant_created"</strong><p>"expiry 2026-07-08 · aud-trust-grant-002"</p></li><li><time>"08:05"</time><span class="status-chip danger">"denied"</span><strong>"Cross-tenant route denied"</strong><p>"other tenant evidence room request failed closed"</p></li></ol></article>
            </div>
        </section>
    }
}

fn resource_audit_console(envelope: TenantRenderEnvelope) -> impl IntoView {
    let resources = resource_inventory_rows();
    let audit_events = audit_receipts();
    let gates = deployment_gates();
    let visible_modules = envelope.modules.len();
    let open_approvals = envelope.approvals.len();

    view! {
        <section
            id="resource-audit-console"
            class="resource-audit-console panel"
            aria-labelledby="resource-audit-title"
        >
            <div class="panel-header resource-console-header">
                <div>
                    <p class="eyebrow">"Operate · Trust"</p>
                    <h3 id="resource-audit-title">"Resource inventory, audit ledger, and deployment gates"</h3>
                </div>
                <div class="resource-tabs" role="tablist" aria-label="Resource and audit console views">
                    <button type="button" class="active" data-resource-tab="inventory" role="tab" aria-selected="true">"Inventory"</button>
                    <button type="button" data-resource-tab="audit" role="tab" aria-selected="false">"Audit ledger"</button>
                    <button type="button" data-resource-tab="gates" role="tab" aria-selected="false">"Deployment gates"</button>
                </div>
            </div>

            <div class="resource-console-spine" aria-label="Console summary">
                <span><strong>{resources.len()}</strong>" resources"</span>
                <span><strong>{audit_events.len()}</strong>" receipts staged"</span>
                <span><strong>{visible_modules}</strong>" services visible"</span>
                <span><strong>{open_approvals}</strong>" approvals linked"</span>
            </div>

            <div class="resource-toolbar" aria-label="Resource console controls">
                <label>
                    <span aria-hidden="true">"⌕"</span>
                    <input data-resource-search="true" aria-label="Search resources and receipts" placeholder="Search resource, owner, region, receipt..." />
                </label>
                <div class="resource-filter-pills" role="toolbar" aria-label="Resource state filters">
                    <button type="button" class="active" data-resource-filter="all">"All"</button>
                    <button type="button" data-resource-filter="attention">"Attention"</button>
                    <button type="button" data-resource-filter="review">"Review"</button>
                    <button type="button" data-resource-filter="active">"Active"</button>
                </div>
                <div class="resource-actions">
                    <button type="button" data-resource-action="refresh">"Refresh data"</button>
                    <button type="button" data-resource-action="export">"Export CSV"</button>
                </div>
                <span data-resource-status="true">"6 visible · local inventory only"</span>
            </div>

            <div class="resource-panels">
                <article
                    id="resource-inventory"
                    class="resource-panel active"
                    data-resource-panel="inventory"
                    aria-labelledby="resource-inventory-title"
                >
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"RESOURCE INVENTORY"</p>
                        <h4 id="resource-inventory-title">"Tenant assets with ownership, region, cost, and risk"</h4>
                    </div>
                    <table class="resource-table">
                        <thead>
                            <tr>
                                <th>"Kind"</th>
                                <th>"Name"</th>
                                <th>"Region"</th>
                                <th>"Owner"</th>
                                <th>"State"</th>
                                <th>"Monthly"</th>
                                <th>"Action"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {resources.into_iter().map(|row| view! {
                                <tr data-resource-row="true" data-resource-state=row.state>
                                    <td><span class="resource-kind">{row.kind}</span></td>
                                    <td><strong>{row.name}</strong><small>{row.description}</small></td>
                                    <td>{row.region}</td>
                                    <td>{row.owner}</td>
                                    <td><span class=resource_status_class(row.state)>{row.state}</span></td>
                                    <td class="numeric">{row.monthly}</td>
                                    <td>
                                        <button
                                            type="button"
                                            data-sidepeek-trigger="resource"
                                            data-sidepeek-title=row.name
                                            data-sidepeek-id=row.side_id
                                            data-sidepeek-desc=row.description
                                            data-sidepeek-owner=row.owner
                                            data-sidepeek-risk=row.risk
                                            data-sidepeek-sla="Inventory staged · no live mutation"
                                        >
                                            "Inspect"
                                        </button>
                                    </td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                    {resource_inventory_anchor_board()}
                </article>

                <article
                    id="audit-ledger"
                    class="resource-panel"
                    data-resource-panel="audit"
                    aria-labelledby="audit-ledger-title"
                >
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"AUDIT LEDGER"</p>
                        <h4 id="audit-ledger-title">"Immutable tenant-workload proof stream"</h4>
                    </div>
                    {receipt_stitching_console()}
                    <div class="audit-proof-grid" aria-label="FD-001 tenant workload receipt proof">
                        <article class="audit-proof-card selected" data-audit-card="fd001">
                            <p class="screen-anchor">"FD-001 RECEIPTS"</p>
                            <h5>"Product delivery remains master plan"</h5>
                            <p>
                                "Every Messenger, Mail, Community, Workflow, Ontology, and Intelligence preview action creates "
                                "a visible receipt so FD-001 can be dogfooded as a real tenant workload."
                            </p>
                            <div class="audit-command-actions">
                                <button type="button" data-audit-anchor-action="open-evidence">"Open evidence"</button>
                                <button type="button" data-audit-anchor-action="route-mail">"Mail brief"</button>
                            </div>
                        </article>
                        <article class="audit-proof-card" data-audit-card="cloud">
                            <p class="screen-anchor">"OYATIE CLOUD"</p>
                            <h5>"Oyatie Cloud substrate proves hosting posture"</h5>
                            <p>
                                "The cloud substrate records residency, release, cost, policy, and rollback checks before a "
                                "tenant surface can claim production readiness."
                            </p>
                            <div class="audit-command-actions">
                                <button type="button" data-audit-anchor-action="route-cloud">"Cloud topology"</button>
                                <button type="button" data-audit-anchor-action="route-gates">"Release gates"</button>
                            </div>
                        </article>
                        <article class="audit-proof-card" data-audit-card="sealed">
                            <p class="screen-anchor">"SEALED PACKET"</p>
                            <h5>"Interactive local receipt vault"</h5>
                            <p>
                                "Operators can inspect, seal, route, and brief a receipt packet visually while backend, billing, "
                                "deploy, and cloud mutations remain disconnected."
                            </p>
                            <div class="audit-command-actions">
                                <button type="button" data-audit-anchor-action="seal-packet">"Seal packet"</button>
                                <button type="button" data-audit-anchor-action="route-policy">"Policy board"</button>
                            </div>
                        </article>
                    </div>
                    <ol class="audit-ledger-list">
                        {audit_events.into_iter().map(|item| view! {
                            <li>
                                <time>{item.time}</time>
                                <span class=resource_status_class(item.severity)>{item.severity}</span>
                                <strong>{item.event}</strong>
                                <p>{item.actor}</p>
                                <code>{item.receipt}</code>
                                <button type="button" data-audit-anchor-action="inspect-receipt">"Inspect"</button>
                            </li>
                        }).collect_view()}
                    </ol>
                    <div class="audit-anchor-footer">
                        <span data-audit-anchor-status="true">
                            "Audit ledger ready · FD-001 tenant workload receipts remain local visual evidence."
                        </span>
                        <div class="audit-command-actions" aria-label="Audit ledger connected routes">
                            <button type="button" data-audit-anchor-action="route-workflow">"Workflow proof"</button>
                            <button type="button" data-audit-anchor-action="route-community">"Community review"</button>
                            <button type="button" data-audit-anchor-action="open-evidence">"Evidence graph"</button>
                        </div>
                    </div>
                </article>

                <article
                    id="deployment-gates"
                    class="resource-panel"
                    data-resource-panel="gates"
                    aria-labelledby="deployment-gates-title"
                >
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"DEPLOYMENT GATES"</p>
                        <h4 id="deployment-gates-title">"FD-001 tenant workload admission gates"</h4>
                    </div>
                    {deployment_gate_command_board()}
                    <div class="gate-grid">
                        {gates.into_iter().map(|gate| view! {
                            <article class="gate-card">
                                <div>
                                    <span class=resource_status_class(gate.state)>{gate.state}</span>
                                    <h5>{gate.label}</h5>
                                    <p>{gate.detail}</p>
                                </div>
                                <span class="gate-progress" style=format!("--bar: {}", gate.progress)>
                                    <em>{gate.progress}</em>
                                </span>
                                <div class="gate-card-actions">
                                    <button type="button" data-gate-action="attach-evidence">"Attach evidence"</button>
                                    <button type="button" data-gate-action="open-evidence">"Evidence"</button>
                                    <button type="button" data-gate-action="route-owner">"Owner route"</button>
                                </div>
                            </article>
                        }).collect_view()}
                    </div>
                    <div class="deployment-gate-footer">
                        <span data-deployment-gate-status="true">
                            "Deployment gates ready · FD-001 microservices are tenant workloads on Oyatie Cloud."
                        </span>
                        <div class="deployment-gate-routes" aria-label="Deployment gate connected routes">
                            <button type="button" data-deployment-gate-action="route-policy">"Policy envelope"</button>
                            <button type="button" data-deployment-gate-action="route-audit">"Audit packet"</button>
                            <button type="button" data-deployment-gate-action="route-community">"Community note"</button>
                            <button type="button" data-deployment-gate-action="route-cloud">"Cloud cells"</button>
                        </div>
                    </div>
                </article>
            </div>
        </section>
    }
}

fn deployment_gate_command_board() -> impl IntoView {
    view! {
        <div class="deployment-proof-grid" aria-label="FD-001 and Oyatie Cloud deployment proof">
            <article class="deployment-proof-card selected" data-deployment-card="fd001">
                <p class="screen-anchor">"FD-001 RELEASE TRAIN"</p>
                <h5>"Product microservices deploy as tenants"</h5>
                <p>
                    "Messenger, Mail, Community, Workflow, Ontology, Intelligence, and core ops stay product-first; "
                    "the gates prove they can run as tenant workloads on the substrate."
                </p>
                <div class="deployment-card-actions">
                    <button type="button" data-deployment-gate-action="admit-fd001">"Admit workload"</button>
                    <button type="button" data-deployment-gate-action="route-workflow">"Workflow runbook"</button>
                </div>
            </article>
            <article class="deployment-proof-card" data-deployment-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD"</p>
                <h5>"Hyperscaler-grade substrate proof"</h5>
                <p>
                    "Cell topology, policy sidecars, cosign receipts, ArgoCD app health, rollback posture, and "
                    "audit-chain freshness must be visible before any promotion claim."
                </p>
                <div class="deployment-card-actions">
                    <button type="button" data-deployment-gate-action="route-cloud">"Inspect cells"</button>
                    <button type="button" data-deployment-gate-action="route-finops">"FinOps guard"</button>
                </div>
            </article>
            <article class="deployment-proof-card" data-deployment-card="control">
                <p class="screen-anchor">"CONTROL PLANE"</p>
                <h5>"Interactive, never wired"</h5>
                <p>
                    "Operators can simulate gate decisions, seal a release packet, and route reviewer work, while "
                    "deploy, DNS, registry, billing, and cloud mutations remain disconnected."
                </p>
                <div class="deployment-card-actions">
                    <button type="button" data-deployment-gate-action="seal-release">"Seal packet"</button>
                    <button type="button" data-deployment-gate-action="route-mail">"Reviewer mail"</button>
                </div>
            </article>
        </div>
        <div class="deployment-promotion-lane" aria-label="Tenant workload promotion lane">
            <button type="button" class="active" data-deployment-gate-action="ci-lane">
                <span>"01"</span><strong>"CI mirror"</strong><em>"Jenkins parity · 92%"</em>
            </button>
            <button type="button" data-deployment-gate-action="attest-lane">
                <span>"02"</span><strong>"Attest"</strong><em>"cosign + SBOM · 61%"</em>
            </button>
            <button type="button" data-deployment-gate-action="admit-lane">
                <span>"03"</span><strong>"Admit tenant"</strong><em>"policy + PIPA · review"</em>
            </button>
            <button type="button" data-deployment-gate-action="observe-lane">
                <span>"04"</span><strong>"Observe"</strong><em>"SLO + audit emit · 48%"</em>
            </button>
        </div>
    }
}

fn resource_inventory_rows() -> [ResourceRow; 6] {
    [
        ResourceRow {
            kind: "K8s",
            name: "tenant-admin-api",
            region: "us-east-2",
            owner: "Cloud infrastructure",
            state: "active",
            monthly: "$12.4k",
            risk: "Medium",
            side_id: "RES-K8S-API",
            description: "Primary tenant administration API workload with policy and audit sidecars.",
        },
        ResourceRow {
            kind: "VPC",
            name: "northwind-prod-mesh",
            region: "us-east-2",
            owner: "Network operations",
            state: "attention",
            monthly: "$4.8k",
            risk: "High",
            side_id: "RES-VPC-MESH",
            description: "Production network mesh awaiting rollback evidence for the hot split.",
        },
        ResourceRow {
            kind: "DNS",
            name: "tenant-control-plane",
            region: "global",
            owner: "SRE",
            state: "active",
            monthly: "$0.9k",
            risk: "Low",
            side_id: "RES-DNS-CTRL",
            description: "Managed DNS and routing policy for tenant control-plane surfaces.",
        },
        ResourceRow {
            kind: "Bucket",
            name: "audit-receipt-vault",
            region: "us-east-2",
            owner: "Trust systems",
            state: "sealed",
            monthly: "$1.7k",
            risk: "Low",
            side_id: "RES-OBJ-AUDIT",
            description: "Object-store vault for immutable audit-chain receipt previews.",
        },
        ResourceRow {
            kind: "KMS",
            name: "northwind-tenant-key",
            region: "us-east-2",
            owner: "Security reviewer",
            state: "review",
            monthly: "$0.4k",
            risk: "Medium",
            side_id: "RES-KMS-TENANT",
            description: "Tenant scoped key rotation is pending quarterly owner attestation.",
        },
        ResourceRow {
            kind: "Topic",
            name: "audit-chain.events",
            region: "multi-region",
            owner: "Governance automation",
            state: "active",
            monthly: "$2.1k",
            risk: "Low",
            side_id: "RES-TOPIC-AUDIT",
            description: "Event topic used by local receipts, evidence spine, and deployment gates.",
        },
    ]
}

fn audit_receipts() -> [AuditReceipt; 5] {
    [
        AuditReceipt {
            time: "09:18",
            event: "Residency guardrail evaluated",
            actor: "POL-RES-014 · policy engine",
            receipt: "REC-NTW-4182-A",
            severity: "sealed",
        },
        AuditReceipt {
            time: "09:42",
            event: "Human reviewer assigned",
            actor: "Infrastructure operations → tenant admin",
            receipt: "REC-NTW-4182-B",
            severity: "active",
        },
        AuditReceipt {
            time: "10:05",
            event: "Messenger, mail, community drafts linked",
            actor: "Workflow Studio output routes",
            receipt: "REC-WF-7741",
            severity: "draft",
        },
        AuditReceipt {
            time: "10:17",
            event: "KMS owner attestation requested",
            actor: "Security reviewer",
            receipt: "REC-KMS-2033",
            severity: "attention",
        },
        AuditReceipt {
            time: "10:29",
            event: "ArgoCD promotion evidence staged",
            actor: "GitOps controller (staged)",
            receipt: "REC-DEP-0904",
            severity: "review",
        },
    ]
}

fn receipt_stitching_console() -> impl IntoView {
    view! {
        <section
            class="receipt-stitching-console"
            data-receipt-stitching-console="true"
            aria-label="FD-001 and Oyatie Cloud receipt stitching console"
        >
            <div class="receipt-stitching-head">
                <div>
                    <p class="screen-anchor">"RECEIPT STITCHING CONSOLE"</p>
                    <h5>"Every product action returns to one proof stream"</h5>
                    <span data-receipt-stitching-status="true">
                        "Workflow output, Work Hub drafts, Cloud workload posture, and Deployment gates are ready to stitch locally."
                    </span>
                </div>
                <button type="button" data-receipt-stitching-action="seal">"Seal visible packet"</button>
            </div>
            <div class="receipt-stitching-grid" aria-label="Receipt source routes">
                <button type="button" class="selected" data-receipt-source="workflow" data-receipt-title="Workflow output bundle" data-receipt-id="REC-FD001-WF-018" data-receipt-route="Workflow → Messenger/Mail/Community → Evidence" data-receipt-owner="Workflow Studio" data-receipt-state="review">
                    <span>"01 · WORKFLOW"</span>
                    <strong>"Run output bundle"</strong>
                    <em>"REC-FD001-WF-018"</em>
                </button>
                <button type="button" data-receipt-source="comms" data-receipt-title="Work Hub handoff draft" data-receipt-id="REC-COMMS-HANDOFF-006" data-receipt-route="Messenger/Mail/Community draft handoff" data-receipt-owner="Work Hub" data-receipt-state="draft">
                    <span>"02 · COMMS"</span>
                    <strong>"Draft handoff proof"</strong>
                    <em>"REC-COMMS-HANDOFF-006"</em>
                </button>
                <button type="button" data-receipt-source="cloud" data-receipt-title="Cloud tenant workload posture" data-receipt-id="REC-FD001-CLOUD-009" data-receipt-route="Oyatie Cloud workload plane → gates" data-receipt-owner="Cloud substrate" data-receipt-state="sealed">
                    <span>"03 · CLOUD"</span>
                    <strong>"Tenant workload proof"</strong>
                    <em>"REC-FD001-CLOUD-009"</em>
                </button>
                <button type="button" data-receipt-source="gates" data-receipt-title="Deployment admission packet" data-receipt-id="REC-DEPLOY-GATE-014" data-receipt-route="Jenkins → ArgoCD → Cosign → Audit" data-receipt-owner="Release governance" data-receipt-state="gate">
                    <span>"04 · GATES"</span>
                    <strong>"Admission proof"</strong>
                    <em>"REC-DEPLOY-GATE-014"</em>
                </button>
            </div>
            <aside class="receipt-stitching-detail" aria-label="Selected receipt stitch detail">
                <dl>
                    <div><dt>"Selected"</dt><dd data-receipt-detail-title="true">"Workflow output bundle"</dd></div>
                    <div><dt>"Receipt"</dt><dd data-receipt-detail-id="true">"REC-FD001-WF-018"</dd></div>
                    <div><dt>"Route"</dt><dd data-receipt-detail-route="true">"Workflow → Messenger/Mail/Community → Evidence"</dd></div>
                    <div><dt>"Owner"</dt><dd data-receipt-detail-owner="true">"Workflow Studio"</dd></div>
                </dl>
                <div class="receipt-stitching-actions" aria-label="Selected receipt actions">
                    <button type="button" data-receipt-stitching-action="workflow">"Workflow"</button>
                    <button type="button" data-receipt-stitching-action="cloud">"Cloud"</button>
                    <button type="button" data-receipt-stitching-action="mail">"Mail brief"</button>
                    <button type="button" data-receipt-stitching-action="community">"Community"</button>
                    <button type="button" data-receipt-stitching-action="graph">"Graph"</button>
                    <button type="button" data-receipt-stitching-action="gates">"Gates"</button>
                </div>
            </aside>
        </section>
    }
}

fn deployment_gates() -> [DeploymentGate; 4] {
    [
        DeploymentGate {
            label: "Jenkins parity",
            detail: "Required CI mirror lanes passed in local evidence preview.",
            state: "active",
            progress: "92%",
        },
        DeploymentGate {
            label: "ArgoCD application",
            detail: "Tenant namespace isolation and rollback preview present.",
            state: "review",
            progress: "74%",
        },
        DeploymentGate {
            label: "Cosign verification",
            detail: "Image signature receipt available; key attestation pending.",
            state: "attention",
            progress: "61%",
        },
        DeploymentGate {
            label: "Audit-chain emit",
            detail: "Deployment event payload shaped but not yet emitted before live integration.",
            state: "draft",
            progress: "48%",
        },
    ]
}

fn resource_status_class(state: &str) -> &'static str {
    match state {
        "active" | "sealed" => "status-chip success",
        "attention" | "review" => "status-chip warning",
        "blocked" => "status-chip danger",
        "draft" => "status-chip ai",
        _ => "status-chip",
    }
}

fn daily_execution_proof_board() -> impl IntoView {
    view! {
        <section class="daily-proof-board" aria-label="FD-001 daily execution and Oyatie Cloud tenant proof">
            <div class="daily-proof-grid">
                <article class="daily-proof-card selected" data-daily-proof-card="daily-fd001">
                    <p class="screen-anchor">"FD-001 DAILY WORKLOAD"</p>
                    <h5>"Today’s work proves product delivery"</h5>
                    <p>
                        "Tasks, approvals, schedule holds, Workflow routes, Mail, Messenger, Community, and evidence receipts are FD-001 tenant workload operations, not detached widgets."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="stage-packet">"Stage work packet"</button>
                        <button type="button" data-daily-proof-action="route-workflow">"Workflow run"</button>
                    </div>
                </article>
                <article class="daily-proof-card" data-daily-proof-card="daily-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD SUBSTRATE"</p>
                    <h5>"The queue dogfoods tenant hosting posture"</h5>
                    <p>
                        "Oyatie Cloud proves the substrate can host real production tenants by tying daily FD-001 work to cell health, policy envelopes, FinOps, and audit freshness."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="route-cloud">"Cloud cells"</button>
                        <button type="button" data-daily-proof-action="route-policy">"Policy envelope"</button>
                    </div>
                </article>
                <article class="daily-proof-card" data-daily-proof-card="daily-local">
                    <p class="screen-anchor">"LOCAL-ONLY COMMAND RAIL"</p>
                    <h5>"Visually functional without side effects"</h5>
                    <p>
                        "Operators can filter, stage, route, brief, and inspect work while backend writes, auth changes, workflow execution, mail sends, payroll, billing, and cloud mutations remain disconnected."
                    </p>
                    <div class="daily-proof-actions">
                        <button type="button" data-daily-proof-action="route-audit">"Audit ledger"</button>
                        <button type="button" data-daily-proof-action="route-mail">"Mail brief"</button>
                    </div>
                </article>
            </div>
            <div class="daily-proof-footer">
                <span data-daily-proof-status="true">"Daily execution ready · FD-001 work queue dogfoods Oyatie Cloud locally."</span>
                <div class="daily-proof-routes" aria-label="Daily execution connected routes">
                    <button type="button" data-daily-proof-action="route-inbox">"Action Inbox"</button>
                    <button type="button" data-daily-proof-action="route-schedule">"Schedule"</button>
                    <button type="button" data-daily-proof-action="route-community">"Community"</button>
                    <button type="button" data-daily-proof-action="route-evidence">"Evidence"</button>
                </div>
            </div>
        </section>
    }
}

fn daily_queue_footer() -> impl IntoView {
    view! {
        <div class="daily-subroute-proof">
            <span data-daily-proof-status="true">"Execution queue ready · FD-001 work items dogfood Oyatie Cloud with no backend mutation."</span>
            <div class="daily-proof-routes" aria-label="Execution queue connected routes">
                <button type="button" data-daily-proof-action="route-inbox">"Action Inbox"</button>
                <button type="button" data-daily-proof-action="route-workflow">"Workflow"</button>
                <button type="button" data-daily-proof-action="route-evidence">"Evidence"</button>
            </div>
        </div>
    }
}

fn daily_schedule_footer() -> impl IntoView {
    view! {
        <div class="daily-subroute-proof">
            <span data-daily-proof-status="true">"Schedule pressure ready · FD-001 calendar risk stays tenant-scoped on Oyatie Cloud with no calendar, workflow, mail, policy, or cloud mutation."</span>
            <div class="daily-proof-routes" aria-label="Schedule connected routes">
                <button type="button" data-daily-proof-action="route-policy">"Policy"</button>
                <button type="button" data-daily-proof-action="route-cloud">"Cloud cells"</button>
                <button type="button" data-daily-proof-action="route-mail">"Reviewer Mail"</button>
            </div>
        </div>
    }
}

fn daily_execution_console(envelope: TenantRenderEnvelope) -> impl IntoView {
    let rows = daily_execution_rows(&envelope);
    let blocking_count = rows.iter().filter(|row| row.state == "blocking").count();
    let task_count = rows.iter().filter(|row| row.kind == "task").count();
    let approval_count = rows.iter().filter(|row| row.kind == "approval").count();
    let schedule_count = rows.iter().filter(|row| row.kind == "schedule").count();
    let evidence_count = rows.iter().filter(|row| row.kind == "evidence").count();

    view! {
        <section id="daily-execution" class="daily-execution-console panel" aria-labelledby="daily-execution-title">
            <div class="daily-execution-head">
                <div>
                    <p class="screen-anchor">"DAILY WORK · PERSONAL OPERATIONS"</p>
                    <h3 id="daily-execution-title">"Tasks, approvals, schedule, and evidence for today"</h3>
                    <p>"A single operator queue connects calendar pressure, approval risk, workflow routes, Mail/Messenger drafts, and receipt evidence."</p>
                </div>
                <span class="status-chip success">"local command surface"</span>
            </div>

            <div class="daily-execution-kpis" aria-label="Daily execution summary">
                <span><strong>{task_count}</strong><small>"tasks"</small></span>
                <span><strong>{approval_count}</strong><small>"approvals"</small></span>
                <span><strong>{schedule_count}</strong><small>"calendar holds"</small></span>
                <span><strong>{blocking_count}</strong><small>"blocking"</small></span>
                <span><strong>{evidence_count}</strong><small>"evidence links"</small></span>
            </div>

            <div class="daily-execution-toolbar" aria-label="Daily execution filters">
                <label>
                    <span aria-hidden="true">"⌕"</span>
                    <input data-daily-search="true" aria-label="Search daily work" placeholder="Search tasks, approvals, owners, receipts..." />
                </label>
                <div class="daily-filter-pills" role="toolbar" aria-label="Daily work filters">
                    <button type="button" class="active" data-daily-filter="all">"All"</button>
                    <button type="button" data-daily-filter="blocking">"Blocking"</button>
                    <button type="button" data-daily-filter="task">"Tasks"</button>
                    <button type="button" data-daily-filter="approval">"Approvals"</button>
                    <button type="button" data-daily-filter="schedule">"Schedule"</button>
                    <button type="button" data-daily-filter="evidence">"Evidence"</button>
                </div>
                <span data-daily-status="true">{format!("{} visible · all work · local only", rows.len())}</span>
            </div>

            {daily_execution_proof_board()}

            <div class="daily-execution-layout">
                <article id="tasks-title" class="daily-execution-list" aria-labelledby="daily-list-title">
                    <div class="daily-column-head">
                        <p class="screen-anchor">"EXECUTION QUEUE"</p>
                        <h4 id="daily-list-title">"One list for personal operations"</h4>
                    </div>
                    <div role="list" aria-label="Daily work rows">
                        {rows.clone().into_iter().map(|row| {
                            let chip = daily_status_class(row.state);
                            view! {
                                <article
                                    class="daily-row"
                                    data-daily-row="true"
                                    data-daily-kind=row.kind
                                    data-daily-state=row.state
                                    role="listitem"
                                >
                                    <button
                                        type="button"
                                        class="daily-row-main"
                                        data-sidepeek-trigger="daily-work"
                                        data-sidepeek-title=row.title.clone()
                                        data-sidepeek-id=row.id.clone()
                                        data-sidepeek-desc=row.body.clone()
                                        data-sidepeek-owner=row.owner.clone()
                                        data-sidepeek-risk=row.state
                                        data-sidepeek-sla=row.due.clone()
                                    >
                                        <span class=chip>{row.state}</span>
                                        <strong>{row.title.clone()}</strong>
                                        <p>{row.body.clone()}</p>
                                    </button>
                                    <dl>
                                        <div><dt>"Kind"</dt><dd>{row.kind}</dd></div>
                                        <div><dt>"Owner"</dt><dd>{row.owner.clone()}</dd></div>
                                        <div><dt>"Due"</dt><dd>{row.due.clone()}</dd></div>
                                    </dl>
                                    <div class="daily-row-actions">
                                        <button type="button" data-daily-action="workflow" data-daily-target=row.route>"Flow"</button>
                                        <button type="button" data-daily-action="mail">"Mail"</button>
                                        <button type="button" data-daily-action="evidence">"Evidence"</button>
                                        <button type="button" data-daily-action="stage">"Stage"</button>
                                    </div>
                                </article>
                            }
                        }).collect_view()}
                    </div>
                    {daily_queue_footer()}
                </article>

                <aside id="schedule-title" class="daily-calendar-rail" aria-label="Today schedule and capacity">
                    <div class="daily-column-head">
                        <p class="screen-anchor">"CALENDAR"</p>
                        <h4>"Today’s schedule pressure"</h4>
                    </div>
                    <ol class="daily-timeline">
                        {envelope.schedule.clone().into_iter().map(|item| view! {
                            <li>
                                <time>{item.time}</time>
                                <strong>{item.title}</strong>
                                <p>{item.detail}</p>
                            </li>
                        }).collect_view()}
                    </ol>
                    <div class="daily-capacity" aria-label="Daily capacity">
                        <span style="--bar: 73%"><em>"Close work · 73%"</em></span>
                        <span style="--bar: 64%"><em>"Approvals · 64%"</em></span>
                        <span style="--bar: 41%"><em>"Context switching · 41%"</em></span>
                    </div>
                    <div class="daily-route-matrix" aria-label="Daily route matrix">
                        <button type="button" data-daily-action="workflow" data-daily-target="#workflow-studio">"Workflow"</button>
                        <button type="button" data-daily-action="mail">"Mail brief"</button>
                        <button type="button" data-daily-action="messenger">"Messenger"</button>
                        <button type="button" data-daily-action="evidence">"Audit evidence"</button>
                    </div>
                    {daily_schedule_footer()}
                </aside>
            </div>
        </section>
    }
}

fn daily_execution_rows(envelope: &TenantRenderEnvelope) -> Vec<ExecutionRow> {
    let mut rows = Vec::new();
    for (index, item) in envelope.daily_tasks.iter().enumerate() {
        let state = if item.priority.eq_ignore_ascii_case("high") {
            "blocking"
        } else {
            "task"
        };
        rows.push(ExecutionRow {
            id: format!("TASK-{}", index + 741),
            kind: "task",
            state,
            title: item.title.clone(),
            body: item.detail.clone(),
            owner: envelope.role_name.clone(),
            due: if index == 0 { "today 18:00" } else { "today" }.to_string(),
            route: "#workflow-studio",
        });
    }
    for (index, item) in envelope.approvals.iter().enumerate() {
        rows.push(ExecutionRow {
            id: format!("APR-{}", index + 274),
            kind: "approval",
            state: if index == 0 { "blocking" } else { "review" },
            title: item.title.clone(),
            body: item.risk_note.clone(),
            owner: item.requester.clone(),
            due: "review queue".to_string(),
            route: "#business-logics",
        });
    }
    for (index, item) in envelope.schedule.iter().enumerate() {
        rows.push(ExecutionRow {
            id: format!("CAL-{}", index + 31),
            kind: "schedule",
            state: "scheduled",
            title: item.title.clone(),
            body: item.detail.clone(),
            owner: "Calendar".to_string(),
            due: item.time.clone(),
            route: "#work-hub",
        });
    }
    rows.extend([
        ExecutionRow {
            id: "REC-WF-7741".to_string(),
            kind: "evidence",
            state: "sealed",
            title: "Workflow output receipts ready".to_string(),
            body: "Messenger, Mail, Community, and Action Inbox outputs share one local evidence packet.".to_string(),
            owner: "Evidence Spine".to_string(),
            due: "sealed draft".to_string(),
            route: "#evidence-spine",
        },
        ExecutionRow {
            id: "REC-PAY-2026-04-PARK".to_string(),
            kind: "evidence",
            state: "blocking",
            title: "Payroll delta evidence needs owner".to_string(),
            body: "Four-insurance change for Park Seo-jun is the current close blocker and routes to audit.".to_string(),
            owner: "Finance close".to_string(),
            due: "4.0h".to_string(),
            route: "#audit-ledger",
        },
    ]);
    rows
}

fn daily_status_class(state: &str) -> &'static str {
    match state {
        "blocking" => "status-chip danger",
        "review" | "scheduled" => "status-chip warning",
        "sealed" => "status-chip success",
        _ => "status-chip",
    }
}

fn work_list(items: Vec<WorkItem>) -> impl IntoView {
    view! {
        <div>
            <h4 id="tasks-title">"Task queue"</h4>
            <ul class="item-list">
                {items.into_iter().map(|item| view! {
                    <li>
                        <span class="priority">{item.priority}</span>
                        <strong>{item.title}</strong>
                        <p>{item.detail}</p>
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

fn approval_list(items: Vec<ApprovalItem>) -> impl IntoView {
    view! {
        <div>
            <h4>"Approval queue"</h4>
            <ul class="item-list compact">
                {items.into_iter().map(|item| view! {
                    <li>
                        <strong>{item.title}</strong>
                        <p>{item.requester}</p>
                        <span>{item.risk_note}</span>
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

fn schedule_list(items: Vec<ScheduleItem>) -> impl IntoView {
    view! {
        <ol class="timeline-list">
            {items.into_iter().map(|item| view! {
                <li>
                    <time>{item.time}</time>
                    <div>
                        <strong>{item.title}</strong>
                        <p>{item.detail}</p>
                    </div>
                </li>
            }).collect_view()}
        </ol>
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Communication surface keeps signal ownership explicit so local-only drafts cannot be confused with server persistence."
)]
fn communication_hub(
    messages: Vec<MessageItem>,
    communities: Vec<CommunityItem>,
    active_surface: ReadSignal<ProductSurface>,
    set_active_surface: WriteSignal<ProductSurface>,
    selected_hub_index: ReadSignal<usize>,
    set_selected_hub_index: WriteSignal<usize>,
    draft_body: ReadSignal<String>,
    set_draft_body: WriteSignal<String>,
    local_drafts: ReadSignal<Vec<LocalDraft>>,
    set_local_drafts: WriteSignal<Vec<LocalDraft>>,
) -> impl IntoView {
    let tabs = [
        ProductSurface::Messenger,
        ProductSurface::Mail,
        ProductSurface::Community,
    ];
    let list_messages = messages.clone();
    let list_communities = communities.clone();
    let detail_messages = messages.clone();
    let detail_communities = communities.clone();

    view! {
        <div class="communications-hub interactive-hub">
            <div class="hub-tabs" role="tablist" aria-label="Work hub channels">
                {tabs.into_iter().map(|surface| view! {
                    <button
                        type="button"
                        role="tab"
                        aria-selected=move || active_surface.get() == surface
                        class=move || if active_surface.get() == surface { "hub-tab active" } else { "hub-tab" }
                        on:click=move |_| {
                            set_active_surface.set(surface);
                            set_selected_hub_index.set(0);
                        }
                    >
                        {surface.label()}
                    </button>
                }).collect_view()}
            </div>

            <div class="comms-kpi-strip" aria-label="Built-in communications summary">
                <span><strong>"18"</strong><small>"threads · drafts"</small></span>
                <span><strong>"6"</strong><small>"workflow routes"</small></span>
                <span><strong>"4"</strong><small>"evidence links"</small></span>
                <span><strong>"0"</strong><small>"external sends"</small></span>
            </div>

            <div class="hub-route-board" aria-label="Workflow output routes">
                <div>
                    <p class="screen-anchor">"OUTPUT ROUTES"</p>
                    <strong>"FD-001 tenant-workload drafts fan out to Messenger, Mail, and Community with evidence return paths"</strong>
                    <span data-comms-route-status="true">"FD-001 workload dogfood · REC-WF-7741 · no backend send"</span>
                </div>
                <button type="button" data-hub-route="Messenger" on:click=move |_| {
                    set_active_surface.set(ProductSurface::Messenger);
                    set_selected_hub_index.set(0);
                }>"Messenger post"</button>
                <button type="button" data-hub-route="Mail" on:click=move |_| {
                    set_active_surface.set(ProductSurface::Mail);
                    set_selected_hub_index.set(0);
                }>"Mail draft"</button>
                <button type="button" data-hub-route="Community" on:click=move |_| {
                    set_active_surface.set(ProductSurface::Community);
                    set_selected_hub_index.set(0);
                }>"Community note"</button>
            </div>

            {move || comms_product_board(active_surface.get())}

            <section class="comms-substrate-strip" aria-label="Oyatie Cloud tenant-workload proof">
                <div>
                    <p class="screen-anchor">"SUBSTRATE PROOF"</p>
                    <strong>"Messenger, Mail, and Community are dogfood tenant workloads on Oyatie Cloud"</strong>
                    <span data-comms-substrate-status="true">
                        {move || format!(
                            "{} route pinned to FD-001 workload · cell-us-east-2 · local visual proof",
                            active_surface.get().label()
                        )}
                    </span>
                </div>
                <button type="button" data-comms-action="prove-substrate">
                    <span>"Cloud cell"</span><strong>"cell-us-east-2"</strong>
                </button>
                <button type="button" data-comms-action="route-cloud">
                    <span>"Tenant workload"</span><strong>"FD-001 microservices"</strong>
                </button>
                <button type="button" data-comms-action="seal-proof">
                    <span>"Evidence"</span><strong>"REC-WF-7741"</strong>
                </button>
            </section>

            {comms_receipt_bridge()}

            <div class="comms-service-toolbar" aria-label="Communications workspace controls">
                <label>
                    <span aria-hidden="true">"⌕"</span>
                    <input data-comms-search="true" aria-label="Search communications" placeholder="Search threads, mail, spaces..." />
                </label>
                <div class="comms-filter-pills" role="toolbar" aria-label="Communication filters">
                    <button type="button" class="active" data-comms-filter="all">"All"</button>
                    <button type="button" data-comms-filter="unread">"Unread"</button>
                    <button type="button" data-comms-filter="draft">"Drafts"</button>
                    <button type="button" data-comms-filter="evidence">"Evidence"</button>
                </div>
                <button type="button" data-comms-action="new-thread">"New thread"</button>
                <button type="button" data-comms-action="attach-evidence">"Attach evidence"</button>
                <button type="button" data-comms-action="directory">"Directory"</button>
                <span data-comms-status="true">"Local service workspace ready · no external send"</span>
            </div>

            <div class="hub-workspace comms-service-shell">
                <aside class="comms-sidebar" aria-label="Communications folders and spaces">
                    <p class="screen-anchor">"WORKSPACES"</p>
                    <button type="button" class="active" data-hub-route="Messenger">
                        <strong>"Ops room"</strong><span>"Messenger · 5 items · 2 unread"</span>
                    </button>
                    <button type="button" data-hub-route="Mail">
                        <strong>"Finance close"</strong><span>"Mail · 4 drafts · 2 evidence"</span>
                    </button>
                    <button type="button" data-hub-route="Community">
                        <strong>"Governance council"</strong><span>"Community · 5 spaces · 1 publish"</span>
                    </button>
                    <button type="button" data-comms-action="notification-filter">
                        <strong>"Notifications"</strong><span>"6 local alerts · no external send"</span>
                    </button>
                </aside>
                <div class="hub-list" role="list" aria-label="Channel items">
                    {move || {
                        let items = hub_items(
                            &list_messages,
                            &list_communities,
                            &local_drafts.get(),
                            active_surface.get(),
                        );
                        let active_index = selected_hub_index.get();
                        items.into_iter().enumerate().map(|(index, item)| {
                            let kind = hub_item_kind(&item, index);
                            let chip_class = hub_item_chip_class(kind);
                            view! {
                                <button
                                    type="button"
                                    class=if index == active_index { "hub-item active" } else { "hub-item" }
                                    data-comms-item="true"
                                    data-comms-kind=kind
                                    on:click=move |_| set_selected_hub_index.set(index)
                                >
                                    <span class=chip_class>{item.source}</span>
                                    <strong>{item.title}</strong>
                                    <p>{item.body}</p>
                                    <small><em>{kind}</em><b>{item.meta}</b></small>
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>

                <div class="hub-detail" aria-live="polite">
                    <div class="comms-message-toolbar" aria-label="Selected communication actions">
                        <span class="status-chip success">"role-visible"</span>
                        <button type="button" data-comms-action="mark-reviewed">"Mark reviewed"</button>
                        <button type="button" data-comms-action="create-task">"Create task"</button>
                        <button type="button" data-comms-action="link-workflow">"Link workflow"</button>
                        <button type="button" data-comms-action="send-preview">"Send preview"</button>
                        <button type="button" data-comms-action="publish-note">"Publish local"</button>
                    </div>
                    {move || {
                        let items = hub_items(
                            &detail_messages,
                            &detail_communities,
                            &local_drafts.get(),
                            active_surface.get(),
                        );
                        match selected_hub_item(&items, selected_hub_index.get()) {
                            Some(item) => {
                                let kind = hub_item_kind(&item, selected_hub_index.get());
                                let chip_class = hub_item_chip_class(kind);
                                let surface_label = item.surface.label();
                                view! {
                                    <article class="comms-detail-card">
                                        <div class="comms-detail-head">
                                            <div>
                                                <p class="eyebrow">{surface_label}</p>
                                                <h4>{item.title.clone()}</h4>
                                            </div>
                                            <span class=chip_class>{kind}</span>
                                        </div>
                                        <p>{item.body.clone()}</p>
                                        <span class="hub-meta">{item.meta.clone()}</span>
                                        <dl class="comms-detail-grid">
                                            <div><dt>"Route"</dt><dd>{surface_label}</dd></div>
                                            <div><dt>"Workflow"</dt><dd>"Tenant change approval"</dd></div>
                                            <div><dt>"Receipt"</dt><dd>"REC-WF-7741"</dd></div>
                                            <div><dt>"Persistence"</dt><dd>"Local browser state only"</dd></div>
                                        </dl>
                                    </article>
                                }.into_any()
                            },
                            None => view! {
                                <article>
                                    <p class="eyebrow">"Empty channel"</p>
                                    <h4>"No visible items"</h4>
                                    <p>"This permitted envelope has no items for the selected channel."</p>
                                </article>
                            }.into_any(),
                        }
                    }}

                    <div class="hub-composer" aria-label="Local draft composer">
                        <label for="hub-composer-input">"Draft a local response"</label>
                        <textarea
                            id="hub-composer-input"
                            rows="3"
                            prop:value=move || draft_body.get()
                            placeholder="Type here; Queue draft keeps it local to this browser island."
                            on:input=move |event| set_draft_body.set(event_target_value(&event))
                        ></textarea>
                        <div class="composer-actions">
                            <button
                                type="button"
                                on:click=move |_| {
                                    let body = draft_body.get().trim().to_string();
                                    if body.is_empty() {
                                        return;
                                    }
                                    let mut drafts = local_drafts.get();
                                    drafts.insert(0, LocalDraft {
                                        surface: active_surface.get(),
                                        title: "Local draft queued".to_string(),
                                        body,
                                    });
                                    set_local_drafts.set(drafts);
                                    set_draft_body.set(String::new());
                                    set_selected_hub_index.set(0);
                                }
                            >
                                "Queue draft"
                            </button>
                            <button
                                type="button"
                                class="secondary"
                                on:click=move |_| set_draft_body.set(String::new())
                            >
                                "Clear"
                            </button>
                        </div>
                        <p>"Visual-only: drafts, sends, posts, and replies stay in local island state."</p>
                    </div>
                </div>

                <aside class="comms-context-rail" aria-label="People, provenance, and notification context">
                    <section>
                        <p class="screen-anchor">"PEOPLE"</p>
                        <div class="presence-stack">
                            <span><em>"OP"</em><strong>"Ops bot"</strong><small>"online"</small></span>
                            <span><em>"SR"</em><strong>"Security reviewer"</strong><small>"watching"</small></span>
                            <span><em>"FL"</em><strong>"Finance lead"</strong><small>"mail owner"</small></span>
                        </div>
                    </section>
                    <section>
                        <p class="screen-anchor">"PROVENANCE"</p>
                        <dl class="comms-kv">
                            <div><dt>"Envelope"</dt><dd>"tenant-admin"</dd></div>
                            <div><dt>"Workflow"</dt><dd>"Tenant change approval"</dd></div>
                            <div><dt>"Receipt"</dt><dd>"REC-WF-7741"</dd></div>
                        </dl>
                    </section>
                    <section class="comms-handoff-card" data-comms-handoff="true" aria-label="Local draft handoff state">
                        <p class="screen-anchor">"DRAFT HANDOFF BUS"</p>
                        <strong data-comms-handoff-title="true">"Messenger → Mail approval brief"</strong>
                        <span data-comms-handoff-status="true">"Select Promote to Mail or Publish local to carry context across surfaces."</span>
                        <dl class="comms-kv compact">
                            <div><dt>"Source"</dt><dd data-comms-handoff-source="true">"Messenger"</dd></div>
                            <div><dt>"Destination"</dt><dd data-comms-handoff-destination="true">"Mail"</dd></div>
                            <div><dt>"Audience"</dt><dd data-comms-handoff-audience="true">"CFO · SRE · Governance"</dd></div>
                            <div><dt>"Persistence"</dt><dd>"Browser local state only"</dd></div>
                        </dl>
                        <div class="comms-handoff-actions">
                            <button type="button" data-comms-action="thread-to-mail">"Promote to Mail"</button>
                            <button type="button" data-comms-action="publish-note">"Publish local"</button>
                        </div>
                    </section>
                    <section>
                        <p class="screen-anchor">"DELIVERY MATRIX"</p>
                        <div class="comms-delivery-matrix" aria-label="Local delivery readiness">
                            <span class="ready"><strong>"Messenger"</strong><em>"ops room draft"</em></span>
                            <span class="ready"><strong>"Mail"</strong><em>"approval brief"</em></span>
                            <span class="review"><strong>"Community"</strong><em>"council review"</em></span>
                            <span><strong>"Audit"</strong><em>"receipt attached"</em></span>
                        </div>
                    </section>
                    <section>
                        <p class="screen-anchor">"LOCAL NOTIFICATIONS"</p>
                        <ol class="notification-stack">
                            <li>"Draft queued locally"</li>
                            <li>"Evidence link available"</li>
                            <li>"No external send enabled"</li>
                            <li>"Workflow route preview ready"</li>
                        </ol>
                    </section>
                </aside>
            </div>
        </div>
    }
}

fn comms_receipt_bridge() -> impl IntoView {
    view! {
        <section
            class="comms-receipt-bridge"
            data-comms-receipt-bridge="true"
            aria-label="Messenger Mail Community receipt bridge"
        >
            <div class="comms-bridge-head">
                <div>
                    <p class="screen-anchor">"COMMS RECEIPT BRIDGE"</p>
                    <h4>"Messenger, Mail, and Community return to one proof packet"</h4>
                    <span data-comms-bridge-status="true">
                        "Ops room, approval brief, council post, and audit receipt are staged as one local FD-001 workload packet."
                    </span>
                </div>
                <button type="button" data-comms-bridge-action="seal">"Seal handoff"</button>
            </div>
            <div class="comms-bridge-routes" aria-label="Communication proof routes">
                <button type="button" class="selected" data-comms-bridge-route="messenger" data-comms-bridge-title="Messenger ops room" data-comms-bridge-receipt="REC-COMMS-MSG-021" data-comms-bridge-target="Ops room → Mail brief → Community note">
                    <span>"01 · Messenger"</span><strong>"Ops room thread"</strong><em>"REC-COMMS-MSG-021"</em>
                </button>
                <button type="button" data-comms-bridge-route="mail" data-comms-bridge-title="Mail approval brief" data-comms-bridge-receipt="REC-COMMS-MAIL-022" data-comms-bridge-target="Formal approval → Evidence packet">
                    <span>"02 · Mail"</span><strong>"Approval brief"</strong><em>"REC-COMMS-MAIL-022"</em>
                </button>
                <button type="button" data-comms-bridge-route="community" data-comms-bridge-title="Community council note" data-comms-bridge-receipt="REC-COMMS-COMM-023" data-comms-bridge-target="Council post → Role-visible vote">
                    <span>"03 · Community"</span><strong>"Governance note"</strong><em>"REC-COMMS-COMM-023"</em>
                </button>
                <button type="button" data-comms-bridge-route="receipt" data-comms-bridge-title="Universal receipt packet" data-comms-bridge-receipt="REC-COMMS-HANDOFF-006" data-comms-bridge-target="Audit ledger → Receipt stitching console">
                    <span>"04 · Receipt"</span><strong>"Audit stitch"</strong><em>"REC-COMMS-HANDOFF-006"</em>
                </button>
            </div>
            <aside class="comms-bridge-detail" aria-label="Selected communication receipt detail">
                <dl>
                    <div><dt>"Selected"</dt><dd data-comms-bridge-detail-title="true">"Messenger ops room"</dd></div>
                    <div><dt>"Receipt"</dt><dd data-comms-bridge-detail-receipt="true">"REC-COMMS-MSG-021"</dd></div>
                    <div><dt>"Route"</dt><dd data-comms-bridge-detail-target="true">"Ops room → Mail brief → Community note"</dd></div>
                </dl>
                <div class="comms-bridge-actions" aria-label="Communication receipt bridge actions">
                    <button type="button" data-comms-bridge-action="workflow">"Workflow"</button>
                    <button type="button" data-comms-bridge-action="cloud">"Cloud"</button>
                    <button type="button" data-comms-bridge-action="audit">"Audit receipt"</button>
                    <button type="button" data-comms-bridge-action="draft">"Draft all"</button>
                </div>
            </aside>
        </section>
    }
}

fn comms_product_board(surface: ProductSurface) -> impl IntoView {
    match surface {
        ProductSurface::Messenger => view! {
            <section class="comms-product-board messenger-board" data-comms-product-board="true" data-comms-board-surface="Messenger" aria-label="Messenger command workspace">
                <div class="comms-board-head">
                    <div><p class="screen-anchor">"MESSENGER COMMAND"</p><h4>"Ops room thread with FD-001 workload evidence"</h4><span>"Fast operational chat for dogfooding FD-001 microservices on Oyatie Cloud, with evidence links and action extraction."</span></div>
                    <div class="comms-board-actions"><span class="status-chip warning">"2 unread"</span><button type="button" data-comms-action="thread-escalate">"Escalate"</button><button type="button" data-comms-action="thread-to-mail">"Promote to Mail"</button><button type="button" data-comms-action="thread-receipt">"Attach receipt"</button></div>
                </div>
                <div class="comms-board-grid">
                    <article class="thread-transcript-card"><p class="screen-anchor">"LIVE THREAD"</p><ol class="comms-transcript"><li><strong>"Ops bot"</strong><span>"Kubernetes runtime tier drift detected in cell-us-east-2."</span><em>"09:18 · unread"</em></li><li class="mine"><strong>"Tenant admin"</strong><span>"Link rollback runbook and notify Finance before close."</span><em>"09:22 · local draft"</em></li><li><strong>"Security reviewer"</strong><span>"Need audit-chain evidence before promotion."</span><em>"09:24 · evidence"</em></li></ol></article>
                    <article><p class="screen-anchor">"ACTION EXTRACTION"</p><div class="comms-action-list"><button type="button" data-comms-action="create-task"><strong>"Create task"</strong><span>"Rollback evidence owner · due 2.1h"</span></button><button type="button" data-comms-action="link-workflow"><strong>"Link workflow"</strong><span>"PROC-PAYROLL-CLOSE critical path"</span></button><button type="button" data-comms-action="thread-to-mail"><strong>"Draft formal mail"</strong><span>"CFO + SRE approval brief"</span></button></div></article>
                    <article><p class="screen-anchor">"PARTICIPANTS"</p><div class="comms-presence-grid"><span><em>"OP"</em><strong>"Ops bot"</strong><small>"online"</small></span><span><em>"SR"</em><strong>"Security"</strong><small>"watching"</small></span><span><em>"FL"</em><strong>"Finance"</strong><small>"mail owner"</small></span><span><em>"GV"</em><strong>"Governance"</strong><small>"council"</small></span></div></article>
                </div>
            </section>
        }.into_any(),
        ProductSurface::Mail => view! {
            <section class="comms-product-board mail-board" data-comms-product-board="true" data-comms-board-surface="Mail" aria-label="Mail command workspace">
                <div class="comms-board-head">
                    <div><p class="screen-anchor">"MAIL COMMAND"</p><h4>"Formal approval brief composer"</h4><span>"Structured mail draft with recipients, subject, FD-001 workload evidence attachments, Oyatie Cloud cell context, approvals, and send preview."</span></div>
                    <div class="comms-board-actions"><span class="status-chip ai">"draft"</span><button type="button" data-comms-action="mail-preview">"Preview"</button><button type="button" data-comms-action="mail-attach">"Attach packet"</button><button type="button" data-comms-action="send-preview">"Send preview"</button></div>
                </div>
                <div class="comms-mail-compose-grid">
                    <article class="mail-envelope-card"><p class="screen-anchor">"ENVELOPE"</p><dl><div><dt>"From"</dt><dd>"Finance lead · Oyatie"</dd></div><div><dt>"To"</dt><dd>"CFO, SRE reviewer"</dd></div><div><dt>"CC"</dt><dd>"Governance council, Audit"</dd></div><div><dt>"Subject"</dt><dd>"Approval needed: payroll close + cloud rollback evidence"</dd></div></dl></article>
                    <article class="mail-body-card"><p class="screen-anchor">"DRAFT BODY"</p><div class="mail-paper"><strong>"Please review the April close packet before 18:00."</strong><p>"Payroll delta, HomeTax readiness, vendor exception, and Oyatie Cloud rollback evidence are attached as read-only receipts for the FD-001 tenant workload. No external send is enabled before live integration."</p><ol><li>"REC-PAY-2026-04-PARK"</li><li>"REC-CLOUD-MESH-4182"</li><li>"REC-WF-7741"</li></ol></div></article>
                    <article><p class="screen-anchor">"APPROVAL CHECKS"</p><div class="mail-checks"><span class="done">"Human reviewer required"</span><span class="done">"PIPA-safe body"</span><span class="review">"CFO signoff pending"</span><span>"External delivery disabled"</span></div></article>
                </div>
            </section>
        }.into_any(),
        ProductSurface::Community => view! {
            <section class="comms-product-board community-board" data-comms-product-board="true" data-comms-board-surface="Community" aria-label="Community command workspace">
                <div class="comms-board-head">
                    <div><p class="screen-anchor">"COMMUNITY COMMAND"</p><h4>"Governance council publication"</h4><span>"Role-aware community post, voting, pinned Oyatie Cloud cell context, and moderation state for FD-001 tenant-workload coordination."</span></div>
                    <div class="comms-board-actions"><span class="status-chip success">"role-aware"</span><button type="button" data-comms-action="community-pin">"Pin"</button><button type="button" data-comms-action="community-poll">"Open poll"</button><button type="button" data-comms-action="publish-note">"Publish local"</button></div>
                </div>
                <div class="community-feed-grid">
                    <article class="community-post-card"><p class="screen-anchor">"PINNED POST"</p><div class="community-post-preview"><span>"Governance council"</span><h5>"April close governance digest"</h5><p>"Payroll blocker, withholding filing readiness, Oyatie Cloud rollback evidence, and reviewer assignments are summarized for role-visible FD-001 review."</p><div><button type="button" data-comms-action="community-upvote">"▲ 24"</button><button type="button" data-comms-action="community-comment">"8 comments"</button><button type="button" data-comms-action="community-save">"Save"</button></div></div></article>
                    <article><p class="screen-anchor">"AUDIENCE"</p><div class="community-audience-grid"><span><strong>"Finance"</strong><em>"required"</em></span><span><strong>"SRE"</strong><em>"review"</em></span><span><strong>"People Ops"</strong><em>"visible"</em></span><span><strong>"Vendors"</strong><em>"blocked"</em></span></div></article>
                    <article><p class="screen-anchor">"MODERATION"</p><dl class="community-moderation"><div><dt>"Policy"</dt><dd>"PIPA-safe"</dd></div><div><dt>"Evidence"</dt><dd>"3 receipts"</dd></div><div><dt>"Publish"</dt><dd>"local only"</dd></div></dl></article>
                </div>
            </section>
        }.into_any(),
        ProductSurface::Workflow => view! {
            <section class="comms-product-board" data-comms-product-board="true"><p>"Workflow route selected."</p></section>
        }.into_any(),
    }
}

fn hub_items(
    messages: &[MessageItem],
    communities: &[CommunityItem],
    drafts: &[LocalDraft],
    surface: ProductSurface,
) -> Vec<HubItem> {
    let mut items = drafts
        .iter()
        .filter(|draft| draft.surface == surface)
        .map(|draft| HubItem {
            surface,
            source: "Local draft".to_string(),
            title: draft.title.clone(),
            body: draft.body.clone(),
            meta: "Stored in WASM island state only".to_string(),
        })
        .collect::<Vec<_>>();

    match surface {
        ProductSurface::Messenger => {
            items.extend(
                messages
                    .iter()
                    .filter(|item| !item.channel.to_ascii_lowercase().contains("mail"))
                    .map(|item| HubItem {
                        surface,
                        source: item.channel.clone(),
                        title: item.from.clone(),
                        body: item.preview.clone(),
                        meta: "Thread preview; no external message sent".to_string(),
                    }),
            );
            items.extend([
                HubItem {
                    surface,
                    source: "Workflow bot".to_string(),
                    title: "Tenant change approval output routes".to_string(),
                    body: "Messenger post links payroll delta, cloud rollback, Mail brief, Community note, and REC-WF-7741 evidence.".to_string(),
                    meta: "Evidence-linked workflow route".to_string(),
                },
                HubItem {
                    surface,
                    source: "Payroll desk".to_string(),
                    title: "4대보험 delta owner needed".to_string(),
                    body: "Park Seo-jun tier change is blocking close; finance owner can review from Action Inbox or Evidence Spine.".to_string(),
                    meta: "Unread operations thread".to_string(),
                },
                HubItem {
                    surface,
                    source: "Cloud SRE".to_string(),
                    title: "Mesh rollback note ready".to_string(),
                    body: "us-east-2 split has runbook, resource inventory, and audit-chain previews attached for reviewer context.".to_string(),
                    meta: "Evidence-linked incident thread".to_string(),
                },
            ]);
        }
        ProductSurface::Mail => {
            items.extend(
                messages
                    .iter()
                    .filter(|item| item.channel.to_ascii_lowercase().contains("mail"))
                    .map(|item| HubItem {
                        surface,
                        source: item.channel.clone(),
                        title: item.from.clone(),
                        body: item.preview.clone(),
                        meta: "Mail preview; compose is local only".to_string(),
                    }),
            );
            items.extend([
                HubItem {
                    surface,
                    source: "Draft".to_string(),
                    title: "Finance approval brief".to_string(),
                    body: "Formal mail for CFO approval includes payroll close, HomeTax transport, vendor threshold, and receipt links.".to_string(),
                    meta: "Draft · requires human send".to_string(),
                },
                HubItem {
                    surface,
                    source: "Review".to_string(),
                    title: "HomeTax attestation request".to_string(),
                    body: "118 employees validated; 사업자등록번호 confirmation and CFO attestation remain before filing preview.".to_string(),
                    meta: "Evidence-linked mail route".to_string(),
                },
                HubItem {
                    surface,
                    source: "Policy".to_string(),
                    title: "Vendor approval route exception".to_string(),
                    body: "Stripe renewal can move to one-step approval below policy threshold; Procurement and Audit are copied.".to_string(),
                    meta: "Draft · policy copy".to_string(),
                },
            ]);
        }
        ProductSurface::Community => {
            items.extend(communities.iter().map(|item| HubItem {
                surface,
                source: item.space.clone(),
                title: item.topic.clone(),
                body: item.activity.clone(),
                meta: "Community post preview; no backend write".to_string(),
            }));
            items.extend([
                HubItem {
                    surface,
                    source: "Governance council".to_string(),
                    title: "April close governance digest".to_string(),
                    body: "Summarizes payroll blocker, withholding filing readiness, cloud rollback evidence, and reviewer assignments.".to_string(),
                    meta: "Draft post · council review".to_string(),
                },
                HubItem {
                    surface,
                    source: "Policy forum".to_string(),
                    title: "Approval threshold clarification".to_string(),
                    body: "Procurement threshold note is ready for role-aware publication with Mail and Audit references attached.".to_string(),
                    meta: "Evidence-linked community note".to_string(),
                },
            ]);
        }
        ProductSurface::Workflow => {}
    }

    items
}

fn hub_item_kind(item: &HubItem, index: usize) -> &'static str {
    let haystack =
        format!("{} {} {} {}", item.source, item.title, item.body, item.meta).to_ascii_lowercase();
    if haystack.contains("draft") || haystack.contains("brief") || haystack.contains("send") {
        "draft"
    } else if haystack.contains("evidence")
        || haystack.contains("receipt")
        || haystack.contains("rec-")
    {
        "evidence"
    } else if index < 2 || haystack.contains("unread") || haystack.contains("blocking") {
        "unread"
    } else {
        "review"
    }
}

fn hub_item_chip_class(kind: &str) -> &'static str {
    match kind {
        "draft" => "status-chip ai",
        "evidence" => "status-chip success",
        "unread" => "status-chip warning",
        _ => "status-chip",
    }
}

fn selected_hub_item(items: &[HubItem], selected_index: usize) -> Option<HubItem> {
    items.get(selected_index).or_else(|| items.first()).cloned()
}

const CATALOG_FILTERS: [(&str, &str); 9] = [
    ("all", "All"),
    ("control", "Control"),
    ("cloud", "Cloud"),
    ("operations", "Ops"),
    ("trust", "Trust"),
    ("corporate", "Corporate"),
    ("daily", "Daily"),
    ("workflow", "Workflow"),
    ("no-code", "No-code"),
];

fn service_catalog_panel(modules: Vec<ModuleCard>, omitted_note: String) -> impl IntoView {
    let total_modules = modules.len();
    let attention_count = modules
        .iter()
        .filter(|module| catalog_state_for(&module.group, &module.name) == "attention")
        .count();
    let cloud_count = modules
        .iter()
        .filter(|module| catalog_group_slug(&module.group) == "cloud")
        .count();
    let trust_count = modules
        .iter()
        .filter(|module| matches!(catalog_group_slug(&module.group), "trust" | "control"))
        .count();

    view! {
        <div class="catalog-kpi-strip" aria-label="Service catalog summary">
            <div class="catalog-kpi accent"><span>"Permitted modules"</span><strong>{total_modules}</strong><small>"from server envelope"</small></div>
            <div class="catalog-kpi"><span>"Cloud dependencies"</span><strong>{cloud_count}</strong><small>"compute · network · cells"</small></div>
            <div class="catalog-kpi warn"><span>"Need attention"</span><strong>{attention_count}</strong><small>"review before promote"</small></div>
            <div class="catalog-kpi"><span>"Trust surfaces"</span><strong>{trust_count}</strong><small>"roles · audit · policy"</small></div>
            <div class="catalog-kpi"><span>"Cross-service routes"</span><strong>"7"</strong><small>"workflow → mail/community/ops"</small></div>
        </div>

        <div class="catalog-toolbar" aria-label="Catalog search and filters">
            <label class="catalog-search">
                <span aria-hidden="true">"⌕"</span>
                <input data-catalog-search="true" type="search" aria-label="Search service catalog" placeholder="Search modules, owners, dependencies..." />
            </label>
            <div class="filter-pills catalog-filters" role="toolbar" aria-label="Catalog filters">
                {CATALOG_FILTERS.into_iter().map(|(slug, label)| view! {
                    <button
                        type="button"
                        class=if slug == "all" { "fp active" } else { "fp" }
                        data-catalog-filter=slug
                    >
                        <span class="fp-dot" aria-hidden="true"></span>{label}
                    </button>
                }).collect_view()}
                <button type="button" class="fp" data-catalog-filter="attention">
                    <span class="fp-dot danger" aria-hidden="true"></span>"Attention"
                </button>
            </div>
            <span class="catalog-status" data-catalog-status="true">
                <strong data-catalog-visible-count="true">{total_modules}</strong>
                " visible · all filter · local catalog only"
            </span>
        </div>

        <div class="catalog-workspace">
            <div class="catalog-table-shell" role="region" aria-label="Permitted modules table">
                <div class="catalog-table-head" aria-hidden="true">
                    <span>"Health"</span>
                    <span>"Module"</span>
                    <span>"Category"</span>
                    <span>"Owner"</span>
                    <span>"Downstream graph"</span>
                    <span>"Actions"</span>
                </div>
                <div class="catalog-module-list">
                    {modules.into_iter().map(service_catalog_module).collect_view()}
                </div>
            </div>

            <aside id="service-graph" class="catalog-service-graph" aria-label="Service graph and module lineage">
                <div class="graph-head">
                    <p class="screen-anchor">"SERVICE GRAPH"</p>
                    <strong>"One cohesive Oyatie nervous system"</strong>
                    <span>"Workflow events fan out to built-in surfaces and return audit evidence."</span>
                </div>
                <ol class="lineage-list">
                    <li class="root"><span>"Workflow"</span><strong>"Tenant change approval"</strong><em>"root event"</em></li>
                    <li><span>"Messenger"</span><strong>"Ops room draft"</strong><em>"delivered"</em></li>
                    <li><span>"Mail"</span><strong>"Formal approval brief"</strong><em>"pending"</em></li>
                    <li><span>"Community"</span><strong>"Governance council note"</strong><em>"review"</em></li>
                    <li><span>"Cloud Ops"</span><strong>"Runbook + FinOps"</strong><em>"guarded"</em></li>
                    <li><span>"Audit"</span><strong>"Receipt spine"</strong><em>"sealed"</em></li>
                </ol>
                <div class="catalog-graph-actions" aria-label="Service graph actions">
                    <button type="button" data-catalog-graph-action="workflow">"Open workflow"</button>
                    <button type="button" data-catalog-graph-action="mail">"Mail route"</button>
                    <button type="button" data-catalog-graph-action="community">"Community route"</button>
                    <button type="button" data-catalog-graph-action="evidence">"Evidence spine"</button>
                </div>
            </aside>
        </div>

        {service_catalog_anchor_board()}

        <p class="catalog-footer-hint omitted-note">
            <span aria-hidden="true">"✦"</span>
            {omitted_note}
        </p>
    }
}

fn service_catalog_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board catalog-trust-board" aria-label="FD-001 service catalog and Oyatie Cloud tenant workload proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="catalog-fd001">
                    <p class="screen-anchor">"FD-001 MODULE CONTRACT"</p>
                    <h5>"Catalog is the service graph manifest"</h5>
                    <p>
                        "Core, Workflow, Messenger, Mail, Community, Finance, Identity, Ontology, Intelligence, and Daily Work are presented as one permitted FD-001 tenant workload graph."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="stage-catalog">"Stage manifest"</button>
                        <button type="button" data-trust-proof-action="route-workflow">"Workflow proof"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="catalog-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD ADMISSION"</p>
                    <h5>"Substrate dependencies are visible first"</h5>
                    <p>
                        "Cloud cells, policy gates, resource inventory, deployment gates, FinOps, and audit receipts make hosting readiness explicit before service claims."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud substrate"</button>
                        <button type="button" data-trust-proof-action="route-gates">"Deployment gates"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="catalog-local">
                    <p class="screen-anchor">"LOCAL-ONLY CATALOG OPS"</p>
                    <h5>"Request, pin, and route without provisioning"</h5>
                    <p>
                        "Operators can filter modules, inspect dependencies, pin rows, and route reviewers visually; no service admission, IAM, deploy, billing, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-policy">"Policy"</button>
                        <button type="button" data-trust-proof-action="route-audit">"Audit ledger"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Service catalog ready · FD-001 module graph dogfoods Oyatie Cloud as local tenant workload proof."
                </span>
                <div class="trust-anchor-routes" aria-label="Service catalog connected routes">
                    <button type="button" data-trust-proof-action="route-finance">"Finance"</button>
                    <button type="button" data-trust-proof-action="route-identity">"Identity"</button>
                    <button type="button" data-trust-proof-action="route-daily">"Daily Work"</button>
                    <button type="button" data-trust-proof-action="route-evidence">"Evidence"</button>
                </div>
            </div>
        </div>
    }
}

fn service_catalog_module(module: ModuleCard) -> impl IntoView {
    let group_slug = catalog_group_slug(&module.group);
    let state = catalog_state_for(&module.group, &module.name);
    let state_label = catalog_state_label(state);
    let health_class = format!("health-dot health-{state}");
    let owner = catalog_owner_for(&module.group, &module.name);
    let avatar = catalog_owner_avatar(owner);
    let criticality = catalog_criticality_for(&module.group, &module.name);
    let criticality_class = format!("crit crit-{criticality}");
    let route = catalog_route_for(&module.name);
    let dependency = catalog_dependency_for(&module.group, &module.name);
    let sidepeek_desc = format!(
        "{} · {} · {}",
        module.description, owner, "local visual-only catalog module"
    );
    let sidepeek_id = format!("CAT-{}", catalog_code_for(&module.name));

    view! {
        <article
            class="catalog-module-row module-card"
            data-catalog-module="true"
            data-catalog-group=group_slug
            data-catalog-state=state
        >
            <span class=health_class aria-label=state_label></span>
            <div class="catalog-module-main">
                <button
                    type="button"
                    class="catalog-module-title"
                    data-sidepeek-trigger="catalog-module"
                    data-sidepeek-title=module.name.clone()
                    data-sidepeek-id=sidepeek_id
                    data-sidepeek-desc=sidepeek_desc
                    data-sidepeek-owner=owner
                    data-sidepeek-risk=state_label
                    data-sidepeek-sla="4.0h review window"
                >
                    {module.name.clone()}
                </button>
                <p>{module.description.clone()}</p>
                <code>{catalog_code_for(&module.name)}</code>
            </div>
            <span class="cat-tag">{module.group.clone()}</span>
            <span class="owner-cell">
                <span class="avatar-xs" aria-hidden="true">{avatar}</span>
                <span>{owner}</span>
            </span>
            <span class="catalog-dependency-chain">
                <em>"Workflow"</em>
                <i aria-hidden="true">"→"</i>
                <em>{dependency}</em>
                <i aria-hidden="true">"→"</i>
                <em>"Audit"</em>
                <span class=criticality_class>{criticality}</span>
            </span>
            <span class="catalog-row-actions">
                <button type="button" data-catalog-action="open" data-catalog-target=route>
                    {module.action_label}
                </button>
                <button type="button" data-catalog-action="pin">"Pin"</button>
                <button type="button" data-catalog-action="request">"Request access"</button>
            </span>
        </article>
    }
}

fn catalog_group_slug(group: &str) -> &'static str {
    match group {
        "Control" => "control",
        "Cloud" => "cloud",
        "Operations" => "operations",
        "Trust" => "trust",
        "Corporate" => "corporate",
        "Daily" => "daily",
        "Workflow" => "workflow",
        "No-code" => "no-code",
        "Healthcare" => "healthcare",
        _ => "other",
    }
}

fn catalog_state_for(group: &str, name: &str) -> &'static str {
    if name.contains("Audit") {
        "sealed"
    } else if group == "Cloud" && name.contains("Network") {
        "attention"
    } else if matches!(group, "Operations" | "Workflow") {
        "review"
    } else {
        "ready"
    }
}

fn catalog_state_label(state: &str) -> &'static str {
    match state {
        "attention" => "attention",
        "review" => "review",
        "sealed" => "sealed",
        _ => "ready",
    }
}

fn catalog_owner_for(group: &str, name: &str) -> &'static str {
    if name.contains("Workflow") || group == "No-code" {
        "Automation council"
    } else {
        match group {
            "Control" => "Tenant admin",
            "Cloud" => "Infrastructure operations",
            "Operations" => "FinOps lead",
            "Trust" => "Security reviewer",
            "Corporate" => "Corporate ops",
            "Daily" => "Work home owner",
            "Workflow" => "Approval owner",
            "Healthcare" => "Clinical ops",
            _ => "Module owner",
        }
    }
}

fn catalog_owner_avatar(owner: &str) -> &'static str {
    match owner {
        "Tenant admin" => "TA",
        "Infrastructure operations" => "PO",
        "FinOps lead" => "FO",
        "Security reviewer" => "SR",
        "Corporate ops" => "CO",
        "Work home owner" => "WH",
        "Automation council" => "AC",
        "Approval owner" => "AO",
        "Clinical ops" => "CL",
        _ => "MO",
    }
}

fn catalog_criticality_for(group: &str, name: &str) -> &'static str {
    if name.contains("Network") || name.contains("Tenant Admin") {
        "P0"
    } else if matches!(group, "Cloud" | "Trust" | "Operations") {
        "P1"
    } else {
        "P2"
    }
}

fn catalog_route_for(name: &str) -> &'static str {
    if name.contains("Workflow") || name.contains("Approvals") {
        "#workflow-studio"
    } else if name.contains("Cloud") || name.contains("FinOps") {
        "#cloud-ops-cockpit"
    } else if name.contains("Audit") {
        "#audit-ledger"
    } else if name.contains("Human") || name.contains("Clinical") || name.contains("Patient") {
        "#identity-employees"
    } else {
        "#work-hub"
    }
}

fn catalog_dependency_for(group: &str, name: &str) -> &'static str {
    if name.contains("FinOps") {
        "Ledger"
    } else if name.contains("Audit") {
        "Evidence"
    } else if name.contains("Workflow") || group == "Workflow" {
        "Messenger/Mail"
    } else if group == "Cloud" {
        "Cloud Ops"
    } else if matches!(group, "Corporate" | "Daily") {
        "Work hub"
    } else if group == "Healthcare" {
        "Care hub"
    } else {
        "Policy"
    }
}

fn catalog_code_for(name: &str) -> String {
    let code = name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || character.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .map(|part| part.to_ascii_uppercase())
        .collect::<Vec<_>>()
        .join("-");
    format!("OYA-{code}")
}

#[expect(
    clippy::too_many_arguments,
    reason = "Workflow panel keeps each reactive control explicit pending a production component boundary."
)]
fn workflow_studio_panel(
    name: String,
    goal: String,
    nodes: Vec<WorkflowNode>,
    selected_node: Option<WorkflowNode>,
    set_selected_node_id: WriteSignal<String>,
    workflow_tool: ReadSignal<WorkflowTool>,
    set_workflow_tool: WriteSignal<WorkflowTool>,
    draft_node_count: ReadSignal<usize>,
    set_draft_node_count: WriteSignal<usize>,
    set_active_surface: WriteSignal<ProductSurface>,
) -> impl IntoView {
    view! {
        <section id="workflow-studio" class="panel workflow-panel cohesive-workflow" aria-labelledby="workflow-title">
            <div class="workflow-topbar">
                <div>
                    <p class="eyebrow">"Workflow Studio"</p>
                    <h3 id="workflow-title">{name}</h3>
                    <div class="workflow-doc-meta">
                        <span>"v18 · draft"</span>
                        <span>"Owner · tenant admin"</span>
                        <span>"SLA · 4.0h"</span>
                    </div>
                </div>
                <div class="workflow-run-chip" aria-label="Run state preview">
                    <span></span>
                    {move || match workflow_tool.get() {
                        WorkflowTool::Select => "draft · select mode",
                        WorkflowTool::Connect => "draft · connect mode",
                        WorkflowTool::Simulate => "simulation preview",
                    }}
                </div>
                <div class="workflow-actions" aria-label="Workflow actions preview">
                    <button type="button">"Fit"</button>
                    <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Select)>"Clear run"</button>
                    <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Select)>"Validate"</button>
                    <button class="primary-action" type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Simulate)>"Run"</button>
                    <button type="button" on:click=move |_| set_draft_node_count.set(draft_node_count.get() + 1)>"Add block"</button>
                    <button class="dark-action" type="button">"Publish"</button>
                </div>
            </div>
            {workflow_process_chrome()}
            {workflow_output_bus()}
            <p class="panel-intro">{goal}</p>

            <div class="workflow-modebar" role="toolbar" aria-label="Workflow editor modes">
                {WorkflowTool::ALL.into_iter().map(|tool| view! {
                    <button
                        type="button"
                        class=move || if workflow_tool.get() == tool { "active" } else { "" }
                        aria-pressed=move || workflow_tool.get() == tool
                        on:click=move |_| set_workflow_tool.set(tool)
                    >
                        {tool.label()}
                    </button>
                }).collect_view()}
            </div>
            {workflow_lens_toolbar()}

            <div class="workflow-ide">
                <aside class="workflow-palette" aria-label="Workflow building blocks">
                    <div class="palette-search">
                        <span aria-hidden="true">"⌕"</span>
                        <input data-workflow-palette-search="true" aria-label="Search workflow blocks" placeholder="Search nodes..." />
                        <kbd>"⌘K"</kbd>
                    </div>
                    <div class="palette-heading"><span>"Primitives"</span><em>"12"</em></div>
                    {[
                        ("System task", "Deterministic step · 0 ms", "S"),
                        ("Approval", "Single, parallel, or quorum", "A"),
                        ("Validation", "Rule check · halts on fail", "V"),
                        ("External call", "HTTP, RPC, or connector", "E"),
                        ("Branch / Switch", "Multi-way condition split", "B"),
                        ("Wait / Timer", "Until time · or duration", "W"),
                        ("Loop / For-each", "Iterate over collection", "L"),
                        ("AI step", "Suggest · classify · extract", "⌥A"),
                        ("중단 / 에스컬레이트", "CFO 알림 · 실행 중단", "H"),
                        ("Form / Input", "Collect data from human", "F"),
                        ("Webhook trigger", "Inbound event start", "T"),
                        ("End / Receipt", "Emit immutable event", "⌘E"),
                    ].into_iter().map(|(label, detail, key)| view! {
                        <button
                            type="button"
                            data-palette-item="primitive"
                            on:click=move |_| set_draft_node_count.set(draft_node_count.get() + 1)
                        >
                            <span>{label}</span><small>{detail}</small><kbd>{key}</kbd>
                        </button>
                    }).collect_view()}
                    <div class="palette-heading"><span>"Actions"</span><em>"6"</em></div>
                    {[
                        ("Task", "Create a governed work item", "T"),
                        ("HTTP request", "Call external REST/HTTP", "H"),
                        ("Database", "Read/write a record", "D"),
                        ("Transform", "Reshape the payload", "X"),
                        ("Filter", "Drop failed items", "F"),
                        ("Write to doc", "Append a row / line", "W"),
                    ].into_iter().map(|(label, detail, key)| view! {
                        <button
                            type="button"
                            data-palette-item="action"
                            on:click=move |_| set_draft_node_count.set(draft_node_count.get() + 1)
                        >
                            <span>{label}</span><small>{detail}</small><kbd>{key}</kbd>
                        </button>
                    }).collect_view()}
                    <div class="palette-heading"><span>"Logic"</span><em>"5"</em></div>
                    {[
                        ("If / Branch", "Two-way condition split", "I"),
                        ("Switch", "Multi-way routing", "S"),
                        ("Loop / For-each", "Iterate collection", "L"),
                        ("Wait", "Delay or duration", "W"),
                        ("Merge", "Wait for branches", "M"),
                    ].into_iter().map(|(label, detail, key)| view! {
                        <button
                            type="button"
                            data-palette-item="logic"
                            on:click=move |_| set_draft_node_count.set(draft_node_count.get() + 1)
                        >
                            <span>{label}</span><small>{detail}</small><kbd>{key}</kbd>
                        </button>
                    }).collect_view()}
                    <div class="palette-heading"><span>"Built-in surfaces"</span><em>"3"</em></div>
                    <button type="button" data-palette-item="surface" on:click=move |_| set_active_surface.set(ProductSurface::Messenger)><span>"Messenger post"</span><small>"Route run summary to Ops room"</small><kbd>"M"</kbd></button>
                    <button type="button" data-palette-item="surface" on:click=move |_| set_active_surface.set(ProductSurface::Mail)><span>"Mail draft"</span><small>"Formal approval note"</small><kbd>"⌘M"</kbd></button>
                    <button type="button" data-palette-item="surface" on:click=move |_| set_active_surface.set(ProductSurface::Community)><span>"Community note"</span><small>"Publish governed update"</small><kbd>"C"</kbd></button>
                    <div class="palette-heading"><span>"Connectors"</span><em>"9"</em></div>
                    <div class="workflow-connector-grid" aria-label="Workflow connector shortcuts">
                        {[
                            ("국세", "HomeTax"),
                            ("국민", "NPS / 4대"),
                            ("신한", "Shinhan"),
                            ("T", "Toss"),
                            ("K", "Kakao Work"),
                            ("#", "Slack"),
                            ("G", "Workspace"),
                            ("Q", "QuickBooks"),
                            ("N", "Notion"),
                        ].into_iter().map(|(mark, label)| view! {
                            <button type="button" data-palette-item="connector" on:click=move |_| set_draft_node_count.set(draft_node_count.get() + 1)>
                                <strong>{mark}</strong><span>{label}</span>
                            </button>
                        }).collect_view()}
                    </div>
                </aside>

                {workflow_canvas(nodes.clone(), set_selected_node_id, workflow_tool, set_workflow_tool)}

                <aside class="workflow-inspector" aria-label="Selected workflow node inspector">
                    <div class="inspector-tabs" aria-hidden="true">
                        <span class="active">"Inspector"</span>
                        <span>"Run log"</span>
                    </div>
                    {selected_node_view(selected_node)}
                    {workflow_property_form()}
                    <dl class="inspector-fields">
                        <div><dt>"Guardrail"</dt><dd>"Human review before action"</dd></div>
                        <div><dt>"Output"</dt><dd>"Task · message · evidence draft"</dd></div>
                        <div><dt>"Execution"</dt><dd>"Disabled until live integration"</dd></div>
                    </dl>
                    <div class="inspector-stat-grid" aria-label="Selected node run statistics">
                        <div><span>"Avg"</span><strong>"0.8s"</strong></div>
                        <div><span>"P95"</span><strong>"2.1s"</strong></div>
                        <div><span>"Errors"</span><strong>"0"</strong></div>
                        <div><span>"Cost"</span><strong>"$0.03"</strong></div>
                    </div>
                    <div class="run-log-preview">
                        <p class="eyebrow">"Run log"</p>
                        <ol>
                            <li><time>"10:31"</time><span>"Validation preview passed"</span></li>
                            <li><time>"10:32"</time><span>"Messenger/Mail/Community drafts generated"</span></li>
                            <li><time>"10:33"</time><span>"Audit receipt staged locally"</span></li>
                        </ol>
                    </div>
                </aside>
            </div>

            <div class="workflow-statusbar" aria-label="Workflow editor status">
                <span>{move || format!("Nodes: {}", nodes.len())}</span>
                <span>{move || format!("Local blocks: {}", draft_node_count.get())}</span>
                <span>"Messenger/Mail/Community outputs are drafts"</span>
                <span>{move || match workflow_tool.get() {
                    WorkflowTool::Select => "Ready · staged",
                    WorkflowTool::Connect => "Click nodes to visualize links",
                    WorkflowTool::Simulate => "Previewing run path only",
                }}</span>
            </div>
        </section>
    }
}

fn workflow_output_bus() -> impl IntoView {
    view! {
        <div
            class="workflow-output-bus"
            data-workflow-output-bus="true"
            aria-label="Workflow output bus for FD-001 tenant workload routes"
        >
            <div class="workflow-output-head">
                <p class="screen-anchor">"FD-001 OUTPUT BUS"</p>
                <strong>"Run preview emits tenant workload drafts"</strong>
                <span data-workflow-output-status="true">
                    "Idle · run/validate/publish stays local until a route is selected"
                </span>
            </div>
            <div class="workflow-output-flow" aria-label="Workflow output routes">
                <button type="button" class="selected" data-workflow-output-route="messenger">
                    <span>"01"</span>
                    <strong>"Messenger"</strong>
                    <em>"Ops room run note"</em>
                </button>
                <button type="button" data-workflow-output-route="mail">
                    <span>"02"</span>
                    <strong>"Mail"</strong>
                    <em>"Approval brief"</em>
                </button>
                <button type="button" data-workflow-output-route="community">
                    <span>"03"</span>
                    <strong>"Community"</strong>
                    <em>"Council digest"</em>
                </button>
                <button type="button" data-workflow-output-route="evidence">
                    <span>"04"</span>
                    <strong>"Evidence"</strong>
                    <em>"Receipt spine"</em>
                </button>
            </div>
            <aside class="workflow-output-proof" aria-label="FD-001 and Oyatie Cloud proof context">
                <dl>
                    <div>
                        <dt>"Product goal"</dt>
                        <dd>"FD-001 delivery"</dd>
                    </div>
                    <div>
                        <dt>"Substrate"</dt>
                        <dd>"Oyatie Cloud · cell-us-east-2"</dd>
                    </div>
                    <div>
                        <dt>"Receipt"</dt>
                        <dd data-workflow-output-receipt="true">"REC-FD001-WF-018 · draft"</dd>
                    </div>
                </dl>
            </aside>
        </div>
    }
}

fn workflow_process_chrome() -> impl IntoView {
    view! {
        <div class="workflow-process-chrome" aria-label="Workflow process command chrome">
            <div class="workflow-process-meta">
                <span>"PROCESS"</span>
                <strong>"PROC-PAYROLL-CLOSE"</strong>
                <span>"OWNER"</span>
                <strong>"Hyo-jin Park · #274"</strong>
                <span>"SLA"</span>
                <strong>"4.0d"</strong>
            </div>
            <div class="workflow-process-actions">
                <button type="button" data-workflow-process-action="validate">"✓ Validate"</button>
                <button type="button" data-workflow-process-action="simulate">"◷ Simulate"</button>
                <button type="button" data-workflow-process-action="diff">"↯ Diff v17 → v18"</button>
                <button type="button" class="dark-action" data-workflow-process-action="publish">"게시 v18"</button>
            </div>
            <span class="workflow-process-status" data-workflow-process-status="true">"autosaved · local visual IDE"</span>
        </div>
    }
}

fn workflow_lens_toolbar() -> impl IntoView {
    view! {
        <div class="workflow-lens-toolbar" aria-label="Workflow layout, overlay, and filter controls">
            <div class="workflow-lens-group">
                <span>"LAYOUT"</span>
                <button type="button" class="active" data-workflow-lens="Graph">"Graph"</button>
                <button type="button" data-workflow-lens="Swimlanes">"Swimlanes"</button>
                <button type="button" data-workflow-lens="Timeline">"Timeline"</button>
                <button type="button" data-workflow-lens="Tree">"Tree"</button>
            </div>
            <div class="workflow-lens-group">
                <span>"OVERLAY"</span>
                <button type="button" data-workflow-overlay="Cycle">"Cycle"</button>
                <button type="button" class="active" data-workflow-overlay="Cost">"Cost"</button>
                <button type="button" data-workflow-overlay="Owner">"Owner"</button>
                <button type="button" data-workflow-overlay="Risk">"Risk"</button>
                <button type="button" data-workflow-overlay="Off">"Off"</button>
            </div>
            <div class="workflow-lens-group">
                <span>"FILTER"</span>
                <button type="button" data-workflow-filter="All">"All"</button>
                <button type="button" class="active" data-workflow-filter="Critical path">"Critical path"</button>
                <button type="button" data-workflow-filter="Bottlenecks">"Bottlenecks"</button>
                <button type="button" data-workflow-filter="AI suggestions">"AI suggestions"</button>
            </div>
        </div>
    }
}

fn workflow_canvas(
    nodes: Vec<WorkflowNode>,
    set_selected_node_id: WriteSignal<String>,
    workflow_tool: ReadSignal<WorkflowTool>,
    set_workflow_tool: WriteSignal<WorkflowTool>,
) -> impl IntoView {
    let board_nodes = nodes.clone();
    let toolbar_nodes = nodes.clone();

    view! {
        <div class="workflow-canvas island-frame" role="img" aria-label="Interactive workflow canvas preview">
            <div class="workflow-toolbar" aria-label="Workflow studio tools">
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Select)>"Select"</button>
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Connect)>"Connect"</button>
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Simulate)>"Simulate"</button>
                <span class="island-label">"interactive island"</span>
            </div>
            <div
                class=move || match workflow_tool.get() {
                    WorkflowTool::Select => "workflow-board selectable",
                    WorkflowTool::Connect => "workflow-board connectable",
                    WorkflowTool::Simulate => "workflow-board simulating",
                }
                data-workflow-board="true"
            >
                <svg class="workflow-board-edges" viewBox="0 0 860 430" aria-hidden="true" focusable="false">
                    <defs>
                        <marker id="workflow-board-arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
                            <path d="M0,0 L0,6 L9,3 z" class="workflow-arrow" />
                        </marker>
                    </defs>
                    {board_edges(&board_nodes).into_iter().map(|(from, to, path)| view! {
                        <path
                            class="workflow-edge workflow-board-edge"
                            data-edge-from=from
                            data-edge-to=to
                            d=path
                            marker-end="url(#workflow-board-arrow)"
                        />
                    }).collect_view()}
                </svg>
                {workflow_canvas_metrics()}
                {board_nodes.into_iter().enumerate().map(|(index, node)| {
                    let id = node.id.clone();
                    let id_attr = id.clone();
                    let label_attr = node.label.clone();
                    let label_text = node.label.clone();
                    let kind_attr = node.kind.clone();
                    let kind_text = node.kind.clone();
                    let desc_attr = node.explanation.clone();
                    let desc_text = node.explanation.clone();
                    view! {
                        <button
                            type="button"
                            class=move || match workflow_tool.get() {
                                WorkflowTool::Select => if index == 0 { "workflow-card active selectable" } else { "workflow-card selectable" },
                                WorkflowTool::Connect => "workflow-card connectable",
                                WorkflowTool::Simulate => "workflow-card simulating",
                            }
                            style=format!(
                                "left: {}px; top: {}px",
                                workflow_board_x(index),
                                workflow_board_y(index, &node)
                            )
                            data-workflow-card="true"
                            data-node-id=id_attr
                            data-node-label=label_attr
                            data-node-kind=kind_attr
                            data-node-desc=desc_attr
                            on:click=move |_| set_selected_node_id.set(id.clone())
                        >
                            <span class="board-port in" aria-hidden="true"></span>
                            <span class="board-port out" aria-hidden="true"></span>
                            <span class="workflow-card-type">{kind_text}</span>
                            <strong>{label_text}</strong>
                            <small>{desc_text}</small>
                        </button>
                    }
                }).collect_view()}
                <div class="workflow-ai-suggestion" aria-label="AI workflow suggestion">
                    <p>"AI · WORKFLOW SUGGESTION"</p>
                    <strong>"CFO 승인이 SLA를 초과할 때 자동 위임 조건을 추가"</strong>
                    <span>"conf 0.86 · model oyatie-flow-sense-1.4 · why →"</span>
                    <div>
                        <button type="button" data-workflow-suggestion="dismiss">"Dismiss"</button>
                        <button type="button" data-workflow-suggestion="preview">"Preview"</button>
                        <button type="button" data-workflow-suggestion="apply">"Apply"</button>
                    </div>
                </div>
                <div class="canvas-drop-hint" aria-hidden="true">
                    "Drag blocks here · connect ports visually · local only"
                </div>
            </div>
            <div class="canvas-footer">
                <div class="zoom-controls" aria-label="Visual zoom controls">
                    <button type="button">"−"</button>
                    <span>"82%"</span>
                    <button type="button">"+"</button>
                </div>
                <div class="mini-map" aria-hidden="true">
                    <span></span><span></span><span></span><span></span>
                </div>
            </div>
            <div class="node-toolbar" aria-label="Select workflow node to inspect">
                {toolbar_nodes.into_iter().map(|node| {
                    let id = node.id.clone();
                    view! {
                        <button type="button" on:click=move |_| set_selected_node_id.set(id.clone())>
                            {node.label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

fn workflow_canvas_metrics() -> impl IntoView {
    view! {
        <div class="workflow-canvas-metrics" aria-label="Workflow simulation metrics overlay">
            <span><small>"CYCLE"</small><strong>"5.4d"</strong><em>"+1.4 vs target"</em></span>
            <span><small>"TARGET"</small><strong>"4.0d"</strong><em>"SLA limit"</em></span>
            <span><small>"COST"</small><strong>"₩2.18M"</strong><em>"delay cost"</em></span>
            <span><small>"REWORK"</small><strong>"8%"</strong><em>"2 loops"</em></span>
        </div>
    }
}

fn workflow_property_form() -> impl IntoView {
    view! {
        <form class="workflow-property-form" aria-label="Selected workflow node properties">
            <label>
                <span>"LABEL · KO"</span>
                <input data-workflow-prop="label-ko" value="재무 검토 · 사인오프" />
            </label>
            <label>
                <span>"TYPE"</span>
                <select data-workflow-prop="type">
                    <option>"Single · auto-delegate"</option>
                    <option>"Parallel quorum"</option>
                    <option>"Human review stop"</option>
                </select>
            </label>
            <label>
                <span>"OWNER"</span>
                <select data-workflow-prop="owner">
                    <option>"Sarah Kim · EMP-188 · HR Manager"</option>
                    <option>"Choi Yu-na · CFO"</option>
                    <option>"David Chen · Delegate"</option>
                </select>
            </label>
            <div class="workflow-form-row">
                <label><span>"SLA TARGET"</span><input data-workflow-prop="sla" value="1.2d" /></label>
                <label><span>"ESCALATE AFTER"</span><input data-workflow-prop="escalate" value="0.8d" /></label>
            </div>
            <fieldset class="workflow-rule-stack">
                <legend>"승인 조건"</legend>
                <label><span>"1"</span><input data-workflow-prop="rule-1" value="payroll.gross > ₩500,000,000" /></label>
                <label><span>"2"</span><input data-workflow-prop="rule-2" value="policy.P0 == active" /></label>
                <button type="button" data-workflow-process-action="add-condition">"+ Add condition"</button>
            </fieldset>
        </form>
    }
}

fn board_edges(nodes: &[WorkflowNode]) -> Vec<(String, String, String)> {
    nodes
        .windows(2)
        .enumerate()
        .map(|(index, pair)| {
            let from = &pair[0];
            let to = &pair[1];
            let start_x = workflow_board_x(index) + 156;
            let start_y = workflow_board_y(index, from) + 36;
            let end_x = workflow_board_x(index + 1);
            let end_y = workflow_board_y(index + 1, to) + 36;
            let control_delta = ((end_x - start_x).abs() / 2).max(64);
            let path = format!(
                "M {start_x} {start_y} C {} {start_y}, {} {end_y}, {end_x} {end_y}",
                start_x + control_delta,
                end_x - control_delta,
            );
            (from.id.clone(), to.id.clone(), path)
        })
        .collect()
}

fn workflow_board_x(index: usize) -> i32 {
    55 + ((index as i32 % 4) * 156)
}

fn workflow_board_y(index: usize, node: &WorkflowNode) -> i32 {
    if node.id.starts_with("draft-block-") {
        258 + ((index as i32 / 4) * 104)
    } else {
        72 + ((index as i32 / 4) * 112)
    }
}

fn selected_node_view(node: Option<WorkflowNode>) -> impl IntoView {
    match node {
        Some(node) => view! {
            <aside class="node-inspector" aria-live="polite">
                <p class="eyebrow">"Selected node"</p>
                <h4>{node.label}</h4>
                <p><strong>{node.kind}</strong>" · "{node.explanation}</p>
            </aside>
        }
        .into_any(),
        None => view! {
            <aside class="node-inspector" aria-live="polite">
                <p>"Select a node to inspect its workflow, ontology, and guardrail meaning."</p>
            </aside>
        }
        .into_any(),
    }
}

fn ontology_list(items: Vec<OntologyFact>) -> impl IntoView {
    let fact_count = items.len();
    view! {
        <div class="ontology-command-console" data-ontology-console="true">
            <div class="ontology-console-head">
                <div>
                    <p class="screen-anchor">"ONTOLOGY · FD-001 TENANT WORKLOAD MAP"</p>
                    <h4>"What exists, who can see it, and where it runs"</h4>
                    <span>"Typed entities connect FD-001 tenant workload delivery to Oyatie Cloud cells, policy envelopes, workflow outputs, and evidence receipts."</span>
                </div>
                <div class="ontology-console-actions">
                    <span class="status-chip success" data-ontology-status="true">{format!("{fact_count} facts · 7 workload nodes · local graph")}</span>
                    <button type="button" data-ontology-action="lineage">"Trace lineage"</button>
                    <button type="button" data-ontology-action="policy">"Policy view"</button>
                    <button type="button" data-ontology-action="evidence">"Evidence"</button>
                </div>
            </div>

            <div class="ontology-topology-grid" aria-label="FD-001 tenant workload ontology graph">
                <button type="button" class="ontology-node root selected" data-ontology-node="Tenant" data-node-route="workload" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Tenant" data-sidepeek-id="ONT-TENANT" data-sidepeek-desc="Tenant owns the permitted FD-001 module set and runtime envelope." data-sidepeek-owner="Tenant admin" data-sidepeek-risk="Visible" data-sidepeek-sla="Local graph only"><span>"TENANT"</span><strong>"Tenant Admin"</strong><em>"owns envelope"</em></button>
                <button type="button" class="ontology-node workload" data-ontology-node="FD-001 Workloads" data-node-route="workflow" data-sidepeek-trigger="ontology-node" data-sidepeek-title="FD-001 workload set" data-sidepeek-id="ONT-FD001" data-sidepeek-desc="Core FD-001 microservices are represented as tenant workloads for dogfood validation." data-sidepeek-owner="Product delivery" data-sidepeek-risk="P0" data-sidepeek-sla="Dogfood proving loop"><span>"FD-001"</span><strong>"Microservice workloads"</strong><em>"product goal"</em></button>
                <button type="button" class="ontology-node cloud" data-ontology-node="Oyatie Cloud" data-node-route="cloud" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Oyatie Cloud substrate" data-sidepeek-id="ONT-CLOUD" data-sidepeek-desc="Hyperscaler-grade substrate hosts dogfood tenant workloads and exposes cell posture." data-sidepeek-owner="Cloud substrate" data-sidepeek-risk="Substrate proof" data-sidepeek-sla="99.95 target staged"><span>"CLOUD"</span><strong>"Cell substrate"</strong><em>"hosts tenants"</em></button>
                <button type="button" class="ontology-node workflow" data-ontology-node="Workflow" data-node-route="workflow" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Workflow runtime" data-sidepeek-id="ONT-WORKFLOW" data-sidepeek-desc="Workflow coordinates payroll close, approval, comms outputs, and receipts." data-sidepeek-owner="Workflow Studio" data-sidepeek-risk="Governed" data-sidepeek-sla="4.0h gate"><span>"FLOW"</span><strong>"Workflow"</strong><em>"orchestrates"</em></button>
                <button type="button" class="ontology-node comms" data-ontology-node="Built-in Comms" data-node-route="mail" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Built-in communications" data-sidepeek-id="ONT-COMMS" data-sidepeek-desc="Messenger, Mail, and Community receive workflow outputs without external send." data-sidepeek-owner="Work Hub" data-sidepeek-risk="Local only" data-sidepeek-sla="No backend send"><span>"COMMS"</span><strong>"Messenger · Mail · Community"</strong><em>"outputs"</em></button>
                <button type="button" class="ontology-node evidence" data-ontology-node="Evidence" data-node-route="evidence" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Evidence spine" data-sidepeek-id="ONT-EVIDENCE" data-sidepeek-desc="Receipts bind workload state, approvals, messages, and deployment gates." data-sidepeek-owner="Audit spine" data-sidepeek-risk="Immutable staged" data-sidepeek-sla="Sealed draft"><span>"AUDIT"</span><strong>"Evidence spine"</strong><em>"proves"</em></button>
                <button type="button" class="ontology-node policy" data-ontology-node="Policy" data-node-route="identity" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Policy envelope" data-sidepeek-id="ONT-POLICY" data-sidepeek-desc="Role, data-class, residency, and autonomy ceilings decide visibility and action eligibility." data-sidepeek-owner="Governance" data-sidepeek-risk="Guardrail" data-sidepeek-sla="Human review"><span>"POLICY"</span><strong>"Access envelope"</strong><em>"permits"</em></button>
            </div>

            <div class="ontology-fact-matrix" aria-label="Current permitted ontology facts">
                {items.into_iter().enumerate().map(|(index, item)| {
                    let key = format!("FACT-{:02}", index + 1);
                    view! {
                        <article class="ontology-fact-card" data-ontology-fact="true">
                            <span class="status-chip">{key}</span>
                            <strong>{item.entity}</strong>
                            <em>{item.relation}</em>
                            <p>{item.access_reason}</p>
                            <div>
                                <button type="button" data-ontology-action="inspect-fact">"Inspect"</button>
                                <button type="button" data-ontology-action="route-workflow">"Workflow"</button>
                            </div>
                        </article>
                    }
                }).collect_view()}
            </div>

            <div class="ontology-proof-rail" aria-label="Substrate proof contract">
                <article><p class="screen-anchor">"SUBSTRATE PROOF"</p><strong>"FD-001 runs as tenant workloads before service claim"</strong><span>"Cloud cells, policy envelopes, and evidence receipts prove the substrate can host real production tenants."</span></article>
                <article><p class="screen-anchor">"VISIBILITY"</p><strong>"Role + data-class gates"</strong><span>"Tenant admin can inspect posture; hidden modules remain server-derived, not client hidden."</span></article>
                <article><p class="screen-anchor">"GRAPH STATUS"</p><strong data-ontology-detail="true">"Tenant selected · workload lineage visible"</strong><span>"Click nodes or facts to stage local graph state."</span></article>
            </div>
        </div>
    }
}

fn suggestion_list(items: Vec<IntelligenceSuggestion>) -> impl IntoView {
    let suggestion_count = items.len();
    view! {
        <div class="intelligence-command-console" data-intelligence-console="true">
            <div class="intelligence-console-head">
                <div>
                    <p class="screen-anchor">"GOVERNED AI · DOGFOOD ADVISOR"</p>
                    <h4>"Recommendations that can explain, route, and prove themselves"</h4>
                    <span>"AI suggestions stay read-only until a human routes them to Workflow, Mail, Community, or Evidence."</span>
                </div>
                <div class="intelligence-console-actions">
                    <span class="status-chip warning" data-intelligence-status="true">{format!("{suggestion_count} suggestions · human gated")}</span>
                    <button type="button" data-intelligence-action="evaluate">"Run eval"</button>
                    <button type="button" data-intelligence-action="explain">"Explain"</button>
                    <button type="button" data-intelligence-action="route-evidence">"Evidence"</button>
                </div>
            </div>

            <div class="intelligence-score-strip" aria-label="Governed AI evaluation summary">
                <span><strong>0.86</strong><small>decision confidence</small></span>
                <span><strong>14</strong><small>policy checks</small></span>
                <span><strong>0</strong><small>auto-executions</small></span>
                <span><strong>3</strong><small>FD-001 routes</small></span>
            </div>

            <div class="intelligence-layout">
                <div class="intelligence-suggestion-stack" role="list" aria-label="Governed AI suggestions">
                    {items.into_iter().enumerate().map(|(index, item)| {
                        let route = match index {
                            0 => "workflow",
                            1 => "mail",
                            _ => "community",
                        };
                        let receipt = match index {
                            0 => "AI-WF-217",
                            1 => "AI-TAX-118",
                            _ => "AI-HR-053",
                        };
                        view! {
                            <article class="intelligence-suggestion-card" data-intelligence-card="true" data-intelligence-route=route role="listitem">
                                <div><span class="status-chip ai">{receipt}</span><strong>{item.title}</strong></div>
                                <p>{item.body}</p>
                                <small>{item.guardrail}</small>
                                <div class="intelligence-card-actions">
                                    <button type="button" data-intelligence-action="preview" data-intelligence-route=route>"Preview"</button>
                                    <button type="button" data-intelligence-action="route" data-intelligence-route=route>"Route"</button>
                                    <button type="button" data-intelligence-action="dismiss">"Dismiss"</button>
                                </div>
                            </article>
                        }
                    }).collect_view()}
                </div>

                <aside class="intelligence-eval-panel" aria-label="AI guardrail evaluation harness">
                    <div><p class="screen-anchor">"EVAL HARNESS"</p><strong>"Before any tenant action"</strong><span>"Policy, data-class, autonomy ceiling, residency, and evidence checks must pass."</span></div>
                    <ol>
                        <li><span class="status-chip success">"pass"</span><strong>"No backend execution"</strong><em>"T1 advisory only"</em></li>
                        <li><span class="status-chip success">"pass"</span><strong>"Human approval required"</strong><em>"CFO / reviewer gate"</em></li>
                        <li><span class="status-chip warning">"review"</span><strong>"Tenant workload impact"</strong><em>"FD-001 cloud dogfood"</em></li>
                        <li><span class="status-chip success">"sealed"</span><strong>"Evidence receipt ready"</strong><em>"REC-AI-GUARD-009"</em></li>
                    </ol>
                    <div class="intelligence-route-grid" aria-label="Recommendation routes">
                        <button type="button" data-intelligence-action="route-workflow">"Workflow"</button>
                        <button type="button" data-intelligence-action="route-mail">"Mail"</button>
                        <button type="button" data-intelligence-action="route-community">"Community"</button>
                        <button type="button" data-intelligence-action="route-evidence">"Evidence"</button>
                    </div>
                </aside>
            </div>
        </div>
    }
}

fn selected_workflow_node<'a>(
    nodes: &'a [WorkflowNode],
    selected_node_id: &str,
) -> Option<&'a WorkflowNode> {
    nodes.iter().find(|node| node.id == selected_node_id)
}

fn context_icon(context: OperatorContext) -> &'static str {
    match context {
        OperatorContext::TenantAdmin => "◇",
        OperatorContext::CorporateOffice => "▣",
        OperatorContext::HealthcareClinician => "✚",
    }
}

#[cfg(any(feature = "ssr", test))]
pub fn render_envelope_json(context_id: &str) -> Option<String> {
    OperatorContext::from_id(context_id)
        .and_then(|context| serde_json::to_string(&server_derived_envelope(context)).ok())
}

#[cfg(any(feature = "ssr", test))]
pub fn static_dashboard_html() -> String {
    let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
    format!(
        r##"<div class="oya-console-app">
  <a class="skip-link" href="#console-shell">Skip to dashboard</a>
  {rail}
  {header}
  <main id="console-shell" class="control-center" aria-labelledby="console-title" aria-describedby="console-notice">
    {hero}
    <div id="oya-dashboard-island-root" class="dashboard-island" data-island="render-envelope-dashboard">
      {dashboard}
    </div>
  </main>
  {utility_panels}
  {side_peek}
  {command_palette}
</div>"##,
        rail = static_rail_html(),
        header = static_header_html(),
        hero = static_hero_html(),
        dashboard = static_dashboard_content(&envelope),
        utility_panels = static_utility_panels_html(),
        side_peek = static_side_peek_html(),
        command_palette = static_command_palette_html(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_rail_html() -> String {
    r##"<aside class="app-rail" aria-label="Product navigation">
    <div class="rail-brand"><span class="rail-mark" aria-hidden="true">O</span><div><strong>Oyatie</strong><span>Control Center</span></div><code>v0.1</code></div>
    <section class="rail-proof-card" aria-label="FD-001 and Oyatie Cloud shell proof">
      <p>FD-001 TENANT WORKLOADS</p>
      <strong>Service graph on Oyatie Cloud</strong>
      <span>Messenger · Mail · Community dogfood the substrate.</span>
      <small data-rail-status="true">REC-WF-7741 · cell-us-east-2 · local visual routes</small>
      <div class="rail-proof-actions" aria-label="Persistent shell proof routes">
        <button type="button" class="is-selected" data-rail-proof-action="service-graph">Service graph</button>
        <button type="button" data-rail-proof-action="cloud">Cloud</button>
        <button type="button" data-rail-proof-action="evidence">Evidence</button>
        <button type="button" data-rail-proof-action="work-hub">Work hub</button>
      </div>
      <div class="rail-comms-switcher" aria-label="Built-in Work Hub surface routes">
        <button type="button" class="is-selected" data-rail-comms-surface="Messenger">Messenger</button>
        <button type="button" data-rail-comms-surface="Mail">Mail</button>
        <button type="button" data-rail-comms-surface="Community">Community</button>
      </div>
    </section>
    <p class="rail-group">Run the company</p>
    <a class="rail-nav active" href="#console-shell"><span aria-hidden="true">⌂</span>Command center</a>
    <a class="rail-nav" href="#command-center-workbench"><span aria-hidden="true">▥</span>Action Inbox<em>8</em></a>
    <a class="rail-nav" href="#governance-analytics"><span aria-hidden="true">↟</span>Governance analytics</a>
    <p class="rail-group">Operate</p>
    <a class="rail-nav" href="#business-logics"><span aria-hidden="true">⌬</span>Business Logics<em>17</em></a>
    <a class="rail-nav" href="#tasks-title"><span aria-hidden="true">☑</span>Tasks<em>73</em></a>
    <a class="rail-nav" href="#schedule-title"><span aria-hidden="true">◷</span>Schedule</a>
    <a class="rail-nav" href="#workflow-studio"><span aria-hidden="true">⌘</span>Workflow Studio</a>
    <a class="rail-nav" href="#work-hub"><span aria-hidden="true">✉</span>Messenger · Mail · Community<em>18</em></a>
    <a class="rail-nav" href="#cloud-ops-cockpit"><span aria-hidden="true">◫</span>Cloud Ops</a>
    <a class="rail-nav" href="#support-advisory-console"><span aria-hidden="true">?</span>Support</a>
    <p class="rail-group">Money</p>
    <a class="rail-nav" href="#payroll-cockpit"><span aria-hidden="true">₩</span>Payroll</a>
    <a class="rail-nav" href="#ledger-preview"><span aria-hidden="true">▤</span>Ledger</a>
    <a class="rail-nav" href="#vendors-spend"><span aria-hidden="true">◇</span>Vendors &amp; spend</a>
    <a class="rail-nav" href="#billing-tax"><span aria-hidden="true">▧</span>Billing &amp; tax</a>
    <a class="rail-nav" href="#finops-pane"><span aria-hidden="true">₩</span>FinOps</a>
    <p class="rail-group">Compliance</p>
    <a class="rail-nav" href="#filing-readiness"><span aria-hidden="true">□</span>Filing readiness<em>2</em></a>
    <a class="rail-nav" href="#audit-ledger"><span aria-hidden="true">◱</span>Audit ledger</a>
    <a class="rail-nav" href="#policy-access"><span aria-hidden="true">⚿</span>Policy &amp; access</a>
    <p class="rail-group">People</p>
            <a class="rail-nav" href="#identity-employees"><span aria-hidden="true">◎</span>Employees</a>
            <a class="rail-nav" href="#leave-time"><span aria-hidden="true">◫</span>Leave &amp; time</a>
            <a class="rail-nav" href="#identity-workforce-service"><span aria-hidden="true">⚿</span>Auth · Org</a>
            <p class="rail-group">Trust</p>
    <a class="rail-nav" href="#trust-center-portal"><span aria-hidden="true">◈</span>Trust Center<em>new</em></a>
    <a class="rail-nav" href="#resource-inventory"><span aria-hidden="true">▤</span>Resource inventory</a>
    <a class="rail-nav" href="#modules-title"><span aria-hidden="true">▦</span>Service catalog</a>
    <a class="rail-nav" href="#evidence-spine"><span aria-hidden="true">▥</span>Evidence spine</a>
    <a class="rail-nav" href="#deployment-gates"><span aria-hidden="true">✓</span>Deployment gates</a>
    <a class="rail-nav" href="#ontology-title"><span aria-hidden="true">◎</span>Object graph</a>
    <a class="rail-nav" href="#intelligence-title"><span aria-hidden="true">✦</span>Copilot rail</a>
    <div class="workspace-switch"><span class="workspace-avatar" aria-hidden="true">N</span><div><strong>Northwind</strong><span>Enterprise · US/EU/KR</span></div></div>
  </aside>"##
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_header_html() -> String {
    r#"<header class="app-header" role="banner">
    <div class="top-breadcrumb" aria-label="Breadcrumb"><span>Oyatie Cloud</span><span class="sep">/</span><span>Operations</span><span class="sep">/</span><strong>Control Center</strong></div>
    <div class="header-route-strip" aria-label="FD-001 and Oyatie Cloud quick routes"><button type="button" class="is-selected" data-header-route="fd001"><span>FD-001</span>Service graph</button><button type="button" data-header-route="cloud"><span>Cloud</span>Substrate</button><button type="button" data-header-route="work-hub"><span>Comms</span>Work hub</button><div class="header-comms-switcher" aria-label="Built-in communications quick routes"><button type="button" class="is-selected" data-header-comms-surface="Messenger">Messenger</button><button type="button" data-header-comms-surface="Mail">Mail</button><button type="button" data-header-comms-surface="Community">Community</button></div><button type="button" data-header-route="evidence"><span>Audit</span>Evidence</button><small data-header-route-status="true">REC-WF-7741 · local quick routes</small></div>
    <button class="command-trigger" type="button" data-command-trigger="true" aria-haspopup="dialog"><span aria-hidden="true">⌕</span><span>Search actions, objects, workflows</span><kbd>⌘K</kbd></button>
    <div class="header-actions" aria-label="Shell render status"><button type="button" class="header-status">SSR shell</button><button type="button" class="header-status muted">Selective WASM islands</button><button type="button" class="header-icon" data-header-action="notifications" aria-label="Open notifications">◔<span class="header-badge" data-activity-badge="true">3</span></button><button type="button" class="header-icon" data-header-action="settings" aria-label="Open settings">⚙</button></div>
  </header>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_hero_html() -> String {
    r#"<section class="hero-panel" aria-labelledby="console-title"><div class="hero-main"><div class="page-title-copy"><p class="screen-anchor">01 / Command Center</p><div class="hero-title-row"><h1 id="console-title">Operations · 2026 May, week 19</h1><span class="hero-lens-chip">● Lens: tenant admin · Finance · 1,000 ppl</span></div><p id="console-notice" class="scope-notice" role="note">이번 주 운영 현황 — 마감, 신고, 인적자원, 결재 대기 <span>This week — close, filings, people, approvals.</span></p></div><section class="hero-close-strip" aria-label="FD-001 close command proof"><div><p class="screen-anchor">FD-001 CLOSE COMMAND</p><strong>April close proves the product workload on Oyatie Cloud</strong><span data-hero-status="true">Ready · REC-CLOSE-2026-04 · cell-us-east-2 · local command only</span></div><div class="hero-close-actions" aria-label="Close package routes"><button type="button" data-hero-action="close-april">Stage close</button><button type="button" data-hero-action="route-ledger">Ledger</button><button type="button" data-hero-action="route-cloud">Cloud proof</button><button type="button" data-hero-action="route-evidence">Evidence</button></div></section><section class="render-architecture-strip" aria-label="SSR shell and selective WASM hydration model"><article class="selected" data-render-arch-card="ssr"><p class="screen-anchor">SSR SHELL</p><strong>Fast baseline, service graph visible first</strong><span>Navigation, proof copy, tenant posture, and core dashboards render before island hydration.</span><button type="button" class="is-selected" data-render-arch-action="ssr">Show shell</button></article><article data-render-arch-card="islands"><p class="screen-anchor">SELECTIVE WASM</p><strong>Only interactive product surfaces hydrate</strong><span>Workflow Studio, Work Hub, filters, canvas state, and local drafts become browser-only islands.</span><button type="button" data-render-arch-action="islands">Show islands</button></article><article data-render-arch-card="boundary"><p class="screen-anchor">LOCAL BOUNDARY</p><strong>Visually functional, deliberately unwired</strong><span data-render-arch-status="true">No workflow execution, external send, IAM, billing, deploy, or cloud mutation.</span><button type="button" data-render-arch-action="boundary">Show evidence</button></article></section></div><div class="hero-side"><div class="hero-copy page-actions"><button type="button" data-sidepeek-trigger="new-action" data-sidepeek-title="Create governed action" data-sidepeek-id="ACT-LOCAL-DRAFT" data-sidepeek-desc="Local visual-only action draft. Nothing is persisted or sent." data-sidepeek-owner="Current operator session" data-sidepeek-risk="Draft" data-sidepeek-sla="No live SLA">New action</button><button type="button" data-command-trigger="true">Search ⌘K</button><button type="button" class="primary" data-hero-action="close-april">Close April →</button></div></div></section>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_command_palette_html() -> String {
    r#"<div class="command-palette-backdrop" data-command-backdrop hidden>
    <section class="command-palette" role="dialog" aria-modal="true" aria-label="Command palette">
      <div class="command-input-row"><span aria-hidden="true">⌕</span><input aria-label="Search command palette" placeholder="Search actions, objects, workflows…" value="" /><kbd>ESC</kbd></div>
      <section class="command-proof-strip" aria-label="FD-001 command palette proof">
        <article><p class="screen-anchor">COMMAND GRAPH</p><strong>One launcher for FD-001 tenant workloads on Oyatie Cloud</strong><span data-command-status="true">15 commands · REC-WF-7741 · cell-us-east-2 · local visual routes only</span></article>
        <div class="command-proof-grid" aria-label="Command proof shortcuts"><button type="button" data-command-proof-action="fd001"><span>FD-001</span><strong>Service graph</strong></button><button type="button" data-command-proof-action="cloud"><span>Cloud</span><strong>Substrate cells</strong></button><button type="button" data-command-proof-action="receipt"><span>Receipt</span><strong>REC-WF-7741</strong></button></div>
      </section>
      <div class="command-results" role="listbox" aria-label="Local command routes">
        <button type="button" data-command-action="workflow"><strong>Open Workflow Studio</strong><span>Build governed no-code flows for FD-001 work</span><kbd>W</kbd></button>
        <button type="button" data-command-action="mail"><strong>Compose mail</strong><span>Draft formal work messages locally</span><kbd>M</kbd></button>
        <button type="button" data-command-action="community"><strong>Post to community</strong><span>Coordinate role-aware spaces</span><kbd>C</kbd></button>
        <button type="button" data-command-action="peek"><strong>Inspect audit chain</strong><span>Open object graph and evidence spine</span><kbd>A</kbd></button>
        <button type="button" data-command-action="business-logics"><strong>Open Business Logic OS</strong><span>Inspect cost, health, owners, dependencies, and workflow routes</span><kbd>B</kbd></button>
        <button type="button" data-command-action="topology"><strong>Open cloud topology</strong><span>Inspect Oyatie Cloud tenant runtime cells and services</span><kbd>T</kbd></button>
        <button type="button" data-command-action="policy"><strong>Review policy access</strong><span>Open role envelope matrix</span><kbd>P</kbd></button>
        <button type="button" data-command-action="inventory"><strong>Open resource inventory</strong><span>Inspect ownership, cost, and risk</span><kbd>R</kbd></button>
        <button type="button" data-command-action="audit"><strong>Open audit ledger</strong><span>Review staged immutable receipts</span><kbd>L</kbd></button>
        <button type="button" data-command-action="gates"><strong>Review deployment gates</strong><span>Check Jenkins, ArgoCD, cosign, and audit evidence</span><kbd>G</kbd></button>
        <button type="button" data-command-action="catalog"><strong>Open Service Catalog</strong><span>Inspect service graph, routes, owners, and module access</span><kbd>⌘S</kbd></button>
        <button type="button" data-command-action="identity"><strong>Open Identity &amp; Workforce</strong><span>Manage auth, org profile, roles, employees, onboarding</span><kbd>I</kbd></button>
        <button type="button" data-command-action="finance"><strong>Open Finance Control</strong><span>Inspect ledger, vendors, billing, tax, leave and time</span><kbd>F</kbd></button>
        <button type="button" data-command-action="notifications"><strong>Open Activity Center</strong><span>Review notifications, approvals, and local events</span><kbd>N</kbd></button>
        <button type="button" data-command-action="settings"><strong>Open Workspace Settings</strong><span>Profile, density, integrations, and audit preferences</span><kbd>S</kbd></button>
      </div>
      <div class="command-palette-footer"><span>Local-only: commands route the visual shell without backend, workflow, mail, IAM, billing, deploy, or cloud mutation.</span><button type="button" data-command-proof-action="local-boundary">Show boundary</button></div>
    </section>
  </div>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_utility_panels_html() -> String {
    r#"<div class="utility-panel-backdrop" data-utility-backdrop hidden></div><section class="utility-panel activity-center" data-utility-panel="notifications" aria-label="Notification and activity center" aria-hidden="true">
    <div class="utility-panel-head"><div><p class="screen-anchor">ACTIVITY CENTER</p><h2>Notifications, approvals, and local events</h2></div><button type="button" data-utility-close="true" aria-label="Close activity center">×</button></div><section class="utility-proof-strip" aria-label="Activity center FD-001 substrate proof"><article><p class="screen-anchor">FD-001 OPERATIONS SIGNALS</p><strong>Notifications are workload control signals, not inbox noise</strong><span data-activity-status="true">Close, filing, vendor, and audit events are Oyatie Cloud tenant workload previews.</span></article><div class="utility-route-grid" aria-label="Activity center routes"><button type="button" data-utility-route="work-hub"><span>Comms</span><strong>Work Hub</strong></button><button type="button" data-utility-route="evidence"><span>Receipt</span><strong>Evidence spine</strong></button><button type="button" data-utility-route="cloud"><span>Substrate</span><strong>Cloud cells</strong></button></div></section>
    <div class="utility-summary"><span><strong data-activity-count="true">3</strong><small>unread</small></span><span><strong>12</strong><small>today</small></span><span><strong>3</strong><small>blocking</small></span></div>
    <div class="utility-filter-row" role="toolbar" aria-label="Activity filters"><button type="button" class="active" data-activity-filter="all">All</button><button type="button" data-activity-filter="unread">Unread</button><button type="button" data-activity-filter="blocking">Blocking</button><button type="button" data-activity-action="clear-read">Clear read</button></div>
    <ol class="activity-list" data-activity-list="true" aria-live="polite"><li data-activity-item="true" data-activity-state="unread" data-activity-severity="blocking"><time>09:18</time><span class="status-chip danger">blocking</span><strong>4대보험 변동 확인 필요</strong><p>Payroll close cannot seal until Park Seo-jun's insurance delta is reviewed.</p><button type="button" data-activity-action="mark-read">Mark read</button></li><li data-activity-item="true" data-activity-state="unread" data-activity-severity="review"><time>09:42</time><span class="status-chip warning">review</span><strong>Withholding tax brief ready</strong><p>HomeTax transport is staged locally; reviewer must approve before send.</p><button type="button" data-activity-action="mark-read">Mark read</button></li><li data-activity-item="true" data-activity-state="unread" data-activity-severity="blocking"><time>10:05</time><span class="status-chip danger">vendor</span><strong>Stripe renewal needs owner</strong><p>Spend approval exceeds one-step threshold and requires CFO attestation.</p><button type="button" data-activity-action="mark-read">Mark read</button></li><li data-activity-item="true" data-activity-state="read" data-activity-severity="info"><time>10:21</time><span class="status-chip success">sealed</span><strong>Audit receipt staged</strong><p>REC-FIN-2026-05 was added to the local close package preview.</p><button type="button" data-activity-action="open-audit">Open audit</button></li></ol>
  </section>
  <section class="utility-panel settings-center" data-utility-panel="settings" aria-label="Workspace settings" aria-hidden="true">
    <div class="utility-panel-head"><div><p class="screen-anchor">SETTINGS</p><h2>Workspace, profile, appearance, and integrations</h2></div><button type="button" data-utility-close="true" aria-label="Close settings">×</button></div><section class="utility-proof-strip settings-proof" aria-label="Settings FD-001 substrate proof"><article><p class="screen-anchor">CONTROL PLANE SETTINGS</p><strong>Workspace preferences stay tied to FD-001, policy, and Oyatie Cloud posture</strong><span>Every preference is local visual state; no auth, IAM, billing, integration, mail, or cloud mutation occurs.</span></article><div class="utility-route-grid" aria-label="Settings connected routes"><button type="button" data-utility-route="identity"><span>Identity</span><strong>Role envelope</strong></button><button type="button" data-utility-route="policy"><span>Policy</span><strong>Access matrix</strong></button><button type="button" data-utility-route="catalog"><span>Catalog</span><strong>Tenant modules</strong></button></div></section>
    <div class="settings-person-card"><span class="workspace-avatar" aria-hidden="true">최</span><div><strong>최유나 · Choi Yu-na</strong><p>Tenant admin · Finance owner · PIPA-safe transitional data</p></div></div>
    <div class="settings-tabs" role="tablist" aria-label="Settings panels"><button type="button" class="active" data-settings-tab="profile" role="tab" aria-selected="true">Profile</button><button type="button" data-settings-tab="appearance" role="tab" aria-selected="false">Appearance</button><button type="button" data-settings-tab="integrations" role="tab" aria-selected="false">Integrations</button><button type="button" data-settings-tab="audit" role="tab" aria-selected="false">Audit</button></div>
    <article class="settings-panel active" data-settings-panel="profile"><dl class="settings-kv"><div><dt>Workspace</dt><dd>Oyatie Corp. · 118 employees</dd></div><div><dt>Role</dt><dd>Admin · payroll close approver</dd></div><div><dt>Region pack</dt><dd>US/EU/KR · Korean payroll enabled</dd></div></dl><button type="button" data-settings-action="open-identity">Open identity profile</button></article>
    <article class="settings-panel" data-settings-panel="appearance"><p>Adjust local visual density and shell language without changing server state.</p><div class="settings-action-grid"><button type="button" data-settings-action="density-comfortable">Comfortable</button><button type="button" data-settings-action="density-compact">Compact</button><button type="button" data-settings-action="locale-ko">한국어 우선</button><button type="button" data-settings-action="locale-en">English labels</button></div></article>
    <article class="settings-panel" data-settings-panel="integrations"><ol class="integration-list"><li><strong>Shinhan Bank</strong><span class="status-chip success">verified</span><small>Bank transport staged locally; no money movement.</small></li><li><strong>HomeTax</strong><span class="status-chip warning">review</span><small>Filing transport waits for human attestation.</small></li><li><strong>Google Workspace</strong><span class="status-chip">local</span><small>Mail and community previews only.</small></li></ol></article>
    <article class="settings-panel" data-settings-panel="audit"><ol class="activity-list compact"><li><time>09:14</time><strong>Settings drawer opened</strong><p>Local shell state only.</p></li><li><time>09:18</time><strong>Density preference staged</strong><p>Stored in this browser session.</p></li><li><time>09:42</time><strong>Identity panel linked</strong><p>No auth mutation.</p></li></ol></article>
    <p class="settings-status" data-settings-status="true">Local settings ready · no backend persistence.</p>
  </section>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_side_peek_html() -> String {
    r#"<aside class="side-peek" data-side-peek="true" aria-label="Object quick view" aria-hidden="true">
    <div class="side-peek-head"><div><p class="screen-anchor">OBJECT QUICK VIEW</p><h2 data-sidepeek-title-target="true">Network hot split</h2></div><button type="button" data-sidepeek-close="true" aria-label="Close object quick view">×</button></div>
    <div class="side-peek-body">
      <section class="quick-identity"><span class="workspace-avatar" aria-hidden="true">N</span><div><strong data-sidepeek-id-target="true">CHG-NTW-4182</strong><p data-sidepeek-desc-target="true">Tenant network split awaiting residency and rollback evidence.</p></div></section>
      <dl class="peek-kv"><div><dt>Owner</dt><dd data-sidepeek-owner-target="true">Infrastructure operations</dd></div><div><dt>Risk</dt><dd><span class="status-chip danger" data-sidepeek-risk-target="true">High</span></dd></div><div><dt>SLA</dt><dd data-sidepeek-sla-target="true">4.0h target · +1.4h over</dd></div><div><dt>Execution</dt><dd>Visual-only until live integration</dd></div></dl>
      <section class="side-peek-proof" aria-label="FD-001 object proof"><p class="screen-anchor">OBJECT PROOF</p><strong>Selected objects resolve to FD-001 workload evidence on Oyatie Cloud</strong><span data-sidepeek-status="true">Inspector ready · REC-WF-7741 · cell-us-east-2 · local visual state only.</span><div class="side-peek-route-grid" aria-label="Object proof routes"><button type="button" data-sidepeek-route="workload"><span>Workload</span><strong>FD-001 graph</strong></button><button type="button" data-sidepeek-route="cloud"><span>Cloud</span><strong>cell-us-east-2</strong></button><button type="button" data-sidepeek-route="evidence"><span>Receipt</span><strong>REC-WF-7741</strong></button></div></section>
      <section><h3>Evidence trail</h3><ol class="peek-timeline"><li><time>09:18</time><span>Policy guardrail matched residency rule for FD-001 tenant workload.</span></li><li><time>09:42</time><span>Oyatie Cloud rollback plan requested from network owner.</span></li><li><time>10:05</time><span>Audit-chain receipt REC-WF-7741 drafted locally.</span></li></ol></section>
      <section class="peek-actions" aria-label="Object actions"><button type="button" data-sidepeek-action="assign-owner">Assign owner</button><button type="button" data-sidepeek-action="draft-note">Draft note</button><button type="button" class="primary" data-sidepeek-action="review-evidence">Review evidence</button></section>
    </div>
  </aside>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_daily_execution_proof_board() -> &'static str {
    r#"<section class="daily-proof-board" aria-label="FD-001 daily execution and Oyatie Cloud tenant proof"><div class="daily-proof-grid"><article class="daily-proof-card selected" data-daily-proof-card="daily-fd001"><p class="screen-anchor">FD-001 DAILY WORKLOAD</p><h5>Today’s work proves product delivery</h5><p>Tasks, approvals, schedule holds, Workflow routes, Mail, Messenger, Community, and evidence receipts are FD-001 tenant workload operations, not detached widgets.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="stage-packet">Stage work packet</button><button type="button" data-daily-proof-action="route-workflow">Workflow run</button></div></article><article class="daily-proof-card" data-daily-proof-card="daily-cloud"><p class="screen-anchor">OYATIE CLOUD SUBSTRATE</p><h5>The queue dogfoods tenant hosting posture</h5><p>Oyatie Cloud proves the substrate can host real production tenants by tying daily FD-001 work to cell health, policy envelopes, FinOps, and audit freshness.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="route-cloud">Cloud cells</button><button type="button" data-daily-proof-action="route-policy">Policy envelope</button></div></article><article class="daily-proof-card" data-daily-proof-card="daily-local"><p class="screen-anchor">LOCAL-ONLY COMMAND RAIL</p><h5>Visually functional without side effects</h5><p>Operators can filter, stage, route, brief, and inspect work while backend writes, auth changes, workflow execution, mail sends, payroll, billing, and cloud mutations remain disconnected.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="route-audit">Audit ledger</button><button type="button" data-daily-proof-action="route-mail">Mail brief</button></div></article></div><div class="daily-proof-footer"><span data-daily-proof-status="true">Daily execution ready · FD-001 work queue dogfoods Oyatie Cloud locally.</span><div class="daily-proof-routes" aria-label="Daily execution connected routes"><button type="button" data-daily-proof-action="route-inbox">Action Inbox</button><button type="button" data-daily-proof-action="route-schedule">Schedule</button><button type="button" data-daily-proof-action="route-community">Community</button><button type="button" data-daily-proof-action="route-evidence">Evidence</button></div></div></section>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_daily_queue_footer() -> &'static str {
    r#"<div class="daily-subroute-proof"><span data-daily-proof-status="true">Execution queue ready · FD-001 work items dogfood Oyatie Cloud with no backend mutation.</span><div class="daily-proof-routes" aria-label="Execution queue connected routes"><button type="button" data-daily-proof-action="route-inbox">Action Inbox</button><button type="button" data-daily-proof-action="route-workflow">Workflow</button><button type="button" data-daily-proof-action="route-evidence">Evidence</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_daily_schedule_footer() -> &'static str {
    r#"<div class="daily-subroute-proof"><span data-daily-proof-status="true">Schedule pressure ready · FD-001 calendar risk stays tenant-scoped on Oyatie Cloud with no calendar, workflow, mail, policy, or cloud mutation.</span><div class="daily-proof-routes" aria-label="Schedule connected routes"><button type="button" data-daily-proof-action="route-policy">Policy</button><button type="button" data-daily-proof-action="route-cloud">Cloud cells</button><button type="button" data-daily-proof-action="route-mail">Reviewer Mail</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_daily_execution_console(envelope: &TenantRenderEnvelope) -> String {
    let rows = daily_execution_rows(envelope);
    let blocking_count = rows.iter().filter(|row| row.state == "blocking").count();
    let task_count = rows.iter().filter(|row| row.kind == "task").count();
    let approval_count = rows.iter().filter(|row| row.kind == "approval").count();
    let schedule_count = rows.iter().filter(|row| row.kind == "schedule").count();
    let evidence_count = rows.iter().filter(|row| row.kind == "evidence").count();
    let row_markup = rows.iter().map(static_daily_row).collect::<String>();
    let schedule_markup = envelope
        .schedule
        .iter()
        .map(|item| {
            format!(
                r#"<li><time>{time}</time><strong>{title}</strong><p>{detail}</p></li>"#,
                time = escape(&item.time),
                title = escape(&item.title),
                detail = escape(&item.detail)
            )
        })
        .collect::<String>();

    format!(
        r##"<section id="daily-execution" class="daily-execution-console panel" aria-labelledby="daily-execution-title"><div class="daily-execution-head"><div><p class="screen-anchor">DAILY WORK · PERSONAL OPERATIONS</p><h3 id="daily-execution-title">Tasks, approvals, schedule, and evidence for today</h3><p>A single operator queue connects calendar pressure, approval risk, workflow routes, Mail/Messenger drafts, and receipt evidence.</p></div><span class="status-chip success">local command surface</span></div><div class="daily-execution-kpis" aria-label="Daily execution summary"><span><strong>{task_count}</strong><small>tasks</small></span><span><strong>{approval_count}</strong><small>approvals</small></span><span><strong>{schedule_count}</strong><small>calendar holds</small></span><span><strong>{blocking_count}</strong><small>blocking</small></span><span><strong>{evidence_count}</strong><small>evidence links</small></span></div><div class="daily-execution-toolbar" aria-label="Daily execution filters"><label><span aria-hidden="true">⌕</span><input data-daily-search="true" aria-label="Search daily work" placeholder="Search tasks, approvals, owners, receipts..." /></label><div class="daily-filter-pills" role="toolbar" aria-label="Daily work filters"><button type="button" class="active" data-daily-filter="all">All</button><button type="button" data-daily-filter="blocking">Blocking</button><button type="button" data-daily-filter="task">Tasks</button><button type="button" data-daily-filter="approval">Approvals</button><button type="button" data-daily-filter="schedule">Schedule</button><button type="button" data-daily-filter="evidence">Evidence</button></div><span data-daily-status="true">{row_count} visible · all work · local only</span></div>{daily_execution_proof_board}<div class="daily-execution-layout"><article id="tasks-title" class="daily-execution-list" aria-labelledby="daily-list-title"><div class="daily-column-head"><p class="screen-anchor">EXECUTION QUEUE</p><h4 id="daily-list-title">One list for personal operations</h4></div><div role="list" aria-label="Daily work rows">{row_markup}</div>{daily_queue_footer}</article><aside id="schedule-title" class="daily-calendar-rail" aria-label="Today schedule and capacity"><div class="daily-column-head"><p class="screen-anchor">CALENDAR</p><h4>Today’s schedule pressure</h4></div><ol class="daily-timeline">{schedule_markup}</ol><div class="daily-capacity" aria-label="Daily capacity"><span style="--bar: 73%"><em>Close work · 73%</em></span><span style="--bar: 64%"><em>Approvals · 64%</em></span><span style="--bar: 41%"><em>Context switching · 41%</em></span></div><div class="daily-route-matrix" aria-label="Daily route matrix"><button type="button" data-daily-action="workflow" data-daily-target="#workflow-studio">Workflow</button><button type="button" data-daily-action="mail">Mail brief</button><button type="button" data-daily-action="messenger">Messenger</button><button type="button" data-daily-action="evidence">Audit evidence</button></div>{daily_schedule_footer}</aside></div></section>"##,
        task_count = task_count,
        approval_count = approval_count,
        schedule_count = schedule_count,
        blocking_count = blocking_count,
        evidence_count = evidence_count,
        row_count = rows.len(),
        row_markup = row_markup,
        schedule_markup = schedule_markup,
        daily_execution_proof_board = static_daily_execution_proof_board(),
        daily_queue_footer = static_daily_queue_footer(),
        daily_schedule_footer = static_daily_schedule_footer()
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_daily_row(row: &ExecutionRow) -> String {
    format!(
        r##"<article class="daily-row" data-daily-row="true" data-daily-kind="{kind}" data-daily-state="{state}" role="listitem"><button type="button" class="daily-row-main" data-sidepeek-trigger="daily-work" data-sidepeek-title="{title}" data-sidepeek-id="{id}" data-sidepeek-desc="{body}" data-sidepeek-owner="{owner}" data-sidepeek-risk="{state}" data-sidepeek-sla="{due}"><span class="{chip}">{state}</span><strong>{title}</strong><p>{body}</p></button><dl><div><dt>Kind</dt><dd>{kind}</dd></div><div><dt>Owner</dt><dd>{owner}</dd></div><div><dt>Due</dt><dd>{due}</dd></div></dl><div class="daily-row-actions"><button type="button" data-daily-action="workflow" data-daily-target="{route}">Flow</button><button type="button" data-daily-action="mail">Mail</button><button type="button" data-daily-action="evidence">Evidence</button><button type="button" data-daily-action="stage">Stage</button></div></article>"##,
        kind = escape(row.kind),
        state = escape(row.state),
        title = escape(&row.title),
        id = escape(&row.id),
        body = escape(&row.body),
        owner = escape(&row.owner),
        due = escape(&row.due),
        chip = daily_status_class(row.state),
        route = escape(row.route)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_dashboard_content(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"{context_switcher}
{surface_commands}
{product_activity}
{envelope_banner}
<section class="metric-grid" aria-label="Dashboard metrics">{metrics}</section>
{command_shell}
{substrate_proof}
{command_workbench}
{tenant_rbac}
{identity_service}
{finance_service}
{operator_intelligence}
{operations_cockpit}
{control_plane_fixture}
{trust_center}
{support_advisory}
{audit_evidence_timeline}
{resource_audit}
<section class="dashboard-grid" aria-label="Personalized dashboard">{daily_execution}<section id="work-hub" class="panel communications-panel"><div class="panel-header"><p class="eyebrow">Messenger · Mail · Community</p><h3>Work hub</h3></div>{communication_hub}</section>{service_catalog}</section>
<section class="studio-grid" aria-label="Workflow, ontology, and intelligence">{workflow_studio}<section id="ontology-command-console" class="panel ontology-command-shell"><div class="panel-header"><p class="eyebrow">Ontology</p><h3 id="ontology-title">Tenant workload graph</h3></div>{ontology}</section><section id="intelligence-command-console" class="panel intelligence-command-shell"><div class="panel-header"><p class="eyebrow">Intelligence</p><h3 id="intelligence-title">Governed AI command</h3></div>{suggestions}</section></section>"#,
        context_switcher = static_context_switcher(envelope.context),
        surface_commands = static_surface_commands(),
        product_activity = static_product_activity_spine(&envelope.product_activity),
        envelope_banner = static_envelope_banner(envelope),
        metrics = envelope
            .metrics
            .iter()
            .map(static_metric)
            .collect::<String>(),
        command_shell = static_command_shell_substrate(),
        substrate_proof = static_substrate_proof_command(envelope),
        command_workbench = static_command_center_workbench(envelope),
        tenant_rbac = static_tenant_rbac_board(envelope),
        identity_service = static_identity_workforce_service(),
        finance_service = static_finance_commercial_service(),
        operator_intelligence = static_operator_intelligence_strip(envelope),
        operations_cockpit = static_tenant_operations_cockpit(envelope),
        control_plane_fixture = static_control_plane_resource_explorer_operation_center(),
        trust_center = static_trust_center_portal(),
        support_advisory = envelope
            .support_advisory
            .as_ref()
            .filter(|_| support_advisory_is_visible(envelope))
            .map(static_support_advisory_case_console)
            .unwrap_or_default(),
        audit_evidence_timeline = static_audit_evidence_timeline_panel(),
        resource_audit = static_resource_audit_console(envelope),
        daily_execution = static_daily_execution_console(envelope),
        communication_hub = static_communication_hub(&envelope.messages, &envelope.community),
        service_catalog = static_service_catalog(envelope),
        workflow_studio = static_workflow_studio_panel(envelope),
        ontology = static_ontology_command_console(&envelope.ontology),
        suggestions = static_intelligence_command_console(&envelope.intelligence),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_surface_commands() -> String {
    let commands = ProductSurface::ALL
        .iter()
        .map(|surface| {
            let class = if *surface == ProductSurface::Workflow {
                "surface-command active"
            } else {
                "surface-command"
            };
            format!(
                r#"<a class="{class}" href="{href}"><span>{label}</span><small>{summary}</small></a>"#,
                class = class,
                href = surface.href(),
                label = escape(surface.label()),
                summary = escape(surface.summary())
            )
        })
        .collect::<String>();

    format!(
        r#"<nav class="surface-command-bar" aria-label="Open built-in product surface">{commands}</nav>"#
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_command_shell_substrate() -> String {
    let routes = COMMAND_SHELL_ROUTES
        .iter()
        .map(|(route, label, detail, target)| {
            let class = if *route == "fd001" {
                " class=\"selected\""
            } else {
                ""
            };
            format!(
                r#"<button type="button"{class} data-shell-context-route="{route}" data-shell-context-target="{target}"><strong>{label}</strong><span>{detail}</span></button>"#,
                class = class,
                route = escape(route),
                target = escape(target),
                label = escape(label),
                detail = escape(detail),
            )
        })
        .collect::<String>();

    format!(
        r##"<section id="command-shell-substrate" class="command-shell-substrate panel" aria-labelledby="command-shell-title" data-command-shell-substrate="true"><div class="command-shell-copy"><p class="screen-anchor">COMMAND SHELL SUBSTRATE</p><h3 id="command-shell-title">Every lower panel inherits the same active route, tenant lens, and local boundary</h3><span data-command-shell-status="true">FD-001 graph is active · lower surfaces will keep the route/status/inspector spine synchronized.</span></div><div class="command-shell-context" aria-live="polite"><span><small>Active route</small><strong data-command-shell-route="true">FD-001 graph</strong></span><span><small>Target</small><strong data-command-shell-target="true">#service-catalog</strong></span><span><small>Updated</small><strong data-command-shell-updated="true">SSR render</strong></span></div><div class="command-shell-routes" role="toolbar" aria-label="Lower dashboard product routes">{routes}</div></section>"##,
        routes = routes,
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_product_activity_spine(spine: &ProductActivitySpine) -> String {
    let active_label = spine
        .steps
        .iter()
        .find(|step| step.route_key == spine.active_route)
        .map(|step| step.label.as_str())
        .unwrap_or(&spine.active_route);
    let route_buttons = spine
        .steps
        .iter()
        .map(|step| {
            let class = if step.route_key == spine.active_route {
                " class=\"selected\""
            } else {
                ""
            };
            format!(
                r#"<button type="button"{class} data-activity-route="{route}" data-activity-target="{target}" data-activity-label="{label}" data-activity-detail="{detail}" data-activity-state="{state}"><strong>{label}</strong><span>{surface}</span></button>"#,
                class = class,
                route = escape(&step.route_key),
                target = escape(&step.target),
                label = escape(&step.label),
                detail = escape(&step.detail),
                state = escape(&step.state),
                surface = escape(&step.surface),
            )
        })
        .collect::<String>();

    let lane_steps = spine
        .steps
        .iter()
        .map(|step| {
            let class = if step.route_key == spine.active_route {
                "activity-step-card selected"
            } else {
                "activity-step-card"
            };
            format!(
                r#"<button type="button" class="{class}" data-activity-route="{route}" data-activity-target="{target}" data-activity-label="{label}" data-activity-detail="{detail}" data-activity-state="{state}" data-spine-step="{route}"><span>{surface}</span><strong>{label}</strong><small>{detail}</small><em>{state}</em></button>"#,
                class = class,
                route = escape(&step.route_key),
                target = escape(&step.target),
                label = escape(&step.label),
                detail = escape(&step.detail),
                state = escape(&step.state),
                surface = escape(&step.surface),
            )
        })
        .collect::<String>();

    format!(
        r#"<section id="product-activity-spine" class="product-activity-spine panel" aria-labelledby="product-activity-title" data-product-activity-spine="true"><div class="activity-spine-head"><div><p class="screen-anchor">PRODUCT ACTIVITY SPINE</p><h3 id="product-activity-title">One operating model for FD-001 tenant workloads on Oyatie Cloud</h3><span data-spine-active-context="true">{context}</span></div><div class="activity-spine-proof"><span data-spine-active-route="true">{route}</span><code data-spine-evidence-id="true">{evidence}</code><strong data-global-activity-status="true">{status}</strong></div></div><div class="activity-spine-grid"><aside class="activity-route-column" aria-label="Cross-surface routes"><p class="screen-anchor">ROUTES</p>{route_buttons}</aside><div class="activity-flow-lane" aria-label="FD-001 workload path">{lane_steps}</div><aside class="activity-inspector-card" aria-label="Selected route inspector"><p class="screen-anchor">INSPECTOR</p><h4 data-spine-inspector-title="true">FD-001 graph · product substrate</h4><p data-spine-inspector-body="true">Service catalog, workflow, Messenger, Mail, Community, cloud posture, and evidence receipts are one cohesive local operating graph.</p><dl><div><dt>Tenant</dt><dd data-spine-inspector-tenant="true">{context}</dd></div><div><dt>Boundary</dt><dd>Visual-only · no backend write</dd></div><div><dt>Receipt</dt><dd>{evidence}</dd></div></dl><div class="activity-inspector-actions"><button type="button" data-activity-route="workflow">Open Workflow</button><button type="button" data-activity-route="mail">Mail brief</button><button type="button" data-activity-route="evidence">Evidence</button></div></aside></div><div class="activity-spine-statusbar" aria-label="Current local shell state"><span>SSR shell</span><span>Selective WASM islands</span><span>Local-only actions</span><span data-spine-last-action="true">Ready · route and inspector state will update visually</span></div></section>"#,
        context = escape(&spine.active_context),
        route = escape(active_label),
        evidence = escape(&spine.evidence_id),
        status = escape(&spine.status_label),
        route_buttons = route_buttons,
        lane_steps = lane_steps,
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_comms_receipt_bridge() -> &'static str {
    r#"<section class="comms-receipt-bridge" data-comms-receipt-bridge="true" aria-label="Messenger Mail Community receipt bridge"><div class="comms-bridge-head"><div><p class="screen-anchor">COMMS RECEIPT BRIDGE</p><h4>Messenger, Mail, and Community return to one proof packet</h4><span data-comms-bridge-status="true">Ops room, approval brief, council post, and audit receipt are staged as one local FD-001 workload packet.</span></div><button type="button" data-comms-bridge-action="seal">Seal handoff</button></div><div class="comms-bridge-routes" aria-label="Communication proof routes"><button type="button" class="selected" data-comms-bridge-route="messenger" data-comms-bridge-title="Messenger ops room" data-comms-bridge-receipt="REC-COMMS-MSG-021" data-comms-bridge-target="Ops room → Mail brief → Community note"><span>01 · Messenger</span><strong>Ops room thread</strong><em>REC-COMMS-MSG-021</em></button><button type="button" data-comms-bridge-route="mail" data-comms-bridge-title="Mail approval brief" data-comms-bridge-receipt="REC-COMMS-MAIL-022" data-comms-bridge-target="Formal approval → Evidence packet"><span>02 · Mail</span><strong>Approval brief</strong><em>REC-COMMS-MAIL-022</em></button><button type="button" data-comms-bridge-route="community" data-comms-bridge-title="Community council note" data-comms-bridge-receipt="REC-COMMS-COMM-023" data-comms-bridge-target="Council post → Role-visible vote"><span>03 · Community</span><strong>Governance note</strong><em>REC-COMMS-COMM-023</em></button><button type="button" data-comms-bridge-route="receipt" data-comms-bridge-title="Universal receipt packet" data-comms-bridge-receipt="REC-COMMS-HANDOFF-006" data-comms-bridge-target="Audit ledger → Receipt stitching console"><span>04 · Receipt</span><strong>Audit stitch</strong><em>REC-COMMS-HANDOFF-006</em></button></div><aside class="comms-bridge-detail" aria-label="Selected communication receipt detail"><dl><div><dt>Selected</dt><dd data-comms-bridge-detail-title="true">Messenger ops room</dd></div><div><dt>Receipt</dt><dd data-comms-bridge-detail-receipt="true">REC-COMMS-MSG-021</dd></div><div><dt>Route</dt><dd data-comms-bridge-detail-target="true">Ops room → Mail brief → Community note</dd></div></dl><div class="comms-bridge-actions" aria-label="Communication receipt bridge actions"><button type="button" data-comms-bridge-action="workflow">Workflow</button><button type="button" data-comms-bridge-action="cloud">Cloud</button><button type="button" data-comms-bridge-action="audit">Audit receipt</button><button type="button" data-comms-bridge-action="draft">Draft all</button></div></aside></section>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_comms_product_board(surface: ProductSurface) -> String {
    match surface {
        ProductSurface::Messenger => r#"<section class="comms-product-board messenger-board" data-comms-product-board="true" data-comms-board-surface="Messenger" aria-label="Messenger command workspace"><div class="comms-board-head"><div><p class="screen-anchor">MESSENGER COMMAND</p><h4>Ops room thread with FD-001 workload evidence</h4><span>Fast operational chat for dogfooding FD-001 microservices on Oyatie Cloud, with evidence links and action extraction.</span></div><div class="comms-board-actions"><span class="status-chip warning">2 unread</span><button type="button" data-comms-action="thread-escalate">Escalate</button><button type="button" data-comms-action="thread-to-mail">Promote to Mail</button><button type="button" data-comms-action="thread-receipt">Attach receipt</button></div></div><div class="comms-board-grid"><article class="thread-transcript-card"><p class="screen-anchor">LIVE THREAD</p><ol class="comms-transcript"><li><strong>Ops bot</strong><span>Kubernetes runtime tier drift detected in cell-us-east-2.</span><em>09:18 · unread</em></li><li class="mine"><strong>Tenant admin</strong><span>Link rollback runbook and notify Finance before close.</span><em>09:22 · local draft</em></li><li><strong>Security reviewer</strong><span>Need audit-chain evidence before promotion.</span><em>09:24 · evidence</em></li></ol></article><article><p class="screen-anchor">ACTION EXTRACTION</p><div class="comms-action-list"><button type="button" data-comms-action="create-task"><strong>Create task</strong><span>Rollback evidence owner · due 2.1h</span></button><button type="button" data-comms-action="link-workflow"><strong>Link workflow</strong><span>PROC-PAYROLL-CLOSE critical path</span></button><button type="button" data-comms-action="thread-to-mail"><strong>Draft formal mail</strong><span>CFO + SRE approval brief</span></button></div></article><article><p class="screen-anchor">PARTICIPANTS</p><div class="comms-presence-grid"><span><em>OP</em><strong>Ops bot</strong><small>online</small></span><span><em>SR</em><strong>Security</strong><small>watching</small></span><span><em>FL</em><strong>Finance</strong><small>mail owner</small></span><span><em>GV</em><strong>Governance</strong><small>council</small></span></div></article></div></section>"#.to_string(),
        ProductSurface::Mail => r#"<section class="comms-product-board mail-board" data-comms-product-board="true" data-comms-board-surface="Mail" aria-label="Mail command workspace"><div class="comms-board-head"><div><p class="screen-anchor">MAIL COMMAND</p><h4>Formal approval brief composer</h4><span>Structured mail draft with recipients, subject, FD-001 workload evidence attachments, Oyatie Cloud cell context, approvals, and send preview.</span></div><div class="comms-board-actions"><span class="status-chip ai">draft</span><button type="button" data-comms-action="mail-preview">Preview</button><button type="button" data-comms-action="mail-attach">Attach packet</button><button type="button" data-comms-action="send-preview">Send preview</button></div></div><div class="comms-mail-compose-grid"><article class="mail-envelope-card"><p class="screen-anchor">ENVELOPE</p><dl><div><dt>From</dt><dd>Finance lead · Oyatie</dd></div><div><dt>To</dt><dd>CFO, SRE reviewer</dd></div><div><dt>CC</dt><dd>Governance council, Audit</dd></div><div><dt>Subject</dt><dd>Approval needed: payroll close + cloud rollback evidence</dd></div></dl></article><article class="mail-body-card"><p class="screen-anchor">DRAFT BODY</p><div class="mail-paper"><strong>Please review the April close packet before 18:00.</strong><p>Payroll delta, HomeTax readiness, vendor exception, and Oyatie Cloud rollback evidence are attached as read-only receipts for the FD-001 tenant workload. No external send is enabled before live integration.</p><ol><li>REC-PAY-2026-04-PARK</li><li>REC-CLOUD-MESH-4182</li><li>REC-WF-7741</li></ol></div></article><article><p class="screen-anchor">APPROVAL CHECKS</p><div class="mail-checks"><span class="done">Human reviewer required</span><span class="done">PIPA-safe body</span><span class="review">CFO signoff pending</span><span>External delivery disabled</span></div></article></div></section>"#.to_string(),
        ProductSurface::Community => r#"<section class="comms-product-board community-board" data-comms-product-board="true" data-comms-board-surface="Community" aria-label="Community command workspace"><div class="comms-board-head"><div><p class="screen-anchor">COMMUNITY COMMAND</p><h4>Governance council publication</h4><span>Role-aware community post, voting, pinned Oyatie Cloud cell context, and moderation state for FD-001 tenant-workload coordination.</span></div><div class="comms-board-actions"><span class="status-chip success">role-aware</span><button type="button" data-comms-action="community-pin">Pin</button><button type="button" data-comms-action="community-poll">Open poll</button><button type="button" data-comms-action="publish-note">Publish local</button></div></div><div class="community-feed-grid"><article class="community-post-card"><p class="screen-anchor">PINNED POST</p><div class="community-post-preview"><span>Governance council</span><h5>April close governance digest</h5><p>Payroll blocker, withholding filing readiness, Oyatie Cloud rollback evidence, and reviewer assignments are summarized for role-visible FD-001 review.</p><div><button type="button" data-comms-action="community-upvote">▲ 24</button><button type="button" data-comms-action="community-comment">8 comments</button><button type="button" data-comms-action="community-save">Save</button></div></div></article><article><p class="screen-anchor">AUDIENCE</p><div class="community-audience-grid"><span><strong>Finance</strong><em>required</em></span><span><strong>SRE</strong><em>review</em></span><span><strong>People Ops</strong><em>visible</em></span><span><strong>Vendors</strong><em>blocked</em></span></div></article><article><p class="screen-anchor">MODERATION</p><dl class="community-moderation"><div><dt>Policy</dt><dd>PIPA-safe</dd></div><div><dt>Evidence</dt><dd>3 receipts</dd></div><div><dt>Publish</dt><dd>local only</dd></div></dl></article></div></section>"#.to_string(),
        ProductSurface::Workflow => r#"<section class="comms-product-board" data-comms-product-board="true"><p>Workflow route selected.</p></section>"#.to_string(),
    }
}

#[cfg(any(feature = "ssr", test))]
fn static_communication_hub(messages: &[MessageItem], communities: &[CommunityItem]) -> String {
    let messenger_items = hub_items(messages, communities, &[], ProductSurface::Messenger);
    let mail_items = hub_items(messages, communities, &[], ProductSurface::Mail);
    let community_items = hub_items(messages, communities, &[], ProductSurface::Community);
    let messenger = messenger_items
        .iter()
        .enumerate()
        .map(|(index, item)| static_hub_button(item, index))
        .collect::<String>();
    let mail = mail_items
        .iter()
        .enumerate()
        .map(|(index, item)| static_hub_button(item, index))
        .collect::<String>();
    let community = community_items
        .iter()
        .enumerate()
        .map(|(index, item)| static_hub_button(item, index))
        .collect::<String>();
    let selected = messenger_items.first().cloned().unwrap_or(HubItem {
        surface: ProductSurface::Messenger,
        source: "Messenger".to_string(),
        title: "Local work hub".to_string(),
        body: "Use the local island to switch channels, inspect items, and queue drafts."
            .to_string(),
        meta: "Visual-only; no backend send".to_string(),
    });
    let selected_kind = hub_item_kind(&selected, 0);
    let selected_chip = hub_item_chip_class(selected_kind);
    let messenger_board = static_comms_product_board(ProductSurface::Messenger);
    let mail_board = static_comms_product_board(ProductSurface::Mail);
    let community_board = static_comms_product_board(ProductSurface::Community);
    let comms_receipt_bridge = static_comms_receipt_bridge();

    format!(
        r#"<div class="communications-hub interactive-hub"><div class="hub-tabs" role="tablist" aria-label="Work hub channels"><button type="button" role="tab" aria-selected="true" class="hub-tab active">Messenger</button><button type="button" role="tab" aria-selected="false" class="hub-tab">Mail</button><button type="button" role="tab" aria-selected="false" class="hub-tab">Community</button></div><div class="comms-kpi-strip" aria-label="Built-in communications summary"><span><strong>18</strong><small>threads · drafts</small></span><span><strong>6</strong><small>workflow routes</small></span><span><strong>4</strong><small>evidence links</small></span><span><strong>0</strong><small>external sends</small></span></div><div class="hub-route-board" aria-label="Workflow output routes"><div><p class="screen-anchor">OUTPUT ROUTES</p><strong>FD-001 tenant-workload drafts fan out to Messenger, Mail, and Community with evidence return paths</strong><span data-comms-route-status="true">FD-001 workload dogfood · REC-WF-7741 · no backend send</span></div><button type="button" data-hub-route="Messenger">Messenger post</button><button type="button" data-hub-route="Mail">Mail draft</button><button type="button" data-hub-route="Community">Community note</button></div>{messenger_board}<section class="comms-substrate-strip" aria-label="Oyatie Cloud tenant-workload proof"><div><p class="screen-anchor">SUBSTRATE PROOF</p><strong>Messenger, Mail, and Community are dogfood tenant workloads on Oyatie Cloud</strong><span data-comms-substrate-status="true">Messenger route pinned to FD-001 workload · cell-us-east-2 · local visual proof</span></div><button type="button" data-comms-action="prove-substrate"><span>Cloud cell</span><strong>cell-us-east-2</strong></button><button type="button" data-comms-action="route-cloud"><span>Tenant workload</span><strong>FD-001 microservices</strong></button><button type="button" data-comms-action="seal-proof"><span>Evidence</span><strong>REC-WF-7741</strong></button></section>{comms_receipt_bridge}<div class="comms-service-toolbar" aria-label="Communications workspace controls"><label><span aria-hidden="true">⌕</span><input data-comms-search="true" aria-label="Search communications" placeholder="Search threads, mail, spaces..." /></label><div class="comms-filter-pills" role="toolbar" aria-label="Communication filters"><button type="button" class="active" data-comms-filter="all">All</button><button type="button" data-comms-filter="unread">Unread</button><button type="button" data-comms-filter="draft">Drafts</button><button type="button" data-comms-filter="evidence">Evidence</button></div><button type="button" data-comms-action="new-thread">New thread</button><button type="button" data-comms-action="attach-evidence">Attach evidence</button><button type="button" data-comms-action="directory">Directory</button><span data-comms-status="true">Local service workspace ready · no external send</span></div><div class="hub-workspace comms-service-shell"><aside class="comms-sidebar" aria-label="Communications folders and spaces"><p class="screen-anchor">WORKSPACES</p><button type="button" class="active" data-hub-route="Messenger"><strong>Ops room</strong><span>Messenger · 5 items · 2 unread</span></button><button type="button" data-hub-route="Mail"><strong>Finance close</strong><span>Mail · 4 drafts · 2 evidence</span></button><button type="button" data-hub-route="Community"><strong>Governance council</strong><span>Community · 5 spaces · 1 publish</span></button><button type="button" data-comms-action="notification-filter"><strong>Notifications</strong><span>6 local alerts · no external send</span></button></aside><div class="hub-list" role="list" aria-label="Channel items">{messenger}</div><div class="hub-detail"><div class="comms-message-toolbar" aria-label="Selected communication actions"><span class="status-chip success">role-visible</span><button type="button" data-comms-action="mark-reviewed">Mark reviewed</button><button type="button" data-comms-action="create-task">Create task</button><button type="button" data-comms-action="link-workflow">Link workflow</button><button type="button" data-comms-action="send-preview">Send preview</button><button type="button" data-comms-action="publish-note">Publish local</button></div><article class="comms-detail-card"><div class="comms-detail-head"><div><p class="eyebrow">{surface}</p><h4>{title}</h4></div><span class="{selected_chip}">{selected_kind}</span></div><p>{body}</p><span class="hub-meta">{meta}</span><dl class="comms-detail-grid"><div><dt>Route</dt><dd>{surface}</dd></div><div><dt>Workflow</dt><dd>Tenant change approval</dd></div><div><dt>Receipt</dt><dd>REC-WF-7741</dd></div><div><dt>Persistence</dt><dd>Local browser state only</dd></div></dl></article><div class="hub-composer"><label for="static-hub-composer">Draft a local response</label><textarea id="static-hub-composer" rows="3" placeholder="Hydration enables local queueing."></textarea><div class="composer-actions"><button type="button">Queue draft</button><button type="button" class="secondary">Clear</button></div><p>Mail previews: {mail_count}. Community spaces: {community_count}. Visual-only; no external send.</p></div></div><aside class="comms-context-rail" aria-label="People, provenance, and notification context"><section><p class="screen-anchor">PEOPLE</p><div class="presence-stack"><span><em>OP</em><strong>Ops bot</strong><small>online</small></span><span><em>SR</em><strong>Security reviewer</strong><small>watching</small></span><span><em>FL</em><strong>Finance lead</strong><small>mail owner</small></span></div></section><section><p class="screen-anchor">PROVENANCE</p><dl class="comms-kv"><div><dt>Envelope</dt><dd>tenant-admin</dd></div><div><dt>Workflow</dt><dd>Tenant change approval</dd></div><div><dt>Receipt</dt><dd>REC-WF-7741</dd></div></dl></section><section class="comms-handoff-card" data-comms-handoff="true" aria-label="Local draft handoff state"><p class="screen-anchor">DRAFT HANDOFF BUS</p><strong data-comms-handoff-title="true">Messenger → Mail approval brief</strong><span data-comms-handoff-status="true">Select Promote to Mail or Publish local to carry context across surfaces.</span><dl class="comms-kv compact"><div><dt>Source</dt><dd data-comms-handoff-source="true">Messenger</dd></div><div><dt>Destination</dt><dd data-comms-handoff-destination="true">Mail</dd></div><div><dt>Audience</dt><dd data-comms-handoff-audience="true">CFO · SRE · Governance</dd></div><div><dt>Persistence</dt><dd>Browser local state only</dd></div></dl><div class="comms-handoff-actions"><button type="button" data-comms-action="thread-to-mail">Promote to Mail</button><button type="button" data-comms-action="publish-note">Publish local</button></div></section><section><p class="screen-anchor">DELIVERY MATRIX</p><div class="comms-delivery-matrix" aria-label="Local delivery readiness"><span class="ready"><strong>Messenger</strong><em>ops room draft</em></span><span class="ready"><strong>Mail</strong><em>approval brief</em></span><span class="review"><strong>Community</strong><em>council review</em></span><span><strong>Audit</strong><em>receipt attached</em></span></div></section><section><p class="screen-anchor">LOCAL NOTIFICATIONS</p><ol class="notification-stack"><li>Draft queued locally</li><li>Evidence link available</li><li>No external send enabled</li><li>Workflow route preview ready</li></ol></section></aside></div><template data-mail-preview="{mail}"></template><template data-community-preview="{community}"></template><template data-comms-board-template="Mail">{mail_board}</template><template data-comms-board-template="Community">{community_board}</template><template data-comms-board-template="Messenger">{messenger_board}</template></div>"#,
        messenger = messenger,
        messenger_board = messenger_board,
        mail_board = mail_board,
        community_board = community_board,
        comms_receipt_bridge = comms_receipt_bridge,
        mail = escape(&mail),
        community = escape(&community),
        surface = selected.surface.label(),
        title = escape(&selected.title),
        body = escape(&selected.body),
        meta = escape(&selected.meta),
        selected_chip = selected_chip,
        selected_kind = selected_kind,
        mail_count = mail_items.len(),
        community_count = community_items.len(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_hub_button(item: &HubItem, index: usize) -> String {
    let kind = hub_item_kind(item, index);
    let chip = hub_item_chip_class(kind);
    let active = if index == 0 { " active" } else { "" };
    format!(
        r#"<button type="button" class="hub-item{active}" data-comms-item="true" data-comms-kind="{kind}"><span class="{chip}">{source}</span><strong>{title}</strong><p>{body}</p><small><em>{kind}</em><b>{meta}</b></small></button>"#,
        active = active,
        kind = kind,
        chip = chip,
        source = escape(&item.source),
        title = escape(&item.title),
        body = escape(&item.body),
        meta = escape(&item.meta),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_palette() -> String {
    r#"<aside class="workflow-palette" aria-label="Workflow building blocks"><div class="palette-search"><span aria-hidden="true">⌕</span><input data-workflow-palette-search="true" aria-label="Search workflow blocks" placeholder="Search nodes..." /><kbd>⌘K</kbd></div><div class="palette-heading"><span>Primitives</span><em>12</em></div><button type="button" data-palette-item="primitive"><span>System task</span><small>Deterministic step · 0 ms</small><kbd>S</kbd></button><button type="button" data-palette-item="primitive"><span>Approval</span><small>Single, parallel, or quorum</small><kbd>A</kbd></button><button type="button" data-palette-item="primitive"><span>Validation</span><small>Rule check · halts on fail</small><kbd>V</kbd></button><button type="button" data-palette-item="primitive"><span>External call</span><small>HTTP, RPC, or connector</small><kbd>E</kbd></button><button type="button" data-palette-item="primitive"><span>Branch / Switch</span><small>Multi-way condition split</small><kbd>B</kbd></button><button type="button" data-palette-item="primitive"><span>Wait / Timer</span><small>Until time · or duration</small><kbd>W</kbd></button><button type="button" data-palette-item="primitive"><span>Loop / For-each</span><small>Iterate over collection</small><kbd>L</kbd></button><button type="button" data-palette-item="primitive"><span>AI step</span><small>Suggest · classify · extract</small><kbd>⌥A</kbd></button><button type="button" data-palette-item="primitive"><span>중단 / 에스컬레이트</span><small>CFO 알림 · 실행 중단</small><kbd>H</kbd></button><button type="button" data-palette-item="primitive"><span>Form / Input</span><small>Collect data from human</small><kbd>F</kbd></button><button type="button" data-palette-item="primitive"><span>Webhook trigger</span><small>Inbound event start</small><kbd>T</kbd></button><button type="button" data-palette-item="primitive"><span>End / Receipt</span><small>Emit immutable event</small><kbd>⌘E</kbd></button><div class="palette-heading"><span>Actions</span><em>6</em></div><button type="button" data-palette-item="action"><span>Task</span><small>Create a governed work item</small><kbd>T</kbd></button><button type="button" data-palette-item="action"><span>HTTP request</span><small>Call external REST/HTTP</small><kbd>H</kbd></button><button type="button" data-palette-item="action"><span>Database</span><small>Read/write a record</small><kbd>D</kbd></button><button type="button" data-palette-item="action"><span>Transform</span><small>Reshape the payload</small><kbd>X</kbd></button><button type="button" data-palette-item="action"><span>Filter</span><small>Drop failed items</small><kbd>F</kbd></button><button type="button" data-palette-item="action"><span>Write to doc</span><small>Append a row / line</small><kbd>W</kbd></button><div class="palette-heading"><span>Logic</span><em>5</em></div><button type="button" data-palette-item="logic"><span>If / Branch</span><small>Two-way condition split</small><kbd>I</kbd></button><button type="button" data-palette-item="logic"><span>Switch</span><small>Multi-way routing</small><kbd>S</kbd></button><button type="button" data-palette-item="logic"><span>Loop / For-each</span><small>Iterate collection</small><kbd>L</kbd></button><button type="button" data-palette-item="logic"><span>Wait</span><small>Delay or duration</small><kbd>W</kbd></button><button type="button" data-palette-item="logic"><span>Merge</span><small>Wait for branches</small><kbd>M</kbd></button><div class="palette-heading"><span>Built-in surfaces</span><em>3</em></div><button type="button" data-palette-item="surface"><span>Messenger post</span><small>Route run summary to Ops room</small><kbd>M</kbd></button><button type="button" data-palette-item="surface"><span>Mail draft</span><small>Formal approval note</small><kbd>⌘M</kbd></button><button type="button" data-palette-item="surface"><span>Community note</span><small>Publish governed update</small><kbd>C</kbd></button><div class="palette-heading"><span>Connectors</span><em>9</em></div><div class="workflow-connector-grid" aria-label="Workflow connector shortcuts"><button type="button" data-palette-item="connector"><strong>국세</strong><span>HomeTax</span></button><button type="button" data-palette-item="connector"><strong>국민</strong><span>NPS / 4대</span></button><button type="button" data-palette-item="connector"><strong>신한</strong><span>Shinhan</span></button><button type="button" data-palette-item="connector"><strong>T</strong><span>Toss</span></button><button type="button" data-palette-item="connector"><strong>K</strong><span>Kakao Work</span></button><button type="button" data-palette-item="connector"><strong>#</strong><span>Slack</span></button><button type="button" data-palette-item="connector"><strong>G</strong><span>Workspace</span></button><button type="button" data-palette-item="connector"><strong>Q</strong><span>QuickBooks</span></button><button type="button" data-palette-item="connector"><strong>N</strong><span>Notion</span></button></div></aside>"#.to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_process_chrome() -> &'static str {
    r#"<div class="workflow-process-chrome" aria-label="Workflow process command chrome"><div class="workflow-process-meta"><span>PROCESS</span><strong>PROC-PAYROLL-CLOSE</strong><span>OWNER</span><strong>Hyo-jin Park · #274</strong><span>SLA</span><strong>4.0d</strong></div><div class="workflow-process-actions"><button type="button" data-workflow-process-action="validate">✓ Validate</button><button type="button" data-workflow-process-action="simulate">◷ Simulate</button><button type="button" data-workflow-process-action="diff">↯ Diff v17 → v18</button><button type="button" class="dark-action" data-workflow-process-action="publish">게시 v18</button></div><span class="workflow-process-status" data-workflow-process-status="true">autosaved · local visual IDE</span></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_lens_toolbar() -> &'static str {
    r#"<div class="workflow-lens-toolbar" aria-label="Workflow layout, overlay, and filter controls"><div class="workflow-lens-group"><span>LAYOUT</span><button type="button" class="active" data-workflow-lens="Graph">Graph</button><button type="button" data-workflow-lens="Swimlanes">Swimlanes</button><button type="button" data-workflow-lens="Timeline">Timeline</button><button type="button" data-workflow-lens="Tree">Tree</button></div><div class="workflow-lens-group"><span>OVERLAY</span><button type="button" data-workflow-overlay="Cycle">Cycle</button><button type="button" class="active" data-workflow-overlay="Cost">Cost</button><button type="button" data-workflow-overlay="Owner">Owner</button><button type="button" data-workflow-overlay="Risk">Risk</button><button type="button" data-workflow-overlay="Off">Off</button></div><div class="workflow-lens-group"><span>FILTER</span><button type="button" data-workflow-filter="All">All</button><button type="button" class="active" data-workflow-filter="Critical path">Critical path</button><button type="button" data-workflow-filter="Bottlenecks">Bottlenecks</button><button type="button" data-workflow-filter="AI suggestions">AI suggestions</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_canvas_metrics() -> &'static str {
    r#"<div class="workflow-canvas-metrics" aria-label="Workflow simulation metrics overlay"><span><small>CYCLE</small><strong>5.4d</strong><em>+1.4 vs target</em></span><span><small>TARGET</small><strong>4.0d</strong><em>SLA limit</em></span><span><small>COST</small><strong>₩2.18M</strong><em>delay cost</em></span><span><small>REWORK</small><strong>8%</strong><em>2 loops</em></span></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_property_form() -> &'static str {
    r#"<form class="workflow-property-form" aria-label="Selected workflow node properties"><label><span>LABEL · KO</span><input data-workflow-prop="label-ko" value="재무 검토 · 사인오프" /></label><label><span>TYPE</span><select data-workflow-prop="type"><option>Single · auto-delegate</option><option>Parallel quorum</option><option>Human review stop</option></select></label><label><span>OWNER</span><select data-workflow-prop="owner"><option>Sarah Kim · EMP-188 · HR Manager</option><option>Choi Yu-na · CFO</option><option>David Chen · Delegate</option></select></label><div class="workflow-form-row"><label><span>SLA TARGET</span><input data-workflow-prop="sla" value="1.2d" /></label><label><span>ESCALATE AFTER</span><input data-workflow-prop="escalate" value="0.8d" /></label></div><fieldset class="workflow-rule-stack"><legend>승인 조건</legend><label><span>1</span><input data-workflow-prop="rule-1" value="payroll.gross > ₩500,000,000" /></label><label><span>2</span><input data-workflow-prop="rule-2" value="policy.P0 == active" /></label><button type="button" data-workflow-process-action="add-condition">+ Add condition</button></fieldset></form>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_output_bus() -> &'static str {
    r#"<div class="workflow-output-bus" data-workflow-output-bus="true" aria-label="Workflow output bus for FD-001 tenant workload routes"><div class="workflow-output-head"><p class="screen-anchor">FD-001 OUTPUT BUS</p><strong>Run preview emits tenant workload drafts</strong><span data-workflow-output-status="true">Idle · run/validate/publish stays local until a route is selected</span></div><div class="workflow-output-flow" aria-label="Workflow output routes"><button type="button" class="selected" data-workflow-output-route="messenger"><span>01</span><strong>Messenger</strong><em>Ops room run note</em></button><button type="button" data-workflow-output-route="mail"><span>02</span><strong>Mail</strong><em>Approval brief</em></button><button type="button" data-workflow-output-route="community"><span>03</span><strong>Community</strong><em>Council digest</em></button><button type="button" data-workflow-output-route="evidence"><span>04</span><strong>Evidence</strong><em>Receipt spine</em></button></div><aside class="workflow-output-proof" aria-label="FD-001 and Oyatie Cloud proof context"><dl><div><dt>Product goal</dt><dd>FD-001 delivery</dd></div><div><dt>Substrate</dt><dd>Oyatie Cloud · cell-us-east-2</dd></div><div><dt>Receipt</dt><dd data-workflow-output-receipt="true">REC-FD001-WF-018 · draft</dd></div></dl></aside></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_studio_panel(envelope: &TenantRenderEnvelope) -> String {
    let display_nodes = workflow_display_nodes(&envelope.workflow.nodes, 0);
    let nodes = display_nodes
        .iter()
        .map(|node| format!("<button type=\"button\">{}</button>", escape(&node.label)))
        .collect::<String>();
    let selected_node = display_nodes
        .first()
        .map(static_selected_node)
        .unwrap_or_default();

    format!(
        r#"<section id="workflow-studio" class="panel workflow-panel cohesive-workflow" aria-labelledby="workflow-title"><div class="workflow-topbar"><div><p class="eyebrow">Workflow Studio</p><h3 id="workflow-title">{name}</h3><div class="workflow-doc-meta"><span>v18 · draft</span><span>Owner · tenant admin</span><span>SLA · 4.0h</span></div></div><div class="workflow-run-chip"><span></span>draft · select mode</div><div class="workflow-actions"><button type="button">Fit</button><button type="button">Clear run</button><button type="button">Validate</button><button class="primary-action" type="button">Run</button><button type="button">Add block</button><button class="dark-action" type="button">Publish</button></div></div>{workflow_process_chrome}{workflow_output_bus}<p class="panel-intro">{goal}</p><div class="workflow-modebar" role="toolbar" aria-label="Workflow editor modes"><button type="button" class="active">Select</button><button type="button">Connect</button><button type="button">Simulate</button></div>{workflow_lens_toolbar}<div class="workflow-ide">{workflow_palette}<div class="workflow-canvas island-frame"><div class="workflow-toolbar"><button type="button">Select</button><button type="button">Connect</button><button type="button">Simulate</button><span class="island-label">interactive island</span></div>{workflow_board}<div class="canvas-footer"><div class="zoom-controls"><button type="button">−</button><span>82%</span><button type="button">+</button></div><div class="mini-map" aria-hidden="true"><span></span><span></span><span></span><span></span></div></div><div class="node-toolbar">{nodes}</div></div><aside class="workflow-inspector"><div class="inspector-tabs" aria-hidden="true"><span class="active">Inspector</span><span>Run log</span><span>Rules</span><span>History</span></div>{selected_node}{workflow_property_form}<dl class="inspector-fields"><div><dt>Guardrail</dt><dd>Human review before action</dd></div><div><dt>Output</dt><dd>Task · message · evidence draft</dd></div><div><dt>Execution</dt><dd>Disabled until live integration</dd></div></dl><div class="inspector-stat-grid" aria-label="Selected node run statistics"><div><span>Avg</span><strong>0.8s</strong></div><div><span>P95</span><strong>2.1s</strong></div><div><span>Errors</span><strong>0</strong></div><div><span>Cost</span><strong>$0.03</strong></div></div><div class="run-log-preview"><p class="eyebrow">Run log</p><ol><li><time>10:31</time><span>Validation preview passed</span></li><li><time>10:32</time><span>Messenger/Mail/Community drafts generated</span></li><li><time>10:33</time><span>Audit receipt staged locally</span></li></ol></div></aside></div><div class="workflow-statusbar"><span>Nodes: {node_count}</span><span>Local blocks: 0</span><span>Messenger/Mail/Community outputs are drafts</span><span>Ready · staged</span></div></section>"#,
        name = escape(&envelope.workflow.name),
        goal = escape(&envelope.workflow.goal),
        workflow_board = static_workflow_board(&display_nodes),
        workflow_palette = static_workflow_palette(),
        workflow_process_chrome = static_workflow_process_chrome(),
        workflow_output_bus = static_workflow_output_bus(),
        workflow_lens_toolbar = static_workflow_lens_toolbar(),
        workflow_property_form = static_workflow_property_form(),
        nodes = nodes,
        selected_node = selected_node,
        node_count = display_nodes.len(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_context_switcher(active: OperatorContext) -> String {
    let cards = OperatorContext::ALL
        .iter()
        .map(|context| {
            let selected = if *context == active { " context-card selected" } else { " context-card" };
            let pressed = if *context == active { "true" } else { "false" };
            format!(
                "<button type=\"button\" class=\"{class}\" aria-pressed=\"{pressed}\"><span class=\"context-icon\" aria-hidden=\"true\">{icon}</span><span class=\"context-label\">{label}</span><span class=\"context-role\">{role}</span></button>",
                class = selected.trim(),
                pressed = pressed,
                icon = escape(context_icon(*context)),
                label = escape(context.label()),
                role = escape(context.role())
            )
        })
        .collect::<String>();

    format!(
        r#"<section class="context-switcher island-frame" aria-labelledby="context-title"><div><p class="eyebrow">Context</p><h2 id="context-title">Switch render envelope</h2><span class="island-label">interactive island</span></div><div class="context-grid" role="list" aria-label="Tenant and role contexts">{cards}</div></section>"#
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_envelope_banner(envelope: &TenantRenderEnvelope) -> String {
    let accreditation_class = if envelope.accreditation.healthcare_enabled {
        "badge success"
    } else {
        "badge warning"
    };

    format!(
        r#"<section class="envelope-banner" aria-labelledby="envelope-title"><div><p class="eyebrow">Active context</p><h2 id="envelope-title">{tenant}</h2><p>{role}</p></div><div class="envelope-detail"><span class="badge">{tenant_class}</span><span class="{accreditation_class}">{accreditation}</span><p>{derivation}</p></div></section>"#,
        tenant = escape(&envelope.tenant_name),
        role = escape(&envelope.role_name),
        tenant_class = escape(&envelope.tenant_class),
        accreditation_class = accreditation_class,
        accreditation = escape(&envelope.accreditation.label),
        derivation = escape(&envelope.server_derivation_note),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_metric(metric: &MetricCard) -> String {
    format!(
        "<article class=\"metric-card\"><p>{}</p><strong>{}</strong><span>{}</span></article>",
        escape(&metric.label),
        escape(&metric.value),
        escape(&metric.detail)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_substrate_proof_command(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section id="substrate-proof" class="substrate-proof-command panel" data-substrate-proof="true" aria-labelledby="substrate-proof-title"><div class="substrate-proof-head"><div><p class="screen-anchor">OYATIE CLOUD · FD-001 DOGFOOD SUBSTRATE</p><h3 id="substrate-proof-title">Prove production tenancy by running FD-001 as real tenant workloads</h3><p>FD-001 remains the product delivery goal. Oyatie Cloud is the hyperscaler-grade substrate proving those microservices can host production tenants before any external claim.</p></div><div class="substrate-proof-actions"><span class="status-chip success" data-substrate-status="true">12 workloads · 3 cells · 0 external writes</span><button type="button" data-substrate-action="cloud">Cloud cells</button><button type="button" data-substrate-action="workflow">Workflow proof</button><button type="button" data-substrate-action="evidence">Evidence</button></div></div><div class="substrate-proof-grid" aria-label="Substrate proof metrics"><article class="substrate-proof-card primary"><p class="screen-anchor">PRODUCT GOAL</p><strong>FD-001 delivery</strong><span>Core, workflow, messenger, mail, community, finance, identity, intelligence, and ontology run as tenant workload previews.</span></article><article class="substrate-proof-card"><p class="screen-anchor">SUBSTRATE</p><strong>Oyatie Cloud</strong><span>Cellular runtime, policy, FinOps, resource inventory, deployment gates, and rollback evidence.</span></article><article class="substrate-proof-card"><p class="screen-anchor">TENANT LENS</p><strong>{tenant}</strong><span>{role} · server-derived envelope · local dogfood only</span></article><article class="substrate-proof-card warning"><p class="screen-anchor">READINESS</p><strong>84% proof</strong><span>3 blockers: payroll delta, cloud rollback receipt, PIPA review.</span></article></div><div class="substrate-workload-map" aria-label="FD-001 tenant workload deployment map"><div class="substrate-map-column substrate-product-column"><p class="screen-anchor">FD-001 WORKLOADS</p><button type="button" data-substrate-action="workflow"><strong>Workflow</strong><span>approval engine · no-code studio</span></button><button type="button" data-substrate-action="messenger"><strong>Messenger</strong><span>ops room thread · evidence extraction</span></button><button type="button" data-substrate-action="mail"><strong>Mail</strong><span>formal approval brief</span></button><button type="button" data-substrate-action="community"><strong>Community</strong><span>governance council post</span></button></div><div class="substrate-map-spine" aria-hidden="true"><span>tenant workload</span><i></i><span>cell runtime</span><i></i><span>evidence receipt</span></div><div class="substrate-map-column substrate-cloud-column"><p class="screen-anchor">OYATIE CLOUD CELLS</p><button type="button" data-substrate-action="cloud"><strong>cell-us-east-2</strong><span>primary · workload dogfood</span></button><button type="button" data-substrate-action="finops"><strong>kr-seoul-1</strong><span>localization pack · FinOps watch</span></button><button type="button" data-substrate-action="deployment"><strong>gitops promotion</strong><span>Jenkins · ArgoCD · cosign · audit</span></button><button type="button" data-substrate-action="evidence"><strong>evidence spine</strong><span>REC-FD001-CLOUD-009</span></button></div></div><div class="substrate-proof-footer" aria-label="Dogfood proof routes"><span>Proof loop: tenant workload → Oyatie Cloud cell → policy gate → human route → evidence receipt</span><button type="button" data-substrate-action="finance">Finance close</button><button type="button" data-substrate-action="identity">Identity policy</button><button type="button" data-substrate-action="catalog">Service catalog</button></div></section>"#,
        tenant = escape(&envelope.tenant_name),
        role = escape(&envelope.role_name),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_action_inbox_proof_board() -> &'static str {
    r#"<section class="daily-proof-board inbox-proof-board" aria-label="FD-001 Action Inbox and Oyatie Cloud execution proof"><div class="daily-proof-grid"><article class="daily-proof-card selected" data-daily-proof-card="inbox-fd001"><p class="screen-anchor">FD-001 ACTION INBOX</p><h5>Priority queue is the product control plane</h5><p>Blocking payroll, vendor, policy, Workflow, Mail, Messenger, Community, and evidence items stay inside the FD-001 tenant workload graph.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="route-daily">Daily queue</button><button type="button" data-daily-proof-action="route-workflow">Workflow gate</button></div></article><article class="daily-proof-card" data-daily-proof-card="inbox-cloud"><p class="screen-anchor">OYATIE CLOUD ADMISSION</p><h5>Every item can prove tenant readiness</h5><p>Oyatie Cloud substrate checks policy, residency, release gates, FinOps, and audit freshness before any FD-001 workload claims production readiness.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="route-cloud">Cloud cells</button><button type="button" data-daily-proof-action="route-policy">Policy board</button></div></article><article class="daily-proof-card" data-daily-proof-card="inbox-local"><p class="screen-anchor">LOCAL-ONLY REVIEW</p><h5>Interactive, never wired</h5><p>Operators can select, defer, brief, and attach receipts visually; no approval, auth, workflow execution, mail send, payroll, billing, or cloud mutation runs.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="stage-packet">Stage packet</button><button type="button" data-daily-proof-action="route-audit">Audit ledger</button></div></article></div><div class="daily-proof-footer"><span data-daily-proof-status="true">Action Inbox ready · FD-001 priority work dogfoods Oyatie Cloud locally.</span><div class="daily-proof-routes" aria-label="Action Inbox connected routes"><button type="button" data-daily-proof-action="route-mail">Reviewer Mail</button><button type="button" data-daily-proof-action="route-community">Community</button><button type="button" data-daily-proof-action="route-evidence">Evidence</button></div></div></section>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_command_center_workbench(envelope: &TenantRenderEnvelope) -> String {
    let tasks = envelope
        .daily_tasks
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let priority_key = if item.priority.eq_ignore_ascii_case("high") {
                "blocking"
            } else if index % 2 == 0 {
                "mine"
            } else {
                "all"
            };
            let class = if priority_key == "blocking" {
                "workbench-row blocking"
            } else {
                "workbench-row"
            };
            let chip = if item.priority.eq_ignore_ascii_case("high") {
                "status-chip danger"
            } else {
                "status-chip"
            };
            format!(
                r#"<article class="{class}" data-workbench-row="{priority_key}" data-inbox-row="true"><label class="inbox-select-cell"><input type="checkbox" data-inbox-select="true" data-inbox-title="{title}" aria-label="Select {title}" /></label><span class="workbench-row-id">ACT-78{row}</span><span class="{chip}">{priority}</span><button type="button" class="inbox-row-main" data-sidepeek-trigger="action-inbox" data-sidepeek-title="{title}" data-sidepeek-id="ACT-78{row}" data-sidepeek-desc="{detail}" data-sidepeek-owner="{owner}" data-sidepeek-risk="{priority}" data-sidepeek-sla="4.0h target · local data"><strong>{title}</strong><p>{detail}</p></button><time>{time}</time><span class="inbox-row-actions"><button type="button" data-inbox-row-action="workflow">Flow</button><button type="button" data-inbox-row-action="mail">Mail</button><button type="button" data-inbox-row-action="audit">Audit</button></span></article>"#,
                class = class,
                priority_key = priority_key,
                title = escape(&item.title),
                row = index + 41,
                detail = escape(&item.detail),
                owner = escape(&envelope.role_name),
                priority = escape(&item.priority),
                chip = chip,
                time = if index == 0 { "오늘 18:00" } else if index == 1 { "내일 09:00" } else { "5월 10일" },
            )
        })
        .collect::<String>();
    let approvals = envelope
        .approvals
        .iter()
        .enumerate()
        .map(|(index, item)| {
            format!(
                r#"<article class="workbench-row approval" data-workbench-row="mine" data-inbox-row="true"><label class="inbox-select-cell"><input type="checkbox" data-inbox-select="true" data-inbox-title="{title}" aria-label="Select {title}" /></label><span class="workbench-row-id">APR-{row}</span><span class="status-chip warning">approval</span><button type="button" class="inbox-row-main" data-sidepeek-trigger="approval" data-sidepeek-title="{title}" data-sidepeek-id="APR-{row}" data-sidepeek-desc="{risk}" data-sidepeek-owner="{owner}" data-sidepeek-risk="Review" data-sidepeek-sla="Reviewer queue · visual only"><strong>{title}</strong><p>{owner} · {risk}</p></button><time>대기</time><span class="inbox-row-actions"><button type="button" data-inbox-row-action="workflow">Flow</button><button type="button" data-inbox-row-action="mail">Mail</button><button type="button" data-inbox-row-action="audit">Audit</button></span></article>"#,
                title = escape(&item.title),
                row = index + 274,
                risk = escape(&item.risk_note),
                owner = escape(&item.requester),
            )
        })
        .collect::<String>();
    let suggestions = envelope
        .intelligence
        .iter()
        .enumerate()
        .map(|(index, suggestion)| {
            format!(
                r#"<article class="copilot-card"><strong>{title}</strong><p>{body}</p><small>{guardrail}</small><div><button type="button" data-copilot-action="apply">Apply delegation</button><button type="button" data-copilot-action="dismiss">Dismiss</button><button type="button" data-sidepeek-trigger="copilot-trace" data-sidepeek-title="Copilot trace" data-sidepeek-id="AI-TRACE-{index}" data-sidepeek-desc="Shows why a governed local suggestion is visible in this render envelope." data-sidepeek-owner="Governed Copilot" data-sidepeek-risk="Advisory" data-sidepeek-sla="Never auto-executes">Trace</button></div></article>"#,
                title = escape(&suggestion.title),
                body = escape(&suggestion.body),
                guardrail = escape(&suggestion.guardrail),
                index = index,
            )
        })
        .collect::<String>();

    format!(
        r#"<section id="command-center-workbench" class="command-center-workbench" aria-labelledby="command-workbench-title"><article class="priority-workbench panel" aria-labelledby="command-workbench-title"><div class="workbench-head"><div><p class="screen-anchor">ACTION INBOX</p><h3 id="command-workbench-title">Priority queue <span>8</span></h3></div><div class="workbench-filters" role="toolbar" aria-label="Action inbox filters"><button type="button" class="active" data-workbench-filter="all">All</button><button type="button" data-workbench-filter="mine">Mine</button><button type="button" data-workbench-filter="blocking">Blocking</button></div></div><div class="workbench-summary-strip" aria-label="Action inbox summary"><span><strong>3</strong><small>blocking</small></span><span><strong>5</strong><small>owned by you</small></span><span><strong>4.0h</strong><small>SLA pressure</small></span><span><strong>12</strong><small>evidence links</small></span></div>{action_inbox_proof_board}<div class="workbench-bulkbar" aria-label="Action inbox bulk operations"><label><input type="checkbox" data-inbox-select-all="true" aria-label="Select all visible inbox items" /><span><strong data-inbox-selected-count="true">0</strong> selected</span></label><div class="workbench-bulk-actions"><button type="button" data-inbox-bulk="approve" disabled>Approve</button><button type="button" data-inbox-bulk="defer" disabled>Defer</button><button type="button" data-inbox-bulk="mail" disabled>Mail brief</button><button type="button" data-inbox-bulk="evidence" disabled>Attach evidence</button></div><span class="workbench-status" data-inbox-status="true">No items selected · local inbox only</span></div><div class="workbench-list" role="list" aria-label="Operational priority queue">{tasks}{approvals}</div></article><aside class="governed-copilot panel" aria-labelledby="copilot-workbench-title"><div class="copilot-head"><div><p class="screen-anchor">COPILOT · GOVERNED</p><h3 id="copilot-workbench-title">Suggested next moves</h3></div><span class="status-chip ai">PIPA-safe</span></div><div class="copilot-suggestions" aria-live="polite">{suggestions}</div><p class="copilot-status" data-copilot-status="true">Read-only · scoped to roster + run + workflow data · suggestions never auto-execute.</p></aside></section>"#,
        tasks = tasks,
        approvals = approvals,
        suggestions = suggestions,
        action_inbox_proof_board = static_action_inbox_proof_board(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_filing_readiness_anchor_board() -> &'static str {
    r#"<div class="trust-anchor-board filing-trust-board" aria-label="FD-001 filing readiness and Oyatie Cloud localization proof"><div class="trust-anchor-grid"><article class="trust-anchor-card selected" data-trust-proof-card="filing-fd001"><p class="screen-anchor">FD-001 LOCALIZATION WORKLOAD</p><h5>Korea filing is tenant workload delivery, not a side widget</h5><p>Withholding return, employee validation, HomeTax transport, billing, Mail, Community, and evidence receipts stay in the FD-001 tenant workload graph.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="stage-filing">Stage filing</button><button type="button" data-trust-proof-action="route-billing">Billing · tax</button></div></article><article class="trust-anchor-card" data-trust-proof-card="filing-cloud"><p class="screen-anchor">OYATIE CLOUD RESIDENCY</p><h5>Substrate proves regional pack posture</h5><p>Oyatie Cloud shows PIPA-aware residency, policy envelope, release gates, audit freshness, and rollback posture before any tax workload readiness claim.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-policy">PIPA policy</button><button type="button" data-trust-proof-action="route-cloud">Cloud cells</button></div></article><article class="trust-anchor-card" data-trust-proof-card="filing-local"><p class="screen-anchor">LOCAL-ONLY FILING RAIL</p><h5>Reviewer-ready, never submitted</h5><p>Operators can inspect readiness, stage a reviewer packet, and route council notes visually; no HomeTax, bank, payroll, billing, mail, or cloud mutation executes.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-mail">Reviewer Mail</button><button type="button" data-trust-proof-action="route-evidence">Receipt spine</button></div></article></div><div class="trust-anchor-footer"><span data-trust-proof-status="true">Filing readiness ready · FD-001 localization workload dogfoods Oyatie Cloud with local-only submission controls.</span><div class="trust-anchor-routes" aria-label="Filing readiness connected routes"><button type="button" data-trust-proof-action="route-finance">Finance close</button><button type="button" data-trust-proof-action="route-community">Community note</button><button type="button" data-trust-proof-action="route-audit">Audit ledger</button><button type="button" data-trust-proof-action="route-catalog">Catalog</button></div></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_tenant_rbac_board(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section class="tenant-rbac-board" aria-labelledby="service-board-title" id="business-logics"><div class="service-board-head"><div><p class="screen-anchor">TENANT RBAC SERVICE GRAPH</p><h3 id="service-board-title">Corporate operations graph</h3></div><span class="status-chip success">{module_count} permitted services</span></div>{business_logic_os}{governance_ops_command}<article id="payroll-cockpit" class="service-card payroll-card"><p class="screen-anchor">PAYROLL CLOSE</p><h4>2026-04 payroll close</h4><p class="service-card-brief">FD-001 payroll workload dogfooded on Oyatie Cloud with workflow, Mail, and evidence return paths.</p><div class="service-metric-row"><span><strong>73%</strong><small>close progress</small></span><span><strong>5.4d</strong><small>cycle time</small></span><span><strong>₩2.18M</strong><small>cost of delay</small></span></div><ol class="service-checklist"><li><span class="status-chip danger">blocking</span>4대보험 변동 확인 필요</li><li><span class="status-chip warning">review</span>Payroll reminder mail draft ready</li><li><span class="status-chip success">sealed</span>Receipt REC-PAY-2026-04 staged</li></ol><div class="service-card-actions"><button type="button" data-service-action="payroll-finance">Finance cockpit</button><button type="button" data-service-action="payroll-workflow">Workflow gate</button><button type="button" data-service-action="payroll-mail">Mail brief</button><button type="button" data-service-action="payroll-evidence">Evidence</button></div></article><article id="filing-readiness" class="service-card filing-card"><p class="screen-anchor">FILING READINESS</p><h4>Withholding return</h4><p class="service-card-brief">Korea localization workload joins the same substrate proof loop: reviewer attestation, transport, and receipt.</p><div class="readiness-bars compact" aria-label="Filing readiness"><span style="--bar: 86%"><em>Employee validation</em></span><span style="--bar: 64%"><em>HomeTax transport</em></span><span style="--bar: 52%"><em>Reviewer attestation</em></span></div><div class="service-card-actions"><button type="button" data-sidepeek-trigger="filing" data-sidepeek-title="Withholding return" data-sidepeek-id="FILE-KR-2026-04" data-sidepeek-desc="Filing readiness is staged locally and never submitted before live integration." data-sidepeek-owner="Finance close" data-sidepeek-risk="2 review" data-sidepeek-sla="Due 2026-05-10">Inspect</button><button type="button" data-service-action="filing-billing">Billing · tax</button><button type="button" data-service-action="filing-community">Council note</button><button type="button" data-service-action="filing-evidence">Receipt</button></div>{filing_readiness_anchor_board}</article><article id="employee-directory" class="service-card employee-card"><p class="screen-anchor">EMPLOYEES</p><h4>Employee directory</h4><p class="service-card-brief">Identity and workforce data remain role-visible while FD-001 tenant workloads prove policy envelopes.</p><div class="service-people-stats"><span><strong>118</strong><small>employees</small></span><span><strong>109</strong><small>active</small></span><span><strong>5</strong><small>probation watch</small></span></div><table class="employee-mini-table"><thead><tr><th>Name</th><th>Role</th><th>Team</th><th>Status</th></tr></thead><tbody><tr><td>이재현 Jaehyun Lee</td><td>CEO</td><td>Office</td><td>활성</td></tr><tr><td>최유나 Yuna Choi</td><td>CFO</td><td>Finance</td><td>활성</td></tr><tr><td>박서준 Seojun Park</td><td>VP Engineering</td><td>Infrastructure</td><td>활성</td></tr><tr><td>김지영 Jiyoung Kim</td><td>Manager</td><td>Infrastructure</td><td>수습</td></tr></tbody></table><div class="service-card-actions"><button type="button" data-service-action="employee-identity">Identity service</button><button type="button" data-service-action="employee-onboarding">Onboarding</button><button type="button" data-service-action="employee-policy">Policy</button><button type="button" data-service-action="employee-mail">Mail reviewer</button></div></article><article id="governance-analytics-summary" class="service-card governance-card"><p class="screen-anchor">GOVERNANCE ANALYTICS</p><h4>Policy, receipts, workflow health</h4><p class="service-card-brief">Executive posture rolls FD-001 workload proof, cloud cell evidence, and built-in surface routes into one council view.</p><div class="service-graph" aria-hidden="true"><span style="--bar: 78%"></span><span style="--bar: 48%"></span><span style="--bar: 92%"></span><span style="--bar: 61%"></span><span style="--bar: 72%"></span></div><dl class="service-kv"><div><dt>Risk</dt><dd>3 high-risk approvals</dd></div><div><dt>Evidence</dt><dd>12 sealed draft receipts</dd></div><div><dt>Workflow</dt><dd>{workflow}</dd></div></dl><div class="service-card-actions"><button type="button" data-service-action="governance-command">Command board</button><button type="button" data-service-action="governance-risk">Risk heatmap</button><button type="button" data-service-action="governance-community">Community</button><button type="button" data-service-action="governance-evidence">Evidence</button></div></article></section>"#,
        module_count = envelope.modules.len(),
        workflow = escape(&envelope.workflow.name),
        business_logic_os = static_business_logic_os_panel(),
        governance_ops_command = static_governance_ops_command_board(&envelope.workflow.name),
        filing_readiness_anchor_board = static_filing_readiness_anchor_board(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_governance_ops_command_board(workflow_name: &str) -> String {
    format!(
        r#"<section id="governance-analytics" class="governance-ops-command" aria-labelledby="governance-ops-title"><div class="governance-command-head"><div><p class="screen-anchor">00 / GOVERNANCE ANALYTICS</p><h4 id="governance-ops-title">Policy, evidence, risk, and business logic control</h4><p>Bominal-grade executive governance: posture, risk heatmap, control attestations, compliance calendar, decision queue, evidence chain, and routes into every built-in surface.</p></div><div class="governance-command-actions"><span class="status-chip warning" data-governance-status="true">84 posture · 5 top risks · local only</span><button type="button" data-governance-action="run-review">Run review</button><button type="button" data-governance-action="seal-brief" data-governance-route="evidence">Seal brief</button><button type="button" data-governance-action="route-inbox" data-governance-route="inbox">Open queue</button></div></div><div class="governance-posture-strip" aria-label="Governance posture overview"><article class="gov-posture-score"><span>Composite posture</span><strong>84</strong><em>A− · +3 vs last quarter · target 90</em></article><article class="gov-posture-trend"><div><span>13 week trend</span><strong>78 → 84</strong></div><svg viewBox="0 0 160 42" aria-hidden="true" class="gov-sparkline"><polyline points="0,34 14,32 28,32 42,29 56,29 70,27 84,29 98,26 112,23 126,23 140,18 154,16" /></svg></article><div class="gov-pillar-grid" aria-label="Governance pillars"><button type="button" data-governance-action="pillar-compliance" data-governance-route="finance"><span>Compliance</span><strong>87</strong><em>24 controls · 1 breach</em></button><button type="button" data-governance-action="pillar-financial" data-governance-route="finance"><span>Financial controls</span><strong>91</strong><em>manual JE 4.2%</em></button><button type="button" data-governance-action="pillar-workforce" data-governance-route="identity"><span>Workforce risk</span><strong>71</strong><em>§53 watch</em></button><button type="button" data-governance-action="pillar-disclosure" data-governance-route="community"><span>Board + disclosure</span><strong>88</strong><em>pack ships May 12</em></button></div></div><div class="governance-command-grid"><article class="governance-command-card policy-gate-card"><div class="governance-card-head"><div><p class="screen-anchor">POLICY GATES</p><h5>Decision rights before risky operation</h5></div><span class="status-chip danger">2 blocks</span></div><div class="policy-gate-list" aria-label="Governance policy gates"><button type="button" class="active" data-governance-action="select-payroll" data-governance-route="workflow"><span>P0</span><strong>Payroll close</strong><em>2-person CFO signoff</em></button><button type="button" data-governance-action="select-hometax" data-governance-route="finance"><span>P0</span><strong>HomeTax filing</strong><em>사업자등록번호 confirmation</em></button><button type="button" data-governance-action="select-cloud" data-governance-route="cloud"><span>P0</span><strong>Network split</strong><em>rollback evidence required</em></button><button type="button" data-governance-action="select-pipa" data-governance-route="identity"><span>P1</span><strong>PIPA boundary</strong><em>vendor cannot view employee PII</em></button></div></article><article class="governance-command-card decision-queue-card"><div class="governance-card-head"><div><p class="screen-anchor">EXEC APPROVALS</p><h5>Owners, SLA, origin, and route</h5></div><button type="button" data-governance-action="open-inbox" data-governance-route="inbox">Inbox</button></div><table class="governance-decision-table"><thead><tr><th>Decision</th><th>Owner</th><th>SLA</th><th>State</th></tr></thead><tbody><tr><td><strong>Park 4대보험 delta</strong><small>REC-PAY-2026-04-PARK</small></td><td>CFO</td><td>4.0h</td><td><span class="status-chip danger">blocking</span></td></tr><tr><td><strong>Stripe approval compression</strong><small>policy ≤ ₩5M route</small></td><td>AP</td><td>1d</td><td><span class="status-chip warning">review</span></td></tr><tr><td><strong>Governance council note</strong><small>Community + Mail packet</small></td><td>Gov</td><td>today</td><td><span class="status-chip success">ready</span></td></tr><tr><td><strong>Board option pool</strong><small>resolution pack May 12</small></td><td>CEO</td><td>6d</td><td><span class="status-chip">scheduled</span></td></tr></tbody></table></article><article class="governance-command-card risk-matrix-card"><div class="governance-card-head"><div><p class="screen-anchor">RISK MATRIX</p><h5>Audit committee 5×5 heatmap</h5></div><span class="status-chip ai">interactive</span></div><div class="gov-risk-workspace"><div class="gov-risk-heatmap" aria-label="Selectable risk matrix"><span class="tone-minimal"></span><span class="tone-minimal"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-minimal"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-extreme"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-extreme"></span><span class="tone-extreme"></span><button type="button" class="gov-risk-pin selected" style="--x: 78%; --y: 42%" data-governance-risk="RISK-042" data-risk-title="AI agent governance" data-risk-detail="Auto-delegation rollout needs change-management, rollback evidence, and human signoff." data-risk-owner="EMP-104" data-risk-score="4×3 High">AI</button><button type="button" class="gov-risk-pin" style="--x: 58%; --y: 30%" data-governance-risk="RISK-014" data-risk-title="LSA §53 weekly-hour breach" data-risk-detail="Yoon Tae-min projected 49.5h; automatic reassignment lowers residual risk." data-risk-owner="EMP-211" data-risk-score="3×4 Moderate">53</button><button type="button" class="gov-risk-pin" style="--x: 40%; --y: 34%" data-governance-risk="RISK-009" data-risk-title="PIPA retention overrun" data-risk-detail="43 medical certificates expire in 14 days; consent renewal or purge decision required." data-risk-owner="EMP-274" data-risk-score="2×4 Moderate">PI</button><button type="button" class="gov-risk-pin" style="--x: 57%; --y: 54%" data-governance-risk="RISK-031" data-risk-title="JE four-eyes gap" data-risk-detail="7 manual journal entries posted with a single approver; enforcement pending." data-risk-owner="EMP-188" data-risk-score="3×3 Moderate">JE</button><button type="button" class="gov-risk-pin" style="--x: 21%; --y: 20%" data-governance-risk="RISK-018" data-risk-title="Missed board resolution" data-risk-detail="Option pool expansion requires May 12 board resolution and quorum confirmation." data-risk-owner="EMP-274" data-risk-score="1×5 Low">BD</button></div><aside class="gov-risk-peek" aria-live="polite"><div><span data-risk-peek-id="true">RISK-042</span><strong data-risk-peek-score="true">4×3 High</strong></div><h6 data-risk-peek-title="true">AI agent governance</h6><p data-risk-peek-detail="true">Auto-delegation rollout needs change-management, rollback evidence, and human signoff.</p><dl><div><dt>Owner</dt><dd data-risk-peek-owner="true">EMP-104</dd></div><div><dt>Next review</dt><dd>2026-05-08</dd></div></dl></aside></div></article><article class="governance-command-card compliance-calendar-card"><div class="governance-card-head"><div><p class="screen-anchor">COMPLIANCE CALENDAR</p><h5>12-month filing commitments</h5></div><button type="button" data-governance-action="calendar-review" data-governance-route="finance">Review</button></div><div class="gov-calendar" aria-label="Compliance calendar"><span class="cal-corner">Family</span><span class="cal-month now">May</span><span class="cal-month">Jun</span><span class="cal-month">Jul</span><span class="cal-month">Aug</span><span class="cal-month">Sep</span><span class="cal-month">Oct</span><span class="cal-month">Nov</span><span class="cal-month">Dec</span><span class="cal-month">Jan</span><span class="cal-month">Feb</span><span class="cal-month">Mar</span><span class="cal-month">Apr</span><strong>Withholding</strong><button type="button" class="ready" data-gov-calendar-cell="Withholding May ready">10 ₩38.2M</button><button type="button" class="pending" data-gov-calendar-cell="Withholding Jun pending">10 ₩39.1M</button><button type="button" class="pending" data-gov-calendar-cell="Withholding Jul pending">10 ₩40.0M</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><strong>4대보험</strong><button type="button" class="review" data-gov-calendar-cell="Social insurance May review">10 ₩57.0M</button><button type="button" class="pending" data-gov-calendar-cell="Social insurance Jun pending">10 ₩58.2M</button><button type="button" class="pending" data-gov-calendar-cell="Social insurance Jul pending">10 ₩59.1M</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><strong>VAT</strong><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q2 prelim pending">25 Q2 prelim</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q3 final pending">25 Q3 final</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q4 prelim pending">25 Q4 prelim</button><button type="button" class="empty">—</button><button type="button" class="empty">—</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q1 final pending">25 Q1 final</button></div></article><article class="governance-command-card evidence-readiness-card"><div class="governance-card-head"><div><p class="screen-anchor">EVIDENCE READINESS</p><h5>Receipts that prove the graph</h5></div><span class="status-chip success">sealed draft</span></div><div class="evidence-readiness-lanes" aria-label="Evidence readiness lanes"><span style="--bar: 92%"><strong>Workflow receipts</strong><em>11 / 12</em></span><span style="--bar: 78%"><strong>Mail approvals</strong><em>7 linked</em></span><span style="--bar: 64%"><strong>Cloud runbooks</strong><em>2 waiting</em></span><span style="--bar: 86%"><strong>PIPA audit</strong><em>vendor gated</em></span></div></article><article class="governance-command-card graph-route-card"><div class="governance-card-head"><div><p class="screen-anchor">ROUTE MATRIX</p><h5>Open connected product surface</h5></div></div><div class="governance-route-grid" aria-label="Governance route matrix"><button type="button" data-governance-action="route-workflow" data-governance-route="workflow">Workflow</button><button type="button" data-governance-action="route-mail" data-governance-route="mail">Mail</button><button type="button" data-governance-action="route-community" data-governance-route="community">Community</button><button type="button" data-governance-action="route-finance" data-governance-route="finance">Finance</button><button type="button" data-governance-action="route-cloud" data-governance-route="cloud">Cloud Ops</button><button type="button" data-governance-action="route-identity" data-governance-route="identity">Identity</button><button type="button" data-governance-action="route-evidence" data-governance-route="evidence">Evidence</button><button type="button" data-governance-action="route-catalog" data-governance-route="catalog">Catalog</button></div><dl class="governance-kv"><div><dt>Graph root</dt><dd>{workflow_name}</dd></div><div><dt>Autonomy ceiling</dt><dd>No auto-approval · visual review only</dd></div></dl></article></div><div class="governance-lower-deck"><article class="governance-command-card control-attestation-card"><div class="governance-card-head"><div><p class="screen-anchor">CONTROL ATTESTATION</p><h5>Controls with overdue evidence</h5></div><span class="status-chip warning">3 due</span></div><div class="gov-attestation-list"><button type="button" data-governance-action="attest-payroll" data-governance-route="workflow"><strong>CTRL-PAY-09 · payroll 4-eyes</strong><span style="--bar: 82%"></span><em>82% · CFO attestation waiting</em></button><button type="button" data-governance-action="attest-pipa" data-governance-route="identity"><strong>CTRL-PIPA-03 · retention boundary</strong><span style="--bar: 68%"></span><em>68% · vendor visibility review</em></button><button type="button" data-governance-action="attest-cloud" data-governance-route="cloud"><strong>CTRL-CLOUD-12 · rollback runbook</strong><span style="--bar: 74%"></span><em>74% · regional evidence stale</em></button></div></article><article class="governance-command-card audit-chain-card"><div class="governance-card-head"><div><p class="screen-anchor">AUDIT CHAIN</p><h5>Immutable receipt lineage</h5></div><button type="button" data-governance-action="route-evidence" data-governance-route="evidence">Evidence</button></div><ol class="gov-audit-chain"><li><span>01</span><strong>Workflow run</strong><em>hash 5bf7…91</em></li><li><span>02</span><strong>Mail approval</strong><em>hash a81d…0c</em></li><li><span>03</span><strong>Community note</strong><em>hash 44c2…bf</em></li><li><span>04</span><strong>Board packet</strong><em>draft</em></li></ol></article><article class="governance-command-card board-cycle-card"><div class="governance-card-head"><div><p class="screen-anchor">BOARD CYCLE</p><h5>Resolution timeline and quorum</h5></div><span class="status-chip success">3 / 5 ack</span></div><div class="gov-board-timeline" aria-label="Board cycle timeline"><span class="done"><strong>Draft</strong><em>May 04</em></span><span class="active"><strong>Review</strong><em>May 07</em></span><span><strong>Send</strong><em>May 10</em></span><span><strong>Vote</strong><em>May 12</em></span></div></article></div></section>"#,
        workflow_name = escape(workflow_name),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_business_logic_os_panel() -> String {
    let rows = BUSINESS_LOGIC_ROWS
        .iter()
        .map(static_business_logic_row)
        .collect::<String>();

    format!(
        r#"<section class="business-logic-os" aria-label="Business logic operating system"><div class="logic-os-kpis" aria-label="Business logic summary"><article class="logic-os-kpi"><span>Active logics</span><strong>17</strong><small>7 visible in this envelope</small></article><article class="logic-os-kpi"><span>P0 critical</span><strong>4</strong><small>cannot fail silently</small></article><article class="logic-os-kpi warn"><span>Need attention</span><strong>3</strong><small>blocked or at-risk</small></article><article class="logic-os-kpi accent"><span>Real cost / month</span><strong>₩9.2M</strong><small>hard + soft + delay</small></article><article class="logic-os-kpi"><span>Annualized</span><strong>₩110M</strong><small>if cadence holds</small></article></div><div class="logic-os-toolbar" aria-label="Business logic filters"><label class="logic-os-search"><span aria-hidden="true">⌕</span><input data-logic-search="true" type="search" aria-label="Search business logics" placeholder="Search logic, owner, route, evidence..." /></label><div class="logic-os-segments" role="toolbar" aria-label="Logic category filters"><button type="button" class="active" data-logic-filter="all">All</button><button type="button" data-logic-filter="workforce">Workforce</button><button type="button" data-logic-filter="finance">Finance</button><button type="button" data-logic-filter="compliance">Compliance</button><button type="button" data-logic-filter="trust">Trust</button><button type="button" data-logic-filter="cloud">Cloud</button><button type="button" data-logic-filter="attention">Attention</button></div><span class="logic-os-status" data-logic-status="true"><strong data-logic-visible-count="true">{row_count}</strong> visible · all categories · local only</span></div><div class="logic-os-layout"><div class="logic-table-shell" role="region" aria-label="Business logic catalog"><table class="logic-os-table"><thead><tr><th>Health</th><th>Logic</th><th>Category</th><th>Owner</th><th>Cadence</th><th>Crit.</th><th>Cost/run</th><th>SLA</th><th>Tasks</th><th>Action</th></tr></thead><tbody>{rows}</tbody></table></div><aside class="logic-os-rail" aria-label="Business logic dependency and evidence rail"><div class="logic-rail-card"><p class="screen-anchor">DEPENDENCY MAP</p><strong>Payroll anomaly → Workflow → Messenger/Mail → Audit</strong><div class="logic-dependency-map" aria-hidden="true"><span>HR</span><i></i><span>Payroll</span><i></i><span>Workflow</span><i></i><span>Mail</span><i></i><span>Audit</span></div><div class="logic-rail-actions"><button type="button" data-logic-graph-action="workflow">Workflow</button><button type="button" data-logic-graph-action="mail">Mail brief</button><button type="button" data-logic-graph-action="catalog">Catalog</button><button type="button" data-logic-graph-action="audit">Evidence</button></div></div><div class="logic-rail-card matrix"><p class="screen-anchor">COST × HEALTH</p><div class="logic-matrix" aria-label="Cost by health preview"><span class="dot danger" style="--x: 78%; --y: 20%" title="Tenant network split"></span><span class="dot warn" style="--x: 54%; --y: 38%" title="Payroll close"></span><span class="dot warn" style="--x: 42%; --y: 52%" title="Vendor renewal"></span><span class="dot ok" style="--x: 24%; --y: 70%" title="New hire onboarding"></span><span class="dot done" style="--x: 12%; --y: 84%" title="Governance council note"></span></div></div></aside></div></section>"#,
        row_count = BUSINESS_LOGIC_ROWS.len(),
        rows = rows
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_business_logic_row(row: &BusinessLogicRow) -> String {
    let criticality_class = format!("crit crit-{}", row.criticality);
    let task_class = if row.tasks == "0" {
        "tasks-cell"
    } else {
        "tasks-cell has-open"
    };

    format!(
        r#"<tr class="logic-os-row" data-logic-row="true" data-logic-category="{category}" data-logic-state="{state}"><td><span class="logic-health-dot {state}" aria-label="{state_label}"></span></td><td><button type="button" class="logic-name-button" data-sidepeek-trigger="business-logic" data-sidepeek-title="{name}" data-sidepeek-id="{id}" data-sidepeek-desc="{description}" data-sidepeek-owner="{owner}" data-sidepeek-risk="{state_label}" data-sidepeek-sla="{sla}">{name}</button><div class="logic-code">{id} · {english_name}</div></td><td><span class="cat-tag">{category}</span></td><td><span class="owner-cell"><span class="avatar-xs" aria-hidden="true">{initials}</span>{owner}</span></td><td class="mono-cell">{cadence}</td><td><span class="{criticality_class}">{criticality}</span></td><td class="cost-cell">{cost}</td><td><span class="{sla_class}">{sla}</span></td><td><span class="{task_class}">{tasks}</span></td><td><span class="logic-row-actions"><button type="button" data-logic-action="open" data-logic-target="{route}">Open</button><button type="button" data-logic-action="run">Run preview</button></span></td></tr>"#,
        category = escape(row.category),
        state = escape(row.state),
        state_label = escape(row.state_label),
        name = escape(row.name),
        id = escape(row.id),
        description = escape(row.description),
        owner = escape(row.owner),
        sla = escape(row.sla),
        english_name = escape(row.english_name),
        initials = escape(logic_owner_initials(row.owner)),
        cadence = escape(row.cadence),
        criticality_class = escape(&criticality_class),
        criticality = escape(row.criticality),
        cost = escape(row.cost),
        sla_class = escape(logic_sla_class(row.state)),
        task_class = escape(task_class),
        tasks = escape(row.tasks),
        route = escape(row.route),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_identity_command_board() -> &'static str {
    r#"<div class="identity-command-board" aria-label="Identity command center">
  <section class="identity-command-card identity-spine-card">
    <div class="identity-command-card-head"><div><p class="screen-anchor">ACCESS SPINE</p><h5>One governed identity path from auth to payroll close</h5></div><span class="status-chip success">PIPA-safe</span></div>
    <div class="identity-spine-flow" aria-label="Identity access lineage"><span class="active"><em>01</em><strong>Passkey</strong><small>verified</small></span><i></i><span><em>02</em><strong>Session</strong><small>current device</small></span><i></i><span class="review"><em>03</em><strong>Role</strong><small>payroll review</small></span><i></i><span><em>04</em><strong>Employee</strong><small>118 records</small></span><i></i><span class="review"><em>05</em><strong>Workflow</strong><small>2-person gate</small></span><i></i><span class="sealed"><em>06</em><strong>Audit</strong><small>REC-ID-2026-05</small></span></div>
    <dl class="identity-command-kv"><div><dt>Autonomy ceiling</dt><dd>No auth mutation · local preview only</dd></div><div><dt>Primary risk</dt><dd>Payroll approver role expires before close</dd></div><div><dt>Connected route</dt><dd>Workflow → Mail → Evidence Spine</dd></div></dl>
  </section>
  <section class="identity-command-card identity-risk-card">
    <div class="identity-command-card-head"><div><p class="screen-anchor">RISK QUEUE</p><h5>Access work that affects today’s operations</h5></div><button type="button" data-identity-route-action="evidence">Evidence</button></div>
    <div class="identity-risk-list" role="list" aria-label="Identity risk queue"><article role="listitem" data-identity-risk-row="review"><span class="status-chip warning">review</span><strong>Payroll approver recertification</strong><p>CFO role grants payroll close and HomeTax transport; 2-person review due today.</p><small>Owner CFO · SLA 4.0h · REC-ID-2026-05</small></article><article role="listitem" data-identity-risk-row="blocking"><span class="status-chip danger">blocking</span><strong>Vendor guest cannot view employee PII</strong><p>Stripe renewal route needs procurement context without exposing workforce records.</p><small>Owner Security reviewer · policy POL-PII-014</small></article><article role="listitem" data-identity-risk-row="sealed"><span class="status-chip success">sealed</span><strong>Passkey challenge evidence sealed</strong><p>MacBook and iPhone passkey state available to audit, not external auth writes.</p><small>Source local island · 09:14 KST</small></article></div>
  </section>
  <section class="identity-command-card identity-lifecycle-card">
    <div class="identity-command-card-head"><div><p class="screen-anchor">WORKFORCE LIFECYCLE</p><h5>Employees, onboarding, roles, and payroll impact</h5></div><button type="button" data-identity-route-action="employees">Open people</button></div>
    <div class="identity-lifecycle-grid" aria-label="Workforce lifecycle state"><span style="--bar: 77%"><strong>Onboarding</strong><em>6 active · 77%</em></span><span style="--bar: 64%"><strong>Role review</strong><em>14 grants · 64%</em></span><span style="--bar: 48%"><strong>Session hygiene</strong><em>3 stale · 48%</em></span><span style="--bar: 83%"><strong>Payroll readiness</strong><em>109 active · 83%</em></span></div>
  </section>
  <section class="identity-command-card identity-route-card">
    <div class="identity-command-card-head"><div><p class="screen-anchor">ROUTE MATRIX</p><h5>Every identity action lands inside the same service graph</h5></div></div>
    <div class="identity-route-grid" aria-label="Identity local routes"><button type="button" data-identity-route-action="workflow">Workflow gate</button><button type="button" data-identity-route-action="mail">Mail reviewer</button><button type="button" data-identity-route-action="sessions">Session audit</button><button type="button" data-identity-route-action="onboarding">Setup checklist</button><button type="button" data-identity-route-action="finance">Payroll close</button><button type="button" data-identity-route-action="evidence">Evidence spine</button></div>
    <p class="identity-command-note">Routes change local visual state only; no SSO, HRIS, payroll, or directory backend is wired.</p>
  </section>
</div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_workforce_anchor_board() -> &'static str {
    r#"<div class="workforce-anchor-grid" aria-label="FD-001 and Oyatie Cloud workforce proof"><article class="workforce-anchor-card selected" data-workforce-card="fd001"><p class="screen-anchor">FD-001 WORKFORCE</p><h5>People data powers product delivery</h5><p>Employee directory, payroll eligibility, reviewer mail, community announcements, and onboarding are FD-001 tenant workloads, not separate HR widgets.</p><div class="workforce-anchor-actions"><button type="button" data-workforce-anchor-action="route-payroll">Payroll impact</button><button type="button" data-workforce-anchor-action="route-workflow">Workflow path</button></div></article><article class="workforce-anchor-card" data-workforce-card="cloud"><p class="screen-anchor">OYATIE CLOUD</p><h5>Hosted as a governed tenant surface</h5><p>PIPA boundaries, regional pack gates, role envelopes, audit receipts, and evidence routes prove the substrate can host real workforce tenants.</p><div class="workforce-anchor-actions"><button type="button" data-workforce-anchor-action="route-policy">Policy envelope</button><button type="button" data-workforce-anchor-action="route-audit">Audit trail</button></div></article><article class="workforce-anchor-card" data-workforce-card="lifecycle"><p class="screen-anchor">LIFECYCLE OPS</p><h5>Interactive, local-only employee command</h5><p>Operators can inspect people, stage invites, route leave/time, and brief reviewers while HRIS, auth, payroll, and cloud mutations remain disconnected.</p><div class="workforce-anchor-actions"><button type="button" data-workforce-anchor-action="stage-invite">Stage invite</button><button type="button" data-workforce-anchor-action="route-leave">Leave &amp; time</button></div></article></div><div class="workforce-anchor-footer"><span data-workforce-anchor-status="true">Employees ready · FD-001 workforce workload dogfoods Oyatie Cloud locally.</span><div class="workforce-anchor-routes" aria-label="Workforce connected routes"><button type="button" data-workforce-anchor-action="route-mail">Reviewer Mail</button><button type="button" data-workforce-anchor-action="route-community">Community update</button><button type="button" data-workforce-anchor-action="route-evidence">Evidence graph</button><button type="button" data-workforce-anchor-action="route-cloud">Cloud cells</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_onboarding_anchor_board() -> &'static str {
    r#"<div class="onboarding-anchor-grid" aria-label="FD-001 tenant admission setup proof"><article class="onboarding-anchor-card selected" data-onboarding-card="tenant"><p class="screen-anchor">FD-001 TENANT ADMISSION</p><h5>Product workload setup path</h5><p>Legal profile, payroll calendar, employee import, policy gates, Mail reviewers, Community launch notes, and evidence receipts become one tenant setup packet.</p><div class="onboarding-anchor-actions"><button type="button" data-onboarding-anchor-action="route-tasks">Today queue</button><button type="button" data-onboarding-anchor-action="import-employees">Import people</button></div></article><article class="onboarding-anchor-card" data-onboarding-card="cloud"><p class="screen-anchor">OYATIE CLOUD</p><h5>Substrate readiness before go-live</h5><p>Region pack, PIPA boundary, role envelope, deployment gates, audit freshness, and rollback posture prove the tenant can be hosted safely.</p><div class="onboarding-anchor-actions"><button type="button" data-onboarding-anchor-action="route-cloud">Cloud cells</button><button type="button" data-onboarding-anchor-action="route-policy">Policy gate</button></div></article><article class="onboarding-anchor-card" data-onboarding-card="launch"><p class="screen-anchor">LAUNCH PACKET</p><h5>Interactive, local-only setup</h5><p>Operators can advance setup, draft reviewer mail, post a community note, and attach evidence while registries, HRIS, payroll, auth, and cloud mutations remain disconnected.</p><div class="onboarding-anchor-actions"><button type="button" data-onboarding-anchor-action="advance-setup">Advance setup</button><button type="button" data-onboarding-anchor-action="route-evidence">Evidence</button></div></article></div><div class="onboarding-anchor-footer"><span data-onboarding-anchor-status="true">Onboarding ready · FD-001 tenant setup dogfoods Oyatie Cloud locally.</span><div class="onboarding-anchor-routes" aria-label="Onboarding connected routes"><button type="button" data-onboarding-anchor-action="route-payroll">Payroll calendar</button><button type="button" data-onboarding-anchor-action="route-mail">Reviewer Mail</button><button type="button" data-onboarding-anchor-action="route-community">Community launch</button><button type="button" data-onboarding-anchor-action="route-schedule">Schedule</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_identity_sessions_anchor_board() -> &'static str {
    r#"<div class="identity-anchor-grid identity-sessions-anchor" aria-label="Session tenant proof"><article class="identity-anchor-card selected" data-identity-anchor-card="sessions"><p class="screen-anchor">FD-001 SESSION PROOF</p><h5>Auth sessions protect product workloads</h5><p>Passkey, device, and payroll-role activity are evidence leaves for FD-001 tenant services.</p><div class="identity-anchor-actions"><button type="button" data-identity-anchor-action="route-roles">Role envelope</button><button type="button" data-identity-anchor-action="route-evidence">Evidence graph</button></div></article><article class="identity-anchor-card" data-identity-anchor-card="cloud"><p class="screen-anchor">OYATIE CLOUD</p><h5>Oyatie Cloud tenant session posture</h5><p>Device locality, PIPA-safe audit, and session freshness prove the substrate can host workforce tenants.</p><div class="identity-anchor-actions"><button type="button" data-identity-anchor-action="route-cloud">Cloud cells</button><button type="button" data-identity-anchor-action="route-policy">PIPA policy</button></div></article></div><div class="identity-anchor-footer"><span data-identity-anchor-status="true">Sessions ready · local-only identity telemetry.</span><div class="identity-anchor-routes"><button type="button" data-identity-anchor-action="route-mail">Reviewer Mail</button><button type="button" data-identity-anchor-action="route-audit">Audit ledger</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_identity_roles_anchor_board() -> &'static str {
    r#"<div class="identity-anchor-grid identity-roles-anchor" aria-label="Role envelope proof"><article class="identity-anchor-card selected" data-identity-anchor-card="roles"><p class="screen-anchor">FD-001 ROLE ENVELOPE</p><h5>Access controls every product workload</h5><p>Payroll, filing, workflow, Mail, Community, and cloud operations share one role envelope.</p><div class="identity-anchor-actions"><button type="button" data-identity-anchor-action="review-roles">Review grants</button><button type="button" data-identity-anchor-action="route-workflow">Workflow gate</button></div></article><article class="identity-anchor-card" data-identity-anchor-card="pipa"><p class="screen-anchor">OYATIE CLOUD POLICY</p><h5>Oyatie Cloud PIPA-safe tenant boundary</h5><p>Role decisions stay auditable before any tenant workload can move through cloud admission gates.</p><div class="identity-anchor-actions"><button type="button" data-identity-anchor-action="route-policy">Policy board</button><button type="button" data-identity-anchor-action="route-cloud">Cloud gate</button></div></article><article class="identity-anchor-card" data-identity-anchor-card="local"><p class="screen-anchor">LOCAL ONLY</p><h5>Interactive access preview</h5><p>Grant reviews, denial traces, and reviewer routes update visual state only; no SSO or IAM mutation runs.</p><div class="identity-anchor-actions"><button type="button" data-identity-anchor-action="route-evidence">Evidence</button><button type="button" data-identity-anchor-action="route-community">Community note</button></div></article></div><div class="identity-anchor-footer"><span data-identity-anchor-status="true">Roles ready · FD-001 access envelope dogfoods Oyatie Cloud locally.</span><div class="identity-anchor-routes"><button type="button" data-identity-anchor-action="route-payroll">Payroll close</button><button type="button" data-identity-anchor-action="route-audit">Audit packet</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_identity_org_anchor_board() -> &'static str {
    r#"<div class="identity-anchor-grid identity-org-anchor" aria-label="Organization tenant proof"><article class="identity-anchor-card selected" data-identity-anchor-card="org"><p class="screen-anchor">FD-001 ORG PROFILE</p><h5>Corporate facts feed every module</h5><p>Legal profile, payroll calendar, tax identifiers, billing, and employee facts become shared tenant context.</p><div class="identity-anchor-actions"><button type="button" data-identity-anchor-action="route-onboarding">Setup packet</button><button type="button" data-identity-anchor-action="route-payroll">Payroll calendar</button></div></article><article class="identity-anchor-card" data-identity-anchor-card="cloud"><p class="screen-anchor">OYATIE CLOUD TENANT</p><h5>Oyatie Cloud hosted profile readiness</h5><p>Region packs, audit receipts, deployment gates, and evidence spine prove this tenant can be hosted.</p><div class="identity-anchor-actions"><button type="button" data-identity-anchor-action="route-cloud">Cloud cells</button><button type="button" data-identity-anchor-action="route-evidence">Evidence spine</button></div></article></div><div class="identity-anchor-footer"><span data-identity-anchor-status="true">Organization ready · local-only tenant profile preview.</span><div class="identity-anchor-routes"><button type="button" data-identity-anchor-action="route-mail">Reviewer Mail</button><button type="button" data-identity-anchor-action="route-community">Community launch</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_identity_workforce_service() -> String {
    format!(
        r#"<section id="identity-workforce-service" class="identity-workforce-service panel" data-identity-service="true" aria-labelledby="identity-service-title">
  <div class="identity-service-head">
    <div><p class="screen-anchor">SETTINGS · WORKFORCE</p><h3 id="identity-service-title">Identity, organization profile, onboarding, and employees</h3></div>
    <div class="identity-service-actions"><span class="status-chip success" data-identity-status="true">local profile ready</span><button type="button" data-identity-action="open-audit">Audit log</button></div>
  </div>
  <div class="identity-service-shell">
    <aside class="identity-settings-rail" aria-label="Identity and organization sections">
      <div class="identity-person"><span aria-hidden="true">최</span><div><strong>최유나 · Choi Yu-na</strong><small>Admin · Oyatie Corp.</small></div></div>
      <p class="screen-anchor">ACCOUNT</p>
      <button type="button" class="active" data-identity-tab="auth">패스키 · MFA</button>
      <button type="button" data-identity-tab="sessions">세션 · 기기</button>
      <button type="button" data-identity-tab="roles">역할 · 권한</button>
      <p class="screen-anchor">WORKSPACE</p>
      <button type="button" data-identity-tab="org">조직 프로필</button>
      <button type="button" data-identity-tab="employees">구성원</button>
      <button type="button" data-identity-tab="onboarding">워크스페이스 설정</button>
      <p class="identity-chain">Oyatie v0.1 · chain 0x4f81 · last sync 09:14 KST</p>
    </aside>
    <div class="identity-service-main">
      <div class="identity-tabs" role="tablist" aria-label="Identity service views">
        <button type="button" class="active" data-identity-tab="auth" role="tab" aria-selected="true">Auth</button>
        <button type="button" data-identity-tab="sessions" role="tab" aria-selected="false">Sessions</button>
        <button type="button" data-identity-tab="roles" role="tab" aria-selected="false">Roles</button>
        <button type="button" data-identity-tab="org" role="tab" aria-selected="false">Org profile</button>
        <button type="button" data-identity-tab="employees" role="tab" aria-selected="false">Employees</button>
        <button type="button" data-identity-tab="onboarding" role="tab" aria-selected="false">Onboarding</button>
      </div>
      <article id="identity-auth" class="identity-panel active" data-identity-panel="auth" aria-labelledby="auth-panel-title">
        <div class="identity-panel-copy"><p class="screen-anchor">ACCOUNT · PASSKEYS</p><h4 id="auth-panel-title">패스키 · MFA</h4><p>가능한 모든 기기에 패스키를 등록하면 더 빠르고 안전하게 로그인합니다.</p></div>
        <div class="auth-grid">
          <div class="auth-method-list" data-passkey-list="true">
            <div class="auth-method"><span>⌘</span><strong>MacBook Pro · Touch ID</strong><small>passkey · macOS · Apple · PRIMARY</small><em>방금 전</em></div>
            <div class="auth-method"><span>◉</span><strong>iPhone 15 Pro · Face ID</strong><small>passkey · iOS · Apple</small><em>12 hours ago</em></div>
            <div class="auth-method"><span>▣</span><strong>Authy · personal phone</strong><small>totp · added 2025-11-03</small><em>3 weeks ago</em></div>
            <div class="auth-method"><span>⌁</span><strong>Recovery codes (10)</strong><small>recovery · printed · 10 unused</small><em>never</em></div>
          </div>
          <aside class="security-score-card"><p class="screen-anchor">SECURITY SCORE</p><strong data-security-score="true">94/100</strong><span class="score-bar" style="--bar: 94%"><em></em></span><ol><li>✓ 패스키 2개 등록됨</li><li>✓ TOTP 백업 활성화</li><li>✓ 복구 코드 미사용</li><li>○ Apple Watch 패스키 미등록</li></ol><button type="button" data-identity-action="add-passkey">+ 패스키 추가</button></aside>
        </div>
        {identity_command_board}
      </article>
      <article id="identity-sessions" class="identity-panel" data-identity-panel="sessions" aria-labelledby="sessions-panel-title">
        <div class="identity-panel-copy"><p class="screen-anchor">ACCOUNT · SESSIONS</p><h4 id="sessions-panel-title">세션 · 기기</h4><p>현재 로그인한 기기, 최근 활동, 의심 신호를 한 화면에서 확인합니다.</p></div>
        <div class="identity-session-grid">
          <article class="session-card primary"><span class="device-glyph" aria-hidden="true">⌘</span><div><strong>MacBook Pro</strong><small>Chrome · Seoul · current session</small></div><em>방금 전</em></article>
          <article class="session-card"><span class="device-glyph" aria-hidden="true">◉</span><div><strong>iPhone 15 Pro</strong><small>Safari · Seoul · passkey verified</small></div><em>12 hours ago</em></article>
          <article class="session-card"><span class="device-glyph" aria-hidden="true">▣</span><div><strong>Edge on Windows</strong><small>Finance office · remembered device</small></div><em>3 days ago</em></article>
        </div>
        <ol class="identity-audit-log"><li><time>09:14</time><strong>New passkey challenge passed</strong><span>MFA guardrail OK</span></li><li><time>08:42</time><strong>Payroll role inspected</strong><span>No write mutation</span></li><li><time>Yesterday</time><strong>Recovery codes viewed</strong><span>Admin acknowledgement required</span></li></ol>
        {identity_sessions_anchor_board}
      </article>
      <article id="identity-roles" class="identity-panel" data-identity-panel="roles" aria-labelledby="roles-panel-title">
        <div class="identity-panel-copy"><p class="screen-anchor">ACCOUNT · ACCESS</p><h4 id="roles-panel-title">역할 · 권한</h4><p>워크플로우, 급여, 감사, 직원 정보 접근이 어떤 근거로 허용되는지 표시합니다.</p></div>
        <table class="role-matrix-table"><thead><tr><th>역할</th><th>범위</th><th>결정</th><th>근거</th></tr></thead><tbody><tr><td><strong>Tenant Admin</strong><small>owner</small></td><td>Workspace · billing · users</td><td><span class="status-chip success">Allow</span></td><td>법인 관리자</td></tr><tr><td><strong>Payroll Approver</strong><small>finance</small></td><td>Payroll close · filing</td><td><span class="status-chip warning">Review</span></td><td>2-person approval</td></tr><tr><td><strong>Workflow Builder</strong><small>studio</small></td><td>Draft · simulate</td><td><span class="status-chip success">Allow</span></td><td>No live execution</td></tr><tr><td><strong>External Vendor</strong><small>guest</small></td><td>Employee PII</td><td><span class="status-chip danger">Deny</span></td><td>PIPA boundary</td></tr></tbody></table>
        {identity_roles_anchor_board}
      </article>
      <article id="identity-org" class="identity-panel" data-identity-panel="org" aria-labelledby="org-panel-title">
        <div class="identity-panel-copy"><p class="screen-anchor">SETTINGS · OVERVIEW</p><h4 id="org-panel-title">조직 프로필</h4><p>모든 핵심 설정을 한 곳에서 관리하고 변경사항은 감사 체인에 자동 기록됩니다.</p></div>
        <div class="org-stat-grid"><span><small>임직원</small><strong>118명</strong><em>▲ +6 last quarter</em></span><span><small>월 인건비</small><strong>₩894,000,000</strong><em>▲ +4.2% MoM</em></span><span><small>활성 워크플로우</small><strong>42개</strong><em>▲ +3 since launch</em></span><span><small>미해결 행동항목</small><strong>7건</strong><em>▼ −2 this week</em></span></div>
        <div class="org-profile-grid"><dl><dt>법인명</dt><dd>오야티 주식회사</dd><dt>사업자등록번호</dt><dd>123-45-67890</dd><dt>대표자</dt><dd>이재현</dd><dt>본점</dt><dd>서울 강남구 테헤란로 521 12층</dd></dl><dl><dt>주거래</dt><dd>신한은행 · 주식회사 오야티</dd><dt>출금 항목</dt><dd>급여 · 원천세 · 4대보험</dd><dt>검증 상태</dt><dd>✓ 1원 검증 완료</dd><dt>결제 카드</dt><dd>신한카드 ****-4081</dd></dl><dl><dt>주기</dt><dd>월급</dd><dt>지급일</dt><dd>매월 25일</dd><dt>근태 마감</dt><dd>3일 전 (22일)</dd><dt>다음 마감</dt><dd>2026-05-22 (금)</dd></dl><dl><dt>국민연금</dt><dd>12345678901</dd><dt>건강보험</dt><dd>234567890</dd><dt>고용보험</dt><dd>EI-2024-0091</dd><dt>산재보험</dt><dd>WCI-2024-0091 · 0.65%</dd></dl></div>
        {identity_org_anchor_board}
      </article>
      <article id="identity-employees" class="identity-panel" data-identity-panel="employees" aria-labelledby="employees-panel-title">
        <div class="identity-panel-copy"><p class="screen-anchor">SETTINGS · EMPLOYEES</p><h4 id="employees-panel-title">직원 디렉토리</h4><p>FD-001 people, payroll, policy, Mail, and Community workloads stay product-first while Oyatie Cloud proves identity data can be hosted as a governed tenant surface.</p></div>
        <div class="employee-directory-stats"><span><small>전체</small><strong>118명</strong></span><span><small>활성</small><strong>109명</strong></span><span><small>최근 30일 입사</small><strong>6명</strong></span><span><small>수습 종료 임박</small><strong>5명</strong></span></div>
        {workforce_anchor_board}
        <div class="employee-directory-tools"><label><span aria-hidden="true">⌕</span><input data-employee-search="true" aria-label="Search employees" placeholder="이름, 직책, 팀, ID 검색..." /></label><div class="employee-filter-pills" aria-label="Employee filters"><button type="button" class="active" data-employee-filter="all">활성 109</button><button type="button" data-employee-filter="infrastructure">플랫폼팀</button><button type="button" data-employee-filter="finance">Finance</button></div><button type="button" data-identity-action="add-employee">+ 직원 추가</button></div>
        <table class="employee-directory-table"><thead><tr><th>이름</th><th>직책</th><th>부서 · 팀</th><th>매니저</th><th>입사일</th><th>상태</th><th>Action</th></tr></thead><tbody><tr data-employee-row="true" data-employee-team="office"><td><strong>이재현</strong><small>Jaehyun Lee · emp_0000</small></td><td>Chief Executive Officer</td><td>Office of CEO</td><td>—</td><td>2021-03-14</td><td><span class="status-chip success">활성</span></td><td><button type="button" data-employee-action="inspect">Inspect</button></td></tr><tr data-employee-row="true" data-employee-team="finance"><td><strong>최유나</strong><small>Yuna Choi · emp_0011</small></td><td>Chief Financial Officer</td><td>Finance</td><td>이재현</td><td>2022-04-01</td><td><span class="status-chip success">활성</span></td><td><button type="button" data-employee-action="inspect">Inspect</button></td></tr><tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>박서준</strong><small>Seojun Park · emp_0001</small></td><td>VP of Engineering</td><td>플랫폼팀</td><td>이재현</td><td>2021-06-01</td><td><span class="status-chip success">활성</span></td><td><button type="button" data-employee-action="inspect">Inspect</button></td></tr><tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>김지영</strong><small>Jiyoung Kim · emp_0002</small></td><td>Engineering Manager</td><td>플랫폼팀</td><td>박서준</td><td>2022-09-12</td><td><span class="status-chip success">활성</span></td><td><button type="button" data-employee-action="inspect">Inspect</button></td></tr><tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>윤태민</strong><small>Taemin Yoon · emp_0003</small></td><td>Senior Software Engineer</td><td>플랫폼팀</td><td>김지영</td><td>2026-05-12</td><td><span class="status-chip warning">수습</span></td><td><button type="button" data-employee-action="inspect">Inspect</button></td></tr><tr data-employee-row="true" data-employee-team="product"><td><strong>임도윤</strong><small>Doyun Lim · emp_0004</small></td><td>Engineering Manager · Product</td><td>프로덕트팀</td><td>박서준</td><td>2022-11-04</td><td><span class="status-chip success">활성</span></td><td><button type="button" data-employee-action="inspect">Inspect</button></td></tr><tr data-employee-row="true" data-employee-team="product"><td><strong>강수아</strong><small>Sua Kang · emp_0005</small></td><td>Software Engineer</td><td>프로덕트팀</td><td>임도윤</td><td>2024-02-19</td><td><span class="status-chip">휴직</span></td><td><button type="button" data-employee-action="inspect">Inspect</button></td></tr><tr data-employee-row="true" data-employee-team="data"><td><strong>정우진</strong><small>Woojin Jung · emp_0006</small></td><td>Software Engineer · Frontend</td><td>데이터팀</td><td>임도윤</td><td>2025-08-07</td><td><span class="status-chip success">활성</span></td><td><button type="button" data-employee-action="inspect">Inspect</button></td></tr></tbody></table>
      </article>
      <article id="identity-onboarding" class="identity-panel" data-identity-panel="onboarding" aria-labelledby="onboarding-panel-title">
        <div class="identity-panel-copy"><p class="screen-anchor">WORKSPACE SETUP</p><h4 id="onboarding-panel-title">워크스페이스 설정</h4><p>Workspace setup is the tenant-admission path: FD-001 product workloads collect legal, payroll, policy, schedule, and evidence facts before Oyatie Cloud hosts the tenant.</p></div>
        {onboarding_anchor_board}
        <div class="onboarding-flow"><div class="onboarding-progress"><strong data-onboarding-percent="true">56%</strong><span class="score-bar" style="--bar: 56%"><em></em></span><button type="button" data-identity-action="advance-onboarding">다음 단계 완료</button></div><article class="setup-current-step"><p class="screen-anchor">CURRENT STEP</p><h5>급여 캘린더 확인</h5><p>지급일, 근태 마감, 원천세 신고 마감이 이번 달 워크플로우와 일치하는지 검토합니다.</p><ul><li>✓ 법인 정보 검증 완료</li><li>✓ 은행 출금 계좌 검증 완료</li><li>○ 캘린더 승인 대기</li></ul></article><ol class="onboarding-steps"><li class="done">1 환영합니다</li><li class="done">2 법인 정보</li><li class="done">3 은행 · 결제</li><li class="active">4 급여 캘린더</li><li>5 4대보험 가입</li><li>6 관할</li><li>7 직원 가져오기</li><li>8 보존 · 정책</li><li>9 검토 · 가동</li></ol></div>
      </article>
    </div>
    <aside class="identity-context-rail" aria-label="Identity provenance">
      <p class="screen-anchor">PROVENANCE</p><dl class="service-kv"><div><dt>Actor</dt><dd>Choi Yu-na · Admin</dd></div><div><dt>Scope</dt><dd>Auth · org · people</dd></div><div><dt>Receipt</dt><dd>REC-ID-2026-05</dd></div></dl>
      <p class="screen-anchor">LOCAL ONLY</p><ol class="notification-stack"><li>No real auth mutation</li><li>No external HR system write</li><li>Audit preview staged visually</li></ol>
    </aside>
  </div>
</section>"#,
        identity_command_board = static_identity_command_board(),
        identity_sessions_anchor_board = static_identity_sessions_anchor_board(),
        identity_roles_anchor_board = static_identity_roles_anchor_board(),
        identity_org_anchor_board = static_identity_org_anchor_board(),
        workforce_anchor_board = static_workforce_anchor_board(),
        onboarding_anchor_board = static_onboarding_anchor_board(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_finance_vendors_anchor_board() -> &'static str {
    r#"<div class="finance-anchor-grid" aria-label="FD-001 vendor spend and Oyatie Cloud tenant proof"><article class="finance-anchor-card selected" data-finance-anchor-card="vendors-fd001"><p class="screen-anchor">FD-001 VENDOR WORKLOAD</p><h5>Procurement is part of product delivery</h5><p>Vendor approvals, spend controls, Workflow tasks, Mail briefs, and Community notes are FD-001 tenant workloads sharing one commercial graph.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="stage-contract">Stage contract</button><button type="button" data-finance-anchor-action="route-workflow">Workflow gate</button></div></article><article class="finance-anchor-card" data-finance-anchor-card="vendors-cloud"><p class="screen-anchor">OYATIE CLOUD FINOPS</p><h5>Cloud substrate proves tenant spend posture</h5><p>Oyatie Cloud hosts FD-001 services as real tenant workloads while FinOps, policy, audit, and regional gates stay visible before production claims.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="route-cloud">Open FinOps</button><button type="button" data-finance-anchor-action="route-policy">Policy envelope</button></div></article><article class="finance-anchor-card" data-finance-anchor-card="vendors-local"><p class="screen-anchor">LOCAL-ONLY RAIL</p><h5>Interactive procurement preview</h5><p>Operators can inspect Stripe, AWS Korea, and bank transport paths; no bank, payroll, tax, billing, vendor, or cloud mutation executes.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="route-audit">Audit trail</button><button type="button" data-finance-anchor-action="route-mail">Reviewer Mail</button></div></article></div><div class="finance-anchor-footer"><span data-finance-anchor-status="true">Vendors ready · FD-001 procurement workload dogfoods Oyatie Cloud locally.</span><div class="finance-anchor-routes" aria-label="Vendor connected routes"><button type="button" data-finance-anchor-action="route-ledger">Ledger</button><button type="button" data-finance-anchor-action="route-billing">Billing</button><button type="button" data-finance-anchor-action="route-community">Community</button><button type="button" data-finance-anchor-action="route-evidence">Evidence</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_finance_billing_anchor_board() -> &'static str {
    r#"<div class="finance-anchor-grid" aria-label="FD-001 billing tax and Oyatie Cloud tenant proof"><article class="finance-anchor-card selected" data-finance-anchor-card="billing-fd001"><p class="screen-anchor">FD-001 REVENUE WORKLOAD</p><h5>Billing supports master-plan product delivery</h5><p>Invoices, plan changes, tax briefs, customer Mail, and evidence receipts stay inside FD-001 so product delivery remains the master-plan goal.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="stage-invoice">Stage invoice</button><button type="button" data-finance-anchor-action="route-mail">Mail customer</button></div></article><article class="finance-anchor-card" data-finance-anchor-card="billing-cloud"><p class="screen-anchor">OYATIE CLOUD TENANT</p><h5>Revenue systems run as tenant workloads</h5><p>Oyatie Cloud proves production hosting through residency, policy, rollback, and audit receipts before any FD-001 billing surface claims readiness.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="route-cloud">Cloud proof</button><button type="button" data-finance-anchor-action="route-policy">Tax policy</button></div></article><article class="finance-anchor-card" data-finance-anchor-card="billing-local"><p class="screen-anchor">LOCAL-ONLY CASH CONTROL</p><h5>Tax and invoice dry-run only</h5><p>Operators can route invoices, HomeTax briefs, and plan reviews visually; no bank, payroll, tax filing, billing send, or cloud mutation executes.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="tax-brief">Tax brief</button><button type="button" data-finance-anchor-action="route-audit">Audit packet</button></div></article></div><div class="finance-anchor-footer"><span data-finance-anchor-status="true">Billing ready · FD-001 revenue workload dogfoods Oyatie Cloud locally.</span><div class="finance-anchor-routes" aria-label="Billing connected routes"><button type="button" data-finance-anchor-action="route-vendors">Vendors</button><button type="button" data-finance-anchor-action="route-leave">Leave cost</button><button type="button" data-finance-anchor-action="route-workflow">Workflow</button><button type="button" data-finance-anchor-action="route-evidence">Evidence</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_finance_leave_anchor_board() -> &'static str {
    r#"<div class="finance-anchor-grid" aria-label="FD-001 leave time and Oyatie Cloud tenant proof"><article class="finance-anchor-card selected" data-finance-anchor-card="leave-fd001"><p class="screen-anchor">FD-001 PEOPLE COST</p><h5>Leave and time feed payroll, schedule, and tenant workload delivery</h5><p>Leave approvals, overtime risk, payroll cutoff, Workflow routes, Mail, and Community updates are FD-001 tenant workload evidence, not a side module.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="approve-leave">Approve preview</button><button type="button" data-finance-anchor-action="route-workflow">Workflow route</button></div></article><article class="finance-anchor-card" data-finance-anchor-card="leave-cloud"><p class="screen-anchor">OYATIE CLOUD WORKFORCE</p><h5>Substrate hosts workforce-cost tenant surfaces</h5><p>Oyatie Cloud proves the workforce substrate with regional policy, audit receipts, and deployment gates before leave or payroll workloads claim readiness.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="route-cloud">Cloud cells</button><button type="button" data-finance-anchor-action="route-policy">PIPA policy</button></div></article><article class="finance-anchor-card" data-finance-anchor-card="leave-local"><p class="screen-anchor">LOCAL-ONLY TIME RAIL</p><h5>Interactive liability preview</h5><p>Operators can reassign coverage, preview timesheet locks, and brief reviewers; no bank, payroll, tax, billing, HRIS, or cloud mutation executes.</p><div class="finance-anchor-actions"><button type="button" data-finance-anchor-action="reassign-time">Reassign time</button><button type="button" data-finance-anchor-action="route-audit">Audit trail</button></div></article></div><div class="finance-anchor-footer"><span data-finance-anchor-status="true">Leave/time ready · FD-001 people-cost workload dogfoods Oyatie Cloud locally.</span><div class="finance-anchor-routes" aria-label="Leave time connected routes"><button type="button" data-finance-anchor-action="route-ledger">Ledger</button><button type="button" data-finance-anchor-action="route-mail">Reviewer Mail</button><button type="button" data-finance-anchor-action="route-community">Community</button><button type="button" data-finance-anchor-action="route-evidence">Evidence</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_ledger_preview_anchor_board() -> &'static str {
    r#"<div class="trust-anchor-board ledger-trust-board" aria-label="FD-001 ledger close and Oyatie Cloud commercial proof"><div class="trust-anchor-grid"><article class="trust-anchor-card selected" data-trust-proof-card="ledger-fd001"><p class="screen-anchor">FD-001 CLOSE PACKAGE</p><h5>Ledger is the commercial product spine</h5><p>Payroll, filing, vendors, billing, leave/time, Workflow, Mail, Community, and audit receipts resolve into one FD-001 tenant workload close packet.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="stage-close">Stage close</button><button type="button" data-trust-proof-action="route-workflow">Workflow gate</button></div></article><article class="trust-anchor-card" data-trust-proof-card="ledger-cloud"><p class="screen-anchor">OYATIE CLOUD FINOPS</p><h5>Substrate cost and audit prove readiness</h5><p>Oyatie Cloud hosts commercial microservices as tenant workloads while FinOps, resource inventory, release gates, and policy guard the close.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-finops">FinOps</button><button type="button" data-trust-proof-action="route-inventory">Resources</button></div></article><article class="trust-anchor-card" data-trust-proof-card="ledger-local"><p class="screen-anchor">LOCAL-ONLY LEDGER CONTROL</p><h5>Dense finance preview, no money movement</h5><p>Operators can stage reconciliations, route reviewers, and attach evidence visually; no bank, payroll, tax, invoice, database, or cloud mutation executes.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-mail">Reviewer Mail</button><button type="button" data-trust-proof-action="route-audit">Audit packet</button></div></article></div><div class="trust-anchor-footer"><span data-trust-proof-status="true">Ledger ready · FD-001 commercial close workload dogfoods Oyatie Cloud with local visual controls only.</span><div class="trust-anchor-routes" aria-label="Ledger preview connected routes"><button type="button" data-trust-proof-action="route-billing">Billing</button><button type="button" data-trust-proof-action="route-vendors">Vendors</button><button type="button" data-trust-proof-action="route-filing">Filing</button><button type="button" data-trust-proof-action="route-evidence">Evidence</button></div></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_finance_commercial_service() -> String {
    format!(
        r#"<section id="finance-commercial-service" class="finance-commercial-service panel" data-finance-service="true" aria-labelledby="finance-service-title">
  <div class="finance-service-head">
    <div><p class="screen-anchor">MONEY · COMMERCIAL OPERATIONS</p><h3 id="finance-service-title">Finance, ledger, vendor spend, billing, and leave-time</h3></div>
    <div class="finance-service-actions"><span class="status-chip success" data-finance-status="true">close package ready</span><button type="button" data-finance-action="reconcile">Reconcile</button><button type="button" data-finance-action="export-pack">Export pack</button></div>
  </div>
  <div class="finance-kpi-strip" aria-label="Finance operations summary"><span><small>Cash runway</small><strong>18.4 mo</strong><em>₩12.6B available</em></span><span><small>Open invoices</small><strong>₩482M</strong><em>12 invoices · 2 overdue</em></span><span><small>Vendor risk</small><strong>3 high</strong><em>Stripe · AWS · payroll bureau</em></span><span><small>Leave liability</small><strong>₩53M</strong><em>49.5h this week</em></span></div>
  <div class="finance-service-shell">
    <aside class="finance-ledger-rail" aria-label="Finance sections">
      <p class="screen-anchor">FINANCE VIEWS</p>
      <button type="button" class="active" data-finance-tab="ledger">Ledger close</button>
      <button type="button" data-finance-tab="vendors">Vendors &amp; spend</button>
      <button type="button" data-finance-tab="billing">Billing &amp; tax</button>
      <button type="button" data-finance-tab="leave">Leave &amp; time</button>
      <div class="finance-close-card"><small>APRIL CLOSE</small><strong>73%</strong><span class="score-bar" style="--bar: 73%"><em></em></span><p>Payroll, withholding, and vendor accruals are staged for 2-person review.</p></div>
    </aside>
    <div class="finance-service-main">
      <div class="finance-tabs" role="tablist" aria-label="Finance service views"><button type="button" class="active" data-finance-tab="ledger" role="tab" aria-selected="true">Ledger</button><button type="button" data-finance-tab="vendors" role="tab" aria-selected="false">Vendors</button><button type="button" data-finance-tab="billing" role="tab" aria-selected="false">Billing</button><button type="button" data-finance-tab="leave" role="tab" aria-selected="false">Leave &amp; time</button></div>
      <article id="ledger-preview" class="finance-panel active" data-finance-panel="ledger" aria-labelledby="ledger-panel-title">
        <div class="finance-panel-copy"><p class="screen-anchor">LEDGER · CLOSE PACKAGE</p><h4 id="ledger-panel-title">Ledger close cockpit</h4><p>Every payroll, filing, vendor, and billing event resolves into one audit-ready close package.</p></div>
        <div class="ledger-layout"><div class="ledger-reconciliation"><span style="--bar: 92%"><em>Bank feed match · 92%</em></span><span style="--bar: 86%"><em>Payroll accrual · 86%</em></span><span style="--bar: 68%"><em>Vendor accrual · 68%</em></span><span style="--bar: 51%"><em>Tax evidence · 51%</em></span></div><table class="finance-table"><thead><tr><th>Time</th><th>Account</th><th>Object</th><th>Amount</th><th>State</th></tr></thead><tbody><tr><td>09:18</td><td>Payroll payable</td><td>APR payroll close</td><td>₩894,000,000</td><td><span class="status-chip warning">review</span></td></tr><tr><td>09:42</td><td>Withholding tax</td><td>HomeTax draft</td><td>₩118,400,000</td><td><span class="status-chip success">matched</span></td></tr><tr><td>10:05</td><td>Vendor accrual</td><td>Stripe invoice</td><td>₩4,820,000</td><td><span class="status-chip danger">blocking</span></td></tr><tr><td>10:21</td><td>Leave liability</td><td>Yoon Tae-min risk</td><td>₩53,000,000</td><td><span class="status-chip">advisory</span></td></tr></tbody></table></div>
        {finance_command_board}
        {ledger_preview_anchor_board}
      </article>
      <article id="vendors-spend" class="finance-panel" data-finance-panel="vendors" aria-labelledby="vendors-panel-title">
        <div class="finance-panel-copy"><p class="screen-anchor">PROCUREMENT · VENDORS</p><h4 id="vendors-panel-title">Vendors &amp; spend control</h4><p>Contracts, approvals, owners, and cost-of-delay are tied to the same service graph as workflow and mail.</p></div>
        <div class="vendor-toolbar"><label><span aria-hidden="true">⌕</span><input data-vendor-search="true" aria-label="Search vendors" placeholder="Search vendor, owner, contract..." /></label><button type="button" data-finance-action="add-vendor">+ Vendor</button></div>
        <table class="finance-table vendor-table"><thead><tr><th>Vendor</th><th>Owner</th><th>Monthly</th><th>Renewal</th><th>Risk</th><th>Action</th></tr></thead><tbody><tr data-vendor-row="true"><td><strong>Stripe</strong><small>Payments · invoice INV-4281</small></td><td>Finance</td><td>₩4.82M</td><td>2026-05-31</td><td><span class="status-chip danger">High</span></td><td><button type="button" data-finance-action="approve-vendor">Review</button></td></tr><tr data-vendor-row="true"><td><strong>AWS Korea</strong><small>Cloud infrastructure · reserved capacity</small></td><td>SRE</td><td>₩63.4M</td><td>2026-06-12</td><td><span class="status-chip warning">Medium</span></td><td><button type="button" data-finance-action="optimize-spend">Optimize</button></td></tr><tr data-vendor-row="true"><td><strong>Shinhan Bank</strong><small>Payroll and withholding transport</small></td><td>CFO</td><td>₩1.2M</td><td>2027-01-10</td><td><span class="status-chip success">Low</span></td><td><button type="button" data-sidepeek-trigger="bank-vendor" data-sidepeek-title="Shinhan Bank transport" data-sidepeek-id="VEN-SHINHAN" data-sidepeek-desc="Bank transport is staged visually and is not connected to a real payment rail." data-sidepeek-owner="CFO" data-sidepeek-risk="Low" data-sidepeek-sla="Staged only">Inspect</button></td></tr></tbody></table>
        {finance_vendors_anchor_board}
      </article>
      <article id="billing-tax" class="finance-panel" data-finance-panel="billing" aria-labelledby="billing-panel-title">
        <div class="finance-panel-copy"><p class="screen-anchor">BILLING · TAX</p><h4 id="billing-panel-title">Billing, plans, and filings</h4><p>Customer invoices, plan changes, HomeTax filing readiness, and payment evidence stay visible next to operations.</p></div>
        <div class="billing-grid"><article><p class="screen-anchor">REVENUE</p><strong>₩2.31B</strong><span>ARR staged · 42 active contracts</span><button type="button" data-finance-action="send-invoice">Stage invoice</button></article><article><p class="screen-anchor">TAX FILING</p><strong>64%</strong><span>HomeTax transport awaiting reviewer</span><button type="button" data-finance-action="tax-brief">Draft brief</button></article><article><p class="screen-anchor">PLAN CHANGES</p><strong>7</strong><span>2 require billing owner review</span><button type="button" data-sidepeek-trigger="billing-plans" data-sidepeek-title="Plan change queue" data-sidepeek-id="BILL-PLAN-7" data-sidepeek-desc="Plan changes are staged queue items with no payment or billing mutation." data-sidepeek-owner="Revenue ops" data-sidepeek-risk="Review" data-sidepeek-sla="Visual only">Open queue</button></article></div>
        {finance_billing_anchor_board}
      </article>
      <article id="leave-time" class="finance-panel" data-finance-panel="leave" aria-labelledby="leave-panel-title">
        <div class="finance-panel-copy"><p class="screen-anchor">PEOPLE COST · LEAVE</p><h4 id="leave-panel-title">Leave &amp; time liability</h4><p>Leave approvals are connected to workforce, payroll, schedule, and financial liability before close.</p></div>
        <div class="leave-layout"><ol class="leave-queue"><li><time>May 13–17</time><strong>김지영 leave request</strong><span>5 days · backup confirmed</span><button type="button" data-finance-action="approve-leave">Approve locally</button></li><li><time>This week</time><strong>윤태민 overtime risk</strong><span>49.5h projected · 2 backup engineers recommended</span><button type="button" data-finance-action="reassign-time">Reassign</button></li><li><time>May 22</time><strong>Payroll cutoff</strong><span>Timesheets lock 3 days before payout</span><button type="button" data-finance-action="lock-timesheets">Preview lock</button></li></ol><div class="time-heatmap" aria-label="Weekly time utilization"><span style="--bar: 58%">Mon</span><span style="--bar: 72%">Tue</span><span style="--bar: 91%">Wed</span><span style="--bar: 84%">Thu</span><span style="--bar: 49%">Fri</span></div></div>
        {finance_leave_anchor_board}
      </article>
    </div>
    <aside class="finance-context-rail" aria-label="Finance provenance"><p class="screen-anchor">CLOSE PROVENANCE</p><dl class="service-kv"><div><dt>Workflow</dt><dd>April close package</dd></div><div><dt>Receipts</dt><dd>18 staged</dd></div><div><dt>Reviewers</dt><dd>CFO · payroll · tax</dd></div></dl><p class="screen-anchor">LOCAL NOTIFICATIONS</p><ol class="notification-stack"><li>Vendor approval waiting</li><li>Tax brief can be drafted</li><li>No real money moves</li></ol></aside>
  </div>
</section>"#,
        finance_command_board = static_finance_command_board(),
        ledger_preview_anchor_board = static_ledger_preview_anchor_board(),
        finance_vendors_anchor_board = static_finance_vendors_anchor_board(),
        finance_billing_anchor_board = static_finance_billing_anchor_board(),
        finance_leave_anchor_board = static_finance_leave_anchor_board(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_finance_command_board() -> &'static str {
    r#"<section class="finance-command-board" aria-labelledby="finance-command-board-title"><div class="finance-command-head"><div><p class="screen-anchor">CLOSE COMMAND · OBJECT GRAPH</p><h4 id="finance-command-board-title">April commercial command center</h4><p>A Bominal-grade operating surface: payroll, ledger, tax, vendors, billing, leave, workflow, messages, and evidence stay in one dense local workspace.</p></div><div class="finance-command-actions"><span class="status-chip warning" data-finance-command-status="true">7 objects · 3 blockers · local dry-run</span><button type="button" data-finance-command-action="run-close">Run close dry-run</button><button type="button" data-finance-command-action="attach-proof" data-finance-route="evidence">Attach proof</button></div></div><div class="finance-command-grid"><article class="finance-command-card finance-close-spine-card"><div class="finance-command-card-head"><div><p class="screen-anchor">CLOSE SPINE</p><h5>Every commercial object joins the same package</h5></div><span class="status-chip danger">3 blockers</span></div><div class="finance-spine-flow" aria-label="Commercial close object spine"><button type="button" class="active" data-finance-command-action="open-payroll" data-finance-route="ledger"><span>Payroll</span><strong>₩894M</strong><em>review</em></button><button type="button" data-finance-command-action="open-tax" data-finance-route="billing"><span>Tax</span><strong>₩118M</strong><em>HomeTax</em></button><button type="button" data-finance-command-action="open-vendor" data-finance-route="vendors"><span>Vendors</span><strong>₩69M</strong><em>Stripe · AWS</em></button><button type="button" data-finance-command-action="open-billing" data-finance-route="billing"><span>Billing</span><strong>₩482M</strong><em>12 invoices</em></button><button type="button" data-finance-command-action="open-leave" data-finance-route="leave"><span>Leave</span><strong>49.5h</strong><em>liability</em></button><button type="button" data-finance-command-action="open-evidence" data-finance-route="evidence"><span>Evidence</span><strong>18</strong><em>receipts</em></button></div><dl class="finance-command-kv"><div><dt>Critical path</dt><dd>Payroll delta → vendor approval → HomeTax filing → CFO signoff</dd></div><div><dt>Autonomy ceiling</dt><dd>Visual dry-run only; no banking, payroll, tax, or invoice rail executes.</dd></div></dl></article><article class="finance-command-card finance-cash-pipeline-card"><div class="finance-command-card-head"><div><p class="screen-anchor">CASH PIPELINE</p><h5>Invoices, plans, and filing readiness</h5></div><button type="button" data-finance-command-action="open-billing" data-finance-route="billing">Billing</button></div><div class="finance-cash-lanes" aria-label="Commercial cash pipeline"><button type="button" class="finance-cash-row" data-finance-command-action="stage-invoice" data-finance-route="billing"><span>Invoice</span><strong>Northwind annual plan</strong><em>₩184M · due May 27</em><i class="status-chip success">ready</i></button><button type="button" class="finance-cash-row" data-finance-command-action="review-plan" data-finance-route="billing"><span>Plan</span><strong>7 contract changes</strong><em>2 owner reviews</em><i class="status-chip warning">review</i></button><button type="button" class="finance-cash-row" data-finance-command-action="tax-transport" data-finance-route="billing"><span>Tax</span><strong>HomeTax withholding</strong><em>118 employees validated</em><i class="status-chip warning">draft</i></button><button type="button" class="finance-cash-row" data-finance-command-action="bank-match" data-finance-route="ledger"><span>Bank</span><strong>Shinhan feed match</strong><em>92% matched</em><i class="status-chip success">matched</i></button></div></article><article class="finance-command-card finance-vendor-risk-card"><div class="finance-command-card-head"><div><p class="screen-anchor">RISK QUEUE</p><h5>Spend, renewal, and approval compression</h5></div><button type="button" data-finance-command-action="open-vendor" data-finance-route="vendors">Vendors</button></div><table class="finance-risk-table"><thead><tr><th>Object</th><th>Owner</th><th>SLA</th><th>Next</th></tr></thead><tbody><tr data-finance-risk-row="true"><td><strong>Stripe invoice</strong><small>approval can collapse to 1-stage</small></td><td>AP</td><td><span class="status-chip danger">4.0h</span></td><td><button type="button" data-finance-command-action="route-stripe" data-finance-route="vendors">Route</button></td></tr><tr data-finance-risk-row="true"><td><strong>AWS reserved capacity</strong><small>commit under-run vs kr-seoul gate</small></td><td>SRE</td><td><span class="status-chip warning">1d</span></td><td><button type="button" data-finance-command-action="route-aws" data-finance-route="cloud">FinOps</button></td></tr><tr data-finance-risk-row="true"><td><strong>Payroll bureau</strong><small>NHIS tier delta requires reviewer</small></td><td>CFO</td><td><span class="status-chip danger">today</span></td><td><button type="button" data-finance-command-action="route-payroll" data-finance-route="workflow">Workflow</button></td></tr></tbody></table></article><article class="finance-command-card finance-evidence-lane-card"><div class="finance-command-card-head"><div><p class="screen-anchor">EVIDENCE LANE</p><h5>Reviewer packet and communication routes</h5></div><span class="status-chip success">sealed draft</span></div><ol class="finance-evidence-lane"><li><span>REC-PAY-2026-04-PARK</span><strong>Payroll delta</strong><em>blocking · Finance close</em></li><li><span>REC-TAX-HOMETAX-118</span><strong>Withholding transport</strong><em>review · CFO desk</em></li><li><span>REC-COMM-GOV-221</span><strong>Council note</strong><em>ready · Community</em></li></ol><div class="finance-mini-actions"><button type="button" data-finance-command-action="mail-brief" data-finance-route="mail">Mail brief</button><button type="button" data-finance-command-action="messenger-room" data-finance-route="messenger">Messenger room</button><button type="button" data-finance-command-action="council-note" data-finance-route="community">Community note</button></div></article></div><div class="finance-route-matrix" aria-label="Commercial operations route matrix"><button type="button" data-finance-command-action="route-ledger" data-finance-route="ledger">Ledger</button><button type="button" data-finance-command-action="route-vendors" data-finance-route="vendors">Vendors</button><button type="button" data-finance-command-action="route-billing" data-finance-route="billing">Billing · Tax</button><button type="button" data-finance-command-action="route-leave" data-finance-route="leave">Leave · Time</button><button type="button" data-finance-command-action="route-workflow" data-finance-route="workflow">Workflow</button><button type="button" data-finance-command-action="route-mail" data-finance-route="mail">Mail</button><button type="button" data-finance-command-action="route-community" data-finance-route="community">Community</button><button type="button" data-finance-command-action="route-evidence" data-finance-route="evidence">Evidence</button></div></section>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_evidence_ledger_anchor_board() -> &'static str {
    r#"<div class="trust-anchor-board" aria-label="FD-001 evidence ledger and Oyatie Cloud substrate proof"><div class="trust-anchor-grid"><article class="trust-anchor-card selected" data-trust-proof-card="ledger-fd001"><p class="screen-anchor">FD-001 RECEIPT SPINE</p><h5>Tenant workload delivery remains the master-plan goal</h5><p>Messenger, Mail, Community, Workflow, Finance, and Daily Work receipts stay one FD-001 tenant workload packet rather than disconnected modules.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="seal-receipt">Seal packet</button><button type="button" data-trust-proof-action="route-workflow">Workflow proof</button></div></article><article class="trust-anchor-card" data-trust-proof-card="ledger-cloud"><p class="screen-anchor">OYATIE CLOUD ADMISSION</p><h5>Substrate proves real tenant hosting</h5><p>Every cloud cell, policy grant, release gate, and FinOps signal attaches evidence before FD-001 services can claim production readiness.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-cloud">Cloud cells</button><button type="button" data-trust-proof-action="route-gates">Release gates</button></div></article><article class="trust-anchor-card" data-trust-proof-card="ledger-local"><p class="screen-anchor">LOCAL-ONLY RECEIPT VAULT</p><h5>Interactive trust without mutation</h5><p>Operators can inspect, seal, brief, and route receipts visually; no backend write, deploy, billing, mail, or cloud mutation executes.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-mail">Mail brief</button><button type="button" data-trust-proof-action="route-community">Community note</button></div></article></div><div class="trust-anchor-footer"><span data-trust-proof-status="true">Evidence ledger ready · FD-001 tenant workload receipts dogfood Oyatie Cloud locally.</span><div class="trust-anchor-routes" aria-label="Evidence ledger connected routes"><button type="button" data-trust-proof-action="route-finops">FinOps</button><button type="button" data-trust-proof-action="route-inventory">Inventory</button><button type="button" data-trust-proof-action="route-graph">Object graph</button><button type="button" data-trust-proof-action="route-policy">Policy</button></div></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_object_graph_anchor_board() -> &'static str {
    r#"<div class="trust-anchor-board" aria-label="FD-001 object graph and Oyatie Cloud substrate proof"><div class="trust-anchor-grid"><article class="trust-anchor-card selected" data-trust-proof-card="graph-fd001"><p class="screen-anchor">FD-001 OBJECT MODEL</p><h5>One service graph spans every surface</h5><p>Workflow, approvals, Messenger, Mail, Community, Finance, Daily Work, and audit nodes resolve to a single tenant operation lineage.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="trace-lineage">Trace lineage</button><button type="button" data-trust-proof-action="route-catalog">Catalog objects</button></div></article><article class="trust-anchor-card" data-trust-proof-card="graph-cloud"><p class="screen-anchor">OYATIE CLOUD GRAPH</p><h5>Substrate nodes join product nodes</h5><p>Cells, resources, policies, deployment gates, FinOps, and receipts prove FD-001 microservices can run as production tenant workloads.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-cloud">Cloud topology</button><button type="button" data-trust-proof-action="route-policy">Policy edge</button></div></article><article class="trust-anchor-card" data-trust-proof-card="graph-local"><p class="screen-anchor">LOCAL-ONLY GRAPH OPS</p><h5>Selectable lineage, no side effects</h5><p>Operators can traverse graph edges, stage evidence, and open communications visually; no database, workflow, deploy, or cloud mutation occurs.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-evidence">Evidence spine</button><button type="button" data-trust-proof-action="route-mail">Reviewer Mail</button></div></article></div><div class="trust-anchor-footer"><span data-trust-proof-status="true">Object graph ready · FD-001 services and Oyatie Cloud substrate stay one local visual lineage.</span><div class="trust-anchor-routes" aria-label="Object graph connected routes"><button type="button" data-trust-proof-action="route-workflow">Workflow</button><button type="button" data-trust-proof-action="route-inventory">Resources</button><button type="button" data-trust-proof-action="route-community">Community</button><button type="button" data-trust-proof-action="route-finops">FinOps</button></div></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_operator_intelligence_strip(envelope: &TenantRenderEnvelope) -> String {
    let evidence_count = envelope.approvals.len() + envelope.workflow.nodes.len() + 6;
    let readiness = if envelope.accreditation.healthcare_enabled {
        "Accredited"
    } else {
        "Gated"
    };
    let object_count =
        envelope.modules.len() + envelope.daily_tasks.len() + envelope.community.len() + 7;
    let signal_count =
        envelope.messages.len() + envelope.community.len() + envelope.approvals.len();

    format!(
        r#"<section id="evidence-spine" class="operator-intelligence evidence-intelligence-console panel" aria-label="Evidence spine, object graph, and governed copilot intelligence"><div class="evidence-console-head"><div><p class="screen-anchor">EVIDENCE SPINE · OBJECT GRAPH</p><h3>Operational intelligence console</h3><p>Workflow, approvals, Messenger, Mail, Community, cloud operations, and audit receipts are shown as one cohesive local service graph.</p></div><div class="evidence-head-actions" aria-label="Evidence console actions"><span class="status-chip success">sealed draft</span><button type="button" data-evidence-action="run-review">Run review</button><button type="button" data-evidence-action="export">Export packet</button></div></div><div class="evidence-kpi-strip" aria-label="Operational intelligence summary"><span><strong>{evidence_count}</strong><small>evidence leaves</small></span><span><strong>{object_count}</strong><small>graph objects</small></span><span><strong>{signal_count}</strong><small>cross-service signals</small></span><span><strong>{readiness}</strong><small>tenant readiness</small></span><span><strong>0</strong><small>backend writes</small></span></div><div class="evidence-console-toolbar" aria-label="Evidence ledger filters"><label class="evidence-search"><span aria-hidden="true">⌕</span><input data-evidence-search="true" type="search" aria-label="Search evidence ledger" placeholder="Search evidence, owner, object, route..." /></label><div class="evidence-filter-pills" role="toolbar" aria-label="Evidence state filters"><button type="button" class="active" data-evidence-filter="all">All</button><button type="button" data-evidence-filter="blocking">Blocking</button><button type="button" data-evidence-filter="review">Review</button><button type="button" data-evidence-filter="sealed">Sealed</button><button type="button" data-evidence-filter="watch">Watch</button></div><span class="evidence-console-status" data-evidence-status="true">6 visible · all states · local evidence only</span></div><div class="evidence-layout"><article id="evidence-ledger" class="evidence-ledger-panel" aria-labelledby="evidence-ledger-title"><div class="evidence-panel-head"><div><p class="screen-anchor">LEDGER</p><h4 id="evidence-ledger-title">Receipt timeline</h4></div><button type="button" data-evidence-action="attach">Attach to inbox</button></div><ol class="evidence-event-list"><li class="evidence-event" data-evidence-event="true" data-evidence-state="blocking"><button type="button" class="evidence-event-main" data-evidence-action="open" data-sidepeek-trigger="evidence" data-sidepeek-title="Payroll delta needs four-insurance approval" data-sidepeek-id="REC-PAY-2026-04-PARK" data-sidepeek-desc="NHIS tier increase detected; owner must approve before April close package seals." data-sidepeek-owner="Finance close" data-sidepeek-risk="blocking" data-sidepeek-sla="4.0h"><span class="status-chip danger">blocking</span><strong>Payroll delta needs four-insurance approval</strong><p>NHIS tier increase detected; owner must approve before April close package seals.</p></button><dl><div><dt>Source</dt><dd>Payroll</dd></div><div><dt>Owner</dt><dd>Finance close</dd></div><div><dt>SLA</dt><dd>4.0h</dd></div></dl></li><li class="evidence-event" data-evidence-event="true" data-evidence-state="review"><button type="button" class="evidence-event-main" data-evidence-action="open" data-sidepeek-trigger="evidence" data-sidepeek-title="HomeTax withholding transport waiting" data-sidepeek-id="REC-TAX-HOMETAX-118" data-sidepeek-desc="118 employees validated; 사업자등록번호 confirmation remains before send preview." data-sidepeek-owner="CFO desk" data-sidepeek-risk="review" data-sidepeek-sla="1d"><span class="status-chip warning">review</span><strong>HomeTax withholding transport waiting</strong><p>118 employees validated; 사업자등록번호 confirmation remains before send preview.</p></button><dl><div><dt>Source</dt><dd>Tax</dd></div><div><dt>Owner</dt><dd>CFO desk</dd></div><div><dt>SLA</dt><dd>1d</dd></div></dl></li><li class="evidence-event" data-evidence-event="true" data-evidence-state="sealed"><button type="button" class="evidence-event-main" data-evidence-action="open" data-sidepeek-trigger="evidence" data-sidepeek-title="Workflow receipt staged from Command Center" data-sidepeek-id="REC-WF-7741" data-sidepeek-desc="Tenant change approval produced Messenger, Mail, Community, and audit drafts." data-sidepeek-owner="Tenant admin" data-sidepeek-risk="sealed" data-sidepeek-sla="sealed"><span class="status-chip success">sealed</span><strong>Workflow receipt staged from Command Center</strong><p>Tenant change approval produced Messenger, Mail, Community, and audit drafts.</p></button><dl><div><dt>Source</dt><dd>Workflow</dd></div><div><dt>Owner</dt><dd>Tenant admin</dd></div><div><dt>SLA</dt><dd>sealed</dd></div></dl></li><li class="evidence-event" data-evidence-event="true" data-evidence-state="review"><button type="button" class="evidence-event-main" data-evidence-action="open" data-sidepeek-trigger="evidence" data-sidepeek-title="Network split rollback evidence requested" data-sidepeek-id="REC-CLOUD-MESH-4182" data-sidepeek-desc="us-east-2 mesh split requires regional capacity and rollback runbook attestation." data-sidepeek-owner="Infrastructure SRE" data-sidepeek-risk="review" data-sidepeek-sla="2.1h"><span class="status-chip warning">review</span><strong>Network split rollback evidence requested</strong><p>us-east-2 mesh split requires regional capacity and rollback runbook attestation.</p></button><dl><div><dt>Source</dt><dd>Cloud Ops</dd></div><div><dt>Owner</dt><dd>Infrastructure SRE</dd></div><div><dt>SLA</dt><dd>2.1h</dd></div></dl></li><li class="evidence-event" data-evidence-event="true" data-evidence-state="sealed"><button type="button" class="evidence-event-main" data-evidence-action="open" data-sidepeek-trigger="evidence" data-sidepeek-title="Governance council broadcast prepared" data-sidepeek-id="REC-COMM-GOV-221" data-sidepeek-desc="Community note links policy rationale, approval owner, and object graph lineage." data-sidepeek-owner="Governance" data-sidepeek-risk="sealed" data-sidepeek-sla="sealed"><span class="status-chip success">sealed</span><strong>Governance council broadcast prepared</strong><p>Community note links policy rationale, approval owner, and object graph lineage.</p></button><dl><div><dt>Source</dt><dd>Community</dd></div><div><dt>Owner</dt><dd>Governance</dd></div><div><dt>SLA</dt><dd>sealed</dd></div></dl></li><li class="evidence-event" data-evidence-event="true" data-evidence-state="watch"><button type="button" class="evidence-event-main" data-evidence-action="open" data-sidepeek-trigger="evidence" data-sidepeek-title="Vendor renewal route can be shortened" data-sidepeek-id="REC-VND-STRIPE-4820" data-sidepeek-desc="Stripe 청구서 approval can move from three-stage to one-stage below policy threshold." data-sidepeek-owner="AP owner" data-sidepeek-risk="watch" data-sidepeek-sla="next run"><span class="status-chip">watch</span><strong>Vendor renewal route can be shortened</strong><p>Stripe 청구서 approval can move from three-stage to one-stage below policy threshold.</p></button><dl><div><dt>Source</dt><dd>Procurement</dd></div><div><dt>Owner</dt><dd>AP owner</dd></div><div><dt>SLA</dt><dd>next run</dd></div></dl></li></ol>{evidence_ledger_anchor_board}</article><article id="object-graph" class="object-graph-panel" aria-labelledby="object-graph-title"><div class="evidence-panel-head"><div><p class="screen-anchor">OBJECT GRAPH</p><h4 id="object-graph-title">Tenant operation lineage</h4></div><span data-object-status="true">Tenant selected · 8 linked objects</span></div><div class="object-graph-canvas" aria-label="Selectable object graph preview"><button type="button" class="object-node active" data-object-node="tenant" data-object-label="Tenant" data-sidepeek-trigger="object-graph" data-sidepeek-title="Tenant" data-sidepeek-id="OBJ-TENANT" data-sidepeek-desc="Authoritative tenant envelope and pack gates" data-sidepeek-owner="Object graph" data-sidepeek-risk="Read-only" data-sidepeek-sla="Local data"><span>Tenant</span><strong>Northwind Corp.</strong></button><button type="button" class="object-node" data-object-node="workflow" data-object-label="Workflow" data-sidepeek-trigger="object-graph" data-sidepeek-title="Workflow" data-sidepeek-id="OBJ-WORKFLOW" data-sidepeek-desc="No-code approval path and run preview" data-sidepeek-owner="Object graph" data-sidepeek-risk="Read-only" data-sidepeek-sla="Local data"><span>Workflow</span><strong>{workflow_name}</strong></button><button type="button" class="object-node" data-object-node="approval" data-object-label="Approval" data-sidepeek-trigger="object-graph" data-sidepeek-title="Approval" data-sidepeek-id="OBJ-APPROVAL" data-sidepeek-desc="Human reviewer checkpoint before action" data-sidepeek-owner="Object graph" data-sidepeek-risk="Read-only" data-sidepeek-sla="Local data"><span>Approval</span><strong>APR-274</strong></button><button type="button" class="object-node" data-object-node="mail" data-object-label="Mail" data-sidepeek-trigger="object-graph" data-sidepeek-title="Mail" data-sidepeek-id="OBJ-MAIL" data-sidepeek-desc="Formal approval route draft" data-sidepeek-owner="Object graph" data-sidepeek-risk="Read-only" data-sidepeek-sla="Local data"><span>Mail</span><strong>Finance close brief</strong></button><button type="button" class="object-node" data-object-node="messenger" data-object-label="Messenger" data-sidepeek-trigger="object-graph" data-sidepeek-title="Messenger" data-sidepeek-id="OBJ-MESSENGER" data-sidepeek-desc="Fast coordination thread" data-sidepeek-owner="Object graph" data-sidepeek-risk="Read-only" data-sidepeek-sla="Local data"><span>Messenger</span><strong>Ops room</strong></button><button type="button" class="object-node" data-object-node="community" data-object-label="Community" data-sidepeek-trigger="object-graph" data-sidepeek-title="Community" data-sidepeek-id="OBJ-COMMUNITY" data-sidepeek-desc="Role-aware broadcast" data-sidepeek-owner="Object graph" data-sidepeek-risk="Read-only" data-sidepeek-sla="Local data"><span>Community</span><strong>Governance council</strong></button><button type="button" class="object-node" data-object-node="cloud" data-object-label="Cloud cell" data-sidepeek-trigger="object-graph" data-sidepeek-title="Cloud cell" data-sidepeek-id="OBJ-CLOUD" data-sidepeek-desc="Runtime, network, and FinOps posture" data-sidepeek-owner="Object graph" data-sidepeek-risk="Read-only" data-sidepeek-sla="Local data"><span>Cloud cell</span><strong>us-east-2</strong></button><button type="button" class="object-node" data-object-node="audit" data-object-label="Receipt" data-sidepeek-trigger="object-graph" data-sidepeek-title="Receipt" data-sidepeek-id="OBJ-AUDIT" data-sidepeek-desc="Immutable local evidence preview" data-sidepeek-owner="Object graph" data-sidepeek-risk="Read-only" data-sidepeek-sla="Local data"><span>Receipt</span><strong>REC-WF-7741</strong></button><svg viewBox="0 0 640 280" aria-hidden="true" class="object-graph-links"><path d="M92 60 C210 40 250 112 324 116 S484 88 552 64" /><path d="M96 118 C190 168 266 158 326 116" /><path d="M324 116 C388 146 452 168 550 166" /><path d="M322 116 C302 198 374 230 548 226" /><path d="M92 222 C190 232 252 204 324 116" /></svg></div><div class="object-graph-table" aria-label="Object graph properties"><dl><div><dt>Graph root</dt><dd>{workflow_name}</dd></div><div><dt>Primary output</dt><dd>Task · message · evidence draft</dd></div><div><dt>Autonomy ceiling</dt><dd>No auto-execution</dd></div><div><dt>Region</dt><dd>us-east-2 active · kr-seoul pack gated</dd></div></dl></div>{object_graph_anchor_board}</article><aside class="copilot-rail-panel" aria-labelledby="intel-copilot-title"><div class="evidence-panel-head compact"><div><p class="screen-anchor">COPILOT RAIL</p><h4 id="intel-copilot-title">Governed next moves</h4></div><span class="status-chip ai">PIPA-safe</span></div><div class="intel-action-stack"><button type="button" data-intel-action="audit"><strong>Draft audit brief</strong><span>Bundle payroll, HomeTax, workflow, and cloud evidence into one reviewer packet.</span></button><button type="button" data-intel-action="workflow"><strong>Simulate critical path</strong><span>Preview CFO escalation and Mail/Messenger/Community outputs before any execution.</span></button><button type="button" data-intel-action="mail"><strong>Compose approval mail</strong><span>Open the built-in Mail surface with receipt links and owner context.</span></button><button type="button" data-intel-action="community"><strong>Post council note</strong><span>Route a governance update to Community without leaving the console.</span></button></div><p class="copilot-rail-status" data-copilot-rail-status="true">Read-only recommendations; every action changes local visual state only.</p></aside><article class="signal-lineage-panel" aria-labelledby="signal-lineage-title"><div class="evidence-panel-head"><div><p class="screen-anchor">CROSS-MODULE SIGNAL</p><h4 id="signal-lineage-title">{workflow_name}</h4></div><span class="status-chip warning">review path</span></div><p>{workflow_goal}</p><ol class="signal-lineage"><li class="root"><span>Workflow</span><strong>Tenant change approval</strong><em>root event</em></li><li><span>Messenger</span><strong>Ops room update</strong><em>drafted</em></li><li><span>Mail</span><strong>Finance approval brief</strong><em>ready</em></li><li><span>Community</span><strong>Governance council note</strong><em>review</em></li><li><span>Cloud Ops</span><strong>Rollback evidence</strong><em>blocking</em></li><li><span>Audit</span><strong>Receipt spine</strong><em>sealed draft</em></li></ol><div class="lineage-actions"><button type="button" data-intel-action="messenger">Messenger</button><button type="button" data-intel-action="mail">Mail</button><button type="button" data-intel-action="community">Community</button><button type="button" data-intel-action="audit">Audit ledger</button></div></article></div></section>"#,
        evidence_count = evidence_count,
        object_count = object_count,
        signal_count = signal_count,
        readiness = escape(readiness),
        workflow_name = escape(&envelope.workflow.name),
        workflow_goal = escape(&envelope.workflow.goal),
        evidence_ledger_anchor_board = static_evidence_ledger_anchor_board(),
        object_graph_anchor_board = static_object_graph_anchor_board()
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_cloud_ops_command_matrix() -> &'static str {
    r#"<div class="ops-command-matrix" aria-label="Cloud operations command matrix"><section class="ops-command-card ops-cell-card"><div class="ops-command-card-head"><div><p class="screen-anchor">CELL CONTROL</p><h5>Runtime cells, residency, and rollback posture</h5></div><span class="status-chip warning">2 guardrails</span></div><div class="ops-cell-grid" role="list" aria-label="Regional cell state"><button type="button" class="active" data-cockpit-action="select-us-east"><strong>us-east-2</strong><span>primary</span><em>99.96% · 72% cap</em></button><button type="button" data-cockpit-action="select-eu-west"><strong>eu-west-1</strong><span>standby</span><em>warm · 44% cap</em></button><button type="button" data-cockpit-action="select-kr-seoul"><strong>kr-seoul</strong><span>pack gated</span><em>residency review</em></button></div><dl class="ops-command-kv"><div><dt>Residency</dt><dd>KR pack gated before workload placement</dd></div><div><dt>Rollback</dt><dd>Network split runbook needs reviewer evidence</dd></div><div><dt>Audit sidecar</dt><dd>Receipt vault sealed draft attached</dd></div></dl></section><section class="ops-command-card ops-workload-card ops-tenant-plane" data-cloud-workload-plane="true"><div class="ops-command-card-head"><div><p class="screen-anchor">FD-001 TENANT WORKLOAD PLANE</p><h5>Product microservices hosted on the dogfood substrate</h5></div><button type="button" data-cockpit-action="open-resource-inventory">Inventory</button></div><div class="ops-plane-summary" aria-label="FD-001 tenant workload summary"><span><strong>9</strong><small>FD-001 services</small></span><span><strong>3</strong><small>cells</small></span><span><strong>0</strong><small>live mutations</small></span></div><div class="ops-workload-list" aria-label="FD-001 microservices running as tenant workloads"><button type="button" class="selected" data-cockpit-workload="workflow" data-workload-title="Workflow runner" data-workload-service="workflow-runner" data-workload-cell="us-east-2" data-workload-state="review" data-workload-route="Workflow → Messenger/Mail/Community → Evidence" data-workload-receipt="REC-FD001-WF-018"><span>Workflow</span><strong>workflow-runner</strong><em>us-east-2 · review</em><small>Runs approvals as visual-only tenant workload previews.</small></button><button type="button" data-cockpit-workload="comms" data-workload-title="Built-in communications" data-workload-service="messenger-mail-community" data-workload-cell="us-east-2 + kr-seoul" data-workload-state="drafts" data-workload-route="Messenger/Mail/Community handoff bus" data-workload-receipt="REC-COMMS-HANDOFF-006"><span>Comms</span><strong>messenger-mail-community</strong><em>multi-surface · drafts</em><small>Local drafts prove FD-001 coordination without delivery.</small></button><button type="button" data-cockpit-workload="evidence" data-workload-title="Evidence spine" data-workload-service="audit-vault" data-workload-cell="multi-cell" data-workload-state="sealed" data-workload-route="Audit ledger + object graph" data-workload-receipt="REC-FD001-CLOUD-009"><span>Evidence</span><strong>audit-vault</strong><em>multi-cell · sealed</em><small>Receipts bind cloud posture, workflow output, and reviewers.</small></button><button type="button" data-cockpit-workload="identity" data-workload-title="Identity envelope" data-workload-service="identity-access" data-workload-cell="kr-seoul gated" data-workload-state="policy" data-workload-route="Identity → Policy → Deployment gates" data-workload-receipt="REC-ID-2026-05"><span>Identity</span><strong>identity-access</strong><em>kr pack · policy</em><small>Role and residency controls prove tenant placement.</small></button></div><div class="ops-workload-detail" aria-label="Selected tenant workload detail"><span class="status-chip warning" data-cockpit-workload-status="true">Workflow runner selected · review gate open · local-only substrate proof</span><dl><div><dt>Service</dt><dd data-workload-detail-service="true">workflow-runner</dd></div><div><dt>Cell</dt><dd data-workload-detail-cell="true">us-east-2</dd></div><div><dt>Route</dt><dd data-workload-detail-route="true">Workflow → Messenger/Mail/Community → Evidence</dd></div><div><dt>Receipt</dt><dd data-workload-detail-receipt="true">REC-FD001-WF-018</dd></div></dl><div class="ops-workload-routes" aria-label="Selected workload routes"><button type="button" data-cockpit-workload-route="workflow">Workflow</button><button type="button" data-cockpit-workload-route="mail">Mail brief</button><button type="button" data-cockpit-workload-route="community">Community</button><button type="button" data-cockpit-workload-route="evidence">Evidence</button><button type="button" data-cockpit-workload-route="gates">Gates</button></div></div></section><section class="ops-command-card ops-release-card"><div class="ops-command-card-head"><div><p class="screen-anchor">RELEASE GATES</p><h5>Jenkins, ArgoCD, cosign, and audit evidence</h5></div><button type="button" data-cockpit-action="open-deployment-gates">Gates</button></div><div class="ops-release-lanes" aria-label="Cloud release readiness"><span style="--bar: 92%"><strong>Jenkins parity</strong><em>92%</em></span><span style="--bar: 74%"><strong>ArgoCD app</strong><em>74%</em></span><span style="--bar: 88%"><strong>Cosign verify</strong><em>88%</em></span><span style="--bar: 69%"><strong>Audit emit</strong><em>69%</em></span></div></section><section class="ops-command-card ops-route-card"><div class="ops-command-card-head"><div><p class="screen-anchor">ROUTES</p><h5>Open the connected product surface without leaving context</h5></div></div><div class="ops-route-grid" aria-label="Cloud operations local routes"><button type="button" data-cockpit-action="open-workflow">Workflow</button><button type="button" data-cockpit-action="open-mail">Mail brief</button><button type="button" data-cockpit-action="open-evidence">Evidence</button><button type="button" data-cockpit-action="open-finops">FinOps</button></div><p>All actions are visual-only local state; no cloud, DNS, deploy, or billing operation is executed.</p></section></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_ops_cluster_health_panel() -> String {
    let api_status = ops_cluster_health_from_typed_api();
    let endpoint = api_status.endpoint();
    let status = ops_cluster_health_panel_status();
    let remediation = status
        .remediation
        .as_ref()
        .map(|route| {
            format!(
                r#"<button type="button" class="ds-remediation-route">{}</button>"#,
                escape(&route.label())
            )
        })
        .unwrap_or_default();
    let rollback_available = status.destructive_apply.is_some()
        || matches!(
            status.variant,
            crate::design_system::ops_deployment_status_panel::PanelVariant::Rollback
        );

    format!(
        r#"<div class="ops-cluster-health-panel" data-ops-cluster-health-source="typed-api" data-ops-cluster-health-endpoint="{endpoint}"><section class="ds-ops-deployment-status-panel" data-variant="{variant}" aria-label="Deployment status panel"><p role="status" aria-live="polite">{announcement}</p><dl><div><dt>Target environment</dt><dd>{target}</dd></div><div><dt>Current step</dt><dd>{step}</dd></div><div><dt>Drift</dt><dd>{drift}</dd></div><div><dt>Rollback available</dt><dd>{rollback_available}</dd></div></dl>{remediation}</section></div>"#,
        endpoint = escape(endpoint),
        variant = status.variant.id(),
        announcement = escape(status.state.announcement()),
        target = escape(&status.target_environment),
        step = escape(&status.current_step),
        drift = escape(&status.drift_status),
        rollback_available = rollback_available,
        remediation = remediation,
    )
}
#[cfg(any(feature = "ssr", test))]
fn static_finops_anchor_board() -> &'static str {
    r#"<div class="trust-anchor-board finops-trust-board" aria-label="FD-001 FinOps and Oyatie Cloud substrate proof"><div class="trust-anchor-grid"><article class="trust-anchor-card selected" data-trust-proof-card="finops-fd001"><p class="screen-anchor">FD-001 WORKLOAD ECONOMY</p><h5>Product delivery remains the north star</h5><p>Run-rate is shown per FD-001 tenant workload: Workflow, Messenger, Mail, Community, Intelligence, and audit services stay in one delivery envelope.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="stage-budget">Stage budget</button><button type="button" data-trust-proof-action="route-finance">Finance close</button></div></article><article class="trust-anchor-card" data-trust-proof-card="finops-cloud"><p class="screen-anchor">OYATIE CLOUD SUBSTRATE</p><h5>Costs prove real tenant hosting</h5><p>Compute, network, storage, audit, residency, and release gates expose hyperscaler-grade posture before FD-001 workloads claim production readiness.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-cloud">Cloud topology</button><button type="button" data-trust-proof-action="route-policy">Policy gate</button></div></article><article class="trust-anchor-card" data-trust-proof-card="finops-local"><p class="screen-anchor">LOCAL-ONLY FINOPS</p><h5>Interactive budget controls, no spend mutation</h5><p>Operators can stage commitments, tag anomalies, and brief reviewers visually; no billing, procurement, deploy, DNS, or cloud mutation executes.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-audit">Audit receipt</button><button type="button" data-trust-proof-action="route-evidence">Evidence spine</button></div></article></div><div class="trust-anchor-footer"><span data-trust-proof-status="true">FinOps ready · FD-001 microservices dogfood Oyatie Cloud as tenant workloads with local-only controls.</span><div class="trust-anchor-routes" aria-label="FinOps connected routes"><button type="button" data-trust-proof-action="route-inventory">Resources</button><button type="button" data-trust-proof-action="route-gates">Gates</button><button type="button" data-trust-proof-action="route-mail">Reviewer Mail</button><button type="button" data-trust-proof-action="route-community">Community</button></div></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_resource_inventory_anchor_board() -> &'static str {
    r#"<div class="trust-anchor-board" aria-label="FD-001 resource inventory and Oyatie Cloud substrate proof"><div class="trust-anchor-grid"><article class="trust-anchor-card selected" data-trust-proof-card="resource-fd001"><p class="screen-anchor">FD-001 SERVICE FLEET</p><h5>Microservices are tenant workloads</h5><p>Tenant admin, workflow runner, audit vault, Mail, Messenger, Community, and Intelligence assets are tracked as one FD-001 product fleet.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="link-resource">Link resource</button><button type="button" data-trust-proof-action="route-catalog">Service catalog</button></div></article><article class="trust-anchor-card" data-trust-proof-card="resource-cloud"><p class="screen-anchor">OYATIE CLOUD INVENTORY</p><h5>Substrate owns residency and release posture</h5><p>Each resource shows cell, owner, cost, risk, policy, deployment gate, and audit receipt so the cloud substrate can prove real tenant hosting.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-finops">FinOps cost</button><button type="button" data-trust-proof-action="route-gates">Admission gates</button></div></article><article class="trust-anchor-card" data-trust-proof-card="resource-local"><p class="screen-anchor">LOCAL-ONLY RESOURCE OPS</p><h5>Inspect without provider mutation</h5><p>Operators can inspect ownership, route evidence, and preview remediation; no cloud provider, database, deploy, billing, or audit mutation executes.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-audit">Audit ledger</button><button type="button" data-trust-proof-action="trace-lineage">Trace lineage</button></div></article></div><div class="trust-anchor-footer"><span data-trust-proof-status="true">Resource inventory ready · FD-001 workload fleet is hosted-proof on Oyatie Cloud with local visual controls only.</span><div class="trust-anchor-routes" aria-label="Resource inventory connected routes"><button type="button" data-trust-proof-action="route-workflow">Workflow</button><button type="button" data-trust-proof-action="route-evidence">Evidence</button><button type="button" data-trust-proof-action="route-mail">Mail</button><button type="button" data-trust-proof-action="route-community">Community</button></div></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_control_plane_resource_explorer_operation_center() -> &'static str {
    r#"<section id="resource-explorer" class="panel resource-explorer-panel" data-fixture-kind="resource-explorer-resource" aria-labelledby="resource-explorer-title"><div class="panel-header"><div><p class="eyebrow">Control plane · fixture only</p><h3 id="resource-explorer-title">Resource Explorer</h3><p>Read-only tenant-scoped index for one representative resource type; no provider CRUD, billing, quota, Cedar runtime, or production persistence is wired.</p></div><span class="status-chip warning">fixture-only non-claim</span></div><p class="scope-notice">Organization → Account → Project → Region → Cell → Resource Group → Resource</p><div class="catalog-toolbar" aria-label="Resource Explorer filters"><label class="catalog-search"><span aria-hidden="true">⌕</span><input type="search" aria-label="Search ORN" value="orn:oya:us-east-2:acct-northwind-prod:cloud-compute:k8s-cluster/cluster-nw-app-01" /></label><span class="catalog-status">GET /v1/cloud-control-plane/resources · X-Oyatie-API-Version: 2026-07-01 (mock)</span></div><article class="resource-explorer-card" data-resource-type="K8sCluster"><span class="status-chip success">K8sCluster</span><strong>cluster-nw-app-01</strong><code>orn:oya:us-east-2:acct-northwind-prod:cloud-compute:k8s-cluster/cluster-nw-app-01</code><dl><div><dt>Organization</dt><dd>org-oyatie-northwind</dd></div><div><dt>Account</dt><dd>acct-northwind-prod</dd></div><div><dt>Project</dt><dd>prj-fd001-control</dd></div><div><dt>Region / Cell</dt><dd>us-east-2 / cell-us-east-2a</dd></div><div><dt>Resource group</dt><dd>rg-platform-admin</dd></div><div><dt>Owner</dt><dd>tenant-admin-platform</dd></div></dl></article><div class="resource-facet-grid" aria-label="Resource detail facets"><span data-resource-facet="lifecycle">lifecycle · ACTIVE fixture</span><span data-resource-facet="identity">identity · workload principal reference only</span><span data-resource-facet="policy">policy · Cedar default-deny expectation</span><span data-resource-facet="quota">quota · mock CPU/memory ceiling</span><span data-resource-facet="billing">billing · mock meter labels only</span><span data-resource-facet="audit">audit · REC-CTRL-001A</span><span data-resource-facet="observability">observability · trace fixture</span><span data-resource-facet="rollback">rollback · compensation pending</span><span data-resource-facet="reconciliation">reconciliation · controller-owned later</span></div><aside class="resource-access-rules" aria-label="Data access rules"><p>Cedar default-deny expectation</p><p>Cross-tenant rows are redacted</p><p>Support access is constrained</p><p>No secret material is exposed</p></aside></section><section id="operation-center" class="panel operation-center-panel" data-fixture-kind="operation-timeline-entry" aria-labelledby="operation-center-title"><div class="panel-header"><div><p class="eyebrow">Operation Center · AIP-151 fixture</p><h3 id="operation-center-title">Operation Center</h3><p>Operation-ledger timeline for create/update/delete attempts; read-only fixture rows until API-001/resource-provider integration.</p></div><span class="status-chip ai">jsonschema.cloud-control-plane.fixture.operation-timeline-entry</span></div><ol class="ops-timeline" aria-label="Operation ledger timeline"><li><time>2026-07-01T09:18:00Z</time><strong>op-20260701-0001 · current state accepted · fixture row</strong><span>client idempotency key idem-ctrlplane-001 · actor tenant-admin-platform · requested mutation create K8sCluster</span><span>timestamps accepted/queued/running fixture · retry classification policy · rollback / compensation pending</span><span>audit-chain pointer REC-CTRL-001A · error / remediation metadata policy reviewer required</span></li><li><time>2026-07-01T09:42:00Z</time><strong>op-20260701-0002 · current state failed · policy denied fixture row</strong><span>client idempotency key idem-ctrlplane-002 · actor support-engineer-redacted · requested mutation update labels</span><span>timestamps accepted/failed fixture · retry classification operator_required · rollback / compensation not-started</span><span>audit-chain pointer REC-CTRL-001B · error / remediation metadata support scope redacted</span></li></ol><div class="policy-decision-strip" aria-label="Persona user-story paths"><article><strong>tenant admin path</strong><p>Search ORN, inspect facets, route reviewer work without live mutation.</p></article><article><strong>operator path</strong><p>Inspect operation retries, rollback state, and remediation metadata.</p></article><article><strong>developer path</strong><p>Use the ORN and operation id from the service catalog/dev portal flow.</p></article><article><strong>security/compliance reviewer path</strong><p>Verify audit, policy, retention, jurisdiction, and no-secret posture.</p></article><article><strong>support engineer path</strong><p>See constrained redacted support context without cross-tenant leakage.</p></article></div><p class="cockpit-note">ADR-0536 D-4/D-5 · specs/masterplan.json:144-257 · specs/cloud-control-plane-canonical.json · specs/cloud-resource-catalog-target.json · specs/api-contract-ssot-canonical.json · ADR-0393 · CONTROLPLANE-UX-001 spec comment</p></section>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_tenant_operations_cockpit(envelope: &TenantRenderEnvelope) -> String {
    let healthcare_gate = if envelope.accreditation.healthcare_enabled {
        "Healthcare surfaces enabled"
    } else {
        "Healthcare surfaces gated"
    };
    let visible_modules = envelope.modules.len();

    format!(
        r#"<section id="cloud-ops-cockpit" class="ops-cockpit panel" aria-labelledby="ops-cockpit-title"><div class="panel-header cockpit-header"><div><p class="eyebrow">Operate</p><h3 id="ops-cockpit-title">Cloud, policy, and FinOps cockpit</h3></div><div class="cockpit-tabs" role="tablist" aria-label="Operations cockpit views"><button type="button" class="active" data-cockpit-tab="topology" role="tab" aria-selected="true">Topology</button><button type="button" data-cockpit-tab="policy" role="tab" aria-selected="false">Policy</button><button type="button" data-cockpit-tab="finops" role="tab" aria-selected="false">FinOps</button></div></div><div class="cockpit-panels"><aside class="ops-cluster-health-fixture" data-ops-cluster-health-source="typed-api" aria-label="Typed ops cluster health fixture"><code>GET /ops/v1/clusters/{{cluster_id}}/health</code><strong>cell-us-east-2</strong><span>yellow · observed 2026-07-01T05:00:00Z · typed ops API</span></aside><article class="cockpit-panel active" data-cockpit-panel="topology" aria-labelledby="cloud-topology-title"><div class="cockpit-column-head"><p class="screen-anchor">CLOUD TOPOLOGY</p><h4 id="cloud-topology-title">Tenant runtime map</h4></div><div class="topology-map" aria-hidden="true"><span class="region primary">us-east-2<em>cell active</em></span><span class="region">eu-west-1<em>warm standby</em></span><span class="region">kr-seoul<em>pack gated</em></span><span class="service compute">Compute</span><span class="service network">Network</span><span class="service storage">Storage</span><span class="service audit">Audit chain</span></div>{ops_command_matrix}<div class="ops-metrics-strip" aria-label="Cloud operations live posture"><span><small>Availability</small><strong>99.96%</strong><em>+0.01 vs SLO</em></span><span><small>Pending rollbacks</small><strong>2</strong><em>1 network · 1 key</em></span><span><small>Run-rate</small><strong>$48.2k</strong><em>4% under commit</em></span><span><small>Evidence age</small><strong>12m</strong><em>fresh</em></span></div><div class="topology-detail-grid"><article><p class="screen-anchor">INCIDENT THREAD</p><ol class="ops-timeline"><li><time>09:18</time><strong>Mesh split detected</strong><span>northwind-prod-mesh · rollback evidence requested</span></li><li><time>09:42</time><strong>DNS policy verified</strong><span>tenant-control-plane routes stay global</span></li><li><time>10:05</time><strong>Audit sidecar healthy</strong><span>receipt vault sealed draft attached</span></li></ol></article><article><p class="screen-anchor">RUNBOOK QUEUE</p><div class="runbook-list"><button type="button" data-cockpit-action="reconcile-cell">Reconcile cell evidence</button><button type="button" data-cockpit-action="simulate-failover">Simulate failover</button><button type="button" data-cockpit-action="queue-runbook">Queue rollback runbook</button></div></article><article><p class="screen-anchor">REGIONAL CAPACITY</p><div class="capacity-bars"><span style="--bar: 72%"><em>us-east-2</em></span><span style="--bar: 44%"><em>eu-west-1</em></span><span style="--bar: 28%"><em>kr-seoul</em></span></div></article></div><div class="cockpit-actions"><button type="button" data-sidepeek-trigger="topology" data-sidepeek-title="Tenant runtime map" data-sidepeek-id="CELL-US-EAST-2" data-sidepeek-desc="Primary cell running compute, network, storage, and audit-chain staged surfaces." data-sidepeek-owner="Cloud infrastructure" data-sidepeek-risk="Medium" data-sidepeek-sla="99.95% target · local data">Inspect cell</button><button type="button" data-command-trigger="true">Search resources</button><span class="cockpit-status" data-cockpit-status="true">Topology ready · local runbooks only.</span></div></article><article id="policy-access" class="cockpit-panel" data-cockpit-panel="policy" aria-labelledby="policy-access-title"><div class="cockpit-column-head"><p class="screen-anchor">POLICY &amp; ACCESS</p><h4 id="policy-access-title">Policy envelope command board</h4></div><div class="policy-command-grid" aria-label="FD-001 and Oyatie Cloud policy proof"><article class="policy-command-card selected" data-policy-card="fd001"><div><p class="screen-anchor">FD-001 TENANT</p><h5>Product delivery stays the goal</h5><p>Messenger, Mail, Community, Workflow, Ontology, and Intelligence run as tenant workloads; Oyatie Cloud proves they can be hosted without moving the tenant workload north star.</p></div><div class="policy-command-actions" aria-label="FD-001 policy routes"><button type="button" data-policy-anchor-action="role-review">Review role grants</button><button type="button" data-policy-anchor-action="open-identity">Open identity</button><button type="button" data-policy-anchor-action="route-evidence">Evidence spine</button></div></article><article class="policy-command-card" data-policy-card="substrate"><div><p class="screen-anchor">OYATIE CLOUD</p><h5>Dogfood substrate boundary</h5><p>Cloud controls stay tenant-scoped, PIPA-aware, auditable, and local-only until real FD-001 services are admitted through release gates.</p></div><div class="policy-command-actions" aria-label="Oyatie Cloud policy routes"><button type="button" data-policy-anchor-action="route-cloud">Cloud topology</button><button type="button" data-policy-anchor-action="pipa-boundary">PIPA boundary</button><button type="button" data-policy-anchor-action="open-audit">Audit trail</button></div></article><article class="policy-command-card" data-policy-card="autonomy"><div><p class="screen-anchor">AUTONOMY CEILING</p><h5>Interactive, never wired</h5><p>Policy can preview allow, gate, deny, rollback, and reviewer paths, but every action is visual state with no cloud, billing, DNS, or workflow mutation.</p></div><div class="policy-command-actions" aria-label="Autonomy policy routes"><button type="button" data-policy-anchor-action="autonomy-ceiling">Show ceiling</button><button type="button" data-policy-anchor-action="residency">Residency pack</button><button type="button" data-policy-anchor-action="route-mail">Mail brief</button></div></article></div><table class="policy-table"><thead><tr><th>Subject</th><th>Scope</th><th>Decision</th><th>Reason</th></tr></thead><tbody><tr><td>Tenant admin</td><td>Cloud controls</td><td><span class="status-chip success">Allow</span></td><td>Owner role</td></tr><tr><td>{role}</td><td>Healthcare</td><td><span class="status-chip warning">{healthcare_gate}</span></td><td>Accreditation</td></tr><tr><td>Workflow builder</td><td>Execution</td><td><span class="status-chip danger">Deny</span></td><td>Autonomy ceiling</td></tr></tbody></table><div class="policy-evidence-grid"><span><strong>12</strong><small>Cedar rules mirrored</small></span><span><strong>7</strong><small>tenant pack grants</small></span><span><strong>3</strong><small>human review stops</small></span></div><div class="policy-decision-strip" aria-label="Policy decision proof path"><article class="policy-decision-card" data-policy-card="allow"><span class="status-chip success">Allow</span><strong>Tenant admin → Cloud controls</strong><p>Owner-scoped controls stay inside the dogfood substrate and attach receipt IDs before promotion.</p><button type="button" data-policy-anchor-action="route-cloud">Inspect controls</button></article><article class="policy-decision-card" data-policy-card="gate"><span class="status-chip warning">Gate</span><strong>{role} → regulated data</strong><p>{healthcare_gate} · reviewer evidence and residency pack required before any FD-001 workload placement.</p><button type="button" data-policy-anchor-action="residency">Review gate</button></article><article class="policy-decision-card" data-policy-card="deny"><span class="status-chip danger">Deny</span><strong>Workflow builder → execution</strong><p>The autonomy ceiling blocks real execution; visual routing proves the UX without wiring side effects.</p><button type="button" data-policy-anchor-action="autonomy-ceiling">Trace denial</button></article></div><div class="policy-anchor-footer"><span class="cockpit-status" data-policy-anchor-status="true">Policy board ready · FD-001 workloads dogfood Oyatie Cloud as tenant surfaces.</span><div class="policy-anchor-routes" aria-label="Connected policy routes"><button type="button" data-policy-anchor-action="route-community">Community review</button><button type="button" data-policy-anchor-action="open-audit">Audit ledger</button><button type="button" data-policy-anchor-action="route-evidence">Evidence graph</button></div></div></article><article id="finops-pane" class="cockpit-panel" data-cockpit-panel="finops" aria-labelledby="finops-title"><div class="cockpit-column-head"><p class="screen-anchor">FINOPS</p><h4 id="finops-title">Run-rate and sustainability</h4></div><div class="finops-bars" aria-label="FinOps breakdown"><span style="--bar: 72%"><em>Compute · $21.4k</em></span><span style="--bar: 51%"><em>Network · $9.8k</em></span><span style="--bar: 43%"><em>Storage · $7.2k</em></span><span style="--bar: 26%"><em>Audit · $3.1k</em></span></div><div class="finops-action-grid"><button type="button" data-cockpit-action="open-commit">Open commit plan</button><button type="button" data-cockpit-action="tag-anomaly">Tag anomaly</button><button type="button" data-cockpit-action="draft-budget-note">Draft budget note</button></div><span class="cockpit-status" data-cockpit-status="true">FinOps ready · local budget actions only.</span>{finops_anchor_board}<p class="cockpit-note">{visible_modules} services visible in this envelope · backend wiring remains disabled</p></article></div></section>"#,
        role = escape(&envelope.role_name),
        healthcare_gate = escape(healthcare_gate),
        visible_modules = visible_modules,
        ops_command_matrix = format!(
            "{}{}",
            static_cloud_ops_command_matrix(),
            static_ops_cluster_health_panel()
        ),
        finops_anchor_board = static_finops_anchor_board(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_trust_center_portal() -> &'static str {
    r#"<section id="trust-center-portal" class="trust-center-portal panel" aria-labelledby="trust-center-title" data-trust-center-fixture="contract-first-slice"><div class="panel-header trust-center-header"><div><p class="eyebrow">Trust Center · customer evidence</p><h3 id="trust-center-title">Trust Center overview</h3><p>Contracted fixture for /specs/trust-center-compliance-evidence-portal.json: tenant-scoped evidence, reviewer room, status link, SBOM/VEX posture, freshness, and access audit.</p></div><span class="status-chip warning">first slice · fixture/API contract</span></div><div class="trust-center-role-strip" aria-label="Trust Center persona paths"><article data-trust-role="tenant-admin"><span class="status-chip success">tenant admin</span><strong>Review posture, filter evidence, invite/revoke reviewers</strong><p>Can inspect summary, open detail, manage bounded reviewer grants, and view access audit.</p></article><article data-trust-role="security-compliance-reviewer"><span class="status-chip">reviewer</span><strong>Read-only evidence room with expiry</strong><p>Reviewer can read approved evidence and safe exports, but cannot administer tenant settings.</p></article><article data-trust-role="oyatie-operator"><span class="status-chip warning">operator</span><strong>Publishability and redaction stay upstream</strong><p>Operator-only detail withheld; UI shows handles, claim tiers, audit refs, and customer-safe summaries.</p></article></div><div class="trust-center-kpi-strip" aria-label="Trust Center evidence freshness summary"><span><strong>3</strong><small>current evidence</small></span><span><strong>2</strong><small>warning/stale</small></span><span><strong>1</strong><small>blocked_missing_evidence · not green</small></span><span><strong>target_non_claim</strong><small>claim tier: target_non_claim</small></span></div><div class="trust-center-toolbar" aria-label="Trust Center filters and public status controls"><label><span aria-hidden="true">⌕</span><input aria-label="Filter trust evidence by class, source, freshness, or claim tier" value="" placeholder="Filter evidence, source, freshness, claim tier..." /></label><div class="trust-filter-pills" role="toolbar" aria-label="Trust Center evidence state filters"><button type="button" class="active" data-trust-filter="all" aria-pressed="true">All</button><button type="button" data-trust-filter="current" aria-pressed="false">Current</button><button type="button" data-trust-filter="stale" aria-pressed="false">Stale</button><button type="button" data-trust-filter="blocked" aria-pressed="false">Blocked</button><button type="button" data-trust-filter="target_non_claim" aria-pressed="false">Target non-claim</button></div><a class="trust-status-link" href="https://status.oyatie.example/northwind">Public status link / embed</a></div><div class="trust-center-layout"><article class="trust-evidence-table-card" aria-labelledby="trust-evidence-table-title"><div class="cockpit-column-head"><p class="screen-anchor">CONTROLS AND EVIDENCE</p><h4 id="trust-evidence-table-title">Evidence table with non-color states</h4></div><table class="resource-table trust-evidence-table" aria-label="Trust Center evidence table"><thead><tr><th>Control</th><th>Source</th><th>Freshness</th><th>Claim tier</th><th>Audit ref</th><th>Detail</th></tr></thead><tbody><tr data-trust-evidence-state="current"><td><strong>SOC2 CC7.2 validation</strong><small>security_validation_controls</small></td><td>security-validation matrix</td><td><span class="status-chip success" aria-label="freshness state current">current · source 2026-07-01T07:42Z</span></td><td>mechanically_enforced</td><td><code>aud-trust-ctrl-cc72-001</code></td><td><button type="button" aria-label="Open SOC2 CC7.2 evidence detail">Open detail</button></td></tr><tr data-trust-evidence-state="stale"><td><strong>DR drill floor</strong><small>slo_dr_status_incident</small></td><td>compliance-pack floors</td><td><span class="status-chip warning" aria-label="freshness state stale warning">stale · refresh required</span></td><td>spec_ready</td><td><code>aud-trust-dr-floor-014</code></td><td><button type="button" aria-label="Open DR drill stale evidence detail">Open detail</button></td></tr><tr data-trust-evidence-state="blocked_missing_evidence"><td><strong>Quality kit abuse drill</strong><small>cloud_quality_kits</small></td><td>quality-kit evidence catalogue</td><td><span class="status-chip danger" aria-label="freshness state blocked missing evidence">blocked_missing_evidence · not green</span></td><td>target_non_claim</td><td><code>gap-trust-qk-abuse-001</code></td><td><button type="button" aria-label="Open missing quality kit evidence gap detail">Open detail</button></td></tr><tr data-trust-evidence-state="target_non_claim"><td><strong>External certification display</strong><small>claim boundary</small></td><td>trust-center spec</td><td><span class="status-chip" aria-label="target non claim not certified">target_non_claim · no certification</span></td><td>claim tier: target_non_claim</td><td><code>nonclaim-trust-cert-000</code></td><td><button type="button" aria-label="Open non certification boundary detail">Open detail</button></td></tr></tbody></table></article><aside class="trust-detail-card" aria-labelledby="trust-detail-title"><p class="screen-anchor">EVIDENCE DETAIL</p><h4 id="trust-detail-title">Customer-safe detail pane</h4><dl class="service-kv"><div><dt>source_system</dt><dd>security-validation matrix</dd></div><div><dt>source_record_ref</dt><dd>secval://lane/api-fuzz/latest</dd></div><div><dt>freshness_state</dt><dd>current</dd></div><div><dt>audit_event_ref</dt><dd>aud-trust-ctrl-cc72-001</dd></div><div><dt>redaction</dt><dd>raw scanner output redacted · operator-only detail withheld</dd></div></dl><p class="scope-notice">Cross-tenant route denied; payload tenant assertions are display-only and never authority.</p><p>keyboard table/detail/filter/reviewer grant paths labelled</p></aside></div><div class="trust-lower-grid"><article class="trust-sbom-card" aria-labelledby="trust-sbom-title"><p class="screen-anchor">SBOM / VEX</p><h4 id="trust-sbom-title">SBOM/VEX posture</h4><ul class="evidence-steps"><li><span>SBOM</span><strong>signed SBOM present</strong></li><li><span>VEX</span><strong>customer-safe VEX summary · exploit details withheld</strong></li><li><span>SLA</span><strong>2 remediation SLA exceptions expire in 6 days</strong></li></ul></article><article class="trust-reviewer-card" aria-labelledby="trust-reviewer-title" data-reviewer-grant-flow="create-revoke-expiring"><p class="screen-anchor">REVIEWER GRANT</p><h4 id="trust-reviewer-title">Reviewer invite</h4><label><span>Reviewer email</span><input aria-label="Reviewer email" type="email" value="security.reviewer@example.com" /></label><label><span>Expiry</span><input aria-label="Reviewer grant expiry" type="datetime-local" value="2026-07-08T00:00" /></label><div class="trust-card-actions"><button type="button" aria-label="Create expiring reviewer grant">Invite reviewer</button><button type="button" aria-label="Revoke reviewer access">Revoke reviewer access</button></div><p>Reviewer cannot administer tenant settings; grant is bounded by purpose, scope, and expiry.</p></article><article class="trust-access-audit-card" aria-labelledby="trust-access-audit-title"><p class="screen-anchor">ACCESS AUDIT</p><h4 id="trust-access-audit-title">Access audit</h4><ol class="audit-ledger-list compact"><li><time>07:42</time><span class="status-chip success">viewed</span><strong>trust_center.evidence_index_viewed</strong><p>tenant_admin · aud-trust-index-001</p></li><li><time>07:48</time><span class="status-chip warning">grant</span><strong>trust_center.access_grant_created</strong><p>expiry 2026-07-08 · aud-trust-grant-002</p></li><li><time>08:05</time><span class="status-chip danger">denied</span><strong>Cross-tenant route denied</strong><p>other tenant evidence room request failed closed</p></li></ol></article></div></section>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_audit_evidence_timeline_panel() -> String {
    let source = audit_evidence_timeline_from_durable_source();
    let rows = source.timeline_rows();
    let row_html = rows
        .iter()
        .map(static_audit_evidence_timeline_row)
        .collect::<String>();

    format!(
        r#"<section id="audit-evidence-timeline" class="audit-evidence-timeline-panel panel" aria-labelledby="audit-evidence-timeline-title" data-audit-evidence-source="durable-cloud-observability-audit-api" data-audit-evidence-endpoint="{endpoint}" data-authz-surface="{authz_surface}" data-audit-operation-class="{operation_class}" data-authority-ref="{authority_ref}" data-source-anchors="{source_anchors}" data-authority-boundary="{authority_boundary}" data-chain-complete="{chain_complete}" data-high-watermark-sequence="{high_watermark}"><div class="panel-header audit-evidence-header"><div><p class="eyebrow">Audit Evidence · durable source</p><h3 id="audit-evidence-timeline-title">Cloud IAM policy evidence timeline</h3><p>Read-only projection from the Cloud Observability audit read API; the UI renders signed digest-chain evidence for one operation class without provider mutation.</p><p class="scope-notice">Authority: contracts/openapi/cloud/cloud-observability-audit-v1.yaml · G007/AUTHZ-006 slice · analysis-only-non-mutating.</p></div><span class="status-chip success">chain complete</span></div><div class="audit-evidence-source-strip" aria-label="Durable audit source metadata"><span><strong>{record_count}</strong><small>records</small></span><span><strong>{request_id}</strong><small>request</small></span><span><strong>{tenant_id}</strong><small>tenant</small></span><span><strong>{region}</strong><small>region</small></span><span><strong>{next_cursor}</strong><small>next cursor</small></span></div><section class="ds-audit-evidence-timeline" data-variant="changeset-provenance" aria-label="Audit evidence timeline"><ol>{row_html}</ol></section></section>"#,
        endpoint = escape(source.endpoint()),
        authz_surface = escape(source.authz_surface()),
        operation_class = escape(source.operation_class()),
        authority_ref = escape(source.authority_ref()),
        source_anchors = escape(source.source_anchors()),
        authority_boundary = escape(source.authority_boundary()),
        chain_complete = source.metadata.chain_complete,
        high_watermark = escape(&source.high_watermark_sequence_label()),
        record_count = source.metadata.record_count,
        request_id = escape(&source.metadata.request_id),
        tenant_id = escape(&source.metadata.tenant_id),
        region = escape(&source.metadata.region),
        next_cursor = escape(source.metadata.next_cursor.as_deref().unwrap_or("none")),
        row_html = row_html,
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_audit_evidence_timeline_row(row: &EvidenceRow) -> String {
    let severity = row_severity(row);
    let severity_label = match severity {
        RowSeverity::Ok => "ok",
        RowSeverity::Warning => "warning",
        RowSeverity::Blocking => "blocking",
    };
    let evidence = row
        .evidence
        .as_ref()
        .map(EvidencePath::display)
        .unwrap_or_else(|| "evidence missing".to_owned());
    let alert_role = if severity == RowSeverity::Blocking {
        " role=\"alert\""
    } else {
        ""
    };
    let blocking_gap = if severity == RowSeverity::Blocking {
        "<p class=\"ds-blocking-gap\">Blocking gap: this row must be resolved before the timeline can certify</p>"
    } else {
        ""
    };

    format!(
        r#"<li data-severity="{severity}"{alert_role}><time>{timestamp}</time><strong>{event_type}</strong><span class="ds-evidence-path">{evidence}</span><span class="ds-signature-status">{signature_status}</span><span class="ds-linked-change">{linked_change_id}</span>{blocking_gap}</li>"#,
        severity = severity_label,
        alert_role = alert_role,
        timestamp = escape(&row.timestamp),
        event_type = escape(&row.event_type),
        evidence = escape(&evidence),
        signature_status = escape(&row.signature_status),
        linked_change_id = escape(&row.linked_change_id),
        blocking_gap = blocking_gap,
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_resource_audit_console(envelope: &TenantRenderEnvelope) -> String {
    let resources = resource_inventory_rows();
    let audit_events = audit_receipts();
    let gates = deployment_gates();
    let resource_rows = resources
        .iter()
        .map(static_resource_row)
        .collect::<String>();
    let audit_rows = audit_events
        .iter()
        .map(static_audit_receipt)
        .collect::<String>();
    let gate_cards = gates.iter().map(static_deployment_gate).collect::<String>();

    format!(
        r#"<section id="resource-audit-console" class="resource-audit-console panel" aria-labelledby="resource-audit-title"><div class="panel-header resource-console-header"><div><p class="eyebrow">Operate · Trust</p><h3 id="resource-audit-title">Resource inventory, audit ledger, and deployment gates</h3></div><div class="resource-tabs" role="tablist" aria-label="Resource and audit console views"><button type="button" class="active" data-resource-tab="inventory" role="tab" aria-selected="true">Inventory</button><button type="button" data-resource-tab="audit" role="tab" aria-selected="false">Audit ledger</button><button type="button" data-resource-tab="gates" role="tab" aria-selected="false">Deployment gates</button></div></div><div class="resource-console-spine" aria-label="Console summary"><span><strong>{resource_count}</strong> resources</span><span><strong>{audit_count}</strong> receipts staged</span><span><strong>{module_count}</strong> modules visible</span><span><strong>{approval_count}</strong> approvals linked</span></div><div class="resource-toolbar" aria-label="Resource console controls"><label><span aria-hidden="true">⌕</span><input data-resource-search="true" aria-label="Search resources and receipts" placeholder="Search resource, owner, region, receipt..." /></label><div class="resource-filter-pills" role="toolbar" aria-label="Resource state filters"><button type="button" class="active" data-resource-filter="all">All</button><button type="button" data-resource-filter="attention">Attention</button><button type="button" data-resource-filter="review">Review</button><button type="button" data-resource-filter="active">Active</button></div><div class="resource-actions"><button type="button" data-resource-action="refresh">Refresh data</button><button type="button" data-resource-action="export">Export CSV</button></div><span data-resource-status="true">6 visible · local inventory only</span></div><div class="resource-panels"><article id="resource-inventory" class="resource-panel active" data-resource-panel="inventory" aria-labelledby="resource-inventory-title"><div class="cockpit-column-head"><p class="screen-anchor">RESOURCE INVENTORY</p><h4 id="resource-inventory-title">Tenant assets with ownership, region, cost, and risk</h4></div><table class="resource-table"><thead><tr><th>Kind</th><th>Name</th><th>Region</th><th>Owner</th><th>State</th><th>Monthly</th><th>Action</th></tr></thead><tbody>{resource_rows}</tbody></table>{resource_inventory_anchor_board}</article><article id="audit-ledger" class="resource-panel" data-resource-panel="audit" aria-labelledby="audit-ledger-title"><div class="cockpit-column-head"><p class="screen-anchor">AUDIT LEDGER</p><h4 id="audit-ledger-title">Immutable tenant-workload proof stream</h4></div>{receipt_stitching_console}<div class="audit-proof-grid" aria-label="FD-001 tenant workload receipt proof"><article class="audit-proof-card selected" data-audit-card="fd001"><p class="screen-anchor">FD-001 RECEIPTS</p><h5>Product delivery remains master plan</h5><p>Every Messenger, Mail, Community, Workflow, Ontology, and Intelligence preview action creates a visible receipt so FD-001 can be dogfooded as a real tenant workload.</p><div class="audit-command-actions"><button type="button" data-audit-anchor-action="open-evidence">Open evidence</button><button type="button" data-audit-anchor-action="route-mail">Mail brief</button></div></article><article class="audit-proof-card" data-audit-card="cloud"><p class="screen-anchor">OYATIE CLOUD</p><h5>Oyatie Cloud substrate proves hosting posture</h5><p>The cloud substrate records residency, release, cost, policy, and rollback checks before a tenant surface can claim production readiness.</p><div class="audit-command-actions"><button type="button" data-audit-anchor-action="route-cloud">Cloud topology</button><button type="button" data-audit-anchor-action="route-gates">Release gates</button></div></article><article class="audit-proof-card" data-audit-card="sealed"><p class="screen-anchor">SEALED PACKET</p><h5>Interactive local receipt vault</h5><p>Operators can inspect, seal, route, and brief a receipt packet visually while backend, billing, deploy, and cloud mutations remain disconnected.</p><div class="audit-command-actions"><button type="button" data-audit-anchor-action="seal-packet">Seal packet</button><button type="button" data-audit-anchor-action="route-policy">Policy board</button></div></article></div><ol class="audit-ledger-list">{audit_rows}</ol><div class="audit-anchor-footer"><span data-audit-anchor-status="true">Audit ledger ready · FD-001 tenant workload receipts remain local visual evidence.</span><div class="audit-command-actions" aria-label="Audit ledger connected routes"><button type="button" data-audit-anchor-action="route-workflow">Workflow proof</button><button type="button" data-audit-anchor-action="route-community">Community review</button><button type="button" data-audit-anchor-action="open-evidence">Evidence graph</button></div></div></article><article id="deployment-gates" class="resource-panel" data-resource-panel="gates" aria-labelledby="deployment-gates-title"><div class="cockpit-column-head"><p class="screen-anchor">DEPLOYMENT GATES</p><h4 id="deployment-gates-title">FD-001 tenant workload admission gates</h4></div>{deployment_gate_command_board}<div class="gate-grid">{gate_cards}</div><div class="deployment-gate-footer"><span data-deployment-gate-status="true">Deployment gates ready · FD-001 microservices are tenant workloads on Oyatie Cloud.</span><div class="deployment-gate-routes" aria-label="Deployment gate connected routes"><button type="button" data-deployment-gate-action="route-policy">Policy envelope</button><button type="button" data-deployment-gate-action="route-audit">Audit packet</button><button type="button" data-deployment-gate-action="route-community">Community note</button><button type="button" data-deployment-gate-action="route-cloud">Cloud cells</button></div></div></article></div></section>"#,
        resource_count = resources.len(),
        audit_count = audit_events.len(),
        module_count = envelope.modules.len(),
        approval_count = envelope.approvals.len(),
        resource_rows = resource_rows,
        audit_rows = audit_rows,
        gate_cards = gate_cards,
        deployment_gate_command_board = static_deployment_gate_command_board(),
        resource_inventory_anchor_board = static_resource_inventory_anchor_board(),
        receipt_stitching_console = static_receipt_stitching_console(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_resource_row(row: &ResourceRow) -> String {
    format!(
        r#"<tr data-resource-row="true" data-resource-state="{state}"><td><span class="resource-kind">{kind}</span></td><td><strong>{name}</strong><small>{description}</small></td><td>{region}</td><td>{owner}</td><td><span class="{state_class}">{state}</span></td><td class="numeric">{monthly}</td><td><button type="button" data-sidepeek-trigger="resource" data-sidepeek-title="{name}" data-sidepeek-id="{side_id}" data-sidepeek-desc="{description}" data-sidepeek-owner="{owner}" data-sidepeek-risk="{risk}" data-sidepeek-sla="Inventory staged · no live mutation">Inspect</button></td></tr>"#,
        kind = escape(row.kind),
        name = escape(row.name),
        description = escape(row.description),
        region = escape(row.region),
        owner = escape(row.owner),
        state_class = resource_status_class(row.state),
        state = escape(row.state),
        monthly = escape(row.monthly),
        side_id = escape(row.side_id),
        risk = escape(row.risk),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_audit_receipt(item: &AuditReceipt) -> String {
    format!(
        r#"<li><time>{time}</time><span class="{severity_class}">{severity}</span><strong>{event}</strong><p>{actor}</p><code>{receipt}</code><button type="button" data-audit-anchor-action="inspect-receipt">Inspect</button></li>"#,
        time = escape(item.time),
        severity_class = resource_status_class(item.severity),
        severity = escape(item.severity),
        event = escape(item.event),
        actor = escape(item.actor),
        receipt = escape(item.receipt),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_receipt_stitching_console() -> &'static str {
    r#"<section class="receipt-stitching-console" data-receipt-stitching-console="true" aria-label="FD-001 and Oyatie Cloud receipt stitching console"><div class="receipt-stitching-head"><div><p class="screen-anchor">RECEIPT STITCHING CONSOLE</p><h5>Every product action returns to one proof stream</h5><span data-receipt-stitching-status="true">Workflow output, Work Hub drafts, Cloud workload posture, and Deployment gates are ready to stitch locally.</span></div><button type="button" data-receipt-stitching-action="seal">Seal visible packet</button></div><div class="receipt-stitching-grid" aria-label="Receipt source routes"><button type="button" class="selected" data-receipt-source="workflow" data-receipt-title="Workflow output bundle" data-receipt-id="REC-FD001-WF-018" data-receipt-route="Workflow → Messenger/Mail/Community → Evidence" data-receipt-owner="Workflow Studio" data-receipt-state="review"><span>01 · WORKFLOW</span><strong>Run output bundle</strong><em>REC-FD001-WF-018</em></button><button type="button" data-receipt-source="comms" data-receipt-title="Work Hub handoff draft" data-receipt-id="REC-COMMS-HANDOFF-006" data-receipt-route="Messenger/Mail/Community draft handoff" data-receipt-owner="Work Hub" data-receipt-state="draft"><span>02 · COMMS</span><strong>Draft handoff proof</strong><em>REC-COMMS-HANDOFF-006</em></button><button type="button" data-receipt-source="cloud" data-receipt-title="Cloud tenant workload posture" data-receipt-id="REC-FD001-CLOUD-009" data-receipt-route="Oyatie Cloud workload plane → gates" data-receipt-owner="Cloud substrate" data-receipt-state="sealed"><span>03 · CLOUD</span><strong>Tenant workload proof</strong><em>REC-FD001-CLOUD-009</em></button><button type="button" data-receipt-source="gates" data-receipt-title="Deployment admission packet" data-receipt-id="REC-DEPLOY-GATE-014" data-receipt-route="Jenkins → ArgoCD → Cosign → Audit" data-receipt-owner="Release governance" data-receipt-state="gate"><span>04 · GATES</span><strong>Admission proof</strong><em>REC-DEPLOY-GATE-014</em></button></div><aside class="receipt-stitching-detail" aria-label="Selected receipt stitch detail"><dl><div><dt>Selected</dt><dd data-receipt-detail-title="true">Workflow output bundle</dd></div><div><dt>Receipt</dt><dd data-receipt-detail-id="true">REC-FD001-WF-018</dd></div><div><dt>Route</dt><dd data-receipt-detail-route="true">Workflow → Messenger/Mail/Community → Evidence</dd></div><div><dt>Owner</dt><dd data-receipt-detail-owner="true">Workflow Studio</dd></div></dl><div class="receipt-stitching-actions" aria-label="Selected receipt actions"><button type="button" data-receipt-stitching-action="workflow">Workflow</button><button type="button" data-receipt-stitching-action="cloud">Cloud</button><button type="button" data-receipt-stitching-action="mail">Mail brief</button><button type="button" data-receipt-stitching-action="community">Community</button><button type="button" data-receipt-stitching-action="graph">Graph</button><button type="button" data-receipt-stitching-action="gates">Gates</button></div></aside></section>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_deployment_gate_command_board() -> &'static str {
    r#"<div class="deployment-proof-grid" aria-label="FD-001 and Oyatie Cloud deployment proof"><article class="deployment-proof-card selected" data-deployment-card="fd001"><p class="screen-anchor">FD-001 RELEASE TRAIN</p><h5>Product microservices deploy as tenants</h5><p>Messenger, Mail, Community, Workflow, Ontology, Intelligence, and core ops stay product-first; the gates prove they can run as tenant workloads on the substrate.</p><div class="deployment-card-actions"><button type="button" data-deployment-gate-action="admit-fd001">Admit workload</button><button type="button" data-deployment-gate-action="route-workflow">Workflow runbook</button></div></article><article class="deployment-proof-card" data-deployment-card="cloud"><p class="screen-anchor">OYATIE CLOUD</p><h5>Hyperscaler-grade substrate proof</h5><p>Cell topology, policy sidecars, cosign receipts, ArgoCD app health, rollback posture, and audit-chain freshness must be visible before any promotion claim.</p><div class="deployment-card-actions"><button type="button" data-deployment-gate-action="route-cloud">Inspect cells</button><button type="button" data-deployment-gate-action="route-finops">FinOps guard</button></div></article><article class="deployment-proof-card" data-deployment-card="control"><p class="screen-anchor">CONTROL PLANE</p><h5>Interactive, never wired</h5><p>Operators can simulate gate decisions, seal a release packet, and route reviewer work, while deploy, DNS, registry, billing, and cloud mutations remain disconnected.</p><div class="deployment-card-actions"><button type="button" data-deployment-gate-action="seal-release">Seal packet</button><button type="button" data-deployment-gate-action="route-mail">Reviewer mail</button></div></article></div><div class="deployment-promotion-lane" aria-label="Tenant workload promotion lane"><button type="button" class="active" data-deployment-gate-action="ci-lane"><span>01</span><strong>CI mirror</strong><em>Jenkins parity · 92%</em></button><button type="button" data-deployment-gate-action="attest-lane"><span>02</span><strong>Attest</strong><em>cosign + SBOM · 61%</em></button><button type="button" data-deployment-gate-action="admit-lane"><span>03</span><strong>Admit tenant</strong><em>policy + PIPA · review</em></button><button type="button" data-deployment-gate-action="observe-lane"><span>04</span><strong>Observe</strong><em>SLO + audit emit · 48%</em></button></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_deployment_gate(gate: &DeploymentGate) -> String {
    format!(
        r#"<article class="gate-card"><div><span class="{state_class}">{state}</span><h5>{label}</h5><p>{detail}</p></div><span class="gate-progress" style="--bar: {progress}"><em>{progress}</em></span><div class="gate-card-actions"><button type="button" data-gate-action="attach-evidence">Attach evidence</button><button type="button" data-gate-action="open-evidence">Evidence</button><button type="button" data-gate-action="route-owner">Owner route</button></div></article>"#,
        state_class = resource_status_class(gate.state),
        state = escape(gate.state),
        label = escape(gate.label),
        detail = escape(gate.detail),
        progress = escape(gate.progress),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_service_catalog_anchor_board() -> &'static str {
    r#"<div class="trust-anchor-board catalog-trust-board" aria-label="FD-001 service catalog and Oyatie Cloud tenant workload proof"><div class="trust-anchor-grid"><article class="trust-anchor-card selected" data-trust-proof-card="catalog-fd001"><p class="screen-anchor">FD-001 MODULE CONTRACT</p><h5>Catalog is the service graph manifest</h5><p>Core, Workflow, Messenger, Mail, Community, Finance, Identity, Ontology, Intelligence, and Daily Work are presented as one permitted FD-001 tenant workload graph.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="stage-catalog">Stage manifest</button><button type="button" data-trust-proof-action="route-workflow">Workflow proof</button></div></article><article class="trust-anchor-card" data-trust-proof-card="catalog-cloud"><p class="screen-anchor">OYATIE CLOUD ADMISSION</p><h5>Substrate dependencies are visible first</h5><p>Cloud cells, policy gates, resource inventory, deployment gates, FinOps, and audit receipts make hosting readiness explicit before service claims.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-cloud">Cloud substrate</button><button type="button" data-trust-proof-action="route-gates">Deployment gates</button></div></article><article class="trust-anchor-card" data-trust-proof-card="catalog-local"><p class="screen-anchor">LOCAL-ONLY CATALOG OPS</p><h5>Request, pin, and route without provisioning</h5><p>Operators can filter modules, inspect dependencies, pin rows, and route reviewers visually; no service admission, IAM, deploy, billing, or cloud mutation executes.</p><div class="trust-anchor-actions"><button type="button" data-trust-proof-action="route-policy">Policy</button><button type="button" data-trust-proof-action="route-audit">Audit ledger</button></div></article></div><div class="trust-anchor-footer"><span data-trust-proof-status="true">Service catalog ready · FD-001 module graph dogfoods Oyatie Cloud as local tenant workload proof.</span><div class="trust-anchor-routes" aria-label="Service catalog connected routes"><button type="button" data-trust-proof-action="route-finance">Finance</button><button type="button" data-trust-proof-action="route-identity">Identity</button><button type="button" data-trust-proof-action="route-daily">Daily Work</button><button type="button" data-trust-proof-action="route-evidence">Evidence</button></div></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
fn static_developer_portal_provisioning(portal: &DeveloperPortalFixture) -> String {
    let template = &portal.approved_template;
    let artifact_count = portal.generated_artifacts.len();
    let resource_facets = [
        "lifecycle",
        "identity",
        "policy",
        "quota",
        "billing",
        "audit",
        "observability",
        "rollback",
        "reconciliation",
    ]
    .iter()
    .map(|facet| {
        format!(
            r#"<span data-resource-facet="{facet}">{facet}</span>"#,
            facet = escape(facet)
        )
    })
    .collect::<String>();
    let resources = template
        .supported_golden_path_resources
        .iter()
        .map(|resource| {
            format!(
                r#"<span class="status-chip" data-devportal-resource="{resource}">{resource}</span>"#,
                resource = escape(resource)
            )
        })
        .collect::<String>();
    let parameters = template
        .parameters
        .iter()
        .map(|parameter| {
            format!(
                r#"<label><span>{name}</span><input data-devportal-parameter="{name}" aria-label="Template parameter {name}" value="{value}" /><small>{field_type}{required}</small></label>"#,
                name = escape(&parameter.name),
                value = escape(&parameter.default_value),
                field_type = escape(&parameter.field_type),
                required = if parameter.required { " · required" } else { "" }
            )
        })
        .collect::<String>();
    let admission_steps = portal
        .admission_sequence
        .iter()
        .enumerate()
        .map(|(index, step)| {
            format!(
                r#"<li><span>{ordinal:02}</span><strong>{step}</strong></li>"#,
                ordinal = index + 1,
                step = escape(step)
            )
        })
        .collect::<String>();
    let artifacts = portal
        .generated_artifacts
        .iter()
        .map(|artifact| {
            format!(
                r#"<li data-devportal-artifact="{kind}"><strong>{kind}</strong><span>{path}</span><small>{description}</small></li>"#,
                kind = escape(&artifact.artifact_kind),
                path = escape(&artifact.path),
                description = escape(&artifact.description)
            )
        })
        .collect::<String>();
    let audit_events = portal
        .audit_event_fixture
        .iter()
        .map(|event| {
            format!(
                r#"<li><strong>{event_type}</strong><code>{receipt}</code></li>"#,
                event_type = escape(&contract_fixture_label(&event.event_type)),
                receipt = escape(&event.receipt)
            )
        })
        .collect::<String>();
    let role_paths = portal
        .role_coverage
        .iter()
        .map(|role| {
            format!(
                r#"<article><strong>{role}</strong><span>{path}</span></article>"#,
                role = escape(&title_case_role(&role.role)),
                path = escape(&role.path)
            )
        })
        .collect::<String>();
    let facet_rows = portal
        .resource_facets
        .iter()
        .map(|facet| {
            format!(
                r#"<tr data-devportal-facet-row="{resource}"><th scope="row">{resource}</th><td>{lifecycle}</td><td>{identity}</td><td>{policy}</td><td>{quota}</td><td>{billing}</td><td>{audit}</td><td>{observability}</td><td>{rollback}</td><td>{reconciliation}</td></tr>"#,
                resource = escape(&facet.resource),
                lifecycle = escape(&facet.lifecycle),
                identity = escape(&facet.identity),
                policy = escape(&facet.policy),
                quota = escape(&facet.quota),
                billing = escape(&facet.billing),
                audit = escape(&facet.audit),
                observability = escape(&facet.observability),
                rollback = escape(&facet.rollback),
                reconciliation = escape(&facet.reconciliation)
            )
        })
        .collect::<String>();

    format!(
        r#"<section id="developer-portal-provisioning" class="developer-portal-provisioning" data-devportal-fixture="approved-template" aria-labelledby="developer-portal-provisioning-title"><div class="devportal-head developer-portal-head"><div><p class="screen-anchor">DEVELOPER PORTAL · FIXTURE ONLY</p><h4 id="developer-portal-provisioning-title">Approved service template provisioning</h4><h5>Resource Explorer</h5><p>Application developer selects an approved service template, enters fixture parameters, previews quota/cost, and receives an operation receipt without provider-live provisioning.</p><p>Organization → Account → Project → Region → Cell → Resource Group → Resource</p><label aria-label="Search ORN"><span>Search ORN</span><input aria-label="Search ORN" value="orn:oya:us-east-2:acct-northwind-prod:cloud-compute:k8s-cluster/cluster-nw-app-01" /></label></div><span class="status-chip success">fixture-only non-claim</span></div><article class="resource-explorer-card" data-resource-type="K8sCluster"><strong>{service}</strong><code>orn:oya:us-east-2:acct-northwind-prod:cloud-compute:k8s-cluster/cluster-nw-app-01</code><span>{template_id} · {artifact_count} generated artifacts</span></article><div class="resource-facet-grid" aria-label="Resource detail facets">{resource_facets}</div><aside class="resource-access-rules" aria-label="Data access rules"><p>Cedar default-deny expectation</p><p>Cross-tenant rows are redacted</p><p>Support access is constrained</p><p>No secret material is exposed</p></aside><div class="devportal-story-grid"><article class="devportal-template-card" aria-label="Approved template details"><p class="screen-anchor">APPROVED TEMPLATE</p><h5>{template_name}</h5><dl class="service-kv"><div><dt>Template id</dt><dd>{template_id}</dd></div><div><dt>Version</dt><dd>{version}</dd></div><div><dt>Owner</dt><dd>{owner}</dd></div><div><dt>Catalog entity</dt><dd>{entity_kind} · {service_slug}</dd></div></dl><div class="devportal-resource-pills" aria-label="Required resources and generated artifacts">{resources}</div></article><form class="devportal-request-form" aria-label="Create service from approved template fixture"><p class="screen-anchor">APPLICATION DEVELOPER PATH</p><div class="devportal-parameter-grid" aria-label="Template parameters">{parameters}</div><div class="devportal-form-actions"><button type="button" data-devportal-action="preview-quota-cost">Quota and cost preview</button><button type="button" class="primary" data-devportal-action="submit-provisioning-request">Submit provisioning request</button></div><p class="settings-status">Request {request_id} uses idempotency key {idempotency_key}; UI stages no service admission, deploy, billing, IAM, database, or cloud mutation.</p></form><aside class="devportal-operation-card" aria-label="Durable operation result"><p class="screen-anchor">DURABLE OPERATION RESULT</p><h5>{operation_id}</h5><dl class="service-kv"><div><dt>State</dt><dd>{operation_state}</dd></div><div><dt>Preview environment</dt><dd>{preview_id} · {preview_state}</dd></div><div><dt>Evidence</dt><dd>{evidence_receipt}</dd></div><div><dt>Cost preview</dt><dd>Quota and cost preview · {currency} {monthly_major}/mo fixture</dd></div></dl><a href="{preview_url}" aria-label="Preview environment fixture">Preview environment fixture</a></aside></div><ol class="devportal-admission-sequence" aria-label="Admission sequence">{admission_steps}</ol><section class="operation-center-card" aria-label="Operation Center"><h5>Operation Center</h5><dl class="service-kv"><div><dt>operation id</dt><dd>op-20260701-0001 · {operation_id}</dd></div><div><dt>client idempotency key</dt><dd>{idempotency_key}</dd></div><div><dt>requested mutation</dt><dd>fixture-only provisioning preview</dd></div><div><dt>current state</dt><dd>{operation_state}</dd></div><div><dt>timestamps</dt><dd>2026-07-01T05:00:00Z</dd></div><div><dt>retry classification</dt><dd>safe-idempotent</dd></div><div><dt>rollback / compensation</dt><dd>delete preview environment fixture</dd></div><div><dt>audit-chain pointer</dt><dd>{evidence_receipt}</dd></div><div><dt>error / remediation metadata</dt><dd>{denial_reason}</dd></div></dl></section><div class="devportal-artifact-grid"><section aria-labelledby="devportal-artifacts-title" aria-label="Generated artifacts"><h5 id="devportal-artifacts-title">Generated artifacts</h5><ol>{artifacts}</ol></section><section aria-labelledby="devportal-policy-title" aria-label="Policy denial and audit evidence"><h5 id="devportal-policy-title">Policy denial and audit evidence</h5><p><strong>{denial_decision}</strong> · {denial_reason}</p><ol>{audit_events}</ol></section><section aria-labelledby="devportal-roles-title" aria-label="Developer Portal role coverage"><h5 id="devportal-roles-title">Role coverage</h5><div class="devportal-role-grid">{role_paths}</div></section></div><div class="devportal-facets" role="region" aria-label="Developer Portal resource facet evidence"><div><p class="screen-anchor">RESOURCE FACET EVIDENCE</p><h5>Missing lifecycle, identity, policy, quota, billing, audit, observability, rollback, or reconciliation facets fail closed</h5><span class="status-chip danger">facet_contract_fail_closed={fail_closed}</span></div><table class="finance-table"><thead><tr><th>Resource</th><th>Lifecycle</th><th>Identity</th><th>Policy</th><th>Quota</th><th>Billing</th><th>Audit</th><th>Observability</th><th>Rollback</th><th>Reconciliation</th></tr></thead><tbody>{facet_rows}</tbody></table><div class="devportal-authority-strip" aria-label="Developer portal authority and persona paths"><span>tenant admin path</span><span>operator path</span><span>developer path</span><span>security/compliance reviewer path</span><span>support engineer path</span><span>fixture-only non-claim</span><span>ADR-0536 D-4/D-5</span><span>specs/masterplan.json:144-257</span><span>specs/cloud-control-plane-canonical.json</span><span>specs/cloud-resource-catalog-target.json</span><span>specs/api-contract-ssot-canonical.json</span><span>ADR-0393</span></div></div></section>"#,
        service = escape(&portal.catalog_entity.display_name),
        template_id = escape(&template.template_id),
        artifact_count = artifact_count,
        resource_facets = resource_facets,
        template_name = escape(&template.display_name),
        version = escape(&template.version.version),
        owner = escape(&template.owner),
        entity_kind = escape(&portal.catalog_entity.kind),
        service_slug = escape(&portal.catalog_entity.service_slug),
        resources = resources,
        parameters = parameters,
        request_id = escape(&portal.provisioning_request.request_id),
        idempotency_key = escape(&portal.provisioning_operation.idempotency_key),
        operation_id = escape(&portal.provisioning_operation.operation_id),
        operation_state = escape(&contract_fixture_label(
            &portal.provisioning_operation.state
        )),
        preview_id = escape(&portal.preview_environment.environment_id),
        preview_state = escape(&portal.preview_environment.lifecycle_state),
        preview_url = escape(&portal.preview_environment.url),
        evidence_receipt = escape(&portal.provisioning_operation.evidence_receipt),
        currency = escape(&portal.cost_preview.currency),
        monthly_major = portal.cost_preview.monthly_minor_units / 100,
        admission_steps = admission_steps,
        artifacts = artifacts,
        denial_decision = escape(&portal.policy_denial_fixture.decision),
        denial_reason = escape(&portal.policy_denial_fixture.reason),
        audit_events = audit_events,
        role_paths = role_paths,
        fail_closed = portal.facet_contract_fail_closed,
        facet_rows = facet_rows,
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_support_advisory_case_console(fixture: &SupportAdvisoryFixture) -> String {
    let bundle_refs = fixture
        .diagnostic_bundle_evidence_refs
        .iter()
        .map(|reference| format!("<li>{}</li>", escape(reference)))
        .collect::<String>();
    let records = fixture
        .api_records
        .iter()
        .map(|record| {
            format!(
                r#"<tr data-support-record="{record_type}"><th scope="row">{record_type}</th><td>{record_id}</td><td>{tenant_id}</td><td>{actor}</td><td>{purpose}</td><td>{data_class}</td><td>{freshness}</td><td>{redaction}</td><td>audit_event_ref={audit}</td></tr>"#,
                record_type = escape(&record.record_type),
                record_id = escape(&record.record_id),
                tenant_id = escape(&record.tenant_id),
                actor = escape(&record.actor),
                purpose = escape(&record.purpose),
                data_class = escape(&record.data_class),
                freshness = escape(&record.freshness),
                redaction = escape(&record.redaction_policy_id),
                audit = escape(&record.audit_event_ref),
            )
        })
        .collect::<String>();
    let incident_refs = fixture
        .incident_refs
        .iter()
        .chain(fixture.status_refs.iter())
        .chain(fixture.trust_evidence_refs.iter())
        .chain(fixture.governance_posture_refs.iter())
        .chain(fixture.finops_refs.iter())
        .chain(fixture.onboarding_refs.iter())
        .chain(fixture.audit_chain_refs.iter())
        .map(|reference| format!("<span class=\"status-chip\">{}</span>", escape(reference)))
        .collect::<String>();
    let security = fixture
        .security_assertions
        .iter()
        .chain(fixture.authority_boundaries.iter())
        .map(|assertion| format!("<p>{}</p>", escape(assertion)))
        .collect::<String>();
    let actions = fixture
        .post_case_actions
        .iter()
        .map(|action| {
            format!(
                "<li><strong>post_case_action_item</strong><span>{}</span></li>",
                escape(action)
            )
        })
        .collect::<String>();

    format!(
        r#"<section id="support-advisory-console" class="panel support-advisory-console" data-fixture-kind="support-advisory-case" aria-labelledby="support-advisory-title"><div class="panel-header"><div><p class="eyebrow">Customer Support + Trusted Advisor · fixture only</p><h3 id="support-advisory-title">Support case diagnostic bundle and advisory</h3><p>Tenant admin opens Support, creates a case, selects severity, attaches a redacted diagnostic bundle preview, sees a trusted-advisor recommendation, links relevant status/incident/evidence, and tracks post-case action items. The support engineer sees support engineer redacted context only.</p></div><span class="status-chip warning">fixture-only non-claim</span></div><div class="catalog-kpi-strip" aria-label="Support entitlement and severity summary"><span><strong>{severity}</strong><small>{response_target}</small></span><span><strong>support_entitlement</strong><small>{entitlement}</small></span><span><strong>customer_communication</strong><small>{customer_state}</small></span><span><strong>support_access_approval</strong><small>{access_state}</small></span></div><div class="devportal-story-grid"><article aria-label="Support case story"><p class="screen-anchor">support_case</p><h5>{case_id}</h5><dl class="service-kv"><div><dt>Tenant</dt><dd>tenant_id={tenant_id}</dd></div><div><dt>Resource</dt><dd>{resource_ref}</dd></div><div><dt>Service</dt><dd>{service_ref}</dd></div><div><dt>Engineer view</dt><dd>{engineer_view}</dd></div></dl><button type="button" data-support-action="attach-bundle">Attach diagnostic bundle preview</button><button type="button" data-support-action="select-severity">Select severity</button></article><article aria-label="Diagnostic bundle allowed refs"><p class="screen-anchor">diagnostic_bundle</p><h5>{bundle_ref}</h5><div class="devportal-resource-pills" aria-label="Support linked refs">{incident_refs}</div><ul>{bundle_refs}</ul></article><article aria-label="Trusted advisor recommendation"><p class="screen-anchor">advisor_recommendation</p><h5>{advisor_key}</h5><dl class="service-kv"><div><dt>Severity</dt><dd>{advisor_severity}</dd></div><div><dt>Owner</dt><dd>{advisor_owner}</dd></div><div><dt>Freshness</dt><dd>{expires}</dd></div></dl><p>{recommended_action}</p><p>{non_goal_copy}</p></article></div><table class="finance-table" aria-label="Support API fixture records"><thead><tr><th>Record</th><th>ID</th><th>Tenant</th><th>Actor</th><th>Purpose</th><th>Data class</th><th>Freshness</th><th>Redaction</th><th>Audit</th></tr></thead><tbody>{records}</tbody></table><div class="resource-access-rules" aria-label="Support redaction and authority boundaries"><p>approval-gated breakglass</p>{security}</div><ol class="ops-timeline" aria-label="Customer communication and post-case action items"><li><strong>customer_communication</strong><span>Customer-visible update is staged; external send is disabled.</span></li><li><strong>support_access_approval</strong><span>{access_state}</span></li>{actions}</ol></section>"#,
        severity = escape(&fixture.severity),
        response_target = escape(&fixture.response_target_label),
        entitlement = escape(&fixture.entitlement_plan),
        customer_state = escape(&fixture.customer_state),
        access_state = escape(&fixture.support_access_state),
        case_id = escape(&fixture.case_id),
        tenant_id = escape(&fixture.tenant_id),
        resource_ref = escape(&fixture.resource_ref),
        service_ref = escape(&fixture.service_ref),
        engineer_view = escape(&fixture.support_engineer_view),
        bundle_ref = escape(&fixture.diagnostic_bundle_ref),
        incident_refs = incident_refs,
        bundle_refs = bundle_refs,
        advisor_key = escape(&fixture.advisor_key),
        advisor_severity = escape(&fixture.advisor_severity),
        advisor_owner = escape(&fixture.advisor_owner),
        expires = escape(&fixture.expires_at),
        recommended_action = escape(&fixture.recommended_action),
        non_goal_copy = escape(&fixture.non_goal_copy),
        records = records,
        security = security,
        actions = actions,
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_service_catalog(envelope: &TenantRenderEnvelope) -> String {
    let total_modules = envelope.modules.len();
    let attention_count = envelope
        .modules
        .iter()
        .filter(|module| catalog_state_for(&module.group, &module.name) == "attention")
        .count();
    let cloud_count = envelope
        .modules
        .iter()
        .filter(|module| catalog_group_slug(&module.group) == "cloud")
        .count();
    let trust_count = envelope
        .modules
        .iter()
        .filter(|module| matches!(catalog_group_slug(&module.group), "trust" | "control"))
        .count();
    let filters = CATALOG_FILTERS
        .iter()
        .map(|(slug, label)| {
            let class = if *slug == "all" { "fp active" } else { "fp" };
            format!(
                r#"<button type="button" class="{class}" data-catalog-filter="{slug}"><span class="fp-dot" aria-hidden="true"></span>{label}</button>"#,
                class = class,
                slug = escape(slug),
                label = escape(label)
            )
        })
        .collect::<String>();
    let modules = envelope
        .modules
        .iter()
        .map(static_catalog_module)
        .collect::<String>();
    let developer_portal = static_developer_portal_provisioning(&envelope.developer_portal);

    format!(
        r#"<section id="service-catalog" class="panel modules-panel catalog-workbench" aria-labelledby="modules-title"><div class="panel-header catalog-header"><div><p class="eyebrow">Service catalog</p><h3 id="modules-title">Permitted service graph</h3></div><span class="catalog-live-chip">local · visually interactive</span></div><div class="catalog-kpi-strip" aria-label="Service catalog summary"><div class="catalog-kpi accent"><span>Permitted modules</span><strong>{total_modules}</strong><small>from server envelope</small></div><div class="catalog-kpi"><span>Cloud dependencies</span><strong>{cloud_count}</strong><small>compute · network · cells</small></div><div class="catalog-kpi warn"><span>Need attention</span><strong>{attention_count}</strong><small>review before promote</small></div><div class="catalog-kpi"><span>Trust surfaces</span><strong>{trust_count}</strong><small>roles · audit · policy</small></div><div class="catalog-kpi"><span>Cross-service routes</span><strong>7</strong><small>workflow → mail/community/ops</small></div></div><div class="catalog-toolbar" aria-label="Catalog search and filters"><label class="catalog-search"><span aria-hidden="true">⌕</span><input data-catalog-search="true" type="search" aria-label="Search service catalog" placeholder="Search modules, owners, dependencies..." /></label><div class="filter-pills catalog-filters" role="toolbar" aria-label="Catalog filters">{filters}<button type="button" class="fp" data-catalog-filter="attention"><span class="fp-dot danger" aria-hidden="true"></span>Attention</button></div><span class="catalog-status" data-catalog-status="true"><strong data-catalog-visible-count="true">{total_modules}</strong> visible · all filter · local catalog only</span></div><div class="catalog-workspace"><div class="catalog-table-shell" role="region" aria-label="Permitted modules table"><div class="catalog-table-head" aria-hidden="true"><span>Health</span><span>Module</span><span>Category</span><span>Owner</span><span>Downstream graph</span><span>Actions</span></div><div class="catalog-module-list">{modules}</div></div><aside id="service-graph" class="catalog-service-graph" aria-label="Service graph and module lineage"><div class="graph-head"><p class="screen-anchor">SERVICE GRAPH</p><strong>One cohesive Oyatie nervous system</strong><span>Workflow events fan out to built-in surfaces and return audit evidence.</span></div><ol class="lineage-list"><li class="root"><span>Workflow</span><strong>Tenant change approval</strong><em>root event</em></li><li><span>Messenger</span><strong>Ops room draft</strong><em>delivered</em></li><li><span>Mail</span><strong>Formal approval brief</strong><em>pending</em></li><li><span>Community</span><strong>Governance council note</strong><em>review</em></li><li><span>Cloud Ops</span><strong>Runbook + FinOps</strong><em>guarded</em></li><li><span>Audit</span><strong>Receipt spine</strong><em>sealed</em></li></ol><div class="catalog-graph-actions" aria-label="Service graph actions"><button type="button" data-catalog-graph-action="workflow">Open workflow</button><button type="button" data-catalog-graph-action="mail">Mail route</button><button type="button" data-catalog-graph-action="community">Community route</button><button type="button" data-catalog-graph-action="evidence">Evidence spine</button></div></aside></div>{developer_portal}{service_catalog_anchor_board}<p class="catalog-footer-hint omitted-note"><span aria-hidden="true">✦</span>{omitted}</p></section>"#,
        total_modules = total_modules,
        cloud_count = cloud_count,
        attention_count = attention_count,
        trust_count = trust_count,
        filters = filters,
        modules = modules,
        developer_portal = developer_portal,
        service_catalog_anchor_board = static_service_catalog_anchor_board(),
        omitted = escape(&envelope.omitted_capability_note),
    )
}

#[cfg(any(feature = "ssr", test))]
fn contract_fixture_label(value: &str) -> String {
    value
        .replace("accepted_fixture", "accepted · fixture row")
        .replace("denied_fixture", "denied · fixture row")
        .replace("non-retryable-policy-gate", "policy")
        .replace("human-remediation", "operator_required")
}

#[cfg(any(feature = "ssr", test))]
fn title_case_role(role: &str) -> String {
    let mut chars = role.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(any(feature = "ssr", test))]
fn static_catalog_module(module: &ModuleCard) -> String {
    let group_slug = catalog_group_slug(&module.group);
    let state = catalog_state_for(&module.group, &module.name);
    let state_label = catalog_state_label(state);
    let owner = catalog_owner_for(&module.group, &module.name);
    let criticality = catalog_criticality_for(&module.group, &module.name);
    let route = catalog_route_for(&module.name);
    let dependency = catalog_dependency_for(&module.group, &module.name);
    let sidepeek_desc = format!(
        "{} · {} · {}",
        module.description, owner, "local visual-only catalog module"
    );
    let sidepeek_id = format!("CAT-{}", catalog_code_for(&module.name));

    format!(
        r#"<article class="catalog-module-row module-card" data-catalog-module="true" data-catalog-group="{group_slug}" data-catalog-state="{state}"><span class="health-dot health-{state}" aria-label="{state_label}"></span><div class="catalog-module-main"><button type="button" class="catalog-module-title" data-sidepeek-trigger="catalog-module" data-sidepeek-title="{name}" data-sidepeek-id="{sidepeek_id}" data-sidepeek-desc="{sidepeek_desc}" data-sidepeek-owner="{owner}" data-sidepeek-risk="{state_label}" data-sidepeek-sla="4.0h review window">{name}</button><p>{description}</p><code>{code}</code></div><span class="cat-tag">{group}</span><span class="owner-cell"><span class="avatar-xs" aria-hidden="true">{avatar}</span><span>{owner}</span></span><span class="catalog-dependency-chain"><em>Workflow</em><i aria-hidden="true">→</i><em>{dependency}</em><i aria-hidden="true">→</i><em>Audit</em><span class="crit crit-{criticality}">{criticality}</span></span><span class="catalog-row-actions"><button type="button" data-catalog-action="open" data-catalog-target="{route}">{action}</button><button type="button" data-catalog-action="pin">Pin</button><button type="button" data-catalog-action="request">Request access</button></span></article>"#,
        group_slug = escape(group_slug),
        state = escape(state),
        state_label = escape(state_label),
        name = escape(&module.name),
        sidepeek_id = escape(&sidepeek_id),
        sidepeek_desc = escape(&sidepeek_desc),
        owner = escape(owner),
        description = escape(&module.description),
        code = escape(&catalog_code_for(&module.name)),
        group = escape(&module.group),
        avatar = escape(catalog_owner_avatar(owner)),
        dependency = escape(dependency),
        criticality = escape(criticality),
        route = escape(route),
        action = escape(&module.action_label),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_task(task: &WorkItem) -> String {
    format!(
        "<li><span class=\"priority\">{}</span><strong>{}</strong><p>{}</p></li>",
        escape(&task.priority),
        escape(&task.title),
        escape(&task.detail)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_approval(approval: &ApprovalItem) -> String {
    format!(
        "<li><strong>{}</strong><p>{}</p><span>{}</span></li>",
        escape(&approval.title),
        escape(&approval.requester),
        escape(&approval.risk_note)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_schedule(item: &ScheduleItem) -> String {
    format!(
        "<li><time>{}</time><div><strong>{}</strong><p>{}</p></div></li>",
        escape(&item.time),
        escape(&item.title),
        escape(&item.detail)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_message(message: &MessageItem) -> String {
    format!(
        "<button type=\"button\" class=\"hub-item\"><span>{}</span><strong>{}</strong><p>{}</p></button>",
        escape(&message.channel),
        escape(&message.from),
        escape(&message.preview)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_community(item: &CommunityItem) -> String {
    format!(
        "<button type=\"button\" class=\"hub-item\"><span>{}</span><strong>{}</strong><p>{}</p></button>",
        escape(&item.space),
        escape(&item.topic),
        escape(&item.activity)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_selected_node(node: &WorkflowNode) -> String {
    format!(
        "<aside class=\"node-inspector\"><p class=\"eyebrow\">Selected node</p><h4>{}</h4><p><strong>{}</strong> · {}</p></aside>",
        escape(&node.label),
        escape(&node.kind),
        escape(&node.explanation)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_board(nodes: &[WorkflowNode]) -> String {
    let edges = board_edges(nodes)
        .into_iter()
        .map(|(from, to, path)| {
            format!(
                "<path class=\"workflow-edge workflow-board-edge\" data-edge-from=\"{}\" data-edge-to=\"{}\" d=\"{}\" marker-end=\"url(#workflow-board-arrow)\"></path>",
                escape(&from),
                escape(&to),
                escape(&path)
            )
        })
        .collect::<String>();
    let workflow_canvas_metrics = static_workflow_canvas_metrics();
    let cards = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let class = if index == 0 {
                "workflow-card active selectable"
            } else {
                "workflow-card selectable"
            };
            format!(
                r#"<button type="button" class="{class}" style="left: {x}px; top: {y}px" data-workflow-card="true" data-node-id="{id}" data-node-label="{label}" data-node-kind="{kind}" data-node-desc="{desc}"><span class="board-port in" aria-hidden="true"></span><span class="board-port out" aria-hidden="true"></span><span class="workflow-card-type">{kind}</span><strong>{label}</strong><small>{desc}</small></button>"#,
                class = class,
                x = workflow_board_x(index),
                y = workflow_board_y(index, node),
                id = escape(&node.id),
                label = escape(&node.label),
                kind = escape(&node.kind),
                desc = escape(&node.explanation)
            )
        })
        .collect::<String>();

    format!(
        r#"<div class="workflow-board selectable" data-workflow-board="true"><svg class="workflow-board-edges" viewBox="0 0 860 430" aria-hidden="true" focusable="false"><defs><marker id="workflow-board-arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth"><path d="M0,0 L0,6 L9,3 z" class="workflow-arrow"></path></marker></defs>{edges}</svg>{workflow_canvas_metrics}{cards}<div class="workflow-ai-suggestion" aria-label="AI workflow suggestion"><p>AI · WORKFLOW SUGGESTION</p><strong>CFO 승인이 SLA를 초과할 때 자동 위임 조건을 추가</strong><span>conf 0.86 · model oyatie-flow-sense-1.4 · why →</span><div><button type="button" data-workflow-suggestion="dismiss">Dismiss</button><button type="button" data-workflow-suggestion="preview">Preview</button><button type="button" data-workflow-suggestion="apply">Apply</button></div></div><div class="canvas-drop-hint" aria-hidden="true">Drag blocks here · connect ports visually · local only</div></div>"#
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_svg(nodes: &[WorkflowNode]) -> String {
    let svg_nodes = nodes
        .iter()
        .map(|node| {
            format!(
                "<g class=\"workflow-node-group selectable\"><rect x=\"{x}\" y=\"{y}\" width=\"130\" height=\"56\" rx=\"10\" class=\"workflow-node\"></rect><circle cx=\"{in_x}\" cy=\"{port_y}\" r=\"4\" class=\"port in\"></circle><circle cx=\"{out_x}\" cy=\"{port_y}\" r=\"4\" class=\"port out\"></circle><text x=\"{label_x}\" y=\"{label_y}\">{label}</text><text x=\"{kind_x}\" y=\"{kind_y}\" class=\"node-kind\">{kind}</text></g>",
                x = node.x,
                y = node.y,
                in_x = node.x + 8,
                out_x = node.x + 122,
                port_y = node.y + 28,
                label_x = node.x + 16,
                label_y = node.y + 24,
                kind_x = node.x + 16,
                kind_y = node.y + 43,
                label = escape(&node.label),
                kind = escape(&node.kind)
            )
        })
        .collect::<String>();

    format!(
        "<svg viewBox=\"0 0 820 310\" aria-hidden=\"true\"><line x1=\"140\" y1=\"120\" x2=\"690\" y2=\"120\" class=\"workflow-edge\"></line>{svg_nodes}</svg>"
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_ontology_command_console(facts: &[OntologyFact]) -> String {
    let fact_cards = facts
        .iter()
        .enumerate()
        .map(|(index, fact)| static_ontology_fact_card(index, fact))
        .collect::<String>();

    format!(
        r#"<div class="ontology-command-console" data-ontology-console="true"><div class="ontology-console-head"><div><p class="screen-anchor">ONTOLOGY · FD-001 TENANT WORKLOAD MAP</p><h4>What exists, who can see it, and where it runs</h4><span>Typed entities connect FD-001 tenant workload delivery to Oyatie Cloud cells, policy envelopes, workflow outputs, and evidence receipts.</span></div><div class="ontology-console-actions"><span class="status-chip success" data-ontology-status="true">{fact_count} facts · 7 workload nodes · local graph</span><button type="button" data-ontology-action="lineage">Trace lineage</button><button type="button" data-ontology-action="policy">Policy view</button><button type="button" data-ontology-action="evidence">Evidence</button></div></div><div class="ontology-topology-grid" aria-label="FD-001 tenant workload ontology graph"><button type="button" class="ontology-node root selected" data-ontology-node="Tenant" data-node-route="workload" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Tenant" data-sidepeek-id="ONT-TENANT" data-sidepeek-desc="Tenant owns the permitted FD-001 module set and runtime envelope." data-sidepeek-owner="Tenant admin" data-sidepeek-risk="Visible" data-sidepeek-sla="Local graph only"><span>TENANT</span><strong>Tenant Admin</strong><em>owns envelope</em></button><button type="button" class="ontology-node workload" data-ontology-node="FD-001 Workloads" data-node-route="workflow" data-sidepeek-trigger="ontology-node" data-sidepeek-title="FD-001 workload set" data-sidepeek-id="ONT-FD001" data-sidepeek-desc="Core FD-001 microservices are represented as tenant workloads for dogfood validation." data-sidepeek-owner="Product delivery" data-sidepeek-risk="P0" data-sidepeek-sla="Dogfood proving loop"><span>FD-001</span><strong>Microservice workloads</strong><em>product goal</em></button><button type="button" class="ontology-node cloud" data-ontology-node="Oyatie Cloud" data-node-route="cloud" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Oyatie Cloud substrate" data-sidepeek-id="ONT-CLOUD" data-sidepeek-desc="Hyperscaler-grade substrate hosts dogfood tenant workloads and exposes cell posture." data-sidepeek-owner="Cloud substrate" data-sidepeek-risk="Substrate proof" data-sidepeek-sla="99.95 target staged"><span>CLOUD</span><strong>Cell substrate</strong><em>hosts tenants</em></button><button type="button" class="ontology-node workflow" data-ontology-node="Workflow" data-node-route="workflow" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Workflow runtime" data-sidepeek-id="ONT-WORKFLOW" data-sidepeek-desc="Workflow coordinates payroll close, approval, comms outputs, and receipts." data-sidepeek-owner="Workflow Studio" data-sidepeek-risk="Governed" data-sidepeek-sla="4.0h gate"><span>FLOW</span><strong>Workflow</strong><em>orchestrates</em></button><button type="button" class="ontology-node comms" data-ontology-node="Built-in Comms" data-node-route="mail" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Built-in communications" data-sidepeek-id="ONT-COMMS" data-sidepeek-desc="Messenger, Mail, and Community receive workflow outputs without external send." data-sidepeek-owner="Work Hub" data-sidepeek-risk="Local only" data-sidepeek-sla="No backend send"><span>COMMS</span><strong>Messenger · Mail · Community</strong><em>outputs</em></button><button type="button" class="ontology-node evidence" data-ontology-node="Evidence" data-node-route="evidence" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Evidence spine" data-sidepeek-id="ONT-EVIDENCE" data-sidepeek-desc="Receipts bind workload state, approvals, messages, and deployment gates." data-sidepeek-owner="Audit spine" data-sidepeek-risk="Immutable staged" data-sidepeek-sla="Sealed draft"><span>AUDIT</span><strong>Evidence spine</strong><em>proves</em></button><button type="button" class="ontology-node policy" data-ontology-node="Policy" data-node-route="identity" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Policy envelope" data-sidepeek-id="ONT-POLICY" data-sidepeek-desc="Role, data-class, residency, and autonomy ceilings decide visibility and action eligibility." data-sidepeek-owner="Governance" data-sidepeek-risk="Guardrail" data-sidepeek-sla="Human review"><span>POLICY</span><strong>Access envelope</strong><em>permits</em></button></div><div class="ontology-fact-matrix" aria-label="Current permitted ontology facts">{fact_cards}</div><div class="ontology-proof-rail" aria-label="Substrate proof contract"><article><p class="screen-anchor">SUBSTRATE PROOF</p><strong>FD-001 runs as tenant workloads before service claim</strong><span>Cloud cells, policy envelopes, and evidence receipts prove the substrate can host real production tenants.</span></article><article><p class="screen-anchor">VISIBILITY</p><strong>Role + data-class gates</strong><span>Tenant admin can inspect posture; hidden modules remain server-derived, not client hidden.</span></article><article><p class="screen-anchor">GRAPH STATUS</p><strong data-ontology-detail="true">Tenant selected · workload lineage visible</strong><span>Click nodes or facts to stage local graph state.</span></article></div></div>"#,
        fact_count = facts.len(),
        fact_cards = fact_cards,
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_ontology_fact_card(index: usize, fact: &OntologyFact) -> String {
    format!(
        r#"<article class="ontology-fact-card" data-ontology-fact="true"><span class="status-chip">FACT-{index:02}</span><strong>{entity}</strong><em>{relation}</em><p>{reason}</p><div><button type="button" data-ontology-action="inspect-fact">Inspect</button><button type="button" data-ontology-action="route-workflow">Workflow</button></div></article>"#,
        index = index + 1,
        entity = escape(&fact.entity),
        relation = escape(&fact.relation),
        reason = escape(&fact.access_reason),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_intelligence_command_console(suggestions: &[IntelligenceSuggestion]) -> String {
    let cards = suggestions
        .iter()
        .enumerate()
        .map(|(index, suggestion)| static_intelligence_card(index, suggestion))
        .collect::<String>();
    format!(
        r#"<div class="intelligence-command-console" data-intelligence-console="true"><div class="intelligence-console-head"><div><p class="screen-anchor">GOVERNED AI · DOGFOOD ADVISOR</p><h4>Recommendations that can explain, route, and prove themselves</h4><span>AI suggestions stay read-only until a human routes them to Workflow, Mail, Community, or Evidence.</span></div><div class="intelligence-console-actions"><span class="status-chip warning" data-intelligence-status="true">{count} suggestions · human gated</span><button type="button" data-intelligence-action="evaluate">Run eval</button><button type="button" data-intelligence-action="explain">Explain</button><button type="button" data-intelligence-action="route-evidence">Evidence</button></div></div><div class="intelligence-score-strip" aria-label="Governed AI evaluation summary"><span><strong>0.86</strong><small>decision confidence</small></span><span><strong>14</strong><small>policy checks</small></span><span><strong>0</strong><small>auto-executions</small></span><span><strong>3</strong><small>FD-001 routes</small></span></div><div class="intelligence-layout"><div class="intelligence-suggestion-stack" role="list" aria-label="Governed AI suggestions">{cards}</div><aside class="intelligence-eval-panel" aria-label="AI guardrail evaluation harness"><div><p class="screen-anchor">EVAL HARNESS</p><strong>Before any tenant action</strong><span>Policy, data-class, autonomy ceiling, residency, and evidence checks must pass.</span></div><ol><li><span class="status-chip success">pass</span><strong>No backend execution</strong><em>T1 advisory only</em></li><li><span class="status-chip success">pass</span><strong>Human approval required</strong><em>CFO / reviewer gate</em></li><li><span class="status-chip warning">review</span><strong>Tenant workload impact</strong><em>FD-001 cloud dogfood</em></li><li><span class="status-chip success">sealed</span><strong>Evidence receipt ready</strong><em>REC-AI-GUARD-009</em></li></ol><div class="intelligence-route-grid" aria-label="Recommendation routes"><button type="button" data-intelligence-action="route-workflow">Workflow</button><button type="button" data-intelligence-action="route-mail">Mail</button><button type="button" data-intelligence-action="route-community">Community</button><button type="button" data-intelligence-action="route-evidence">Evidence</button></div></aside></div></div>"#,
        count = suggestions.len(),
        cards = cards,
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_intelligence_card(index: usize, suggestion: &IntelligenceSuggestion) -> String {
    let route = match index {
        0 => "workflow",
        1 => "mail",
        _ => "community",
    };
    let receipt = match index {
        0 => "AI-WF-217",
        1 => "AI-TAX-118",
        _ => "AI-HR-053",
    };
    format!(
        r#"<article class="intelligence-suggestion-card" data-intelligence-card="true" data-intelligence-route="{route}" role="listitem"><div><span class="status-chip ai">{receipt}</span><strong>{title}</strong></div><p>{body}</p><small>{guardrail}</small><div class="intelligence-card-actions"><button type="button" data-intelligence-action="preview" data-intelligence-route="{route}">Preview</button><button type="button" data-intelligence-action="route" data-intelligence-route="{route}">Route</button><button type="button" data-intelligence-action="dismiss">Dismiss</button></div></article>"#,
        route = route,
        receipt = receipt,
        title = escape(&suggestion.title),
        body = escape(&suggestion.body),
        guardrail = escape(&suggestion.guardrail),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_ontology(fact: &OntologyFact) -> String {
    format!(
        "<li><strong>{}</strong><span>{}</span><p>{}</p></li>",
        escape(&fact.entity),
        escape(&fact.relation),
        escape(&fact.access_reason)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_suggestion(suggestion: &IntelligenceSuggestion) -> String {
    format!(
        "<li><strong>{}</strong><p>{}</p><span>{}</span></li>",
        escape(&suggestion.title),
        escape(&suggestion.body),
        escape(&suggestion.guardrail)
    )
}

#[cfg(any(feature = "ssr", test))]
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::{static_dashboard_content, static_dashboard_html};
    use crate::render_envelope::{OperatorContext, server_derived_envelope};

    #[test]
    fn resource_explorer_fixture_exposes_canonical_hierarchy_and_one_resource_type() {
        let html = static_dashboard_html();

        assert!(html.contains("Resource Explorer"));
        assert!(html.contains(
            "Organization → Account → Project → Region → Cell → Resource Group → Resource"
        ));
        assert!(html.contains("aria-label=\"Search ORN\""));
        assert!(html.contains(
            "orn:oya:us-east-2:acct-northwind-prod:cloud-compute:k8s-cluster/cluster-nw-app-01"
        ));
        assert!(html.contains("data-resource-type=\"K8sCluster\""));
        assert!(!html.contains("data-resource-type=\"Vpc\""));
    }

    #[test]
    fn resource_detail_exposes_nine_facets_and_data_access_rules() {
        let html = static_dashboard_html();

        for facet in [
            "data-resource-facet=\"lifecycle\"",
            "data-resource-facet=\"identity\"",
            "data-resource-facet=\"policy\"",
            "data-resource-facet=\"quota\"",
            "data-resource-facet=\"billing\"",
            "data-resource-facet=\"audit\"",
            "data-resource-facet=\"observability\"",
            "data-resource-facet=\"rollback\"",
            "data-resource-facet=\"reconciliation\"",
        ] {
            assert!(html.contains(facet), "missing facet marker {facet}");
        }

        assert!(html.contains("Cedar default-deny expectation"));
        assert!(html.contains("Cross-tenant rows are redacted"));
        assert!(html.contains("Support access is constrained"));
        assert!(html.contains("No secret material is exposed"));
    }

    #[test]
    fn operation_center_fixture_exposes_aip151_ledger_fields() {
        let html = static_dashboard_html();
        let operation_center_start = html
            .find("id=\"operation-center\"")
            .expect("operation center fixture should render");
        let operation_center_end = html[operation_center_start..]
            .find("aria-label=\"Persona user-story paths\"")
            .map(|relative| operation_center_start + relative)
            .unwrap_or(html.len());
        let operation_center_html = &html[operation_center_start..operation_center_end];

        for marker in [
            "Operation Center",
            "op-20260701-0001",
            "client idempotency key",
            "requested mutation",
            "current state",
            "current state accepted · fixture row",
            "current state failed · policy denied fixture row",
            "timestamps",
            "retry classification",
            "retry classification policy",
            "retry classification operator_required",
            "rollback / compensation",
            "audit-chain pointer",
            "error / remediation metadata",
        ] {
            assert!(
                html.contains(marker),
                "missing operation ledger marker {marker}"
            );
        }

        for forbidden in [
            "accepted_fixture",
            "denied_fixture",
            "retry classification non-retryable-policy-gate",
            "retry classification human-remediation",
        ] {
            assert!(
                !operation_center_html.contains(forbidden),
                "operation fixture leaked non-contract marker {forbidden}"
            );
        }
    }

    #[test]
    fn audit_evidence_timeline_uses_durable_cloud_observability_source() {
        let html = static_dashboard_html();
        let timeline_start = html
            .find("id=\"audit-evidence-timeline\"")
            .expect("audit evidence timeline should render");
        let timeline_end = html[timeline_start..]
            .find("</section>")
            .map(|relative| timeline_start + relative)
            .unwrap_or(html.len());
        let timeline_html = &html[timeline_start..timeline_end];

        for marker in [
            "data-audit-evidence-source=\"durable-cloud-observability-audit-api\"",
            "data-audit-evidence-endpoint=\"QUERY /v1/cloud/observability/audit-records:read\"",
            "data-authz-surface=\"cloud.observability.audit.read\"",
            "data-audit-operation-class=\"cloud_iam_policy\"",
            "data-authority-ref=\"contracts/openapi/cloud/cloud-observability-audit-v1.yaml#readCloudObservabilityAudit\"",
            "data-source-anchors=\"G007 goals.json + AUTHZ-006 + specs/root-hub-pointers.json#entry_points.agent_operating_contract\"",
            "data-authority-boundary=\"analysis-only-non-mutating\"",
            "data-chain-complete=\"true\"",
            "data-high-watermark-sequence=\"42\"",
            "Authority: contracts/openapi/cloud/cloud-observability-audit-v1.yaml · G007/AUTHZ-006 slice · analysis-only-non-mutating.",
            "data-severity=\"ok\"",
            "oya.audit.cloud_iam_policy · iam_policy_changed",
            "signed:sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "oyaaudit://signed-export/northwind/cloud-iam-policy?sig=sig_cloud_iam_policy_001",
            "idem/cloud-iam-policy/001",
        ] {
            assert!(
                timeline_html.contains(marker),
                "missing durable audit evidence timeline marker {marker}"
            );
        }

        for forbidden in ["fixture-only", "local visual evidence", "mock", "ssh into"] {
            assert!(
                !timeline_html.contains(forbidden),
                "audit evidence timeline must not downgrade durable source to {forbidden}"
            );
        }
    }

    #[test]
    fn developer_portal_story_has_accessible_landmarks_for_fixture_paths() {
        let html = static_dashboard_html();

        for marker in [
            "aria-labelledby=\"developer-portal-provisioning-title\"",
            "aria-label=\"Template parameters\"",
            "aria-label=\"Generated artifacts\"",
            "aria-label=\"Policy denial and audit evidence\"",
            "aria-label=\"Developer Portal role coverage\"",
            "aria-label=\"Developer Portal resource facet evidence\"",
        ] {
            assert!(
                html.contains(marker),
                "missing accessibility marker {marker}"
            );
        }
    }

    #[test]
    fn control_plane_slice_names_persona_paths_non_claim_and_authorities() {
        let html = static_dashboard_html();

        for marker in [
            "tenant admin path",
            "operator path",
            "developer path",
            "security/compliance reviewer path",
            "support engineer path",
            "fixture-only non-claim",
            "ADR-0536 D-4/D-5",
            "specs/masterplan.json:144-257",
            "specs/cloud-control-plane-canonical.json",
            "specs/cloud-resource-catalog-target.json",
            "specs/api-contract-ssot-canonical.json",
            "ADR-0393",
        ] {
            assert!(html.contains(marker), "missing contract marker {marker}");
        }
    }

    #[test]
    fn control_plane_slice_uses_dedicated_resource_explorer_and_operation_center_surface() {
        let html = static_dashboard_html();

        for marker in [
            "id=\"resource-explorer\"",
            "aria-labelledby=\"resource-explorer-title\"",
            "data-fixture-kind=\"resource-explorer-resource\"",
            "id=\"operation-center\"",
            "aria-labelledby=\"operation-center-title\"",
            "data-fixture-kind=\"operation-timeline-entry\"",
            "X-Oyatie-API-Version: 2026-07-01 (mock)",
            "GET /v1/cloud-control-plane/resources",
        ] {
            assert!(
                html.contains(marker),
                "missing dedicated surface marker {marker}"
            );
        }
    }

    #[test]
    fn trust_center_slice_is_wired_into_static_and_interactive_shells() {
        let static_html = static_dashboard_html();
        assert!(static_html.contains("href=\"#trust-center-portal\""));
        assert!(static_html.contains("id=\"trust-center-portal\""));

        let source = include_str!("app.rs");
        let rail_start = source
            .find("fn ShellRail(")
            .expect("ShellRail source should be present");
        let dashboard_start = source
            .find("fn dashboard_view(")
            .expect("dashboard view source should be present");
        let rail_source = &source[rail_start..dashboard_start];
        assert!(rail_source.contains("href=\"#trust-center-portal\""));

        let dashboard_source = &source[dashboard_start..];
        let control_plane = dashboard_source
            .find("{control_plane_resource_explorer_operation_center()}")
            .expect("control-plane fixture call should be present");
        let trust_center = dashboard_source
            .find(concat!("{", "trust_center_portal", "()}"))
            .expect("Trust Center portal call should be present in interactive dashboard view");
        let support_advisory = dashboard_source
            .find("{support_advisory_view}")
            .expect("support advisory call should be present");
        assert!(control_plane < trust_center && trust_center < support_advisory);
    }

    #[test]
    fn trust_center_slice_exposes_admin_reviewer_and_evidence_room_markers() {
        let html = static_dashboard_html();

        for marker in [
            "href=\"#trust-center-portal\"",
            "id=\"trust-center-portal\"",
            "data-trust-center-fixture=\"contract-first-slice\"",
            "Trust Center overview",
            "data-trust-role=\"tenant-admin\"",
            "data-trust-role=\"security-compliance-reviewer\"",
            "data-trust-role=\"oyatie-operator\"",
            "aria-label=\"Filter trust evidence by class, source, freshness, or claim tier\"",
            "data-trust-filter=\"all\" aria-pressed=\"true\"",
            "Public status link / embed",
            "aria-label=\"Trust Center evidence table\"",
            "data-trust-evidence-state=\"current\"",
            "data-trust-evidence-state=\"stale\"",
            "data-trust-evidence-state=\"blocked_missing_evidence\"",
            "data-reviewer-grant-flow=\"create-revoke-expiring\"",
            "aria-label=\"Reviewer grant expiry\" type=\"datetime-local\"",
            "Create expiring reviewer grant",
            "Revoke reviewer access",
            "SBOM/VEX posture",
            "trust_center.evidence_index_viewed",
            "trust_center.access_grant_created",
        ] {
            assert!(
                html.contains(marker),
                "missing Trust Center marker {marker}"
            );
        }
    }

    #[test]
    fn trust_center_slice_marks_non_claim_boundaries_and_redaction_rules() {
        let html = static_dashboard_html();

        for marker in [
            "blocked_missing_evidence · not green",
            "target_non_claim · no certification",
            "claim tier: target_non_claim",
            "raw scanner output redacted · operator-only detail withheld",
            "Cross-tenant route denied; payload tenant assertions are display-only and never authority.",
            "customer-safe VEX summary · exploit details withheld",
            "Reviewer cannot administer tenant settings",
        ] {
            assert!(
                html.contains(marker),
                "missing Trust Center boundary marker {marker}"
            );
        }

        for forbidden in [
            "SECRET=",
            "BEGIN PRIVATE KEY",
            "raw scanner token",
            "exploit_payload=",
            "credit_card",
            "ssn",
        ] {
            assert!(
                !html
                    .to_ascii_lowercase()
                    .contains(&forbidden.to_ascii_lowercase()),
                "Trust Center fixture leaked forbidden marker {forbidden}"
            );
        }
    }

    #[test]
    fn support_advisory_story_exposes_case_bundle_and_advisor_fixture() {
        let html = static_dashboard_html();

        for marker in [
            "id=\"support-advisory-console\"",
            "aria-labelledby=\"support-advisory-title\"",
            "support_case",
            "support_entitlement",
            "diagnostic_bundle",
            "advisor_recommendation",
            "customer_communication",
            "support_access_approval",
            "post_case_action_item",
            "budget_anomaly_followup",
            "sreops_incident_ref=inc-sreops-20260701-17",
            "status_ref=status-component-orders-api-yellow",
            "trust_evidence_ref=trustcenter-control-soc2-cc7-2-redacted",
            "governance_posture_ref=govposture-policy-drift-asset-cluster-nw-app-01",
            "finops_ref=budget-anomaly-northwind-compute-20260701",
            "admin_onboarding_ref=domain-verified-northwind.example",
            "Attach diagnostic bundle preview",
            "approval-gated breakglass",
            "support engineer redacted context",
            "SEV-2 · 4h target label",
        ] {
            assert!(
                html.contains(marker),
                "missing support/advisory marker {marker}"
            );
        }
    }

    #[test]
    fn support_advisory_surface_is_not_rendered_for_unpermitted_contexts() {
        let admin_html =
            static_dashboard_content(&server_derived_envelope(OperatorContext::TenantAdmin));
        assert!(admin_html.contains("id=\"support-advisory-console\""));

        for context in [
            OperatorContext::CorporateOffice,
            OperatorContext::HealthcareClinician,
        ] {
            let html = static_dashboard_content(&server_derived_envelope(context));
            assert!(
                !html.contains("id=\"support-advisory-console\""),
                "{context:?} must not render the tenant-admin support/advisory surface"
            );
            assert!(
                !html.contains("tenant_id=tn_northwind_prod"),
                "{context:?} must not receive the Northwind support/advisory fixture"
            );
        }
    }

    #[test]
    fn support_advisory_fixture_denies_cross_tenant_and_raw_sensitive_data() {
        let html = static_dashboard_html();

        for marker in [
            "tenant_id=tn_northwind_prod",
            "other tenant denied",
            "raw_log_absent=true",
            "raw_pii_absent=true",
            "raw_phi_absent=true",
            "financial_detail_absent=true",
            "exploit_payload_absent=true",
            "cross-tenant list/open/count/attach/export/search/support-mode denied",
            "no status publish authority",
            "no Trust Center publishability mutation",
            "no SREOPS incident resolution authority",
            "audit_event_ref=aud-support-case-created-001",
        ] {
            assert!(
                html.contains(marker),
                "missing support security marker {marker}"
            );
        }

        for forbidden in [
            "SECRET=",
            "BEGIN PRIVATE KEY",
            "ssn",
            "credit_card",
            "patient_dob",
        ] {
            assert!(
                !html
                    .to_ascii_lowercase()
                    .contains(&forbidden.to_ascii_lowercase()),
                "support fixture leaked forbidden marker {forbidden}"
            );
        }
    }
}

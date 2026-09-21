use super::builders::{activity_step, s};
use super::types::{OperatorContext, ProductActivitySpine};

pub(super) fn product_activity_spine(context: OperatorContext) -> ProductActivitySpine {
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

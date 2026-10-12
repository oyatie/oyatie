use super::*;

pub(super) fn resource_inventory_rows() -> [ResourceRow; 6] {
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

pub(super) fn audit_receipts() -> [AuditReceipt; 5] {
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

pub(super) type ReceiptSource = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

pub(super) fn receipt_sources() -> [ReceiptSource; 4] {
    [
        (
            "workflow",
            "Workflow output bundle",
            "REC-FD001-WF-018",
            "Workflow → Messenger/Mail/Community → Evidence",
            "Workflow Studio",
            "review",
        ),
        (
            "comms",
            "Work Hub handoff draft",
            "REC-COMMS-HANDOFF-006",
            "Messenger/Mail/Community draft handoff",
            "Work Hub",
            "draft",
        ),
        (
            "cloud",
            "Cloud tenant workload posture",
            "REC-FD001-CLOUD-009",
            "Oyatie Cloud workload plane → gates",
            "Cloud substrate",
            "sealed",
        ),
        (
            "gates",
            "Deployment admission packet",
            "REC-DEPLOY-GATE-014",
            "Jenkins → ArgoCD → Cosign → Audit",
            "Release governance",
            "gate",
        ),
    ]
}

pub(super) fn receipt_stitching_console() -> impl IntoView {
    view! {
        <section class="receipt-stitching-console" data-receipt-stitching-console="true" aria-label="Illustrative receipt source inspection">
            <div class="receipt-stitching-head"><div><p class="screen-anchor">"SAMPLE RECEIPT SOURCES"</p>
                <h5>"Four illustrative source records"</h5><span data-receipt-stitching-status="true" role="status">"Select a sample source to inspect its exact fields; no packet is stitched."</span>
            </div>{ae_action("stitch", "seal", "Seal visible packet")}</div>
            <div class="receipt-stitching-grid" aria-label="Illustrative receipt sources">
                {receipt_sources().into_iter().map(|(key, title, receipt, route, owner, state)| view! {
                    <button type="button" data-receipt-source=key data-receipt-title=title data-receipt-id=receipt data-receipt-route=route data-receipt-owner=owner data-receipt-state=state
                        data-ae-select=format!("source:{key}") data-ae-record=ae_record(&format!("source:{key}")) data-ae-focus=format!("source:{key}") aria-pressed="false">
                        <span>"Sample "{key}</span><strong data-ae-value="Title">{title}</strong><em data-ae-value="Receipt">{receipt}</em>
                    </button>
                }).collect_view()}
            </div>
            <div class="receipt-stitching-actions" aria-label="Existing sample source destinations">
                {ae_action("stitch", "workflow", "Workflow")}{ae_action("stitch", "cloud", "Cloud")}{ae_action("stitch", "mail", "Sample Mail")}
                {ae_action("stitch", "community", "Sample Community")}{ae_action("stitch", "graph", "Object graph")}{ae_action("stitch", "gates", "Gates")}
            </div>
        </section>
    }
}

pub(super) fn deployment_gates() -> [DeploymentGate; 4] {
    [
        DeploymentGate {
            label: "Jenkins parity",
            detail: "Illustrative CI mirror percentage; no required check result measured or verified.",
            state: "active",
            progress: "92%",
        },
        DeploymentGate {
            label: "ArgoCD application",
            detail: "Illustrative application percentage; no isolation or rollback readiness verified.",
            state: "review",
            progress: "74%",
        },
        DeploymentGate {
            label: "Cosign verification",
            detail: "Illustrative signature percentage; no image signature, receipt or key verified.",
            state: "attention",
            progress: "61%",
        },
        DeploymentGate {
            label: "Audit-chain emit",
            detail: "Illustrative audit emission percentage; no deployment event or receipt emitted.",
            state: "draft",
            progress: "48%",
        },
    ]
}

pub(super) fn resource_status_class(state: &str) -> &'static str {
    match state {
        "active" | "sealed" => "status-chip success",
        "attention" | "review" => "status-chip warning",
        "blocked" => "status-chip danger",
        "draft" => "status-chip ai",
        _ => "status-chip",
    }
}

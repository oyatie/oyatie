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

pub(super) fn receipt_stitching_console() -> impl IntoView {
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

pub(super) fn deployment_gates() -> [DeploymentGate; 4] {
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

pub(super) fn resource_status_class(state: &str) -> &'static str {
    match state {
        "active" | "sealed" => "status-chip success",
        "attention" | "review" => "status-chip warning",
        "blocked" => "status-chip danger",
        "draft" => "status-chip ai",
        _ => "status-chip",
    }
}

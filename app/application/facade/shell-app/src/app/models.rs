#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProductSurface {
    Workflow,
    Messenger,
    Mail,
    Community,
}

impl ProductSurface {
    pub(super) const ALL: [Self; 4] =
        [Self::Workflow, Self::Messenger, Self::Mail, Self::Community];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Workflow => "Workflow Studio",
            Self::Messenger => "Messenger",
            Self::Mail => "Mail",
            Self::Community => "Community",
        }
    }

    pub(super) const fn summary(self) -> &'static str {
        match self {
            Self::Workflow => "Build governed no-code flows",
            Self::Messenger => "Discuss operational threads",
            Self::Mail => "Draft formal work messages",
            Self::Community => "Coordinate role-aware spaces",
        }
    }

    pub(super) const fn href(self) -> &'static str {
        match self {
            Self::Workflow => "#workflow-studio",
            Self::Messenger | Self::Mail | Self::Community => "#work-hub",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorkflowTool {
    Select,
    Connect,
    Simulate,
}

impl WorkflowTool {
    pub(super) const ALL: [Self; 3] = [Self::Select, Self::Connect, Self::Simulate];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Connect => "Connect",
            Self::Simulate => "Simulate",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LocalDraft {
    pub(super) surface: ProductSurface,
    pub(super) title: String,
    pub(super) body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HubItem {
    pub(super) surface: ProductSurface,
    pub(super) source: String,
    pub(super) title: String,
    pub(super) body: String,
    pub(super) meta: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ExecutionRow {
    pub(super) id: String,
    pub(super) kind: &'static str,
    pub(super) state: &'static str,
    pub(super) title: String,
    pub(super) body: String,
    pub(super) owner: String,
    pub(super) due: String,
    pub(super) route: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResourceRow {
    pub(super) kind: &'static str,
    pub(super) name: &'static str,
    pub(super) region: &'static str,
    pub(super) owner: &'static str,
    pub(super) state: &'static str,
    pub(super) monthly: &'static str,
    pub(super) risk: &'static str,
    pub(super) side_id: &'static str,
    pub(super) description: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AuditReceipt {
    pub(super) time: &'static str,
    pub(super) event: &'static str,
    pub(super) actor: &'static str,
    pub(super) receipt: &'static str,
    pub(super) severity: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DeploymentGate {
    pub(super) label: &'static str,
    pub(super) detail: &'static str,
    pub(super) state: &'static str,
    pub(super) progress: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BusinessLogicRow {
    pub(super) id: &'static str,
    pub(super) name: &'static str,
    pub(super) english_name: &'static str,
    pub(super) category: &'static str,
    pub(super) owner: &'static str,
    pub(super) cadence: &'static str,
    pub(super) criticality: &'static str,
    pub(super) cost: &'static str,
    pub(super) sla: &'static str,
    pub(super) state: &'static str,
    pub(super) state_label: &'static str,
    pub(super) tasks: &'static str,
    pub(super) route: &'static str,
    pub(super) description: &'static str,
}

pub(super) const BUSINESS_LOGIC_ROWS: [BusinessLogicRow; 7] = [
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

//! Executable ERP/SAP parity composition map for Tenant RBAC planning.
//!
//! The map translates ADR-0315 SAP module parity into flat Oyatie service
//! destinations. It is intentionally an evidence/control-plane artifact: it
//! does not create an ERP platform service, attach cloud infrastructure, deploy a
//! listener, execute Workflow, persist business documents, or emit runtime
//! audit-chain events.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SapModuleCode {
    Fi,
    Co,
    Mm,
    Sd,
    Pp,
    Qm,
    Pm,
    Hcm,
    Ps,
    Plm,
    Ehs,
    Srm,
    Crm,
    ScmApo,
    Gts,
    Tm,
    Ewm,
    Trm,
    ReFx,
    IndustrySolutions,
    Network,
    Platform,
    Data,
}

impl SapModuleCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fi => "FI",
            Self::Co => "CO",
            Self::Mm => "MM",
            Self::Sd => "SD",
            Self::Pp => "PP",
            Self::Qm => "QM",
            Self::Pm => "PM",
            Self::Hcm => "HCM",
            Self::Ps => "PS",
            Self::Plm => "PLM",
            Self::Ehs => "EHS",
            Self::Srm => "SRM",
            Self::Crm => "CRM",
            Self::ScmApo => "SCM/APO",
            Self::Gts => "GTS",
            Self::Tm => "TM",
            Self::Ewm => "EWM",
            Self::Trm => "TRM",
            Self::ReFx => "RE-FX",
            Self::IndustrySolutions => "IS-*",
            Self::Network => "NETWORK",
            Self::Platform => "PLATFORM",
            Self::Data => "DATA",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParityTier {
    FullExistingCoverage,
    PartialExistingCoverage,
    ComposedCoverage,
    NewFlatServiceRequired,
    PartnerOrPackOverlay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HyperscalerParityFacet {
    ControlPlaneApi,
    ResourceModel,
    LifecycleOperations,
    TenantAccountIsolation,
    IamAuthzPolicy,
    QuotaCapacity,
    BillingMetering,
    AuditEventTrail,
    ObservabilitySlos,
    RegionalCellResidency,
    BackupRestoreRollback,
    SecurityThreatModel,
    ComplianceEvidence,
    SdkApiErgonomics,
    OperationalRunbooks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HyperscalerParityStatus {
    /// Reserved for rows backed by semantic, crate-owned executable checks.
    Verified,
    GapTracked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErpHyperscalerParityCriterion {
    pub facet: HyperscalerParityFacet,   // data_class: PUBLIC
    pub benchmark_surface: &'static str, // data_class: PUBLIC
    pub oyatie_evidence_refs: &'static [&'static str], // data_class: PUBLIC
    pub status: HyperscalerParityStatus, // data_class: PUBLIC
    pub gap_closure_gate: &'static str,  // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErpParityModuleCoverage {
    pub sap_code: SapModuleCode,                      // data_class: PUBLIC
    pub module_name: &'static str,                    // data_class: PUBLIC
    pub surfaces: &'static [&'static str],            // data_class: PUBLIC
    pub oyatie_destinations: &'static [&'static str], // data_class: PUBLIC
    pub first_write_owner: &'static str,              // data_class: PUBLIC
    pub tier: ParityTier,                             // data_class: PUBLIC
    pub status: &'static str,                         // data_class: PUBLIC
    pub evidence_refs: &'static [&'static str],       // data_class: PUBLIC
    pub cloud_integration_ready: bool,                // data_class: INTERNAL_ONLY
    pub production_runtime_claimed: bool,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErpParityMapCapabilities {
    pub map_name: &'static str,                         // data_class: PUBLIC
    pub sap_module_count: usize,                        // data_class: PUBLIC
    pub compositional_parity_map_attached: bool,        // data_class: PUBLIC
    pub erp_platform_microservice_created: bool,        // data_class: PUBLIC
    pub deployed_listener_attached: bool,               // data_class: PUBLIC
    pub durable_business_document_store_attached: bool, // data_class: PUBLIC
    pub workflow_engine_execution_attached: bool,       // data_class: PUBLIC
    pub cloud_deployment_attached: bool,                // data_class: PUBLIC
    pub runtime_audit_chain_emission_attached: bool,    // data_class: PUBLIC
    pub schema_version: u32,                            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErpParityMapError {
    MissingRequiredModule(&'static str),
    EmptyDestination(SapModuleCode),
    MissingFirstWriteOwner(SapModuleCode),
    ForbiddenErpPlatformDestination {
        sap_code: SapModuleCode,
        destination: &'static str,
    },
    UnsupportedRuntimeClaim(SapModuleCode),
    MissingHyperscalerParityFacet(&'static str),
    DuplicateHyperscalerParityFacet(HyperscalerParityFacet),
    MissingHyperscalerParityBenchmark(HyperscalerParityFacet),
    EmptyHyperscalerParityEvidence(HyperscalerParityFacet),
    MissingHyperscalerParityGate(HyperscalerParityFacet),
    UnverifiedHyperscalerParityClaim(HyperscalerParityFacet),
}

const REQUIRED_MODULES: &[(SapModuleCode, &str)] = &[
    (SapModuleCode::Fi, "FI"),
    (SapModuleCode::Co, "CO"),
    (SapModuleCode::Mm, "MM"),
    (SapModuleCode::Sd, "SD"),
    (SapModuleCode::Pp, "PP"),
    (SapModuleCode::Qm, "QM"),
    (SapModuleCode::Pm, "PM"),
    (SapModuleCode::Hcm, "HCM"),
    (SapModuleCode::Ps, "PS"),
    (SapModuleCode::Plm, "PLM"),
    (SapModuleCode::Ehs, "EHS"),
    (SapModuleCode::Srm, "SRM"),
    (SapModuleCode::Crm, "CRM"),
    (SapModuleCode::ScmApo, "SCM/APO"),
    (SapModuleCode::Gts, "GTS"),
    (SapModuleCode::Tm, "TM"),
    (SapModuleCode::Ewm, "EWM"),
    (SapModuleCode::Trm, "TRM"),
    (SapModuleCode::ReFx, "RE-FX"),
    (SapModuleCode::IndustrySolutions, "IS-*"),
    (SapModuleCode::Network, "NETWORK"),
    (SapModuleCode::Platform, "PLATFORM"),
    (SapModuleCode::Data, "DATA"),
];

const REQUIRED_HYPERSCALER_PARITY_FACETS: &[(HyperscalerParityFacet, &str)] = &[
    (HyperscalerParityFacet::ControlPlaneApi, "control-plane API"),
    (HyperscalerParityFacet::ResourceModel, "resource model"),
    (
        HyperscalerParityFacet::LifecycleOperations,
        "lifecycle operations",
    ),
    (
        HyperscalerParityFacet::TenantAccountIsolation,
        "tenant/project/account isolation",
    ),
    (HyperscalerParityFacet::IamAuthzPolicy, "IAM/authz policy"),
    (HyperscalerParityFacet::QuotaCapacity, "quota/capacity"),
    (HyperscalerParityFacet::BillingMetering, "billing/metering"),
    (HyperscalerParityFacet::AuditEventTrail, "audit/event trail"),
    (
        HyperscalerParityFacet::ObservabilitySlos,
        "observability/SLOs",
    ),
    (
        HyperscalerParityFacet::RegionalCellResidency,
        "regional/cell/residency behavior",
    ),
    (
        HyperscalerParityFacet::BackupRestoreRollback,
        "backup/restore or rollback",
    ),
    (
        HyperscalerParityFacet::SecurityThreatModel,
        "security/threat model",
    ),
    (
        HyperscalerParityFacet::ComplianceEvidence,
        "compliance evidence",
    ),
    (
        HyperscalerParityFacet::SdkApiErgonomics,
        "SDK/API ergonomics",
    ),
    (
        HyperscalerParityFacet::OperationalRunbooks,
        "operational runbooks",
    ),
];

const ERP_HYPERSCALER_PARITY_MATRIX: &[ErpHyperscalerParityCriterion] = &[
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::ControlPlaneApi,
        benchmark_surface: "hyperscaler resource control planes and ERP SaaS administration APIs",
        oyatie_evidence_refs: &[
            "docs/products/erp-coverage/PRD.md",
            "oya/financial-planning/contracts/openapi-v1.yaml",
            "iam/facade/tenant-rbac-erp-parity-map/src/lib.rs",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Every ERP module parity row must remain API-contract-backed before any runtime claim.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::ResourceModel,
        benchmark_surface: "hyperscaler resource graphs and enterprise ERP module taxonomy",
        oyatie_evidence_refs: &[
            "docs/products/erp-coverage/PRD.md",
            "iam/facade/tenant-rbac-erp-parity-map/src/lib.rs",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "ERP modules must map to flat Oyatie destinations without introducing an ERP platform service.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::LifecycleOperations,
        benchmark_surface: "ERP SaaS module activation and cloud resource lifecycle operations",
        oyatie_evidence_refs: &[
            "docs/products/erp-coverage/PRD.md",
            "oya/financial-planning/capabilities/forecast-version-open.yaml",
            "oya/financial-planning/capabilities/scenario-recalculate.yaml",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Lifecycle semantics stay service-owned until each module publishes capability-level operations.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::TenantAccountIsolation,
        benchmark_surface: "cloud account, project, cell, and ERP SaaS tenant isolation",
        oyatie_evidence_refs: &[
            "oya/financial-planning/manifest.json",
            "oya/financial-planning/contracts/openapi-v1.yaml",
            "iam/facade/tenant-rbac-erp-parity-map/src/lib.rs",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "ERP parity rows must preserve tenant and cell boundaries through existing domain services.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::IamAuthzPolicy,
        benchmark_surface: "cloud IAM, ERP authorization objects, and Cedar policy enforcement",
        oyatie_evidence_refs: &[
            "oya/financial-planning/cedar/policies.cedar",
            "oya/financial-planning/policy/forecast-scenario-authorization.cedar",
            "iam/facade/tenant-rbac-erp-parity-map/tests/parity.rs",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "ERP composition must retain Cedar-backed authorization instead of vendor-suite authority.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::QuotaCapacity,
        benchmark_surface: "cloud quotas, capacity reservations, and SaaS planning workload limits",
        oyatie_evidence_refs: &[
            "oya/financial-planning/manifest.json",
            "oya/financial-planning/dashboards/tenant-cost-and-capacity.json",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Quota and capacity claims must remain explicit per composed service before ERP-wide promotion.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::BillingMetering,
        benchmark_surface: "cloud billing, ERP entitlement, and SaaS usage meters",
        oyatie_evidence_refs: &[
            "docs/products/erp-coverage/PRD.md",
            "oya/financial-planning/manifest.json",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Billing/metering remains DealSet and service-meter-owned until each module exposes meter events.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::AuditEventTrail,
        benchmark_surface: "cloud audit logs and ERP financial control evidence",
        oyatie_evidence_refs: &[
            "docs/products/erp-coverage/PRD.md",
            "oya/financial-planning/contracts/asyncapi-v1.yaml",
            "oya/financial-planning/contracts/openapi-v1.yaml",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Audit event references must remain part of ERP product and API evidence before runtime claims.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::ObservabilitySlos,
        benchmark_surface: "cloud monitoring, ERP operations cockpit, and planning SaaS SLOs",
        oyatie_evidence_refs: &[
            "oya/financial-planning/slos/availability.openslo.yaml",
            "oya/financial-planning/slos/read-latency.openslo.yaml",
            "oya/financial-planning/dashboards/slo-and-error-budget.json",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Each promoted ERP domain must retain SLO and dashboard evidence under its owning service.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::RegionalCellResidency,
        benchmark_surface: "cloud regions, accounts, projects, and sovereign ERP tenant cells",
        oyatie_evidence_refs: &[
            "oya/financial-planning/manifest.json",
            "oya/financial-planning/iac/dr-failover.yaml",
            "oya/financial-planning/runbooks/regional-failover.md",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "ERP parity must preserve cell placement, residency, and pack constraints in composed services.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::BackupRestoreRollback,
        benchmark_surface: "cloud backup/restore, ERP period rollback, and planning scenario replay",
        oyatie_evidence_refs: &[
            "oya/financial-planning/manifest.json",
            "oya/financial-planning/runbooks/driver-model-import-rollback.md",
            "oya/financial-planning/runbooks/regional-failover.md",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Backup, restore, and rollback evidence must stay service-local until an ERP control plane exists.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::SecurityThreatModel,
        benchmark_surface: "cloud shared-responsibility models and ERP segregation-of-duties controls",
        oyatie_evidence_refs: &[
            "docs/products/erp-coverage/PRD.md",
            "oya/financial-planning/dpia/dpia.md",
            "oya/financial-planning/policy/abuse-defence.cedar",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Security posture must remain explicit per module and policy file before ERP-wide exposure.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::ComplianceEvidence,
        benchmark_surface: "cloud compliance reports, SOX evidence, and ERP audit packs",
        oyatie_evidence_refs: &[
            "docs/products/erp-coverage/PRD.md",
            "oya/financial-planning/manifest.json",
            "oya/financial-planning/dpia/dpia.md",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Compliance-pack evidence must be cited before any module parity row moves past composition.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::SdkApiErgonomics,
        benchmark_surface: "cloud SDKs, ERP extension APIs, and financial planning public contracts",
        oyatie_evidence_refs: &[
            "oya/financial-planning/contracts/openapi-v1.yaml",
            "oya/financial-planning/contracts/financial-planning-v1.proto",
            "oya/financial-planning/contracts/asyncapi-v1.yaml",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "SDK/API ergonomics stays contract-backed and provider-neutral before generated clients land.",
    },
    ErpHyperscalerParityCriterion {
        facet: HyperscalerParityFacet::OperationalRunbooks,
        benchmark_surface: "cloud service runbooks, ERP operations, and planning SaaS incident guides",
        oyatie_evidence_refs: &[
            "oya/financial-planning/runbooks/regional-failover.md",
            "oya/financial-planning/runbooks/budget-lock-breakglass.md",
            "oya/financial-planning/runbooks/forecast-version-conflict.md",
        ],
        status: HyperscalerParityStatus::GapTracked,
        gap_closure_gate: "Every promoted ERP capability must retain an owning runbook or remain a tracked gap.",
    },
];

const ERP_PARITY_MODULES: &[ErpParityModuleCoverage] = &[
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Fi,
        module_name: "Financial Accounting",
        surfaces: &[
            "General Ledger",
            "Accounts Receivable",
            "Accounts Payable",
            "Fixed Assets",
            "Bank Accounting",
            "Cash Management",
        ],
        oyatie_destinations: &[
            "specs/microservices/accounting.json",
            "specs/microservices/treasury.json",
            "microservices/treasury/crates/oya-treasury-cash-domain",
            "microservices/payments",
            "microservices/finops-portal",
            "microservices/treasury",
        ],
        first_write_owner: "accounting",
        tier: ParityTier::ComposedCoverage,
        status: "accounting-plus-treasury-cash-domain-foundation",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-accounting-journal-domain-foundation-1779522601.json",
            "evidence/multispectrum/cs-ent-accounting-storage-adapter-inmemory-1779540600.json",
            "evidence/multispectrum/cs-ent-treasury-cash-domain-1779546000.json",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Co,
        module_name: "Controlling",
        surfaces: &[
            "Cost Centers",
            "Internal Orders",
            "Profit Centers",
            "CO-PA",
            "Product Costing",
        ],
        oyatie_destinations: &[
            "microservices/finops-portal",
            "microservices/ontology",
            "microservices/workflow-engine",
            "microservices/supply-chain-planning",
        ],
        first_write_owner: "finops-portal",
        tier: ParityTier::ComposedCoverage,
        status: "covered-by-composition",
        evidence_refs: &[
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Mm,
        module_name: "Materials Management",
        surfaces: &[
            "Procurement",
            "Inventory Management",
            "Goods Receipt",
            "Vendor Master Data",
            "Purchase Requisitions",
        ],
        oyatie_destinations: &[
            "specs/microservices/procurement.json",
            "microservices/procurement/crates/oya-procurement-source-to-pay-domain",
            "specs/microservices/warehouse.json",
            "microservices/warehouse/crates/oya-warehouse-inventory-domain",
            "microservices/marketplace",
            "microservices/workflow-engine",
            "microservices/warehouse",
        ],
        first_write_owner: "procurement-source-to-pay",
        tier: ParityTier::ComposedCoverage,
        status: "procurement-and-warehouse-domain-foundations",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-procurement-source-to-pay-domain-1779545400.json",
            "evidence/multispectrum/cs-ent-warehouse-inventory-domain-1779546600.json",
            "docs/decisions/ADR-0705-product-protocol-live-apex.md",
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Sd,
        module_name: "Sales and Distribution",
        surfaces: &[
            "Sales Orders",
            "Pricing",
            "Deliveries",
            "Billing",
            "Credit Management",
            "Customer Master Data",
        ],
        oyatie_destinations: &[
            "microservices/marketplace",
            "microservices/payments",
            "microservices/crm",
            "microservices/warehouse",
        ],
        first_write_owner: "marketplace",
        tier: ParityTier::ComposedCoverage,
        status: "partial-existing-plus-new-crm-and-warehouse",
        evidence_refs: &[
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Pp,
        module_name: "Production Planning",
        surfaces: &["BOM", "MRP", "Capacity Planning", "Shop Floor", "Routing"],
        oyatie_destinations: &[
            "specs/microservices/production-planning.json",
            "microservices/production-planning/crates/oya-production-planning-domain",
            "microservices/production-planning",
        ],
        first_write_owner: "production-planning-domain",
        tier: ParityTier::ComposedCoverage,
        status: "production-planning-domain-foundation-with-execution-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-production-planning-domain-1779547200.json",
            "microservices/production-planning/PHASE-01-PRODUCTION-PLANNING-PARITY.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Qm,
        module_name: "Quality Management",
        surfaces: &[
            "Inspection Plans",
            "Inspection Lots",
            "Usage Decisions",
            "Certificates of Analysis",
            "Quality Notifications",
            "Nonconformance Management",
        ],
        oyatie_destinations: &[
            "specs/microservices/quality-management.json",
            "microservices/quality-management/crates/oya-quality-management-domain",
            "microservices/quality-management",
        ],
        first_write_owner: "quality-management-domain",
        tier: ParityTier::ComposedCoverage,
        status: "quality-management-domain-foundation-with-runtime-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-quality-management-domain-1779547800.json",
            "microservices/quality-management/PHASE-01-QUALITY-MANAGEMENT-PARITY.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Pm,
        module_name: "Plant Maintenance",
        surfaces: &[
            "Equipment Master",
            "Functional Locations",
            "Maintenance Plans",
            "Work Orders",
            "Preventive Maintenance",
            "Spare Parts",
        ],
        oyatie_destinations: &[
            "specs/microservices/plant-maintenance.json",
            "microservices/plant-maintenance/crates/oya-plant-maintenance-domain",
            "microservices/plant-maintenance",
        ],
        first_write_owner: "plant-maintenance-domain",
        tier: ParityTier::ComposedCoverage,
        status: "plant-maintenance-domain-foundation-with-runtime-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-plant-maintenance-domain-1779548400.json",
            "microservices/plant-maintenance/PHASE-01-PLANT-MAINTENANCE-PARITY.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Hcm,
        module_name: "Human Capital Management",
        surfaces: &[
            "Organizational Management",
            "Personnel Administration",
            "Time Management",
            "Payroll",
            "Talent Management",
        ],
        oyatie_destinations: &[
            "specs/microservices/hr.json",
            "specs/microservices/payroll.json",
            "docs/products/workplace-integration/PRD.md",
            "microservices/workflow-engine",
        ],
        first_write_owner: "hr-employment",
        tier: ParityTier::ComposedCoverage,
        status: "planned-existing-spec-coverage",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-hr-domain-foundation-1779520348.json",
            "evidence/multispectrum/cs-ent-payroll-run-domain-foundation-1779522600.json",
            "evidence/multispectrum/cs-ent-platform-local-inmemory-harness-1779541800.json",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Ps,
        module_name: "Project System",
        surfaces: &[
            "Work Breakdown Structure",
            "Networks",
            "Milestone Billing",
            "Project Cost Management",
        ],
        oyatie_destinations: &[
            "microservices/workflow-engine",
            "microservices/ontology",
            "microservices/finops-portal",
            "microservices/payments",
        ],
        first_write_owner: "workflow-engine",
        tier: ParityTier::ComposedCoverage,
        status: "covered-by-composition",
        evidence_refs: &[
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Plm,
        module_name: "Product Lifecycle Management",
        surfaces: &["Master Data Governance", "Engineering Change Management"],
        oyatie_destinations: &[
            "microservices/ontology",
            "microservices/workflow-engine",
            "microservices/connector",
            "microservices/production-planning",
        ],
        first_write_owner: "ontology",
        tier: ParityTier::ComposedCoverage,
        status: "covered-by-composition",
        evidence_refs: &[
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Ehs,
        module_name: "Environment Health and Safety",
        surfaces: &[
            "Hazardous Substances",
            "Industrial Hygiene",
            "Incident Management",
        ],
        oyatie_destinations: &[
            "microservices/compliance",
            "microservices/workflow-engine",
            "microservices/ontology",
            "microservices/quality-management",
        ],
        first_write_owner: "compliance",
        tier: ParityTier::PartnerOrPackOverlay,
        status: "covered-by-composition-plus-pack-overlays",
        evidence_refs: &[
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Srm,
        module_name: "Supplier Relationship Management",
        surfaces: &["Sourcing", "Contract Management", "Supplier Performance"],
        oyatie_destinations: &[
            "specs/microservices/procurement.json",
            "microservices/procurement/crates/oya-procurement-source-to-pay-domain",
            "microservices/marketplace",
            "microservices/workflow-engine",
            "microservices/ontology",
            "microservices/payments",
        ],
        first_write_owner: "procurement-source-to-pay",
        tier: ParityTier::ComposedCoverage,
        status: "procurement-supplier-foundation-plus-composition",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-procurement-source-to-pay-domain-1779545400.json",
            "docs/decisions/ADR-0705-product-protocol-live-apex.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Crm,
        module_name: "Customer Relationship Management",
        surfaces: &["Sales Force", "Service", "Marketing", "Loyalty"],
        oyatie_destinations: &[
            "specs/microservices/crm.json",
            "microservices/crm/crates/oya-crm-customer-engagement-domain",
            "microservices/crm",
            "microservices/community",
            "microservices/marketplace",
            "microservices/intelligence",
        ],
        first_write_owner: "crm-customer-engagement-domain",
        tier: ParityTier::ComposedCoverage,
        status: "crm-customer-engagement-domain-foundation-with-runtime-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-crm-customer-engagement-domain-1779549600.json",
            "microservices/crm/PRD.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::ScmApo,
        module_name: "Supply Chain Management and Advanced Planning",
        surfaces: &[
            "Demand Planning",
            "Supply Network Planning",
            "Detailed Scheduling",
            "Global ATP",
            "Transportation Planning",
        ],
        oyatie_destinations: &[
            "specs/microservices/supply-chain-planning.json",
            "microservices/supply-chain-planning/crates/oya-supply-chain-planning-domain",
            "microservices/supply-chain-planning",
            "microservices/production-planning",
            "microservices/warehouse",
        ],
        first_write_owner: "supply-chain-planning-domain",
        tier: ParityTier::ComposedCoverage,
        status: "supply-chain-planning-domain-foundation-with-runtime-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-supply-chain-planning-domain-1779549000.json",
            "microservices/supply-chain-planning/PRD.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Gts,
        module_name: "Global Trade Services",
        surfaces: &[
            "Customs Management",
            "Sanctioned Party Screening",
            "Export Control",
            "Trade Compliance",
        ],
        oyatie_destinations: &[
            "specs/microservices/global-trade.json",
            "microservices/global-trade/crates/oya-global-trade-compliance-domain",
            "microservices/global-trade",
            "microservices/compliance",
            "microservices/connector",
        ],
        first_write_owner: "global-trade-compliance-domain",
        tier: ParityTier::ComposedCoverage,
        status: "global-trade-compliance-domain-foundation-with-runtime-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-global-trade-compliance-domain-1779550200.json",
            "microservices/global-trade/PRD.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Tm,
        module_name: "Transportation Management",
        surfaces: &[
            "Freight Order Management",
            "Carrier Selection",
            "Charge Management",
        ],
        oyatie_destinations: &[
            "microservices/supply-chain-planning",
            "microservices/warehouse",
            "microservices/marketplace",
            "microservices/global-trade",
        ],
        first_write_owner: "supply-chain-planning",
        tier: ParityTier::ComposedCoverage,
        status: "covered-by-initial-composition",
        evidence_refs: &[
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Ewm,
        module_name: "Extended Warehouse Management",
        surfaces: &[
            "Inbound Processing",
            "Outbound Processing",
            "Slotting",
            "Yard Management",
            "Labor Management",
        ],
        oyatie_destinations: &[
            "specs/microservices/warehouse.json",
            "microservices/warehouse/crates/oya-warehouse-inventory-domain",
            "microservices/warehouse",
        ],
        first_write_owner: "warehouse-inventory",
        tier: ParityTier::ComposedCoverage,
        status: "warehouse-inventory-domain-foundation-with-runtime-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-warehouse-inventory-domain-1779546600.json",
            "microservices/warehouse/PRD.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Trm,
        module_name: "Treasury and Risk Management",
        surfaces: &[
            "Cash Management",
            "Liquidity Planning",
            "FX Management",
            "Debt Management",
            "Hedging",
        ],
        oyatie_destinations: &[
            "specs/microservices/treasury.json",
            "microservices/treasury/crates/oya-treasury-cash-domain",
            "microservices/treasury",
            "microservices/payments",
            "microservices/finops-portal",
        ],
        first_write_owner: "treasury-cash",
        tier: ParityTier::ComposedCoverage,
        status: "treasury-cash-domain-foundation-with-payment-bank-network-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-treasury-cash-domain-1779546000.json",
            "microservices/treasury/PRD.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::ReFx,
        module_name: "Real Estate Flexible Management",
        surfaces: &[
            "Lease Management",
            "Facility Management",
            "Space Management",
        ],
        oyatie_destinations: &[
            "specs/microservices/real-estate.json",
            "microservices/real-estate/crates/oya-real-estate-portfolio-domain",
            "microservices/real-estate",
            "microservices/plant-maintenance",
            "microservices/finops-portal",
        ],
        first_write_owner: "real-estate-portfolio-domain",
        tier: ParityTier::ComposedCoverage,
        status: "real-estate-portfolio-domain-foundation-with-runtime-nonclaims",
        evidence_refs: &[
            "evidence/multispectrum/cs-ent-real-estate-portfolio-domain-1779550800.json",
            "microservices/real-estate/PRD.md",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::IndustrySolutions,
        module_name: "Industry Solutions",
        surfaces: &[
            "Banking",
            "Insurance",
            "Retail",
            "Healthcare",
            "Public Sector",
            "Automotive",
            "Utilities",
            "Oil and Gas",
            "Pharma",
        ],
        oyatie_destinations: &[
            "packs/industry",
            "microservices/ontology",
            "microservices/workflow-engine",
            "microservices/compliance",
        ],
        first_write_owner: "industry-pack-overlay",
        tier: ParityTier::PartnerOrPackOverlay,
        status: "pack-overlay-no-vertical-platform",
        evidence_refs: &[
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Network,
        module_name: "Network Products",
        surfaces: &[
            "Ariba",
            "Concur",
            "Fieldglass",
            "SuccessFactors",
            "Commerce Network",
        ],
        oyatie_destinations: &[
            "microservices/marketplace",
            "microservices/payments",
            "docs/products/workplace-integration/PRD.md",
            "microservices/crm",
        ],
        first_write_owner: "marketplace",
        tier: ParityTier::ComposedCoverage,
        status: "covered-by-composition",
        evidence_refs: &["docs/decisions/ADR-0705-product-protocol-live-apex.md"],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Platform,
        module_name: "Platform and Extensibility",
        surfaces: &["SAP BTP", "CAP", "Fiori", "Extension Runtime"],
        oyatie_destinations: &[
            "microservices/plugin-app-store",
            "microservices/developer-sdk",
            "microservices/workflow-studio",
            "microservices/workflow-engine",
            "microservices/ontology",
        ],
        first_write_owner: "developer-sdk",
        tier: ParityTier::ComposedCoverage,
        status: "covered-by-composition-no-proprietary-runtime-clone",
        evidence_refs: &[
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
    ErpParityModuleCoverage {
        sap_code: SapModuleCode::Data,
        module_name: "Data and Analytics",
        surfaces: &["Analytics Cloud", "Datasphere", "HANA-like Analytics"],
        oyatie_destinations: &[
            "microservices/analytics",
            "microservices/ontology",
            "microservices/intelligence",
            "microservices/observability",
            "microservices/data-warehouse",
        ],
        first_write_owner: "analytics",
        tier: ParityTier::ComposedCoverage,
        status: "covered-by-composition-with-data-warehouse-in-flight",
        evidence_refs: &[
            "microservices/data-warehouse/PRD.md",
            "docs/architecture/tenant-rbac-software-coverage-matrix-2026-05-21.md#3.1",
        ],
        cloud_integration_ready: false,
        production_runtime_claimed: false,
    },
];

pub fn tenant_rbac_erp_parity_map() -> &'static [ErpParityModuleCoverage] {
    ERP_PARITY_MODULES
}

pub fn erp_parity_map_capabilities() -> ErpParityMapCapabilities {
    ErpParityMapCapabilities {
        map_name: "tenant-rbac-erp-sap-parity-composition-map",
        sap_module_count: ERP_PARITY_MODULES.len(),
        compositional_parity_map_attached: true,
        erp_platform_microservice_created: false,
        deployed_listener_attached: false,
        durable_business_document_store_attached: false,
        workflow_engine_execution_attached: false,
        cloud_deployment_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: 1,
    }
}

pub fn erp_hyperscaler_parity_matrix() -> &'static [ErpHyperscalerParityCriterion] {
    ERP_HYPERSCALER_PARITY_MATRIX
}

pub fn find_erp_module(code: SapModuleCode) -> Option<&'static ErpParityModuleCoverage> {
    ERP_PARITY_MODULES
        .iter()
        .find(|module| module.sap_code == code)
}

pub fn validate_erp_parity_map(rows: &[ErpParityModuleCoverage]) -> Result<(), ErpParityMapError> {
    for (required, label) in REQUIRED_MODULES {
        if !rows.iter().any(|row| row.sap_code == *required) {
            return Err(ErpParityMapError::MissingRequiredModule(label));
        }
    }

    for row in rows {
        if row.oyatie_destinations.is_empty() {
            return Err(ErpParityMapError::EmptyDestination(row.sap_code));
        }
        if row.first_write_owner.trim().is_empty() {
            return Err(ErpParityMapError::MissingFirstWriteOwner(row.sap_code));
        }
        if row.cloud_integration_ready || row.production_runtime_claimed {
            return Err(ErpParityMapError::UnsupportedRuntimeClaim(row.sap_code));
        }
        if let Some(destination) = row
            .oyatie_destinations
            .iter()
            .copied()
            .find(|destination| is_forbidden_erp_platform_destination(destination))
        {
            return Err(ErpParityMapError::ForbiddenErpPlatformDestination {
                sap_code: row.sap_code,
                destination,
            });
        }
    }

    Ok(())
}

pub fn validate_erp_hyperscaler_parity_matrix(
    matrix: &[ErpHyperscalerParityCriterion],
) -> Result<(), ErpParityMapError> {
    for (required, label) in REQUIRED_HYPERSCALER_PARITY_FACETS {
        let count = matrix
            .iter()
            .filter(|criterion| criterion.facet == *required)
            .count();
        if count == 0 {
            return Err(ErpParityMapError::MissingHyperscalerParityFacet(label));
        }
        if count > 1 {
            return Err(ErpParityMapError::DuplicateHyperscalerParityFacet(
                *required,
            ));
        }
    }

    for criterion in matrix {
        if criterion.benchmark_surface.trim().is_empty() {
            return Err(ErpParityMapError::MissingHyperscalerParityBenchmark(
                criterion.facet,
            ));
        }
        if criterion.oyatie_evidence_refs.is_empty() {
            return Err(ErpParityMapError::EmptyHyperscalerParityEvidence(
                criterion.facet,
            ));
        }
        if criterion.gap_closure_gate.trim().is_empty() {
            return Err(ErpParityMapError::MissingHyperscalerParityGate(
                criterion.facet,
            ));
        }
        if criterion.status == HyperscalerParityStatus::Verified {
            return Err(ErpParityMapError::UnverifiedHyperscalerParityClaim(
                criterion.facet,
            ));
        }
    }

    Ok(())
}

pub fn is_forbidden_erp_platform_destination(destination: &str) -> bool {
    let normalized = destination.trim().to_ascii_lowercase();
    normalized == "microservices/erp"
        || normalized.starts_with("microservices/erp/")
        || normalized == "microservices/erp-platform"
        || normalized.starts_with("microservices/erp-platform/")
}

// j176-migration-from-sap-s4hana-to-oyatie-finance-month-1 Cedar policy
// Incumbent: SAP S/4HANA Finance
// Target: Oyatie finance
// ADRs: ADR-0131-per-microservice-flat-layout, ADR-0145-inter-microservice-communication-reform, ADR-0243-cedar-as-universal-gate, ADR-0244-tenant-as-universal-scoping-primitive, ADR-0251-compliance-pack-cell-certification-levels, ADR-0263-observability-emission-contract, ADR-0317-role-based-projection-unified-ux-shell

permit (
    principal == User::"mara.bell@brindleworks-manufacturing.example",
    action in [
        Action::"migration.extract.start",
        Action::"migration.extract.read",
        Action::"migration.mapping.apply",
        Action::"migration.projection.load",
        Action::"migration.parallel_run.compare",
        Action::"migration.exception.route",
        Action::"migration.rollback.rehearse",
        Action::"migration.cutover.promote",
        Action::"migration.archive.seal"
    ],
    resource is VendorMigration
) when {
    resource.tenant_id == "brindleworks-manufacturing" &&
    resource.source_system == "sap-s4hana-prd-100" &&
    context.passkey_step_up_within_seconds <= 120 &&
    context.change_window_id != "" &&
    context.rollback_plan_id != "" &&
    context.audit_chain_enabled == true
};

permit (
    principal == ServicePrincipal::"workflow-engine",
    action in [
        Action::"migration.exception.route",
        Action::"migration.rollback.rehearse",
        Action::"migration.cutover.pause"
    ],
    resource is VendorMigration
) when {
    resource.tenant_id == "brindleworks-manufacturing" &&
    context.material_delta_class in ["P0", "P1", "P2", "P3"] &&
    context.audit_event_class.startsWith("EVT-J176-")
};

permit (
    principal == ServicePrincipal::"data-pipeline",
    action in [
        Action::"migration.extract.read",
        Action::"migration.projection.load"
    ],
    resource is VendorExtractBatch
) when {
    resource.tenant_id == "brindleworks-manufacturing" &&
    resource.source_system == "sap-s4hana-prd-100" &&
    resource.row_hash_algorithm == "sha256" &&
    resource.object_name in [
        "BKPF",
        "BSEG",
        "ACDOCA",
        "SKAT",
        "T001"
    ]
};

forbid (
    principal,
    action == Action::"migration.cutover.promote",
    resource is VendorMigration
) when {
    context.open_p0_delta_count > 0 ||
    context.rollback_rehearsal_passed == false ||
    context.audit_chain_enabled == false
};

forbid (
    principal,
    action == Action::"migration.extract.read",
    resource is VendorExtractBatch
) when {
    resource.tenant_id != "brindleworks-manufacturing" ||
    context.source_credential_inline == true
};

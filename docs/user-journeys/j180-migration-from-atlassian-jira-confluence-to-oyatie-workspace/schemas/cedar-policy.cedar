// j180-migration-from-atlassian-jira-confluence-to-oyatie-workspace Cedar policy
// Incumbent: Atlassian Jira Software plus Confluence
// Target: Oyatie workspace
// ADRs: ADR-0131-per-microservice-flat-layout, ADR-0145-inter-microservice-communication-reform, ADR-0243-cedar-as-universal-gate, ADR-0244-tenant-as-universal-scoping-primitive, ADR-0251-compliance-pack-cell-certification-levels, ADR-0263-observability-emission-contract, ADR-0317-role-based-projection-unified-ux-shell

permit (
    principal == User::"nora.stein@atlasbridge-robotics.example",
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
    resource.tenant_id == "atlasbridge-robotics" &&
    resource.source_system == "atlassian-cloud-site-atlasbridge" &&
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
    resource.tenant_id == "atlasbridge-robotics" &&
    context.material_delta_class in ["P0", "P1", "P2", "P3"] &&
    context.audit_event_class.startsWith("EVT-J180-")
};

permit (
    principal == ServicePrincipal::"data-pipeline",
    action in [
        Action::"migration.extract.read",
        Action::"migration.projection.load"
    ],
    resource is VendorExtractBatch
) when {
    resource.tenant_id == "atlasbridge-robotics" &&
    resource.source_system == "atlassian-cloud-site-atlasbridge" &&
    resource.row_hash_algorithm == "sha256" &&
    resource.object_name in [
        "Issue types",
        "Workflow schemes",
        "Permission schemes",
        "Confluence space",
        "Project board"
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
    resource.tenant_id != "atlasbridge-robotics" ||
    context.source_credential_inline == true
};

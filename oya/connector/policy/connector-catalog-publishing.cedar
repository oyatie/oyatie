// Cedar v4.2 LTS fragment — connector-catalog-publishing
// microservice: connector / BC: connector-catalog
// soak-window-seconds: 60
// publish-stage: active
// binding-adr: ADR-0243, ADR-0247, ADR-0249, ADR-0294
//
// Governs who may publish, update, or retire connector adapters in the catalog.
// Default-deny baseline. All permits are explicit. Defence-in-depth FORBID rules
// gate high-risk publish/retire actions even for authenticated publishers.

// ─── Default-deny baseline ───────────────────────────────────────────────────
forbid (principal, action, resource)
unless {
  action in [
    Action::"catalog:read",
    Action::"catalog:publish",
    Action::"catalog:update-metadata",
    Action::"catalog:retire",
    Action::"catalog:attest"
  ]
};

// ─── Public catalog read ──────────────────────────────────────────────────────
// Any authenticated tenant may browse the connector catalog.
permit (
  principal in Group::"authenticated-tenants",
  action == Action::"catalog:read",
  resource in ResourceType::"ConnectorCatalogEntry"
)
when {
  principal.tenant.lifecycle_state == "active"
};

// ─── Internal oyatie catalog read (cross-tenant) ──────────────────────────────
permit (
  principal in Group::"oyatie.connector.catalog-api",
  action == Action::"catalog:read",
  resource in ResourceType::"ConnectorCatalogEntry"
);

// ─── Marketplace publisher: publish new connector ─────────────────────────────
// MPO (Marketplace Publisher Operator) may publish after security-review attestation.
permit (
  principal,
  action == Action::"catalog:publish",
  resource in ResourceType::"ConnectorCatalogEntry"
)
when {
  principal has marketplace_publisher_role &&
  principal.marketplace_publisher_role == "mpo" &&
  resource has security_review_attested &&
  resource.security_review_attested == true &&
  resource has sbom_provided &&
  resource.sbom_provided == true &&
  // Publisher may only publish under their own namespace
  resource.publisher_namespace == principal.publisher_namespace
};

// ─── FORBID: publish without security review ─────────────────────────────────
forbid (
  principal,
  action == Action::"catalog:publish",
  resource in ResourceType::"ConnectorCatalogEntry"
)
when {
  !(resource has security_review_attested) ||
  resource.security_review_attested == false
};

// ─── oyatie internal publish (first-party connectors) ────────────────────────
permit (
  principal in Group::"oyatie.connector.catalog-api",
  action in [Action::"catalog:publish", Action::"catalog:update-metadata"],
  resource in ResourceType::"ConnectorCatalogEntry"
)
when {
  resource.publisher_namespace == "oyatie"
};

// ─── Metadata update (own connectors) ────────────────────────────────────────
permit (
  principal,
  action == Action::"catalog:update-metadata",
  resource in ResourceType::"ConnectorCatalogEntry"
)
when {
  principal has marketplace_publisher_role &&
  resource.publisher_namespace == principal.publisher_namespace
};

// ─── Retire (own connectors; soft-retire only) ───────────────────────────────
permit (
  principal,
  action == Action::"catalog:retire",
  resource in ResourceType::"ConnectorCatalogEntry"
)
when {
  principal has marketplace_publisher_role &&
  resource.publisher_namespace == principal.publisher_namespace &&
  // Retirement requires prior sunset-notice ≥30d
  resource has sunset_notice_issued &&
  resource.sunset_notice_issued == true
};

// ─── FORBID: hard-delete any connector entry ─────────────────────────────────
// Connectors are soft-retired only; hard-delete is forbidden (audit-trail integrity).
forbid (
  principal,
  action == Action::"catalog:hard-delete",
  resource in ResourceType::"ConnectorCatalogEntry"
);

// ─── Attestation (security team + CI) ────────────────────────────────────────
permit (
  principal in [Group::"security-review-team", Group::"oyatie.connector.ci-scope"],
  action == Action::"catalog:attest",
  resource in ResourceType::"ConnectorCatalogEntry"
);

// ─── Anti-scrape: unauthenticated bulk-read forbidden ────────────────────────
forbid (
  principal,
  action == Action::"catalog:read",
  resource in ResourceType::"ConnectorCatalogEntry"
)
when {
  !(principal has tenant) &&
  resource has request_count_last_60s &&
  resource.request_count_last_60s > 30
};

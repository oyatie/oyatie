// community µservice — ci-scope policy fragment
// Scope: CI agents may read non-tenant data (schemas, types, indexes) only.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// CI agents may read schema + manifests but never tenant data
permit (
  principal in CiAgent::?p,
  action in [
    Action::"read_openslo_manifests",
    Action::"read_openapi_spec",
    Action::"read_proto_spec",
    Action::"read_catalog_record",
    Action::"read_iac_chart"
  ],
  resource in CommunityArtifact::?r
)
when {
  resource has artifact_class &&
  resource.artifact_class in ["openslo", "openapi", "proto", "catalog", "iac"]
};

// Explicit forbid — CI agent may never read tenant data
forbid (
  principal in CiAgent::?p,
  action in [
    Action::"read_post",
    Action::"read_kb_article",
    Action::"read_vote_tally",
    Action::"read_moderation_action",
    Action::"read_thread"
  ],
  resource
);

// Audit hook
forbid (
  principal in CiAgent::?p,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class in ["BEHAVIORAL_TENANT_PRODUCT", "AUDIT", "SECRET"]
};

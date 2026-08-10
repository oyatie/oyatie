// translate µservice — tenant-scope policy fragment
// Cedar v4.2; conforms to microservices/translate/policy/schema.cedarschema
//
// Purpose: enforce per-tenant + per-pack scope on translate REST + WS + bulk +
//          TM + termbase + doc + stream surfaces.
// Owner:   ops-security + axis-translate
// Audit:   every deny emits oya_translate_unauthorized_attempt_total
// References:
//  - ADR-TRANSLATE-0004 (residency-bound inference)
//  - microservices/translate/policy/data-residency.md
//  - microservices/translate/threat-model.md (T-04, T-05, P-01, P-05)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// DEFAULT DENY — every action is refused unless an explicit PERMIT matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant principal reads/writes own tenant's translate resources
// =============================================================================

permit (
  principal in TenantUser::?p,
  action in [
    Action::"translate.invoke",
    Action::"translate.batch",
    Action::"translate.detect_language",
    Action::"translate.estimate_quality",
    Action::"tm.lookup",
    Action::"tm.upsert",
    Action::"tm.export",
    Action::"tm.import",
    Action::"termbase.read",
    Action::"termbase.write",
    Action::"termbase.import",
    Action::"termbase.export",
    Action::"bulk.submit",
    Action::"bulk.read",
    Action::"bulk.cancel",
    Action::"doc.translate",
    Action::"doc.read",
    Action::"stream.open"
  ],
  resource in TenantResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant administrator manages tenant policy
// =============================================================================

permit (
  principal in TenantAdmin::?p,
  action in [
    Action::"policy.read",
    Action::"policy.update",
    Action::"language_pack.enable",
    Action::"language_pack.disable",
    Action::"engine_pin.set",
    Action::"engine_pin.clear",
    Action::"glossary.review",
    Action::"glossary.publish"
  ],
  resource in TenantResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 3 — workflow-engine principals invoke human-in-the-loop transitions
// =============================================================================

permit (
  principal in WorkflowEngine::?p,
  action in [
    Action::"translate.invoke",
    Action::"translate.review_complete",
    Action::"tm.upsert",
    Action::"qe.estimate"
  ],
  resource in TenantResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 4 — sibling µservice service principals (mail, messenger, social,
// docs, sheets, slides, meet, shorts, workflow-studio) invoke translate on
// behalf of an authenticated end-user
// =============================================================================

permit (
  principal in SiblingService::?p,
  action in [
    Action::"translate.invoke",
    Action::"translate.batch",
    Action::"translate.detect_language",
    Action::"translate.estimate_quality",
    Action::"stream.open"
  ],
  resource in TenantResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has on_behalf_of_user_id &&
  resource has end_user_id &&
  principal.on_behalf_of_user_id == resource.end_user_id
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defense-in-depth; T-04, T-05)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"translate.invoke",
    Action::"translate.batch",
    Action::"tm.lookup",
    Action::"tm.export",
    Action::"termbase.read",
    Action::"bulk.read",
    Action::"doc.read",
    Action::"stream.open"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — explicit cross-pack deny (P-01 + P-05 + FM-72)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"translate.invoke",
    Action::"tm.lookup",
    Action::"bulk.read",
    Action::"doc.read",
    Action::"stream.open"
  ],
  resource
)
unless {
  principal has pack &&
  resource has pack &&
  principal.pack == resource.pack
};

// =============================================================================
// FORBID — engine selection outside pack whitelist
// =============================================================================

forbid (
  principal,
  action == Action::"engine.invoke",
  resource in EngineCandidate::?ec
)
unless {
  principal has pack &&
  resource has permitted_packs &&
  principal.pack in resource.permitted_packs
};

// =============================================================================
// FORBID — pack-cn-stub external vendor block
// =============================================================================

forbid (
  principal,
  action == Action::"engine.invoke",
  resource in EngineCandidate::?ec
)
when {
  principal has pack &&
  principal.pack == "cn" &&
  resource has vendor &&
  resource.vendor in ["anthropic", "openai", "google-translate", "deepl", "microsoft-translator", "amazon-translate"]
};

// =============================================================================
// FORBID — high-risk content class without FRIA
// =============================================================================

forbid (
  principal,
  action == Action::"qe.estimate",
  resource in TenantResource::?r
)
when {
  resource has content_class &&
  resource.content_class in ["medical", "employment", "credit", "legal"] &&
  principal has fria_on_file &&
  principal.fria_on_file == false
};

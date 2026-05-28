// LANE-1 hardening (2026-05-28): the previous policy carried an implicit
// `principal in Role::"InternalDogfood"` escape clause that shorted forbid
// rules — incompatible with the default-deny forbid-wins semantics required
// by ADR-0183 (Cedar app-authz + Kyverno admission). That bypass is removed.
//
// Operators using dogfood principals MUST now grant them explicit Role
// bindings (admin / ingress_proxy / audit) — no implicit superuser.
//
// Migration path tracked in:
//   registry/placeholder-debt/adr-follow-ups.yaml#llm-gateway-internaldogfood-bypass-removal

// Cedar policy — llm-gateway µservice realm + tenant authorization.
//
// Authority: ADR-0007 Cedar fragment coverage, ADR-0373 (key-pool resilience +
// per-tenant isolation), ADR-0373 (audit). Research grounding:
// microservices/llm-gateway/design/hyperscaler-best-practice-brief.md
// (brief §2 two security schemes / per-operation realm; brief §5 constant-time tokens,
//  per-realm rate limits; brief §6 per-tenant isolation unit; brief §7 audit-read gated
//  behind a distinct authority).
//
// Two realms, modeled as Cedar Roles bound to the bearer token at authentication:
//   Role::"IngressRealm"  — the agent/tenant data plane (InvokeChatCompletion,
//                           InvokeEmbeddings, ListModels). Tenant-scoped.
//   Role::"AdminRealm"    — control plane (ReadPoolStatus, RefreshKeyPool).
//   Role::"AuditReader"   — distinct audit-read authority (ReadAudit), SIEM-forwarded.
//
// The data-plane actions are tenant-scoped: a principal may only invoke against its own
// per-tenant key pool (the isolation unit, brief §6). Admin/audit are realm-scoped, not
// tenant-scoped.

// --- Ingress data plane: tenant-scoped inference ---

@id("llm-gateway-ingress-chat-same-tenant")
permit (
  principal in Role::"IngressRealm",
  action == Action::"InvokeChatCompletion",
  resource
) when {
  principal.tenant_id == resource.tenant_id
};

@id("llm-gateway-ingress-embeddings-same-tenant")
permit (
  principal in Role::"IngressRealm",
  action == Action::"InvokeEmbeddings",
  resource
) when {
  principal.tenant_id == resource.tenant_id
};

@id("llm-gateway-ingress-list-models")
permit (
  principal in Role::"IngressRealm",
  action == Action::"ListModels",
  resource
);

// --- Admin control plane: realm-scoped (NOT available to ingress tokens) ---

@id("llm-gateway-admin-pool-status")
permit (
  principal in Role::"AdminRealm",
  action == Action::"ReadPoolStatus",
  resource
);

@id("llm-gateway-admin-key-refresh")
permit (
  principal in Role::"AdminRealm",
  action == Action::"RefreshKeyPool",
  resource
);

// --- Audit read: distinct authority, gated separately from admin (brief §7) ---

@id("llm-gateway-audit-read-distinct-authority")
permit (
  principal in Role::"AuditReader",
  action == Action::"ReadAudit",
  resource
);

// --- Cross-realm explicit-deny ---
// Forbid wins in Cedar, so every action family below has an explicit negative
// boundary in addition to the narrow permits above. This is the Cedar expression
// of the brief's "two securitySchemes, realm marked per-operation" (brief §2):
// ingress tokens cannot operate the control/audit plane, admin tokens cannot
// invoke tenant inference, and audit-reader tokens are read-only for audit.

@id("llm-gateway-forbid-inference-outside-ingress-realm")
forbid (
  principal,
  action in [Action::"InvokeChatCompletion", Action::"InvokeEmbeddings"],
  resource
)
unless {
  principal in Role::"IngressRealm"
};

@id("llm-gateway-forbid-list-models-outside-ingress-realm")
forbid (
  principal,
  action == Action::"ListModels",
  resource
)
unless {
  principal in Role::"IngressRealm"
};

@id("llm-gateway-forbid-admin-actions-outside-admin-realm")
forbid (
  principal,
  action in [Action::"ReadPoolStatus", Action::"RefreshKeyPool"],
  resource
)
unless {
  principal in Role::"AdminRealm"
};

@id("llm-gateway-forbid-audit-read-outside-audit-authority")
forbid (
  principal,
  action == Action::"ReadAudit",
  resource
)
unless {
  principal in Role::"AuditReader"
};

@id("llm-gateway-forbid-audit-authority-non-audit-actions")
forbid (
  principal in Role::"AuditReader",
  action in [
    Action::"InvokeChatCompletion",
    Action::"InvokeEmbeddings",
    Action::"ListModels",
    Action::"ReadPoolStatus",
    Action::"RefreshKeyPool"
  ],
  resource
);

@id("llm-gateway-forbid-admin-realm-data-plane-actions")
forbid (
  principal in Role::"AdminRealm",
  action in [Action::"InvokeChatCompletion", Action::"InvokeEmbeddings", Action::"ListModels"],
  resource
);

// --- Cross-tenant explicit-deny for the data plane (brief §6 isolation unit) ---
// Any inference action is forbidden when the principal's tenant does not match the
// resource's tenant. There is no internal bypass: the per-tenant key pool is the
// isolation unit, and exception paths must be modeled as separate actions/resources.

@id("llm-gateway-forbid-cross-tenant-inference")
forbid (
  principal,
  action in [Action::"InvokeChatCompletion", Action::"InvokeEmbeddings"],
  resource
)
unless {
  principal.tenant_id == resource.tenant_id
};

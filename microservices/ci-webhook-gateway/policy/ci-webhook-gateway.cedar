// ci-webhook-gateway.cedar — D6 per-tenant webhook authorization policy
// ADR-0387 § D6 + ADR-0183 (forbid-wins Cedar semantics)
//
// Stage-4 RED skeleton. Stage-5 GREEN wires the Cedar adapter crate.
//
// Authorization model:
//   Principal: WebhookSource { tenant: TenantId, source_ip: Ip }
//   Action:    WebhookAction::TriggerCiJob
//   Resource:  Repository { owner: String, name: String }
//
// Dogfood doctrine: the oyatie-dogfood tenant is a regular tenant and goes
// through the same policy path as every external tenant. No internal bypass.
//
// Invariants:
//   1. Forbid-wins: explicit forbid rules always win over permit rules.
//   2. Only sources in the authorized_sources set for a tenant may trigger jobs.
//   3. A source may only trigger jobs for repositories belonging to its tenant.

// ---------------------------------------------------------------------------
// Permit: allow an authorized webhook source to trigger a CI job
// ---------------------------------------------------------------------------

permit (
    principal is WebhookSource,
    action == WebhookAction::"TriggerCiJob",
    resource is Repository
)
when {
    // The triggering source must be in the tenant's authorized_sources list.
    principal in principal.tenant.authorized_sources &&
    // The repository must belong to the same tenant (no cross-tenant triggers).
    resource.owner == principal.tenant.id
};

// ---------------------------------------------------------------------------
// Forbid: explicit cross-tenant trigger attempt — forbid-wins
// ---------------------------------------------------------------------------

forbid (
    principal is WebhookSource,
    action == WebhookAction::"TriggerCiJob",
    resource is Repository
)
when {
    // Block any attempt to trigger a job for a repository owned by a different tenant.
    resource.owner != principal.tenant.id
};

// ---------------------------------------------------------------------------
// Forbid: unauthenticated / unregistered source — fail-closed
// ---------------------------------------------------------------------------

forbid (
    principal is WebhookSource,
    action == WebhookAction::"TriggerCiJob",
    resource is Repository
)
unless {
    // Unless the source is explicitly registered, deny by default.
    principal in principal.tenant.authorized_sources
};

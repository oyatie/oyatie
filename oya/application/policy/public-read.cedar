// application µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access to public-class shell assets
//          (Leptos WASM bundle, CSS, fonts, login page, status page).
// Owner:   ops-security + gtm-marketing
// References: threat-model.md (Anonymous browser actor); route-isolation.md (global-public category)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — anonymous fetches public-class shell asset (WASM, CSS, fonts)
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"fetch_asset",
  resource in Asset::?a
)
when {
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has asset_kind &&
  resource.asset_kind in ["wasm", "css", "font", "image-static", "manifest-public"]
};

// =============================================================================
// PERMIT 2 — anonymous renders login page (global-public)
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"render_route",
  resource in Route::?r
)
when {
  resource has tenant_scope &&
  resource.tenant_scope == "global-public" &&
  resource has data_class &&
  resource.data_class == "PUBLIC"
};

// =============================================================================
// PERMIT 3 — anonymous reads status-page health summary
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_status",
  resource in StatusSummary::?s
)
when {
  resource has data_class &&
  resource.data_class == "PUBLIC"
};

// =============================================================================
// FORBID — anonymous never reads anything not flagged PUBLIC
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
unless {
  resource has data_class &&
  resource.data_class == "PUBLIC"
};

// =============================================================================
// FORBID — anonymous never executes mutating actions
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"write_session",
    Action::"publish_module_manifest",
    Action::"purge_cdn",
    Action::"export_audit",
    Action::"render_admin_route"
  ],
  resource
);

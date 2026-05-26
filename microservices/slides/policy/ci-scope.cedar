// Cedar v4.2 LTS — CI scope policy for slides µservice
// CI principals (GitHub Actions runners + governance-pipeline reviewers + admission-gate validators)
// have scoped read access. Default-deny baseline preserved.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (principal, action, resource);

// CI principals may read public catalog metadata (catalog/, contracts/, slos/)
permit (
    principal is CiPrincipal,
    action in [
        Action::"catalog:read",
        Action::"contract:read",
        Action::"slo:read",
        Action::"dashboard:read",
        Action::"runbook:read",
        Action::"adr:read"
    ],
    resource is CiArtifact
)
when {
    principal.scope == "slides" &&
    principal.bears_oidc_action_token == true &&
    resource.kind in ["catalog", "contract", "slo", "dashboard", "runbook", "adr"]
};

// CI gate validators may read µservice layout to enforce per-microservice-layout
permit (
    principal is CiPrincipal,
    action == Action::"gate:validate",
    resource is CiArtifact
)
when {
    principal.scope == "slides" &&
    principal.gate_name in [
        "lean-a1",
        "lean-a2",
        "port-location",
        "layer-correctness",
        "per-microservice-layout",
        "statelessness",
        "shardability",
        "slides-pptx-roundtrip-subset",
        "cedar-preview-required",
        "wasm-bundle-sri",
        "reduced-motion-fallback-mandatory",
        "ai-act-risk-class-stamp",
        "authority-cohesion"
    ]
};

// CI principals may NEVER read tenant deck content
forbid (
    principal is CiPrincipal,
    action in [Action::"deck:read", Action::"slide:read", Action::"comment:read", Action::"version:read"],
    resource
);

// CI principals may NEVER invoke AI capabilities
forbid (
    principal is CiPrincipal,
    action in [Action::"ai_design:invoke", Action::"ai_content_generation:invoke"],
    resource
);

// CI principals may NEVER start broadcasts
forbid (
    principal is CiPrincipal,
    action in [Action::"broadcast:start", Action::"broadcast:join"],
    resource
);

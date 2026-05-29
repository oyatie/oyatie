// Cedar capability fragment per ADR-0183.
// Auditor read-only access scoped per engagement.

permit (
  principal in Auditor::"engagement-id-pattern",
  action == Action::"read-artifact",
  resource is EvidenceArtifact
) when {
  resource.tenant_id in principal.engagement_tenants &&
  resource.emitted_unix_ms >= principal.engagement_window_open_unix_ms &&
  resource.emitted_unix_ms <= principal.engagement_window_close_unix_ms
};

// Read-only — no write actions permitted.
forbid (
  principal in Auditor::"engagement-id-pattern",
  action in [
    Action::"emit-artifact",
    Action::"manual-upload",
    Action::"rectify-data"
  ],
  resource is EvidenceArtifact
);

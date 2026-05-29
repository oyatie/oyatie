// Cedar capability fragment per ADR-0183.
// Compliance admin can upload manual evidence (pen-test reports, BAA inventory).

permit (
  principal in Role::"compliance-admin",
  action == Action::"manual-upload",
  resource is EvidenceArtifact
) when {
  resource.tenant_id == principal.tenant_id &&
  resource.kind in [
    "pen-test-report",
    "baa-inventory-entry",
    "vuln-scan-report"
  ]
};

// Compliance admin cannot read PHI access logs in cleartext — only audit metadata.
forbid (
  principal in Role::"compliance-admin",
  action == Action::"read-phi-payload",
  resource is MinimumNecessaryAccessLog
);

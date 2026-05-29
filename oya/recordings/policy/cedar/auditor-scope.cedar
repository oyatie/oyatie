// recordings µservice — auditor-scope policy fragment
// Cedar v4.2
//
// Purpose: read-only access for ops-compliance auditors + external regulators.

forbid (
  principal,
  action,
  resource
);

// Auditor read-only over a tenant's recordings + transcripts + audit-chain.
permit (
  principal in AuditorPrincipal::?a,
  action in [
    Action::"list_recordings",
    Action::"read_recording",
    Action::"read_transcript",
    Action::"read_chapter_markers",
    Action::"read_summary",
    Action::"read_legal_hold_history",
    Action::"read_retention_history",
    Action::"read_audit_chain_segment"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has audited_tenants &&
  resource.tenant_id in principal.audited_tenants &&
  principal has roles &&
  "external_auditor" in principal.roles
};

// Auditors may NEVER write, delete, redact, hold, or export.
forbid (
  principal in AuditorPrincipal::?a,
  action in [
    Action::"ingest_recording",
    Action::"update_recording_metadata",
    Action::"add_redaction_overlay",
    Action::"compensate_redaction",
    Action::"engage_legal_hold",
    Action::"release_legal_hold",
    Action::"export_for_ediscovery",
    Action::"delete_recording",
    Action::"purge_retention",
    Action::"kms_shred"
  ],
  resource
);

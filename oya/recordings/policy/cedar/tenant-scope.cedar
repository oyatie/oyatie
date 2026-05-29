// recordings µservice — tenant-scope policy fragment
// Cedar v4.2
//
// Purpose: enforce tenant boundary at every read/write of a Recording,
//          Transcript, Redaction, LegalHold, ShareLink, Export, ChapterMarker,
//          or Summary resource.
// Owner:   axis-recordings + ops-security + council-privacy

// =============================================================================
// Default deny (defence-in-depth FORBID)
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant member reads recordings in their tenant
// =============================================================================

permit (
  principal,
  action in [
    Action::"list_recordings",
    Action::"read_recording",
    Action::"read_transcript",
    Action::"read_chapter_markers",
    Action::"read_summary",
    Action::"read_thumbnail_pack",
    Action::"start_playback_session",
    Action::"list_share_links_for_resource"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  // Cell-isolation: legal-hold cells are isolated; principal must be in the same cell
  resource has cell_id &&
  principal has cell_id &&
  resource.cell_id == principal.cell_id
};

// =============================================================================
// PERMIT 2 — recording authors can update title + description within window
// =============================================================================

permit (
  principal,
  action in [Action::"update_recording_metadata"],
  resource
)
when {
  resource has author_ref &&
  principal has user_ref &&
  resource.author_ref == principal.user_ref &&
  resource has ingested_at &&
  context has now &&
  // 30-day edit window for metadata
  (context.now - resource.ingested_at) < 2592000
};

// =============================================================================
// PERMIT 3 — tenant compliance officers can engage / release legal hold
// =============================================================================
// Note: legal-hold engagement requires four-eyes — Cedar PERMIT here is
// necessary but not sufficient; the legal-hold-kernel additionally requires
// a paired-approver claim.

permit (
  principal,
  action in [
    Action::"engage_legal_hold",
    Action::"release_legal_hold",
    Action::"export_for_ediscovery",
    Action::"read_legal_hold_history"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  principal has roles &&
  "compliance_officer" in principal.roles
};

// =============================================================================
// PERMIT 4 — share-link creation by recording author OR tenant-admin
// =============================================================================

permit (
  principal,
  action in [
    Action::"create_share_link",
    Action::"revoke_share_link",
    Action::"list_my_share_links"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  (
    (
      resource has author_ref &&
      principal has user_ref &&
      resource.author_ref == principal.user_ref
    ) ||
    (
      principal has roles &&
      "tenant_admin" in principal.roles
    )
  )
};

// =============================================================================
// PERMIT 5 — export by tenant member; ediscovery export gated to compliance
// =============================================================================

permit (
  principal,
  action in [
    Action::"export_mp4",
    Action::"export_mp3",
    Action::"export_wav",
    Action::"export_transcript_vtt",
    Action::"export_transcript_srt",
    Action::"export_transcript_pdf",
    Action::"export_transcript_docx"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id == principal.tenant_id
};

// =============================================================================
// PERMIT 6 — redaction by compliance officer
// =============================================================================

permit (
  principal,
  action in [
    Action::"add_redaction_overlay",
    Action::"list_redactions",
    Action::"compensate_redaction"  // un-redact via new compensating overlay row
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  principal has roles &&
  "compliance_officer" in principal.roles
};

// =============================================================================
// PERMIT 7 — ingest from producer µservices via SPIFFE identity
// =============================================================================

permit (
  principal in ProducerSpiffeIdentity::?p,
  action == Action::"ingest_recording",
  resource
)
when {
  // SPIFFE identity must be on the producer-allowlist
  principal has spiffe_id &&
  (
    principal.spiffe_id == "spiffe://oyatie/meet/recorder" ||
    principal.spiffe_id == "spiffe://oyatie/messenger/huddle-recorder" ||
    principal.spiffe_id == "spiffe://oyatie/live-broadcast/recorder" ||
    principal.spiffe_id == "spiffe://oyatie/workflow-engine/screen-capture"
  ) &&
  // Recording-consent banner must be confirmed at producer side
  resource has consent_banner_confirmed &&
  resource.consent_banner_confirmed == true
};

// =============================================================================
// PERMIT 8 — manual upload via per-tenant rate-limited presigned URL
// =============================================================================

permit (
  principal,
  action in [Action::"create_manual_upload_presign", Action::"finalize_manual_upload"],
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  // Quotas enforced separately by the rate-limiter; Cedar is the auth gate.
  principal has roles &&
  ("tenant_member" in principal.roles || "tenant_admin" in principal.roles)
};

// =============================================================================
// FORBID — cross-tenant attempts
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id != principal.tenant_id
};

// =============================================================================
// FORBID — cross-pack attempts
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has pack_id &&
  principal has pack_id &&
  resource.pack_id != principal.pack_id
};

// =============================================================================
// FORBID — destructive operations on legal-hold-engaged recordings
// =============================================================================

forbid (
  principal,
  action in [
    Action::"delete_recording",
    Action::"purge_retention",
    Action::"kms_shred"
  ],
  resource
)
when {
  resource has legal_hold_engaged &&
  resource.legal_hold_engaged == true
};

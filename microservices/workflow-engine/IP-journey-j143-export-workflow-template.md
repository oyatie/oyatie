---
doc_class: Implementation-Plan-Journey-Slice
journey_id: j143
microservice: workflow-engine
status: draft
date: 2026-05-20
authority_tier: 3
intern_buildable: true
adr_anchors: [ADR-0145, ADR-0247, ADR-0251, ADR-0311]
---

# workflow-engine — IP slice for j143 (export workflow template)

## Scope

Deliver the 18-step `work_drive_export_with_dlp_scrub_v2` template.

## Template YAML skeleton

```yaml
template_id: work_drive_export_with_dlp_scrub_v2
version: 2.0.0
overlay_class: drive_export
required_inputs:
  - subject_principal     # = self (the demoted employee)
  - selected_categories   # subset of {portfolio_safe, reference_letter, non_confidential_work_sample}
steps:
  - id: validate_export_eligibility
    µservice: identity + drive
    rpc: identity.v1.Sessions.IsInReadOnlyDemotionWindow
    on_fail: terminate { reason: "export window expired or principal not demoted" }
  - id: enumerate_exportable_files
    µservice: drive
    rpc: Files.EnumerateExportable
  - id: dlp_scrub_pass_1
    µservice: compliance
    rpc: DLP.ScrubBatch (pass=1)
  - id: dlp_scrub_pass_2
    µservice: compliance
    rpc: DLP.ScrubBatch (pass=2)
  - id: emit_dlp_report
    µservice: compliance
    rpc: DLP.EmitReport
  - id: request_hr_attestation
    µservice: workflow-engine
    rpc: Task.Assign (assignee = HR-admin)
    blocks_on_human: true
  - id: wait_for_hr_attestation
    blocks_on_event: ScrubAttestationSigned
    timeout: 5d
    on_timeout: escalate_to_backup_attestor
  - id: bundle_archive
    µservice: drive
    rpc: Files.BundleArchive
    inputs: { files: scrubbed_list, attestation_receipt: attestation_envelope }
  - id: cross_tenant_transfer_init
    µservice: drive
    rpc: Files.TransferCrossTenant
    is_cross_tenant: true
    cross_tenant_purpose: post_employment_portfolio_export
  - id: chunk_upload_loop
    µservice: drive
    rpc: Files.UploadChunk
    parallel: true
    chunk_size: 256MB
  - id: verify_checksum
    µservice: drive
    rpc: Files.VerifyTransferChecksum
  - id: seal_audit_chain_source
    µservice: audit-chain
    rpc: Seal.EmitCrossTenant
  - id: seal_audit_chain_dest
    µservice: audit-chain (personal-tenant)
    rpc: Seal.EmitCrossTenant
  - id: notify_chris_personal_mail
    µservice: mail (personal-tenant cross-tenant emit)
    rpc: OutboundMail.Send (cross-tenant)
  - id: emit_ops_dashboard
    µservice: ops-dashboard-control-center
    rpc: ExportTracking.Update
  - id: schedule_t30d_cleanup
    µservice: workflow-engine
    rpc: Checkpoint.Schedule
  - id: emit_attestation_trail_link
    µservice: audit-chain (personal-tenant)
    rpc: AttestationTrail.Link
  - id: workflow_close
    µservice: workflow-engine
    rpc: Workflow.Close { status: completed_clean }
```

## State machine semantics

- Steps 1-5: in-tenant (work-tenant).
- Step 6-7: human-in-loop (Karim).
- Steps 8-14: cross-tenant transfer + double-seal.
- Steps 15-18: notification + close.

## Cedar permits

| Permit | Granted to | Purpose |
|---|---|---|
| `b2b.workflow.export.start` | demoted principal (self) | Start the export |
| `b2b.workflow.export.attest` | HR-admin | Sign attestation step |
| `b2b.workflow.export.escalate_attestor` | HR-admin alternate | Backup if primary AFK |

## Acceptance criteria

- [ ] Template loads + validates schema.
- [ ] All 18 steps emit audit-trace under single `audit_trace_id`.
- [ ] Step 7 timeout escalation works (B.D.1 chaos test).
- [ ] Resume from chunk checkpoint works (B.10 + D.2 chaos).

## Out of scope

- The DLP scrub itself (compliance IP).
- The drive transfer (drive IP).
- HR-admin shell UI for attestation (HR-tools out of scope for v1 backend).

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

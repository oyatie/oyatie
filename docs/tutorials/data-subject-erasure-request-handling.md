---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-DSR-ERASE-005
persona: "Marta Novak, privacy operations analyst for Acme Robotics EU"
prerequisite_packs:
  - canonical-base
  - eu-gdpr
  - privacy-operations
  - audit-evidence
related_oyatie_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0276
  - ADR-0311
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "100 minutes"
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Handle a GDPR Art. 17 Erasure Request End-to-End

## Goal

You will intake and verify a GDPR Article 17 erasure request for `chris.volkov@example.eu`, classify what can be erased versus retained, run the microservice cascade, notify the requester, and prove that personal data was erased without deleting lawful audit, tax, security, or work-tenant retention records.

## Prerequisites

- Privacy analyst account: `marta.novak@acme.example`.
- Requester account: `chris.volkov@example.eu`.
- Requester personal tenant: `b2c-chris-volkov`.
- Related work tenant: `tenant-acme-robotics`.
- DSR case id: `dsr-erasure-2026-05-20-chris-volkov`.
- Region: `EU - Frankfurt cell`.
- Request type: `GDPR Art. 17 erasure`.
- Legal basis pack: `EU-GDPR-2018-baseline`.
- Subscribed microservices: `identity`, `tenancy`, `consent-graph`, `messenger`, `mail`, `drive`, `marketplace`, `payments`, `ontology`, `workflow-engine`, `policy-engine`, `audit-chain`.
- Required Cedar permit: `privacy.dsr.case.create`.
- Required Cedar permit: `privacy.dsr.identity.verify`.
- Required Cedar permit: `privacy.dsr.erasure.plan`.
- Required Cedar permit: `privacy.dsr.erasure.execute`.
- Required Cedar permit: `privacy.dsr.exception.record`.
- Required Cedar permit: `audit.privacy.read`.
- Required Cedar permit: `mail.notice.send`.
- Required Cedar permit: `workflow.run.start`.
- Named saved query: `tutorial.gdpr_erasure_case_status`.
- SLA target: `P30D`.
- Training fixture: `Chris Volkov personal account`.
- Do not use this tutorial on a live requester without legal authorization.

## Step-by-Step

1. Open the DSR intake queue.
   - Sign in as `marta.novak@acme.example`.
   - Switch to `Acme Robotics EU Privacy`.
   - Open `Privacy -> Data Subject Requests`.
   - Confirm tenant context: `tenant-acme-robotics`.
   - Confirm pack badge: `EU-GDPR-2018-baseline`.
   - Click `New request`.
   - Request type: `Erasure - GDPR Art. 17`.
   - Requester email: `chris.volkov@example.eu`.
   - Case id: `dsr-erasure-2026-05-20-chris-volkov`.
   - Screenshot checkpoint: capture the DSR case draft.

2. Record request details.
   - Source channel: `Privacy portal`.
   - Received at: `2026-05-20T14:00:00Z`.
   - Requester statement: `Please delete my personal Oyatie account and associated personal data.`
   - Requested scope: `Personal tenant only`.
   - Related work tenant disclosure: `Former Acme Robotics employee`.
   - Preferred response language: `English`.
   - Response address: `chris.volkov@example.eu`.
   - Click `Save intake`.
   - Expected toast: `DSR intake saved`.
   - Screenshot checkpoint: capture the saved case header.

3. Verify requester identity.
   - Click `Verify identity`.
   - Method: `Passkey challenge`.
   - Challenge target: `chris.volkov@example.eu`.
   - Expiration: `PT30M`.
   - Click `Send challenge`.
   - Chris completes the passkey prompt from the privacy portal.
   - Expected status: `Identity verified`.
   - Evidence id should begin `identity-proof-dsr-erasure`.
   - Screenshot checkpoint: capture the verified identity badge.
   - Do not accept email-only proof for this training case.

4. Confirm tenant and boundary scope.
   - Open the `Scope` tab.
   - Personal tenant should resolve to `b2c-chris-volkov`.
   - Work tenant relationship should show `tenant-acme-robotics - former employee`.
   - Personal tenant scope: `erasable subject-owned data`.
   - Work tenant scope: `retained according to employer records policy`.
   - Boundary rule: `ADR-0311 personal/work separation`.
   - Click `Lock scope`.
   - Expected toast: `Scope locked`.
   - Screenshot checkpoint: capture personal versus work scope rows.
   - This prevents accidental deletion of work-owned records.

5. Run discovery across microservices.
   - Click `Discover data`.
   - Select microservices: `identity`, `tenancy`, `consent-graph`, `messenger`, `mail`, `drive`, `marketplace`, `payments`, `ontology`.
   - Include `workflow-engine` for active automations.
   - Include `audit-chain` for retention exception evidence.
   - Click `Start discovery`.
   - Expected state: `Discovery running`.
   - Wait until all services show `Complete`.
   - Screenshot checkpoint: capture the discovery matrix.
   - Export discovery as `dsr-erasure-chris-discovery.json`.
   - Do not execute erasure from the discovery screen.

6. Review discovered personal data.
   - Identity row: `profile, passkeys, sessions`.
   - Tenancy row: `personal tenant membership`.
   - Consent row: `marketing, analytics, marketplace consent`.
   - Messenger row: `personal DMs and metadata`.
   - Mail row: `personal inbox metadata`.
   - Drive row: `personal files and folders`.
   - Marketplace row: `personal listings and purchases`.
   - Payments row: `tokenized payment references`.
   - Ontology row: `personal graph projections`.
   - Screenshot checkpoint: capture all rows before planning.

7. Review lawful retention exceptions.
   - Open `Exceptions`.
   - Work tenant records: `retain under employer legitimate interest`.
   - Marketplace tax invoices: `retain tax evidence for required period`.
   - Payment chargeback evidence: `retain tokenized dispute records`.
   - Audit-chain events: `retain immutable audit record with subject redaction`.
   - Security events: `retain abuse and fraud evidence if present`.
   - Active legal hold: expected `none` for this training case.
   - Click `Approve exception set`.
   - Expected toast: `Retention exceptions approved`.
   - Screenshot checkpoint: capture exception reasons.
   - Every retained row needs a reason, not a vague note.

8. Build the erasure plan.
   - Click `Build plan`.
   - Plan name: `erase-b2c-chris-volkov-v1`.
   - Plan mode: `Subject erasure with retained-evidence redaction`.
   - Personal tenant action: `Deactivate then purge`.
   - Identity action: `Revoke sessions, delete passkeys after export window`.
   - Consent action: `Withdraw all active consent`.
   - Drive action: `Delete personal files`.
   - Messenger action: `Delete personal content, retain required delivery audit`.
   - Mail action: `Delete personal mailbox content`.
   - Marketplace action: `Anonymize profile, retain tax invoices`.
   - Screenshot checkpoint: capture the generated plan.

9. Notify the requester before execution.
   - Click `Generate notice`.
   - Template: `GDPR Art. 17 erasure pre-execution notice`.
   - Recipient: `chris.volkov@example.eu`.
   - Include scope summary: enabled.
   - Include retained exception list: enabled.
   - Include expected completion time: `within 30 days`.
   - Click `Send notice`.
   - Expected event: `DsrRequesterNoticeSent`.
   - Screenshot checkpoint: capture the sent notice status.
   - The notice should not expose work tenant internal evidence.
   - Return to the plan.

10. Run dry-run impact analysis.
    - Click `Dry run`.
    - Confirm `No mutation` is enabled.
    - Click `Start dry run`.
    - Expected output: `erasable_objects_count` greater than `0`.
    - Expected output: `work_tenant_mutations` equals `0`.
    - Expected output: `blocked_by_legal_hold` equals `0`.
    - Expected output: `retention_exceptions_count` greater than `0`.
    - Screenshot checkpoint: capture the dry-run summary.
    - If any work tenant mutation appears, stop and repair scope.
    - Export dry run as `dsr-erasure-chris-dry-run.json`.

11. Execute erasure.
    - Click `Execute erasure`.
    - Confirmation phrase: `ERASE b2c-chris-volkov`.
    - Execution window: `Immediate`.
    - Enable `emit audit-chain evidence`.
    - Enable `send completion notice on success`.
    - Click `Execute`.
    - Expected state: `Erasure cascade running`.
    - Screenshot checkpoint: capture the cascade progress view.
    - Keep the browser open until all service rows settle.
    - Do not manually delete rows outside the cascade.

12. Inspect service-level cascade results.
    - Identity should show `sessions revoked`.
    - Tenancy should show `personal tenant deactivated`.
    - Consent graph should show `all active consent withdrawn`.
    - Messenger should show `personal content erased`.
    - Mail should show `personal mailbox erased`.
    - Drive should show `personal files erased`.
    - Marketplace should show `profile anonymized`.
    - Payments should show `payment token detached`.
    - Ontology should show `personal projections removed`.
    - Screenshot checkpoint: capture the green service matrix.

13. Review retained evidence.
    - Open `Retained evidence`.
    - Confirm work tenant row is `retained`.
    - Confirm tax invoice row is `retained`.
    - Confirm audit-chain rows are `retained with subject redaction`.
    - Confirm security event row is `none`.
    - Confirm legal hold row is `none`.
    - Click `Export retained-evidence register`.
    - Save as `dsr-erasure-chris-retained-evidence.pdf`.
    - Screenshot checkpoint: capture retained evidence reasons.
    - Retained evidence is not a failure when each row has lawful basis.

14. Send completion notice.
    - Open `Notices`.
    - Confirm pre-execution notice state: `sent`.
    - Click `Send completion notice`.
    - Template: `GDPR Art. 17 erasure completion`.
    - Include case id: enabled.
    - Include completed services: enabled.
    - Include retained exception categories: enabled.
    - Click `Send`.
    - Expected event: `DsrCompletionNoticeSent`.
    - Screenshot checkpoint: capture notice state `sent`.
    - Close the notice drawer.

15. Confirm requester access is closed.
    - Attempt sign-in as `chris.volkov@example.eu` in a training browser.
    - Expected result: `Account closed by privacy request`.
    - Click `Download response letter`.
    - Expected file name: `dsr-erasure-2026-05-20-chris-volkov-response.pdf`.
    - Attempt to open personal Drive.
    - Expected result: no active tenant session.
    - Screenshot checkpoint: capture the closed account state.
    - Do not attempt to restore access.
    - Return to Marta's privacy console.
    - The account closure is expected for a completed erasure.

16. Run final verification query.
    - Open `Privacy -> Saved checks`.
    - Choose `tutorial.gdpr_erasure_case_status`.
    - Input `case_id=dsr-erasure-2026-05-20-chris-volkov`.
    - Input `subject_tenant=b2c-chris-volkov`.
    - Input `work_tenant=tenant-acme-robotics`.
    - Click `Run`.
    - Expected title: `GDPR erasure complete`.
    - Expected state: `PASS`.
    - Screenshot checkpoint: capture query output.
    - Save the query result to the case file.
    - Mark the case `Closed - completed`.

## Verification

- Named query: `tutorial.gdpr_erasure_case_status`.
- Query location: `Privacy -> Saved checks -> Tutorial Checks`.
- Query input `case_id`: `dsr-erasure-2026-05-20-chris-volkov`.
- Query input `subject_email`: `chris.volkov@example.eu`.
- Query input `subject_tenant`: `b2c-chris-volkov`.
- Query input `work_tenant`: `tenant-acme-robotics`.
- Expected output field: `identity_verified`.
- Expected output value: `true`.
- Expected output field: `scope_locked`.
- Expected output value: `personal_tenant_only`.
- Expected output field: `erasure_plan`.
- Expected output value: `erase-b2c-chris-volkov-v1`.
- Expected output field: `work_tenant_mutations`.
- Expected output value: `0`.
- Expected output field: `services_completed_count`.
- Expected output value: `9`.
- Expected output field: `active_consent_remaining`.
- Expected output value: `0`.
- Expected output field: `personal_drive_objects_remaining`.
- Expected output value: `0`.
- Expected output field: `personal_mail_objects_remaining`.
- Expected output value: `0`.
- Expected output field: `personal_messenger_objects_remaining`.
- Expected output value: `0`.
- Expected output field: `retention_exceptions_have_basis`.
- Expected output value: `true`.
- Expected output field: `completion_notice_sent`.
- Expected output value: `true`.
- Expected output field: `result_label`.
- Expected output value: `GDPR erasure complete`.
- CLI equivalent:

```bash
oya privacy verify erasure \
  --case dsr-erasure-2026-05-20-chris-volkov \
  --subject-tenant b2c-chris-volkov \
  --work-tenant tenant-acme-robotics
```

- CLI expected line: `PASS tutorial.gdpr_erasure_case_status`.
- CLI expected line: `work_tenant_mutations=0`.
- CLI expected line: `active_consent_remaining=0`.
- CLI expected line: `completion_notice_sent=true`.
- Audit event to inspect: `DsrCaseCreated`.
- Audit event to inspect: `DsrIdentityVerified`.
- Audit event to inspect: `DsrErasurePlanApproved`.
- Audit event to inspect: `DsrErasureCascadeStarted`.
- Audit event to inspect: `DsrServiceErasureCompleted`.
- Audit event to inspect: `DsrCompletionNoticeSent`.
- Evidence artifact: `dsr-erasure-chris-discovery.json`.
- Evidence artifact: `dsr-erasure-chris-dry-run.json`.
- Evidence artifact: `dsr-erasure-chris-retained-evidence.pdf`.
- Evidence artifact: `dsr-erasure-2026-05-20-chris-volkov-response.pdf`.

## Common Pitfalls + Recovery

- Pitfall: the request scope says `all Acme data`.
- Recovery: clarify with the requester; work-owned records may require access/export handling, not deletion.
- Pitfall: identity proof is email-only.
- Recovery: require the passkey challenge or the approved step-up identity method.
- Pitfall: discovery excludes `payments`.
- Recovery: rerun discovery; payment tokens and tax evidence must be classified.
- Pitfall: discovery excludes `ontology`.
- Recovery: rerun discovery; projections can retain derived personal fields.
- Pitfall: the dry run shows `work_tenant_mutations=1`.
- Recovery: stop and adjust scope before execution.
- Pitfall: legal hold is active.
- Recovery: do not execute erasure for held objects; record the lawful exception and legal owner.
- Pitfall: audit-chain rows are proposed for deletion.
- Recovery: change action to `retain with subject redaction`; immutable audit evidence is not deleted.
- Pitfall: the requester asks to erase marketplace tax invoices.
- Recovery: retain invoices under lawful basis and explain the exception in the response notice.
- Pitfall: completion notice includes internal work tenant notes.
- Recovery: regenerate the notice from the external response template only.
- Pitfall: service matrix remains stuck on `running`.
- Recovery: open the service row, inspect retry state, and escalate only the failed microservice.
- Pitfall: Messenger says content erased but metadata remains unexplained.
- Recovery: classify metadata as retained evidence or rerun erasure for personal metadata.
- Pitfall: personal Drive count is not zero.
- Recovery: inspect retained files; delete unless they have explicit lawful hold.
- Pitfall: the response letter is not generated.
- Recovery: resend completion notice and confirm `mail.notice.send` permit.
- Pitfall: a user attempts to restore the account after closure.
- Recovery: deny restore; a new account requires a new tenant and cannot revive erased data.
- Pitfall: the analyst executes a manual SQL deletion.
- Recovery: stop; manual deletion bypasses audit evidence and must not be used.
- Pitfall: the DSR case is closed before completion notice.
- Recovery: reopen the case, send the notice, rerun verification, then close.
- Pitfall: retained evidence lacks basis.
- Recovery: add a specific basis such as tax, legal hold, security, or employer record retention.
- Pitfall: the SLA timer is missing.
- Recovery: set target `P30D` and link it to the case timeline.

## Service-by-Service Erasure Evidence

The erasure case should show a separate outcome for each affected service.

- Identity outcome should be `account_closed`.
- Profile outcome should be `personal_profile_erased`.
- Mail outcome should be `personal_mail_erased_or_retained_with_basis`.
- Messenger outcome should be `content_erased_metadata_classified`.
- Drive outcome should be `personal_files_erased`.
- Marketplace outcome should be `profile_erased_invoices_retained_with_basis`.
- Audit-chain outcome should be `subject_redacted_evidence_retained`.
- Backup outcome should be `erasure_tombstone_scheduled`.
- Search outcome should be `subject_index_removed`.
- Consent outcome should be `consent_records_closed`.
- Notification outcome should be `delivery_tokens_revoked`.
- Intelligence outcome should be `personal_ai_artifacts_erased`.

The case owner must record the basis for every retained object.

Use `tax_retention` for required marketplace invoice retention.

Use `security_audit` for immutable audit-chain evidence.

Use `employer_record_retention` for work-tenant HR records that are not erased by personal request.

Use `legal_hold` only when legal has attached a named hold id.

Never use `business convenience` as a lawful basis in this tutorial.

The completion notice should include erased categories and retained categories.

The completion notice should not expose internal service names to the requester.

The internal evidence packet may expose service names.

The external response should use plain language such as `messages`, `files`, and `marketplace receipts`.

If a service is still running, the case remains open.

If a service failed and was manually skipped, the case remains open.

If retained evidence lacks a basis, the case remains open.

The tutorial is complete when the case timeline shows `CompletionNoticeSent`.

## Next Tutorials

- [Capture and propagate consent across microservices](consent-cascade-across-microservices.md).
- [Activate multiple compliance packs on a tenant](multi-pack-tenant-activation.md).
- [First-day-on-Oyatie quickstart](quickstart-new-user-day-one.md).
- [Project Salesforce CRM data into Oyatie ontology](ontology-projection-from-external-source.md).

## References

- [Privacy Review Standard](../standards/privacy-review.md).
- [Compliance Evidence Automation Standard](../standards/compliance-evidence-automation.md).
- [Consent withdrawal cascade runbook](../runbooks/consent-withdrawal-cascade.md).
- [ADR-0311 Dual-Tenant Identity Personal vs Work Boundary](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md).
- [ADR-0276 Backup Portability GDPR Art. 20](../decisions/ADR-0276-backup-portability-gdpr-art-20.md).
- [GDPR compliance pack](../../registry/compliance-packs/GDPR.yaml).
- [Audit evidence timeline design spec](../../specs/design-system/audit-evidence-timeline.json).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [Doc Style](../standards/doc-style.md).

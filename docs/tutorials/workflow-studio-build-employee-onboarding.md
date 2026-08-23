---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-WF-ONBOARD-003
persona: "Mila Santos, HR operations manager at Acme Robotics"
prerequisite_packs:
  - canonical-base
  - workplace-core
  - hr-foundation
  - workflow-studio-builder
related_oyatie_adrs:
  - ADR-0035
  - ADR-0185
  - ADR-0204
  - ADR-0243
  - ADR-0311
  - ADR-0316
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "90 minutes"
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Build an Employee-Onboarding Workflow in Workflow Studio

## Goal

You will build and test a complete employee-onboarding workflow named `wf-acme-employee-onboarding-v1` that collects a new hire profile, issues a work-tenant invite, creates a Drive folder, sends Mail, opens a Messenger thread, schedules manager approval, and verifies the run without giving the employer access to the employee's personal tenant.

## Prerequisites

- Builder account: `mila.santos@acme.example`.
- New hire account: `aisha.khan@example.com`.
- Hiring manager: `samir.patel@acme.example`.
- Work tenant: `tenant-acme-robotics`.
- Workspace: `workspace-hr-operations`.
- Workflow id to create: `wf-acme-employee-onboarding-v1`.
- Test employee id: `emp-aisha-khan-2026`.
- Subscribed microservices: `workflow-studio`, `workflow-engine`, `hr`, `identity`, `tenancy`, `messenger`, `mail`, `drive`, `policy-engine`, `audit-chain`.
- Required Cedar permit: `workflow.template.create`.
- Required Cedar permit: `workflow.template.publish_preview`.
- Required Cedar permit: `workflow.run.start`.
- Required Cedar permit: `hr.employee.draft.create`.
- Required Cedar permit: `identity.work_invite.issue`.
- Required Cedar permit: `drive.work-folder.create`.
- Required Cedar permit: `mail.work-message.send`.
- Required Cedar permit: `messenger.work-thread.create`.
- Required Cedar permit: `audit.workflow.read`.
- Required Cedar permit: `capability_tier.workflow.start`.
- Active capability tier: `hr-core`.
- Active capability tier: `workflow-studio-builder`.
- Sample start date: `2026-06-01`.
- Sample department: `Manufacturing Engineering`.
- Sample work location: `Seoul Robotics Lab`.
- Test offer letter file: `offer-aisha-khan-redacted.pdf`.

## Step-by-Step

1. Open Workflow Studio in the HR workspace.
   - Sign in as `mila.santos@acme.example`.
   - Switch to tenant `Acme Robotics`.
   - Open workspace `HR Operations`.
   - Click `Workflow Studio`.
   - Confirm header text: `Tenant context: tenant-acme-robotics`.
   - Confirm builder badge: `Workflow Builder`.
   - Screenshot checkpoint: capture the blank canvas and tenant switcher.
   - If the builder opens in personal context, close it and relaunch from the work workspace.
   - The workflow will create work-tenant artifacts only.
   - Keep the side panel open for node configuration.

2. Start a new workflow from a blank canvas.
   - Click `New workflow`.
   - Choose `Blank workflow`.
   - Name: `Acme employee onboarding`.
   - Workflow id: `wf-acme-employee-onboarding-v1`.
   - Owner team: `HR Operations`.
   - Data class: `Confidential`.
   - Run mode: `Human-gated automation`.
   - Click `Create`.
   - Expected canvas title: `Acme employee onboarding`.
   - Screenshot checkpoint: capture the empty workflow metadata drawer.

3. Add the intake form trigger.
   - Drag `Form submitted` from the trigger palette.
   - Rename node to `New hire intake submitted`.
   - Form id: `form-new-hire-intake-v1`.
   - Required fields: `legal_name`, `personal_email`, `start_date`, `department`, `manager_email`, `work_location`.
   - Set `personal_email` value for test run: `aisha.khan@example.com`.
   - Set `start_date` value for test run: `2026-06-01`.
   - Set `department` value for test run: `Manufacturing Engineering`.
   - Set `manager_email` value for test run: `samir.patel@acme.example`.
   - Click `Save node`.
   - Screenshot checkpoint: capture the form field mapping.

4. Add validation for the personal/work boundary.
   - Drag `Policy check` after the form trigger.
   - Rename node to `Validate tenant boundary`.
   - Policy bundle: `policy/tenant-boundary-work-vs-personal.cedar`.
   - Principal source: `manager_email`.
   - Resource source: `new_hire_profile`.
   - Expected decision: `Permit work invite, deny personal tenant read`.
   - Failure route label: `Boundary failed`.
   - Success route label: `Boundary valid`.
   - Click `Save node`.
   - Screenshot checkpoint: capture the Cedar decision preview.

5. Create the HR draft record.
   - Drag `HR action` onto the `Boundary valid` route.
   - Action: `Create employee draft`.
   - Employee id: `emp-aisha-khan-2026`.
   - Legal name: map from `legal_name`.
   - Personal email: map from `personal_email`.
   - Department: map from `department`.
   - Manager: map from `manager_email`.
   - Work location: map from `work_location`.
   - Data class: `Confidential`.
   - Click `Save node`.
   - Expected node badge: `Cedar permit hr.employee.draft.create`.

6. Add manager approval.
   - Drag `Approval` after `Create employee draft`.
   - Rename node to `Manager approves onboarding`.
   - Approver: `samir.patel@acme.example`.
   - SLA: `P2D`.
   - Escalation: `hr-ops-leads@acme.example`.
   - Approval buttons: `Approve onboarding`, `Request correction`.
   - Rejection route: `Correction needed`.
   - Approval route: `Approved`.
   - Add instruction: `Confirm role, location, and start date before invite issuance.`
   - Click `Save node`.
   - Screenshot checkpoint: capture the approval node and SLA.

7. Add the work tenant invitation step.
   - Drag `Identity action` onto the `Approved` route.
   - Action: `Issue work tenant invitation`.
   - Tenant id: `tenant-acme-robotics`.
   - Recipient email: map from `personal_email`.
   - Role: `Employee`.
   - Invitation purpose: `Employee onboarding`.
   - Expiration: `P14D`.
   - Consent prompt: `Work activity is owned by Acme Robotics`.
   - Permit: `identity.work_invite.issue`.
   - Click `Save node`.
   - Screenshot checkpoint: capture the invitation node settings.

8. Add the Drive folder creation step.
   - Drag `Drive action` after the invitation node.
   - Action: `Create folder`.
   - Folder path: `/Employees/Aisha Khan/Onboarding`.
   - Owner tenant: `tenant-acme-robotics`.
   - Data class: `Confidential`.
   - Retention: `employee-record-7y`.
   - Initial file: `offer-aisha-khan-redacted.pdf`.
   - File label: `Offer letter - redacted`.
   - Click `Save node`.
   - Expected node badge: `drive.work-folder.create`.
   - Screenshot checkpoint: capture the Drive node.

9. Add the Mail welcome step.
   - Drag `Mail action` after Drive folder creation.
   - Action: `Send work mail`.
   - To: map from `personal_email`.
   - From: `hr-ops@acme.example`.
   - Subject: `Welcome to Acme Robotics`.
   - Body line 1: `Your work tenant invitation is ready.`
   - Body line 2: `Your onboarding folder is prepared.`
   - Body line 3: `Your manager is Samir Patel.`
   - Attach link: `/Employees/Aisha Khan/Onboarding`.
   - Click `Save node`.
   - Screenshot checkpoint: capture the mail template preview.

10. Add the Messenger introduction step.
    - Drag `Messenger action` after Mail.
    - Action: `Create work thread`.
    - Thread name: `Aisha onboarding`.
    - Members: `aisha.khan@example.com`, `samir.patel@acme.example`, `mila.santos@acme.example`.
    - First message: `Welcome Aisha. This thread is for your Acme onboarding.`
    - Retention: `employee-record-7y`.
    - Tenant owner: `tenant-acme-robotics`.
    - Click `Save node`.
    - Expected node badge: `messenger.work-thread.create`.
    - Screenshot checkpoint: capture the Messenger node member list.

11. Add the correction branch.
    - Click the `Correction needed` route from manager approval.
    - Drag `Mail action`.
    - Action: `Send correction request`.
    - To: `mila.santos@acme.example`.
    - Subject: `Onboarding correction needed for Aisha Khan`.
    - Body: `Manager requested a correction. Review the intake form and rerun approval.`
    - Add task link: `Open intake submission`.
    - Click `Save node`.
    - Add a `Stop run` node after the correction mail.
    - Stop reason: `Manager requested correction`.
    - Screenshot checkpoint: capture the branch split.

12. Add failure handling for tenant-boundary denial.
    - Click the `Boundary failed` route.
    - Drag `Stop run`.
    - Stop reason: `Tenant boundary policy denied onboarding`.
    - Add audit severity: `high`.
    - Add human message: `Do not proceed until HR admin reviews policy denial.`
    - Click `Save node`.
    - The workflow should never silently continue after a policy denial.
    - Screenshot checkpoint: capture the failure route.
    - This branch protects the personal tenant from employer overreach.
    - It is the critical safety route in this tutorial.

13. Validate the workflow.
    - Click `Validate`.
    - Expected validation item: `All nodes connected`.
    - Expected validation item: `Cedar permits resolved`.
    - Expected validation item: `Tenant boundary branch present`.
    - Expected validation item: `No personal tenant write actions`.
    - Expected validation item: `Audit events configured`.
    - If any item fails, click the item and repair the referenced node.
    - Screenshot checkpoint: capture the validation panel.
    - Click `Save draft`.
    - Expected toast: `Workflow draft saved`.

14. Publish to preview.
    - Click `Publish`.
    - Choose `Preview only`.
    - Version: `1.0.0-preview.1`.
    - Release note: `Initial HR onboarding workflow tutorial build`.
    - Required approver: `mila.santos@acme.example`.
    - Click `Publish preview`.
    - Expected toast: `Preview version published`.
    - Expected version badge: `1.0.0-preview.1`.
    - Screenshot checkpoint: capture the version badge.
    - Do not publish to general availability in this tutorial.

15. Run the preview with sample data.
    - Click `Run preview`.
    - Set `legal_name`: `Aisha Khan`.
    - Set `personal_email`: `aisha.khan@example.com`.
    - Set `start_date`: `2026-06-01`.
    - Set `department`: `Manufacturing Engineering`.
    - Set `manager_email`: `samir.patel@acme.example`.
    - Set `work_location`: `Seoul Robotics Lab`.
    - Click `Start preview run`.
    - Expected run id format: `run-wf-acme-onboarding-<timestamp>`.
    - Screenshot checkpoint: capture the started run timeline.

16. Complete the manager approval.
    - Sign in or impersonation-test as `samir.patel@acme.example`.
    - Open `Tasks -> Approvals`.
    - Select `Manager approves onboarding`.
    - Confirm employee: `Aisha Khan`.
    - Confirm start date: `2026-06-01`.
    - Confirm location: `Seoul Robotics Lab`.
    - Click `Approve onboarding`.
    - Expected toast: `Approval recorded`.
    - Return to Mila's run timeline.
    - Screenshot checkpoint: capture the approval completion.

17. Inspect generated artifacts.
    - In the run timeline, expand `Issue work tenant invitation`.
    - Confirm invite status: `Issued`.
    - Expand `Create folder`.
    - Confirm folder path: `/Employees/Aisha Khan/Onboarding`.
    - Expand `Send work mail`.
    - Confirm message state: `Sent`.
    - Expand `Create work thread`.
    - Confirm thread name: `Aisha onboarding`.
    - Screenshot checkpoint: capture the four successful node outputs.
    - Copy the run id for verification.

18. Run replay verification.
    - Click `Replay`.
    - Choose `Dry-run from audit events`.
    - Ensure `Do not resend messages` is enabled.
    - Click `Start replay`.
    - Expected replay state: `Matches original run`.
    - Expected mutation count: `0`.
    - Click `Export run summary`.
    - Save summary as `wf-acme-employee-onboarding-v1-preview-summary.pdf`.
    - Screenshot checkpoint: capture replay result.
    - This proves the workflow is observable and replayable without duplicate side effects.

## Verification

- Named query: `tutorial.workflow_employee_onboarding_status`.
- Query location: `Workflow Studio -> Runs -> Saved checks`.
- Query input `tenant_id`: `tenant-acme-robotics`.
- Query input `workflow_id`: `wf-acme-employee-onboarding-v1`.
- Query input `workflow_version`: `1.0.0-preview.1`.
- Query input `employee_id`: `emp-aisha-khan-2026`.
- Expected output field: `template_state`.
- Expected output value: `preview_published`.
- Expected output field: `boundary_policy_branch`.
- Expected output value: `present`.
- Expected output field: `preview_run_state`.
- Expected output value: `completed`.
- Expected output field: `approval_state`.
- Expected output value: `approved`.
- Expected output field: `work_invite_status`.
- Expected output value: `issued`.
- Expected output field: `drive_folder_created`.
- Expected output value: `/Employees/Aisha Khan/Onboarding`.
- Expected output field: `mail_sent`.
- Expected output value: `true`.
- Expected output field: `messenger_thread_created`.
- Expected output value: `Aisha onboarding`.
- Expected output field: `personal_tenant_mutations`.
- Expected output value: `0`.
- Expected output field: `dry_run_replay`.
- Expected output value: `matches_original`.
- Expected output field: `result_label`.
- Expected output value: `Employee onboarding workflow complete`.
- CLI equivalent:

```bash
oya workflow verify tutorial-onboarding \
  --tenant tenant-acme-robotics \
  --workflow wf-acme-employee-onboarding-v1 \
  --version 1.0.0-preview.1 \
  --employee emp-aisha-khan-2026
```

- CLI expected line: `PASS tutorial.workflow_employee_onboarding_status`.
- CLI expected line: `template_state=preview_published`.
- CLI expected line: `preview_run_state=completed`.
- CLI expected line: `personal_tenant_mutations=0`.
- Audit event to inspect: `WorkflowTemplatePreviewPublished`.
- Audit event to inspect: `WorkflowRunStarted`.
- Audit event to inspect: `WorkflowApprovalRecorded`.
- Audit event to inspect: `IdentityWorkInviteIssued`.
- Audit event to inspect: `DriveWorkFolderCreated`.
- Audit event to inspect: `MailWorkMessageSent`.
- Audit event to inspect: `MessengerWorkThreadCreated`.
- Evidence artifact: `wf-acme-employee-onboarding-v1-preview-summary.pdf`.
- Evidence folder: `/HR Operations/Workflow Evidence/Onboarding`.

## Common Pitfalls + Recovery

- Pitfall: the workflow is created under a personal workspace.
- Recovery: archive the draft and recreate it from `HR Operations` under `tenant-acme-robotics`.
- Pitfall: the form trigger omits `manager_email`.
- Recovery: add the field and rerun validation; manager approval cannot route without it.
- Pitfall: the Cedar policy node is skipped as "optional."
- Recovery: add `Validate tenant boundary`; this tutorial is not complete without it.
- Pitfall: the manager approval SLA is blank.
- Recovery: set `P2D` so stale onboarding runs escalate instead of waiting forever.
- Pitfall: the identity invite is sent before approval.
- Recovery: move the invite node after the approval success route and rerun preview.
- Pitfall: the Drive folder is owned by Mila.
- Recovery: set owner tenant to `tenant-acme-robotics`, not the builder account.
- Pitfall: the Mail step sends from Mila's personal address.
- Recovery: set From to `hr-ops@acme.example` and verify the work tenant badge.
- Pitfall: the Messenger thread includes the new hire before invite acceptance.
- Recovery: use the workflow's invite token member placeholder; do not add a personal tenant user directly.
- Pitfall: replay resends messages.
- Recovery: enable `Do not resend messages`; replay must be observational for verification.
- Pitfall: the correction branch loops back without audit.
- Recovery: stop the run after correction and require an edited form submission.
- Pitfall: validation says `No audit profile`.
- Recovery: open workflow settings and set audit profile `hr-onboarding-v1`.
- Pitfall: the capability tier `hr-core` is not active.
- Recovery: ask a tenant admin to grant `hr-core` before building HR nodes.
- Pitfall: the preview run hangs after approval.
- Recovery: inspect node output for missing `identity.work_invite.issue` permit.
- Pitfall: the folder path contains unescaped characters.
- Recovery: use `/Employees/Aisha Khan/Onboarding` exactly for this tutorial.
- Pitfall: the evidence export contains test secrets.
- Recovery: delete the export and rerun using only the redacted offer letter.
- Pitfall: the workflow is published to GA.
- Recovery: revert to preview by creating a new patch version and disabling GA grant.
- Pitfall: the personal tenant receives a Drive folder.
- Recovery: disable the workflow and open a privacy incident.
- Pitfall: the verification query returns `personal_tenant_mutations=1`.
- Recovery: inspect the offending node before any future onboarding runs.

## Workflow Run Evidence Notes

Keep the preview run as the canonical proof for this tutorial.

- Workflow definition id should be `wf-acme-employee-onboarding-v1`.
- Preview run id should begin with `run-preview-onboarding-aisha-`.
- New hire fixture should be `Aisha Khan`.
- Hiring manager fixture should be `Riley Chen`.
- HR sender should be `hr-ops@acme.example`.
- Drive folder should be `/Employees/Aisha Khan/Onboarding`.
- Messenger thread should be `aisha-onboarding-mentor`.
- Mail subject should be `Welcome to Acme Robotics`.
- Audit profile should be `hr-onboarding-v1`.
- Replay mode should be `observational`.
- Published state should be `preview`.
- GA state should be `disabled`.

If the preview evidence is reused in a team review, include node output for `Issue invite`, `Create folder`, `Send welcome mail`, and `Notify mentor`.

The reviewer should be able to trace each output back to the new hire form.

No reviewer should need to infer the tenant boundary from surrounding prose.

The tenant badge and run metadata must carry the work tenant id.

## Next Tutorials

- [First-day-on-Oyatie quickstart](quickstart-new-user-day-one.md).
- [Capture and propagate consent across microservices](consent-cascade-across-microservices.md).
- [Handle a GDPR erasure request](data-subject-erasure-request-handling.md).
- [Activate multiple compliance packs on a tenant](multi-pack-tenant-activation.md).

## References

- [Workflow Studio Guide](../site/src/studio/workflow-studio-guide.md).
- [Workflow Studio Canvas Library ADR](../decisions/ADR-0204-workflow-studio-canvas-library.md).
- [Workflow Studio Client Stack ADR](../decisions/ADR-0185-workflow-studio-client-stack.md).
- [Workflow Engine State Machine and DAG Hybrid ADR](../decisions/ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md).
- [Workflow Canvas Design System Spec](../../specs/design-system/workflow-canvas.json).
- [Workflow Node Config Panel Spec](../../specs/design-system/workflow-node-config-panel.json).
- [Workflow Substrate Engine Standard](../standards/workflow-substrate-engine.md).
- [B2B workflow engine approval cascade journey](../user-journeys/j36-b2b-workflow-engine-approval-cascade/README.md).
- [Tenant lifecycle workflow ADR](../decisions/ADR-0175-tenant-lifecycle-workflow.md).
- [Documentation Rigor](../standards/documentation-rigor.md).

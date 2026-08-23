---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-DAYONE-001
persona: "Nadia Park, first-day employee joining Acme Robotics"
prerequisite_packs:
  - canonical-base
  - workplace-core
  - messenger-mail-drive-starter
related_oyatie_adrs:
  - ADR-0244
  - ADR-0299
  - ADR-0311
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "55 minutes"
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# First-Day-on-Oyatie Quickstart

## Goal

You will create a new Oyatie account, join the `acme-robotics` work tenant, create your first workspace, send a work Messenger message, send a work Mail message, and upload a Drive file while keeping the active tenant context visible at every click.

## Prerequisites

- Account: `nadia.park@example.com`.
- Work tenant: `tenant-acme-robotics`.
- Personal tenant: `b2c-nadia-park`.
- Workspace slug to create: `workspace-acme-day-one`.
- Subscribed microservices: `identity`, `tenancy`, `messenger`, `mail`, `drive`, `policy-engine`, `audit-chain`.
- Required Cedar permit: `identity.passkey.register`.
- Required Cedar permit: `tenancy.workspace.create`.
- Required Cedar permit: `messenger.work-thread.create`.
- Required Cedar permit: `messenger.work-message.send`.
- Required Cedar permit: `mail.work-message.send`.
- Required Cedar permit: `drive.work-file.create`.
- Required Cedar permit: `audit.self.read`.
- Browser: Chrome, Edge, or Safari with WebAuthn passkey support.
- Test file: `day-one-notes.txt` containing `First Oyatie Drive upload for Acme Robotics`.
- Named admin contact: `it-admin@acme.example`.
- Named buddy contact: `mentor.lee@acme.example`.
- Device posture: laptop enrolled in `Acme Robotics MDM`.
- Compliance posture: default `canonical-base` pack only; no regulated data in this tutorial.

## Step-by-Step

1. Open the sign-up page.
   - Navigate to `https://app.oyatie.example/signup`.
   - Confirm the page title reads `Create your Oyatie account`.
   - Screenshot checkpoint: capture the hero with the `Continue with passkey` button.
   - Enter email `nadia.park@example.com`.
   - Select `Region: United States - us-east-1`.
   - Leave `Create a personal tenant for me` enabled.
   - Click `Continue with passkey`.
   - The passkey dialog should display the account label `Nadia Park - Oyatie`.
   - Complete the platform passkey prompt.
   - Expected banner: `Passkey registered`.
   - Do not choose a password fallback for this tutorial.

2. Confirm the personal tenant shell exists.
   - You land on `/home/personal`.
   - The tenant switcher should show `Personal - Nadia Park`.
   - The header should include `Tenant context: b2c-nadia-park`.
   - Screenshot checkpoint: capture the tenant switcher and the empty activity panel.
   - Open the user menu.
   - Select `Account settings`.
   - Confirm `Primary identity: nadia.park@example.com`.
   - Confirm `Personal tenant id: b2c-nadia-park`.
   - Close settings with `Esc`.
   - This check matters because ADR-0311 keeps personal and work data separate.

3. Accept the Acme work invitation.
   - Open the invitation email from `it-admin@acme.example`.
   - Click `Join Acme Robotics in Oyatie`.
   - Confirm the invitation page says `Join tenant-acme-robotics`.
   - Verify role preview shows `Employee`.
   - Verify workspace preview shows `Acme Robotics HQ`.
   - Click `Review tenant terms`.
   - The consent panel should show `Work activity is owned by Acme Robotics`.
   - Click `Accept work tenant membership`.
   - Expected banner: `You joined Acme Robotics`.
   - Screenshot checkpoint: capture the banner and work tenant switcher state.

4. Switch into the work tenant deliberately.
   - Open the tenant switcher.
   - Select `Acme Robotics`.
   - Confirm the header changes to `Tenant context: tenant-acme-robotics`.
   - Confirm the color strip changes from personal blue to work graphite.
   - Confirm the sidebar label reads `Work`.
   - If it still reads `Personal`, stop and refresh.
   - Screenshot checkpoint: capture the top-left tenant switcher.
   - The page route should be `/work/tenant-acme-robotics`.
   - This is the first visible ADR-0311 safety check in the user flow.
   - Every remaining action in this tutorial should happen in this work tenant.

5. Create the first workspace.
   - In the work sidebar, click `Workspaces`.
   - Click `New workspace`.
   - Name: `Acme Day One`.
   - Slug: `workspace-acme-day-one`.
   - Purpose: `First-day onboarding practice space`.
   - Visibility: `Private to tenant`.
   - Default data class: `Internal`.
   - Members: add `nadia.park@example.com` and `mentor.lee@acme.example`.
   - Click `Create workspace`.
   - Expected toast: `Workspace Acme Day One created`.
   - Screenshot checkpoint: capture the workspace overview card.

6. Pin the workspace.
   - From the workspace overview, click the star icon beside `Acme Day One`.
   - The star should fill and the tooltip should read `Pinned`.
   - Open the command palette with `Ctrl+K`.
   - Type `workspace`.
   - Confirm `Acme Day One` appears under `Pinned workspaces`.
   - Select it with Enter.
   - Confirm the URL includes `/workspaces/workspace-acme-day-one`.
   - This gives new users a stable return path.
   - The UI should not show personal tenant files in this workspace.
   - Screenshot checkpoint: capture the command palette result.

7. Create your first Messenger work thread.
   - In the workspace toolbar, click `Messenger`.
   - Click `New thread`.
   - Recipient: `mentor.lee@acme.example`.
   - Thread name: `Nadia day-one check-in`.
   - Thread owner: `Acme Day One`.
   - Retention: `Acme standard - 7 years`.
   - Data class: `Internal`.
   - Message: `Hi Mentor, I am set up in Oyatie and using the Acme Day One workspace.`
   - Click `Send`.
   - Expected state: single check mark followed by double check mark within 10 seconds.
   - Screenshot checkpoint: capture the sent message and `Work` context label.

8. Inspect Messenger audit visibility.
   - Click the message overflow menu.
   - Select `Message details`.
   - Confirm `Tenant owner: tenant-acme-robotics`.
   - Confirm `Audit stream: audit.connect.messenger.sent`.
   - Confirm `Retention policy: Acme standard - 7 years`.
   - Confirm `Personal tenant access: Denied`.
   - Close the details drawer.
   - This drawer is not for daily use, but it teaches the ownership boundary.
   - Screenshot checkpoint: capture the details drawer before closing.
   - Do not export the audit record in this quickstart.

9. Send a first work Mail message.
   - Click `Mail` in the workspace toolbar.
   - Click `Compose`.
   - To: `mentor.lee@acme.example`.
   - Subject: `Day one Oyatie workspace ready`.
   - Body line 1: `I created workspace-acme-day-one.`
   - Body line 2: `I sent the first Messenger check-in.`
   - Body line 3: `I will upload the day-one notes next.`
   - Data class: `Internal`.
   - Click `Send`.
   - Expected toast: `Message sent from Acme Robotics`.
   - Screenshot checkpoint: capture the sent confirmation.

10. Upload the first Drive file.
    - Click `Drive` in the workspace toolbar.
    - Click `Upload`.
    - Choose `day-one-notes.txt`.
    - Confirm detected data class `Internal`.
    - Confirm owner `tenant-acme-robotics`.
    - Confirm folder path `/Acme Day One/Onboarding`.
    - Click `Upload file`.
    - Expected toast: `day-one-notes.txt uploaded`.
    - Screenshot checkpoint: capture the Drive file row with owner and class.
    - Do not drag a personal file into the work workspace.

11. Share the Drive file with your mentor.
    - Select `day-one-notes.txt`.
    - Click `Share`.
    - Add `mentor.lee@acme.example`.
    - Permission: `Can comment`.
    - Expiration: `No expiration`.
    - Message: `Please confirm you can see my first upload.`
    - Click `Share`.
    - Expected row: `mentor.lee@acme.example - Can comment`.
    - Screenshot checkpoint: capture the share drawer.
    - Confirm no external domains are present in the share list.

12. Use the workspace activity feed.
    - Click `Activity`.
    - Confirm the feed contains `Workspace created`.
    - Confirm the feed contains `Messenger thread created`.
    - Confirm the feed contains `Mail message sent`.
    - Confirm the feed contains `File uploaded`.
    - Click the `File uploaded` event.
    - The right drawer should open the Drive object summary.
    - Screenshot checkpoint: capture the four activity rows.
    - The feed is a user-facing view over audit-chain events.
    - It should not expose low-level event ids unless expanded.

13. Try a safe personal boundary check.
    - Open the tenant switcher.
    - Switch to `Personal - Nadia Park`.
    - Open `Drive`.
    - Search for `day-one-notes.txt`.
    - Expected empty state: `No personal files match this search`.
    - Do not change search filters.
    - Switch back to `Acme Robotics`.
    - Open `Drive`.
    - Search for `day-one-notes.txt`.
    - Expected result: the file appears under `Acme Day One`.
    - Screenshot checkpoint: capture both search states if training evidence is required.

14. Save the quickstart completion note.
    - In `Acme Day One`, click `Notes`.
    - Click `New note`.
    - Title: `Day one setup complete`.
    - Body: `Signup, workspace, Messenger, Mail, and Drive are complete.`
    - Tag: `onboarding`.
    - Click `Save`.
    - Expected toast: `Note saved`.
    - Confirm note owner is `tenant-acme-robotics`.
    - This note gives a human-readable checkpoint inside the workspace.
    - Screenshot checkpoint: capture the saved note header.

15. Run the end-user verification query.
    - Open the command palette with `Ctrl+K`.
    - Type `verify day one`.
    - Select `Run tutorial verification: day-one`.
    - Input `workspace_slug=workspace-acme-day-one`.
    - Input `expected_user=nadia.park@example.com`.
    - Click `Run`.
    - Expected result panel title: `Day-one quickstart complete`.
    - Expected result count: `5 of 5 required actions found`.
    - Screenshot checkpoint: capture the result panel.
    - Keep this panel open until you have copied the result id into your onboarding tracker.

## Verification

- Named query: `tutorial.day_one_quickstart_status`.
- Query location: `Admin Console -> Audit -> Saved Queries -> Tutorial Checks`.
- Query input `tenant_id`: `tenant-acme-robotics`.
- Query input `workspace_slug`: `workspace-acme-day-one`.
- Query input `principal_email`: `nadia.park@example.com`.
- Expected output field: `signup_status`.
- Expected output value: `passkey_registered`.
- Expected output field: `work_tenant_membership`.
- Expected output value: `active`.
- Expected output field: `workspace_created`.
- Expected output value: `workspace-acme-day-one`.
- Expected output field: `messenger_thread_count`.
- Expected output value: `1`.
- Expected output field: `mail_sent_count`.
- Expected output value: `1`.
- Expected output field: `drive_file_count`.
- Expected output value: `1`.
- Expected output field: `personal_boundary_search_result`.
- Expected output value: `0 personal results`.
- Expected output field: `audit_events_minimum`.
- Expected output value: `5`.
- Expected output field: `tenant_context_warnings`.
- Expected output value: `0`.
- Expected output field: `result_label`.
- Expected output value: `Day-one quickstart complete`.
- CLI equivalent for tenant admins:

```bash
oya tutorial verify day-one \
  --tenant tenant-acme-robotics \
  --workspace workspace-acme-day-one \
  --principal nadia.park@example.com
```

- CLI expected line: `PASS tutorial.day_one_quickstart_status`.
- CLI expected line: `workspace_created=workspace-acme-day-one`.
- CLI expected line: `messenger_thread_count=1`.
- CLI expected line: `mail_sent_count=1`.
- CLI expected line: `drive_file_count=1`.
- CLI expected line: `personal_boundary_search_result=0`.
- Dashboard: `Tenant Console -> Health -> Tutorial completion`.
- Dashboard tile: `Day One`.
- Expected tile state: `Complete`.
- Audit event to spot-check: `TenantBoundaryOnboardingConsent`.
- Messenger event to spot-check: `connect.messenger.sent`.
- Mail event to spot-check: `connect.mail.sent`.
- Drive event to spot-check: `drive.file.created`.
- Evidence id format: `tut-dayone-tenant-acme-robotics-<timestamp>`.
- Screenshot evidence folder: `/Acme Day One/Onboarding/Screenshots`.

## Common Pitfalls + Recovery

- Pitfall: the user completes signup but never joins the work tenant.
- Recovery: reopen the invitation link and confirm `Join tenant-acme-robotics` before creating a workspace.
- Pitfall: the header says `Personal - Nadia Park` while sending the Messenger message.
- Recovery: delete the personal draft, switch to `Acme Robotics`, and recreate the work thread.
- Pitfall: the passkey prompt creates a second browser profile credential.
- Recovery: open `Account settings -> Passkeys`, remove the duplicate label, and register one device label.
- Pitfall: the workspace slug auto-generates as `acme-day-one-1`.
- Recovery: edit the slug before creation or rerun the verification query with the actual slug.
- Pitfall: the mentor cannot see the Drive file.
- Recovery: open the file share drawer and confirm `mentor.lee@acme.example - Can comment`.
- Pitfall: the upload is marked `Personal` by mistake.
- Recovery: move the file to personal Drive only if it is truly personal; otherwise delete and upload again inside the work workspace.
- Pitfall: Mail compose opens from the global Mail app instead of the workspace toolbar.
- Recovery: close the draft and relaunch Mail from `Acme Day One` so the workspace context is attached.
- Pitfall: Messenger read receipts do not reach double check mark.
- Recovery: open `Message details` and confirm delivery status; retry only if status remains `queued` after 60 seconds.
- Pitfall: the activity feed appears empty.
- Recovery: refresh the workspace and set feed filter to `All activity`.
- Pitfall: the verification query returns `tenant_context_warnings=1`.
- Recovery: expand the warning and repeat the specific action in the work tenant.
- Pitfall: a personal search finds the uploaded work file.
- Recovery: treat this as a Sev-2 privacy incident; notify `it-admin@acme.example` and stop the tutorial.
- Pitfall: the test file contains regulated data.
- Recovery: delete it, create the exact harmless `day-one-notes.txt` value, and upload that file instead.
- Pitfall: the command palette does not show `verify day one`.
- Recovery: ask the tenant admin to grant `audit.self.read` to the employee onboarding role.
- Pitfall: screenshots are stored on the desktop only.
- Recovery: upload them to `/Acme Day One/Onboarding/Screenshots` so the mentor can inspect the evidence.
- Pitfall: the user tries to invite an external address in step 11.
- Recovery: remove the external address and complete this quickstart with only Acme tenant members.
- Pitfall: the note saves in the personal tenant.
- Recovery: delete the personal note and recreate it from the `Acme Day One` workspace toolbar.
- Pitfall: browser autofill substitutes a different account.
- Recovery: sign out, clear the active account chooser, and sign in as `nadia.park@example.com`.
- Pitfall: a VPN or MDM check blocks work tenant access.
- Recovery: enroll the laptop in `Acme Robotics MDM`, then reopen the invitation link.

## Day-One Evidence Checklist

Use this checklist before closing the first-day session with the mentor.

- Account evidence: screenshot `day-one-01-signin.png` shows `nadia.park@example.com`.
- Tenant evidence: screenshot `day-one-02-tenant-switcher.png` shows `Acme Robotics`.
- Workspace evidence: screenshot `day-one-03-workspace-created.png` shows `Acme Day One`.
- Messenger evidence: screenshot `day-one-04-message-details.png` shows `delivered`.
- Mail evidence: screenshot `day-one-05-mail-sent.png` shows sender `nadia.park@acme.example`.
- Drive evidence: screenshot `day-one-06-drive-file.png` shows `/Acme Day One/Onboarding/day-one-notes.txt`.
- Search evidence: screenshot `day-one-07-search-work.png` shows the work file in work search.
- Boundary evidence: screenshot `day-one-08-search-personal.png` shows no work file in personal search.
- Audit evidence: query `tutorial.day_one_workspace_status` returns `day_one_ready`.
- Mentor evidence: `mentor@acme.example` can open the workspace activity feed.
- Device evidence: MDM status reads `Acme Robotics MDM enrolled`.
- Notification evidence: Messenger notification route is `work tenant only`.
- Retention evidence: Drive folder retention reads `workspace-default-3y`.
- Accessibility evidence: the user can reach Mail, Messenger, and Drive from keyboard navigation.
- Recovery evidence: no personal tenant artifacts remain from accidental sign-in attempts.

Record the final activity timeline in this order.

1. `UserAcceptedInvitation`.
2. `WorkTenantSelected`.
3. `WorkspaceCreated`.
4. `MessengerThreadCreated`.
5. `MailMessageSent`.
6. `DriveFileUploaded`.
7. `WorkspaceActivityFeedUpdated`.
8. `DayOneVerificationQueryRun`.

The timeline matters because it proves the quickstart exercised the minimum Oyatie surface.

It also proves the user did not start from an existing workspace.

If the timeline is missing an event, repeat only the missing step.

Do not recreate the user account unless sign-in itself failed.

Do not upload a different file name for evidence.

Do not send the mail from a personal address.

Do not add external recipients during this tutorial.

The day-one quickstart is complete only after the mentor can see the same workspace evidence.

If the mentor view differs from Nadia's view, compare tenant badge, role, and workspace membership.

The expected workspace membership is exactly `nadia.park@example.com` and `mentor@acme.example`.

The expected workspace role for Nadia is `Member`.

The expected workspace role for mentor is `Mentor`.

The expected Drive file owner is `nadia.park@acme.example`.

The expected Mail thread subject is `Day-one Oyatie check-in`.

The expected Messenger channel name is `day-one-mentor-check`.

The expected verification status is `ready_for_second_day`.

Leave the workspace intact for later tutorials.

Future tutorials assume `Acme Day One` exists and contains this evidence.

## Next Tutorials

- [Set up a cross-tenant messenger channel](cross-tenant-channel-setup.md).
- [Build an employee-onboarding workflow](workflow-studio-build-employee-onboarding.md).
- [Capture and propagate consent across microservices](consent-cascade-across-microservices.md).
- [Activate multiple compliance packs on a tenant](multi-pack-tenant-activation.md).

## References

- [ADR-0311 Dual-Tenant Identity Personal vs Work Boundary](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md).
- [Tenant Lifecycle Standard](../standards/tenant-lifecycle.md).
- [Messenger E2E Encryption and Work Channel Standard](../standards/messenger-e2e-encryption-mls.md).
- [Doc Style](../standards/doc-style.md).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [First-week personal mail journey](../user-journeys/j22-personal-mail-inbox-first-week/README.md).
- [B2B workplace mail and calendar journey](../user-journeys/j35-b2b-workplace-mail-and-calendar/README.md).
- [Drive family backup journey](../user-journeys/j26-drive-family-photo-backup/README.md).
- [Tenant admin guide](../site/src/admin/tenant-admin-guide.md).
- [Operate a tenant guide](../site/src/guides/operate-a-tenant.md).

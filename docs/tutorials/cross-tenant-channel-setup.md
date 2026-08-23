---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-XTENANT-MSG-002
persona: "Priya Krishnan, Acme Robotics partnership lead"
prerequisite_packs:
  - canonical-base
  - workplace-core
  - cross-tenant-collaboration
  - legal-nda-enforcement
related_oyatie_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0263
  - ADR-0311
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "75 minutes"
---

# Set Up a Cross-Tenant Messenger Channel with NDA Enforcement

## Goal

You will create a secure cross-tenant Messenger channel between `tenant-acme-robotics` and `tenant-northwind-design`, require both sides to accept the `NDA-MUTUAL-2026-ACME-NORTHWIND` agreement, send the first message, and verify that work-tenant collaboration succeeds without exposing either participant's personal tenant.

## Prerequisites

- Acme account: `priya.krishnan@acme.example`.
- Northwind account: `lucas.meyer@northwind.example`.
- Acme tenant: `tenant-acme-robotics`.
- Northwind tenant: `tenant-northwind-design`.
- Channel slug to create: `x-acme-northwind-prototype`.
- NDA template id: `nda-mutual-template-v3`.
- NDA envelope id to issue: `NDA-MUTUAL-2026-ACME-NORTHWIND`.
- Subscribed microservices: `messenger`, `identity`, `tenancy`, `policy-engine`, `audit-chain`, `drive`, `mail`, `workflow-engine`.
- Required Cedar permit: `messenger.cross_tenant_channel.create`.
- Required Cedar permit: `messenger.cross_tenant_channel.invite`.
- Required Cedar permit: `messenger.work-message.send`.
- Required Cedar permit: `legal.nda.envelope.issue`.
- Required Cedar permit: `legal.nda.accept`.
- Required Cedar permit: `drive.work-file.share_cross_tenant`.
- Required Cedar permit: `audit.cross_tenant.read`.
- Acme admin group: `grp-acme-partnership-admins`.
- Northwind approver group: `grp-northwind-legal-approvers`.
- Retention profile: `cross-tenant-nda-7y`.
- Data class: `Confidential`.
- Test attachment: `prototype-brief-redacted.pdf`.
- Named verification query: `tutorial.cross_tenant_channel_nda_status`.

## Step-by-Step

1. Confirm you are in Acme's work tenant.
   - Sign in as `priya.krishnan@acme.example`.
   - Open the tenant switcher.
   - Select `Acme Robotics`.
   - Confirm header text: `Tenant context: tenant-acme-robotics`.
   - Confirm role badge: `Partnership Admin`.
   - Open `/messenger/channels`.
   - Screenshot checkpoint: capture the channel list and tenant switcher.
   - If the page shows `Personal - Priya`, switch before continuing.
   - ADR-0311 requires visible tenant context for cross-tenant work.
   - Do not create the channel from a personal tenant.

2. Validate Northwind as an allowed counterparty.
   - In Messenger, click `External tenants`.
   - Search for `tenant-northwind-design`.
   - Confirm organization label: `Northwind Design GmbH`.
   - Confirm trust status: `Verified business tenant`.
   - Confirm residency: `EU - Frankfurt cell`.
   - Confirm collaboration policy: `NDA required before message send`.
   - Screenshot checkpoint: capture the verified tenant profile.
   - Click `View policy diff`.
   - Confirm Acme permits confidential design collaboration with verified tenants.
   - Close the policy diff drawer.

3. Issue the NDA envelope.
   - Open `Legal -> NDA envelopes`.
   - Click `New NDA envelope`.
   - Template: `nda-mutual-template-v3`.
   - Envelope id: `NDA-MUTUAL-2026-ACME-NORTHWIND`.
   - Acme signer: `priya.krishnan@acme.example`.
   - Northwind signer: `lucas.meyer@northwind.example`.
   - Protected project label: `Prototype Armature Review`.
   - Effective date: `2026-05-20`.
   - Expiration: `2028-05-20`.
   - Click `Issue envelope`.
   - Expected toast: `NDA envelope issued`.

4. Confirm NDA evidence before inviting.
   - Open the issued envelope.
   - Confirm state: `Awaiting counterparty acceptance`.
   - Confirm Acme acceptance state: `Accepted by Priya Krishnan`.
   - Confirm Northwind acceptance state: `Pending`.
   - Confirm audit stream: `legal.nda.envelope.issued`.
   - Confirm linked tenant ids include both tenants.
   - Screenshot checkpoint: capture the envelope summary.
   - Copy the envelope id.
   - Return to Messenger.
   - The channel invite will reference this exact id.

5. Create the cross-tenant channel shell.
   - Click `New channel`.
   - Choose `Cross-tenant channel`.
   - Channel name: `Acme x Northwind prototype`.
   - Channel slug: `x-acme-northwind-prototype`.
   - Owning workspace: `Acme Partnerships`.
   - Counterparty tenant: `tenant-northwind-design`.
   - Data class: `Confidential`.
   - Retention: `cross-tenant-nda-7y`.
   - Encryption mode: `tenant-DEK per side`.
   - Click `Continue`.

6. Attach the NDA enforcement rule.
   - In the `Policy gates` step, enable `NDA required`.
   - NDA envelope id: `NDA-MUTUAL-2026-ACME-NORTHWIND`.
   - Enforcement mode: `Block send until both tenants accepted`.
   - Allowed actions before acceptance: `View channel metadata only`.
   - Blocked actions before acceptance: `Send message`, `Upload file`, `Mention user`.
   - Cedar permit set: `permit-set-cross-tenant-nda-channel-v1`.
   - Default-deny fragment: `policy/tenant-boundary-work-vs-personal.cedar`.
   - Click `Validate gate`.
   - Expected result: `Gate valid`.
   - Screenshot checkpoint: capture the policy gate result.

7. Invite the Northwind signer.
   - In `Participants`, add `lucas.meyer@northwind.example`.
   - Role: `Counterparty contributor`.
   - Required action: `Accept NDA then join`.
   - Notification method: `Mail + Messenger external invite`.
   - Message: `Please accept the NDA for Prototype Armature Review.`
   - Click `Send invite`.
   - Expected toast: `Invite sent to Northwind Design`.
   - Expected channel state: `Pending counterparty NDA`.
   - Screenshot checkpoint: capture the pending state.
   - Do not override the policy gate.

8. Verify the pre-acceptance send block.
   - In the new channel, type `Can you see this before acceptance?`.
   - The Send button should be disabled.
   - Hover the disabled Send button.
   - Tooltip should read `NDA acceptance required before messages can be sent`.
   - Screenshot checkpoint: capture the disabled send state.
   - Open `Channel details -> Policy`.
   - Confirm `Blocked by NDA-MUTUAL-2026-ACME-NORTHWIND`.
   - Close the details drawer.
   - Delete the draft text.
   - This proves the tutorial is enforcing before trust is complete.

9. Have the counterparty accept from Northwind.
   - Lucas signs in as `lucas.meyer@northwind.example`.
   - Lucas selects tenant `Northwind Design`.
   - Lucas opens `Inbox -> External collaboration invites`.
   - Lucas opens `Acme x Northwind prototype`.
   - The page shows `NDA acceptance required`.
   - Lucas clicks `Review NDA`.
   - Lucas confirms envelope id `NDA-MUTUAL-2026-ACME-NORTHWIND`.
   - Lucas clicks `Accept NDA and join channel`.
   - Expected toast: `NDA accepted. Channel joined.`
   - Screenshot checkpoint: capture Northwind's joined state.

10. Confirm the channel is active on Acme's side.
    - Priya returns to the Acme browser session.
    - Refresh `x-acme-northwind-prototype`.
    - Expected banner: `Counterparty accepted NDA`.
    - Channel state should read `Active`.
    - Participants should list Priya and Lucas.
    - Policy tab should show `NDA enforcement: active`.
    - Encryption tab should show `Acme DEK: isolated`.
    - Encryption tab should show `Northwind DEK: isolated`.
    - Screenshot checkpoint: capture the active banner and participant list.
    - The channel can now accept messages.

11. Send the first post-NDA message.
    - Priya types: `Lucas, NDA is active. I am sharing the redacted prototype brief now.`
    - Click `Send`.
    - Expected delivery: double check mark within 10 seconds.
    - Message details should show `policy_gate=NDA accepted`.
    - Message owner should show `tenant-acme-robotics`.
    - Counterparty visibility should show `tenant-northwind-design`.
    - Audit topic should show `connect.messenger.sent`.
    - Screenshot checkpoint: capture the message details drawer.
    - Close the drawer.
    - Do not paste unredacted design material in this tutorial.

12. Share the redacted Drive attachment.
    - Click the attachment icon.
    - Choose `prototype-brief-redacted.pdf`.
    - Confirm data class: `Confidential`.
    - Confirm share scope: `This cross-tenant channel only`.
    - Confirm NDA envelope: `NDA-MUTUAL-2026-ACME-NORTHWIND`.
    - Confirm download policy: `Watermark and audit`.
    - Click `Attach`.
    - Expected scan result: `No high-risk secrets found`.
    - Click `Send attachment`.
    - Screenshot checkpoint: capture the attachment card with NDA badge.

13. Verify Northwind can view but not over-share.
    - Lucas opens the attachment card.
    - Expected viewer banner: `Shared under NDA-MUTUAL-2026-ACME-NORTHWIND`.
    - Lucas clicks `Download`.
    - Expected behavior: watermark prompt appears.
    - Lucas clicks `Share`.
    - Expected behavior: `Sharing outside this channel is blocked`.
    - Screenshot checkpoint: capture the blocked share message.
    - Lucas closes the viewer.
    - This confirms the deal is collaboration-scoped, not tenant-wide.
    - It also prevents accidental onward disclosure.

14. Inspect cross-tenant audit evidence.
    - Priya opens `Channel details`.
    - Select `Audit`.
    - Confirm events include `CrossTenantChannelCreated`.
    - Confirm events include `NdaEnvelopeIssued`.
    - Confirm events include `NdaAccepted`.
    - Confirm events include `CrossTenantMessageSent`.
    - Confirm events include `CrossTenantAttachmentShared`.
    - Confirm each event lists both tenant ids.
    - Screenshot checkpoint: capture the audit event list.
    - Click `Export summary`.
    - Save as `acme-northwind-channel-audit-summary.pdf`.

15. Test personal tenant denial.
    - Priya switches to `Personal - Priya Krishnan`.
    - Priya opens Messenger.
    - Search `Acme x Northwind prototype`.
    - Expected result: `No personal channels match this search`.
    - Priya opens Drive.
    - Search `prototype-brief-redacted.pdf`.
    - Expected result: `No personal files match this search`.
    - Screenshot checkpoint: capture both personal empty states.
    - Switch back to `Acme Robotics`.
    - This is the ADR-0311 boundary check for the tutorial.

16. Run the channel verification query.
    - Open `Admin Console -> Audit -> Saved Queries`.
    - Choose `tutorial.cross_tenant_channel_nda_status`.
    - Input `source_tenant=tenant-acme-robotics`.
    - Input `counterparty_tenant=tenant-northwind-design`.
    - Input `channel_slug=x-acme-northwind-prototype`.
    - Input `nda_envelope_id=NDA-MUTUAL-2026-ACME-NORTHWIND`.
    - Click `Run`.
    - Expected result title: `Cross-tenant NDA channel complete`.
    - Expected status: `PASS`.
    - Screenshot checkpoint: capture the query output.

## Verification

- Named query: `tutorial.cross_tenant_channel_nda_status`.
- Query location: `Admin Console -> Audit -> Saved Queries -> Tutorial Checks`.
- Query input `source_tenant`: `tenant-acme-robotics`.
- Query input `counterparty_tenant`: `tenant-northwind-design`.
- Query input `channel_slug`: `x-acme-northwind-prototype`.
- Query input `nda_envelope_id`: `NDA-MUTUAL-2026-ACME-NORTHWIND`.
- Expected output field: `channel_state`.
- Expected output value: `active`.
- Expected output field: `nda_state`.
- Expected output value: `accepted_by_all_required_tenants`.
- Expected output field: `send_block_before_acceptance`.
- Expected output value: `observed`.
- Expected output field: `source_tenant_dek_isolated`.
- Expected output value: `true`.
- Expected output field: `counterparty_tenant_dek_isolated`.
- Expected output value: `true`.
- Expected output field: `personal_tenant_visibility`.
- Expected output value: `denied`.
- Expected output field: `attachment_share_scope`.
- Expected output value: `channel_only`.
- Expected output field: `audit_event_count_minimum`.
- Expected output value: `5`.
- Expected output field: `result_label`.
- Expected output value: `Cross-tenant NDA channel complete`.
- CLI equivalent:

```bash
oya tutorial verify cross-tenant-channel \
  --source-tenant tenant-acme-robotics \
  --counterparty-tenant tenant-northwind-design \
  --channel x-acme-northwind-prototype \
  --nda NDA-MUTUAL-2026-ACME-NORTHWIND
```

- CLI expected line: `PASS tutorial.cross_tenant_channel_nda_status`.
- CLI expected line: `channel_state=active`.
- CLI expected line: `nda_state=accepted_by_all_required_tenants`.
- CLI expected line: `personal_tenant_visibility=denied`.
- Audit event to inspect: `CrossTenantChannelCreated`.
- Audit event to inspect: `NdaEnvelopeIssued`.
- Audit event to inspect: `NdaAccepted`.
- Audit event to inspect: `CrossTenantMessageSent`.
- Audit event to inspect: `CrossTenantAttachmentShared`.
- Dashboard: `Tenant Console -> Collaboration -> External channel health`.
- Expected dashboard tile: `Acme x Northwind prototype - healthy`.
- Evidence artifact: `acme-northwind-channel-audit-summary.pdf`.
- Evidence location: `Drive -> Acme Partnerships -> Legal Evidence`.

## Common Pitfalls + Recovery

- Pitfall: the channel is created without an NDA envelope.
- Recovery: open `Channel details -> Policy`, attach `NDA-MUTUAL-2026-ACME-NORTHWIND`, and rerun `Validate gate`.
- Pitfall: the channel was created as a normal internal channel.
- Recovery: archive it and create a new `Cross-tenant channel`; do not convert internal history across tenant boundaries.
- Pitfall: Northwind accepts using a personal tenant.
- Recovery: revoke the invite and resend to `lucas.meyer@northwind.example` under `tenant-northwind-design`.
- Pitfall: the Send button is enabled before acceptance.
- Recovery: disable the channel, notify security, and inspect the Cedar permit set for missing default-deny.
- Pitfall: attachment scanning flags the file.
- Recovery: remove the attachment, upload a redacted file, and keep the original in Acme-only Drive.
- Pitfall: Lucas can reshare outside the channel.
- Recovery: revoke file grant immediately and check `drive.work-file.share_cross_tenant` scope.
- Pitfall: the query reports one tenant id only.
- Recovery: inspect audit-chain emission; cross-tenant channel events must list both tenant ids.
- Pitfall: the retention profile is `default-1y`.
- Recovery: change it to `cross-tenant-nda-7y` before sending any message.
- Pitfall: the wrong NDA template is issued.
- Recovery: void the envelope and issue `nda-mutual-template-v3`; keep the void event for audit.
- Pitfall: the counterparty cannot open the invite.
- Recovery: verify `tenant-northwind-design` is still a verified business tenant and invite domain policy allows `northwind.example`.
- Pitfall: a user mentions a personal email address in the channel.
- Recovery: remove the mention, educate the user, and confirm participant list contains work identities only.
- Pitfall: the channel search appears in Priya's personal tenant.
- Recovery: treat as a privacy incident and run `tutorial.cross_tenant_channel_nda_status` before resuming.
- Pitfall: Northwind sees Acme's internal-only workspace path.
- Recovery: remove workspace path rendering from the counterparty view; the external channel should expose channel metadata only.
- Pitfall: the evidence export fails.
- Recovery: rerun from `Channel details -> Audit` after refreshing, then confirm `audit.cross_tenant.read` is granted.
- Pitfall: the legal team wants a paper copy.
- Recovery: use `Export summary`, not screenshots alone, because the export includes audit-chain seals.
- Pitfall: the channel was named with project secrets.
- Recovery: rename the channel to a neutral slug and leave secret details in NDA-protected messages.
- Pitfall: the counterparty asks for unrestricted download.
- Recovery: keep watermark and audit enabled unless legal issues a narrower exception permit.
- Pitfall: the Acme user lacks `Partnership Admin`.
- Recovery: ask a tenant admin to grant `grp-acme-partnership-admins`, then repeat from step 1.

## NDA Enforcement Evidence Checklist

Before users begin real partner work, collect these cross-tenant checks.

- Channel id should be `channel-acme-northwind-2026-design-review`.
- Acme tenant id should be `tenant-acme-robotics`.
- Northwind tenant id should be `tenant-northwind-design`.
- NDA envelope id should be `NDA-MUTUAL-2026-ACME-NORTHWIND`.
- NDA template should be `nda-mutual-template-v3`.
- Channel policy should be `cross-tenant-nda-enforced-v1`.
- Retention profile should be `cross-tenant-nda-7y`.
- Encryption context should show both tenant ids.
- Tenant DEK unwrap should happen only inside each tenant boundary.
- Personal tenant participation count should be `0`.
- External download policy should be `watermarked`.
- Drive share policy should be `nda-scoped-view-only`.
- Messenger message policy should be `nda-required-before-read`.
- Audit event `CrossTenantChannelCreated` should have two tenant ids.
- Audit event `NdaEnvelopeSignedByAllParties` should precede the first readable message.
- Audit event `CrossTenantMessageSent` should include the channel id.
- Audit event `CrossTenantFileShared` should include the NDA envelope id.
- Verification query should return `nda_gate_state=enforced`.

Use a three-message acceptance test after setup.

1. Priya sends `Design kickoff ready after NDA`.
2. A Northwind user replies `Northwind confirms NDA-gated access`.
3. Priya shares `fixture-redacted-design-brief.pdf`.

Each message must show the `NDA enforced` lock in message details.

Each message must show tenant context in the detail drawer.

The file share must show `View only`.

The file share must show `Watermark on`.

The file share must not show `Download original`.

The channel member list must group users by tenant.

The member list must not collapse both tenants into one company group.

The evidence export must include the final member list.

The export must include policy evaluation ids for the NDA checks.

The export must include the signed envelope digest.

The export must include a redacted preview of the shared file name.

Do not rely on screenshots alone for the legal handoff.

Legal needs the sealed export because it binds signatures, policy, and message events.

If a participant changes companies, remove them before continuing the channel.

If the NDA expires, the channel should become read-only until renewal.

If either tenant is suspended, the channel should become inaccessible to that tenant.

Record those expected failure states in the partner launch note.

This tutorial is complete when both tenant admins can independently verify the same channel id.

## Next Tutorials

- [First-day-on-Oyatie quickstart](quickstart-new-user-day-one.md).
- [List, sell, buy, and settle a marketplace deal](marketplace-list-sell-buy.md).
- [Project Salesforce CRM data into Oyatie ontology](ontology-projection-from-external-source.md).
- [Capture and propagate consent across microservices](consent-cascade-across-microservices.md).
- [Handle a GDPR erasure request](data-subject-erasure-request-handling.md).

## References

- [ADR-0311 Dual-Tenant Identity Personal vs Work Boundary](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md).
- [Messenger E2E Encryption and Work Channel Standard](../standards/messenger-e2e-encryption-mls.md).
- [Cross-tenant real-time visibility ADR](../decisions/ADR-0214-cross-tenant-real-time-visibility.md).
- [Cedar Policy Evaluation Flow](../architecture/diagrams/cedar-policy-evaluation-flow.md).
- [Cross-tenant access detected runbook](../runbooks/cross-axis/cross-tenant-access-detected.md).
- [Employee secondment cross-tenant journey](../user-journeys/j114-employee-secondment-cross-tenant/README.md).
- [Hospital network cross-tenant referral journey](../user-journeys/j64-hospital-network-cross-tenant-referral/README.md).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [Doc Style](../standards/doc-style.md).

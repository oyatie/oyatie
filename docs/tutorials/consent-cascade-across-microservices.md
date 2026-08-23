---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-CONSENT-CASCADE-006
persona: "Hae-Won Kim, tenant privacy administrator"
prerequisite_packs:
  - canonical-base
  - consent-management
  - eu-gdpr
  - workplace-core
related_oyatie_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0272
  - ADR-0311
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "80 minutes"
---

# Capture and Propagate Consent Across Microservices

## Goal

You will capture a granular consent grant for `sofia.martin@example.eu`, propagate that consent across twelve Oyatie microservices, prove that allowed actions become available, withdraw one purpose, and verify that dependent services stop using the withdrawn purpose without disturbing unrelated consent.

## Prerequisites

- Privacy admin account: `haewon.kim@acme.example`.
- Data subject account: `sofia.martin@example.eu`.
- Tenant: `tenant-acme-robotics`.
- Personal tenant: `b2c-sofia-martin`.
- Consent campaign id: `consent-campaign-productivity-ai-2026`.
- Consent record id: `consent-sofia-productivity-ai-2026-05-20`.
- Jurisdiction pack: `EU-GDPR-2018-baseline`.
- Subscribed microservices: `identity`, `tenancy`, `consent-graph`, `policy-engine`, `messenger`, `mail`, `drive`, `marketplace`, `intelligence`, `ontology`, `workflow-engine`, `audit-chain`.
- Required Cedar permit: `consent.campaign.create`.
- Required Cedar permit: `consent.record.capture`.
- Required Cedar permit: `consent.record.propagate`.
- Required Cedar permit: `consent.record.withdraw`.
- Required Cedar permit: `policy.consent.evaluate`.
- Required Cedar permit: `intelligence.summary.run`.
- Required Cedar permit: `audit.consent.read`.
- Consent purpose to allow: `productivity_ai_summary`.
- Consent purpose to allow: `workplace_analytics_basic`.
- Consent purpose to deny later: `marketplace_personalization`.
- Test document: `team-retrospective-notes-redacted.pdf`.
- Test Messenger thread: `thread-q2-retro-planning`.
- Test Mail label: `Productivity AI eligible`.
- Named saved query: `tutorial.consent_cascade_status`.

## Step-by-Step

1. Open consent administration.
   - Sign in as `haewon.kim@acme.example`.
   - Switch to `Acme Robotics`.
   - Open `Privacy -> Consent`.
   - Confirm header text: `Tenant context: tenant-acme-robotics`.
   - Confirm pack badge: `EU-GDPR-2018-baseline`.
   - Confirm role badge: `Privacy Admin`.
   - Screenshot checkpoint: capture the consent dashboard.
   - If the pack badge is missing, activate GDPR before proceeding.
   - This tutorial assumes consent is purpose-scoped, not blanket authorization.
   - Leave `Show inherited consent` enabled.

2. Create the consent campaign.
   - Click `New consent campaign`.
   - Campaign id: `consent-campaign-productivity-ai-2026`.
   - Display name: `Productivity AI and analytics consent`.
   - Audience: `EU employees`.
   - Legal basis: `Consent`.
   - Expiration: `P1Y`.
   - Renewal reminder: `P30D before expiration`.
   - Click `Create campaign`.
   - Expected toast: `Consent campaign created`.
   - Screenshot checkpoint: capture the campaign summary.

3. Define purpose `productivity_ai_summary`.
   - In the campaign, click `Add purpose`.
   - Purpose id: `productivity_ai_summary`.
   - Description: `Allow AI summaries of work documents and work discussions.`
   - Data classes: `Internal`, `Confidential`.
   - Services: `messenger`, `mail`, `drive`, `intelligence`, `ontology`.
   - Default state: `Off`.
   - Withdrawal behavior: `Stop future processing; retain audit evidence`.
   - Click `Save purpose`.
   - Expected purpose card: `productivity_ai_summary - Off by default`.
   - Screenshot checkpoint: capture the purpose card.

4. Define purpose `workplace_analytics_basic`.
   - Click `Add purpose`.
   - Purpose id: `workplace_analytics_basic`.
   - Description: `Allow aggregate workplace productivity analytics.`
   - Data classes: `Internal metadata`.
   - Services: `identity`, `tenancy`, `messenger`, `mail`, `workflow-engine`, `audit-chain`.
   - Default state: `Off`.
   - Withdrawal behavior: `Stop future aggregation`.
   - Click `Save purpose`.
   - Screenshot checkpoint: capture the analytics purpose.
   - This purpose should never expose message contents.
   - Keep the purpose separate from AI summary consent.

5. Define purpose `marketplace_personalization`.
   - Click `Add purpose`.
   - Purpose id: `marketplace_personalization`.
   - Description: `Allow marketplace recommendations based on work profile signals.`
   - Data classes: `Profile metadata`.
   - Services: `marketplace`, `ontology`, `policy-engine`, `audit-chain`.
   - Default state: `Off`.
   - Withdrawal behavior: `Stop recommendation use immediately`.
   - Click `Save purpose`.
   - Screenshot checkpoint: capture the marketplace purpose.
   - This purpose will be withdrawn later in the tutorial.
   - It proves one-purpose withdrawal does not revoke all consent.

6. Publish the campaign to preview.
   - Click `Validate`.
   - Expected validation: `All purposes have withdrawal behavior`.
   - Expected validation: `Cedar policies resolved`.
   - Expected validation: `No purpose is on by default`.
   - Click `Publish preview`.
   - Version: `2026.05.preview`.
   - Expected toast: `Consent campaign preview published`.
   - Screenshot checkpoint: capture validation and preview version.
   - Do not publish to every employee in this tutorial.
   - Use Sofia as a targeted preview subject.

7. Send Sofia the consent request.
   - Click `Send preview request`.
   - Subject: `sofia.martin@example.eu`.
   - Subject tenant: `b2c-sofia-martin`.
   - Work tenant: `tenant-acme-robotics`.
   - Request language: `English`.
   - Include plain-language summary: enabled.
   - Include service list: enabled.
   - Click `Send`.
   - Expected event: `ConsentRequestSent`.
   - Screenshot checkpoint: capture request sent status.
   - Copy the request id shown in the drawer.

8. Capture Sofia's consent.
   - Sofia opens the consent portal link.
   - Sofia signs in with passkey.
   - Sofia sees `Productivity AI and analytics consent`.
   - Sofia enables `AI summaries of work documents and work discussions`.
   - Sofia enables `Aggregate workplace productivity analytics`.
   - Sofia enables `Marketplace recommendations based on work profile signals`.
   - Sofia clicks `Save choices`.
   - Expected toast: `Your consent choices were saved`.
   - Consent record id: `consent-sofia-productivity-ai-2026-05-20`.
   - Screenshot checkpoint: capture the saved choices screen.

9. Confirm consent record in admin view.
   - Hae-Won opens `Consent records`.
   - Search `sofia.martin@example.eu`.
   - Open `consent-sofia-productivity-ai-2026-05-20`.
   - Confirm `productivity_ai_summary: granted`.
   - Confirm `workplace_analytics_basic: granted`.
   - Confirm `marketplace_personalization: granted`.
   - Confirm signature method: `passkey`.
   - Confirm pack: `EU-GDPR-2018-baseline`.
   - Screenshot checkpoint: capture the record detail.

10. Propagate consent to services.
    - Click `Propagate`.
    - Select services: `identity`, `tenancy`, `policy-engine`, `messenger`, `mail`, `drive`, `marketplace`, `intelligence`, `ontology`, `workflow-engine`, `audit-chain`, `consent-graph`.
    - Propagation mode: `Immediate`.
    - Click `Start propagation`.
    - Expected state: `Propagation running`.
    - Wait until all service rows show `Applied`.
    - Screenshot checkpoint: capture the twelve-service matrix.
    - Export matrix as `consent-sofia-propagation-matrix.json`.
    - This is the 8+ microservice cascade required by the tutorial.

11. Inspect service-specific effects.
    - Identity row: `subject consent claims refreshed`.
    - Tenancy row: `tenant membership consent overlay updated`.
    - Policy-engine row: `Cedar consent predicates available`.
    - Messenger row: `thread summary eligibility updated`.
    - Mail row: `mail summary eligibility updated`.
    - Drive row: `document summary eligibility updated`.
    - Marketplace row: `recommendation eligibility updated`.
    - Intelligence row: `purpose token accepted`.
    - Ontology row: `projection filters updated`.
    - Workflow-engine row: `consent-aware workflows unblocked`.
    - Audit-chain row: `ConsentRecordPropagated sealed`.

12. Test an allowed AI summary.
    - Open Drive as Sofia under `tenant-acme-robotics`.
    - Select `team-retrospective-notes-redacted.pdf`.
    - Click `Ask AI`.
    - Choose `Summarize document`.
    - Consent badge should read `Allowed: productivity_ai_summary`.
    - Click `Generate summary`.
    - Expected result title: `Summary of team-retrospective-notes-redacted.pdf`.
    - Screenshot checkpoint: capture consent badge and generated summary.
    - Save the summary as `retro-summary-consent-allowed`.
    - Do not use documents outside Sofia's allowed work scope.

13. Test Messenger eligibility.
    - Open Messenger thread `thread-q2-retro-planning`.
    - Click `Summarize thread`.
    - Consent badge should read `Allowed by consent-sofia-productivity-ai-2026-05-20`.
    - Click `Generate`.
    - Expected summary length: `5 bullets`.
    - Expected audit event: `ConsentBoundSummaryGenerated`.
    - Screenshot checkpoint: capture the thread summary panel.
    - Confirm no personal tenant messages appear in the summary.
    - Close the panel.
    - Return to the consent record.

14. Test marketplace personalization before withdrawal.
    - Open `Marketplace`.
    - Open `Recommendations`.
    - Consent badge should read `Allowed: marketplace_personalization`.
    - Expected recommendation example: `Ergonomic desk accessories`.
    - Screenshot checkpoint: capture the recommendations panel.
    - Open `Why am I seeing this?`.
    - Expected reason: `Work profile signals allowed by consent`.
    - Close the explanation drawer.
    - This verifies the purpose is active before withdrawal.
    - The next step withdraws this purpose only.

15. Withdraw marketplace personalization.
    - Hae-Won opens Sofia's consent record.
    - Click `Edit purposes`.
    - Set `marketplace_personalization` to `Withdrawn`.
    - Leave `productivity_ai_summary` granted.
    - Leave `workplace_analytics_basic` granted.
    - Reason: `User requested no marketplace personalization`.
    - Click `Save withdrawal`.
    - Expected toast: `Consent purpose withdrawn`.
    - Expected event: `ConsentPurposeWithdrawn`.
    - Screenshot checkpoint: capture the updated purposes.

16. Propagate the withdrawal.
    - Click `Propagate withdrawal`.
    - Services should include `marketplace`, `ontology`, `policy-engine`, `audit-chain`.
    - Services may include `consent-graph` for source of truth update.
    - Click `Start propagation`.
    - Expected service state: `Applied`.
    - Marketplace row should show `recommendations disabled`.
    - Ontology row should show `marketplace projection filter revoked`.
    - Policy row should show `Cedar predicate now denies marketplace purpose`.
    - Screenshot checkpoint: capture the withdrawal propagation matrix.
    - Export matrix as `consent-sofia-withdrawal-matrix.json`.

17. Verify allowed purposes still work.
    - Sofia opens Drive.
    - Select `team-retrospective-notes-redacted.pdf`.
    - Click `Ask AI -> Summarize document`.
    - Expected badge: `Allowed: productivity_ai_summary`.
    - Generate a new summary.
    - Expected success: summary returns.
    - Open `Marketplace -> Recommendations`.
    - Expected banner: `Personalized recommendations are off`.
    - Screenshot checkpoint: capture both post-withdrawal states.
    - This proves withdrawal is purpose-scoped, not all-or-nothing.

18. Run the consent cascade verification query.
    - Open `Privacy -> Consent -> Saved checks`.
    - Choose `tutorial.consent_cascade_status`.
    - Input `tenant_id=tenant-acme-robotics`.
    - Input `subject=b2c-sofia-martin`.
    - Input `consent_record_id=consent-sofia-productivity-ai-2026-05-20`.
    - Click `Run`.
    - Expected title: `Consent cascade complete`.
    - Expected status: `PASS`.
    - Screenshot checkpoint: capture the query output.
    - Save the output to `/Privacy Evidence/Consent Cascades`.
    - The tutorial is complete when grant and withdrawal states both match expected values.

## Verification

- Named query: `tutorial.consent_cascade_status`.
- Query location: `Privacy -> Consent -> Saved checks`.
- Query input `tenant_id`: `tenant-acme-robotics`.
- Query input `subject_tenant`: `b2c-sofia-martin`.
- Query input `consent_record_id`: `consent-sofia-productivity-ai-2026-05-20`.
- Expected output field: `campaign_state`.
- Expected output value: `preview_published`.
- Expected output field: `record_signature`.
- Expected output value: `passkey`.
- Expected output field: `services_initially_propagated`.
- Expected output value: `12`.
- Expected output field: `productivity_ai_summary`.
- Expected output value: `granted`.
- Expected output field: `workplace_analytics_basic`.
- Expected output value: `granted`.
- Expected output field: `marketplace_personalization`.
- Expected output value: `withdrawn`.
- Expected output field: `marketplace_recommendation_policy`.
- Expected output value: `denied`.
- Expected output field: `drive_summary_policy`.
- Expected output value: `allowed`.
- Expected output field: `messenger_summary_policy`.
- Expected output value: `allowed`.
- Expected output field: `withdrawal_services_applied`.
- Expected output value: `5`.
- Expected output field: `audit_chain_seals_present`.
- Expected output value: `true`.
- Expected output field: `result_label`.
- Expected output value: `Consent cascade complete`.
- CLI equivalent:

```bash
oya privacy verify consent-cascade \
  --tenant tenant-acme-robotics \
  --subject b2c-sofia-martin \
  --record consent-sofia-productivity-ai-2026-05-20
```

- CLI expected line: `PASS tutorial.consent_cascade_status`.
- CLI expected line: `services_initially_propagated=12`.
- CLI expected line: `marketplace_personalization=withdrawn`.
- CLI expected line: `drive_summary_policy=allowed`.
- CLI expected line: `marketplace_recommendation_policy=denied`.
- Audit event to inspect: `ConsentRequestSent`.
- Audit event to inspect: `ConsentRecordCaptured`.
- Audit event to inspect: `ConsentRecordPropagated`.
- Audit event to inspect: `ConsentBoundSummaryGenerated`.
- Audit event to inspect: `ConsentPurposeWithdrawn`.
- Evidence artifact: `consent-sofia-propagation-matrix.json`.
- Evidence artifact: `consent-sofia-withdrawal-matrix.json`.
- Dashboard: `Privacy -> Consent propagation health`.
- Expected tile: `No service drift`.

## Common Pitfalls + Recovery

- Pitfall: the campaign enables a purpose by default.
- Recovery: set every purpose default to `Off` and republish preview.
- Pitfall: multiple purposes are collapsed into one checkbox.
- Recovery: split AI summaries, analytics, and marketplace personalization into separate purposes.
- Pitfall: propagation omits `policy-engine`.
- Recovery: rerun propagation; service state alone is useless without Cedar predicate updates.
- Pitfall: propagation omits `audit-chain`.
- Recovery: rerun propagation so consent capture and withdrawal are sealed.
- Pitfall: the withdrawal revokes all purposes.
- Recovery: restore the two still-granted purposes and propagate only marketplace withdrawal.
- Pitfall: Drive summaries fail after marketplace withdrawal.
- Recovery: inspect purpose mapping; Drive should depend on `productivity_ai_summary`, not marketplace personalization.
- Pitfall: Marketplace still recommends after withdrawal.
- Recovery: inspect `marketplace_recommendation_policy`; it should be `denied`.
- Pitfall: Sofia's personal tenant messages appear in work thread summary.
- Recovery: stop AI summaries and inspect ADR-0311 tenant-boundary enforcement.
- Pitfall: the consent record has no passkey signature.
- Recovery: resend the request and require passkey confirmation.
- Pitfall: jurisdiction pack is missing.
- Recovery: activate `EU-GDPR-2018-baseline` before collecting EU employee consent.
- Pitfall: the consent campaign has no expiration.
- Recovery: set expiration `P1Y` and renewal reminder `P30D`.
- Pitfall: withdrawal reason is blank.
- Recovery: enter `User requested no marketplace personalization`.
- Pitfall: intelligence service accepts a summary without a purpose token.
- Recovery: revoke the capability grant and inspect `intelligence.summary.run` policy.
- Pitfall: ontology projection filters do not update.
- Recovery: rerun withdrawal propagation and validate `marketplace projection filter revoked`.
- Pitfall: audit query shows drift.
- Recovery: click the drifting microservice row and replay the propagation event.
- Pitfall: the subject cannot see their saved choices.
- Recovery: grant self-service consent view through `consent.record.read_self`.
- Pitfall: a tenant admin changes consent on behalf of Sofia.
- Recovery: only record user-authenticated choices unless legal basis permits admin action.
- Pitfall: screenshots contain personal unrelated data.
- Recovery: retake screenshots with only the consent record and service matrix visible.

## Consent Propagation Matrix

Record the final propagation matrix after capture and after withdrawal.

- Consent service should show `source_of_truth=true`.
- Policy engine should show purpose token `productivity_ai_summary`.
- Intelligence should show `summary_allowed=true` before withdrawal.
- Drive should show `document_summary_allowed=true` before withdrawal.
- Mail should show `thread_summary_allowed=true` before withdrawal.
- Messenger should show `work_thread_summary_allowed=true` before withdrawal.
- Marketplace should show `recommendation_allowed=false` after withdrawal.
- Ontology should show `projection_filter=consent_scoped`.
- Search should show `personalization_index_allowed=false` after withdrawal.
- Notification should show `renewal_notice_allowed=true`.
- Audit-chain should show both `ConsentCaptured` and `ConsentWithdrawn`.
- Observability should show propagation lag below `30s`.

The matrix is important because one green service row is not enough.

Consent must be coherent across the surface where user data can be read, projected, summarized, recommended, or indexed.

The expected final result is mixed, not all allow or all deny.

Productivity summary can stay allowed only for still-consented purposes.

Marketplace personalization must be denied after Sofia withdraws that purpose.

The tutorial is complete when the matrix and query agree.

## Next Tutorials

- [Handle a GDPR erasure request](data-subject-erasure-request-handling.md).
- [Use intelligence to summarize a 200-page contract](ai-assisted-document-summarization.md).
- [Activate HIPAA, SOC 2, GDPR, and KR-PIPA on a tenant](multi-pack-tenant-activation.md).
- [First-day-on-Oyatie quickstart](quickstart-new-user-day-one.md).
- [Project Salesforce CRM data into Oyatie ontology](ontology-projection-from-external-source.md).

## References

- [Cookie consent per-purpose analytics opt-in ADR](../decisions/ADR-0272-cookie-consent-per-purpose-analytics-opt-in.md).
- [Consent withdrawal cascade runbook](../runbooks/consent-withdrawal-cascade.md).
- [Privacy Review Standard](../standards/privacy-review.md).
- [GDPR compliance pack](../../registry/compliance-packs/GDPR.yaml).
- [Autonomy Ceiling Standard](../standards/autonomy-ceiling.md).
- [ADR-0311 Dual-Tenant Identity Personal vs Work Boundary](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md).
- [Cedar Policy Authoring Standard](../standards/cedar-policy-authoring.md).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [Doc Style](../standards/doc-style.md).

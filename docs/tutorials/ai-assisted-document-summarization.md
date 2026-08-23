---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-AI-SUMMARY-008
persona: "Valeria Ionescu, legal counsel reviewing vendor contracts"
prerequisite_packs:
  - canonical-base
  - workplace-core
  - intelligence-summary
  - legal-document-review
related_oyatie_adrs:
  - ADR-0243
  - ADR-0251
  - ADR-0255
  - ADR-0263
  - ADR-0272
  - ADR-0311
  - ADR-0316
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "70 minutes"
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Use Intelligence to Summarize a 200-Page Contract

## Goal

You will upload a 200-page contract, classify it, confirm consent and pack policy, run an intelligence summarization job, inspect page-cited output, ask follow-up questions, export a legal review packet, and verify that the AI job stayed inside the tenant, data-class, and consent boundaries.

## Prerequisites

- Legal counsel account: `valeria.ionescu@acme.example`.
- Tenant: `tenant-acme-robotics`.
- Workspace: `workspace-legal-vendor-review`.
- Contract file: `northwind-master-services-agreement-200p.pdf`.
- Contract id: `contract-northwind-msa-2026`.
- Summary job id: `summary-contract-northwind-msa-2026`.
- Consent record id: `consent-legal-productivity-ai-2026-05-20`.
- Capability tier: `contract-lifecycle-core`.
- Capability tier: `intelligence-summary-core`.
- Subscribed microservices: `drive`, `intelligence`, `ontology`, `policy-engine`, `audit-chain`, `workflow-engine`, `mail`, `messenger`, `consent-graph`.
- Required Cedar permit: `drive.work-file.create`.
- Required Cedar permit: `drive.work-file.classify`.
- Required Cedar permit: `intelligence.summary.run`.
- Required Cedar permit: `intelligence.summary.export`.
- Required Cedar permit: `ontology.projection.read`.
- Required Cedar permit: `consent.record.evaluate`.
- Required Cedar permit: `audit.intelligence.read`.
- Data class: `Confidential`.
- Pack posture: `SOC2-Type-II`.
- Optional pack posture: `EU-GDPR-2018-baseline` if EU personal data is detected.
- Named saved query: `tutorial.contract_summary_status`.
- Review output file: `northwind-msa-summary-review-packet.pdf`.

## Step-by-Step

1. Open the legal review workspace.
   - Sign in as `valeria.ionescu@acme.example`.
   - Switch to tenant `Acme Robotics`.
   - Open workspace `Legal Vendor Review`.
   - Confirm header text: `Tenant context: tenant-acme-robotics`.
   - Confirm role badge: `Legal Counsel`.
   - Confirm capability tier badge: `contract-lifecycle-core`.
   - Screenshot checkpoint: capture workspace and tenant context.
   - Do not upload the contract from a personal tenant.
   - This tutorial assumes the contract is approved for training use.
   - Keep the activity panel open.

2. Upload the contract to Drive.
   - Click `Drive`.
   - Open folder `/Legal/Vendor Contracts/Northwind`.
   - Click `Upload`.
   - Select `northwind-master-services-agreement-200p.pdf`.
   - Confirm file size is below tenant limit.
   - Confirm page count: `200`.
   - Set document title: `Northwind Master Services Agreement 2026`.
   - Set contract id: `contract-northwind-msa-2026`.
   - Click `Upload file`.
   - Expected toast: `Contract uploaded`.

3. Classify the contract.
   - Select the uploaded PDF.
   - Click `Classify`.
   - Data class: `Confidential`.
   - Document type: `Master Services Agreement`.
   - Counterparty: `Northwind Design GmbH`.
   - Governing law field: `New York`.
   - Contains personal data: `Unknown - scan required`.
   - Retention: `legal-contract-7y`.
   - Click `Run classification`.
   - Expected result: `Confidential legal contract`.
   - Screenshot checkpoint: capture classification panel.

4. Run sensitive-data scan.
   - In the classification panel, click `Scan sensitive data`.
   - Scan profile: `legal-contract-scan-v2`.
   - Include PII detection: enabled.
   - Include PHI detection: disabled unless healthcare content appears.
   - Include secrets detection: enabled.
   - Include financial terms extraction: enabled.
   - Click `Start scan`.
   - Expected state: `Scan complete`.
   - Expected result: `No secrets detected`.
   - Expected PII result: `Limited business contact data`.
   - Screenshot checkpoint: capture scan result.

5. Confirm AI consent and purpose.
   - Click `Ask AI`.
   - The consent panel should open before any summary runs.
   - Purpose: `productivity_ai_summary`.
   - Consent record: `consent-legal-productivity-ai-2026-05-20`.
   - Scope: `work tenant documents only`.
   - Training use: `Off`.
   - Data retention for model input: `No training retention`.
   - Click `Confirm purpose`.
   - Expected badge: `Consent and policy satisfied`.
   - Screenshot checkpoint: capture the consent badge.

6. Choose summarization mode.
   - In the `Ask AI` panel, select `Summarize`.
   - Mode: `Legal contract review`.
   - Output type: `Executive summary with citations`.
   - Citation granularity: `Page and clause`.
   - Max summary length: `1,500 words`.
   - Risk extraction: enabled.
   - Obligations table: enabled.
   - Renewal and termination dates: enabled.
   - Click `Configure`.
   - Screenshot checkpoint: capture the summarization configuration.

7. Set review questions.
   - Add question: `What are the payment obligations?`
   - Add question: `What are the termination rights?`
   - Add question: `What indemnities are mutual and what indemnities are one-way?`
   - Add question: `Are there data processing terms that require GDPR review?`
   - Add question: `What operational SLAs are promised?`
   - Add question: `Which clauses need legal follow-up before signature?`
   - Set answer style: `Concise with citations`.
   - Click `Save questions`.
   - Expected toast: `Review questions saved`.
   - Screenshot checkpoint: capture the question list.
   - These questions guide the output but do not replace legal review.

8. Start the summary job.
   - Click `Run summary`.
   - Job id: `summary-contract-northwind-msa-2026`.
   - Confirm document: `contract-northwind-msa-2026`.
   - Confirm pages: `200`.
   - Confirm data class: `Confidential`.
   - Confirm tenant: `tenant-acme-robotics`.
   - Confirm training use: `Off`.
   - Click `Start`.
   - Expected state: `Summary running`.
   - Screenshot checkpoint: capture the running job state.
   - Keep the job panel open until it completes.

9. Monitor progress.
   - The progress bar should show `Extracting pages`.
   - Then it should show `Building citation map`.
   - Then it should show `Generating summary`.
   - Then it should show `Running policy checks`.
   - Expected completion time: under `PT10M` for the training fixture.
   - If progress stalls for more than 15 minutes, use the recovery section.
   - Expected final state: `Summary ready`.
   - Screenshot checkpoint: capture the completed state.
   - The job should not expose raw text to any personal tenant.
   - The job should emit audit events.

10. Inspect the executive summary.
    - Open the `Executive summary` tab.
    - Confirm every paragraph has page citations.
    - Confirm first citation format: `p. 3, Definitions`.
    - Confirm key terms include `services`, `fees`, `data processing`, `termination`, `liability`.
    - Confirm no paragraph says `citation unavailable`.
    - Screenshot checkpoint: capture the top of the summary.
    - Click citation `p. 3`.
    - Confirm the PDF opens to the cited page.
    - Close the citation preview.
    - Keep the summary in draft state.

11. Inspect obligations table.
    - Open `Obligations`.
    - Confirm columns: `party`, `obligation`, `deadline`, `citation`, `risk`.
    - Row example: `Acme`, `pay undisputed invoices`, `Net 30`, `p. 42`, `medium`.
    - Row example: `Northwind`, `maintain service availability`, `monthly`, `p. 87`, `high`.
    - Row example: `Both parties`, `protect confidential information`, `term plus 5 years`, `p. 112`, `high`.
    - Screenshot checkpoint: capture the obligations table.
    - Sort by `risk`.
    - Confirm high-risk obligations move to the top.
    - Export obligations as `northwind-msa-obligations.csv`.
    - Return to summary job.

12. Inspect risk findings.
    - Open `Risk findings`.
    - Confirm finding: `Liability cap excludes confidentiality breach`.
    - Confirm citation: `p. 118`.
    - Confirm finding: `Data processing addendum references EU subprocessors`.
    - Confirm citation: `p. 151`.
    - Confirm finding: `Auto-renewal requires 90-day notice`.
    - Confirm citation: `p. 73`.
    - Mark each finding status: `Needs counsel review`.
    - Screenshot checkpoint: capture risk findings with statuses.
    - Do not mark legal risks as resolved inside the AI panel.

13. Ask a follow-up question.
    - Open `Ask follow-up`.
    - Type: `Which clauses require privacy counsel before signature?`
    - Confirm purpose badge remains `productivity_ai_summary`.
    - Click `Ask`.
    - Expected answer cites privacy, DPA, subprocessors, transfer terms, and retention clauses.
    - Expected citations include pages around `p. 148-160`.
    - Screenshot checkpoint: capture the follow-up answer.
    - Click `Add to review packet`.
    - Confirm toast: `Answer added to packet`.
    - Follow-up answers inherit the original job audit id.

14. Redact the packet for non-legal stakeholders.
    - Click `Prepare packet`.
    - Packet audience: `Procurement and executive sponsor`.
    - Redact raw contract text: enabled.
    - Include executive summary: enabled.
    - Include obligations table: enabled.
    - Include risk findings: enabled.
    - Include citations: enabled.
    - Include AI limitations note: enabled.
    - Click `Generate packet`.
    - Expected packet name: `northwind-msa-summary-review-packet.pdf`.
    - Screenshot checkpoint: capture packet settings.

15. Send the packet for review.
    - Click `Send for review`.
    - Recipients: `procurement@acme.example`, `executive-sponsor@acme.example`.
    - Subject: `Northwind MSA AI-assisted review packet`.
    - Message: `Please review the cited findings before the legal sync.`
    - Permissions: `Can view`.
    - Expiration: `P14D`.
    - Click `Send`.
    - Expected toast: `Review packet sent`.
    - Expected Mail event: `MailWorkMessageSent`.
    - Screenshot checkpoint: capture the send confirmation.
    - The packet link should stay inside work tenant access.

16. Inspect audit and policy evidence.
    - Open the summary job `Audit` tab.
    - Confirm event: `IntelligenceSummaryRequested`.
    - Confirm event: `ConsentRecordEvaluated`.
    - Confirm event: `DocumentClassificationChecked`.
    - Confirm event: `IntelligenceSummaryGenerated`.
    - Confirm event: `SummaryPacketExported`.
    - Confirm model training flag: `false`.
    - Confirm tenant id: `tenant-acme-robotics`.
    - Screenshot checkpoint: capture audit evidence.
    - Export evidence as `summary-contract-northwind-msa-2026-audit.json`.

17. Run final verification query.
    - Open `Intelligence -> Saved checks`.
    - Choose `tutorial.contract_summary_status`.
    - Input `tenant_id=tenant-acme-robotics`.
    - Input `contract_id=contract-northwind-msa-2026`.
    - Input `summary_job_id=summary-contract-northwind-msa-2026`.
    - Input `consent_record_id=consent-legal-productivity-ai-2026-05-20`.
    - Click `Run`.
    - Expected title: `Contract summary complete`.
    - Expected state: `PASS`.
    - Screenshot checkpoint: capture the query output.
    - Save the query result next to the packet.
    - The tutorial is complete when the packet and audit export exist.

## Verification

- Named query: `tutorial.contract_summary_status`.
- Query location: `Intelligence -> Saved checks`.
- Query input `tenant_id`: `tenant-acme-robotics`.
- Query input `contract_id`: `contract-northwind-msa-2026`.
- Query input `summary_job_id`: `summary-contract-northwind-msa-2026`.
- Query input `consent_record_id`: `consent-legal-productivity-ai-2026-05-20`.
- Expected output field: `document_pages`.
- Expected output value: `200`.
- Expected output field: `data_class`.
- Expected output value: `Confidential`.
- Expected output field: `consent_purpose`.
- Expected output value: `productivity_ai_summary`.
- Expected output field: `training_use`.
- Expected output value: `false`.
- Expected output field: `summary_state`.
- Expected output value: `ready`.
- Expected output field: `citation_coverage`.
- Expected output value: `complete`.
- Expected output field: `obligations_exported`.
- Expected output value: `true`.
- Expected output field: `review_packet_exported`.
- Expected output value: `true`.
- Expected output field: `audit_exported`.
- Expected output value: `true`.
- Expected output field: `personal_tenant_reads`.
- Expected output value: `0`.
- Expected output field: `result_label`.
- Expected output value: `Contract summary complete`.
- CLI equivalent:

```bash
oya intelligence verify summary \
  --tenant tenant-acme-robotics \
  --contract contract-northwind-msa-2026 \
  --job summary-contract-northwind-msa-2026 \
  --consent consent-legal-productivity-ai-2026-05-20
```

- CLI expected line: `PASS tutorial.contract_summary_status`.
- CLI expected line: `document_pages=200`.
- CLI expected line: `citation_coverage=complete`.
- CLI expected line: `training_use=false`.
- Audit event to inspect: `IntelligenceSummaryRequested`.
- Audit event to inspect: `ConsentRecordEvaluated`.
- Audit event to inspect: `DocumentClassificationChecked`.
- Audit event to inspect: `IntelligenceSummaryGenerated`.
- Audit event to inspect: `SummaryPacketExported`.
- Evidence artifact: `northwind-msa-obligations.csv`.
- Evidence artifact: `northwind-msa-summary-review-packet.pdf`.
- Evidence artifact: `summary-contract-northwind-msa-2026-audit.json`.
- Dashboard: `Intelligence -> Tenant AI usage`.
- Expected tile: `No policy violations`.

## Common Pitfalls + Recovery

- Pitfall: the contract is uploaded from a personal tenant.
- Recovery: delete the personal upload and re-upload from `workspace-legal-vendor-review`.
- Pitfall: the document is not classified.
- Recovery: run classification before opening `Ask AI`.
- Pitfall: consent badge is missing.
- Recovery: verify `consent-legal-productivity-ai-2026-05-20` grants `productivity_ai_summary`.
- Pitfall: training use is enabled.
- Recovery: stop the job, disable training use, and rerun.
- Pitfall: citations are missing.
- Recovery: rerun in `Executive summary with citations` mode.
- Pitfall: the PDF page count is not 200.
- Recovery: use the training fixture `northwind-master-services-agreement-200p.pdf`.
- Pitfall: sensitive-data scan finds secrets.
- Recovery: stop the AI job and upload a redacted contract.
- Pitfall: risk findings are marked resolved by AI.
- Recovery: reset findings to `Needs counsel review`; legal counsel owns resolution.
- Pitfall: follow-up answer cites no pages.
- Recovery: ask again with `Concise with citations` enabled.
- Pitfall: packet includes raw contract text.
- Recovery: regenerate with `Redact raw contract text` enabled.
- Pitfall: packet is sent to a personal email.
- Recovery: revoke link and resend to work tenant recipients only.
- Pitfall: audit export lacks `ConsentRecordEvaluated`.
- Recovery: rerun the job after purpose confirmation.
- Pitfall: the job stalls.
- Recovery: cancel the job, inspect `IntelligenceSummaryRequested`, and rerun once service health is green.
- Pitfall: marketplace personalization consent is used.
- Recovery: switch purpose to `productivity_ai_summary`; marketplace consent is not valid for contract review.
- Pitfall: personal tenant reads appear in verification.
- Recovery: disable the job and inspect ADR-0311 boundary enforcement.
- Pitfall: stakeholders treat the summary as legal advice.
- Recovery: keep `AI limitations note` in the packet and route final sign-off to counsel.
- Pitfall: output omits obligations table.
- Recovery: enable `Obligations table` and regenerate.
- Pitfall: packet expiration is unlimited.
- Recovery: set expiration `P14D` for this tutorial.

## Contract Summary Review Packet

The review packet should let counsel validate the AI output without reopening the full contract.

- Packet title should be `Northwind MSA AI Summary Review`.
- Source document should be `northwind-master-services-agreement-200p.pdf`.
- Source document page count should be `200`.
- Summary job id should be `summary-contract-northwind-msa-2026`.
- Purpose token should be `productivity_ai_summary`.
- Model policy should be `enterprise-contract-summary-v1`.
- Output section `Executive summary` should cite pages.
- Output section `Key obligations` should cite pages.
- Output section `Renewal and termination` should cite pages.
- Output section `Risk flags` should cite pages.
- Output section `Open questions for counsel` should cite pages.
- Packet setting `Redact raw contract text` should be enabled.
- Packet setting `AI limitations note` should be enabled.
- Packet expiration should be `P14D`.
- Recipient list should contain only work tenant addresses.
- Audit event `IntelligenceSummaryRequested` should precede `SummaryPacketShared`.

The packet should not make legal decisions.

It should narrow review by pointing counsel to cited pages.

It should preserve human review as the final decision authority.

The tutorial is complete when counsel can open the packet and every risk flag has at least one page citation.

## Next Tutorials

- [Capture and propagate consent across microservices](consent-cascade-across-microservices.md).
- [Project Salesforce CRM data into Oyatie ontology](ontology-projection-from-external-source.md).
- [Activate HIPAA, SOC 2, GDPR, and KR-PIPA](multi-pack-tenant-activation.md).
- [Handle a GDPR erasure request](data-subject-erasure-request-handling.md).

## References

- [Intelligence as two-layer AI substrate ADR](../decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md).
- [Consumer intelligence substrate ADR](../decisions/ADR-0220-consumer-intelligence-substrate.md).
- [Consent cascade tutorial](consent-cascade-across-microservices.md).
- [Autonomy Ceiling Standard](../standards/autonomy-ceiling.md).
- [Capability Tier Over Product Fragmentation ADR](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
- [Audit evidence timeline design spec](../../specs/design-system/audit-evidence-timeline.json).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [Doc Style](../standards/doc-style.md).

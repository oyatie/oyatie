---
doc_class: Onboarding
microservice: contract-lifecycle-management
persona: legal-ops
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# contract-lifecycle-management — Legal Ops First Week

Audience: a Legal Operations professional joining a tenant's legal team responsible for contract templates, workflow design, e-signature delivery, and obligation tracking. You have admin access but no engineering background.

## Day 1 — orientation + access

Morning (3 h):

1. Receive `iam` invite. Cedar role `clm::legal-ops` binds `contract::*::{read,write}`, `contract::template::publish`, `contract::workflow::publish`, `contract::obligation::*`.
2. Log in to the CLM admin portal: `https://clm-admin.<tenant>.oyatie.io`.
3. Explore the dashboard: contracts in-flight by stage, recent activity, expiring/renewal contracts, obligation calendar.
4. Click "Contract types" — see the empty contract-type roster.

Afternoon (4 h):

5. Read the substrate primer: portal → Help → "CLM 101" (~ 45 min).
6. Watch the substrate walkthrough video (45 min).
7. List the contract types your tenant uses: NDAs, MSAs, SOWs, employment offers, vendor agreements, customer order forms, DPAs (Data Processing Agreements), partnership agreements, settlement agreements, etc.
8. Inventory existing template DOCX files (likely in a shared drive). Estimate the migration scope.

End of Day 1 deliverable: contract-type inventory committed to `inventory/clm-types.md`.

## Day 2 — first template

Morning (4 h):

1. Pick the simplest contract type to start (typically a one-way NDA).
2. Open Templates → "New template" → upload your existing NDA DOCX.
3. The template editor parses the DOCX and highlights placeholder candidates (text in `[brackets]`, `{{handlebars}}`, or `__underlines__`).
4. For each placeholder, bind to a contract field: e.g. `[Counterparty Name]` → `counterparty.legal_name`; `[Effective Date]` → `effective_date`; `[Counterparty Address]` → `counterparty.registered_address`.
5. Save the template as a draft.

Afternoon (3 h):

6. Define the contract type's data model: portal → Contract Types → "New type" → "Unilateral NDA". Fields: `counterparty.legal_name` (text, required), `counterparty.registered_address` (multi-line text), `effective_date` (date, default today), `term_months` (integer, default 24), `governing_law` (dropdown: NY / DE / EU / KR-Seoul / JP-Tokyo / SG / UK-England-Wales).
7. Bind the template to the contract type.
8. Test render: click "Test render" with sample data. The portal returns the populated DOCX + PDF preview side-by-side. Verify all placeholders bind correctly.

End of Day 2 deliverable: 1 contract type + 1 template + sample render passing visual inspection.

## Day 3 — workflow (draft → approve → send → sign)

Morning (4 h):

1. Open Workflows → "New workflow" → "Standard NDA workflow".
2. Configure stages:
   - Draft: any user with `contract::create` can initiate.
   - Review: assign to the contract's `requester.manager` (auto-pulled from IAM) + tenant Legal team.
   - Approve: ≥ 1 approver from the Legal team.
   - Send for signature: e-signature gateway delivery to the counterparty's `counterparty.signer_email`.
   - Signed: collected when the counterparty signs; auto-emit `contract.signed` event.
3. Configure parallel review for fast-track contracts: if `contract_value_usd < 25 000`, skip the Review stage and go straight to Approve.

Afternoon (3 h):

4. Configure e-signature settings: portal → E-Signature → "Default provider". Choose: DocuSign (most flexible), Adobe Sign (Adobe Creative tenants), HelloSign (small-business), OneSpan (regulated industries), or native QES (EU eIDAS Art. 28).
5. For the NDA workflow, choose DocuSign with AES signature class.
6. Test the workflow: click "Initiate test contract" with sample data. Follow the stages — approve as yourself, send to a test email (e.g. `signature-test+nda@<your-domain>`).
7. Click the email link, complete the signature flow, confirm the contract reaches "Signed" state in the portal.

End of Day 3 deliverable: workflow live + end-to-end test signature received + state advance to Signed.

## Day 4 — obligation tracking + AI redlining

Morning (3 h):

1. Open the signed contract from Day 3. Click "Extract obligations" — the AI extracts obligations from the contract body (e.g. "Confidentiality survives for 5 years post-termination", "Notice of breach within 10 business days").
2. Review the extracted obligations — accept correct ones; correct or delete incorrect ones; add any the AI missed.
3. For each obligation, set the responsible party (your tenant / counterparty) and the trigger date.
4. The obligation tracker shows upcoming obligations on the calendar; reminders dispatch via the `notifications` µservice.

Afternoon (4 h):

5. Test AI redlining on a new contract: portal → Contracts → "New from external" → upload a counterparty-provided contract (e.g. a vendor MSA).
6. Click "AI-redline" — the system highlights clauses that differ from your tenant's standard template + adds suggested edits with rationale ("This indemnification clause is missing a cap on aggregate liability — suggest adding 'Notwithstanding the foregoing, Vendor's total liability under this Agreement shall not exceed Fees paid in the 12 months preceding the claim.'").
7. Accept, reject, or modify each suggestion. The portal tracks every change with author + timestamp for audit purposes.

End of Day 4 deliverable: obligation tracker populated + AI redline test green on a real counterparty contract.

## Day 5 — pack overlays + sign-off

Morning (4 h):

1. Determine which packs your tenant is on (KR-PIPA, GDPR, HIPAA-Provider, etc).
2. Read the pack overlay reference: portal → Help → "Pack overlays for CLM".
3. For each pack, configure the overlay:
   - KR-PIPA: contract retention ≥ 5 y; QES required for contracts > KRW 30M; KR-Seoul jurisdiction allowed.
   - GDPR: DPA template auto-attached to any contract involving personal-data processing; right-to-erasure exclusions documented.
   - HIPAA-Provider: BAA template auto-attached to any contract with PHI access; signed contracts retained 6 y minimum from termination date.
4. Verify overlays apply: initiate a test contract with a counterparty whose tenant is on the relevant pack; confirm the overlay-mandated templates auto-attach.

Afternoon (4 h):

5. Document your tenant's CLM playbook: template inventory, workflow descriptions, e-signature provider choice, pack overlays, escalation paths.
6. Set up reporting: Reports → Templates → schedule daily/weekly summaries to your team.
7. Brief the substrate team on your tenant's onboarding completion; receive escalation channel info.

End of Week 1 deliverable: 1+ contract type live, workflow tested end-to-end, obligation tracker populated, pack overlays configured, playbook documented.

## What you should know by end of week 1

- Template authoring + field binding (OOXML + JSPath).
- Workflow design (stages, approvers, conditionals, parallel paths).
- E-signature provider selection + signature-class semantics (AES vs QES).
- Obligation extraction + tracking + reminder delivery.
- AI redlining (suggestion, accept/reject, rationale).
- Pack overlays + automatic compliance attachments.

## What you should NOT do in week 1

- Don't publish a template without test-rendering it with sample data first.
- Don't bypass the approval workflow for high-value contracts (substrate enforces approver Cedar permits).
- Don't disable obligation tracking — even if you don't actively use it, it's the audit-trail substrate for renewals + breach evidence.
- Don't downgrade signature class (e.g. AES → simple electronic) without a legal review; pack overlays mandate signature class for certain contract types.
- Don't share contract templates outside your tenant (Cedar enforces template-read permission per-tenant by default; you would have to explicitly grant cross-tenant which is logged + flagged).

---
doc_class: FAQ
microservice: contract-lifecycle-management
persona: legal-ops
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# contract-lifecycle-management — Legal Ops FAQ

## Q1: What's the difference between AES (Advanced Electronic Signature) and QES (Qualified Electronic Signature)?

AES under eIDAS Art. 26: "linked uniquely to the signatory, capable of identifying the signatory, created using data the signatory can use under their sole control, linked to the data such that any subsequent change is detectable." Practically: DocuSign, Adobe Sign, HelloSign-style click-to-sign with audit trail.

QES under eIDAS Art. 28: AES + created by a Qualified Signature Creation Device (QSCD) + based on a qualified certificate from a Qualified Trust Service Provider (QTSP) on the EU Trust List. Practically: signing requires the signatory's hardware token (smartcard or USB device) OR a remote QES service that holds the signatory's key in a QSCD-certified HSM. Legally equivalent to a handwritten signature across the entire EU under Art. 25(2) of eIDAS.

When to use QES: any contract where you might litigate cross-border in the EU AND a counterparty might dispute the signature (e.g. large commercial agreements, employment terminations, real estate, government contracts). When AES is sufficient: NDAs, internal HR documents, low-value vendor agreements.

## Q2: Can I use my existing DocuSign account or do I have to use oyatie's bundled DocuSign?

You can use your own DocuSign account. Portal → E-Signature → "Bring your own DocuSign". Provide: DocuSign integrator key, RSA private key for JWT auth, base URL (`demo.docusign.net` for testing, `na3.docusign.net` for US production). The substrate proxies calls through your account, so signatures show your tenant's branding + appear in your DocuSign reporting. The bundled DocuSign at retired-standard+ is convenient for tenants who don't have their own account; if you already pay DocuSign separately, bring your own and skip the bundled cost.

## Q3: How does the AI redlining work? Will it leak my contracts to a cloud provider?

By default at retired-standard tier: AI runs on the local Llama-3.1-8B-Instruct model (fine-tuned on Legal-Pile-v2 + your tenant's prior contracts after explicit consent). No contract content leaves the tenant boundary. At retired-advanced tier: same local Llama-3.1-70B-Instruct + optionally cross-emit to Claude-3.7-Sonnet via the `intelligence` µservice (Anthropic API). The cross-emit is OFF by default; you opt in per-contract-type. When ON, the contract body is sent to Anthropic over an enterprise-grade contract with zero-retention; per Anthropic's enterprise terms, prompts are NOT used for training and are deleted from Anthropic's systems within 24 h. At retired-sovereign tier: only the local in-pack model; cross-emit is disabled by construction for sovereign-residency reasons.

## Q4: How do I handle a counterparty who refuses to use e-signature and insists on wet-ink?

Workflow → branch on `signature_modality`. If counterparty selects wet-ink: (a) the contract advances to "Out for Wet-Ink Signature"; (b) the substrate generates a PDF + cover sheet + return instructions; (c) you mail the package; (d) counterparty signs + returns; (e) you scan the signed package and upload via portal → "Confirm wet-ink signature" → attach scanned PDF + counterparty's signing date + your witness (if required by jurisdiction); (f) the substrate emits the same `contract.signed` event + cross-emits a Merkle-anchor to `audit-chain`. The signed-contract retention is identical to e-signed (both go to SeaweedFS WORM at retired-standard+).

## Q5: A regulator (e.g. EU DPA, US FTC, KR PIPC) demands all contracts with party X over the last 3 years. How do I produce them?

Portal → Reports → "Regulator evidence export". Filters: party_name = X (uses fuzzy match + manual confirmation), signature_state = "signed", signed_date BETWEEN <range>. Export format: ZIP containing (a) original-signed PDFs, (b) signature audit trails from the e-signature provider (DocuSign Certificate of Completion, Adobe Sign Audit Report, etc), (c) audit-chain Merkle proofs for each contract's lifecycle, (d) chain-of-custody log of every access. The auditor can verify the Merkle proofs independently against the audit-chain public key.

## Q6: What's the obligation extraction accuracy? Will it miss critical obligations?

Internal accuracy benchmark on a 1,000-contract test set (mix of NDA, MSA, SOW, employment, vendor): precision 94.2 %, recall 91.7 % vs human-attorney baseline. The "missed" obligations (8.3 % recall gap) are typically: (a) implicit obligations embedded in defined terms ("Confidential Information shall be returned upon termination" — the AI flags it but sometimes misses the cross-reference to the definitions section), (b) renewal-clauses with non-standard phrasing ("This Agreement shall automatically extend for additional twelve-month periods unless either party..." — the AI catches the renewal but sometimes mis-calculates the notice window). Per our deployment guidance, always run a human review on the extracted obligation set before considering it canonical, especially for high-value contracts.

## Q7: Can I version contract templates? What happens to in-flight contracts when I publish a new template version?

Yes. Every template has a semantic version (`v1.2.3`). When you publish `v1.3.0`, in-flight contracts that haven't yet been signed CONTINUE on `v1.2.3` (the version captured at contract-create time). New contracts initiated after the publish use `v1.3.0`. You can FORCE in-flight contracts to upgrade by clicking "Migrate to latest template" on each — this triggers a re-render with new template + a clear audit-log entry. For signed contracts: the version at sign-time is captured in the signature evidence; subsequent template changes don't affect signed contracts.

## Q8: How do I integrate with my CRM (Salesforce, HubSpot, oyatie crm) to pull party data?

Portal → Integrations → CRM connector. Configure the source (Salesforce / HubSpot / Pipedrive / oyatie crm). Map CRM fields to contract fields: Salesforce `Account.Name` → contract `counterparty.legal_name`; `Account.BillingAddress` → contract `counterparty.registered_address`. When a sales rep initiates a contract from an Opportunity, the CRM pushes the party data + the substrate prefills the contract draft. For oyatie's own crm µservice, the integration is native (no configuration needed; permission via Cedar `crm::account::read`).

## Q9: My pack is HIPAA-Provider. Are there special requirements for contracts involving PHI?

Yes. The HIPAA pack overlay enforces:
- Any contract whose `data_classes` field includes `phi` MUST attach a Business Associate Agreement (BAA) template before reaching "Out for Signature" state.
- Signed BAAs retained 6 y from contract termination per HIPAA §164.530(j)(2).
- The signed contract + signature evidence retained in WORM Compliance mode.
- The BAA template is shipped with the pack; you can customise but cannot remove the required BAA-pack clauses (e.g. Subcontractor BAA flow-down per §164.504(e)(5)).
- Obligation tracker automatically populates breach-notification obligations per §164.410.

## Q10: A counterparty wants me to use their CLM system (e.g. Ironclad) instead of mine. How do we negotiate?

Cross-CLM negotiation is a known pain point. Three options:
1. **Email/PDF exchange**: download the contract draft as a PDF, attach to email, counterparty redlines in Word/Acrobat, you upload the redlined version, repeat. Slow but universal. The substrate tracks each version as a separate revision; you can run AI-redline against each new counterparty version.
2. **CLM-Bridge protocol** (CLMA standard): both CLMs implement the CLM-Bridge protocol (draft RFC, expected 2027). The contract negotiates between the two CLMs natively. oyatie supports the draft; Ironclad doesn't yet. Watch the standardisation.
3. **DocuSign Negotiate (now retired) or Adobe Document Cloud**: use a neutral negotiation tool. Less integrated but works for the negotiation phase; once finalised, you bring the signed PDF into your CLM.

In practice, option 1 is the default. Plan an extra 2-3 weeks of negotiation time for high-value contracts with sophisticated counterparties.

## Q11: How do I handle contracts in non-English languages?

The substrate is multi-lingual:
- Templates can be in any UTF-8 language; the OOXML format supports RTL (Arabic, Hebrew), CJK (Korean, Japanese, Chinese), and complex scripts (Devanagari, Thai).
- AI redlining: at retired-standard, Llama-3.1-8B is English-primary; for non-English you cross-emit to Claude-3.7-Sonnet which is multilingual. At retired-advanced+, the local Llama-3.1-70B is multilingual at adequate quality for redlining English / French / German / Spanish / Portuguese / Italian / Korean / Japanese / Chinese. Other languages: cross-emit to Claude or use the cross-emit-disabled fallback (no AI redline, human-only).
- Obligation extraction: same as redlining — Llama-70B handles the top-10 languages adequately; rare languages need a human pass.
- E-signature: DocuSign + Adobe Sign + native QES support 40+ languages for the signing UI.

## Q12: What's the failure mode if e-signature provider (e.g. DocuSign) has an outage?

The substrate detects provider outages within 60 s (synthetic-call probe). If your tenant has only DocuSign configured: in-flight signature requests queue; new requests fail with a clear error message; signed contracts already in our store are unaffected. If you have multiple providers configured: the substrate auto-routes new requests to the secondary provider (e.g. Adobe Sign). For high-availability tenants, configure multi-provider routing at retired-advanced+ tier — DocuSign primary, Adobe Sign secondary, native QES tertiary. The substrate honours signature-class requirements when routing (won't downgrade AES to simple electronic without your consent).

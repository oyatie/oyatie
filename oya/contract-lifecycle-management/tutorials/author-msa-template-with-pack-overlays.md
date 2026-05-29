---
doc_class: Tutorial
microservice: contract-lifecycle-management
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Tutorial — Author an MSA template with GDPR + HIPAA + KR-PIPA pack overlays

Goal: author a Master Service Agreement (MSA) template that adapts based on the counterparty's pack. The template should:
- Default to a standard US/DE governing-law MSA.
- Auto-attach a DPA (Data Processing Agreement) when the counterparty processes EU personal data.
- Auto-attach a BAA (Business Associate Agreement) when the counterparty handles PHI.
- Auto-attach KR-PIPA-specific data-export clauses when the counterparty is in Korea + processes Korean personal data.
- Enforce QES signature class when contract value > €25 000 and counterparty is in the EU.

Prereqs:

- `clm::legal-ops` Cedar role.
- retired-standard tier or higher (pack overlays require retired-standard's pack-conditional engine).
- A baseline MSA DOCX from your legal team (we'll use a generic 12-page MSA in this walkthrough).
- ~ 3 hours.

## Step 1 — upload the baseline DOCX

Portal → Templates → "New template" → upload `MSA-baseline-v2.docx`. The editor parses it and detects 47 placeholder candidates (text in `[brackets]`, `{{handlebars}}`, and `__underscores__`).

Review the auto-detected placeholders and confirm the field binding. Common ones:

| Placeholder text in DOCX | Bind to field |
|---|---|
| `[Customer Legal Name]` | `customer.legal_name` |
| `[Customer Address]` | `customer.registered_address` |
| `[Effective Date]` | `effective_date` |
| `[Service Description]` | `services.description` |
| `[Service Fees]` | `services.fees_display` |
| `[Term Months]` | `term_months` |
| `[Governing Law]` | `governing_law` |
| `[Jurisdiction]` | `jurisdiction` |

Click "Save draft". The template now has bound fields but no overlays yet.

## Step 2 — define the contract data model

Portal → Contract Types → "New type" → "Master Service Agreement".

Required fields:
- `customer` (object): `legal_name` (text), `registered_address` (multi-line), `country_code` (ISO 3166-1 alpha-2), `data_classes` (multi-select: `personal-data-eu`, `personal-data-kr`, `phi-us`, `pci`, `none`)
- `services.description` (multi-line, required)
- `services.fees_eur` (decimal, required)
- `effective_date` (date, default today)
- `term_months` (integer, default 36)
- `governing_law` (dropdown: `NY`, `DE-Berlin`, `KR-Seoul`, `JP-Tokyo`, `SG`, `UK-England-Wales`)
- `jurisdiction` (auto-derived from governing_law)
- `signature_class` (auto-derived: see Step 5)

Validation rules:
- `services.fees_eur > 0`
- `term_months IN [12, 24, 36, 48, 60]`
- `customer.country_code MUST match jurisdiction's permitted-customer-country set` (e.g. governing_law=DE-Berlin can have customer in any EU member state)

## Step 3 — bind the base template to the contract type

Templates → MSA-baseline-v2 → "Bind to contract type" → "Master Service Agreement".

## Step 4 — add the DPA overlay

Conditional attachment for GDPR-applicable contracts.

Templates → "New overlay" → "GDPR Data Processing Agreement":
- Upload `DPA-GDPR-v3.docx` (your tenant's DPA template).
- Bind fields: `customer.legal_name`, `customer.registered_address`, `customer.dpo_email`, `customer.transfer_mechanism` (dropdown: `SCC-2021/914`, `BCR`, `adequacy-decision`, `not-applicable`).
- Attach condition: `'personal-data-eu' IN customer.data_classes`.
- Position in final document: appendix after the base MSA.

Now any MSA where the customer's `data_classes` includes `personal-data-eu` will auto-attach the DPA when generated.

## Step 5 — add the BAA overlay

Conditional attachment for HIPAA-Provider customers.

Templates → "New overlay" → "HIPAA Business Associate Agreement":
- Upload `BAA-HIPAA-v2.docx`.
- Bind fields: `customer.legal_name`, `customer.privacy_officer_email`, `customer.subcontractor_flowdown_required` (boolean).
- Attach condition: `'phi-us' IN customer.data_classes`.
- Position: appendix after DPA (if present), else after base MSA.

## Step 6 — add the KR-PIPA overlay

Conditional attachment for Korean customers processing Korean personal data.

Templates → "New overlay" → "KR-PIPA cross-border transfer disclosure":
- Upload `KR-PIPA-overseas-transfer-v1.docx` (Korean-language disclosure per KR-PIPA Art. 28-8).
- Bind fields: `customer.korean_data_protection_officer`, `customer.korean_user_consent_evidence_id`.
- Attach condition: `'personal-data-kr' IN customer.data_classes`.
- Position: appendix at the end.

## Step 7 — configure signature-class derivation

Portal → Contract Types → "Master Service Agreement" → "Signature class rules":

```
IF (services.fees_eur > 25000) AND (jurisdiction IN ['DE-Berlin', 'FR-Paris', 'IT-Rome', 'ES-Madrid', 'NL-Amsterdam']):
    signature_class = "QES"  # eIDAS Art. 28 qualified
ELSE IF (services.fees_eur > 10000):
    signature_class = "AES"  # eIDAS Art. 26 advanced
ELSE:
    signature_class = "SES"  # simple electronic; click-to-sign
```

Save the rules. The substrate evaluates them at contract-create time and routes the signature request to the appropriate e-signature provider.

## Step 8 — test render with sample data

Portal → Templates → "Test render" → fill in test data:

**Test case 1: US customer, no special data**
```json
{
  "customer": {
    "legal_name": "Acme Corp",
    "registered_address": "123 Main St, San Francisco, CA 94102, USA",
    "country_code": "US",
    "data_classes": ["none"]
  },
  "services": {
    "description": "Cloud computing services per Order Form #1",
    "fees_eur": 60000
  },
  "effective_date": "2026-06-01",
  "term_months": 36,
  "governing_law": "NY"
}
```

Expected render: base MSA only, no overlays. Signature class: AES.

**Test case 2: EU customer with personal data, high value**
```json
{
  "customer": {
    "legal_name": "Beispiel GmbH",
    "registered_address": "Friedrichstraße 1, 10117 Berlin, Germany",
    "country_code": "DE",
    "data_classes": ["personal-data-eu"]
  },
  "services": {
    "description": "Cloud computing services per Order Form #1",
    "fees_eur": 60000
  },
  "effective_date": "2026-06-01",
  "term_months": 36,
  "governing_law": "DE-Berlin"
}
```

Expected: base MSA + DPA appendix. Signature class: QES.

**Test case 3: US healthcare customer with PHI**
```json
{
  "customer": {
    "legal_name": "MedCenter LLC",
    "registered_address": "456 Hospital Blvd, Cleveland, OH 44106, USA",
    "country_code": "US",
    "data_classes": ["phi-us"]
  },
  "services": {
    "description": "Cloud computing services for ePHI",
    "fees_eur": 80000
  },
  "effective_date": "2026-06-01",
  "term_months": 36,
  "governing_law": "NY"
}
```

Expected: base MSA + BAA appendix. Signature class: AES.

**Test case 4: Korean customer with Korean personal data**
```json
{
  "customer": {
    "legal_name": "한국기업 주식회사",
    "registered_address": "서울특별시 강남구 테헤란로 123",
    "country_code": "KR",
    "data_classes": ["personal-data-kr"]
  },
  "services": {
    "description": "클라우드 서비스",
    "fees_eur": 45000
  },
  "effective_date": "2026-06-01",
  "term_months": 36,
  "governing_law": "KR-Seoul"
}
```

Expected: base MSA + KR-PIPA appendix. Signature class: AES (Korean 전자서명법 equivalent to AES; QES is EU-specific).

## Step 9 — publish + workflow integration

Once all 4 test cases render correctly:

Portal → Templates → "Publish v1.0.0".

The template is now live. Any contract initiated under the "Master Service Agreement" type uses this template. Existing in-flight contracts continue on their prior version.

Configure the standard MSA workflow:

Portal → Workflows → "New workflow" → "MSA standard":
- Draft → Review (Legal) → Approve (Legal Lead) → Send for Signature (signature_class determined per Step 7) → Signed → Effective
- Parallel branch: if `services.fees_eur > 250000` → require additional VP-Legal approval before Send.
- Conditional notification: if HIPAA-BAA attached → email the Privacy Officer with attached BAA copy at "Signed" event.

## Step 10 — compliance evidence

Verify the substrate is emitting the right evidence:

```sh
oya audit-chain query --tenant <your-tenant> \
    --event-class contract::template::published \
    --since "1 hour ago"

oya audit-chain query --tenant <your-tenant> \
    --event-class contract::overlay::attached \
    --since "1 hour ago"
```

Each event includes the template version, overlay rules evaluated, signature class derived, and the cryptographic anchor for the contract content + appendices. This evidence is what a regulator audits when asking "how do you ensure DPA is attached every time a counterparty processes EU personal data?" — you produce the audit-chain query showing 100 % attachment for EU customers, with overlay attribution.

## What you've built

A production-ready MSA template with:
- 47 bound placeholder fields.
- 3 conditional overlays (DPA, BAA, KR-PIPA disclosure).
- Auto-derived signature class (SES / AES / QES) based on contract value + jurisdiction.
- Test-render validation across 4 representative scenarios.
- Audit-chain evidence for every overlay attachment + template publish.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| DOCX placeholders that don't match field names | Use the "Auto-detect placeholders" feature; verify each binding before publish |
| Overlap between DPA and BAA on identical clauses | Order the overlays so DPA comes first; the substrate de-duplicates identical clauses across attachments |
| Forgetting to bind a placeholder | The render fails at publish-time with a clear "unbound placeholder" error |
| Pack overlay rule that fires incorrectly | Run the 4 representative test cases before publish; verify each overlay attaches exactly when expected |
| QES routing failing for a non-EU customer | The signature-class derivation rule explicitly checks jurisdiction; verify the rule before publish |

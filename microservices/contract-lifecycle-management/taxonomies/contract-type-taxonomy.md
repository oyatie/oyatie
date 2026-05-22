---
doc_class: Taxonomy
microservice: contract-lifecycle-management
dimension_id: S-005
related_packs: [hipaa-baa, sox-404, sec-17a-4, gdpr]
date: 2026-05-21
---

# Contract Type Taxonomy

Canonical contract types recognized by CLM. Each type triggers specific approval routing (per `legal-dimensions/approval-routing-matrix.md`), retention policy (per `legal-dimensions/retention-overlay-by-contract-type.md`), and pack-overlay activation.

## Canonical types

| Type | Description | Default approval | Default retention |
|---|---|---|---|
| `MSA` | Master Service Agreement | Per value tier + Legal | 7y |
| `SOW` | Statement of Work under MSA | Contracts Mgr + Procurement | 7y or MSA-bound |
| `NDA-Unilateral` | One-way NDA | Legal | Term + 5y |
| `NDA-Mutual` | Two-way NDA | Legal | Term + 5y |
| `DPA` | Data Processing Addendum (GDPR Art. 28) | Legal + DPO | Term + 3y |
| `BAA` | HIPAA Business Associate Agreement | Legal + HIPAA Privacy Officer | 6y |
| `SaaS-Subscription` | SaaS subscription agreement | Per value tier + Legal | 7y |
| `Reseller-Channel` | Reseller or channel partner agreement | Per value tier + Legal | 7y |
| `License-Software` | Software license | Legal | License term + 5y |
| `License-IP` | IP license (patent, copyright, trademark) | Legal + IP counsel | License term + 5y |
| `Settlement-Agreement` | Litigation settlement | General Counsel | Permanent |
| `Employment-Agreement` | Employment contract | HR Director + Legal | Tenure + 7y |
| `Independent-Contractor` | Independent contractor agreement | HR Director + Legal | Tenure + 7y |
| `IP-Assignment` | IP assignment (from employee, contractor, or counterparty) | Legal + IP counsel | Permanent |
| `M&A-SPA` | M&A Stock Purchase Agreement | Full executive + Board | Permanent |
| `M&A-APA` | M&A Asset Purchase Agreement | Full executive + Board | Permanent |
| `Real-Estate-Lease` | Commercial real estate lease | Real Estate Committee + Legal | Term + 10y |
| `Real-Estate-Purchase` | Real estate purchase | Real Estate Committee + Board | Permanent |
| `Government-Contract` | US federal / state / local govt contract | Gov't Affairs + Compliance + Legal | Permanent |
| `Government-Subcontract` | Subcontract under prime gov't contract | Same as gov't contract | Permanent |
| `Procurement-PO` | Purchase order | Procurement | 7y |
| `Vendor-Agreement` | Vendor / supplier agreement | Per value tier + Procurement | 7y |
| `Joint-Venture` | JV agreement | Legal + executive | Permanent |
| `Partnership-Agreement` | Partnership agreement | Legal + executive | Permanent |
| `Consulting-Agreement` | Professional services consulting | Legal | Term + 7y |
| `Sponsorship-Agreement` | Marketing or event sponsorship | Marketing + Legal | Term + 5y |
| `Affiliate-Agreement` | Affiliate marketing | Marketing + Legal | Term + 5y |
| `Loan-Agreement` | Commercial loan | CFO + Treasury + Legal | Term + 7y |
| `Promissory-Note` | Promissory note | CFO + Treasury + Legal | Until paid + 7y |
| `Lease-Equipment` | Equipment lease | Procurement + Finance | Term + 5y |
| `Distribution-Agreement` | Distribution / dealer | Per value tier + Legal | Term + 7y |
| `OEM-Agreement` | OEM / private-label | Per value tier + Legal | Term + 7y |
| `Insurance-Policy` | Insurance policy | Risk Manager + Legal | Term + 10y |
| `Surety-Bond` | Surety bond | CFO + Treasury | Term + 10y |
| `Settlement-Release` | Release of claims | General Counsel | Permanent |
| `Confidentiality-Agreement` | Standalone confidentiality | Legal | Term + 5y |
| `Non-Compete` | Non-compete agreement | Legal + HR (if employee) | Restriction period + 7y |
| `Non-Solicitation` | Non-solicitation agreement | Legal + HR (if employee) | Restriction period + 7y |
| `Endorsement-Agreement` | Athlete/celebrity endorsement | Marketing + Legal | Term + 5y |
| `Sponsored-Research` | University / lab sponsored research | Legal + R&D | Term + 7y |
| `Clinical-Trial-Agreement` | Clinical trial agreement (CTA) | Legal + Regulatory + Compliance | Term + 25y (FDA) |
| `Material-Transfer-Agreement` | MTA for biological / chemical samples | Legal + Compliance | Term + 25y |
| `Subscription-Trial` | Trial / pilot subscription | Contracts Mgr | Term + 1y |
| `Subscription-Free-Tier` | Free-tier ToS | Auto-approved | Per tenant retention policy |
| `Click-Through-ToS` | Click-through terms of service | Auto-approved | Per tenant retention policy |
| `Privacy-Policy` | Privacy policy (not contract per se, but tracked) | Legal + DPO | Permanent |
| `User-Agreement-Consumer` | Consumer terms-of-service | Legal | Per ESIGN consumer disclosure |
| `Marketing-Agreement` | Marketing services | Marketing + Legal | Term + 5y |
| `Sales-Commission-Agreement` | Sales commission | HR + Finance + Legal | Tenure + 7y |
| `Manufacturing-Agreement` | Contract manufacturing | Procurement + Legal | Term + 10y |
| `Quality-Agreement` | Quality + supply quality | Procurement + Quality + Legal | Term + 10y |
| `Pharmaceutical-Supply` | Pharmaceutical supply | Procurement + Regulatory + Legal | Term + 25y (FDA) |
| `Construction-Contract` | Construction prime / subcontract | Project + Legal | Term + 10y |
| `Marriage-Settlement` (where electronic permitted) | Domestic | Specialty | Permanent |
| `Estate-Plan` (where electronic permitted) | Wills, trusts | Specialty | Permanent |

## Type-driven pack activation

| Type | Auto-activates pack |
|---|---|
| `BAA` | `hipaa-baa` |
| `DPA` | `gdpr` (when EU); `kr-pipa` (when KR) |
| `Government-Contract` | `sox-404` (if tenant is SOX-registrant); FAR overlay |
| `Clinical-Trial-Agreement` | `hipaa-baa` (typically); FDA 21 CFR overlay |
| `Subscription-Trial` | `tenant_class=demo_trial` defaults applied |
| `Loan-Agreement` (for broker-dealer tenants) | `sec-17a-4` |

## Type-driven jurisdiction overlay

The contract type combined with the counterparty's jurisdiction determines the applicable jurisdiction pack:

- `Real-Estate-Lease` + KR → KR Civil Code + 부동산임대차보호법.
- `Employment-Agreement` + CA → CA Labor Code overlay.
- `Consumer-Credit` + US → Truth in Lending Act overlay.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ContractCreate",
  resource is Contract
) when {
  resource.contract_type == "BAA" &&
  resource.tenant.active_packs.contains("hipaa-baa") == false
};

forbid (
  principal,
  action == Action::"ContractExecute",
  resource is Contract
) when {
  resource.contract_type in ["Clinical-Trial-Agreement", "Pharmaceutical-Supply"] &&
  resource.tenant.regulatory_signoff_obtained == false
};
```

## Audit events

- `oya.contract.lifecycle.management.contract.type_classified`
- `oya.contract.lifecycle.management.contract.type_misclassified` (after human review)
- `oya.contract.lifecycle.management.contract.type_triggered_pack`

## Standards references

- IACCM / WorldCC Contract Lifecycle Standard.
- ABA Section of Business Law Model Forms.
- FAR (Federal Acquisition Regulation).
- FDA 21 CFR Parts 11 + 312.
- AICPA Audit Risk Alert: Contracts.

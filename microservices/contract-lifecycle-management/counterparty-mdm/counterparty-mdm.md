---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-012 + Q-008
authoritative_source: ISO 17442 (LEI) + UCC Article 9-503 + FATF guidance
related_packs: [gdpr, kr-pipa, sox-404]
date: 2026-05-21
---

# Counterparty Master Data Management

CLM contracts have at least two parties (the tenant and the counterparty). Counterparty resolution — establishing the legal-entity identity of a counterparty across signature events, name changes, mergers, acquisitions, dissolutions — is a legal-grade MDM problem because contract enforcement depends on correctly identifying the obligor.

## Counterparty identity model

```
Counterparty {
  counterparty_id: UUIDv7,                          // immutable internal identifier
  tenant_id: TenantId,                              // tenant-scoped (per ADR-0244)
  legal_name_current: LegalName,                    // current full legal name
  legal_name_history: [LegalNameChange],            // chronological renames
  lei: LEI?,                                        // ISO 17442 Legal Entity Identifier
  ein: EIN?,                                        // US Employer Identification Number
  vat_id: VATIdentifier?,                           // EU VAT
  registration_jurisdiction: CountryCode,           // jurisdiction of incorporation
  registration_id: String,                          // company registry ID (e.g. Delaware file number, Companies House number, 사업자등록번호)
  registration_state_or_subdivision: String?,       // e.g. "DE" for Delaware
  entity_type: EntityType,                          // LLC, Inc, GmbH, S.A., 株式会社, 주식회사, etc.
  formation_date: Date,
  dissolution_date: Date?,
  parent_counterparty_id: CounterpartyId?,          // ownership tree
  predecessor_counterparty_ids: [CounterpartyId],   // merger / acquisition trail
  successor_counterparty_id: CounterpartyId?,       // forward link if acquired
  signatory_authorities: [SignatoryAuthority],
  addresses: [Address],
  contact_methods: [ContactMethod],
  sanctions_check_state: SanctionsCheckState,       // OFAC + UN + EU + UK + per-tenant
  ofac_sdn_check_at: Timestamp<RFC3339>,
  data_class: DataClassification,                   // PII_QUASI for individuals; PUBLIC_RECORD for entities
  audit_event_id: AuditEventId,
}

enum EntityType {
  USCorporation,       // Inc
  USLLC,
  USPartnership,
  USSCorp,
  USCorpDelaware,      // common subtype
  UKLimited,           // Ltd
  UKPLC,
  UKLLP,
  GermanGmbH,
  GermanAG,
  FrenchSARL,
  FrenchSA,
  KoreanJusikhoesa,    // 주식회사
  KoreanYuhanhoesa,    // 유한회사
  JapaneseKabushikiKaisha, // 株式会社
  IndianPrivateLimited,
  IndianPublicLimited,
  IndividualSoleProprietor,
  Government,          // government entity
  NGO,                 // non-governmental organization
  Trust,
  Other { description: String },
}

struct LegalNameChange {
  changed_at: Date,
  prior_name: LegalName,
  new_name: LegalName,
  reason: NameChangeReason,
  evidence_artefact_id: ArtefactId,        // amendment to articles, court order, etc.
}

enum NameChangeReason {
  ArticlesAmendment,
  CorporateAction,         // merger, demerger
  RebrandingResolution,
  CourtOrder,
  AdministrativeCorrection,
}

struct SignatoryAuthority {
  signatory_principal_id: PrincipalId,
  full_legal_name: String,
  role_title: String,                             // CEO, CFO, GC, etc.
  authority_scope: AuthorityScope,                // full / contract-type limited / monetary limited / etc.
  monetary_limit: MoneyAmount?,
  contract_type_limit: [ContractType]?,
  effective_from: Date,
  effective_to: Date?,
  authority_evidence_artefact_id: ArtefactId,    // board resolution, POA, etc.
  authority_evidence_seal: BLAKE3Hash,
}

enum SanctionsCheckState {
  NeverChecked,
  CheckedClean { last_check_at: Timestamp<RFC3339>, sources_checked: [SanctionsList] },
  CheckedFlagged { last_check_at: Timestamp<RFC3339>, match: SanctionsMatch },
}

enum SanctionsList {
  OFACSpeciallyDesignatedNationals,    // US Treasury OFAC SDN
  OFACSectoral,                         // OFAC Sectoral Sanctions
  EUFinancialSanctions,                // EU Council
  UKFinancialSanctions,                // OFSI UK
  UNSecurityCouncilConsolidated,
  KRSanctionsList,                     // 외교부 제재 명단
  JPMOFASanctions,                     // 財務省 経済制裁
  Custom { tenant_list_id: String },
}
```

## Resolution algorithm

When a contract draft references a counterparty, the µservice resolves:

1. **LEI lookup** (if provided): canonical via GLEIF Global LEI Index.
2. **Company registry lookup**: per `registration_jurisdiction` + `registration_id`.
3. **VAT/EIN lookup**: per `registration_jurisdiction`.
4. **Name-based fuzzy match**: against tenant's prior counterparties using normalized legal name (strip "Inc", "Ltd", "LLC", "주식회사" suffixes; ASCII fold; Levenshtein <= 2).
5. **Address match**: registered address overlap.

If any of (1)-(3) yields a match, use that match. If (4)-(5) yields a tentative match, surface for human confirmation.

## Merger / acquisition resolution

When a counterparty is acquired:

- `predecessor.successor_counterparty_id = acquirer.counterparty_id`.
- Outstanding contracts continue (per assignment provisions); the µservice creates a `CounterpartyAssignmentEvent` linking the prior counterparty to the new entity.
- Audit-chain records the merger and the contract chain-of-title.

When a counterparty is dissolved:

- `counterparty.dissolution_date` populated.
- Outstanding contracts flagged for review (potential breach or assignment).
- No new contracts may be authored with a dissolved counterparty.

## Sanctions screening

Every counterparty creation triggers OFAC SDN check (and UN + EU + UK + KR/JP if applicable). High-confidence matches block contract creation; low-confidence matches surface for manual review.

The sanctions check is re-run:

- At every contract creation.
- At every contract amendment.
- Monthly for active contracts.

A previously-clean counterparty being flagged triggers an active-contract review and potential contract suspension.

## Cross-tenant counterparty privacy

Counterparty data is tenant-scoped. The µservice does **not** maintain a cross-tenant master counterparty registry (would violate ADR-0244 tenant isolation). However:

- Each tenant may opt into the optional `governance` µservice global LEI/entity registry for read-only enrichment.
- Read-only enrichment provides legal-name normalization, LEI lookup, and sanctions screening; never reveals other tenants' contract or relationship data.

## CRM integration

CLM cross-emits to the `crm` µservice when a counterparty also exists as a CRM Account:

- `crm.account.id` ↔ `clm.counterparty.id` resolution is per-tenant.
- Counterparty changes (name change, merger) propagate to CRM via the ontology projection.
- CRM-originated counterparty updates propagate to CLM via the same path.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ContractCreate",
  resource is Contract
) when {
  resource.counterparty.dissolution_date != null
};

forbid (
  principal,
  action == Action::"ContractCreate",
  resource is Contract
) when {
  resource.counterparty.sanctions_check_state matches "CheckedFlagged" &&
  resource.counterparty.sanctions_match.confidence >= 0.85
};

forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.signatory.authority_scope == "limited" &&
  resource.contract.financial_value > resource.signatory.monetary_limit
};
```

## Audit events

- `oya.contract.lifecycle.management.counterparty.created`
- `oya.contract.lifecycle.management.counterparty.name_changed`
- `oya.contract.lifecycle.management.counterparty.merged`
- `oya.contract.lifecycle.management.counterparty.dissolved`
- `oya.contract.lifecycle.management.counterparty.sanctions_screened`
- `oya.contract.lifecycle.management.counterparty.sanctions_flagged`

## Standards references

- ISO 17442:2020 — Legal Entity Identifier (LEI).
- GLEIF Global LEI Index.
- UCC Article 9-503 (sufficient name in financing statement).
- FATF Recommendations (counterparty due diligence).
- US Treasury OFAC Specially Designated Nationals (SDN) List.
- EU Council Regulation (EC) No 2580/2001 (financial sanctions).
- UK OFSI Consolidated List.
- KR 외교부 경제제재 명단.

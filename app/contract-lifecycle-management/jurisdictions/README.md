---
doc_class: JurisdictionOverlayCatalog
microservice: contract-lifecycle-management
dimension_id: L-010
related_packs: [gdpr, eidas, esign, kr-pipa]
date: 2026-05-21
---

# Jurisdiction Overlay Catalog

CLM contracts are governed by jurisdiction-specific law. The jurisdiction pack overlays compose with compliance packs (per `packs/README.md`) to produce the active rules for a given `(tenant, contract)` tuple.

## Catalog

| Jurisdiction pack | Authoritative source | File |
|---|---|---|
| `us-federal` | USC + CFR + federal common law | `us-federal.md` |
| `us-state-ca` | CA Civil Code + CA Labor Code + CCPA/CPRA | `us-state-ca.md` |
| `us-state-ny` | NY GBL + NY Tech Law § 304 (ESRA) | `us-state-ny.md` |
| `us-state-tx` | TX Business & Commerce Code | `us-state-tx.md` |
| `us-state-de` | DE General Corporation Law + DE UCC | `us-state-de.md` |
| `eu-eidas-qes` | Regulation (EU) 910/2014 + 2024/1183 | `eu-eidas-qes.md` |
| `eu-member-de` | Bürgerliches Gesetzbuch + HGB + DSGVO | `eu-member-de.md` |
| `eu-member-fr` | Code civil + Code de commerce + RGPD | `eu-member-fr.md` |
| `eu-member-ie` | Statute Law of Ireland | `eu-member-ie.md` |
| `uk-ea2000` | Electronic Communications Act 2000 + UK GDPR | `uk-ea2000.md` |
| `kr-pipa-sovereign` | 개인정보 보호법 + 전자서명법 | `kr-pipa-sovereign.md` |
| `jp-denshi-shomei` | 電子署名法 + 民法 | `jp-denshi-shomei.md` |
| `in-it-act-2000` | Information Technology Act 2000 | `in-it-act-2000.md` |
| `ca-province-on` | Electronic Commerce Act 2000 (Ontario) | `ca-province-on.md` |
| `ca-province-qc` | Charter of the French Language + Bill 96 | `ca-province-qc.md` |
| `au-eta-1999` | Electronic Transactions Act 1999 (Australia) | `au-eta-1999.md` |
| `sg-eta-2010` | Electronic Transactions Act 2010 (Singapore) | `sg-eta-2010.md` |

## Composition

Multiple jurisdictions may apply (e.g. counterparty in US, governing law DE, performance in JP). The µservice computes the active jurisdiction set per contract and applies higher-restriction-wins on:

- Retention.
- Signature evidence level (SES / AES / QES / certified-electronic).
- Translation language requirement.
- Notice method (certified mail required in some jurisdictions).
- Cross-border transfer rules.
- Consumer disclosure language and detail level.

## Per-jurisdiction headline rules

### US Federal
- ESIGN Act 15 USC § 7001 (federal floor for electronic signature).
- UCC (varies by state adoption; commercial sales).
- FAR (federal procurement).
- SOX (public companies).
- HIPAA (health).
- FCPA (anti-corruption abroad).

### EU
- GDPR (data protection).
- eIDAS (electronic signature).
- Rome I (choice of law for contracts).
- UCITS / MiFID II (financial services).
- EU AI Act (high-risk AI).

### Korea
- PIPA (data protection).
- 전자서명법 (Digital Signature Act).
- CSAP (cloud security for public sector).
- Commercial Code (general contracts).

### Japan
- 個人情報保護法 (APPI).
- 電子署名法 (Digital Signature Act).
- 民法 (Civil Code).
- 商法 (Commercial Code).
- 会社法 (Companies Act).

### India
- IT Act 2000 + IT Rules 2021.
- Indian Contract Act 1872.
- Digital Personal Data Protection Act 2023.

### UK
- UK GDPR.
- Electronic Communications Act 2000.
- UK Bribery Act 2010.

### Canada
- PIPEDA (federal privacy).
- Provincial laws (PIPA-AB, PIPA-BC, Law 25 Québec).
- Bill 96 / Charter of the French Language (Québec).

### Australia
- Electronic Transactions Act 1999.
- Privacy Act 1988.

### Singapore
- Electronic Transactions Act 2010.
- PDPA 2012.

## Tenant configuration

Per-tenant jurisdiction declaration:

```
TenantJurisdictionConfig {
  tenant_id: TenantId,
  primary_jurisdiction: JurisdictionRef,
  additional_jurisdictions: [JurisdictionRef],
  default_governing_law: GoverningLawRef,
  default_venue: VenueRef,
  cross_border_authorized_destinations: [CountryCode],
}
```

## Cedar gate composition

Per-jurisdiction Cedar fragments are composed at tenant activation time:

```cedar
// US contracts require ESIGN intent capture
forbid (principal, action == Action::"SignaturePacketSeal", resource)
when { resource.governing_law.country == "US" && resource.esign_intent_evidence == null };

// EU contracts require eIDAS AES minimum
forbid (principal, action == Action::"SignaturePacketSeal", resource)
when { resource.governing_law.country in eu_member_states &&
       resource.signature_level not in ["AES", "QES"] };

// KR sovereign contracts require KISA-rooted TSA
forbid (principal, action == Action::"SignaturePacketSeal", resource)
when { resource.active_packs.contains("kr-pipa-sovereign") &&
       resource.signature_level == "KR_CERTIFIED" &&
       resource.tsa.kisa_qualified == false };
```

## Audit events

- `oya.contract.lifecycle.management.jurisdiction.resolved`
- `oya.contract.lifecycle.management.jurisdiction.conflict_detected`
- `oya.contract.lifecycle.management.jurisdiction.higher_restriction_applied`

Note: per-jurisdiction full overlay files referenced in the catalog table above are scaffolded with their core rules; complete content authoring is staged across Wave 15B+ as jurisdiction-specific legal counsel is engaged per ADR-0328 substance-bar sequencing.

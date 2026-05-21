---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-019
authoritative_source: International commercial practice + Hague Convention + Rome I (EU)
related_packs: [gdpr, eidas, kr-pipa]
date: 2026-05-21
---

# Multi-Language Contract Overlay

Cross-border commercial contracts are frequently executed in two or more languages. CLM supports side-by-side language versions with a governing-language clause that establishes which language version controls in case of interpretive conflict.

## Multi-language contract model

```
MultiLanguageContract {
  contract_id: ContractId,
  primary_language: BCP47Tag,                    // governing language
  primary_version: ContractVersion,
  alternate_versions: [LanguageVersion],
  governing_language_clause: GoverningLanguageClause,
  translation_attestations: [TranslationAttestation],
  conflict_resolution_rule: ConflictResolutionRule,
}

struct LanguageVersion {
  language: BCP47Tag,
  contract_version: ContractVersion,
  translation_source: TranslationSource,
  translator_attestation: TranslatorAttestation?,
  hash: BLAKE3Hash,
}

enum TranslationSource {
  HumanCertified {
    translator_principal_id: PrincipalId,
    certification_body: CertificationBody,     // ATA, ITI, JSAT, etc.
    sworn_translation: bool,
  },
  AIAssistedHumanReviewed {
    ai_model: ModelId,
    human_reviewer_principal_id: PrincipalId,
  },
  AIOnly {
    ai_model: ModelId,
    review_status: ReviewStatus,
  },
  PartyProvided {
    providing_party: LegalEntityRef,
    attestation: PartyAttestation,
  },
}

enum ConflictResolutionRule {
  GoverningLanguageControls,                    // primary language is dispositive
  CourtInterpretation,                          // court interprets conflict
  BothAuthentic { tie_breaker: TieBreaker },    // both versions authoritative; tie-breaker rule
  SpecificClauseGoverningLanguage {
    clause_specific_rules: HashMap<ClauseId, BCP47Tag>,
  },
}
```

## Supported language pairs (current)

The µservice ships with structured support for the following language pairs (BCP 47):

- en-US ↔ es-MX, es-419
- en-US ↔ fr-FR, fr-CA
- en-US ↔ de-DE
- en-US ↔ ja-JP
- en-US ↔ ko-KR
- en-US ↔ zh-CN, zh-TW
- en-US ↔ pt-BR, pt-PT
- en-US ↔ ar-SA
- en-US ↔ hi-IN
- en-US ↔ ru-RU
- en-US ↔ it-IT
- en-US ↔ nl-NL
- en-US ↔ pl-PL
- en-GB ↔ all the above
- de-DE ↔ fr-FR, es-ES, it-IT
- fr-FR ↔ es-ES, it-IT, pt-PT
- es-ES ↔ pt-PT
- ko-KR ↔ ja-JP, zh-CN
- ja-JP ↔ zh-CN

Other pairs supported via AI-assisted translation with mandatory human review.

## Side-by-side rendering

The CLM UI renders side-by-side language versions:

- Primary on the left, alternate on the right.
- Clause-numbered alignment (clauses with the same numbering align visually).
- Conflict markers visually distinct when clauses do not align (different clause count, different ordering).

## Governing-language clause template

Standard governing-language clause for an MSA with English as primary + French as alternate:

```
GOVERNING LANGUAGE / LANGUE FAISANT FOI

This Agreement is executed in both the English and French languages. In the
event of any conflict or inconsistency between the English version and the
French version of this Agreement, the [English / French] version shall
control and shall be deemed the authoritative version.

Le présent Contrat est rédigé en langue anglaise et en langue française. En
cas de divergence d'interprétation entre la version anglaise et la version
française, la version [anglaise / française] prévaudra et sera considérée
comme la version faisant foi.
```

## Jurisdiction-specific overrides

Some jurisdictions impose specific language requirements:

- **France**: Loi Toubon (Law No. 94-665) requires consumer contracts in France to be in French. A French version must be available; English may be the governing language between businesses but consumer-facing must be French.
- **Québec**: Charter of the French Language (Bill 96 amendments 2022) requires French versions for consumer contracts and adhesion contracts; the parties must explicitly opt for English with a French version available.
- **Korea**: 약관의 규제에 관한 법률 (Standard Terms Regulation Act) requires Korean for consumer-facing standard terms.
- **China**: contracts performed in China typically require a Chinese version; if both English and Chinese exist, the Chinese version may control by default unless explicitly stated otherwise.
- **Japan**: not generally mandated, but the Japanese version of cross-border B2B contracts is often the governing version for Japanese-court adjudication.
- **Saudi Arabia**: Arabic version required for enforceability in Saudi courts.

## Translation attestation

When `translation_source = HumanCertified`, the µservice retains:

- Translator's credentials (ATA, ITI, JSAT, KAFOL, etc.).
- Sworn translation attestation if required.
- Translator's signed declaration of accuracy.
- BLAKE3 hash of the translated version.

Sworn translations are typically required for court-filed documents; the µservice flags `sworn_translation_required = true` for litigation-bound contracts.

## Hash binding

Each language version has its own BLAKE3 hash; the contract's `MultiLanguageContract` record carries all hashes. Signature envelopes seal both:

- The primary language hash.
- A composite Merkle hash of all language version hashes.

This prevents tampering with any language version without invalidating the seal.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.contract.required_languages.contains("fr-FR") &&
  resource.contract.has_french_version == false &&
  resource.contract.consumer_facing == true &&
  resource.contract.governing_law.country == "FR"
};

forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.contract.required_languages.contains("ko-KR") &&
  resource.contract.has_korean_version == false &&
  resource.contract.consumer_facing == true &&
  resource.contract.governing_law.country == "KR"
};
```

## Audit events

- `oya.contract.lifecycle.management.multilingual.version_added`
- `oya.contract.lifecycle.management.multilingual.translation_attested`
- `oya.contract.lifecycle.management.multilingual.governing_language_set`
- `oya.contract.lifecycle.management.multilingual.conflict_detected`

## Standards references

- BCP 47 (IETF) — Tags for Identifying Languages.
- Loi Toubon No. 94-665 (France).
- Charter of the French Language (Québec, Bill 96 amendments 2022).
- 약관의 규제에 관한 법률 (Korea Standard Terms Regulation Act).
- People's Republic of China Contract Law.
- Hague Convention on Choice of Court Agreements (2005).
- Rome I Regulation (EC) No 593/2008.

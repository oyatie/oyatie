---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-001
authoritative_source: GDPR Article 7 + EDPB Guidelines 05/2020 on consent
related_packs: [gdpr]
date: 2026-05-21
---

# GDPR Article 7 — Consent Records

## Statutory text (paraphrase)

GDPR Article 7 imposes four cumulative requirements on consent as a lawful basis for processing personal data:

1. **(1) Demonstrability**: the controller must be able to demonstrate that the data subject has consented to processing of his or her personal data.
2. **(2) Distinguishability**: where consent is given in the context of a written declaration which also concerns other matters, the request for consent must be presented in a manner clearly distinguishable from the other matters, in an intelligible and easily accessible form, using clear and plain language.
3. **(3) Withdrawability**: the data subject has the right to withdraw consent at any time; withdrawal must be as easy as giving consent.
4. **(4) Conditionality**: when assessing whether consent is freely given, utmost account must be taken of whether the performance of a contract is conditional on consent to processing of personal data that is not necessary for that contract's performance.

Recital 32 elaborates: consent must be by clear affirmative act establishing freely given, specific, informed and unambiguous indication of the data subject's agreement. Silence, pre-ticked boxes, or inactivity do not constitute consent.

## CLM consent record schema

```
ConsentRecord {
  consent_id: UUIDv7,                          // immutable
  tenant_id: TenantId,                          // tenant-scoped per ADR-0244
  data_subject_principal_id: PrincipalId,      // FK to identity µservice
  contract_id: ContractId?,                     // optional link to contract
  purpose_text: String,                         // verbatim, in subject's chosen language
  purpose_text_locale: BCP47Tag,                // e.g. "ko-KR", "de-DE"
  consent_given_at: Timestamp<RFC3339>,         // tenant-home-cell time zone
  consent_mechanism: ConsentMechanism,
  controller_identity: LegalEntityRef,          // tenant + tenant DPO contact
  third_party_recipients: [LegalEntityRef]?,    // if disclosure to third parties
  cross_border_destinations: [CountryCode]?,    // GDPR Chapter V
  retention_period_days: u32?,                  // explicit retention horizon
  withdrawal_endpoint: HTTPUrl,                  // must be no harder than giving consent
  evidence_hash: BLAKE3Hash,                    // hash of the consent artefact bundle
  audit_event_id: AuditEventId,                 // cross-reference to audit-chain
  withdrawal_evidence: WithdrawalEvent?,        // null until withdrawn
  legal_basis_pre_consent: LawfulBasis?,        // if consent overrides another basis
}

enum ConsentMechanism {
  ClickwrapWithSeparateCheckbox {
    checkbox_unchecked_by_default: true,        // pre-ticked forbidden by Recital 32
    surrounding_text_hash: BLAKE3Hash,
  },
  EsignatureWithConsentClause {
    signature_packet_id: SignaturePacketId,
    clause_id: ClauseId,
  },
  WrittenSignatureWithConsentClause {
    scan_artefact_id: ArtefactId,
    clause_id: ClauseId,
  },
  OralConsentRecorded {
    audio_artefact_id: ArtefactId,
    transcription_artefact_id: ArtefactId,
    interpreter_attestation: PrincipalId?,
  },
  EUDIWalletAttestation {                       // eIDAS 2.0
    pid_attestation_id: WalletAttestationId,
    issuer_id: WalletIssuerId,
  },
}

struct WithdrawalEvent {
  withdrawn_at: Timestamp<RFC3339>,
  withdrawal_mechanism: WithdrawalMechanism,    // must be at most as hard as the giving mechanism
  effect_on_dependent_obligations: [ObligationId],
  audit_event_id: AuditEventId,
}
```

## Article 7(1) — demonstrability

The `evidence_hash` is the BLAKE3 hash of the full consent artefact bundle:

- The purpose_text exactly as displayed to the data subject.
- The surrounding UI state (where consent was given electronically): button positions, checkbox labels, any disclosure links.
- The data subject's authenticated principal_id and authentication ladder satisfied.
- The mechanism-specific evidence (clickwrap state, signature envelope, audio recording, etc.).

The hash is sealed into the tenant's audit chain. On an Article 7(1) demonstrability challenge, the µservice produces the bundle and the seal evidence; the seal proves the bundle has not been altered post-consent.

## Article 7(2) — distinguishability

The µservice enforces distinguishability by:

- Rendering the consent clause in a separate UI block with a visible boundary (border, background color contrast ≥ WCAG 4.5:1).
- Requiring the consent clause to be hash-bound separately from the surrounding contract text.
- Refusing to seal a consent record where the surrounding clause text overlaps the consent text (detected via tokenization).

## Article 7(3) — withdrawability

The `withdrawal_endpoint` must satisfy:

- Discoverable from the data subject's account (no more than 2 clicks from any logged-in page).
- No paywall or contract-renewal requirement.
- Functional 24/7 with the same availability SLO as the consent-giving path.
- Returns within 72 hours with a `WithdrawalEvent` record.

Withdrawal triggers re-evaluation of any obligation depending on the withdrawn consent:

- Cross-border transfers under the withdrawn basis are halted.
- Marketing communications cease within 24 hours.
- Third-party recipients listed in `third_party_recipients` are notified of withdrawal.

## Article 7(4) — conditionality

When a contract bundles consent for ancillary processing (e.g. marketing) with consent for contract performance, the µservice splits the consent into separate records. The contract performance proceeds without the ancillary consent; refusing the ancillary consent does not block contract execution.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ProcessPersonalData",
  resource is PersonalDataRecord
) when {
  resource.lawful_basis == "consent" &&
  (
    resource.consent_record == null ||
    resource.consent_record.withdrawal_evidence != null
  )
};

forbid (
  principal,
  action == Action::"ConsentBundle",
  resource is ConsentRecord
) when {
  resource.purpose_text contains_multiple_purposes
};
```

## Retention

Consent records are retained for the duration of the consent + 3 years post-withdrawal to demonstrate Article 7(1) for any retrospective challenge.

## API surface

```
POST /v1/tenants/{tenant_id}/consent-records
  body: ConsentRecord (without consent_id / audit_event_id / evidence_hash)
  response: 201 with full ConsentRecord

GET /v1/tenants/{tenant_id}/consent-records/{consent_id}
  response: 200 with full ConsentRecord

POST /v1/tenants/{tenant_id}/consent-records/{consent_id}/withdraw
  body: WithdrawalMechanism
  response: 200 with WithdrawalEvent

GET /v1/tenants/{tenant_id}/consent-records/{consent_id}/evidence-bundle
  response: 200 with the full evidence artefact bundle (multipart)
```

## Audit event

`oya.contract.lifecycle.management.consent.recorded` with dimensions:

- tenant_id, tenant_class, principal_id, data_subject_principal_id
- contract_id (if linked)
- consent_mechanism, purpose_text_locale
- evidence_hash, audit_event_id

`oya.contract.lifecycle.management.consent.withdrawn` with dimensions:

- tenant_id, tenant_class, principal_id, data_subject_principal_id
- consent_id, withdrawn_at, withdrawal_mechanism
- effect_on_dependent_obligations count

## Open issue tracking

If a data subject claims a consent record is invalid (Article 7(1) challenge), the µservice produces the full evidence bundle + seal verification. If the seal verifies, the consent is presumed valid; the data subject must demonstrate the bundle does not satisfy Article 7 requirements.

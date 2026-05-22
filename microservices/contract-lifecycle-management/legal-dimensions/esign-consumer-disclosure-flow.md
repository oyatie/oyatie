---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-004
authoritative_source: ESIGN Act 15 USC § 7001(c)
related_packs: [esign]
date: 2026-05-21
---

# ESIGN Consumer Disclosure Flow

ESIGN § 7001(c) imposes specific pre-signature disclosures when the counterparty is a **consumer** (natural person entering the contract for personal, family, or household purposes; 15 USC § 7006(1) consumer definition).

## When the flow applies

The flow applies when `counterparty.role == "consumer"` AND the contract is governed by a US law (`governing_law ∈ us_federal ∪ us_states`). The µservice classifies the counterparty as a consumer when any of:

- `counterparty.principal_class == "natural_person"` AND `counterparty.purpose_class == "personal_family_household"`.
- Contract type ∈ `{consumer_credit, consumer_lease, consumer_purchase, residential_lease, residential_purchase, consumer_warranty, telecom_service, utility_service, insurance_consumer}`.
- Counterparty signed up via a consumer onboarding path (declared in identity µservice).

## Disclosure elements (15 USC § 7001(c)(1))

The µservice surfaces a Disclosure Page to the consumer prior to electronic consent. The page must include:

### (A) Right to receive paper records

```
DISCLOSURE — Your right to a paper copy

You have the right to receive a paper copy of [this contract / these records]
at any time and at no charge. To request a paper copy, contact us at:
  [tenant.consumer_disclosure_paper_contact]
or visit:
  [tenant.consumer_disclosure_paper_url]

If you choose to receive paper, we will deliver it within
[tenant.consumer_disclosure_paper_delivery_days] business days at no charge.
```

### (B) Right to withdraw consent

```
DISCLOSURE — Your right to withdraw consent

You have the right to withdraw your consent to receive [these / future] records
electronically at any time. To withdraw:
  - Open your account → Privacy & Communications → Electronic Delivery → Withdraw.
  - Or email: [tenant.consumer_disclosure_withdrawal_email]
  - Or call: [tenant.consumer_disclosure_withdrawal_phone]

Withdrawal will [not / will] terminate your contract. [If withdrawal terminates,
state the consequences here.] No fee will be charged for withdrawing your consent.
```

### (C) Scope of consent

```
DISCLOSURE — Scope of consent

By consenting, you agree that we may deliver the following records electronically:
  [enumerate: this contract, future amendments, billing notices,
   regulatory communications, marketing communications if separately consented,
   etc.]

If you only wish to consent to this single transaction, click "This transaction
only" below. If you wish to consent to electronic delivery for all future
records, click "All future records". You may change your scope at any time.
```

### (D) Hardware/software requirements

```
DISCLOSURE — System requirements

To access and retain electronic records, you need:
  - A web browser supporting TLS 1.3 (any modern browser released after 2020).
  - A PDF reader (free options: Adobe Acrobat Reader, web browser built-ins).
  - At least 50 MB of free disk space to retain a typical contract bundle.
  - A working email address to receive notifications.

These requirements may change. If they change, we will notify you in writing
[email and on-screen at next login] at least 30 days in advance, and you may
withdraw consent without penalty.
```

### (Demonstration of ability)

The consumer must demonstrate ability to access electronic records before consent is valid. The µservice implements this via:

1. Email the consumer a link to a one-page demonstration document.
2. Require the consumer to click the link AND submit a one-click confirmation.
3. Record the click + confirmation timestamp as the demonstration evidence.

If the demonstration fails (no click within 14 days), the µservice falls back to wet-ink signing.

## Evidence schema

```
ConsumerDisclosureEvidence {
  evidence_id: UUIDv7,
  tenant_id: TenantId,
  contract_id: ContractId,
  consumer_principal_id: PrincipalId,
  disclosure_text_hash: BLAKE3Hash,           // hash of the verbatim disclosure displayed
  disclosure_displayed_at: Timestamp<RFC3339>,
  disclosure_locale: BCP47Tag,                 // typically en-US, es-US, etc.
  paper_copy_contact: ContactRef,
  withdrawal_endpoint: HTTPUrl,
  withdrawal_email: EmailAddress,
  withdrawal_phone: PhoneNumber,
  scope_chosen: ConsentScope,                  // single_transaction | all_future_records
  hardware_software_requirements_text_hash: BLAKE3Hash,
  demonstration_link_id: ArtefactId,
  demonstration_email_sent_at: Timestamp<RFC3339>,
  demonstration_link_clicked_at: Timestamp<RFC3339>,
  demonstration_confirmation_at: Timestamp<RFC3339>,
  consent_given_at: Timestamp<RFC3339>,
  network_attestation: NetworkAttestation,
  audit_event_id: AuditEventId,
}

enum ConsentScope {
  SingleTransaction { contract_id: ContractId },
  AllFutureRecords {
    scope_text_hash: BLAKE3Hash,
    revocation_endpoint: HTTPUrl,
  },
}
```

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.active_packs.contains("esign") &&
  resource.counterparty.role == "consumer" &&
  (
    resource.consumer_disclosure_evidence == null ||
    resource.consumer_disclosure_evidence.demonstration_confirmation_at == null
  )
};
```

## Disclosure refresh on requirements change

When `tenant.consumer_disclosure_hardware_software_requirements` changes:

1. The µservice notifies all consumers with active `AllFutureRecords` consent at least 30 days in advance.
2. Notification is delivered via the consumer's preferred channel (email + in-app on next login).
3. Consumer may withdraw consent without penalty.
4. If consumer does not respond within 30 days, the µservice does NOT auto-revoke consent (per § 7001(c)(1)(D) — the disclosure suffices), but flags the consumer for re-disclosure prompts on next login.

## Audit event

`oya.contract.lifecycle.management.esign.consumer_disclosure.demonstrated` with dimensions:

- tenant_id, tenant_class, contract_id, consumer_principal_id
- evidence_id, consent_scope
- demonstration_email_sent_at, demonstration_confirmation_at
- audit_event_id

## Retention

Consumer disclosure evidence retained for the contract retention period + 6 years (the longest applicable US statute of limitations for consumer contracts).

## ESIGN excluded records

Per § 7003, ESIGN does not apply to certain record types (wills, family-law matters, court orders, certain notices). The µservice rejects consumer consent flows for these record types and routes to wet-ink signing.

---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: esign
authoritative_source: ESIGN Act (15 USC § 7001-7006)
related_adrs: [ADR-0251, ADR-0244, ADR-0263]
date: 2026-05-21
---

# ESIGN Pack Overlay — CLM

The ESIGN Act (Electronic Signatures in Global and National Commerce Act, Pub. L. 106-229, 15 USC § 7001-7006) governs electronic signature enforceability in U.S. interstate commerce. Where ESIGN does not preempt, the Uniform Electronic Transactions Act (UETA) governs in 49 states and DC (New York follows ETAA / NY Tech Law § 304).

## Active triggers

The `esign` pack is **mandatory** when:

- `contract.governing_law ∈ {US federal, any US state}`.
- `contract.signatory.residency ∈ US_states ∪ DC ∪ territories`.
- `tenant.declared_jurisdictions` includes the US or any US state.

ESIGN applies to interstate or international commerce; UETA-only governs purely intrastate transactions where the state has adopted UETA. The µservice applies ESIGN as the floor and adds UETA per-state overlays via `jurisdictions/ueta-states.md`.

## § 7001(a) general rule

A signature, contract, or other record shall not be denied legal effect, validity, or enforceability solely because it is in electronic form. The µservice enforces this by:

- Treating electronic and wet-ink signature paths as equivalent at the contract data model level.
- Refusing to discriminate against electronic signatures in any internal validation.

## § 7001(b) preservation of state law

ESIGN preempts UETA only where state UETA is inconsistent with ESIGN. The µservice's `jurisdictions/ueta-states.md` codifies the variations.

## § 7001(c) consumer disclosure flow (P0 LEGAL — see `legal-dimensions/esign-consumer-disclosure-flow.md`)

When the counterparty is a **consumer** (natural person entering the contract for personal, family, or household purposes), § 7001(c) imposes specific pre-signature disclosures. The µservice marks contracts with `counterparty_role = consumer` and enforces:

1. **§ 7001(c)(1)(A) — disclosure of right to receive paper records**. Before consent to electronic delivery, the consumer must be informed in a clear and conspicuous statement of the right to receive paper.
2. **§ 7001(c)(1)(B) — withdrawal of consent**. The consumer must be informed of the right to withdraw consent to electronic records and any consequences (including fees) of withdrawal.
3. **§ 7001(c)(1)(C) — scope**. The consumer must be informed whether consent applies only to a particular transaction or to all future records.
4. **§ 7001(c)(1)(D) — hardware/software requirements**. The consumer must be informed of the hardware and software requirements for access to and retention of electronic records, and on changes to those requirements.
5. **§ 7001(c)(1)(C)(ii) — demonstration**. The consumer must demonstrate the ability to access information in the electronic form that will be used (e.g. by clicking a link in an email and submitting back the result).

The µservice persists the disclosure artefact bundle and the consumer's demonstrable assent as `esign_consumer_disclosure_evidence` records cross-referenced from the `signature_packet`.

## § 7001(d) retention

Where a law requires that a contract or other record be retained, an electronic record satisfies the requirement if (a) the record accurately reflects the information set forth in the original, and (b) the record is accessible for later reference. The µservice satisfies (a) with hash-bound immutable storage (per `legal-dimensions/worm-binding-model.md`) and (b) with the `signature_packet_export` capability.

## § 7001(e) accuracy and accessibility

Where a record must be provided to a person, the record must be in a form that the person can retain. The µservice provides:

- Plain PDF/A-3 conformant signed artefact downloadable for 7 years (or longer per pack overlay).
- HTML email or paper-mail delivery on demand.
- Self-service signature-packet export endpoint.

## Intent-to-sign capture

ESIGN does not specify a particular intent-capture mechanism. The µservice's canonical intent capture is:

- Display the entire document body (no opaque link-out) before the sign button.
- Display a clear statement: "By clicking 'Sign', I, [signatory full legal name], confirm my intent to sign this [contract type] electronically and to be bound by its terms."
- Capture: timestamp, IP address (or equivalent network identifier when behind NAT), user agent, signatory authentication ladder satisfied, declared signatory legal name, and signatory's typed full legal name as a secondary intent witness.

## Tenant-class composition

- `tenant_class=demo_trial`: ESIGN AES + intent capture only; consumer-disclosure-flow gated off (demo_trial restricted to B2B counterparties).
- `tenant_class=paid + billing_components=[per_seat]`: full ESIGN + UETA + consumer disclosure flow available.

## Composition with other packs

- `esign` + `gdpr`: signatory PII (IP, user agent, declared name) is subject to GDPR Article 5 minimization; retain only for the signature evidence period.
- `esign` + `eidas`: when both apply (US+EU cross-border), the signature envelope must satisfy both AES and ESIGN. Practical resolution: PAdES-B-LTA envelope + ESIGN intent-capture artefact bundle attached as a sub-evidence record.
- `esign` + `sox-404`: SOX-relevant contracts (audit-relevant, public company) take seven-year retention.

## Cedar gate fragment

```cedar
forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.active_packs.contains("esign") &&
  resource.counterparty.role == "consumer" &&
  resource.consumer_disclosure_evidence == null
};
```

## Evidence on activation

Activation of the `esign` pack emits:

- `oya.contract.lifecycle.management.pack.esign.activated` audit event.
- Cedar policy compilation against the tenant-scoped schema with consumer-disclosure gate enabled.
- A UETA per-state overlay snapshot for the tenant's declared US jurisdictions.

## Statutory exclusions (§ 7003)

ESIGN does **not** apply to:

- Wills, codicils, testamentary trusts.
- Adoption, divorce, family-law matters.
- Court orders and notices, official court documents.
- Notices of cancellation of utility services.
- Notices of foreclosure, eviction, repossession.
- Notices of cancellation of health insurance or life insurance benefits.
- Notices of product recall affecting health or safety.
- Documents required to accompany hazardous materials transport.
- UCC §§ 1-107, 1-206, and all of Articles 3, 4, 4A, 5, 6, 7, 8, 9 (the µservice handles UCC contracts under separate state-law UCC overlays; ESIGN does not preempt).

The µservice rejects contract creation with `contract_type ∈ esign_excluded_set` under the `esign` pack and routes to the `wet-ink-required` workflow.

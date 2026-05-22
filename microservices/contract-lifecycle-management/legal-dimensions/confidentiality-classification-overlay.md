---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-018
authoritative_source: NDA practice + Defend Trade Secrets Act 18 USC § 1836 + state UTSA
related_packs: [soc-2, iso-27001, sox-404]
date: 2026-05-21
---

# Confidentiality Classification Overlay

Contracts and contract negotiations frequently carry explicit confidentiality markings ("HIGHLY CONFIDENTIAL — ATTORNEYS' EYES ONLY", "TRADE SECRET", "INTERNAL USE ONLY"). CLM enforces classification at the document, clause, and annotation level.

## Classification levels

```
enum ConfidentialityClassification {
  Public,                                      // freely shareable
  Internal,                                    // tenant-internal only
  Confidential,                                // counterparty + tenant only
  HighlyConfidential,                          // restricted within counterparty
  AttorneysEyesOnly,                            // only outside counsel may see
  TradeSecret {                                 // 18 USC § 1836
    designation_date: Date,
    designating_principal_id: PrincipalId,
    reasonable_measures_attestation: ArtefactId,
  },
  Classified {                                  // government-classified
    classification: GovClassification,         // CUI, CONFIDENTIAL, SECRET, TOP SECRET
    declassification_date: Date?,
  },
}
```

## Applied at three granularities

- **Document level**: the contract body has a classification (default `Confidential` for new contracts; `Internal` for templates; `Public` for marketed terms-of-service).
- **Clause level**: individual clauses may carry their own classification (e.g. a pricing schedule may be `HighlyConfidential` while the main MSA body is `Confidential`).
- **Annotation level**: per `legal-dimensions/privilege-tagging-overlay.md`, lawyer comments may carry their own confidentiality + privilege classification.

## Visual markings

The µservice renders confidentiality markings on:

- PDF watermarks (top + bottom of every page).
- Header text in the document body.
- UI banners when viewing the document.
- Export filenames (prefixed with classification level).
- Email subjects when emailed.

## Access control

```cedar
forbid (
  principal,
  action == Action::"ContractRead",
  resource is Contract
) when {
  resource.confidentiality_classification == "HighlyConfidential" &&
  principal.access_level !in ["highly_confidential_authorized",
                                "attorney", "outside_counsel"]
};

forbid (
  principal,
  action == Action::"ContractRead",
  resource is Contract
) when {
  resource.confidentiality_classification == "AttorneysEyesOnly" &&
  principal.role !in ["inside_counsel", "outside_counsel"]
};

forbid (
  principal,
  action == Action::"ContractRead",
  resource is Contract
) when {
  resource.confidentiality_classification matches "TradeSecret" &&
  principal.trade_secret_access_authorization == false
};
```

## Trade Secret protection (Defend Trade Secrets Act)

DTSA 18 USC § 1839(3) requires the owner of a trade secret to "have taken reasonable measures to keep such information secret". CLM provides:

- Designation timestamp + designating principal.
- Reasonable-measures attestation (lock down access, audit log, NDAs in place).
- Access log per principal per access event.
- Termination + export audit log.

When a trade secret is misappropriated (DTSA private right of action), the access log + designation provides evidence.

## Disclosure log

Every access to a `HighlyConfidential`, `AttorneysEyesOnly`, or `TradeSecret`-classified item is logged with:

- Principal id, principal role.
- Access time, access duration.
- Action (read | redline | export | print).
- Source IP, user agent.

The log is itself classified `Confidential` (it reveals who accessed sensitive items).

## Reclassification

Classification may be elevated (more restrictive) by any authorized principal. Demotion (less restrictive) requires a formal review by general counsel + audit-chain event.

## Cross-tenant sharing

When a contract is shared with a counterparty (the contract's other party), the counterparty inherits the confidentiality marking. The counterparty's CLM instance (or whatever they use) is expected to honour the marking; the µservice cannot enforce on the counterparty's side, but the audit-chain seal proves the marking was communicated.

## Audit events

- `oya.contract.lifecycle.management.confidentiality.classified`
- `oya.contract.lifecycle.management.confidentiality.elevated`
- `oya.contract.lifecycle.management.confidentiality.demoted`
- `oya.contract.lifecycle.management.confidentiality.trade_secret_designated`
- `oya.contract.lifecycle.management.confidentiality.access_logged`

## Composition with packs

- `gdpr`: confidentiality classification does not extend GDPR retention; the rules compose.
- `sec-17a-4`: trade-secret status does not exempt from SEC examination; trade-secret materials still produced under SEC examination with appropriate protective order.
- `sox-404`: trade-secret classification is recorded but does not exempt from internal-control documentation.

## Standards references

- Defend Trade Secrets Act 18 USC §§ 1836, 1839.
- Uniform Trade Secrets Act (state adoption).
- UK Trade Secrets Regulations 2018.
- EU Trade Secrets Directive (Directive (EU) 2016/943).
- NIST SP 800-171 (Controlled Unclassified Information).
- ISO/IEC 27001:2022 Annex A.5.13.

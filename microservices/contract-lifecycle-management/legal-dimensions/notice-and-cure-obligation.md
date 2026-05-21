---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-017
authoritative_source: Common-law notice-and-cure doctrine + UCC § 2-602 + civil-law analogues
related_packs: [sox-404]
date: 2026-05-21
---

# Notice-and-Cure Obligation Tracking

Most commercial contracts contain notice-and-cure provisions: a party alleging breach must first notify the other party in writing and provide a specified cure period (typically 15-30-60 days) before declaring default, suspending performance, or terminating. Notice-and-cure is a first-class obligation in CLM's obligation taxonomy.

## Notice-and-cure obligation schema

```
NoticeAndCureObligation {
  obligation_id: UUIDv7,
  contract_id: ContractId,
  source_clause_id: ClauseId,
  source_span: ClauseSourceSpan,           // verbatim span of the notice-and-cure clause
  triggering_breach_class: BreachClass,    // payment | performance | covenant | warranty
  cure_period_days: u32,                   // typically 15, 30, 60
  notice_method: [NoticeMethod],           // mail, email, courier, etc.
  notice_addresses: [Address],             // contractual notice addresses (often distinct from billing)
  cure_extension_rights: Option<CureExtensionRight>,
  consequence_on_failure_to_cure: Consequence,  // termination | acceleration | suspension | etc.
  evidence_required: [EvidenceClass],
}

enum BreachClass {
  Payment,             // failure to pay invoice
  Performance,         // failure to deliver service
  Covenant,            // failure to satisfy covenant (e.g. financial-ratio covenant)
  Warranty,            // breach of warranty
  Confidentiality,     // breach of NDA / confidentiality clause
  IPInfringement,      // alleged IP violation
  AntiCorruption,      // FCPA / UKBA breach
  ChangeOfControl,     // unauthorized assignment / change of control
  ServiceLevel,        // SLA breach
  Other { description: String },
}

enum NoticeMethod {
  CertifiedMail { return_receipt_required: bool },
  RegisteredMail,
  Email { specific_address: EmailAddress },
  Courier { provider: String },              // FedEx, DHL, etc.
  HandDelivery,
  ElectronicCommunicationsPlatform { platform: String },  // contract-stipulated platform
}

enum Consequence {
  Termination { effective: TerminationEffective },
  Acceleration { amount: MoneyAmount },     // accelerate all unpaid amounts
  Suspension { scope: SuspensionScope },
  LiquidatedDamages { amount: MoneyAmount },
  SpecificPerformance,
  CombinationOf(Vec<Consequence>),
}
```

## Notice-and-cure state machine

```
   alleged_breach_detected
            |
            v
   +------------------+
   |  NOTICE_PENDING  |  alleged breach detected; notice not yet sent
   +------------------+
            |
            | issue_notice()
            v
   +------------------+
   |  NOTICE_SENT     |  cure clock running
   +------------------+
            |
            +-- cure_within_period ----> +------------------+
            |                           |  CURED           |  obligation resolved
            |                           +------------------+
            |
            +-- cure_extension_granted -> +------------------+
            |                            |  CURE_EXTENDED   |
            |                            +------------------+
            |
            +-- cure_period_expired ----> +------------------+
                                          |  CURE_FAILED     |
                                          +------------------+
                                                  |
                                                  | declare_consequence()
                                                  v
                                          +------------------+
                                          | CONSEQUENCE_     |
                                          |   DECLARED        |
                                          +------------------+
```

## Extraction (IP-027 pipeline)

The obligation-extraction pipeline detects notice-and-cure clauses by:

- Lexical patterns: "shall notify", "written notice", "cure period", "cure within [N] days", "thirty (30) days to cure".
- Structural patterns: numbered clauses titled "Default", "Termination", "Remedies", "Notice of Breach".
- Cross-reference patterns: definitions section pointing to "Notice Addresses" with addresses.

Each detected notice-and-cure provision generates a `NoticeAndCureObligation` record with `confidence_band`. High-confidence (≥ 0.95) records are auto-tracked; low-confidence (< 0.85) records surface for human review.

## Calendar integration

Notice-and-cure cure periods are critical timing. The µservice cross-emits to the `calendar` substrate:

- At notice issuance: schedule a `cure_period_expires` reminder at notice + cure_period_days.
- At T-7 days: warning reminder.
- At T-2 days: escalation reminder.
- At T+0: cure-failed event if no acknowledgement received.

## Cedar gate

```cedar
forbid (
  principal,
  action in [Action::"ContractTerminate", Action::"ContractAccelerate",
             Action::"ContractSuspend"],
  resource is Contract
) when {
  resource.has_notice_and_cure_provision == true &&
  (
    resource.notice_and_cure_state != "CURE_FAILED" &&
    resource.notice_and_cure_state != "CONSEQUENCE_DECLARED"
  )
};

permit (
  principal,
  action == Action::"NoticeIssue",
  resource is NoticeAndCureObligation
) when {
  principal.role in ["contracts_manager", "general_counsel", "deal_desk"] &&
  resource.contract.tenant_id == principal.tenant_id
};
```

## Evidence required

The µservice maintains evidence of notice issuance:

- Notice text (verbatim).
- Notice method evidence (certified mail receipt; email transmission log; courier tracking number).
- Notice recipient confirmation (return receipt; email delivery; courier signature).
- Notice timestamp (issued + delivered).

For consumer contracts under `esign`, additional evidence of delivery is required.

## Composition with packs

- `sox-404`: notice-and-cure records for material contracts retained 7 years.
- `gdpr`: notice contents may contain PII; subject to GDPR retention.
- `sec-17a-4`: broker-dealer notice-and-cure records subject to SEC retention floor.

## Audit events

- `oya.contract.lifecycle.management.notice_and_cure.alleged_breach`
- `oya.contract.lifecycle.management.notice_and_cure.notice_issued`
- `oya.contract.lifecycle.management.notice_and_cure.notice_delivered`
- `oya.contract.lifecycle.management.notice_and_cure.cure_extension_granted`
- `oya.contract.lifecycle.management.notice_and_cure.cured`
- `oya.contract.lifecycle.management.notice_and_cure.cure_failed`
- `oya.contract.lifecycle.management.notice_and_cure.consequence_declared`

## Standards references

- UCC § 2-602 (notice of rejection of goods).
- UCC § 2-607 (notice of breach).
- Restatement (Second) of Contracts § 235 (effect of substantial performance).
- Restatement (Second) of Contracts § 241 (material vs immaterial breach).
- Civil Code Article 1601 (France); § 326 BGB (Germany); 民法 § 541 (Japan).

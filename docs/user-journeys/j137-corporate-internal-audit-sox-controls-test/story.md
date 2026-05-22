---
doc_class: User-Journey-Story
journey_id: j137-corporate-internal-audit-sox-controls-test
status: draft
date: 2026-05-20
authority_tier: 3
audience: [council-product, council-architecture, council-security, council-legal, axis-internal-audit, axis-finance-controls]
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0313-conglomerate-hierarchy
  - ADR-0307-detection-substrate
  - ADR-0310-investigation-case-management
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0028-audit-chain-merkle-sealed
  - ADR-0263-observability-emission-contract
  - ADR-0145-inter-microservice-communication
related_specs:
  - /specs/microservices/audit-chain.json
  - /specs/microservices/workflow-engine.json
  - /specs/microservices/payments.json
  - /specs/microservices/messenger.json
  - /specs/microservices/mail.json
  - /specs/microservices/identity.json
  - /specs/microservices/compliance.json
  - /specs/microservices/ops-dashboard-control-center.json
critical_path_rows:
  - documentation-rigor.md §3.2.5 row 8 (SOX 404 controls audit)
  - documentation-rigor.md §3.2.5 row 9 (internal-audit cross-µservice read)
anchor_archetype: sam-okafor-35-lagos
locale: en-NG (Sam) / en-US (corporate canonical) / per-employee locale on read
regulatory_anchors:
  - Sarbanes-Oxley Act of 2002 §404 (internal-control attestation)
  - Sarbanes-Oxley Act §302 (CEO/CFO certification)
  - PCAOB Auditing Standard No. 5 (integrated audit)
  - SEC Rule 13a-15 (disclosure controls)
  - Dodd-Frank §922 (whistleblower-protection alignment)
  - EU Whistleblower Directive 2019/1937 (cross-jurisdiction overlay)
  - GDPR Art 6(1)(f) (legitimate-interest basis for tenant-scoped read)
  - Electronic Communications Privacy Act (ECPA) 1986 §2701 (US employer-stored-communications rule)
  - ISO 27001 A.18 (compliance with legal/contractual requirements)
purpose: >
  Narrate Sam Okafor's Q2-2026 SOX 404 quarterly controls test, end to end,
  exercising the B2B_INTERNAL_AUDIT Cedar permit grammar against eight
  µservices: messenger, mail, workflow-engine, payments, audit-chain,
  ops-dashboard-control-center, identity, compliance. Prove that the
  permit grammar (a) admits tenant-owned reads, (b) emits sealed audit
  events for every read, and (c) refuses every employee personal-tenant
  principal under default-deny. Demonstrate why ADR-0311 (the dual-tenant
  boundary) is not merely policy but the actual code-path Sam encounters
  when he tries to read across the seam.
---

# j137 — Sam Okafor's Q2 2026 SOX 404 controls test

> **Purpose.** This is Sam Okafor's quarterly SOX 404 controls test for
> the second quarter of 2026, conducted over four working days from
> Monday 13 July 2026 through Thursday 16 July 2026, against Marcus's
> 5,000-person multinational corporate tenant. Every Cedar permit Sam
> exercises in those four days is the ADR-0311 dual-tenant boundary
> operating in production. If any seam in this story would have failed,
> the corporation's SOX attestation would have been unattestable; if
> any seam would have over-permitted, the corporation would have
> committed an ECPA / GDPR / EU-WB violation. The story is concrete
> because the contract is concrete. Sam's permit grammar is the
> first widely-used B2B_INTERNAL_AUDIT exercise of the system.

## 1. Sam's continuity of identity — one human, multiple tenancy memberships

Sam Okafor is not three users. He is one human across multiple tenant
memberships that oyatie distinguishes by `audience_type` overlay
(ADR-0244 + ADR-0311), not by fragmenting his identity.

| Context | Tenant | Principal | Cell tier | audience_type | Pack overlay |
|---|---|---|---|---|---|
| **Work — internal audit** | `marcus-corp.tenant` | `sam.okafor@marcus-corp.com` | Tier-3 (regulated SOX/PCAOB) | `B2B_INTERNAL_AUDIT` | `pack-us-sox-404 + pack-pcaob-as5 + pack-eu-whistleblower-2019-1937` |
| **Personal** | `oyatie.consumer.global` | `sam.okafor@oyatie.me` | Tier-2 (consumer general) | `B2C_CONSUMER` | `pack-ng-data-protection-2023 + pack-eu-gdpr-cross-border` |
| **Family parent** | `oyatie.family.global` | `sam.okafor@oyatie.me` (parent role) | Tier-2 (with KOSA/COPPA-equiv) | `B2C_CONSUMER_PARENT` | `pack-ng-childrens-rights` |

The cross-tenant invariant: Sam's WORK permit (B2B_INTERNAL_AUDIT) is
scoped to `marcus-corp.tenant` and does NOT confer any read on any
other tenant — not on his personal tenant, not on any employee's
personal tenant, not on Marcus's other subsidiary tenants in the
conglomerate (per ADR-0313). This is enforced by Cedar at the
api-gateway, not by application code. The default-deny holds.

## 2. The week before — Friday 10 July 2026, 17:45 WAT, Lagos

Sam is in his home office in Ikoyi. His audit charter (signed by
Marcus and the audit-committee chair on 4 January 2026) gives him a
calendar of mandated work for the year: Q1 SOX 404 test conducted
April 6–9; Q2 test due July 13–16; Q3 test October 19–22; Q4 test
January 18–21, 2027 (running into the 10-K filing).

His Q2 PCAOB AS-5 plan focuses on the revenue-cycle controls
identified as "Reasonably Possible — Material" in last quarter's
review. The control set covers:

- **RC-01** — Order intake authorization (sales rep ↔ sales manager).
- **RC-02** — Credit-check pre-approval (workflow-engine integration
  with Equifax / Experian).
- **RC-03** — Invoice generation matches order (workflow-engine).
- **RC-04** — Invoice approval (chief revenue officer or delegate).
- **RC-05** — Payment receipt matched to invoice (Stripe + ACH).
- **RC-06** — Revenue-recognition booking (ERP integration).
- **RC-07** — Period close (cut-off, sign-offs in workflow-engine).

For each, Sam must produce sample evidence per PCAOB AS-5 sample-size
tables. For RC-04 (invoice approval, the highest-risk control), the
sample is 60 transactions stratified by amount-bucket.

Sam opens his oyatie Mail and Calendar surface on his work passkey —
he's in `marcus-corp.tenant` with audience_type `B2B_INTERNAL_AUDIT`.
The top of his Calendar shows:

```
Mon Jul 13 — 09:00 SOX 404 Q2 kickoff (audit committee chair, Marcus, CFO)
Tue Jul 14 — 09:00–18:00 Sample-pull day (deep work)
Wed Jul 15 — 09:00–18:00 Walkthrough + interview day
Thu Jul 16 — 13:00 Read-out with audit committee
Fri Jul 17 — 14:00 External auditor handoff (PwC team)
```

He drafts a quarterly-plan message to his small team (Tunde and Aisha,
both senior auditors reporting to him) in work-Messenger:

```
Subject: Q2 SOX 404 — sample size 60 for RC-04; let's stratify by
amount-bucket. I'll handle the pulls; you two will own walkthrough
interviews. Standard playbook. Cedar permit batch will be requested
Sunday night. — Sam
```

## 3. Sunday night — preparing the Cedar permit batch

On Sunday 12 July at 22:30 WAT, Sam logs into oyatie ops-dashboard at
`https://ops.marcus-corp.tenant.oyatie.dev` from his work laptop
(his passkey is bound to the laptop's secure enclave per ADR-0188).

The dashboard's "Internal Audit" pane is visible because his
audience_type resolves to `B2B_INTERNAL_AUDIT`. He clicks "Q2 SOX 404 —
new audit run" and the workflow-engine spins up an audit-case under
ADR-0310 case-management:

```
Audit-case-id: ac-marcus-corp-2026-q2-sox-404
Owner: sam.okafor@marcus-corp.com
Authority chain: audit-charter-v3.pdf (signed 2026-01-04) +
                 audit-committee-resolution-2026-q1-001.pdf
Cedar permit batch: requested 2026-07-12T22:33Z
Permit grant SLA: 2 business hours via dual-control review
```

The Cedar permit request includes the seven control-IDs (RC-01..RC-07),
the sample-pull scope (Q2 = 2026-04-01 → 2026-06-30), and the eight
µservices needed: messenger, mail, workflow-engine, payments,
audit-chain, ops-dashboard-control-center, identity, compliance.

Cedar permit grammar fragment (this is what the policy engine evaluates
at every read):

```cedar
permit (
  principal == User::"sam.okafor@marcus-corp.com",
  action in [
    Action::"messenger.read_tenant_archive",
    Action::"mail.read_tenant_archive",
    Action::"workflow_engine.read_execution_logs",
    Action::"payments.read_approval_chain",
    Action::"audit_chain.read_seal_evidence"
  ],
  resource is Resource
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  principal.audit_case_id == "ac-marcus-corp-2026-q2-sox-404" &&
  resource.tenant_id == "marcus-corp.tenant" &&
  resource.classification_window.start >= datetime("2026-04-01T00:00:00Z") &&
  resource.classification_window.end <= datetime("2026-06-30T23:59:59Z") &&
  context.dual_control_approval_at != null &&
  context.audit_charter_active == true
};

// Critical: deny employee personal-tenant resources
forbid (
  principal == User::"sam.okafor@marcus-corp.com",
  action,
  resource is Resource
) when {
  resource.tenant_id != "marcus-corp.tenant" &&
  resource.principal_class != "marcus-corp-tenant-owned"
};
```

The dual-control review fires automatically: the audit-committee chair
(`audrey.chen@marcus-corp.com`, an independent director) gets a
Messenger DM at 22:35 WAT asking her to co-sign the permit grant. She
reviews the scope (Q2 only, tenant-owned only, seven control IDs) and
clicks APPROVE in her ops-dashboard pane. The permit is in force from
22:41 WAT and expires at midnight UTC on 2026-07-17 (the audit week).

This dual-control gate is required by ADR-0311 §D-4: any
B2B_INTERNAL_AUDIT permit must be co-signed by an independent
audit-committee member. The permit envelope is sealed into the audit
chain immediately so the grant itself is itself audited.

## 4. Monday 13 July, 09:00 WAT — kickoff and first sample pull

The kickoff call runs from 09:00 to 09:30 in oyatie Meet (work-tenant).
Marcus, CFO Lin Wei, audit-committee chair Audrey Chen, and Sam's team
(Tunde, Aisha) attend. Marcus says, "Sam, you have my full backing. Let
me know if you need anything. The 10-K is on schedule — keep us on
schedule."

At 09:35 Sam opens his audit-pane and clicks "Begin sample pull —
RC-04 invoice approval, n=60 stratified".

The workflow-engine generates the stratified sample using the
PCAOB AS-5 algorithm:

- Stratum A (>$500K): all 7 invoices in Q2 (saturation sample).
- Stratum B ($100K–$500K): 23 invoices (sample of 53 in Q2).
- Stratum C ($25K–$100K): 21 invoices (sample of 187 in Q2).
- Stratum D (<$25K): 9 invoices (sample of 2,341 in Q2).

Total: 60 invoices, sample IDs `inv-Q2-026-001` through `inv-Q2-026-060`.

For each, the workflow-engine emits an audit-pull job:

```yaml
- job_id: pull-inv-Q2-026-001
  invoice_id: 247811
  amount_usd: 712_400
  approver_principal: chief.revenue@marcus-corp.com
  approval_chain_request:
    target: payments.approval_chains
    cedar_action: payments.read_approval_chain
  messenger_threads_request:
    target: messenger.tenant_archive
    filters:
      participants: [
        sales.rep.tobi.adeyemi@marcus-corp.com,
        sales.manager@marcus-corp.com,
        chief.revenue@marcus-corp.com,
        customer.contact@example.com
      ]
      keywords: ["247811", "Q2 deal", "order intake"]
      date_window: [2026-04-01, 2026-06-30]
    cedar_action: messenger.read_tenant_archive
  mail_threads_request:
    target: mail.tenant_archive
    filters: # same logic
    cedar_action: mail.read_tenant_archive
  workflow_log_request:
    target: workflow_engine.executions
    filters:
      workflow_id: order-to-cash-v3
      invoice_id: 247811
    cedar_action: workflow_engine.read_execution_logs
  audit_chain_proof_request:
    target: audit_chain.seal_evidence
    leaf_filter:
      classes: [PaymentsApprovalGranted, PaymentsApprovalDelegated, PaymentsApprovalReversed]
      invoice_id: 247811
    cedar_action: audit_chain.read_seal_evidence
```

At 09:42 the first job lands. Sam watches the audit-pane progress bar:
"Pulling sample 1/60... Cedar evaluated PERMIT. Audit-chain seal
emitted (audit_id=`audit:b39c4...`). Payments approval chain returned
(4 nodes: order → credit-check → sales-manager → chief-revenue).
Messenger threads returned (12 messages over 14 days). Mail threads
returned (8 messages). Workflow logs returned (1 execution, 4 stages,
all PASS). Audit-chain seal proofs (4 leaves, Merkle path verified)."

The pane shows a "[evidence pack assembled]" green check. The first
sample took 8.4 seconds end-to-end (well under the 60-second p95
target).

## 5. The first hard-boundary encounter — sample 17

At 10:48 WAT, Sam pulls sample 17: invoice 247829, $284,000, sold by
Tobi Adeyemi (mid-level sales rep) to a tier-2 healthcare distributor
based in Munich, Germany. Tobi is the assigned sales rep; the approval
chain is normal (order → credit-check → sales-manager → chief-revenue,
all approved).

When the Messenger threads load, Sam sees something unusual: the
audit-pane shows a small badge at the top right:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Sample 17 — invoice 247829 — Tobi Adeyemi                                │
│                                                                          │
│ Work-tenant Messenger: 18 messages over 11 days [LOADED]                │
│ Personal-tenant principals related: 1 [DENY-BY-DEFAULT — not loaded]    │
│                                                                          │
│ Hover for detail:                                                        │
│   tobi.adeyemi@oyatie.me — personal-tenant; default-deny per ADR-0311.   │
│   Personal-tenant content was NOT read. To access this content, a       │
│   subpoena scoped under ADR-0312 is required. Sam: this surface is      │
│   intentionally invisible to you.                                       │
└──────────────────────────────────────────────────────────────────────────┘
```

Sam pauses. He has worked at four other multinationals before this one,
and at two of them the "audit" tool would have happily exfiltrated
Tobi's personal-tenant Messenger threads — and would have happily
exposed Sam, the corporation, and Tobi to a torrent of ECPA and GDPR
liability. Here, the system says no. Sam exhales.

He does what he is supposed to do: he documents the deny in the audit
workpapers. He clicks "Document deny" and a workpaper template loads:

```
Workpaper: WP-Q2-026-017a
Type: Cedar permit denial (ADR-0311 personal-tenant boundary)
Subject: tobi.adeyemi@oyatie.me (personal tenant)
Reason for denial: personal-tenant resource; B2B_INTERNAL_AUDIT permit
  scope excludes per Cedar default-deny.
Sam's notes: confirmed expected behavior per ADR-0311; no escalation
  required. The work-tenant evidence (18 messages in marcus-corp
  archive) is sufficient for the SOX 404 sample.
Workpaper sealed: audit_id=audit:c47d2... at 2026-07-13T10:51:14Z.
```

The workpaper itself is a sealed audit-chain leaf — Sam's notation
that he saw the deny IS evidence Sam is operating in compliance with
ADR-0311.

## 6. Monday afternoon — the messenger archive reveals an irregularity

Sam continues. Sample 18, 19, 20 — all clean. At sample 21 (invoice
247841, $612,000), the Messenger archive of the work-tenant returns 47
messages over 23 days — far more than the median 12.

Sam reads them. The sales rep (Yusuf Onuoha) and the customer
contact (Klaus Fischer, at a German distributor) had an unusually long
back-and-forth that includes a phrase Sam flags mentally: "as we
discussed offline, we can structure the payment terms so the books
look different for Q1 vs Q2".

Sam's eyebrows rise. "Structure the payment terms so the books look
different" sounds like channel-stuffing — recognizing revenue in a
period it doesn't belong. He doesn't conclude anything yet. He pulls
the workflow-engine execution log for this invoice and sees that the
credit-check stage ran on 31 March (Q1) but the final invoice approval
ran on 2 April (Q2). The amount is large enough to flip
revenue-recognition between quarters.

Sam files a flag in the audit-case (audit-case `ac-marcus-corp-2026-q2-sox-404`)
under finding type `PossibleChannelStuffing`:

```yaml
finding_id: F-Q2-026-021-001
type: PossibleChannelStuffing
subject_invoice: 247841
amount: 612_000
subject_principals_work_tenant: [
  yusuf.onuoha@marcus-corp.com,
  klaus.fischer@germandistributor.example.com (external counterparty)
]
evidence_class: tenant-owned-messenger-thread + workflow-engine-execution-log
recommendation: WALKTHROUGH_INTERVIEW with Yusuf during Wednesday's
  interview day
sealed_at: 2026-07-13T14:12:08Z
sealed_audit_id: audit:f8a31...
```

Sam continues the sample pull. Three more potentially-similar flags
arise over the rest of Monday. He'll triage them on Wednesday.

## 7. Tuesday 14 July — full sample completed, evidence pack assembled

Tuesday is deep sample-pull work. By 17:00 WAT Sam has pulled all 60
samples for RC-04 plus the smaller samples (n=25) for RC-01..RC-07.
The audit-pane shows:

```
Q2 SOX 404 — Marcus Corp — sample completion summary
─────────────────────────────────────────────────────
RC-01 Order intake authorization        ✓ n=25 evidence assembled
RC-02 Credit-check pre-approval         ✓ n=25 evidence assembled
RC-03 Invoice generation matches order  ✓ n=25 evidence assembled
RC-04 Invoice approval                  ✓ n=60 evidence assembled (4 flags raised)
RC-05 Payment receipt matched to inv    ✓ n=25 evidence assembled
RC-06 Revenue-recognition booking       ✓ n=25 evidence assembled
RC-07 Period close                      ✓ n=25 evidence assembled

Total evidence leaves: 1,247 sealed audit-chain leaves
Total Merkle proofs verified: 1,247
Total Cedar evaluations: 4,892 (1,247 PERMITs + 3,645 DENY-by-default
  on personal-tenant principals — all expected)
Total observability emissions: 9,784 traces; 36,217 metric points
Total elapsed time: 13:42 wall-clock (across 2 days)
```

The 3,645 personal-tenant denies are striking. On average, each
sample's Messenger / Mail thread set was correlated to ~3 employees,
each of whom had a personal-tenant principal that surfaced in a
deny-by-default record. None of those personal-tenant principals
exposed any content; only the COUNT was visible to Sam.

That count, in itself, is information. Sam confirms with privacy
counsel (in a quick work-Messenger to legal counsel) that "count of
personal-tenant principals in deny set" is not itself
personally-identifying or behavior-revealing — and counsel confirms
per the privacy review the count is fine because the principal
identifiers are not exposed (only "N personal-tenant principals were
in the deny set for this sample").

## 8. Wednesday 15 July — walkthrough interviews

Tunde and Aisha conduct the walkthrough interviews. Tunde takes
Yusuf Onuoha aside in a private Meet room (work-tenant). Yusuf
explains that the "offline conversation" was about payment-term
structure — the customer wanted to pay 50% in Q1 and 50% in Q2, and
under the revenue-recognition rules the entire invoice books at
delivery in Q2 anyway. There's no channel-stuffing. The "different
books" phrase was loose talk about Q1 cash flow vs Q2 revenue, and
Yusuf agrees the choice of words was poor.

Tunde documents the interview. The finding `F-Q2-026-021-001` is
downgraded from `PossibleChannelStuffing` to
`AmbiguousLanguageInRepCommunication` with a recommendation that
the sales-rep coaching curriculum be updated. Sam concurs and seals
the resolution into the audit-case.

The other three flags are similarly resolved through walkthrough.

## 9. Wednesday late — preparing the read-out

Sam drafts the audit-committee read-out. The PCAOB AS-5 finding
sections:

- **RC-01 Order intake authorization** — control operating
  effectively; no exceptions in n=25.
- **RC-02 Credit-check pre-approval** — control operating
  effectively; no exceptions.
- **RC-03 Invoice generation matches order** — control operating
  effectively; no exceptions.
- **RC-04 Invoice approval** — control operating effectively; 4
  flags raised in n=60 sample, all resolved as ambiguous-language
  findings (no control failure). Recommendation: update sales-rep
  communication coaching curriculum.
- **RC-05 Payment receipt matched to invoice** — control operating
  effectively; no exceptions.
- **RC-06 Revenue-recognition booking** — control operating
  effectively; no exceptions.
- **RC-07 Period close** — control operating effectively; no
  exceptions.

**Overall opinion (preliminary, subject to external auditor
concurrence):** Marcus Corp's revenue-cycle controls are operating
effectively for Q2 2026. No material weakness. No significant
deficiency. Recommendation: minor coaching update.

## 10. Thursday 16 July, 13:00 — read-out with audit committee

Sam presents in oyatie Meet. Marcus, Lin Wei (CFO), Audrey Chen (audit
chair), and two other independent directors attend. Sam shares the
audit pane on screen — the evidence-pack Merkle root is shown live:

```
Q2 SOX 404 Evidence Pack
────────────────────────
Merkle root: 0x9f3c4e2a8d1b...
Leaf count: 1,247
Cedar evaluations: 4,892 (1,247 PERMIT, 3,645 DENY-by-default expected)
Audit-chain seal proofs: all verified
Pack ID: ep-marcus-corp-2026-q2-sox-404
Created: 2026-07-15T18:42:11Z
Signed by: sam.okafor@marcus-corp.com (audit director)
Co-signed by: audrey.chen@marcus-corp.com (audit chair)
```

Audrey asks, "Sam, the 3,645 deny-by-default — what is that?"

Sam answers, "Those are personal-tenant principals — employees'
personal oyatie accounts — that came up in correlation to work-tenant
threads. The Cedar permit grammar, per ADR-0311, forbids my reading
any personal-tenant content. The system surfaced only the COUNT.
Three thousand six hundred forty-five times during the four-day
audit, the system refused to share what an employee was doing on
their own personal account. That's the dual-tenant boundary working.
I'd be sitting in front of an ECPA lawsuit right now if any of those
3,645 had not denied."

Marcus nods. Audrey signs the audit-committee acceptance memo on
screen — the signature is a passkey ceremony.

## 11. Friday 17 July — external-auditor handoff

PwC's external-audit team led by partner Sandra Liu receives the
evidence pack at 14:00 UTC on Friday. Sandra's team verifies the
Merkle root against the audit-chain seal API independently.

"Sam, the evidence pack verifies clean. We'll fold this into our
year-end procedures. Marcus's 404(a) attestation is on solid
ground."

Sam closes the audit-case in his workflow-engine pane. The case
seals with all 1,247 evidence leaves and the read-out memo as the
final leaf.

The Q2 audit is done. Sam takes Friday afternoon off.

## 12. What this story proves (architectural significance)

1. **B2B_INTERNAL_AUDIT permit grammar works end-to-end.** Sam's
   Cedar permit was scoped to one tenant, one quarter, one set of
   controls. Every read fired through the policy gate. Every read
   that should have permitted, permitted. Every read that should
   have denied, denied (3,645 personal-tenant denies — and no false
   positives, no false negatives).

2. **Audit-chain Merkle sealing of the audit work itself.** Every
   one of Sam's 1,247 PERMIT reads emitted a sealed `InternalAuditRead`
   event. The audit work is itself audited. PCAOB AS-5
   sample-traceability is mechanical: any sample row in the
   evidence pack traces to a Merkle leaf hash to a tree root.

3. **Dual-control gate.** Sam could not unilaterally grant himself
   the permit. Audrey Chen (independent audit-chair) had to co-sign.
   This is enforced by Cedar context attribute
   `dual_control_approval_at` — not a process discipline, an
   architectural property.

4. **Per-jurisdiction overlay.** The Munich-based customer in
   sample 17 triggered the EU-WB cross-jurisdiction pack overlay
   (Whistleblower Directive 2019/1937) — but because Tobi (the
   sales rep) is a Nigerian national working from Lagos, the
   NDPR-2023 overlay was the operative pack. The compliance
   µservice composed the right pack set automatically.

5. **The hard boundary holds even on suspicion.** Sam's instincts
   in sample 17 — when he noticed Tobi had a personal-tenant
   thread — could have led him to a permission-creep request.
   The system did not present that option. The deny was final.
   Sam's instinct was correctly redirected to the work-tenant
   evidence, which was sufficient.

6. **Conglomerate scoping (ADR-0313).** Marcus's other subsidiaries
   in the conglomerate (mentioned in passing — there are three more)
   were NOT in Sam's scope, and Sam's Cedar permit did not extend
   to them. If Sam needed to audit a subsidiary, he would need a
   separate permit from THAT subsidiary's audit committee. This is
   the multi-entity boundary working as designed.

## 13. Postscript — the next quarter

Sam's Q3 audit (October 19–22, 2026) is on his calendar. The
workflow-engine has already begun pre-staging the credit-check
control samples. The audit-case template is one click away.

Sam ends the week with a Messenger note to his team:

> "Tunde, Aisha — clean Q2. The dual-tenant boundary held 3,645
> times this week. Whoever designed ADR-0311, send them a beer. —
> Sam"

That note is sealed in the audit-chain as a final leaf. The audit
is over. The system is auditable.

## 14. Cross-references to handshake + IPs + tests

- See `handshake.md` Phase 1 for the per-µservice sequence when Sam
  triggers a single sample pull.
- See `microservices/messenger/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md`
  for the messenger-archive read implementation.
- See `microservices/payments/IP-journey-j137-corporate-internal-audit-sox-controls-test-approval-chain-exporter.md`
  for payments approval-graph export.
- See `microservices/audit-chain/IP-journey-j137-corporate-internal-audit-sox-controls-test-evidence-bundler.md`
  for evidence-pack assembly.
- See `integration-test-plan.md` §2 for the test that verifies the
  3,645-personal-tenant-deny invariant.
- See `microservices/identity/IP-journey-j137-corporate-internal-audit-sox-controls-test-permit-resolver.md`
  for the B2B_INTERNAL_AUDIT principal resolver.

## 15. Operating notes (non-narrative)

- **Performance budget.** Per `docs/standards/cross-microservice-latency-budget.md`
  §4, audit-pull p95 ≤ 60s end-to-end per sample. Sam observed
  8.4s on sample 1; 13s on sample 21 (the heaviest); all under
  budget.
- **Cell-tier.** Sam's work-tenant lives in Tier-3 (regulated
  SOX/PCAOB) per ADR-0248. Read traffic stays within Tier-3.
- **Observability.** The audit run emitted 9,784 traces; the
  `oya_internal_audit_read_count{tenant_id="marcus-corp.tenant"}`
  metric incremented by 1,247; the
  `oya_internal_audit_deny_personal_tenant_count{tenant_id="marcus-corp.tenant"}`
  metric incremented by 3,645. These metrics anchor the SOX
  internal-audit dashboard.
- **Brownout behavior.** Had audit-chain entered brownout per
  ADR-0176, the workflow-engine would have paused sample-pull
  with a `WAITING_ON_AUDIT_CHAIN_SEAL` state and resumed when
  green. Sam would have seen a progress-pause indicator. No
  silent drops.

## 16. Sam's reflective note (locale-aware)

Sam writes a short note to his personal journal (in oyatie Notes
under his PERSONAL tenant — not the work tenant; he is careful
about that):

> Audit week was clean. The new oyatie audit pane is the best
> internal-audit tool I have ever used. The hard boundary at the
> personal tenant means I never had to make a judgement call I
> shouldn't have made. The system makes the right thing the easy
> thing. — Sam

The note lives in Sam's personal tenant, sealed by his personal
audit-chain — not the corporate one. Sam's reflection on the
corporate audit is itself personal, and the system respects that.

## 17. Closing invariants this story enshrines

- One human (Sam) operates across multiple tenant memberships
  without identity fragmentation.
- The audit work is itself audited (audit-chain self-reference is
  not circular because the chain is append-only and the seal is
  Merkle-rooted).
- The personal-tenant boundary holds 3,645 out of 3,645 times. The
  ratio is not "high"; it is exactly 100%.
- The Cedar permit grammar is the architecture, not the policy.
  Policy lives in code (the permit text), not in PDFs.
- The audit-committee chair's dual-control approval is itself a
  Cedar context attribute (`dual_control_approval_at`), not a
  process artifact. The architecture enforces the governance.

## 18. Extended ledger of Cedar evaluations (selected)

Below is a stratified excerpt of Cedar evaluations recorded during
the audit week. Each line is a real evaluation that fired against
the policy gate at api-gateway. Each line is also a leaf in the
evidence-pack Merkle tree.

```
2026-07-13T09:42:08Z PERMIT messenger.read_tenant_archive sample=1  amt=$712,400 lat=82ms
2026-07-13T09:42:08Z PERMIT mail.read_tenant_archive       sample=1                 lat=64ms
2026-07-13T09:42:09Z PERMIT workflow_engine.read_logs      sample=1  invoice=247811 lat=121ms
2026-07-13T09:42:10Z PERMIT payments.read_approval_chain   sample=1  invoice=247811 lat=143ms
2026-07-13T09:42:11Z PERMIT audit_chain.read_seal_evidence sample=1                 lat=58ms
2026-07-13T10:48:32Z PERMIT messenger.read_tenant_archive  sample=17 amt=$284,000   lat=91ms
2026-07-13T10:48:32Z DENY   messenger.read_personal_tenant sample=17 principal=tobi.adeyemi@oyatie.me
                            reason="default-deny per ADR-0311; personal-tenant"
2026-07-13T10:48:33Z PERMIT mail.read_tenant_archive       sample=17                lat=72ms
2026-07-13T10:48:34Z PERMIT workflow_engine.read_logs      sample=17 invoice=247829 lat=109ms
2026-07-13T11:13:08Z PERMIT messenger.read_tenant_archive  sample=21 amt=$612,000   lat=104ms
2026-07-13T11:13:09Z DENY   messenger.read_personal_tenant sample=21 principal=yusuf.onuoha@oyatie.me
2026-07-13T11:13:10Z DENY   messenger.read_personal_tenant sample=21 principal=klaus.fischer@oyatie.me
... [4,889 lines redacted in this excerpt; full ledger at
    audit-chain.api-gateway.marcus-corp.tenant/q2-sox-evidence/cedar.log] ...
2026-07-16T17:31:42Z PERMIT audit_chain.read_seal_evidence sample=60                lat=51ms
```

The 3,645 DENY lines for personal-tenant principals were never visible
as content to Sam. They appeared only as counts in the audit pane and
as opaque hashes in the evidence pack. Sam's read API never returned
any personal-tenant body, sender, recipient, or timestamp — only the
deny event.

## 19. Sam's evidence-pack handoff envelope

The final evidence pack delivered to PwC contains:

```yaml
pack_id: ep-marcus-corp-2026-q2-sox-404
created_at: 2026-07-15T18:42:11Z
sealed_at: 2026-07-15T18:42:14Z
audit_case_ref: ac-marcus-corp-2026-q2-sox-404
authority_chain:
  - audit-charter-v3.pdf (signed 2026-01-04)
  - audit-committee-resolution-2026-q1-001.pdf
  - audit-committee-resolution-2026-q2-001.pdf
signatures:
  - sam.okafor@marcus-corp.com (audit director; passkey ceremony)
  - audrey.chen@marcus-corp.com (audit chair; passkey ceremony)
merkle_root: 0x9f3c4e2a8d1b0f72a934fc8e1d4b62a7c0e8b3a1f9d2c4e6b8a0c2e4d6f8a0b2c
leaf_count: 1247
contents:
  control_test_evidence:
    rc_01: 25 samples, all PASS
    rc_02: 25 samples, all PASS
    rc_03: 25 samples, all PASS
    rc_04: 60 samples, all PASS (4 ambiguous-language flags resolved)
    rc_05: 25 samples, all PASS
    rc_06: 25 samples, all PASS
    rc_07: 25 samples, all PASS
  cedar_evaluations_ledger:
    permits: 1247
    denies_personal_tenant: 3645
    denies_other: 0
  audit_chain_proofs: included as Merkle paths per leaf
  observability_summary:
    p95_pull_latency_ms: 13800
    p99_pull_latency_ms: 28200
    error_rate: 0.0%
    brownout_incidents: 0
external_auditor_handoff:
  recipient: PwC (Sandra Liu, lead partner)
  delivery_method: signed_url_with_passkey_attestation
  delivery_at: 2026-07-17T14:02:18Z
  verification_status: verified-clean by PwC at 2026-07-17T14:08:42Z
```

## 20. Pack overlay composition for this run

The compliance µservice composed the active pack set as follows:

```
Base: pack-corporate-internal-audit-baseline
Region overlay: pack-us-sox-404 + pack-us-sec-disclosure-controls + pack-pcaob-as5
Jurisdiction overlay (per employee residency):
  - Lagos-based employees (Sam, Yusuf, Tobi): pack-ng-data-protection-2023
  - German counterparty involvement: pack-eu-gdpr-cross-border
  - EU customer involvement: pack-eu-whistleblower-2019-1937
Tenant overlay: marcus-corp-tenant-internal-policies
Effective pack stack: 7 packs, composed in priority order
Conflict resolution: none (no contradictions detected)
Audit-pack-stack-snapshot: sealed as ledger leaf
```

## 21. Looking forward — what this enables

With Sam's Q2 audit clean and the audit-pane patterns proven, the
following are unblocked:

- Q3 2026 SOX audit (Oct 19–22) using the same audit-case template.
- Year-end 2026 10-K filing (March 2027) with PwC's external opinion
  citing the four quarterly internal-audit packs as supporting
  evidence.
- ADR-0307 detection-substrate signals can be routed into Sam's
  audit queue as automatic findings — this is the j138 story.
- ADR-0310 case-management primitive proven viable for SOX-class
  audits; can be extended to operational audits, ESG audits, and
  agentic-pipeline reviews.

## 22. Word from Marcus

In the Friday-afternoon all-hands, Marcus mentioned the audit
result: "Clean quarter. Thank you to Sam, Tunde, Aisha, and the
audit committee. Internal-audit done right protects everyone in
this room — including the people we audit. The system that lets
us do it right is the system we built together."

That message lives in the work-tenant Messenger broadcast channel,
sealed with audit ID `audit:m1a2r3...` and visible to all 5,000
employees.

The story ends here. The contract continues.


## Completion expansion — j137 story rigor pass

Scope: quarterly SOX 404 audit of work surfaces only.
Persona: Sam Okafor.
Services: messenger + mail + workflow-engine + payments + audit-chain + ops-dashboard-control-center + identity + compliance.
Applicable ADRs: ADR-0244, ADR-0299, ADR-0311, ADR-0312, ADR-0313, ADR-0319.
The expansion below is intentionally explicit so an intern can build and verify the journey without oral context.

Narrative beat 001: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 002: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 003: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 004: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 005: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 006: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 007: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 008: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 009: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 010: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 011: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 012: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 013: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 014: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 015: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 016: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Checkpoint 01: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Narrative beat 017: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 018: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 019: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 020: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 021: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 022: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 023: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 024: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 025: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 026: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 027: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 028: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 029: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 030: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 031: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 032: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Checkpoint 02: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Narrative beat 033: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 034: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 035: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 036: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 037: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 038: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 039: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 040: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 041: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 042: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 043: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 044: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 045: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 046: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 047: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 048: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Checkpoint 03: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Narrative beat 049: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 050: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 051: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 052: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 053: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 054: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 055: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 056: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 057: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 058: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 059: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 060: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 061: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 062: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 063: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 064: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Checkpoint 04: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Narrative beat 065: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any mail action is accepted.
Boundary assertion 066: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 067: payments emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 068: Sam Okafor sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 069: Sam Okafor advances quarterly SOX 404 audit of work surfaces only; the active tenant label remains visible before any ops-dashboard-control-center action is accepted.
Boundary assertion 070: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 071: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.

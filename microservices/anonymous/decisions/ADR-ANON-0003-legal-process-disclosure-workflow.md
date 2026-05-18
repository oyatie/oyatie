---
id: ADR-ANON-0003
status: Accepted
date: 2026-05-17
microservice: anonymous
deciders: ops-security, general-counsel, council-privacy, council-architecture
owner: ops-security + general-counsel
supersedes: []
superseded_by: []
related:
  - ADR-0028
  - ADR-ANON-0001
related_artifacts:
  - microservices/anonymous/PRD.md (I7, FR-17, FR-18)
  - microservices/anonymous/policy/legal-process-disclosure.cedar
  - microservices/anonymous/runbooks/legal-process-court-order-receipt.md
  - microservices/anonymous/IP-011-legal-process-workflow.md
  - microservices/anonymous/slos/legal-process-disclosure-chain-of-custody-correctness.openslo.yaml
purpose: |
  Define the canonical workflow for honouring court-ordered identity disclosure
  on the anonymous µservice. The workflow MUST preserve the I1 invariant
  (no correlation outside this workflow) while also satisfying ECPA / SCA + UK
  IPA 2016 + KR 통신비밀보호법 + JP 通信の秘密 + EU MLAT obligations.
---

# ADR-ANON-0003: Legal-process disclosure workflow — court-order receipt → counsel review → dual-control approval → 14-day notice (or gag-order) → key-ceremony → disclosure execute → audit-chain seal → transparency-report

## Status

Accepted — 2026-05-17.

## Context

PRD I7 commits the platform to honouring legal-process disclosure (subpoena / warrant / court-order) under a structured workflow that:

1. is the **only** authorised path for `user_id ↔ post_id` correlation (preserving I1);
2. requires **dual-control** (two distinct approvers from distinct organisational units);
3. provides **14-day end-user notice** by default (ECPA §2705(a) US convention) unless the court issues a gag-order (ECPA §2705(b) / UK IPA 2016 §57 / KR 통신비밀보호법 §9-2);
4. produces an **audit-chain Merkle-sealed chain-of-custody hash**;
5. includes the disclosure in the **quarterly transparency report** per EU DSA Art. 27 (or with a gag-order flag where applicable);
6. handles the cross-pack constraint: EU MLAT requests execute in the user's pack, not the requesting pack.

Competing designs:

- **Single-approver workflow** (industry-typical at competitors). Faster; less friction. Rejected as a structural weakness (single point of failure for tenant trust).
- **Multi-step state machine in workflow-engine** with Cedar policy gates at every step. Slower; more friction. Strongest auditability. **Selected.**
- **Centralised disclosure broker** with no Cedar gates (broker is trusted). Faster; one place to enforce. Rejected (single-point-of-failure for tenant isolation).

## Decision

Adopt a **7-step state machine** in the `workflow-engine` µservice, with Cedar policy fragment `policy/legal-process-disclosure.cedar` gating each transition. The 7 steps:

1. **Court-order receipt** (intake). Recorded via `LegalProcessIntake` principal with `record_court_order` action.
2. **Counsel review** (general-counsel). Validates doctype, scope, authority. Can refuse / challenge in court.
3. **Dual-control approval**. Two distinct `LegalProcessApprover` principals from distinct organisational units sign approvals.
4. **User-notice posture decision**. Either 14-day user notice OR court-prohibited gag-order. Per ECPA / SCA + UK IPA 2016 + KR / JP statutes.
5. **Key-ceremony / chain-of-custody init**. Chain-of-custody Merkle hash initialised; signed by both approvers.
6. **Disclosure execute**. `LegalProcessExecutor` performs the correlation (the ONLY platform operation that does); the platform's blind-signature private key set is reconstituted under Shamir 3-of-5 if needed; the disclosure package is built.
7. **Audit-chain seal + transparency-report**. Audit-chain Merkle-seals the chain-of-custody; transparency-report aggregator records the event (with gag-order flag where applicable).

Each step's Cedar policy enforcement is `default-deny` with explicit permits per `policy/legal-process-disclosure.cedar`. The workflow-engine orchestrates the state machine.

Cross-pack constraint: an EU MLAT request from a non-EU requesting state executes within the user's pack (e.g., pack-eu); the disclosure package is delivered to the host-state ministry of justice; the requesting state receives via MLAT channel. **Disclosure data NEVER leaves the user's pack.**

## Alternatives Considered

### A. Single-approver workflow (industry-typical)

- **Pros**: Faster turnaround for law-enforcement; less staff burden.
- **Cons**: Single-point-of-failure for tenant trust; defeats the dual-control invariant; auditors at SOC 2 + ISO 27001 + GDPR DPA all expect dual-control for identity-disclosure operations.
- **Rejected because**: Tenant-trust regression; auditor friction; EU DSA Art. 27 transparency expectation implies procedural rigor.

### B. Centralised disclosure broker without Cedar gates

- **Pros**: Single place to enforce policy.
- **Cons**: Cedar policy at the broker is a god-handler; bug = catastrophic; tenant isolation across packs would require ad hoc broker configuration; auditability is harder.
- **Rejected because**: Cedar-at-every-step is the audit-grade pattern.

### C. Manual paper-only workflow (skip state machine; lawyers handle)

- **Pros**: No software exposure to legal-process surface.
- **Cons**: No auditable chain-of-custody; no transparency-report aggregator; cannot scale; auditor friction.
- **Rejected because**: Scaling + audit-grade rigor require the state machine.

### D. State machine without 14-day notice (always gag by default)

- **Pros**: Simpler.
- **Cons**: ECPA §2705(a) convention is 14-day notice unless court-prohibited. Skipping notice is a regulatory regression for US tenants and conflicts with EU DSA Art. 14 user-rights.
- **Rejected because**: Statutory convention requires notice.

### E. State machine with single user-notice default (no gag support)

- **Pros**: Maximal user-side transparency.
- **Cons**: Cannot honour court-issued gag-orders (UK IPA §57 + ECPA §2705(b) + KR §9-2); platform would face contempt-of-court exposure.
- **Rejected because**: Statutorily required to support gag-orders.

## Consequences

### Positive

- **I7 invariant structurally enforced.** Disclosure is the ONLY operation that correlates; it requires dual-control + chain-of-custody + audit-chain seal.
- **I1 invariant preserved.** Outside the legal-process workflow, no platform code path correlates.
- **Auditability**: Every step is a workflow-engine event sealed to audit-chain.
- **Transparency**: Quarterly transparency report includes counts by doctype + jurisdiction; gag-order entries contribute to anonymised aggregate.
- **Cross-pack constraint enforced.** MLAT requests execute in user's pack; data never crosses pack boundary.

### Negative

- **Higher operational cost** per disclosure vs single-approver. Mitigated: dual-control is the regulatory + tenant-trust expectation; the cost is justified.
- **Slower turnaround for law-enforcement** than the industry norm (single-approver). Mitigated: 7-step workflow can complete in 24-48h under non-urgent posture; emergency paths (e.g., NCMEC, KR §9-2) have expedited variants per `runbooks/legal-process-court-order-receipt.md`.
- **Workflow-engine + audit-chain dependency** = if either is down, disclosure cannot execute. Mitigated: both are P0 µservices with 99.99% availability target.

### Operational

- `runbooks/legal-process-court-order-receipt.md` covers 6 paths (A US, B UK, C KR, D JP, E NCMEC, F-MLAT).
- IP-011 implements the state machine.
- Quarterly transparency-report aggregator emits report.
- Annual disclosure-process audit by external counsel.

### Regulatory

- **ECPA / Stored Communications Act §§2701-2712**: §2703(d) court-order honoured; §2705(a) 14-day notice OR §2705(b) gag-order honoured.
- **UK Investigatory Powers Act 2016 §§70-71 + §57**: targeted-interception + targeted-equipment-interference warrants honoured; gag-order default.
- **KR 통신비밀보호법 §9 + §9-2**: standard + emergency disclosure honoured; PIPC retroactive validation procedure documented.
- **JP 通信の秘密 (Constitution Art. 21) + Telecom Business Act**: court-order honoured; default gag-order.
- **EU MLAT + DSA Art. 27**: cross-border requests execute in user's pack; quarterly transparency report.
- **GDPR Art. 6(1)(c)** (legal obligation): legal basis for the processing during disclosure.
- **18 USC §2258A NCMEC**: CyberTipline path E expedited.

### Invariant Preservation

I7 is structurally satisfied by this decision. I1 is preserved because the only correlation path is gated by this workflow + Cedar policy.

## References

- ECPA / Stored Communications Act §§2510-2523, 2701-2712 (US)
- 18 USC §2258A (NCMEC CyberTipline)
- UK Investigatory Powers Act 2016 §§57, 70-71
- KR 통신비밀보호법 (Telecommunication Secrecy Act) Arts. 9, 9-2
- JP Constitution Art. 21 (通信の秘密); Telecom Business Act
- EU DSA Reg. 2022/2065 Art. 27 (transparency report)
- EU MLAT framework
- ADR-0028 (audit-chain Merkle / Ed25519)
- ADR-ANON-0001 (cryptographic-blinding protocol)
- Two-Person Rule (industry pattern; cryptographic-ceremony precedent at CA root-key + Bitcoin custodians)

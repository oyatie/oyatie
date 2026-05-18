---
id: ADR-NET-0003
status: Accepted
date: 2026-05-17
microservice: network
deciders: council-architecture, ops-security, axis-network, axis-messenger, council-privacy
owner: axis-network + axis-messenger
supersedes: []
superseded_by: []
related:
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-MSGR-0004
  - ADR-NET-0001
related_artifacts:
  - microservices/network/PRD.md (FR-14, FR-33)
  - microservices/network/policy/professional-context-isolation.md (PCI-09)
  - microservices/network/runbooks/inmail-fanout-degraded.md
  - microservices/network/slos/inmail-send-latency.openslo.yaml
purpose: Establish the InMail bridge from `network` to `messenger` µservice — Professional-tier-only routing; never federates to Personal-tier DM; throughput + rate-limit + spam-classifier discipline aligned with sibling messenger ADR-MSGR-0004 (mail ADR-MAIL-0004 pattern reference for spam-classifier).
---

# ADR-NET-0003: InMail bridge to messenger — Professional-tier-only; never Personal-tier DM; per-tenant rate budget + spam classifier + audit-chain seal

## Status

Accepted — 2026-05-17.

## Context

`network` ships an InMail feature: a Professional-context premium messaging surface where a user can send a message to another Professional user **with whom they are not connected**. LinkedIn's InMail product is the precedent; tenants expect this as a competitive parity feature for Professional networking.

InMail must reach the recipient through the `messenger` µservice's Professional-tier surface (the messenger µservice owns canonical DM delivery + reading + receipts). The bridge is the integration boundary.

Key constraints:

1. **Professional-tier-only invariant** (PCI-09): the InMail-bridge must NEVER deliver to messenger's Personal-tier DM surface. This is enforced compile-time + runtime + Cedar + at messenger µservice's bridge-receive endpoint.
2. **Rate budget**: per-tenant InMail send-rate budget (default 250/day production tier); per ADR-0140 (retired per ADR-0145) default-deny posture, additional sends rejected at REST handler.
3. **Spam classifier**: every InMail body classified by foundry-runtime spam-classifier before bridge dispatch; spam verdicts are surfaced to user but do NOT auto-drop (false-positive avoidance — recruiters often write similar-looking InMails); high-confidence spam verdicts queue for human-review.
4. **Audit-chain seal**: every send + delivery + read receipt sealed for retention + employment-law audit.
5. **Cross-µservice contract**: the bridge protocol must be contract-versioned per ADR-0131 µservice-boundary discipline; future schema evolution managed via dual-version window.
6. **InMail rate-budget management**: tenant-admin can adjust budget per Cedar `tenant-admin` entitlement.
7. **Minor-account protection**: minor accounts may not receive InMails from un-connected adults (Cedar FORBID per `tenant-scope.cedar`).
8. **Aligned with sibling messenger ADR-MSGR-0004 + mail ADR-MAIL-0004 spam-classifier pattern**: spam-classifier shape is reused; messenger µservice owns the canonical spam-classifier-verdict model.

## Decision

oyatie network's InMail-bridge BC implements the following:

1. **Bridge crate**: `oya-network-inmail-bridge-{kernel,domain,usecase,api,adapter,adapter-messenger-bridge,worker,sdk}`.
2. **gRPC contract** to messenger µservice: `InMailBridgeService` (defined in `contracts/proto/network.proto`); contract version v1 in P01; future v2 dual-version-window per ADR-NET-0004 pattern.
3. **Professional-tier-only enforcement**:
   - Compile-time: kernel port trait `InMailBridge::send(inmail: InMail)` accepts only `InMail` (which is bound to `context_kind: Professional` per ADR-NET PCI-01).
   - Runtime: bridge worker checks messenger-response carries `context_kind: Professional` AND `inmail_target_channel: professional`; mismatch is Sev-1 (PCI-09).
   - Cedar (`tenant-scope.cedar` PERMIT 9): messenger principal can only receive InMail with `inmail_target_channel: professional`.
   - messenger µservice's bridge-receive endpoint mirrors the Cedar policy.
4. **Per-tenant rate budget**:
   - Default tier production: 250 InMails/day/account.
   - Trial: 5/day.
   - Sandbox: 25/day.
   - Internal: 2500/day.
   - Enforced at REST handler via Valkey token-bucket per tenant_id × user_ref.
5. **Spam classifier**:
   - foundry-runtime spam-classifier invoked before bridge dispatch.
   - Verdict surfaced to sender: "Your message may be flagged as promotional. Send anyway? [Y/N]".
   - High-confidence spam verdicts route to a manual-review queue; recipient receives notification "An InMail is awaiting review".
   - Verdict + audit-chain seal recorded per send.
   - Classifier bounds per `runbooks/recruiter-classifier-rollback.md` §"ranker fallback" pattern.
6. **Audit-chain seal**: every `InMailSent`, `InMailDelivered`, `InMailRead` event sealed per Bominal ADR-0028.
7. **Backpressure**: per `runbooks/inmail-fanout-degraded.md`, queue holds in Valkey Streams (Redis wire-compat); per-tenant rate-degradation when messenger µservice or spam-classifier degraded.
8. **Minor-account FORBID**: Cedar `tenant-scope.cedar` `forbid send_inmail when resource.minor_protect == true && context.sender_connected_to_target == false`.
9. **Recipient opt-out**: recipient profile flag `inmail_opt_out: bool`; when true, sender receives 403 + opt-out message. Per pack-eu, this is the default for minor accounts; tenants may default-on for all accounts.
10. **eDiscovery hold**: tenant-admin may issue hold on InMail thread per `tenant-scope.cedar` PERMIT 6.

## Alternatives Considered

### A. Don't bridge; build InMail-native messaging stack inside `network`

- Pros: no cross-µservice complexity; full control of DM stack inside `network`.
- Cons: duplicates messenger µservice's investment in DM (per ADR-0135 messenger is the canonical DM µservice); two DM stacks to maintain; tenant tooling fragmentation; ADR-0131 µservice-boundary discipline argues against duplication.
- Rejected.

### B. Bridge to messenger but allow Personal-tier DM delivery as fallback

- Pros: greater delivery surface area; user receives InMail in their primary DM inbox regardless of context.
- Cons: violates PCI-09 + ADR-0135 Professional-context invariant; would create a Personal-tier privacy leak (recruiter sees Personal-DM inbox); regulatory + privacy violation; gross security violation.
- Rejected: this is exactly what PCI-09 forbids.

### C. Email-bridge to user's primary email instead of DM (mail µservice bridge)

- Pros: bypasses DM-bridge complexity; user receives standard email.
- Cons: doesn't match LinkedIn InMail UX (real-time + persistent thread); email-spam-tarpit-loss; mismatched expectation.
- Partial accept: digest-mode InMail summary can be sent via mail µservice for users who prefer it; primary delivery remains messenger-bridge.

### D. No spam classifier (trust the sender)

- Pros: lowest complexity; lowest false-positive friction.
- Cons: InMail abuse is a known threat in LinkedIn-class platforms (recruiter-spam, scams, romance-baits); without spam-classifier, the surface degrades quickly. Industry precedent (LinkedIn, Xing, Wantedly) all classify.
- Rejected.

### E. Hard-cap InMail per-day per-account at 25 (very conservative)

- Pros: lowest abuse risk.
- Cons: too low for enterprise recruiter use; tenants will churn to LinkedIn Recruiter.
- Partial accept: 250/day is the production default; tenant-admin can adjust within per-tenant cap.

## Consequences

### Positive

- InMail delivery routes through messenger µservice's canonical DM surface; no duplicate stack.
- Professional-tier-only invariant enforced at 4 layers (compile-time + runtime + Cedar in network + Cedar in messenger).
- Per-tenant rate budget protects against runaway abuse + per-user spam vector.
- Spam classifier surfaces risk to sender + recipient; manual-review queue catches edge cases.
- Audit-chain seal supports eDiscovery + employment-law audit + KR 통신비밀보호법 four-eyes posture.
- Contract-versioned bridge supports future evolution.

### Negative

- Cross-µservice dependency on messenger µservice availability; bridge degrades gracefully per `runbooks/inmail-fanout-degraded.md`.
- Spam classifier has false-positive friction; mitigated by surface-to-sender + manual-review queue.
- Contract-version drift between network + messenger could create FM-22 Sev-2 events; mitigated by `oya-gate validate inmail-bridge-contract` CI lane + dual-version window discipline.
- Bridge worker is a stateless dispatch; messenger µservice's own SLO is the binding constraint for end-to-end p95.

### Operational

- Cargo workspace: `oya-network-inmail-bridge-*` crates per BNF v4.1.
- gRPC contract: `contracts/proto/network.proto` §"InMail Bridge".
- Helm: `inmailBridgeWorker.replicas: 3` per `iac/helm/network/values.yaml`.
- LEAN lane: `oya-check-professional-context-isolation` validates PCI-09; `oya-gate validate inmail-bridge-contract` validates contract-version.
- SLO: `network-inmail-send-latency` (p95 ≤ 100ms for bridge dispatch).
- Runbook: `inmail-fanout-degraded.md`.

### Regulatory

- **KR 통신비밀보호법** (Communications Secrecy Act): InMail intercept only via four-eyes audit; covered by audit-chain + `policy/auditor-scope.cedar` PERMIT 7 four-eyes flow.
- **EU GDPR Art. 7 + Art. 21**: recipient opt-out flag (`inmail_opt_out`).
- **EU ePrivacy Directive Art. 13**: B2B exception applies for first contact with prior business contact; tenant onboarding documents the lawful-basis position.
- **CAN-SPAM Act (US)**: when InMail is promotional, tenant-admin must comply with sender-identification + unsubscribe; bridge does not auto-comply (tenant duty); SDK helper `formatCANSPAMUnsubscribe` provided.
- **GDPR Art. 22** (when InMail is auto-suggested by recruiter-stub): inherits ADR-NET-0002 opt-out + human-review surface.
- **Minor-account protection**: Cedar FORBID covers COPPA + GDPR Art. 8 + KR 청소년 보호법.

## References

- ADR-0135 (Connect dissolution, parallel).
- ADR-0131 (per-microservice flat layout; bridge crate convention).
- ADR-0132 (suite-and-bundle dissolution).
- ADR-MSGR-0004 (sibling messenger Professional-tier surface; paired pattern; assumed to be authored in messenger µservice).
- ADR-MAIL-0004 (mail spam-classifier pattern; reference for spam-classifier discipline; assumed to be authored in mail µservice).
- ADR-NET-0001 (storage; InMail body stored under tenant-DEK).
- ADR-NET-0002 (recommender bounds; recruiter-stub may auto-suggest InMail).
- `microservices/network/policy/professional-context-isolation.md` PCI-09.
- `microservices/network/runbooks/inmail-fanout-degraded.md`.
- `microservices/network/slos/inmail-send-latency.openslo.yaml`.
- `microservices/network/capacity-model.md` §"Per-Tenant Limits".
- KR 통신비밀보호법; EU GDPR Arts. 7, 21, 22; EU ePrivacy Directive Art. 13; CAN-SPAM Act 15 USC §§7701–7713.
- LinkedIn InMail product docs `learn.microsoft.com/linkedin`.

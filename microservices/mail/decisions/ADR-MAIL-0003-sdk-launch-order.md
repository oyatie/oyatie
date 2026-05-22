---
id: ADR-MAIL-0003
status: Accepted
date: 2026-05-17
microservice: mail
deciders: axis-mail, axis-developer-experience, council-architecture
owner: axis-mail
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-MAIL-0002
related_artifacts:
  - microservices/mail/PRD.md (Open Question 1 + Open Question 5; FR-03)
  - microservices/mail/sdk-plan.md
  - microservices/mail/contracts/openapi/
purpose: Close PRD-mail Open Question 1 (JMAP vs IMAP priority) and Open Question 5 (SDK first-launch ordering) — fix the canonical protocol and the SDK ship sequence.
---

# ADR-MAIL-0003: SDK launch order — native JMAP for Swift, JMAP-jam wrapper for TypeScript, IMAP4rev2 as fallback after JMAP feature-parity

## Status

Accepted — 2026-05-17.

## Context

PRD-mail FR-03 mandates IMAP + JMAP + REST mailbox-read surfaces; the µservice owns the protocol-edge entirely. Open Question 1 ("JMAP vs IMAP priority for client SDK first-launch") and Open Question 5 ("SDK ship sequence") are entangled and unresolvable separately. The PRD's `sdk-plan.md` defers the ordering decision to this ADR.

The market situation at M03 launch:

- **iOS / macOS clients** — Apple Mail is the default mail client for ~55% of iOS users (Litmus 2025 H1) and ~25% of macOS desktop mail. Apple Mail speaks IMAP4rev1 / IMAP4rev2 natively; JMAP support is absent from Apple Mail and unlikely to appear until Apple ships its own JMAP push notification adapter. Third-party iOS clients (Spark, Edison, Airmail) speak both. Native Swift SDKs are how high-end iOS productivity apps deliver native-feel UX; React Native + JS bridges feel laggy in mail-list scrolling and search.
- **Web / Node** — JMAP (RFC 8620 + 8621) is the modern protocol: HTTP/JSON, batched, push-friendly via SSE, much friendlier than IMAP4 to web ecosystems. FastMail's `jmap-client-ts` and the IETF `jmap-jam` reference wrapper are widely used. TypeScript SDK consumers are dominated by web apps and Node middleware that prefer JSON over the binary-tagged IMAP4 wire format.
- **Android** — Android mail clients vary widely; the dominant pattern is IMAP+OAuth. Native Kotlin SDK is desirable but lower priority than iOS at M03 (KR market launch is ~70% iOS for the high-end enterprise productivity segment per IDC KR 2025 data).
- **Desktop (Thunderbird, etc.)** — IMAP-first; JMAP support is recent (Thunderbird 115+) but considered experimental.

A naïve "ship all SDKs simultaneously" doubles the initial maintenance surface without commensurate market reach. A naïve "IMAP-first universal" stunts the modern client ecosystem (JMAP web clients, push notifications, batched operations) by deferring JMAP into a second wave that historically never ships on time. A naïve "TypeScript first" leaves Apple Mail / Spark / Edison consumers on a lower-priority path on day one and effectively concedes the iOS-heavy KR launch market.

The µservice already commits to all three protocols on the wire (FR-03); this ADR is solely about which SDK ships first, second, third, and which protocol each SDK speaks first.

## Decision

oyatie mail SDKs launch in this sequence and protocol order:

1. **Wave 1 (M03 launch wave)**:
   - **Swift SDK — native JMAP (RFC 8620 + 8621).** Apple ecosystem clients that adopt oyatie SDK skip IMAP entirely and speak JMAP to the `oya-mail-imap-frontend` JMAP endpoint. JMAP's batched operations + push-via-EventSource give native-feel UX on iOS that IMAP cannot match. Bundled with Apple Mail bridging guidance (oyatie ships a separate `oya-mail-imap-frontend-app` IMAP server for users who refuse to install the SDK; that IMAP path is feature-parity but not the recommended path).
   - **TypeScript SDK — JMAP wrapper (jmap-jam style).** Built on top of `jmap-jam` (or an oyatie fork pinned at `@oyatie/mail-sdk/jmap-jam@1.x`); web + Node share the JS ecosystem. JMAP-over-HTTP/JSON is the natural fit; no IMAP4 binary protocol in the TS SDK at any point.
2. **Wave 2 (M03-onward1 quarter)**:
   - **IMAP4rev2 fallback SDK + reference docs.** Only AFTER JMAP coverage reaches feature parity (Push, EmailSubmission, Sieve, Search, Identity, MailboxQuery) does oyatie publish an IMAP4rev2-only client guide for third-party clients (Thunderbird, mobile clients that refuse JMAP, legacy MUAs). The µservice always serves IMAP4rev2 on the wire (FR-03 invariant); this is about which SDK we publish.
   - **Kotlin (Android) SDK — native JMAP.** Mirrors the Swift posture; AndroidX-flavored.
3. **Wave 3 (M03-onward2 quarters)**:
   - **Python + Go SDKs — JMAP wrappers** for backend automation use cases (tenant migration, eDiscovery export tooling, bulk-mail workflows).
4. **Protocol invariant**: every SDK speaks JMAP as its primary protocol. IMAP4rev2 stays a wire-protocol commitment (the µservice serves it), not a primary SDK target. REST endpoints exist for narrow tenant-automation use cases not for end-user clients.
5. **Apple Mail bridging**: the µservice ships a standards-conformant IMAP4rev2 server on `:993` (implicit TLS) so users who prefer Apple Mail get full mailbox access without any oyatie SDK on the client. This is the *minimum* IMAP commitment; SDKs aim higher.

## Alternatives Considered

### A. IMAP4-first universal SDK (Swift + TS + Kotlin all ship IMAP4 first)
- Pros: maximum compatibility on day one; lowest learning curve for engineers familiar with classical mail.
- Cons: stunts modern-client capability (no batched ops, no native push, awkward search); JMAP becomes a "later" project that historically slips; competitors (FastMail, JMAP-native Stalwart consumers) ship JMAP-first SDKs and out-feature oyatie on day one.
- Rejected: violates "industry-leader competitive parity" mandate at the protocol-stack layer.

### B. Simultaneous Swift + TS + Kotlin + Python launch at M03
- Pros: maximum surface coverage at launch.
- Cons: doubles the initial maintenance + documentation + bug-triage surface; SDK developer-experience suffers because no one SDK gets the polish budget; SDK regressions on launch are catastrophic for adoption.
- Rejected: contradicts the "polish, then expand" principle; the Wave 1/2/3 staging protects the early SDK quality bar.

### C. TypeScript-first (web + Node ship; iOS Swift waits)
- Pros: largest single-platform reach (web is the broadest surface); fastest time to broad availability.
- Cons: iOS-heavy KR launch market gets a lower-priority path on day one; high-end productivity buyers will perceive oyatie mail as "not serious about Apple"; Apple Mail bridging alone is not enough for native-feel UX inside third-party iOS clients that adopt the SDK.
- Rejected: KR launch market composition makes Swift a deal-breaker at Wave 1.

### D. JMAP-only forever (no IMAP4rev2 SDK guide ever)
- Pros: cleanest protocol posture; one wire protocol to maintain in the SDK story.
- Cons: third-party IMAP-only clients (Thunderbird users, niche enterprise MUAs, on-prem appliances) get no SDK path; the wire-protocol commitment to IMAP4rev2 becomes a lower-priority surface with no documentation.
- Rejected: undermines the FR-03 commitment to IMAP4rev2 as a first-class wire protocol.

### E. REST-first SDK (oyatie's own JSON API rather than JMAP)
- Pros: simplest for tenant-automation consumers; aligned with REST conventions other oyatie µservices use.
- Cons: forks the SDK from the standards-conformant mail-client ecosystem; means oyatie SDK is "yet another proprietary mail API" instead of a standards-conformant JMAP client; locks consumers into oyatie's API shape rather than IETF JMAP shape.
- Rejected: contradicts the entire vendor-coupling-refusal posture of PRD-mail.

## Consequences

### Positive

- iOS-first KR launch supported on day one with a native Swift SDK that gives Apple ecosystem clients native-feel UX through JMAP.
- Web + Node consumers get a JS-ecosystem-conformant JMAP client (jmap-jam wrapper) that re-uses the broader IETF JMAP tooling ecosystem.
- JMAP-first posture aligns with ADR-MAIL-0002 backend choice: Stalwart is JMAP-native and Postfix+Dovecot can be bridged via the JMAP-bridge adapter; the SDK story is uniform across backends.
- IMAP4rev2 wire-protocol commitment preserved (the µservice serves it on `:993`); Apple Mail / Thunderbird / legacy clients keep working without any oyatie SDK on the client.
- Wave-2 IMAP SDK + Kotlin SDK ship only after JMAP coverage reaches parity, preventing a "ship-everything-half-baked" anti-pattern.

### Negative

- Three Wave-1 + Wave-2 SDKs to maintain (Swift, TypeScript, IMAP4rev2 reference, Kotlin). Mitigated by the JMAP-jam-shared core + protocol-conformance contract tests that run once and cover all SDK wrappers.
- IMAP4rev2 SDK arrives in Wave 2, meaning early third-party-client integrators have to build against the wire protocol with no oyatie-side SDK help for ~1 quarter. We mitigate with a published JMAP+IMAP feature-parity matrix and a first-wave wire-protocol conformance suite consumers can self-test against.
- JMAP push (`@push:` URL push channel per RFC 8620 §7.3) requires HTTP server-side infrastructure beyond IMAP IDLE; mitigated by re-using the µservice's WebSocket gateway (per messenger PRD) for the push channel; cross-µservice import refused by LEAN-A2 so push runs via the `audit-chain`-compatible event bus instead.
- Python + Go SDKs slip to Wave 3; tenant-automation consumers needing those languages early use raw JMAP HTTP/JSON for one extra quarter.

### Operational

- New cargo workspace members at Wave 1: `microservices/mail/src/packages/oya-mail-sdk-swift/` (Swift Package Manager target) + `microservices/mail/src/packages/oya-mail-sdk-typescript/` (npm `@oyatie/mail-sdk`). Per ADR-0131 each µservice's `src/` is the canonical code root for path-based ownership; Swift + TS packages live there.
- Conformance tests: every SDK runs against the µservice's contract tests in `microservices/mail/contracts/openapi/` + a JMAP-conformance harness; lane `mail-sdk-protocol-conformance` BLOCKS merges of SDK PRs that regress the protocol-conformance matrix.
- SDK docs published at `microservices/mail/sdk-plan.md` + per-SDK READMEs; the SDK plan doc moves from "draft" to "operative" with this ADR.
- Apple Mail bridging UX guide (`microservices/mail/runbooks/apple-mail-bridging.md`) added in IP-M03-MAIL-SDK-001.

### Regulatory

- **RFC 8620** (JMAP Core) + **RFC 8621** (JMAP Mail): conformance is the SDK's core obligation; tested per the JMAP conformance harness; conformance evidence emitted in audit-chain on every SDK release.
- **RFC 9051** (IMAP4rev2): wire-protocol commitment preserved; Apple Mail bridging path covered.
- **RFC 5598** (Internet Mail Architecture): respected; the SDK story treats the µservice as the "MS" (message store) in the architecture rather than re-inventing it.
- **GDPR Art. 25** (data protection by design): JMAP's batched operations + scoped tokens give scope minimisation a first-class API surface; SDK design honours this.
- **No EU AI Act implications** (SDK is non-AI; AI features are gated through separate consent paths per `capabilities/T1-assist.yaml`).

## References

- RFC 8620 — JMAP Core
- RFC 8621 — JMAP Mail
- RFC 9051 — IMAP4rev2
- RFC 5598 — Internet Mail Architecture
- IETF JMAP working group — `https://datatracker.ietf.org/wg/jmap/`
- FastMail JMAP rationale + jmap-client-ts — `https://www.fastmail.com/developer/`
- jmap-jam reference wrapper — `https://github.com/jmapio/jmap-jam`
- Apple Mail IMAP documentation — Apple Developer (Mail extensions + IMAP usage)
- Stalwart JMAP-native implementation — `https://stalw.art/docs/jmap`
- Thunderbird JMAP experimental support release notes (115.x)
- Litmus Email Client Market Share 2025 H1
- IDC Korea Mobile OS Share Report 2025
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-suite-and-bundle dissolution
- ADR-MAIL-0002 — Mail-server backend per tenant_class and workload profile (paired choice)
- `microservices/mail/PRD.md` Open Questions 1 + 5
- `microservices/mail/sdk-plan.md`
- `microservices/mail/contracts/openapi/`

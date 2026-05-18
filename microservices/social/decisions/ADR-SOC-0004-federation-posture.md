---
id: ADR-SOC-0004
status: Accepted
date: 2026-05-17
microservice: social
deciders: council-architecture, ops-security, axis-social, council-privacy, gtm-customer-success
owner: axis-social
supersedes: []
superseded_by: []
related:
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-SOC-0005
  - ADR-MSGR-0004
related_artifacts:
  - microservices/social/PRD.md (Open Question 2 + Open Question 3)
  - microservices/social/policy/dual-context-isolation.md (DCI-08)
  - microservices/social/policy/public-read.cedar
  - microservices/social/policy/data-residency.md
  - microservices/social/runbooks/federation-bridge-degraded.md
  - microservices/social/threat-model.md (T-S-03, T-I-08, T-D-07, T-D-S-13)
  - microservices/social/dpia.md (R-08, R-17)
purpose: Establish ActivityPub federation posture — OFF by default; per-tenant opt-in for Professional-tier only; Personal-tier NEVER federates (compile-time invariant DCI-08).
---

# ADR-SOC-0004: Federation posture — ActivityPub OFF by default; per-tenant opt-in for Professional-tier only; Personal-tier NEVER federates (compile-time invariant DCI-08)

## Status

Accepted — 2026-05-17.

## Context

PRD §"Functional Requirements" FR-29 + Open Question 2 ask: what is the federation posture for the social µservice? Sibling industry products demonstrate three approaches:

- Twitter/X / Threads / Instagram / Facebook: NO federation (closed platforms; Threads has announced ActivityPub interop but as of 2026-05-17 it's tenant-tier opt-in for select accounts).
- Mastodon, Lemmy, PixelFed: federation by default via ActivityPub (W3C Recommendation 2018-01-23).
- Bluesky: federation via AT Protocol (different protocol; reaches different ecosystem; PRD Open Question 2 considers AT Protocol as a successor-IP).
- Matrix / Element (messenger): federation via Matrix Client-Server protocol (per ADR-MSGR-0004).

ActivityPub is the most widely-deployed open federation protocol for social platforms (~10M+ active users on Mastodon ecosystem; W3C Recommendation status). It's well-suited to oyatie's dual-context model with one critical constraint: ActivityPub assumes public-by-default semantics with optional follower-restricted distribution; it has no built-in concept of "Professional vs Personal context."

Key tensions:

1. **Dual-context invariant (parallel ADR-0238 + DCI-08)**: Personal-tier user data must NEVER cross pack borders without explicit user consent; federation egress is by definition a cross-pack flow.
2. **Regulatory residency (pack residency per `policy/data-residency.md`)**: Cross-pack data flow requires SCC for GDPR-scope tenants + tenant-level opt-in for KR PIPA Art. 28 cross-border consent.
3. **Federation security (T-S-03 forged @-mention; T-I-08 personal-tier leak; T-D-07 federation peer flood)**: untrusted peers can send spam, malicious content, or attempt protocol-level attacks.
4. **HIPAA Safe Harbor (pack-us-healthcare)**: federation by default would create per-peer covered-entity relationship complications; default-OFF is the safer posture.
5. **EU DSA Art. 14 transparency**: per-tenant terms-of-service must disclose federation peer relationships if active.
6. **Tenant choice**: enterprise tenants (LinkedIn-class) may prefer no-federation closed platform; consumer-tenant Personal-tier users (Twitter/X-class) may prefer federation; the answer is per-tenant + per-tier configuration.

The decision needs to: (a) pick a federation strategy that respects DCI-08 + pack residency, (b) define opt-in mechanism + audit-chain trail, (c) align with messenger's ADR-MSGR-0004 federation pattern for cross-µservice consistency, (d) leave AT Protocol open as a future successor-IP (PRD Open Question 2), (e) define operational posture (peer allowlist, HTTP Signatures, rate-limit, runbook).

## Decision

oyatie social adopts a **conservative + opt-in + tier-shaped federation posture**:

1. **Federation OFF by default for all packs + tenants + tiers.**
   - `federationGateway.enabled: false` in Helm values; no outbox / inbox processing until explicitly enabled per tenant.
   - This is a global default across all 11 packs; pack-us-healthcare is permanently locked to default-OFF (HIPAA Safe Harbor).
2. **Per-tenant opt-in is the only activation mechanism.**
   - Tenant-admin invokes opt-in via `POST /tenants/{tenant_id}/federation/opt-in` (Slice B endpoint; not in P01 OpenAPI).
   - Opt-in requires (a) SCC + transfer-register entry per `policy/data-residency.md`, (b) peer allowlist with at-least-one peer attested, (c) regulatory acknowledgement (EU DSA Art. 14 + per-pack overlay).
   - Opt-in is sealed via audit-chain Ed25519; opt-out is sealed similarly.
3. **Personal-tier NEVER federates (compile-time invariant DCI-08).**
   - The `federation-gateway` outbox port trait `FederationOutbox::publish(post: ProfessionalPost)` accepts only `ProfessionalPost`; passing `PersonalPost` is a compile-time type error.
   - Runtime guard belt-and-suspenders: federation worker checks `post.context_kind == Professional` and emits Sev-1 (`oya_social_personal_tier_federation_attempt_total > 0`) if violated.
   - LEAN lane `oya-check-federation-personal-tier-refused` validates the type-system constraint at every PR.
   - Cedar `policy/public-read.cedar` FORBID rule belt-and-suspenders forbids Personal-context delivery.
4. **Professional-tier federation egress: opt-in tenant only.**
   - Outbox dispatches `Create`, `Update`, `Delete`, `Follow`, `Like`, `Announce`, `Undo` ActivityPub Activities (W3C Recommendation 2018) to allowlisted peers.
   - HTTP Signatures (RFC 9421) per egress.
   - Per-peer rate limit (default 1k req/min).
   - Audit-chain seal per egressed Activity.
   - Pack residency: egress is per `policy/data-residency.md` cross-border-transfer rules; SCC + per-tenant attestation required.
5. **Inbox ingestion accepted from allowlisted peers only.**
   - HTTP Signatures verification required; sig-verify failure → reject + log.
   - Per-peer rate limit (default 1k req/min during normal; reduce to 100/min during attack-mode).
   - Mass-spam from peer → `social_federation_peer_spam_rate` metric → trigger `runbooks/federation-bridge-degraded.md` Path B (peer compromise).
6. **No AT Protocol in P01 / P02.**
   - PRD Open Question 2 (AT Protocol in addition to ActivityPub) is scheduled-for-distinct-tracked-work to ADR-SOC successor-IP after federation minimum-shippable-tier (ActivityPub) ships.
   - Bluesky is the principal AT Protocol implementation; if oyatie's strategic direction shifts toward AT Protocol primary, file successor-IP ADR.
7. **Alignment with sibling messenger ADR-MSGR-0004.**
   - Both ADRs share: opt-in only, Personal-tier never federates, peer allowlist, HTTP Signatures, audit-chain seal.
   - Differences: messenger federation focuses on Matrix Client-Server protocol (more team-collaboration native); social federation focuses on ActivityPub (more public-social native).
8. **Operational substrate.**
   - `oya-social-federation-gateway-*` BC (P02 IP) implements outbox + inbox; the BC exists in P01 codebase but is OFF by default.
   - `oya-social-federation-gateway-adapter-activitypub` backend per ADR-0105 Amendment 3.
   - `runbooks/federation-bridge-degraded.md` (Slice A authored) covers 5 paths: Personal-tier leak attempt (Sev-1), peer compromise (Sev-2), inbox flood (Sev-2), outbox delivery lag (Sev-3), planned resync (operational).
   - Dashboard `dashboards/federation-and-cross-context.json` provides real-time visibility.

## Alternatives Considered

### A. Federation ON by default (Mastodon-style global federation)

- Pros: out-of-the-box federation; lowest friction for tenant adoption.
- Cons: violates DCI-08 (Personal-tier would federate by default); violates pack residency without SCC; violates HIPAA Safe Harbor; violates KR PIPA Art. 28 cross-border consent; regulatory non-compliance.
- Rejected: incompatible with parallel ADR-0238 + regulatory posture.

### B. Federation OFF, no future opt-in capability (Twitter/X-style closed platform)

- Pros: simplest; zero federation risk.
- Cons: closes architectural option; consumer-tenant Personal-tier users + tenant admins requesting federation interop have no path; competitive disadvantage vs Mastodon / Bluesky / Threads (which is moving toward federation).
- Rejected: closes future optionality.

### C. Per-tenant opt-in for Professional-tier; Personal-tier follows tenant default (this ADR's choice except Personal-tier override)

- Pros: tenant choice; flexible.
- Cons: even with tenant opt-in, federating Personal-tier breaks the dual-context invariant — Personal-tier is by definition not tenant-scoped, so a tenant cannot opt-in their users' Personal-tier data.
- Rejected: the dual-context invariant is structural, not tenant-controllable.

### D. Per-tenant opt-in Professional-tier only; Personal-tier NEVER federates (this ADR's choice)

- Pros: tenant choice for Professional-tier (enterprise use case for cross-org federation); Personal-tier invariant preserved at compile-time + LEAN lane; aligns with messenger ADR-MSGR-0004.
- Accepted.

### E. AT Protocol primary (skip ActivityPub)

- Pros: Bluesky AT Protocol offers richer protocol semantics; potentially more aligned with content-addressed records.
- Cons: smaller ecosystem (~5M users vs ~10M+ Mastodon ecosystem); fewer existing tenants would federate; ActivityPub is W3C Recommendation status with broader adoption.
- Rejected (for P01-P02); kept open as ADR-SOC successor-IP per PRD Open Question 2.

### F. Per-user federation opt-in (user-level, not tenant-level)

- Pros: maximum user autonomy.
- Cons: regulatory complexity (per-user SCC, per-user consent management); tenant attestation can't be granular at user-level; for Personal-tier explicitly: per-user federation would still leak Personal-tier across pack borders → violates DCI-08.
- Rejected.

### G. Federation only on a separate "federated-tier" entity (not Personal/Professional)

- Pros: cleanly separates federation from dual-context tiers.
- Cons: adds a third context kind which complicates ADR-0135 invariant; tenants have to manage 3 tiers; UX confusion.
- Rejected (for P01-P02); the principle of fewer-tiers is preserved.

## Consequences

### Positive

- Personal-tier privacy preserved as structural invariant (compile-time + LEAN-lane); no possible regression.
- Tenant choice respected for Professional-tier (cross-org / cross-platform interop where wanted).
- Pack residency preserved (federation egress requires SCC + per-tenant attestation).
- HIPAA Safe Harbor preserved (pack-us-healthcare default-OFF + opt-in blocked unless BAA + per-peer attestation).
- Aligns with messenger ADR-MSGR-0004 federation pattern; cross-µservice consistency.
- AT Protocol (PRD Open Question 2) preserved as future option; this ADR does not foreclose.
- Federation operations (egress + ingress) are auditable via audit-chain Ed25519 seals.
- Operational substrate `federation-bridge-degraded.md` runbook covers all 5 paths.

### Negative

- Federation is OFF by default; tenants get no federation experience unless they actively opt in; some marketing-friction.
- Per-tenant opt-in flow requires legal + ops attestation; not a one-click toggle; SLA on opt-in = days.
- Peer-allowlist maintenance burden; ops-security must approve peer additions.
- AT Protocol scheduled-for-distinct-tracked-work (PRD Open Question 2 open); tenants requesting Bluesky interop have to wait for successor-IP ADR.

### Operational

- `oya-social-federation-gateway-*` BC scaffolded in P01 codebase (Cargo workspace), but `federationGateway.enabled: false` in default Helm values.
- P02 IP authoring: federation gateway end-to-end + peer allowlist management + opt-in workflow.
- Cedar policy `policy/public-read.cedar` PERMIT 5 (federation inbox) belt-and-suspenders forbids Personal-tier delivery.
- LEAN lane `oya-check-federation-personal-tier-refused` validates type-system constraint.
- Runbook `runbooks/federation-bridge-degraded.md` (Slice A) covers Sev-1 (Personal-tier leak) + Sev-2 + Sev-3 paths.
- Dashboard `dashboards/federation-and-cross-context.json` provides operational visibility.
- Per-pack residency: pack-kr + pack-us-healthcare lock federation default-OFF; other packs allow opt-in.

### Regulatory

- **DCI-08 + parallel ADR-0238**: Personal-tier never federates; preserved at type-system level.
- **GDPR Arts. 44-50**: cross-border transfer (federation = cross-border) requires SCC; per-tenant attestation enforced.
- **KR PIPA Art. 28**: explicit user-consent for cross-border data; tenant attestation flows through this requirement.
- **HIPAA Safe Harbor §164.514**: pack-us-healthcare federation default-OFF; activation requires BAA + per-peer attestation.
- **EU DSA Art. 14**: per-tenant ToS must disclose federation peer relationships when opted-in; tenant onboarding flow includes the disclosure.
- **eIDAS 910/2014**: Ed25519 audit-chain seals on opt-in / outbox / inbox events satisfy AdES electronic-signature requirement.
- **UK Online Safety Act 2023**: federated content from external peers subject to UK illegal-content duty if delivered to UK users; tenant-admin notification + Ofcom reporting per significance.
- **AU Online Safety Act 2021**: BOSE applies to federated content delivered to AU users; eSafety Commissioner notification per significance.

## Future Evolution

- After federation minimum-shippable-tier (P02 ActivityPub) ships:
  - ADR-SOC successor-IP for AT Protocol (PRD Open Question 2).
  - ADR-SOC successor-IP for federation peer-allowlist policy (current is operator-curated; future may be tenant-self-service or community-curated).
  - ADR-SOC successor-IP if federation default shifts (e.g., enterprise-tier defaults to federation-on).
- M04-onward: if oyatie's strategic direction shifts toward federated-by-default, file successor-IP ADR superseding this one (with the DCI-08 invariant preserved regardless).

## References

- ADR-0135 — Connect dissolution (parallel; DCI-08 source).
- ADR-0131 — Per-microservice flat layout.
- ADR-0132 — Suite-and-bundle dissolution.
- ADR-SOC-0005 — Dual-context feed isolation (paired DCI ADR; federation per-tier).
- ADR-MSGR-0004 — Messenger federation posture (sibling ADR; aligned pattern).
- ActivityPub W3C Recommendation 2018-01-23 `www.w3.org/TR/activitypub/`.
- RFC 9421 HTTP Signatures.
- AT Protocol `docs.bsky.app` (PRD Open Question 2 context).
- Mastodon federation precedent `docs.joinmastodon.org/spec/activitypub/`.
- Threads ActivityPub interop announcement (2024-present).
- GDPR Arts. 44-50.
- KR PIPA Art. 28.
- HIPAA 45 CFR §164.502, §164.514.
- EU DSA 2065/2022 Art. 14.
- eIDAS 910/2014.
- UK Online Safety Act 2023.
- AU Online Safety Act 2021 BOSE.
- `microservices/social/PRD.md` Open Questions 2 + 3.
- `microservices/social/policy/dual-context-isolation.md` DCI-08.
- `microservices/social/policy/public-read.cedar`.
- `microservices/social/policy/data-residency.md`.
- `microservices/social/runbooks/federation-bridge-degraded.md`.
- `microservices/social/threat-model.md` T-S-03, T-I-08, T-D-07.
- `microservices/social/dpia.md` R-08, R-17.

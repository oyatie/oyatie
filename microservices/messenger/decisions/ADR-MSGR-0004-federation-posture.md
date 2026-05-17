---
id: ADR-MSGR-0004
status: Accepted
date: 2026-05-17
microservice: messenger
deciders: ops-security, council-privacy, council-architecture, axis-messenger
owner: ops-security
supersedes: []
superseded_by: []
related:
  - ADR-0126
  - ADR-0131
  - ADR-0132
  - ADR-MSGR-0002
related_artifacts:
  - microservices/messenger/PRD.md (Open Question 3 — Slack/Teams federation; FR-15 deliverability dashboard analog)
  - microservices/messenger/policy/dual-context-isolation.md
  - microservices/messenger/policy/personal-dm-scope.cedar
  - microservices/messenger/multi-region.md
  - microservices/messenger/threat-model.md
purpose: Close PRD-messenger Open Question 3 — establish federation posture (default-off, opt-in mechanism, supported protocols, dual-context isolation interactions).
---

# ADR-MSGR-0004: Federation posture — OFF by default; tenant-opt-in via Cedar policy + admin gate; Matrix Client-Server r0.6+ supported, XMPP refused, Personal-DM tier never federated

## Status

Accepted — 2026-05-17.

## Context

PRD-messenger Open Question 3 asks: federation with external Slack/Teams via adapter — who owns the security review and what is the per-tenant opt-in posture? The broader question is whether oyatie messenger federates at all, with what protocol, and how the federation surface interacts with the dual-context invariant.

Three federation surfaces in the field:

- **Slack Connect / Teams External Access** — proprietary federation; closed protocols; both have their own E2E + audit-chain assumptions; integrating means accepting their tenancy model.
- **Matrix (RFC + matrix.org spec)** — open federation protocol (Matrix Client-Server r0.6+, Server-Server r0.x); used by Element, Beeper, mozilla.org, ietf.org, kde.org; supports MLS-style E2E (ratchet); federation traffic over HTTPS with Ed25519 server signing.
- **XMPP (RFC 6120 / 6121 / 6122)** — older federation; server-to-server (S2S) over TLS; ecosystem aging; protocol complexity high; modern E2E support (OMEMO) less mature than Matrix's MLS.

Federation cuts against several oyatie invariants:

- **Dual-context isolation (ADR-0126 + Bominal ADR-0208)**: federating Personal-DM tier means foreign servers see traffic patterns + metadata of personal communications; the Personal pillar's trust model assumes traffic stays local-only.
- **Per-tenant residency (ADR-0117)**: federating a Professional channel to a foreign Matrix server inherently crosses tenant residency boundaries; the federation traffic exits the pack.
- **Audit-chain integrity (Bominal ADR-0028)**: foreign-server messages don't have oyatie audit-chain seals; the receiving side cannot verify provenance with the same guarantee as local messages.
- **Cedar policy + ACL coverage**: federated principals don't have oyatie OIDC subjects; Cedar's `principal in Tenant::?t` evaluation needs a federation-shim principal.

At the same time, enterprise interop is real: a tenant whose external partner runs on Element / Matrix has a legitimate need to federate one specific channel. Refusing all federation forever cedes the enterprise-interop market.

## Decision

oyatie messenger ships **federation OFF by default**, with a strict opt-in posture:

1. **Default: federation OFF.** No federation traffic enters or leaves the µservice; foreign Matrix-server HTTPS requests are refused at the edge; Matrix federation port is not bound.

2. **Opt-in mechanism: per-tenant Cedar policy + admin gate**:
   - Tenant admin opts in via tenant configuration: `federation_enabled: true` + `federation_kind: matrix-client-server-r06`.
   - Cedar policy `microservices/messenger/policy/federation-scope.cedar` (NEW) gates which actions federation principals can perform on which resources; default policy forbids federation principals from all actions except `Action::"post_federated_message"` and `Action::"join_federated_channel"`, both scoped to channels explicitly marked `federation_eligible: true`.
   - Per-channel `federation_eligible: true` toggle by channel admin; default false.

3. **Supported protocols**:
   - **Matrix Client-Server API r0.6+** (the modern API).
   - NO Matrix Server-Server federation in the M03 launch tier — that's a follow-up ADR if needed; client-server federation alone enables 99% of "external partner on Element" interop use cases.
   - **NO XMPP** — S2S complexity, weaker E2E ecosystem (OMEMO is less mature than MLS), aging operator pool. Refused.
   - **NO Slack Connect / Teams External Access** at the protocol level (proprietary, closed) — but a per-tenant ADAPTER from oyatie messenger to Slack Connect could be authored in the future as a substrate adapter behind a port trait; that would be its own ADR, not this one. M03 launch tier does NOT include such an adapter.

4. **Personal-DM tier NEVER federated**:
   - Federation is forbidden on `DirectConversation` resources where `active_context=Personal`. Cedar `personal-dm-scope.cedar` already contains the unconditional forbid for context-drift; this ADR adds an explicit forbid for federation actions on Personal-DM resources.
   - Personal-pillar trust model preserved as a structural property.

5. **Federation principals get reduced ACL surface**:
   - No reactions, no thread-create (only thread-reply), no `Action::"upload_attachment"` over federation (attachments via federated transport are subject to a separate malware-scan posture); no mention-emit to local principals (foreign principals can only mention foreign principals).
   - All federation actions audit-chained with `federation_origin: <foreign_homeserver>` annotation.

6. **Cross-pack federation**: federation traffic NEVER crosses pack residency boundaries automatically. A tenant in pack-kr cannot federate to a Matrix homeserver in pack-eu unless the tenant signs an SCC (Standard Contractual Clauses) addendum and the tenant admin explicitly opts into cross-pack federation.

7. **MLS E2E posture under federation**:
   - Federated channels MAY use MLS (Matrix MSC4079) — both sides must support; otherwise federation falls back to TLS-in-transit (not E2E).
   - Personal-DM tier never federates → MLS-no-escrow posture in ADR-MSGR-0002 unaffected.
   - Professional-channel federation uses MLS WITH the Professional-tier escrow record (per ADR-MSGR-0002), and the federation principal's KeyPackage is recorded as a group member in the MLS group.

## Alternatives Considered

### A. Federate-all by default (any messenger-to-messenger federation)
- Pros: maximum interop; matches Matrix-native deployments' default posture.
- Cons: violates dual-context-isolation (Personal-DM federation breaches Personal pillar); breaches per-tenant residency invariants on cross-pack federation; introduces audit-chain integrity gaps (foreign messages have no oyatie seal); Cedar ACL coverage incomplete on federated principals.
- Rejected: default-on is incompatible with the dual-context posture.

### B. Federate Matrix + XMPP (both supported)
- Pros: broader interop reach (XMPP ecosystem still exists for some enterprises).
- Cons: S2S federation complexity; XMPP OMEMO E2E maturity lower than MLS; aging operator pool; doubled federation surface; ~2x the security review.
- Rejected: Matrix alone covers the modern interop need at much lower cost.

### C. Slack Connect / Teams native federation (proprietary protocols)
- Pros: covers the largest interop market (Slack + Teams dominate enterprise messaging).
- Cons: requires per-protocol adapter + ongoing compatibility maintenance as Slack / Microsoft change their protocols; ToS exposure; the adapters themselves are large engineering investments; better deferred to a follow-up substrate-adapter ADR.
- Rejected at M03: deferred to a future ADR; not in launch scope.

### D. No federation ever
- Pros: simplest invariant; tightest posture; zero federation security surface.
- Cons: cedes enterprise-interop market; partners on Element / Matrix cannot federate; a real enterprise requirement goes unserved.
- Rejected: too restrictive for the regulated-enterprise interop need.

### E. Per-channel federation enabled by channel admin without tenant-admin gate
- Pros: lower friction for end-user channel admins.
- Cons: removes tenant-admin authority over federation posture; a single channel admin going rogue can federate a sensitive channel; legal-hold + eDiscovery semantics get murky if a channel is federated without tenant-admin awareness.
- Rejected: tenant-admin gate is the right authority for federation enable; channel-admin gate is the right authority for marking a specific channel eligible *after* tenant-admin enables federation at the tenant level.

### F. Matrix Server-Server federation (full S2S) at M03 launch
- Pros: native Matrix federation; full Matrix ecosystem interop.
- Cons: S2S federation is a much larger security surface than C2S; requires homeserver-key trust establishment + Ed25519 signature verification + federation-loopback prevention; doubles the launch-tier security review burden; can be added in a follow-up ADR once the C2S federation surface is stable.
- Rejected at M03: deferred; C2S federation covers the interop need at much lower launch-tier risk.

## Consequences

### Positive

- Default-off posture preserves dual-context invariant for tenants who don't need federation (~95% of starter/pro tier; ~70% of enterprise tier).
- Tenant-admin gate + per-channel eligibility toggle provides explicit, auditable enablement path — no silent federation drift.
- Matrix C2S r0.6+ covers the modern interop need at minimum surface area.
- Personal-DM tier never-federated invariant preserves Personal pillar's trust model under federation as a structural property.
- Cross-pack federation requires SCC addendum + tenant-admin opt-in — residency-preservation defaults are correct.
- Audit-chain `federation_origin` annotation maintains provenance even for federated messages.

### Negative

- Federation principal ACL is intentionally narrow (no reactions, no thread-create, no mentions to local principals); some legitimate interop use cases will hit friction. Mitigated by clear documentation + future ADRs that may relax specific restrictions after operational experience.
- XMPP-only partners are excluded; documented as a hard limitation. Future XMPP support via a substrate adapter remains possible (separate ADR).
- Slack Connect / Teams External Access excluded at M03; users wanting that interop must use email / Workflow Studio integrations instead. Future substrate adapter possible via separate ADR.
- Matrix S2S federation deferred; tenants whose partners run their own Matrix homeservers must use C2S bridges or a Matrix bot until S2S is added.

### Operational

- New Cedar policy `microservices/messenger/policy/federation-scope.cedar` encodes federation principal ACL.
- New tenant config field `federation_enabled: bool` + `federation_kind: matrix-client-server-r06` (only valid value at M03); new channel config field `federation_eligible: bool`.
- IaC: optional Matrix Client-Server federation endpoint behind `oya-messenger-channel-store-adapter-matrix-cs` adapter (NEW); not deployed unless any tenant enables federation.
- Federation principal bridge: oyatie issues federation-scoped OIDC subjects for foreign Matrix-server users; subjects carry `federation_origin: <foreign_homeserver>` audit-tag.
- Dashboards: per-tenant federation panel (federation principal count, federation message rate, federation message bounce rate, audit-chain federation-event rate); federation-active tenants get an SLO panel.
- New CI lane `messenger-federation-scope-conformance` validates: (a) federation principal can ONLY perform federation-scoped actions, (b) Personal-DM resources never reachable by federation principal, (c) cross-pack federation gated by SCC evidence.
- Runbook `microservices/messenger/runbooks/federation-enable-disable.md` (NEW) documents the tenant-admin opt-in ceremony + disable ceremony.

### Regulatory

- **Matrix specification (r0.6+ Client-Server API)** — `https://spec.matrix.org/`; oyatie's adapter conforms.
- **RFC 9420 (MLS)** — preserved under federation per Matrix MSC4079 when both sides support it; falls back to TLS-in-transit otherwise.
- **GDPR Art. 44-49** — cross-pack federation requires SCC addendum; documented in tenant config + DPIA per pack.
- **KR PIPA Art. 17** — cross-border transfer requires explicit consent; cross-pack federation enable triggers consent collection.
- **HIPAA 45 CFR §164.314** — Business Associate Agreement required for any federated entity receiving PHI; tenant DPIA enforces.
- **ePrivacy Directive 2002/58/EC Art. 5** — communications confidentiality preserved under MLS-over-federation; TLS-in-transit-only federation labelled as such in audit-chain.
- **EU AI Act**: out of scope (federation is not automated decision-making per se).

## References

- Matrix specification (Client-Server r0.6+) — `https://spec.matrix.org/v1.10/client-server-api/`
- Matrix MSC4079 (MLS over Matrix) — `https://github.com/matrix-org/matrix-spec-proposals`
- RFC 9420 — Messaging Layer Security (MLS)
- RFC 6120 / 6121 / 6122 — XMPP (referenced for rejected alternative)
- Element / matrix.org operational deployments — public docs
- Slack Connect documentation — `https://api.slack.com/connect` (referenced for deferred alternative)
- Microsoft Teams External Access — `https://learn.microsoft.com/en-us/microsoftteams/manage-external-access`
- GDPR Arts. 44-49 — cross-border transfer
- KR PIPA Art. 17 — cross-border transfer
- HIPAA 45 CFR §164.314 — Business Associate Agreements
- ePrivacy Directive 2002/58/EC Art. 5 — confidentiality of communications
- ADR-0126 — Connect full social network super-app (dual-context source)
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-suite-and-bundle dissolution
- ADR-MSGR-0002 — E2E personal-DM key escrow tier-split (paired posture)
- `microservices/messenger/PRD.md` Open Question 3
- `microservices/messenger/policy/dual-context-isolation.md`
- `microservices/messenger/policy/personal-dm-scope.cedar`
- `microservices/messenger/multi-region.md`
- `microservices/messenger/threat-model.md`

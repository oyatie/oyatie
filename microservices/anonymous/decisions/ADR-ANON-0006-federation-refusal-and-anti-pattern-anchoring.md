---
id: ADR-ANON-0006
status: Accepted
date: 2026-05-17
microservice: anonymous
deciders: axis-anonymous, council-architecture, council-privacy, ops-security, general-counsel
owner: axis-anonymous + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-ANON-0001
  - ADR-ANON-0003
related_artifacts:
  - microservices/anonymous/PRD.md (I5, FR-26)
  - microservices/anonymous/competitor-parity-matrix.md
purpose: |
  Refuse, permanently, all federation protocols for the anonymous µservice;
  anchor the refusal against documented anti-patterns from Secret (defunct),
  4chan, Whisper, and decentralised-anonymous-network research.
---

# ADR-ANON-0006: Federation refusal — ActivityPub / AT Proto / Matrix / XMPP REFUSED forever, anchored against Secret / 4chan / Whisper anti-patterns + Tor lessons

## Status

Accepted — 2026-05-17. Permanent.

## Context

PRD I5 commits the µservice to **refuse federation forever**. The decision is whether this refusal should be: (a) the default with toggle, (b) the default with no toggle, or (c) structurally impossible (no BC; no chart toggle; refused at build).

Federation in the social-platform space typically means one of:

- **ActivityPub** (Mastodon, Pixelfed, Lemmy, PeerTube)
- **AT Protocol** (Bluesky)
- **Matrix federation** (Element)
- **XMPP federation** (legacy XMPP networks)

Each enables cross-server post / message exchange. None can preserve I1 (no user_id↔post_id correlation outside legal-process) because:

1. Peer servers cannot be bound to the platform's Cedar policy.
2. Peer servers cannot be bound to the platform's legal-process workflow (ADR-ANON-0003).
3. Peer servers' identity-correlation posture is opaque to the platform's auditors.

Anti-pattern anchors:

- **Whisper (2014 breach)**: server-side correlation was discoverable; even without federation, a non-cryptographic platform can leak. We solve this via ADR-ANON-0001 cryptographic blinding. But federation would re-introduce the failure mode AT EVERY PEER.
- **Secret (defunct, 2015)**: anonymous app with no moderation → harassment cascade → class-action shutdown. Federation would expand the harassment surface across peers we cannot moderate.
- **4chan**: cross-board no-account posting has historically enabled CSAM hosting + harassment; federation-light failure mode. Federation would expand this surface.
- **Burnbook (defunct, 2017)**: school-bound app with no minor protection → bullying lawsuits. Federation would expand to peers we cannot vet for minor-protection posture.
- **Tor onion-routed forums**: network-anonymity is good for transport; does NOT solve server-side identity correlation. We build on top.

## Decision

Federation is **structurally refused**:

1. **No BC for federation.** The µservice does not have a `federation-gateway` BC. Compare with `social` µservice which has one (OFF by default; a deliberate design choice for social's product scope).
2. **No chart toggle.** Helm values.yaml has no `federation.enabled` field. There is no way to "turn on" federation at deployment time.
3. **No code path.** No Rust crate in the workspace handles federation traffic. CI lint refuses if any crate is named `*-federation-*` or imports an ActivityPub / AT Proto / Matrix / XMPP library.
4. **Permanent**. Reversing this decision requires a superseding ADR + Council Privacy chair approval + 12-month migration plan + tenant-side opt-in disclosure.

## Alternatives Considered

### A. Federation OFF by default with toggle

- **Pros**: Optionality; tenants who want federation can opt in.
- **Cons**: Defeats I5 structurally; tenants might toggle without understanding the implications; the toggle's existence is a permanent attack-surface for misconfiguration.
- **Rejected because**: I5 is the structural promise; "OFF by default with toggle" is a marketing claim with no structural guarantee.

### B. Federation on a separate µservice tier (anonymous-federated)

- **Pros**: Could serve the federation-curious tenant via a separate product without compromising this µservice.
- **Cons**: Creates a sister µservice that competitor researchers would inevitably point to as "the platform federates"; tenant confusion; product strategy regression.
- **Rejected because**: Product strategy commitment is to NO federation; we don't half-commit.

### C. Federation refused but cross-pack data flow allowed (e.g., pack-eu to pack-us posts)

- **Pros**: Some cross-pack discovery.
- **Cons**: Same legal-process / Cedar-policy / pack-jurisdiction problems as federation; would weaken pack residency posture.
- **Rejected because**: Pack residency is independently a structural promise.

### D. Federation refused for posts but allowed for hashtag-corpus

- **Pros**: Cross-platform discovery via hashtag aggregation.
- **Cons**: Hashtag corpus is a fingerprint; federated hashtag-corpus federation enables re-identification (small affinity clusters' hashtag distributions are unique); contradicts ADR-ANON-0007 k-anonymity.
- **Rejected because**: Even hashtag-corpus federation is structurally re-identifying.

## Consequences

### Positive

- **I5 invariant structurally enforced.** Cannot be misconfigured; cannot be toggled; cannot be code-changed without superseding ADR.
- **Anti-pattern anchors documented.** Secret + 4chan + Whisper + Burnbook examples make the refusal defensible.
- **Audit-grade clarity.** Auditors see federation refused at build time, not at deployment time.
- **Tenant-trust differentiator.** "The platform structurally cannot federate" is a stronger claim than "the platform doesn't federate by default."

### Negative

- **Lost market opportunity.** Federation-curious tenants must use a different product. Mitigated: oyatie's `social` µservice has a federation-gateway BC (OFF by default; opt-in) for tenants who want federation in the non-anonymity tier.
- **No cross-platform discovery.** Mitigated: in-tenant affinity feed is what the product promises; cross-platform discovery is explicitly out-of-scope.

### Operational

- LEAN lane `oya-check-federation-refused` refuses any crate matching `*-federation-*` or importing federation libraries.
- Helm chart lint refuses any `federation.*` value.
- Per-pack overlay lint refuses federation-related configuration.

### Regulatory

- **EU DSA Art. 28**: cross-platform interoperability of "very large online platforms" — anonymous is not a VLOP (under 45M monthly active users threshold per current scope); the interoperability obligation does not apply to us.
- **KR 통신비밀보호법**: federation peers cannot be bound to legal-process workflow per Art. 9; refusal is regulatory-aligned.
- **EU MLAT**: cross-pack federation would create MLAT-bypass concerns; refusal is regulatory-aligned.

### Invariant Preservation

I5 structurally satisfied; I1 protected against the federation re-introduction failure mode.

## References

- ActivityPub W3C Recommendation — `https://www.w3.org/TR/activitypub/`
- AT Protocol (Bluesky) — `https://atproto.com/`
- Matrix Specification — `https://spec.matrix.org/`
- XMPP RFC 6120
- Whisper / Washington Post 2014 — server-side identity correlation breach
- Secret shutdown analysis — TechCrunch / Recode 2015
- Burnbook shutdown analysis — TechCrunch 2017
- 4chan moderation analysis — various academic + journalistic sources
- Tor Project — `https://www.torproject.org/` (transport-anonymity reference)
- ADR-ANON-0001 (cryptographic blinding — the protection federation would defeat)
- ADR-ANON-0003 (legal-process workflow — the workflow federation would bypass)
- EU DSA Art. 28 (VLOP interoperability — not applicable)

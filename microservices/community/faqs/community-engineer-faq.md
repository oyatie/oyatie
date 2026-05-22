---
doc_class: FAQ
microservice: community
persona: community-engineer + trust-and-safety-engineer
date: 2026-05-20
doc_status: published
---

# Community Engineer FAQ — community

## Why is `community` the canonical name + the `anonymous` µservice folded in?

Per the 2026-05 wave-3-G doctrine cluster + `project_wave_3_g_state_2026_05_21`. We renamed `anonymous` → `community` and folded the anonymous-mode primitives into the community µservice because:

1. Anonymous posting is a SUBSET of community posting; modeling them as separate µservices doubled the surface for the same underlying primitives.
2. The verified-anonymous (TeamBlind-class) model requires identity-verification to fully work; that's a community-µservice concern.
3. Cross-tenant federation crosses both anonymous + non-anonymous; modeling boundaries are easier in one µservice.

The merge is named `community` because the canonical product surface is community-shaped (forums, threads, voting); anonymity is one mode.

## What's the TeamBlind-class verification model? How is it different from "anonymous"?

Per IP-002 + IP-journey-j32. Plain anonymous = no identity verified. TeamBlind-class anonymous = identity verified BUT shown publicly as a pseudonym + verified claims badge.

The flow:

1. User claims `corporate-email: alice@acme.com`.
2. They redeem an email magic-link sent to `alice@acme.com`. Proves they own the inbox.
3. The verified claim is stored in an HSM-encrypted vault keyed by the user's principal ID.
4. The user's posts can carry the "✓ verified Acme employee" badge while showing only a pseudonym (e.g., "anon_engineer_4218").
5. The pseudonym → real identity mapping is unsealable only with Cedar permission (typically requires a court order or trust-and-safety incident).

This is what TeamBlind does + LinkedIn-mode posts can do; we make it a first-class primitive.

## How is identity stored to prevent leak?

Per IP-002 + IP-011 (Cedar policy fragments):

- The corporate-email claim is hashed (BLAKE3-256) + the hash stored in the public identity table.
- The plaintext is encrypted with the tenant's KMS-resident key (paid: FIPS 140-3 L2; paid compliance-pack: FIPS 140-3 L3).
- De-anonymization requires `community::identity::deanonymize` Cedar permission — typically held only by trust-and-safety personnel + the tenant's legal counsel.
- The de-anonymization itself emits a `community.identity.deanonymized` audit event; the de-anonymization request becomes part of the audit trail.

For SecureDrop + whistleblower intake (paid compliance-pack), the identity is sealed with Shamir's Secret Sharing across multiple geographic facilities — unsealable only with court order + 3-of-5 quorum.

## How does cross-tenant federation work without leaking?

Per IP-journey-j134 (cross-tenant staffing engagement). Three federation modes:

1. **Federated** (cross-tenant board): verified-corporate-email users from allowlisted tenants can post + read. Each tenant maintains its own copy of moderation rules; the most-restrictive applies.
2. **Cross-posting** (user-driven): a user explicitly cross-posts content from tenant-A to tenant-B (e.g., a recruiting post that the user wants visible in multiple talent communities).
3. **Federated read-only** (subscription): tenant-A subscribes to tenant-B's public boards for read-only consumption.

Privacy guarantees:

- Cross-tenant federation respects the source tenant's anonymous-mode settings.
- A user's reputation is portable across federated boards but their identity vault is per-tenant.
- Cedar enforces per-tenant boundaries on every cross-tenant action.

## What's the LinkedIn-class reputation model at paid advanced?

Per IP-journey-j108-j113 + j149. Each user has:

- **Skills**: self-declared + endorseable by peers.
- **Career history**: claim (with date range + employer-tenant ID); verified by employer-tenant (via the employer's HR system).
- **Public profile**: optional, includes portfolio, links, reputation.
- **Reputation score**: weighted aggregate (upvotes × tier-weight + skill endorsements + cross-board reputation + verified-purchase signals + manager-given signals).

The verification path is bidirectional: the employee claims employment; the employer-tenant (via authorized HR principal) verifies. Cross-tenant: a user who claims employment at tenant-X can have that claim verified by tenant-X regardless of which tenant they primarily post in.

## What's Handshake-class jobs at paid advanced?

Per IP-journey-j147 + j56. Tenants can post jobs to their community + accept applications. Cross-tenant: a user at tenant-A can apply to a job at tenant-B (the application carries the user's verified reputation + skills + career history per the LinkedIn-class model).

The hiring funnel:

1. Tenant-B posts a job (Cedar `community::job::post`).
2. Job appears on tenant-B's job board + (optionally) on cross-tenant boards.
3. User at tenant-A finds the job + clicks Apply.
4. Their profile (with verified claims) is sent to tenant-B.
5. Tenant-B reviews + advances through a workflow (typically `workflow-engine` orchestrated).

Reputation portability matters here: tenant-B sees the candidate's full cross-tenant reputation, not just their tenant-A activity.

## How does LLM-assisted moderation work?

Per IP-010 (foundry-guardrails-moderation-bridge). Every post + comment is forwarded to the `intelligence` µservice's moderation classifier (a fine-tuned Llama 3.3 70B with a moderation LoRA trained on labelled examples of spam, harassment, hate-speech, CSAM, doxxing, self-harm).

The classifier returns:

- Class predictions with confidence (e.g., `spam: 0.92, harassment: 0.04, ...`).
- Per-class action recommendation (`auto-remove`, `queue-for-human`, `allow`).

Per-tenant moderation policy maps class → action:

```yaml
auto_remove_class: [csam]  # mandatory; cannot be disabled
auto_flag_class: [spam, harassment, hate-speech, doxxing, self-harm]
queue_for_human_class: [borderline]
allow_with_warning_class: [strong-language]
```

CSAM auto-remove is mandatory per US 18 USC § 2258A reporting obligations + EU DSA Article 16. Other classes are tenant-configurable.

## What's SecureDrop integration at paid compliance-pack?

Per IP-journey-j06 (securedrop-intake). SecureDrop is the Tor-anonymous-source-protection tool used by news organizations. We integrate at paid compliance-pack for tenants offering whistleblower channels:

- Sources connect via Tor hidden service (`*.onion`).
- Submissions go into a journalist-only board with chain-of-custody seal in audit-chain.
- No metadata leaks: TOR + ephemeral PostgreSQL row + HSM-encrypted at-rest.

The intake is mandatory at paid compliance-pack for media tenants + opt-in for other regulated tenants.

## How are spam waves handled?

Per `runbooks/community-spam-flood.md`. Layers of defense:

1. **Rate-limit**: per-user, per-IP, per-tenant. Default: 10 posts/hour/user, 100/hour/IP, 10k/hour/tenant.
2. **Captcha**: triggered after rate-limit threshold OR LLM-classified spam confidence > 0.4.
3. **LLM moderation**: auto-remove > 0.95 confidence; queue 0.4-0.95.
4. **Shadow-ban**: user's posts are visible only to themselves (they don't realize they're banned, so they don't sign up with a new account).
5. **IP / range blocks**: at the load-balancer (Cilium).
6. **Tenant-level emergency**: temporarily disable posting on the tenant.

Most spam waves resolve at layer 2-3. Severe waves (organized attack) need layers 4-6.

## What's the reputation recompute model?

Per IP-006 (voting-engine). Reputation = sum-of-weighted-upvotes - sum-of-weighted-downvotes - 10 × moderator-removal-count + 5 × verified-purchase-signal + 3 × manager-endorsement-signal.

Recompute is incremental on every vote; nightly batch reconciles. If a user's upvote pattern is detected as manipulation (e.g., reciprocal upvoting), the votes are devalued + their reputation is recomputed.

## How does this differ from `messenger`?

- `community`: many-to-many publication (forum posts, comments). Public or board-scoped.
- `messenger`: one-to-one or small-group direct messages. MLS-encrypted (RFC 9420 per `feedback_mls_rfc_9420_e2ee_personal_messenger`).

Some boundary cases: a "direct message" to a community member is messenger; a "reply to a public post" is community.

## What about CSAM + child-safety reporting at paid compliance-pack?

Per IP-journey-j18 (child-safety-report-intake). At paid compliance-pack:

- Auto-flag for any image / text matching CSAM detection (via `intelligence` + Microsoft PhotoDNA where licensed).
- Mandatory report to NCMEC (US) within 24 h per 18 USC § 2258A.
- Per-jurisdiction report to: UK IWF, AU eSafety Commissioner, IN MeitY, CN reporting agencies.
- Audit-chain emits the report with chain-of-custody seal.
- Tenant cannot opt out of CSAM reporting at paid compliance-pack.

## Why fold `anonymous` instead of keeping it as a separate µservice?

The original design had `anonymous` as a separate µservice intended to handle TeamBlind-class anonymous identity. The folding rationale:

1. The primitives (identity vault, pseudonym registration, claim verification) are shared with community moderation + identity-management + reputation. Splitting them required duplicated infrastructure.
2. Cedar policy authoring was awkward at the boundary (some `community::*` permissions implied `anonymous::*` and vice-versa; cross-µservice permission inheritance is brittle).
3. The cross-tenant federation model needs to span anonymous + non-anonymous flows in one transaction.

Folding gives us one µservice + one Cedar namespace + one audit-chain emitter family for community surfaces.

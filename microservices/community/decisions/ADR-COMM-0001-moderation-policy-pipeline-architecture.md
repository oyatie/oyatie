---
id: ADR-COMM-0001
status: Accepted
date: 2026-05-17
microservice: community
deciders: axis-community, council-architecture, council-privacy, ops-security
owner: axis-community
supersedes: []
superseded_by: []
related:
  - ADR-0028
  - ADR-0105
  - ADR-0135
  - ADR-0131
  - ADR-0132
related_artifacts:
  - microservices/community/PRD.md (§"Security", §"Audit + Compliance", FR-06, FR-10)
  - microservices/community/PHASE-01-COMMUNITY-SUBSTRATE.md (IP-007, IP-010, IP-011)
  - microservices/community/IP-007-moderation-queue.md
  - microservices/community/IP-010-foundry-guardrails-moderation-bridge.md
  - microservices/community/policy/tenant-scope.cedar
  - microservices/community/runbooks/moderation-queue-clear.md
purpose: Close the moderation-pipeline composition question — fix the canonical pattern by which `auto-classifier → human moderation queue → appeal → audit-chain seal` compose, including the Cedar evaluation point at every hop.
---

# ADR-COMM-0001: Moderation policy pipeline — chain-of-responsibility composition with Cedar policy evaluation + audit-chain seal at every hop

## Status

Accepted — 2026-05-17.

## Context

PRD-community FR-06 commits the µservice to moderator actions (hide / lock / pin / move / merge / delete) and FR-10 commits to an event-driven integration with `foundry-guardrails` for spam / abuse classification. The PRD §"Security" further commits every moderation action to an audit-chain Merkle / Ed25519 seal per ADR-0028. The PRD does not, however, fix the *composition pattern* for the moderation pipeline: how the auto-classifier verdict (`PostShouldHide`) flows into the human moderation queue, how the human moderator's verdict supersedes the classifier, how appeals re-enter the pipeline, and where the Cedar policy check fires at each hop.

The composition pattern is a load-bearing decision because:

- A wrong pattern can let the auto-classifier hide a post *without an auditable moderator override path* (UX + regulatory failure under EU Digital Services Act Art. 14 right-of-appeal).
- A wrong pattern can let a moderator action skip the audit-chain seal (regulatory failure under SOX-equivalent recordkeeping + tenant-contract auditability commitments).
- A wrong pattern can let a Cedar policy gap silently permit cross-tenant moderation (catastrophic tenant-isolation failure).

Three architectural patterns compete:

- **Chain-of-Responsibility (CoR)**: classifier → queue → moderator → appeal as discrete handlers; each handler is independent and sealed; passes a typed envelope. Industry precedent: Reddit's Automoderator + ModSupport, Discord's AutoMod + Trust & Safety, Stack Overflow's flag queue + diamond-mod review.
- **Event-Sourced State Machine**: every action is an event appended to an immutable log; current state is computed by folding the log. Industry precedent: Twitter (X) Birdwatch / Community Notes, Wikipedia revision-history-as-mod-trail, Mastodon report-status state machine.
- **Hub-and-Spoke Broker**: a central moderation broker receives all signals and dispatches to handlers; handlers do not call each other. Industry precedent: Discourse's flag aggregator, Lemmy's report dispatcher, Notion's report router.

The competing forces:
- Auditability: event-sourced wins (full history is the source-of-truth), but CoR is fine *if* each hop emits its own audit-chain seal.
- Latency: CoR + hub-and-spoke are comparable; event-sourced has slightly higher write cost (every state change is an event append).
- Reviewability + Cedar fit: CoR's per-hop handler boundary is the natural place to evaluate Cedar policy (one fragment per handler); event-sourced requires policy evaluation on event *projection* which is harder to reason about; hub-and-spoke centralises policy at the broker which creates a single-point-of-failure for tenant isolation.
- Appeal flow: CoR's natural pattern is "re-enter at a higher hop"; event-sourced's pattern is "append an appeal-event"; hub-and-spoke's pattern is "broker re-dispatches" which is implicit and easy to break.

## Decision

The community µservice adopts a **Chain-of-Responsibility composition** with the following fixed hops:

```
PostCreated ──┐
              │
              ▼
        ┌───────────┐    PostShouldHide       ┌──────────────────┐
        │ classifier│ ──────────────────────▶│ moderation-queue │
        │ (foundry- │   (or PostSpamScore)   │  (human review)  │
        │guardrails)│                         └────────┬─────────┘
        └───────────┘                                  │
                                                       ▼
                                              ┌──────────────────┐
                                              │ moderator verdict│
                                              │  (apply / deny)  │
                                              └────────┬─────────┘
                                                       │
                                                       ▼ (optional)
                                              ┌──────────────────┐
                                              │     appeal       │
                                              │ (re-enters queue │
                                              │  at higher tier) │
                                              └──────────────────┘
```

The pattern fixes the following five non-negotiable invariants:

1. **Each hop is its own typed handler** behind a `ModerationHop` trait in `oya-community-moderation-queue-kernel`. Implementations: `ClassifierHop`, `QueueAdmitHop`, `ModeratorVerdictHop`, `AppealHop`, `AuditSealHop`. The handler signature is `Result<ModerationEnvelope, ModerationDeny>`; only `AuditSealHop` is mandatory in the chain (you cannot omit it).

2. **Cedar policy evaluation fires at every hop** via `policy/tenant-scope.cedar` + `policy/auditor-scope.cedar`. The hop reads (subject, action, resource, context) and asks Cedar `is_authorized`. If `Deny`, the envelope is dropped with an audit-chain seal recording the denial reason. There is no "Cedar runs at the entry point and not later" optimisation — every hop independently re-evaluates because each hop has a different subject (classifier vs. moderator vs. appellant).

3. **Every hop emits exactly one audit-chain seal record** (`ModerationActioned`, `ClassifierVerdict`, `QueueAdmitted`, `AppealLodged`) per ADR-0028 Merkle / Ed25519. Seal latency p99 ≤ 1 s. The audit-chain root advances per hop; missing a seal is a P0 incident.

4. **The moderator verdict is the supreme override**, including over the classifier verdict — but only with two-eyes for destructive verbs (delete / ban) over a per-tenant threshold (default: ban affecting > 100 posts requires two-eyes per `policy/two-eyes.cedar` fragment authored in this ADR).

5. **The appeal hop is a real hop, not a state transition**. An appeal generates a new `ModerationEnvelope` that re-enters the queue at a *higher* tier (a different moderator group than the original verdict-issuer), so the original verdict-issuer cannot review their own decision. This is the EU DSA Art. 14 "internal complaint mechanism" requirement and the Stack Overflow / Discourse industry precedent.

The chain is composed at the `oya-community-moderation-queue-usecase` layer; the kernel only defines `ModerationHop` and `ModerationEnvelope`. The chain is *not* configurable at runtime — the hop set is fixed at compile time. Per-tenant variation (e.g., disabling classifier hop for a tenant that wants pure-human review) is a *Cedar policy decision* that short-circuits the classifier hop, not a chain-rewiring decision. This keeps the topology auditable.

## Alternatives Considered

### A. Event-Sourced State Machine (Reddit Birdwatch / Wikipedia revision-history pattern)
- Pros: full immutable history by construction; trivial to reconstruct moderator decisions for audit; well-understood industry pattern; reduces "did we seal that?" failures because the event log *is* the audit-chain projection.
- Cons: Cedar policy evaluation on event projection is hard to reason about (the policy may see different state at different projection points); appeal flow becomes "append an event" which is implicit and easy to miss in code review; latency is higher because every state change is an event append + projection re-compute; harder to write per-hop tests because the hop boundary is implicit in the projection logic.
- Rejected: Cedar-evaluation-on-projection is too subtle for a regulatory-critical surface; per-hop CoR boundaries are the natural Cedar attachment points.

### B. Hub-and-Spoke Broker (Discourse flag aggregator / Lemmy dispatcher pattern)
- Pros: central place to enforce invariants (e.g., "every signal lands in the queue first"); easy to add new signal sources; broker is a single point of metrics + tracing.
- Cons: single point of failure for tenant isolation (a bug in the broker bypasses every tenant's policy at once); Cedar policy is evaluated centrally which creates a god-handler that is hard to review; appeal flow is implicit ("broker re-dispatches") which is documented in code comments not in a typed signature; over time the broker accretes business logic and stops being a broker.
- Rejected: central-broker pattern is an attractor for unrelated logic; tenant-isolation blast radius too large; the auditability benefit is illusory because the broker is just one layer of indirection over what is effectively the same chain.

### C. CoR with policy evaluation at the entry hop only (latency-optimised)
- Pros: lower latency; fewer Cedar evaluations per envelope.
- Cons: subject identity changes between hops (classifier is a service principal; moderator is a human; appellant is a different human); policy must re-evaluate at the new subject; latency saving is ~100µs which is not load-bearing; auditors expect per-hop policy enforcement because that's the industry pattern.
- Rejected: a latency optimisation that loses regulatory defensibility is not an optimisation.

### D. No chain — direct API surface ("moderator action goes straight into Postgres")
- Pros: simplest possible implementation.
- Cons: skips the classifier hop entirely (no spam coverage); skips the appeal hop (DSA non-compliant); skips the queue admission hop (no batching, no rate-limit, no overflow); skips the audit-chain seal at the wrong layer.
- Rejected: doesn't satisfy FR-10 or §"Audit + Compliance".

## Consequences

### Positive

- Every moderation envelope has an end-to-end audit trail with per-hop seals; an auditor can reconstruct any moderator action from first principles by replaying the audit-chain.
- Cedar policy fragments are per-hop and reviewable in isolation; policy regressions are localised.
- Appeal flow is a typed signature, not a comment in code; reviewers cannot accidentally remove the appeal hop without breaking the type system.
- Industry-aligned pattern (Reddit Automoderator + Stack Overflow flag queue + Discourse + Discord AutoMod all converge on chain-of-handlers); migration of moderator playbooks from those platforms onto oyatie is a documentation exercise rather than a re-training one.
- Per-tenant variation lives in Cedar policy (data) not in chain topology (code); tenants cannot accidentally rewire the chain by mis-configuration.

### Negative

- Five hops × Cedar evaluation per envelope = ~5× the Cedar QPS of a single-evaluation design. Mitigated by Cedar's `is_authorized` cost being sub-100µs and by caching policy ASTs per fragment.
- New hops require both a new handler implementation *and* a new Cedar fragment + audit-chain event type; this is more work per change than a hub-and-spoke pattern. Accepted because the cost is paid once per hop, not per change-to-existing-hop.
- Two-eyes flow for destructive verbs adds a synchronous step that can degrade latency to several seconds when the second moderator is offline. Mitigated by a 30-min two-eyes window + fallback to "queued pending second eye" UX.

### Operational

- New runbook `runbooks/moderation-queue-clear.md` (already exists) updated to document per-hop failure isolation (e.g., classifier-down does not block manual moderator action; appeal hop short-circuits cleanly).
- Cargo workspace adds `ModerationHop` trait + per-hop concrete crates: `oya-community-moderation-queue-kernel`, `oya-community-moderation-queue-domain`, `oya-community-moderation-queue-usecase`, `oya-community-moderation-queue-adapter-moderation-bridge` (already present in catalog).
- Cedar policy fragments: `policy/tenant-scope.cedar` + new `policy/moderation-hop-classifier.cedar` + `policy/moderation-hop-moderator.cedar` + `policy/moderation-hop-appeal.cedar` (to be authored in IP-011 successor-IP).
- Dashboards: `dashboards/moderation-queue-depth.json` extended with per-hop latency + seal-latency panels.
- CI lane `community-moderation-pipeline-integrity` BLOCKS PRs that add a hop without a Cedar fragment + an audit-chain event type.

### Regulatory

- **EU Digital Services Act (DSA) Art. 14** — internal complaint mechanism: the appeal hop is the canonical implementation; the higher-tier moderator routing satisfies the "independent review" requirement.
- **Section 230 + similar safe-harbor** (US): the chain-of-responsibility pattern is consistent with "good-faith moderation" — every moderator decision is logged with the moderator's identity + reason + Cedar policy ID, supporting good-faith defence.
- **GDPR Art. 17 right to erasure**: moderator-initiated deletion is a documented chain entry; the audit-chain seal includes a structured `reason` field so post-erasure the log itself records why erasure happened (not what was erased).
- **KR PIPA Art. 28**: per-pack overlay enforces KR-resident audit-chain storage for KR-tenant moderation events.
- **HIPAA 45 CFR §164.312** (when pack-us-healthcare is active): moderator actions on posts containing PHI are sealed with the same envelope; PHI redaction at the queue admission hop is a policy decision, not a chain rewiring.

## References

- ADR-0028 — audit-chain Merkle / Ed25519 sealing
- ADR-0135 — Connect-unbundle (parent ADR establishing the community µservice)
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-platform-and-bundle dissolution
- Reddit Automoderator + ModSupport documentation — `https://www.reddit.com/wiki/automoderator`
- Stack Overflow flag queue + diamond-mod escalation — `https://meta.stackexchange.com/q/161541`
- Discord AutoMod + Trust & Safety architecture — `https://discord.com/safety`
- Discourse flag aggregator + reviewable queue — `https://github.com/discourse/discourse/blob/main/app/models/reviewable.rb`
- Lemmy report dispatcher — `https://github.com/LemmyNet/lemmy`
- Mastodon report state machine — `https://docs.joinmastodon.org/admin/moderation/`
- Wikipedia revision-history-as-mod-trail — `https://en.wikipedia.org/wiki/Wikipedia:Revision_deletion`
- EU Digital Services Act Art. 14 — internal complaint mechanism — `https://eur-lex.europa.eu/eli/reg/2022/2065`
- Cedar policy language reference — `https://docs.cedarpolicy.com/`
- Gang of Four — Chain of Responsibility pattern
- `microservices/community/PRD.md` FR-06, FR-10, §"Security"
- `microservices/community/IP-007-moderation-queue.md`
- `microservices/community/IP-010-foundry-guardrails-moderation-bridge.md`
- `microservices/community/policy/tenant-scope.cedar`

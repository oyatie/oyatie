---
doc_class: Onboarding
microservice: community
persona: community-engineer + trust-and-safety-engineer + community-platform-engineer
related_adrs: [ADR-0263, ADR-0316, ADR-0131]
date: 2026-05-20
doc_status: published
---

# Community Engineer onboarding — first 5 working days on `community`

Audience: a new community engineer, trust-and-safety engineer, or community-platform engineer joining the `community` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, created a tenant forum, exercised the corporate-email verification + anonymous-post flow (paid shadow), run LLM-assisted moderation, configured cross-tenant federation, and walked the moderation-storm runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 45 min). Note the TeamBlind + Reddit + LinkedIn + Handshake hybrid model + the anonymous-fold doctrine.
2. Read `ARCHITECTURE.md` § post-store + § thread-tree + § anonymous-mode-isolation + § moderation-bridge (∼ 60 min).
3. Read the IP series for this µservice (IP-001 through IP-015) to understand the substrate composition.
4. Open the Grafana folder `community`. Primary boards: `community-post-rate`, `community-moderation-action-rate`, `community-anonymous-post-fraction`, `community-cross-tenant-federation-rate`, `community-identity-verification-success-rate`, `community-reputation-score-distribution`.
5. Walk `runbooks/README.md`. The on-call runbooks: `moderation-storm.md`, `anonymous-identity-leak.md`, `cross-tenant-federation-deny.md`, `reputation-recompute-stuck.md`, `securedrop-bridge-down.md`, `corporate-email-verification-failed.md`, `comment-tree-corruption.md`, `kb-search-stale.md`, `community-spam-flood.md`.
6. Sit in on the Wednesday community-substrate handoff. Watch the outgoing rotation review the past-week moderation-action rate + anonymous-post fraction.

Acceptance: you can sketch the post path: tenant API → Cedar gate → identity-verification (if anonymous-mode) → moderation bridge (auto-flag) → post-store insert → audit-chain emit → notification fanout. Plus the verify path: corporate-email claim → SES/Mailgun verification email → token redemption → HSM-encrypt identity → pseudonym registration.

## Day 2 — demo_trial cell bootstrap + first forum

```text
Native operation: community bootstrap
Route: cloud control-plane operation ledger (not local retired CLI/raw Cargo)
Required evidence:
- Buck2 target(s) for the changed contract/runtime
- Prow/Kubernetes-native `oya-ci-required` job URL
- operation ledger id and emitted audit-chain event ids
```

Expected runtime: ≤ 12 min. Verify:

```sh
oya community health --cell drill-syd-1
# Expected:
#   postgres.posts: up (lag_ms=12)
#   elasticsearch.search-index: up
#   seaweedfs-s3.kb-articles: up
#   pulsar.community-events: connected
#   audit-chain.emit: up
#   intelligence.moderation-bridge: connected
```

Create a tenant + a forum board:

```sh
oya community tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --display-name "ACME Community" \
    --anonymous-mode-policy disabled \
    --cross-tenant-federation false

oya community board create \
    --tenant drill-acme \
    --board-id general \
    --display-name "General Discussion" \
    --description "Anything goes in here." \
    --moderation-policy default
```

Post the first message:

```sh
oya community post create \
    --tenant drill-acme \
    --board general \
    --author u-alice@drill.test \
    --title "Welcome to ACME Community" \
    --body "First post on the new forum. Please introduce yourselves."
# Output: post_id=p_drill_001
```

Comment on it:

```sh
oya community comment create \
    --tenant drill-acme \
    --post p_drill_001 \
    --author u-bob@drill.test \
    --body "Hi Alice! Excited to be here."
```

Verify audit emissions:

```sh
oya audit query --tenant drill-acme --event-class "community.*" --since 5m
# Expected: post.created + comment.created events
```

Acceptance: cell bootstrap; tenant + board + post + comment round-trip.

## Day 3 — Identity verification + anonymous mode (paid shadow)

paid tier enables identity verification + anonymous mode. Shadow at demo_trial:

```sh
oya community tenant update \
    --cell drill-syd-1 \
    --tenant drill-acme \
    --anonymous-mode-policy verified-corporate-employees-only \
    --identity-verification-providers persona,corporate-email
```

Verify a corporate-email claim:

```sh
oya community identity verify-init \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --claim 'corporate-email:alice@acme-real.example' \
    --proof-method email-magic-link
# Expected: 200 OK; verification email sent to alice@acme-real.example
```

Simulate the user clicking the magic link (in production, they click the email):

```sh
oya community identity verify-complete \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --verification-token <token-from-magic-link>
```

Verify the identity is registered + the corporate-email claim sealed:

```sh
oya community identity show \
    --tenant drill-acme \
    --user u-alice@drill.test
# Output:
#   user_id: u-alice@drill.test
#   verified_claims:
#     - claim: corporate-email
#       value: alice@acme-real.example (HSM-vaulted; only sealed hash visible)
#       verified_at: 2026-05-20T14:32:17Z
#       verification_method: email-magic-link
#       trust_score: 0.85
```

Create an anonymous post (now permitted because identity is verified):

```sh
oya community post create \
    --tenant drill-acme \
    --board general \
    --author u-alice@drill.test \
    --anonymous true \
    --pseudonym auto-generated \
    --title "Anonymous question: how does promotion work here?" \
    --body "..."
# Output:
#   post_id=p_drill_anon_001
#   visible_author: "anon_engineer_4218" (auto-generated pseudonym; tied to verified identity in HSM vault)
#   verified_claims_shown: ["corporate-email-verified"]
```

The post is visible as authored by `anon_engineer_4218` with a "✓ verified employee" badge. The underlying identity is HSM-vaulted; only Cedar-authorized roles (e.g., on-call moderator with court-order Cedar permit) can de-anonymize.

Acceptance: identity verification verified; anonymous post created with TeamBlind-class verification.

## Day 4 — LLM-assisted moderation + moderation-storm runbook

Per IP-010 (foundry-guardrails-moderation-bridge), the `community` µservice consults `intelligence` for LLM-assisted moderation.

Configure moderation:

```sh
oya community tenant update \
    --tenant drill-acme \
    --moderation-policy '{
      "auto_flag_class": ["spam", "harassment", "csam", "doxxing", "hate-speech", "self-harm"],
      "auto_remove_class": ["csam"],
      "queue_for_human_class": ["spam", "harassment", "doxxing", "hate-speech", "self-harm"],
      "intelligence_model_class": "moderation_v3"
    }'
```

Post a spam-classified message:

```sh
oya community post create \
    --tenant drill-acme \
    --board general \
    --author u-bob@drill.test \
    --title "FREE BITCOIN! Click here NOW!" \
    --body "Visit example.com to claim your free bitcoin..."
# Expected:
#   post_id=p_drill_spam_001
#   status=quarantined-for-moderation
#   moderation_flag_class=spam
#   moderation_confidence=0.93
```

The post lands in the moderation queue, not the public feed. The on-call moderator (or auto-action policy) processes:

```sh
oya community moderation queue-list --tenant drill-acme --status pending
# Lists pending items with their classification confidence.

oya community moderation action \
    --tenant drill-acme \
    --post p_drill_spam_001 \
    --action remove \
    --reason spam \
    --moderator-user u-mod@drill.test
```

The action emits `community.moderation.applied` to audit-chain.

Walk the moderation-storm runbook. Read `runbooks/moderation-storm.md`. Scenario: a sudden spike in posts triggers the moderation queue to grow > 1000 pending. Runbook covers:

1. Identify the storm from `community-moderation-action-rate` panel.
2. Check if the storm is organic (e.g., a celebrity tweet drove traffic) or attack (spam flood).
3. If attack: trigger emergency rate-limit on the board.
4. Scale up moderation worker pool.
5. Enable conservative LLM-moderation (auto-remove instead of queue-for-human).
6. Audit-chain the storm event for post-incident review.

Target end-to-end recovery: ≤ 30 min for the drill.

Acceptance: LLM moderation verified; runbook walked.

## Day 5 — Cross-tenant federation + reputation portability

Configure cross-tenant federation:

```sh
oya community tenant update \
    --tenant drill-acme \
    --cross-tenant-federation true \
    --federation-allowlist drill-betta,drill-charlie
```

Create a cross-tenant board:

```sh
oya community board create \
    --tenant drill-acme \
    --board-id sales-engineers-everywhere \
    --display-name "Sales Engineers Everywhere" \
    --cross-tenant-mode federated \
    --federation-policy verified-corporate-email-only
```

Now a verified-corporate-email user at drill-betta can post to this board:

```sh
# Posting from drill-betta into drill-acme's federated board
oya community post create \
    --tenant drill-betta \
    --board-cross-tenant drill-acme/sales-engineers-everywhere \
    --author u-alice@drill-betta.test \
    --title "Hot take: technical sales is undervalued" \
    --body "..."
```

The post is visible to verified-corporate-email users across all federated tenants. Cedar enforces the cross-tenant write permission.

Reputation portability:

```sh
oya community reputation show --tenant drill-acme --user u-alice@drill-betta.test
# Expected:
#   reputation:
#     drill-acme/general: 24 (5 posts, 14 upvotes received)
#     drill-acme/sales-engineers-everywhere: 8 (1 post, 8 upvotes)
#   cross-tenant aggregate: 32
```

Acceptance: cross-tenant federation verified; reputation portability verified across boards.

## What you've learned

- demo_trial bootstrap + tenant + board + post + comment.
- Identity verification + anonymous mode (TeamBlind-class).
- LLM-assisted moderation via the `intelligence` µservice bridge.
- Cross-tenant federation with verified-corporate-email gates.
- Moderation-storm runbook.

Next week: paid promotion (multi-region identity + anonymous mode), paid advanced tour (LinkedIn reputation + Handshake jobs + marketplace integration), paid compliance-pack tour (SecureDrop bridge + sovereign-pack), and your first production shadow.

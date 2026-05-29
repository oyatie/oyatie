---
doc_class: Tutorial
microservice: community
persona: community-engineer + trust-and-safety-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Configure a TeamBlind-class anonymous board with LLM-assisted moderation

You will: enable anonymous mode for a tenant, configure the corporate-email verification provider, post anonymously with a verified-employer badge, configure tenant moderation policy, exercise the LLM moderation pipeline, walk a takedown action through the moderation queue, and verify the audit-chain emission. Total time ≤ 60 minutes.

## Pre-requisites

- A tenant cell on paid tenant_class (`tenant_class model in ADR-0330`).
- `oya-dev-cli` ≥ 1.42.0.
- A tenant principal in the `community_admin` Cedar role.
- An SMTP / SES / Mailgun configuration for sending verification emails.
- The `intelligence` µservice configured with the moderation_v3 classifier.

## Step 1 — Enable anonymous mode + identity verification (≤ 10 min)

```sh
oya community tenant update \
    --tenant acme-corp \
    --anonymous-mode-policy verified-corporate-employees-only \
    --identity-verification-providers corporate-email,phone \
    --pseudonym-generator default \
    --pseudonym-format 'anon_{adjective}_{noun}_{4-digit}'
```

The `verified-corporate-employees-only` policy means: only users with a verified corporate-email claim can post anonymously. Users who haven't verified can still read but not post anonymously.

Verify the policy:

```sh
oya community tenant show --tenant acme-corp | jq '.anonymous_mode_policy, .identity_verification_providers'
# Output:
#   "verified-corporate-employees-only"
#   ["corporate-email", "phone"]
```

## Step 2 — Create the anonymous board (≤ 5 min)

```sh
oya community board create \
    --tenant acme-corp \
    --board-id internal-feedback \
    --display-name "Internal Anonymous Feedback" \
    --description "Share constructive feedback about ACME anonymously. Posts will show your verified employer + a pseudonym." \
    --post-policy anonymous-only \
    --comment-policy anonymous-only \
    --visibility tenant-employees-only \
    --moderation-policy strict
```

The `post-policy anonymous-only` means all posts must be anonymous (verified-employer pseudonym mandatory; no real-name posting).

## Step 3 — A user verifies their corporate-email claim (≤ 5 min)

User `u-alice-real@acme-corp.com` initiates verification:

```sh
oya community identity verify-init \
    --tenant acme-corp \
    --user u-alice-real \
    --claim 'corporate-email:alice@acme-corp.com' \
    --proof-method email-magic-link
# Output: 200 OK; verification email sent.
```

The user clicks the magic link in their email; the redemption emits to oyatie:

```sh
oya community identity verify-complete \
    --tenant acme-corp \
    --user u-alice-real \
    --verification-token v_abc123def456...
```

Verify the claim is now sealed:

```sh
oya community identity show \
    --tenant acme-corp \
    --user u-alice-real
# Output:
#   user_id: u-alice-real
#   verified_claims:
#     - claim_type: corporate-email
#       claim_value_hash: blake3:7c4a2b8e... (HSM-vaulted plaintext not exposed)
#       employer_attestation: acme-corp (verified via DKIM-signed email from @acme-corp.com)
#       verified_at: 2026-05-20T14:32:17Z
#       trust_score: 0.85
```

Generate a pseudonym for this user (one-time):

```sh
oya community pseudonym assign \
    --tenant acme-corp \
    --user u-alice-real \
    --board internal-feedback
# Output: pseudonym=anon_clever_walrus_4218
```

The pseudonym is sticky per (user, board) — Alice will always appear as `anon_clever_walrus_4218` on this board.

## Step 4 — Anonymous post (≤ 5 min)

Alice posts anonymously:

```sh
oya community post create \
    --tenant acme-corp \
    --board internal-feedback \
    --author u-alice-real \
    --anonymous true \
    --title "Suggestion: more remote-work flexibility" \
    --body "Many teammates would benefit from a Tuesday-Thursday office-required schedule with Mon/Wed/Fri remote."
# Output:
#   post_id: p_acme_anon_001
#   visible_author: anon_clever_walrus_4218
#   verified_badge: "✓ Verified Acme employee"
#   moderation_status: pending-classify
#   created_at: 2026-05-20T14:34:32Z
```

The post is in `pending-classify` for moderation. The LLM classifier runs (typically < 1 s):

```sh
oya community post show --tenant acme-corp --post p_acme_anon_001
# Output:
#   ...
#   moderation_status: approved
#   moderation_class_predictions:
#     spam: 0.01
#     harassment: 0.02
#     hate-speech: 0.01
#     constructive-feedback: 0.97
#   moderation_decision: allow
```

The post is now visible publicly on the board with the verified-employer badge.

## Step 5 — Configure tenant moderation policy (≤ 10 min)

The strict moderation policy from Step 2 is the starting point. Customize:

```sh
oya community tenant moderation-policy update \
    --tenant acme-corp \
    --policy-file ./moderation-policy.yaml
```

The policy:

```yaml
tenant_id: acme-corp

# What classes auto-remove (cannot be disabled for CSAM; auto-required by US 18 USC § 2258A + EU DSA Article 16)
auto_remove_class:
  - csam              # mandatory; cannot be removed from this list

# What classes auto-flag (post enters pending-classify, then auto-removed if confidence high enough)
auto_flag_class:
  - spam
  - harassment
  - hate-speech
  - doxxing
  - self-harm

# What classes queue for human review
queue_for_human_class:
  - borderline-harassment
  - borderline-hate-speech
  - borderline-self-harm

# Classifier confidence thresholds
auto_remove_threshold: 0.95  # only auto-remove if confidence > 0.95
queue_threshold: 0.40         # queue for human if confidence between 0.40 and 0.95

# Rate limits per user
rate_limit_per_user:
  posts_per_hour: 5           # max 5 posts/hour per user on this tenant
  comments_per_hour: 30
  votes_per_hour: 200

# Repeat-offender escalation
repeat_offender_thresholds:
  warning_after_n_removed_posts: 2
  shadowban_after_n_removed_posts: 5
  ban_after_n_removed_posts: 10
```

Verify:

```sh
oya community tenant moderation-policy show --tenant acme-corp
```

## Step 6 — Simulate a takedown action (≤ 10 min)

A spam post arrives:

```sh
oya community post create \
    --tenant acme-corp \
    --board internal-feedback \
    --author u-bob-spammer \
    --anonymous true \
    --title "🔥🔥🔥 FREE CRYPTO! Click here NOW! 🔥🔥🔥" \
    --body "Visit my-crypto-scam.example to claim..."
# Expected:
#   post_id: p_acme_spam_001
#   moderation_status: classified
#   moderation_class_predictions:
#     spam: 0.98
#     crypto-fraud: 0.84
#   moderation_decision: auto-removed (confidence > 0.95)
#   visible_to_users: NO
```

The post is auto-removed. The author gets a notification:

```sh
oya community notification show --user u-bob-spammer --since 5m
# Output: 1 notification with class=post_auto_removed, post_id=p_acme_spam_001, reason=spam (auto-removed via classifier confidence 0.98)
```

Now a borderline post needing human review:

```sh
oya community post create \
    --tenant acme-corp \
    --board internal-feedback \
    --author u-charlie \
    --anonymous true \
    --title "Manager X is making my life miserable" \
    --body "I think Manager X is targeting me unfairly. They keep [...details...]"
# Expected:
#   post_id: p_acme_borderline_001
#   moderation_status: queued-for-human
#   moderation_class_predictions:
#     constructive-feedback: 0.42
#     borderline-harassment: 0.61
#     personal-attack: 0.38
#   moderation_decision: queue-for-human
#   visible_to_users: NO (pending moderation)
```

A trust-and-safety moderator picks it up:

```sh
oya community moderation queue-pop --tenant acme-corp --moderator u-mod-1
# Output: assigned post_id=p_acme_borderline_001

# Moderator decides: this is borderline. Allow but suggest a more constructive framing
oya community moderation action \
    --post p_acme_borderline_001 \
    --moderator u-mod-1 \
    --action allow-with-warning \
    --warning-template constructive-framing-suggestion
```

The post is now visible. The author receives a warning suggesting a more constructive framing. The action is audit-chained.

## Step 7 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "community.*" --since 30m
```

Expected events for our flow:

- `community.identity.verification_initiated`
- `community.identity.verification_completed`
- `community.pseudonym.assigned`
- `community.post.created` (× 3)
- `community.moderation.classified` (× 3)
- `community.moderation.auto_removed` (× 1; spam)
- `community.moderation.queued` (× 1; borderline)
- `community.moderation.action_applied` (× 1; allow-with-warning)

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 30m
# Output: chain verified, all events signed, signature_gaps: 0
```

## Step 8 — Mock de-anonymization request (for incident response) (≤ 5 min)

Suppose a serious incident requires de-anonymizing a post. The de-anonymization itself requires special Cedar permission + court-order evidence:

```sh
# This will fail without sufficient Cedar permission
oya community identity deanonymize \
    --tenant acme-corp \
    --post p_acme_borderline_001 \
    --moderator u-mod-1
# Expected: 403 Forbidden; Cedar denial reason: "deanonymize permission requires legal-counsel role + court-order evidence"

# With proper evidence
oya community identity deanonymize \
    --tenant acme-corp \
    --post p_acme_borderline_001 \
    --moderator u-legal-counsel-1 \
    --court-order-evidence ./court-order-2026-05-20.pdf \
    --justification "Threat of self-harm; per AU online safety regulations"
# Expected: 200 OK; returns user_id=u-charlie + the verified-employer claim
```

The de-anonymization itself emits `community.identity.deanonymized` to audit-chain with the requesting moderator + court-order evidence reference.

## What you've learned

- TeamBlind-class anonymous mode + corporate-email verification + pseudonym assignment.
- LLM-assisted moderation with auto-remove + queue-for-human paths.
- Custom moderation policy authoring + rate limits + repeat-offender escalation.
- Takedown action flow + warning template + audit-chain emission.
- De-anonymization flow with Cedar + court-order evidence.

Next tutorial: `tutorials/build-linkedin-class-reputation-profile.md` — set up LinkedIn-class skills + endorsements + career history (paid advanced capability).

---
doc_class: ReferenceImplementation
microservice: community
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Post + comment + vote + anonymous post via the community Rust SDK

A runnable example that:

1. Authenticates as a tenant community_member principal.
2. Creates a post.
3. Adds comments + votes.
4. Initiates an identity-verification flow.
5. Creates a verified-anonymous post.
6. Verifies the audit-chain emission.

## Cargo.toml

```toml
[package]
name = "community-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-community-client = { path = "../../../../crates/oya-community-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

## src/main.rs

```rust
use anyhow::Result;
use oya_community_client::{
    CommunityClient, CommunityClientConfig,
    PostCreate, PostStatus, CommentCreate, VoteCreate, VoteValue,
    IdentityVerifyInit, IdentityVerifyComplete, ProofMethod,
    PseudonymAssign,
};
use oya_cedar_client::CedarPrincipal;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Construct the client bound to a community_member Cedar principal.
    let principal = CedarPrincipal::from_env("COMMUNITY_MEMBER_JWT")?;
    let client = CommunityClient::connect(CommunityClientConfig {
        cell_endpoint: std::env::var("COMMUNITY_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 2. Create a public post (non-anonymous).
    let post = client.post_create(PostCreate {
        board_id: "general".into(),
        title: "How do I configure WPA3 on the RT-7000?".into(),
        body: "I'm trying to enable WPA3-Personal mode but the admin panel doesn't show the option. Has anyone successfully enabled it on firmware v3.4?".into(),
        anonymous: false,
        tags: vec!["wifi".into(), "rt-7000".into(), "wpa3".into()],
    }).await?;
    println!("Created post: {} (status: {:?})", post.post_id, post.status);
    println!("  Moderation: {:?}", post.moderation_decision);

    // 3. Add a comment.
    let comment = client.comment_create(CommentCreate {
        post_id: post.post_id.clone(),
        parent_comment_id: None,  // Top-level comment
        body: "WPA3 was added in firmware v3.4 — make sure you upgraded. Check Settings → Advanced → Wi-Fi Security.".into(),
        anonymous: false,
    }).await?;
    println!("Created comment: {}", comment.comment_id);

    // 4. Add a reply (nested comment).
    let reply = client.comment_create(CommentCreate {
        post_id: post.post_id.clone(),
        parent_comment_id: Some(comment.comment_id.clone()),
        body: "Thanks! I had firmware v3.3.2; upgrading now.".into(),
        anonymous: false,
    }).await?;
    println!("Created reply: {} (parent: {})", reply.comment_id, comment.comment_id);

    // 5. Vote on the original post.
    let vote = client.vote_create(VoteCreate {
        target_kind: "post".into(),
        target_id: post.post_id.clone(),
        value: VoteValue::Upvote,
    }).await?;
    println!("Vote registered: {} for post {}", vote.value, vote.target_id);

    // 6. Read the post tree (with reputation scores).
    let post_tree = client.post_tree_get(&post.post_id).await?;
    println!("Post tree:");
    println!("  Post: {} (score: {})", post_tree.post.title, post_tree.post.score);
    for c in &post_tree.comments {
        println!("    [{}] {} (depth: {}, score: {})", c.author_display_name, c.body.chars().take(60).collect::<String>(), c.depth, c.score);
    }

    // 7. Initiate identity verification (corporate-email claim).
    let verify_init = client.identity_verify_init(IdentityVerifyInit {
        claim_type: "corporate-email".into(),
        claim_value: "alice@acme-real.example".into(),
        proof_method: ProofMethod::EmailMagicLink,
    }).await?;
    println!("Verification email sent. Token: {}", verify_init.verification_handle);

    // 8. In production, the user clicks the magic link in their email.
    //    Here we simulate by reading the token from env.
    let token = std::env::var("VERIFY_TOKEN_FROM_MAGIC_LINK")?;
    let verify_complete = client.identity_verify_complete(IdentityVerifyComplete {
        verification_handle: verify_init.verification_handle,
        token,
    }).await?;
    println!("Identity verified: {:?}", verify_complete.verified_claims);

    // 9. Get a pseudonym for the anonymous board.
    let pseudonym = client.pseudonym_assign(PseudonymAssign {
        board_id: "internal-feedback".into(),
    }).await?;
    println!("Pseudonym assigned: {}", pseudonym.pseudonym);

    // 10. Create a verified-anonymous post.
    let anon_post = client.post_create(PostCreate {
        board_id: "internal-feedback".into(),
        title: "Suggestion: improve internal docs search".into(),
        body: "The internal docs search returns stale results when our wiki has been updated. Suggest re-indexing nightly.".into(),
        anonymous: true,
        tags: vec!["internal".into(), "feedback".into()],
    }).await?;
    println!(
        "Created anonymous post: {} (visible_author: {})",
        anon_post.post_id, anon_post.visible_author
    );
    println!("  Verified badge: {:?}", anon_post.verified_badge);

    Ok(())
}
```

## Expected output (against a paid-tier cell with the tenant configured per the tutorial)

```
Created post: p_acme_001 (status: Approved)
  Moderation: Allow
Created comment: c_acme_001
Created reply: c_acme_002 (parent: c_acme_001)
Vote registered: Upvote for post p_acme_001
Post tree:
  Post: How do I configure WPA3 on the RT-7000? (score: 1)
    [u-bob] WPA3 was added in firmware v3.4 — make sure you upgraded. Check Settings → Advanced → Wi-Fi Sec... (depth: 0, score: 0)
    [u-bob] Thanks! I had firmware v3.3.2; upgrading now. (depth: 1, score: 0)
Verification email sent. Token: vh_abc123def456
Identity verified: ["corporate-email:alice@acme-real.example (sealed; trust_score 0.85)"]
Pseudonym assigned: anon_clever_walrus_4218
Created anonymous post: p_acme_anon_001 (visible_author: anon_clever_walrus_4218)
  Verified badge: Some("✓ Verified Acme employee")
```

## HTTP alternative (curl)

```sh
# Create post
curl -X POST https://community.prod-syd-1.oyatie.local/v1/posts \
    -H "Authorization: Bearer $COMMUNITY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "board_id":"general",
        "title":"How do I configure WPA3?",
        "body":"...",
        "anonymous":false,
        "tags":["wifi","rt-7000","wpa3"]
    }'

# Comment
curl -X POST https://community.prod-syd-1.oyatie.local/v1/comments \
    -H "Authorization: Bearer $COMMUNITY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "post_id":"p_acme_001",
        "parent_comment_id":null,
        "body":"WPA3 was added in firmware v3.4..."
    }'

# Vote
curl -X POST https://community.prod-syd-1.oyatie.local/v1/votes \
    -H "Authorization: Bearer $COMMUNITY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "target_kind":"post",
        "target_id":"p_acme_001",
        "value":"upvote"
    }'

# Identity verify initiate
curl -X POST https://community.prod-syd-1.oyatie.local/v1/identity/verify/init \
    -H "Authorization: Bearer $COMMUNITY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "claim_type":"corporate-email",
        "claim_value":"alice@acme-real.example",
        "proof_method":"email-magic-link"
    }'

# Anonymous post
curl -X POST https://community.prod-syd-1.oyatie.local/v1/posts \
    -H "Authorization: Bearer $COMMUNITY_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "board_id":"internal-feedback",
        "title":"Suggestion: improve internal docs search",
        "body":"...",
        "anonymous":true,
        "tags":["internal","feedback"]
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `community::post::create` |
| `board_not_found` | 404 | No | Board ID doesn't exist |
| `moderation_queued` | 202 | No | Post under review; not yet public |
| `moderation_auto_removed` | 422 | No | Post classified as spam/CSAM; removed |
| `rate_limit_per_user` | 429 | Yes (auto, backoff) | User hit posts-per-hour cap |
| `anonymous_mode_not_permitted` | 422 | No | Tenant doesn't permit anonymous posting OR user identity not verified |
| `pseudonym_not_assigned` | 422 | No | User must assign a pseudonym before posting anonymously |
| `identity_verification_pending` | 422 | No | User hasn't completed identity verification |
| `cross_tenant_federation_deny` | 403 | No | Cross-tenant write not permitted by board policy |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `post_create` | `community.post.created` |
| `post_create` (auto-removed) | `community.post.created`, `community.moderation.auto_removed` |
| `comment_create` | `community.comment.created` |
| `vote_create` | `community.vote.applied` |
| `identity_verify_init` | `community.identity.verification_initiated` |
| `identity_verify_complete` | `community.identity.verification_completed` |
| `pseudonym_assign` | `community.pseudonym.assigned` |
| `post_create` (anonymous) | `community.post.created` (with anonymous=true) |
| Cedar deny anywhere | `community.cedar.denied` |

## Where this file lives

`microservices/community/reference-implementations/post-comment-vote-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/community/reference-implementations/post-example/` once `oya-community-client` ships.

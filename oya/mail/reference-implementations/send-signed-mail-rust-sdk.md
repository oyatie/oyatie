---
doc_class: ReferenceImplementation
microservice: mail
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Send DKIM-signed mail via the mail Rust SDK

A runnable example that:

1. Authenticates as a tenant mail_sender principal.
2. Looks up the active DKIM selector for the sending domain.
3. Composes a message with proper headers.
4. Requests a signing lease from OpenBao (per ADR-MAIL-001).
5. DKIM-signs + sends via SMTP.
6. Verifies the audit-chain emission.

## Cargo.toml

```toml
[package]
name = "mail-send-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-mail-client = { path = "../../../../crates/oya-mail-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
mail-builder = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use oya_mail_client::{
    MailClient, MailClientConfig,
    MessageCompose, MessageHeaders, SigningRequest, SmtpDeliveryOptions,
    DkimAlgorithm, DmarcAlignmentMode,
};
use oya_cedar_client::CedarPrincipal;
use mail_builder::MessageBuilder;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Construct the client bound to a mail_sender Cedar principal.
    let principal = CedarPrincipal::from_env("MAIL_SENDER_JWT")?;
    let client = MailClient::connect(MailClientConfig {
        cell_endpoint: std::env::var("MAIL_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(60),
    }).await?;

    // 2. Look up the active DKIM selector for the sending domain.
    let domain_info = client.domain_show(
        "dom_acme_com_001",
    ).await?;
    let active_selector = domain_info.dkim_selectors
        .iter()
        .find(|s| s.state == "active")
        .ok_or_else(|| anyhow::anyhow!("no active DKIM selector"))?;
    info!("Active DKIM selector: {} (algorithm: {})",
          active_selector.selector, active_selector.algorithm);

    // 3. Compose the message using mail-builder.
    let message_bytes = MessageBuilder::new()
        .from(("Alice", "alice@acme.com"))
        .to(("Bob", "bob@external.example"))
        .subject("Hello from oyatie mail")
        .text_body("This is a DKIM-signed test message from the oyatie mail Rust SDK.\n\nBest,\nAlice")
        .write_to_vec()?;

    // 4. Request signing lease from OpenBao + sign the message.
    let signed_message = client.message_sign(SigningRequest {
        domain_id: "dom_acme_com_001".into(),
        selector: active_selector.selector.clone(),
        message_bytes: message_bytes.clone(),
        canonicalization_header: "relaxed".into(),
        canonicalization_body: "relaxed".into(),
        idempotency_key: format!("send-{}", chrono::Utc::now().timestamp_millis()),
    }).await?;
    info!("Message signed: DKIM-Signature header added (selector={}, algorithm={})",
          active_selector.selector, active_selector.algorithm);

    // 5. Send via SMTP with SPF + DMARC alignment evaluated server-side.
    let delivery = client.message_send(MessageCompose {
        from_address: "alice@acme.com".into(),
        to_addresses: vec!["bob@external.example".into()],
        message_bytes: signed_message.signed_message_bytes,
        smtp_options: SmtpDeliveryOptions {
            require_mta_sts: true,
            require_tls: true,
            arc_forwarder_overrides_applied: vec![],
            outbound_signing_lease_id: signed_message.lease_id,
        },
    }).await?;
    info!("Message sent: msg_id={}, dmarc_result={}, audit_event_id={}",
          delivery.message_id, delivery.dmarc_alignment_result, delivery.audit_event_id);

    // 6. Query the audit-chain to verify the emission.
    let audit_event = client.audit_event_show(&delivery.audit_event_id).await?;
    info!("Audit event: class={}, signature_valid={}",
          audit_event.event_class, audit_event.signature_valid);

    Ok(())
}
```

## Expected output (against a `tenant_class=paid` cell with the tenant configured per the tutorial)

```
INFO Active DKIM selector: s20260520a (algorithm: Ed25519)
INFO Message signed: DKIM-Signature header added (selector=s20260520a, algorithm=Ed25519)
INFO Message sent: msg_id=m_acme_outbound_001, dmarc_result=pass, audit_event_id=ae_mail_outbound_signed_001
INFO Audit event: class=mail.auth.outbound.signed.v1, signature_valid=true
```

## HTTP alternative (curl)

```sh
# 1. Look up domain DKIM selectors
curl -X GET https://mail.prod-syd-1.oyatie.local/v1/mail/auth/domains/dom_acme_com_001/dns-records \
    -H "Authorization: Bearer $MAIL_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp"

# 2. Compose + sign + send
curl -X POST https://mail.prod-syd-1.oyatie.local/v1/mail/messages/send \
    -H "Authorization: Bearer $MAIL_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "from":"alice@acme.com",
        "to":["bob@external.example"],
        "subject":"Hello from oyatie mail",
        "body_text":"This is a DKIM-signed test message.",
        "smtp_options":{
            "require_mta_sts":true,
            "require_tls":true,
            "selector":"s20260520a"
        }
    }'
# Response:
#   {
#     "message_id":"m_acme_outbound_001",
#     "dkim_signature":"v=1; a=ed25519-sha256; d=acme.com; s=s20260520a; ...",
#     "dmarc_alignment_result":"pass",
#     "audit_event_id":"ae_mail_outbound_signed_001"
#   }

# 3. Inbound mail (server receives + auth-evaluates)
# This is server-driven; no client call. The server emits:
#   - oya.mail.auth.inbound.spf_evaluated.v1
#   - oya.mail.auth.inbound.dkim_evaluated.v1
#   - oya.mail.auth.inbound.dmarc_evaluated.v1
#   - oya.mail.auth.inbound.arc_evaluated.v1
#   - oya.mail.auth.inbound.delivered.v1 (or rejected.v1 / quarantined.v1)

# 4. JMAP RFC 8620 client integration (for mail clients)
# Initial JMAP session
curl -X POST https://mail.prod-syd-1.oyatie.local/jmap/session \
    -H "Authorization: Bearer $MAIL_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp"

# Get mailboxes
curl -X POST https://mail.prod-syd-1.oyatie.local/jmap/api \
    -H "Authorization: Bearer $MAIL_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "using":["urn:ietf:params:jmap:mail"],
        "methodCalls":[
            ["Mailbox/get", {"accountId":"acme-corp.alice@acme.com"}, "0"]
        ]
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `mail::message::send` or domain ownership |
| `domain_not_verified` | 422 | No | Tenant must verify domain first via `mail::domain::verify` |
| `dkim_selector_not_active` | 422 | No | Activate a selector via `mail::dkim_selector::activate` |
| `openbao_signing_lease_failed` | 503 | Yes (auto, 1-5s backoff) | OpenBao temporarily unavailable; lease expired before use |
| `dmarc_alignment_failed` | 422 | No (config error) | Sender header doesn't align with DKIM/SPF; fix sender domain |
| `mta_sts_policy_violation` | 451 | No (retry later) | Recipient MTA doesn't satisfy MTA-STS; tenant alert + TLSRPT |
| `outbound_auth_hold` | 503 | No (queued) | Message held because alignment couldn't be confirmed; auto-resume |
| `rate_limit_per_user` | 429 | Yes (auto, backoff) | User hit per-hour cap |
| `pack_residency_violation` | 403 | No | Pack requires home-cell signing; cross-region signing denied |
| `dmarc_inbound_reject` | 550 | No (sender problem) | Inbound message DMARC-rejected at SMTP |
| `eu_ai_act_classifier_unavailable` | 503 | Yes | LLM spam classifier denied for pack; falling back to Rspamd |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `domain_verify` | `mail.auth.domain.verified.v1` |
| `dkim_selector_activate` | `mail.auth.dkim.selector.activated.v1` |
| `dmarc_policy_promote` | `mail.auth.dmarc.policy.promoted.v1` |
| `arc_forwarder_grant` | `mail.auth.arc-forwarder.granted.v1` |
| `message_sign` | `mail.auth.outbound.signed.v1` |
| `message_send` (delivered) | `mail.auth.outbound.delivered.v1` |
| `message_send` (held) | `mail.auth.outbound.held.v1` |
| `inbound_dmarc_evaluated` | `mail.auth.dmarc.disposition.v1` |
| `inbound_dmarc_rejected` | `mail.auth.dmarc.inbound.rejected.v1` |
| `signing_failure` | `mail.auth.signing.failure.v1` |
| Cedar deny anywhere | `mail.cedar.denied.v1` |

## Where this file lives

`microservices/mail/reference-implementations/send-signed-mail-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/mail/reference-implementations/send-example/` once `oya-mail-client` ships.

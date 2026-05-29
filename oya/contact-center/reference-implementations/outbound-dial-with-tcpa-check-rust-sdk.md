---
doc_class: ReferenceImplementation
microservice: contact-center
language: rust
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Reference — Place an outbound call with TCPA + DNC compliance checks (Rust SDK)

This walkthrough shows a tenant's CRM-side worker placing an outbound call via the oyatie contact-center substrate while enforcing US TCPA (47 CFR § 64.1200) compliance — checking the National Do-Not-Call Registry, the tenant's internal DNC list, calling-time-window restrictions (8 AM-9 PM caller-local), and abandonment-rate cap.

## Cargo.toml

```toml
[package]
name = "outbound-dialer-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-contact-center-sdk = { path = "../../crates/oya-contact-center-sdk" }
oya-iam-sdk = { path = "../../crates/oya-iam-sdk" }
oya-observability-sdk = { path = "../../crates/oya-observability-sdk" }
tokio = { version = "1.42", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.10"
anyhow = "1.0"
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
phonenumber = "0.3"
```

## src/main.rs

```rust
use anyhow::{anyhow, Context};
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use oya_contact_center_sdk::{
    ContactCenterClient, DncCheckResult, OutboundCallRequest, OutboundCallResponse,
};
use oya_iam_sdk::{IamClient, Principal};
use oya_observability_sdk::ObservabilityGuard;
use phonenumber::PhoneNumber;
use tracing::{error, info, warn};

struct DialerWorker {
    cc_client: ContactCenterClient,
    iam_client: IamClient,
    tenant_id: String,
    campaign_id: String,
}

impl DialerWorker {
    async fn dial_one(&self, lead: Lead) -> anyhow::Result<OutboundCallResponse> {
        let phone: PhoneNumber = lead.phone_e164.parse()
            .with_context(|| format!("invalid E.164 phone: {}", lead.phone_e164))?;

        if !self.is_callable_time(&lead).await? {
            return Err(anyhow!(
                "outside callable window 8AM-9PM local for {} ({})",
                lead.phone_e164, lead.timezone
            ));
        }

        let dnc = self
            .cc_client
            .check_dnc(&self.tenant_id, &lead.phone_e164)
            .await
            .context("dnc check failed")?;
        if matches!(dnc, DncCheckResult::OnFederalDnc | DncCheckResult::OnTenantDnc) {
            warn!(phone = %lead.phone_e164, ?dnc, "DNC hit; skipping");
            return Err(anyhow!("on DNC: {:?}", dnc));
        }

        let consent = self
            .cc_client
            .check_express_written_consent(&self.tenant_id, &lead.phone_e164)
            .await
            .context("express-written-consent lookup failed")?;
        if !consent.has_valid_consent {
            warn!(phone = %lead.phone_e164, "no express written consent; skipping per TCPA 64.1200(a)(2)");
            return Err(anyhow!("no express written consent"));
        }

        let abandonment_window = self.cc_client
            .get_abandonment_rate_30d(&self.tenant_id, &self.campaign_id)
            .await
            .context("abandonment rate lookup failed")?;
        if abandonment_window.rate_30d > 0.025 {
            warn!(rate = abandonment_window.rate_30d, "abandonment headroom thin; deferring");
            return Err(anyhow!(
                "abandonment rate {} > 2.5% headroom for 3% TCPA cap; pause campaign",
                abandonment_window.rate_30d
            ));
        }

        let call_request = OutboundCallRequest {
            tenant_id: self.tenant_id.clone(),
            campaign_id: self.campaign_id.clone(),
            destination_e164: lead.phone_e164.clone(),
            caller_id_e164: lead.intended_caller_id.clone(),
            stir_shaken_attestation: "A".into(),
            recording_policy: "always-on".into(),
            consent_evidence_id: consent.evidence_id.clone(),
            agent_queue_on_answer: Some("outbound-sales-q".into()),
            max_ring_seconds: 25,
            metadata: serde_json::json!({
                "lead_id": lead.lead_id,
                "campaign_name": "spring-2026-renewals",
            }),
        };

        let response = self
            .cc_client
            .place_outbound_call(call_request)
            .await
            .context("place_outbound_call failed")?;

        info!(
            call_id = %response.call_id,
            phone = %lead.phone_e164,
            "outbound call placed"
        );

        Ok(response)
    }

    async fn is_callable_time(&self, lead: &Lead) -> anyhow::Result<bool> {
        let tz: Tz = lead.timezone.parse()
            .with_context(|| format!("invalid timezone: {}", lead.timezone))?;
        let now_local = Utc::now().with_timezone(&tz);
        let hour = now_local.hour();
        Ok((8..21).contains(&hour))
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Lead {
    lead_id: String,
    phone_e164: String,
    timezone: String,
    intended_caller_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _observability = ObservabilityGuard::init("outbound-dialer-worker")?;

    let cc_client = ContactCenterClient::from_env()?;
    let iam_client = IamClient::from_env()?;

    let principal: Principal = iam_client.whoami().await?;
    let tenant_id = principal.tenant_id.clone();

    let worker = DialerWorker {
        cc_client,
        iam_client,
        tenant_id,
        campaign_id: "spring-2026-renewals".into(),
    };

    let leads: Vec<Lead> = serde_json::from_str(
        &tokio::fs::read_to_string("./leads.json").await?,
    )?;

    for lead in leads {
        match worker.dial_one(lead.clone()).await {
            Ok(resp) => info!(call_id = %resp.call_id, lead_id = %lead.lead_id, "ok"),
            Err(e) => warn!(lead_id = %lead.lead_id, error = %e, "skipped"),
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(())
}
```

## leads.json (example input)

```json
[
  {
    "lead_id": "L-001",
    "phone_e164": "+15555551234",
    "timezone": "America/Los_Angeles",
    "intended_caller_id": "+18005551111"
  },
  {
    "lead_id": "L-002",
    "phone_e164": "+15555556789",
    "timezone": "America/New_York",
    "intended_caller_id": "+18005551111"
  }
]
```

## Required Cedar permits

The worker's principal must hold these Cedar permits:

```cedar
permit (
    principal == User::"outbound-dialer-worker@tenant-acme",
    action in [
        Action::"contact_center::dnc::check",
        Action::"contact_center::consent::check",
        Action::"contact_center::abandonment_rate::read",
        Action::"contact_center::outbound::place_call"
    ],
    resource in Tenant::"tenant_acme"
);
```

## Compliance evidence emitted

Every successful `place_outbound_call` emits to `audit-chain`:

```json
{
    "event_class": "contact_center::outbound::call_placed",
    "tenant_id": "tenant_acme",
    "call_id": "call_01HXYZ...",
    "destination_e164": "+15555551234",
    "caller_id_e164": "+18005551111",
    "stir_shaken_attestation": "A",
    "consent_evidence_id": "consent_evt_...",
    "dnc_check_at": "2026-05-20T14:32:11Z",
    "tcpa_callable_window_check_at": "2026-05-20T14:32:11Z",
    "abandonment_rate_30d_at_dial_time": 0.018,
    "campaign_id": "spring-2026-renewals"
}
```

This chain-of-custody satisfies FCC TCPA enforcement evidence + private-right-of-action defence (47 USC § 227(b)(3)).

## Run + verify

```sh
OYA_TENANT_ID=tenant_acme \
OYA_CONTACT_CENTER_API=https://contact-center-api.dev.oyatie.io \
OYA_IAM_API=https://iam-api.dev.oyatie.io \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    cargo run --release
```

Verify in the admin portal: portal → Outbound Campaigns → "spring-2026-renewals" → see the calls placed, DNC skips, consent skips. Verify in `audit-chain`: `oya audit-chain query --tenant tenant_acme --event-class contact_center::outbound::call_placed --since "1 hour ago"`.

## Notes

- The TCPA callable window (8 AM-9 PM caller-local) is a US federal floor. Some states tighten further (e.g. Colorado, Florida prohibit calls on Sundays for certain categories); the substrate's `check_callable_time` extended-rule set encodes the state overlays — set `respect_state_overlays = true` in your campaign config to apply them.
- Express written consent (TCPA 64.1200(a)(2)) requires a signed consent record obtained before any auto-dial; the `check_express_written_consent` call validates against your tenant's consent registry in the `consent-graph` µservice.
- Abandonment rate is enforced fleet-wide; if your campaign nudges the tenant's 30-day rate above 3 %, the substrate auto-pauses the campaign and notifies the admin. You cannot disable this enforcement.
- For international outbound, the equivalent regulations apply: KR PIPA Art. 22 + 통신비밀보호법; EU GDPR Art. 6 + ePrivacy Directive 2002/58/EC; UK PECR + ICO TPS check. The substrate's `check_dnc` cascades through the relevant jurisdiction's DNC/TPS registry based on the destination E.164 country code.

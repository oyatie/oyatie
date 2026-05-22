# Migration playbook — Stripe + Twilio SDKs → Oyatie `developer-sdk`

Audience: a developer with multiple Stripe + Twilio SDKs woven through their codebase who wants to migrate the equivalent
functionality to Oyatie's `developer-sdk` (using the `payments` and `comms-email` + `meet` + `messenger` µservices as targets).

> Phase budget: 30 days per language per source vendor for a clean migration. Multi-language migrations parallelize.

## Phase 0 — Inventory (Day 0…3)

1. Find every Stripe SDK import:
   ```bash
   rg "from stripe|require\\(['\"]stripe['\"]\\)|import Stripe|use stripe::" -t py -t js -t rb -t go -t java
   ```
2. Find every Twilio import:
   ```bash
   rg "from twilio|require\\(['\"]twilio['\"]\\)|import com.twilio|using Twilio" -t py -t js -t rb -t go -t cs -t java
   ```
3. Per call site, classify into one of:
   - **Stripe → payments µservice**: `stripe.PaymentIntent.create` → `client.payments.intent.create`.
   - **Stripe → marketplace µservice**: `stripe.Connect.account.create` → `client.marketplace.merchant.onboard`.
   - **Twilio → comms-email**: `twilio.messages.create` (SMS) → `client.comms_email.send_sms`.
   - **Twilio Programmable Voice → meet µservice**: `twilio.calls.create` → `client.meet.dial`.
   - **Twilio Programmable Chat / Conversations → messenger µservice**: `twilio.conversations.*` → `client.messenger.thread.*`.

## Phase 1 — Install `developer-sdk` (Day 3…5)

Node:
```bash
npm config set @oyatie:registry https://registry.oyatie.io/npm
npm install @oyatie/sdk@stable
```

Python:
```bash
pip install oya-canonical-sdk
```

Rust:
```toml
[dependencies]
oya-canonical-sdk = "0.42.0"
```

Go:
```bash
go get github.com/oyatie/sdk-go@v0.42.0
```

Java/Kotlin (Maven):
```xml
<dependency>
  <groupId>io.oyatie</groupId>
  <artifactId>sdk</artifactId>
  <version>0.42.0</version>
</dependency>
```

## Phase 2 — Side-by-side mapping (Day 5…15)

| Stripe API call | Oyatie equivalent |
| --- | --- |
| `stripe.Customer.create(email=...)` | `client.payments.customer.create({ email })` |
| `stripe.PaymentIntent.create(amount=..., currency=..., customer=...)` | `client.payments.intent.create({ amount, currency, customerId })` |
| `stripe.PaymentMethod.attach(pm_id, customer=...)` | `client.payments.payment_method.attach(pmId, { customerId })` |
| `stripe.Subscription.create(customer=..., items=...)` | `client.payments.subscription.create({ customerId, items })` |
| `stripe.Webhook.construct_event(payload, signature, secret)` | `client.payments.webhook.verify(payload, signature)` — secret is fetched from `cloud-secrets` |
| `stripe.Connect.account.create(...)` | `client.marketplace.merchant.onboard(...)` |

| Twilio API call | Oyatie equivalent |
| --- | --- |
| `twilio.messages.create(to=..., from_=..., body=...)` | `client.comms_email.sms.send({ to, fromNumber, body })` |
| `twilio.calls.create(to=..., from_=..., twiml=...)` | `client.meet.dial({ to, fromNumber, ivrScript })` |
| `twilio.conversations.v1.conversations.create(...)` | `client.messenger.thread.create(...)` |
| `twilio.verify.v2.services(sid).verifications.create(to=..., channel=...)` | `client.identity.verify.start({ to, channel })` |

## Phase 3 — Shadow-mode dual-fire (Day 15…25)

For each migrated call site, fire both old and new SDKs in parallel:
```python
async def create_payment(amount: int, currency: str, customer: str):
    stripe_result = stripe.PaymentIntent.create(amount=amount, currency=currency, customer=customer)
    try:
        oya_result = await oya_client.payments.intent.create({ amount, currency, customerId: customer })
        await migration_diff.record(stripe_result, oya_result)  # logs differences for review
    except Exception as e:
        await migration_diff.record_error(stripe_result, e)
    return stripe_result  # source of truth still Stripe
```

Aim for 99.95 % outcome-parity over 10 days before flipping.

## Phase 4 — Flip + remove old SDK (Day 25…30)

Per call site:
1. Make the Oyatie call the source of truth.
2. Keep the Stripe/Twilio call as the shadow for 5 more days.
3. Remove the shadow after 5 days clean.
4. Run `cargo upgrade` / `npm uninstall stripe twilio` / etc.

## Phase 5 — Cutover billing + numbers (Day 30+)

If your business depends on Stripe Connect or Twilio phone numbers:
- Stripe Connect → `marketplace` µservice (ADR-0314 universal deal-settlement). Stripe accounts can be linked rather than migrated;
  the `marketplace` µservice orchestrates both Stripe and direct settlement.
- Twilio phone numbers → port to a SIP carrier supported by `meet` (Twilio Elastic SIP, Bandwidth, Telnyx). Number porting is a 7-30
  day process driven by the carrier; `meet` provisions the Oyatie side.

## Rollback

Within the 5-day shadow window: re-flip the source-of-truth back to Stripe/Twilio. The shadow data is preserved; rollback latency
is ≤ 1 deploy.

After SDK removal: rollback requires re-adding the vendor SDK + re-routing the call site. Plan on 1-2 d per call site.

## What you gain

- 13-22 languages from one publish (vs 7-9 per vendor).
- Cosign-signed artifacts.
- Cedar at the SDK — fail-fast permit checks.
- HTTP/3 + tail-latency wins.
- One tenant primitive across all Oyatie µservices (vs Stripe account-id + Twilio account-sid mental models).
- Audit chain across SDK calls.

## What you give up

- Vendor-specific dashboards (Stripe Dashboard, Twilio Console). The `workflow-studio` + `finops-portal` µservices cover most of this.
- Vendor-specific Stack Overflow ecosystem (you'll lean more on Oyatie docs).
- Some niche edge cases (Stripe Climate, Stripe Tax in 50+ jurisdictions, Twilio Frontline). These have roadmap status — check
  `microservices/payments/roadmap.md` and `microservices/comms-email/roadmap.md`.

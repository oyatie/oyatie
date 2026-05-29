---
doc_class: Tutorial
microservice: contact-center
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Tutorial — Build an IVR flow with PCI-DTMF suppression for card-payment-over-voice

Goal: build an IVR flow where customers call your billing line, identify themselves, then pay an outstanding invoice with their credit card via DTMF entry. The flow must (a) authenticate the caller, (b) suppress DTMF during card entry so the agent never hears the digits and the recording never captures them, (c) tokenise the card via the payment processor, (d) confirm payment to the caller.

Prereqs:

- `contact-center::admin` Cedar role.
- Your tenant on paid tenant_class or higher (PCI-DTMF suppression is not certified at demo_trial).
- A configured payment processor in the `cloud-billing-tax-app` µservice (Stripe / Adyen / Worldpay).
- A DID number provisioned for the billing line.
- ~ 2 hours.

## Step 1 — sketch the flow

Open the admin portal → Flows → "New flow" → "Blank canvas". Sketch the node sequence:

1. Welcome + recording consent.
2. Authenticate caller (account number + birth-year DTMF).
3. Read outstanding balance.
4. Ask if customer wants to pay now.
5. If yes → PCI suppression block → card-entry → tokenise → charge.
6. Confirm payment + offer follow-up.
7. End.

## Step 2 — Welcome + consent node

Drag a "Play prompt" node onto the canvas. Configure:

- Prompt: "Thank you for calling Acme Corp Billing. This call may be recorded for quality and compliance purposes. To opt out of recording, press star at any time. To continue, please stay on the line."
- Voice: select your tenant's default TTS voice (Coqui XTTS v2 for paid+; Mary TTS for paid).
- Star-key handler: connect to a "Set call tag" node that sets `recording.consent = false`.

## Step 3 — Authenticate caller

Drag a "Collect DTMF" node. Configure:

- Prompt: "Please enter your 10-digit account number followed by the pound key."
- Min digits: 10; max digits: 10; termination key: `#`; timeout: 15 s.
- Variable name: `account_number`.

Drag a second "Collect DTMF" node:

- Prompt: "Please enter your 4-digit birth year."
- Variable name: `birth_year`.

Drag an "HTTP call" node to verify the caller:

- URL: `https://api.<tenant>.oyatie.io/contact-center/internal/authenticate`.
- Method: POST.
- Headers: `Authorization: Bearer ${ivr_internal_token}` (substrate provides this; do not hardcode).
- Body: `{ "account_number": "${account_number}", "birth_year": "${birth_year}" }`.
- Timeout: 5 s.
- Response variable: `auth_result`.

Branch:

- If `auth_result.verified == true` → continue.
- Else → "Play prompt" → "Authentication failed. Please call back with your account information available or hold for an agent." → route to general queue.

## Step 4 — Read outstanding balance

Drag an "HTTP call" node:

- URL: `https://api.<tenant>.oyatie.io/cloud-billing-tax/customers/${auth_result.customer_id}/balance`.
- Method: GET.
- Response variable: `balance`.

Drag a "Play prompt with variable" node:

- Prompt: "Your current outstanding balance is ${balance.amount_display}. Press 1 to pay this amount now. Press 2 to speak to a billing agent. Press 9 to hear this menu again."

## Step 5 — PCI suppression block

This is the critical security boundary. Drag a "Begin PCI suppression" node. Configure:

- Suppression mode: `tone-mask` (replaces each DTMF tone with a non-pitched white-noise burst in both the agent's audio stream and the recording).
- Token store: `kms-pci-token-store` (default; refers to the per-pack KMS-encrypted token store).
- Suppression duration: until "End PCI suppression" node is reached.

Inside the suppression block, drag a "Collect card data" specialised node (only available inside a PCI suppression block):

- Field 1: card number (13-19 digits). Variable: `card_pan_token` (note: NOT the raw PAN — the substrate immediately tokenises).
- Field 2: expiry MM/YY (4 digits). Variable: `card_expiry_token`.
- Field 3: CVV (3-4 digits). Variable: `card_cvv_token`.
- Prompts: "Please enter your card number." / "Please enter your card expiry, month then year." / "Please enter your card security code."
- Per-field timeout: 30 s.

Drag an "HTTP call" node (still inside suppression block):

- URL: `https://api.<tenant>.oyatie.io/cloud-billing-tax/payments`.
- Method: POST.
- Body: `{ "customer_id": "${auth_result.customer_id}", "amount_cents": ${balance.amount_cents}, "card_pan_token": "${card_pan_token}", "card_expiry_token": "${card_expiry_token}", "card_cvv_token": "${card_cvv_token}" }`.
- Response variable: `payment_result`.

Drag an "End PCI suppression" node. After this node, normal media flow resumes; the agent (if any) hears audio again.

## Step 6 — Confirm payment

Drag a "Play prompt with variable" node (outside suppression block):

- If `payment_result.status == "succeeded"`: "Your payment of ${payment_result.amount_display} has been processed. A confirmation email will arrive within 5 minutes. Thank you for your business. Goodbye."
- Else: "Your payment could not be processed. Please try again or speak to a billing agent."

Branch:

- Success → "Hangup" node.
- Failure → "Route to queue" → `billing-agent-fallback`.

## Step 7 — Validate in simulator

Save the flow as a draft. Open the simulator: Flows → your flow → "Simulate".

Test cases:

| Case | Input | Expected outcome |
|---|---|---|
| Happy path | Valid account number + birth year + valid card | Reaches "Payment succeeded" prompt |
| Wrong auth | Wrong birth year | Reaches "Authentication failed" prompt |
| Star-opt-out | Press `*` during welcome | `recording.consent = false` tag set; flow continues |
| Card decline | Valid auth + declined card (use test card 4000 0000 0000 0002) | Reaches "Payment could not be processed" + routes to billing-agent-fallback |
| Timeout | No DTMF for 60 s during card entry | Flow exits suppression block, plays "We did not receive your input", routes to agent |

For each test, verify after the run:
- Recording (admin portal → Recordings → simulator-recordings → most recent): play the audio. During the card-entry block, you should hear white noise where DTMF would be. The agent-audio export should show the same.
- Audit log: `audit::contact_center::pci_suppression_started` and `audit::contact_center::pci_suppression_ended` events with the correct call_id and timestamps.

## Step 8 — Compliance evidence

Per PCI DSS 4.0 Req 3.2.1 + 12.8, you must retain evidence that the suppression operated correctly. Run:

```sh
cargo run -p oya-dev-cli -- contact-center pci-suppression-audit \
    --tenant <your-tenant-id> \
    --since-date 2026-05-13 \
    --until-date 2026-05-20
```

Output:

```
PCI suppression events: 412
  - Started: 412
  - Ended: 412
  - Started but not ended (orphan): 0
  - Recordings with DTMF artefacts in suppressed window: 0 (audited via DTMF-frequency-FFT scan)
  - Token-store writes during suppression: 412
  - Token-store reads (payment processor calls): 410 (2 calls were authentication failures, no payment attempted)
  - Average suppression duration: 47 s
  - Maximum suppression duration: 184 s
PCI DSS 4.0 Req 3.2.1 compliance: ✔
PCI DSS 4.0 Req 3.4.1 compliance: ✔
```

This report is queryable by your QSA (Qualified Security Assessor) for annual PCI DSS attestation.

## Step 9 — publish the flow

Once the simulator + audit run are green:

```sh
cargo run -p oya-dev-cli -- contact-center flow-publish \
    --tenant <your-tenant-id> \
    --flow billing-payment-with-pci \
    --version 1.0.0
```

The flow becomes live on your DID within 30 s. Existing in-flight calls finish on the old flow; new calls use the new flow.

## What you've built

A production-ready IVR with:
- Recording consent + opt-out.
- DTMF-based caller authentication.
- PCI DSS 4.0-compliant DTMF suppression during card entry.
- Tokenised card data (PAN never reaches the contact center substrate).
- Payment processor integration via the cloud-billing-tax-app µservice.
- Confirmation + agent fallback for declines.
- Auditable compliance evidence.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| Forgetting to wrap card entry in "Begin PCI suppression" / "End PCI suppression" | The simulator + audit run will flag this; the flow refuses to publish if a "Collect card data" node is outside a suppression block |
| Using a regular "Collect DTMF" node for the PAN | The publish step will fail; PAN MUST go through the specialised "Collect card data" node which auto-tokenises |
| Forgetting to tokenise expiry + CVV | These too go through the specialised node; raw expiry/CVV never reach the agent or recording |
| Suppression block too long (covers non-card prompts) | The audit report flags suppression > 5 min as a smell; trim the block to just the card entry |
| Calling the payment processor outside the suppression block | The token store rejects reads from outside the originating call's suppression window |

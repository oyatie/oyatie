---
doc_class: FAQ
microservice: contact-center
persona: contact-center-admin
related_adrs: [ADR-0316, ADR-0263, ADR-0251]
date: 2026-05-20
doc_status: published
---

# contact-center — Admin FAQ

## Q1: Can I record only specific calls based on consent, instead of recording all?

Yes. The recording policy is hierarchical: tenant default → queue override → per-call override. The IVR welcome prompt can include a consent question ("Press 1 to consent to recording; press 2 to opt out"). The flow sets a `recording.consent=false` tag on the call which the SBC honours by routing media to a non-recording path. Per GDPR Art. 6(1)(a), call recording without consent is unlawful in EU jurisdictions unless you have a different legal basis (contract necessity, legitimate interest with balancing test). Per CCPA, consumer opt-out is a right; the consent prompt + opt-out path is the practical compliance pattern.

## Q2: How does PCI-DTMF suppression work? Will the agent hear the card number?

When PCI suppression is enabled on a queue, the SBC routes media to a separate "secure entry" mode during the card-entry window. The customer hears the prompt ("please enter your 16-digit card number") and types DTMF; the SBC intercepts each tone, replaces it with a non-pitched white-noise burst in the agent's audio stream, and writes the actual digits to a pack-encrypted token store (via the `kms` µservice). The agent never hears the digits and the recording never captures them. The card number is delivered to the payment processor (Stripe / Adyen / Worldpay) via a separate secure-element API call — never via the voice path. This satisfies PCI DSS 4.0 Req 3.2.1 (no storage of PAN) + Req 3.4.1 (PAN unreadable wherever stored).

## Q3: A customer calls from a non-NENA-compliant country (e.g. Vietnam) and dials 911. What happens?

Per the FCC TRACED Act, oyatie's E911 routing is US-only by construction. For non-US callers, the contact center does NOT attempt to route emergency calls — instead, the IVR plays a country-specific emergency-services prompt ("if this is an emergency, please hang up and dial 119 (Korea) / 112 (EU) / 999 (UK) / 110 (Japan) directly from a mobile phone"). The substrate logs an `emergency_call_attempted` event to `audit-chain`. Your tenant cannot directly route emergency calls in non-US jurisdictions — you would need a local PSAP integration which is a per-country pack overlay.

## Q4: How do I integrate with my existing CRM (Salesforce / HubSpot / oyatie's own crm µservice)?

Three integration paths:

1. **Open-Cadence URL (CTI link)**: when a call arrives, the agent desktop opens a URL `https://your-crm.example/contact?phone=<caller_number>&call_id=<id>`. The CRM displays the matching contact record. Works with every CRM that supports URL-based screen-pop. Configure: portal → Integrations → "Open URL on call arrival".
2. **Webhook (POST per call event)**: events `call.started`, `call.answered`, `call.ended`, `call.recording.available` POST to your CRM endpoint. JSON payload includes call_id, caller_number, agent_id, queue_id, duration_seconds, recording_url. Configure: portal → Integrations → Webhooks.
3. **oyatie crm µservice native binding**: if you're on oyatie's `crm` µservice, the contact-center cross-emits to the `crm.contact.touch` event family directly; the contact record is updated automatically with the call as a touchpoint. No webhook configuration needed; permission via Cedar `crm::contact::touch`.

## Q5: My queue's abandonment rate spiked to 12 %. What does that mean and what do I do?

Abandonment rate = (callers who hung up before being connected to an agent) / (total inbound). A spike usually means: (a) understaffed queue (more inbound than agents can handle), (b) IVR too long (callers give up during the menu), or (c) outage in the routing path. Diagnostic:

- Check supervisor dashboard → queue → "queue depth over time". If queue depth is consistently > 5× agent count, you need more agents OR a callback-on-abandon flow.
- Check Reports → "AHT by skill". If AHT is climbing, agents are slow; consider AI-assist coaching or training.
- Check IVR analytics: portal → Flows → your flow → "Branch traversal stats". If > 30 % of callers abandon during the welcome prompt, it's too long; trim to ≤ 6 s.

For US outbound TCPA compliance, abandonment rate must stay ≤ 3 % over a 30-day window per 47 CFR § 64.1200(a)(7). If your tenant is on outbound predictive dialler at paid+, the dialler auto-throttles to stay under the 3 % threshold; you can't manually exceed it.

## Q6: Can I use my own SIP trunk provider instead of oyatie's?

At demo_trial and paid, no — the substrate's trunks are bundled in the billing_components contract. At paid, yes via BYOC (Bring Your Own Carrier): configure → Trunks → "Add external trunk" → provide the carrier's SIP URI, credentials, codec preferences. Common providers: Twilio Programmable Voice, Bandwidth.com, Inteliquent, Telnyx, Voxbone (Bandwidth subsidiary), 8x8. The SBC mediates between your trunk and the media-relay fleet. Note: at paid compliance-pack, the trunk MUST be in-pack (pack-resident SIP termination, e.g. KT 070 trunks for KR-PIPA pack).

## Q7: Does the AI-coaching at paid/paid compliance-pack read call content in real-time? Is that GDPR-compliant?

Yes, it reads call content via the GPU-backed ASR pipeline. GDPR compliance is achieved via: (a) recording consent (Q1) — the same consent covers AI processing; (b) data-minimisation per GDPR Art. 5(1)(c) — only the current call's transcript is held in the ASR pipeline; nothing is stored beyond the recording-retention window; (c) Art. 22 (automated decisions) — the AI coaching is *advisory* to the human agent; it never autonomously routes or escalates without the agent's action. Per ADR-0251 § D-3 the EU AI Act high-risk classification does NOT apply to AI-coaching (it's a productivity tool, not a high-risk decision system); however per Art. 12 logging obligation, all AI-coaching prompts are logged to `audit-chain` for the EU-AI-Act pack.

## Q8: A regulator asks for "all recordings of calls between agent X and customer Y over the last 6 months." How do I produce this?

Reports → "Regulator evidence export". Filter: agent_id = X, caller_number = Y, date_range. Click "Export with chain-of-custody". The substrate produces a ZIP containing: (a) WAV/Opus recording files; (b) call-metadata JSON; (c) audit-chain Merkle proofs (per ADR-0028) for each recording; (d) chain-of-custody log of every access (who downloaded the recording, when, why). The auditor can independently verify the Merkle proofs against the audit-chain public verification key (`oya audit-chain verify --bundle export.zip`) — no oyatie tooling required on their side. SLA: export completes within 24 h for ≤ 10 000 calls; longer for larger ranges.

## Q9: My pack is KR-PIPA. Can my outbound dialler call US numbers?

Cross-border outbound is gated by the pack policy. The KR-PIPA pack's outbound policy defaults to "Korea-only" (numbers starting with +82 or domestic 02/051/053/etc); cross-border requires a per-tenant exception via the `governance` µservice. Reason: KR 통신비밀보호법 + KR-PIPA Art. 17 (cross-border personal data transfer) require explicit consent + an adequacy assessment for the destination country. The exception flow involves filing a justification, the governance lane reviews it, the pack admin approves, and the dialler's allow-list is updated. Typical turnaround: 3-5 business days.

## Q10: How do I migrate from Genesys Cloud CX without an observability gap?

See `migration-playbooks/from-genesys.md`. Summary: (a) provision oyatie in parallel; (b) port DID numbers via FCC LNP (Local Number Portability) or KCC equivalent — typically 10-15 business days; (c) migrate IVR flows from Genesys Architect JSON to oyatie's flow JSON (we provide a converter for 80 % of node types; manual review for the rest); (d) re-provision agents (Genesys agent skills don't auto-translate; you must re-tag); (e) cut over by switching the DID's SIP routing from Genesys to oyatie's SBC. Run dual operation for 2 weeks with both systems live to handle any edge cases; finalise after no Genesys traffic for 7 consecutive days.

## Q11: What's the difference between MOS 4.0 (paid) and 4.2 (paid)? Is it audible?

Marginally. MOS (Mean Opinion Score) is a 1-5 listener-rating scale standardised in ITU-T P.800. MOS 4.0 = "perceptible distortion but not annoying"; MOS 4.2 = "slight distortion, generally acceptable". The audible difference is largely in burst packet loss handling — paid's anycast SRTP routing recovers from inter-AZ packet loss within 40 ms whereas paid's single-AZ relay recovers within 120 ms. For typical conversational speech most listeners cannot tell. For music-on-hold or IVR prompts with high-fidelity audio, paid is noticeably better. For executive boardroom-quality conferencing (CEO-to-CEO calls), paid compliance-pack's Opus 256 kbps wideband is recommended.

## Q12: Can I provision a Genesys-style "Mobile Agent" (an agent who works from their mobile phone, no headset)?

Yes — provision the agent with `agent.mobile_redirect = true` and a target mobile number. When a call routes to the agent, the SBC bridges the call to their mobile number via PSTN; the agent hears the customer and speaks via their mobile. Caveats: (a) media quality is bounded by the mobile carrier's codec (usually G.711 or AMR-NB; no Opus), (b) recording still works (the SBC records both legs), (c) AI-coaching is unavailable (the mobile leg has no real-time ASR/whisper integration), (d) mobile-roaming charges may apply to your tenant's PSTN bill, (e) E911 cannot resolve a precise location for the agent's mobile-bridged calls (the location is the mobile carrier's cell tower, not a NENA i3 dispatchable address).

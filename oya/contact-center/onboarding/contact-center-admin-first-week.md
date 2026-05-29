---
doc_class: Onboarding
microservice: contact-center
persona: contact-center-admin
related_adrs: [ADR-0316, ADR-0263, ADR-0251]
date: 2026-05-20
doc_status: published
---

# contact-center — Contact Center Admin First Week

Audience: a newly-hired contact-center administrator responsible for IVR flows, agent provisioning, queue routing, and recording-retention policy. You have admin-portal access but no engineering background.

## Day 1 — orientation + access

Morning (3 h):

1. Receive your `iam` invite. Cedar role `contact-center::admin` binds permissions: `contact_center::{flow,queue,agent,recording-policy}::{read,write}`, `contact_center::report::read`. Confirm via the admin portal at `https://contact-center-admin.<tenant>.oyatie.io`.
2. Click "Phone numbers" — confirm at least one DID (Direct Inward Dial) number is provisioned for your tenant. If none, file a ticket with the substrate team (DID provisioning requires carrier coordination + can take 24-48 h).
3. Click "Agents" — see the empty agent roster.
4. Click "Queues" — see the default `general-inbound` queue.

Afternoon (4 h):

5. Read the IVR-flow primer: portal → Help → "IVR Authoring 101". ~ 45 min.
6. Watch the recorded substrate walkthrough video (45 min) embedded in the portal under Help → Videos.
7. Provision a test agent (yourself or a buddy). Required: name, email, hire date, primary skill tag, secondary skill tags (optional), supervisor binding. Send the invite — the agent receives an email with a one-time password and WebRTC headset-setup instructions.
8. Run the WebRTC smoke test: portal → Diagnostics → "Test my browser + headset". This verifies the browser supports WebRTC, the microphone has 16 kHz input capability, and STUN servers are reachable.

End of Day 1 deliverable: 1 test agent provisioned + browser/headset smoke-test green.

## Day 2 — IVR flow authoring

Morning (4 h):

1. Open Flows → "New flow" → choose template "Simple inbound triage".
2. Edit the welcome prompt: "Thank you for calling Acme Corp. For sales, press 1. For support, press 2. For billing, press 3."
3. Configure DTMF branches:
   - Branch 1 (sales) → queue `sales-inbound`.
   - Branch 2 (support) → queue `support-inbound`.
   - Branch 3 (billing) → queue `billing-inbound`.
4. Add a fallback: if no DTMF input within 10 s, route to `general-inbound` queue.
5. Save as draft. The flow is NOT live until you publish it.

Afternoon (3 h):

6. Test the flow in the simulator: portal → Flows → your flow → "Simulate". Click DTMF buttons; verify each branch routes correctly.
7. Author the queues (Sales / Support / Billing) under Queues → "New queue". Required fields: name, skill tags required, max wait time, abandonment threshold, call-back-on-abandon enabled (recommend yes), recording mode (recommend "always-on" for compliance), AI-coaching enabled (paid tenant_class only).
8. Map agents to queues via Skills. E.g. agent Alice has `skill:sales-english`; queue `sales-inbound` requires `skill:sales-english`. Alice is now eligible to receive sales calls.

End of Day 2 deliverable: 1 IVR flow + 3 queues + 1 agent skill-bound + simulator-green.

## Day 3 — recording + privacy policy

Morning (3 h):

1. Read the recording-policy primer: portal → Help → "Recording, Retention, and Privacy". Understand: (a) recording is per-tenant default, but per-queue and per-call overrides exist; (b) retention is per-pack — KR-PIPA pack mandates 5 y, HIPAA-Provider mandates 7 y, no-pack default is 90 d; (c) PCI DSS Req 3.2.1 mandates suppression of card-data DTMF — enable it on any queue that handles card payments.
2. Configure tenant-default recording policy: portal → Policy → Recording. Choose: always-on / consent-gated / off. Recommend always-on with a consent prompt in the IVR welcome.
3. Add the consent prompt to your IVR: insert a node before the DTMF menu — "This call may be recorded for quality and compliance purposes. To opt out, press star at any time."

Afternoon (4 h):

4. Configure PCI-DTMF suppression on the billing queue: Queues → billing-inbound → Advanced → "Suppress DTMF during card-entry windows" (enable). Test via the simulator with a synthetic card-entry block.
5. Author the retention overlay for your pack: portal → Policy → Retention. The pack default is shown; you can override DOWN (shorter retention) only with legal sign-off via the `governance` µservice — never UP without escalation. Sign off as required.
6. Cross-check: portal → Audit Log → search "policy_change". Confirm your policy changes are logged with your principal-id, timestamp, before/after diff.

End of Day 3 deliverable: recording policy authored + consent prompt live + PCI suppression configured on billing queue + audit-log entries verified.

## Day 4 — go-live for first real agents

Morning (4 h):

1. Provision 5-10 real agents per Day-1 process. Send invites in batches.
2. For each agent: validate their browser/headset via the diagnostics tool. Common issues: Bluetooth headsets with low-quality codec (recommend USB or 2.4 GHz wired); browsers behind corporate firewalls blocking STUN (require IT to allow UDP 3478 + TURN 5349).
3. Run a sandbox call: have agent Alice call the tenant DID from her cell phone; verify the IVR plays, DTMF routes to the queue, Alice receives the call in her WebRTC desktop, recording is captured.

Afternoon (3 h):

4. Open the supervisor dashboard: portal → Supervise. Real-time view of agents (available / on-call / wrap-up / break), queue depth, current MOS scores, oldest waiting call.
5. Configure SLA alerts: queues → each queue → SLAs. Set service-level target (e.g. "80 % of calls answered within 30 s") + abandonment-rate target (e.g. "≤ 5 %"). Breach alerts route to the `notifications` µservice.
6. Publish the IVR flow: Flows → your flow → "Publish". The flow becomes live within 30 s.

End of Day 4 deliverable: first 5-10 agents live + supervisor dashboard configured + flow published.

## Day 5 — reporting + handoff

Morning (4 h):

1. Open Reports → Templates. Built-in reports:
   - Call volume by queue, day-of-week, hour-of-day.
   - Agent occupancy (talk time / available time).
   - Average handle time (AHT) by skill.
   - First-call-resolution (FCR) — measured via outbound follow-up survey or no-callback-within-7-d heuristic.
   - SLA compliance.
   - Abandonment rate.
2. Schedule a daily PDF email report to yourself + your team manager.
3. Schedule a weekly executive summary (volume, AHT, SLA, abandonment) to the VP.

Afternoon (4 h):

4. Document your tenant configuration: tenant-name-conventions, queue-naming, skill-naming, IVR-flow naming, agent-onboarding runbook. Save to your tenant's documentation drive.
5. Run a tabletop exercise with the substrate team: simulate a peak-volume incident (e.g. 5× normal call volume) and confirm the queue-overflow behaviour (route to overflow queue / play "please call back later" message / SMS callback link).
6. Receive substrate-team contact info for escalation. Note the substrate's on-call rotation, business hours, severity-class thresholds.

End of Week 1 deliverable: production live for first wave of agents, reports flowing, escalation paths documented.

## What you should know by end of week 1

- IVR-flow authoring + publishing.
- Queue + skill provisioning.
- Recording policy + consent prompts.
- PCI-DTMF suppression.
- Agent provisioning + WebRTC diagnostics.
- Supervisor dashboard + SLA alert configuration.
- Reporting templates + scheduled exports.

## What you should NOT do in week 1

- Don't disable recording globally without legal sign-off. Recording is the audit-trail substrate.
- Don't reduce retention below pack-mandated minima (KR-PIPA: 5 y; HIPAA-Provider: 7 y).
- Don't publish IVR-flow changes to production without simulator-green validation.
- Don't bind agents to queues whose skill requirements don't match the agent's actual skills — the queue will refuse to route and create silent backlog.
- Don't provision more concurrent agents than your tier supports (see tenant-class-behavior.md for caps).

---
doc_class: Onboarding
microservice: incident-management
persona: incident-commander
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# incident-management — Incident Commander First Week

Audience: a senior SRE/engineer designated as Incident Commander (IC) for your tenant. You'll lead high-severity incident response, author runbooks, and drive post-mortems.

## Day 1 — orientation + access + on-call shadowing

Morning (3 h):

1. Receive `iam` invite. Cedar role `incident::commander` binds: `incident::*` + `oncall::rotation::override` + `post-mortem::*::publish`.
2. Log in to the incident portal: `https://incident.<tenant>.oyatie.io`.
3. Verify: see the current on-call rotations, recent incidents (last 30 d), active escalation policies, paging-provider status (Twilio / Bandwidth / etc).
4. Configure your notification preferences: Slack, SMS, voice. The substrate's diagnostics tool tests each — click "Test all channels".

Afternoon (4 h):

5. Read substrate primer: portal → Help → "Incident Management 101" (~ 30 min).
6. Read NIST SP 800-61r2 "Computer Security Incident Handling Guide" Chapters 2-3 (~ 60 min).
7. Read the IC playbook: portal → Help → "Incident Commander Playbook" (~ 45 min).
8. Read your tenant's last 5 SEV-1 post-mortems. Understand the common failure classes.

End of Day 1 deliverable: notifications tested green; reading list complete.

## Day 2 — runbook authoring

Morning (4 h):

1. List the top-10 services your tenant operates. For each: identify the on-call team, escalation policy, and existing runbooks.
2. Identify gaps: services with no runbook for SEV-1, services with stale runbooks (older than 6 months), services without an explicit "first 15 minutes" checklist.
3. Pick one gap and author the runbook. Use the template at `microservices/incident-management/templates/runbook-template.md`.

Runbook structure:
- **Symptom**: what does the operator see that triggers this runbook?
- **First glance**: which dashboards to open immediately.
- **Top-5 likely causes + diagnostics**: for each, the commands to verify.
- **Mitigation steps**: minimal viable actions to reduce customer impact (e.g. restart a pod, fail over to standby, disable a feature flag).
- **Escalation**: who to page if mitigation doesn't work in 15 min.
- **Customer comms**: what (if anything) to communicate publicly via the status page.
- **Post-incident**: what evidence to capture for the post-mortem.

Afternoon (3 h):

4. Commit the runbook: `microservices/<service-ms>/runbooks/<incident-class>-breach.md`.
5. Link from the related OpenSLO manifest's `annotations.runbook` field.
6. Test: trigger a synthetic incident via `oya incident simulate-trigger --service <ms> --severity SEV-2`. Confirm: (a) the synthetic page arrives via your configured channels; (b) the runbook URL is in the alert payload; (c) the war-room channel auto-creates.

End of Day 2 deliverable: 1 runbook committed + simulated trigger green.

## Day 3 — on-call rotation + escalation policy

Morning (4 h):

1. List the on-call rotations for your services. Each rotation has: name, members (in rotation order), shift length (typically 1 week), handoff time, override window.
2. For your services, audit each rotation:
   - Is it follow-the-sun? (paid tenant_class feature)
   - Are there gaps (no one assigned for some hours)?
   - Is the handoff documented (Slack channel exchange, runbook handoff)?
3. If your tenant lacks 24/7 coverage, propose a follow-the-sun rotation: e.g. NA-EAST team 06:00-14:00 UTC; EU team 14:00-22:00 UTC; APAC team 22:00-06:00 UTC. Three time-zone-aligned teams provide round-the-clock coverage without overnight pages.

Afternoon (3 h):

4. Author or audit the escalation policy. Typical pattern:
   - Level 1: page the primary on-call. Wait 5 min for ack.
   - Level 2: page the secondary on-call. Wait 5 min.
   - Level 3: page the engineering manager.
   - Level 4: page the on-call IC (you).
   - Level 5: page the VP-Engineering. (Don't ladder past this in normal operations.)
5. For SEV-1 incidents, the policy should escalate FASTER (1-2 min between levels) because the customer impact is high.
6. Test the escalation policy: simulate a non-acked SEV-1; confirm escalation flows correctly through all levels.

End of Day 3 deliverable: on-call rotations audited + escalation policy validated end-to-end.

## Day 4 — war-room + status-page + customer comms

Morning (4 h):

1. Configure the war-room channel template: portal → Settings → "War-room defaults". For SEV-1/SEV-2: auto-create a Slack channel `#incident-<sev>-<short-id>`, invite the on-call, IC, EM, and any pre-defined stakeholders (CTO, Customer Success on-call).
2. Configure the status-page integration: portal → Integrations → "Status page". Choose: oyatie's `community` µservice public-status surface, or external (Statuspage by Atlassian, Instatus, BetterStack Status, FreshStatus). For each SEV class, configure default visibility (SEV-1 = public; SEV-2 = subscriber-only; SEV-3 = internal-only).
3. Configure customer-comms templates: pre-approved Slack snippets, email templates, status-page incident-update templates. Pre-approval matters — during an incident, you don't have time to debate copy.

Afternoon (3 h):

4. Run a tabletop exercise: simulate a SEV-1 with your team. Use a recent industry incident as the scenario (e.g. "Our CDN provider has region-wide failure. Customer traffic is failing 35 %.").
5. The tabletop should exercise:
   - Initial triage + IC assignment.
   - Internal Slack war-room setup.
   - Customer status-page first update (within 15 min of detection).
   - Mitigation steps + decision points (e.g. "do we fail over to secondary CDN?").
   - Customer follow-up updates (every 30 min until resolved).
   - Resolution + post-mortem scheduling.
6. Debrief: what worked, what didn't, what to improve.

End of Day 4 deliverable: war-room template + status-page integration + tabletop exercise complete.

## Day 5 — post-mortem authoring + sign-off

Morning (4 h):

1. Author a post-mortem for a recent real incident. Use the substrate template: portal → Post-Mortems → "New from incident" → select recent incident.
2. Template structure:
   - **Summary**: 2-3 sentences on what happened + impact.
   - **Timeline**: time-stamped sequence of events (detection → mitigation → resolution).
   - **Root cause**: the underlying cause (NOT the proximate cause; ask "why" 5 times).
   - **Contributing factors**: things that made the incident worse than it had to be.
   - **What went well**: detection speed, communication, mitigation effectiveness.
   - **What went poorly**: response delays, missed signals, incorrect assumptions.
   - **Action items**: specific, owned, dated. Each has a "type" (preventive / detective / corrective) and a target date.
   - **Customer impact**: number of affected customers, duration, financial estimate (cross-emitted from the customer-impact estimator at paid tenant_class tier).
3. Use the blameless-postmortem framework: focus on systems + processes, not individuals.

Afternoon (4 h):

4. Review the post-mortem with the team. Iterate on root-cause clarity + action item specificity.
5. Publish the post-mortem: portal → Post-Mortems → "Publish v1.0". Once published, the substrate emits `post-mortem::published` to audit-chain + notifies subscribers (typically the engineering org).
6. Track action items: portal → Action Items → filter by your team. Each action item has an owner + due date + tracking status. Action items overdue ≥ 30 d auto-escalate to the EM.

End of Week 1 deliverable: 1 published post-mortem + tracked action items + IC playbook understood.

## What you should know by end of week 1

- Incident lifecycle + state machine.
- Runbook structure + linking to OpenSLO.
- On-call rotation design (follow-the-sun, handoff, override).
- Escalation policy design + testing.
- War-room + status-page automation.
- Blameless-postmortem authoring.
- Action-item tracking.

## What you should NOT do in week 1

- Don't disable on-call notifications because you're "testing". The substrate enforces a minimum of 1 active on-call per service.
- Don't write a runbook that's longer than a single screen. First-glance dashboards + top-3 causes max. Anything longer is reading material, not a runbook.
- Don't merge a post-mortem with vague action items. Every action item is owned by a person + has a date + has a measurable completion criterion.
- Don't bypass escalation policies in production. They exist to ensure incidents get attention.
- Don't blame individuals in post-mortems. Blame systems + processes; people respond rationally to the incentives in front of them.

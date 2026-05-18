---
id: ADR-NOTES-0005
status: Accepted
date: 2026-05-17
microservice: notes
deciders: council-privacy, axis-notes, axis-foundry-runtime, ops-legal
owner: council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0008
  - ADR-0135
  - ADR-0131
  - ADR-NOTES-0001
  - ADR-NOTES-0004
related_artifacts:
  - microservices/notes/PRD.md (NFR §Security; FR-21; AC-03)
  - microservices/notes/policy/e2e-personal-tier-default.md
  - microservices/notes/capabilities/T0-suggest.yaml
  - microservices/notes/capabilities/T1-assist.yaml
  - microservices/notes/capabilities/T2-auto.yaml
  - microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md
purpose: Establish the structural invariant that AI assist (T0/T1/T2) is NEVER invoked over E2E-encrypted Personal-tier notes; bound Professional-tier AI assist to tenant-admin opt-in + EU AI Act + KR PIPA + GDPR transparency.
---

# ADR-NOTES-0005: AI assist (T0/T1/T2) over Personal-tier E2E notes is STRUCTURALLY IMPOSSIBLE; over Professional-tier requires tenant-admin opt-in + transparency + audit-chain seal

## Status

Accepted — 2026-05-17.

## Context

The notes µservice introduces an AI surface (T0 next-word / title suggest; T1 summarize / tag-suggest / link-suggest; T2 auto-organize). Without bounds, AI features create a **structural backdoor in the E2E posture**: a malicious or compromised AI provider (or a future PM who wires AI on Personal notes) would become an effective decryption oracle. This vector is unique to notes-µservice in the oyatie portfolio — the docs µservice (collaborative long-form) doesn't have this risk surface at minimum-shippable-tier, and messenger's AI features are restricted to Professional channels by ADR-MSGR-0003.

Regulatory drivers:

- **GDPR Art. 9** — special-category content (health / sexuality / religion / political opinion / biometric) often appears in personal notes; AI processing of these categories without explicit consent violates Art. 9.
- **GDPR Art. 22** — automated decision-making with significant effects requires human-in-loop; T2 auto-organize potentially triggers.
- **GDPR Art. 32** — appropriate technical measures; structural impossibility is stronger than runtime gating.
- **KR PIPA Art. 29** — security obligations; Art. 28 cross-border transfer; Art. 22-2 protection officer.
- **EU AI Act Art. 50** — transparency for limited-risk AI systems including content generation.
- **EU AI Act Art. 27** — conformity assessment for high-risk; notes T1/T2 are not high-risk but the transparency obligation applies.
- **NIST AI RMF 1.0** — trustworthy-AI characteristics.

Industry posture comparison:

- **Standard Notes** — no AI; ships nothing of this kind, claims privacy.
- **Mem / Reflect** — AI over note content; no E2E; users accept exposure.
- **Notion AI** — AI over Notion content; no E2E; tenant-admin opt-in.
- **Apple Intelligence / Notes** — on-device AI for Lockable notes; explicit user consent + on-device processing.

oyatie's commitment to E2E-default on Personal-tier (ADR-NOTES-0001) requires a sharper posture than the AI-over-everything incumbents and at least as strong as Apple's on-device-only-when-locked posture.

## Decision

oyatie notes adopts a **structural-impossibility posture for AI assist over Personal-tier E2E notes**, with bounded Professional-tier AI assist:

1. **Personal-tier (E2E) AI assist: STRUCTURALLY REFUSED.**
   - Type-system enforcement: `AssistInvoker::invoke(ProfessionalNoteRef) -> Result<AssistResult, AssistError>` — accepts only `ProfessionalNoteRef`; `PersonalNoteRef` cannot be passed (compiler refuses).
   - Cedar policy enforcement: `policy/ai-assist-scope.cedar` unconditional `forbid` on `Action::ai_call` over resources with `context_kind=Personal`.
   - CI-lane enforcement: `oya-check-e2e-ai-refusal` is a BLOCKER lane; greps for any path from `PersonalNoteRef` to `AssistInvoker::invoke` and BLOCKS on match (analogue of dual-context-isolation lane).
   - Runtime metric: `oya_notes_ai_call_blocked_e2e_total` increments on any forbidden attempt; PrometheusRule alarm at > 0 fires Sev-1.
   - Client-side-only AI: a Personal-tier user MAY run on-device AI in the client SDK if the SDK is built with the optional `on-device-ai` feature (Apple Intelligence on iOS/macOS, local LLM on desktop); this is not "AI assist over E2E content from the server's perspective" — the server is uninvolved.

2. **Professional-tier AI assist: tenant-admin opt-in + per-user-consent + transparency**:
   - Default: AI features OFF at tenant creation.
   - Tenant-admin opt-in via tenant-settings; explicit per-pack overlay (e.g., `pack-eu` requires EU AI Act Art. 50 transparency label; `pack-us-healthcare` requires explicit per-channel opt-in + signed BAA).
   - Per-user consent at first-use within tenant.
   - Capability tiers per `capabilities/`:
     - **T0 next-word / title suggest** (no autonomy; user-keystroke-driven) — minimum disclosure; included in standard product copy.
     - **T1 summarize / tag-suggest / link-suggest** (user-invoked; human-in-loop) — explicit per-tenant opt-in; transparency label "AI-generated" on output; `evidence_topic: oya.notes.capability.t1_assist.evidence` records every call for EU AI Act Art. 13 obligation.
     - **T2 auto-organize** (autonomous reorganisation suggestions) — disabled at minimum-shippable-tier; if enabled, requires explicit per-user opt-in + tenant-admin enable + human-in-loop "review changes before apply"; falls under GDPR Art. 22 caveat.
   - foundry-runtime provider must contract no-train clause via DPA; per-pack provider gating (e.g., pack-eu uses EU-regional providers only).
   - Every AI call writes audit-chain seal `AiAssistInvoked{request_id, capability_tier, model_version, input_hash, output_hash, principal_ref, tenant_ref}` (Bominal ADR-0028 Merkle + Ed25519).

3. **Cross-tier transitions forbidden**:
   - A Personal-tier user cannot "share into a Professional context to get AI"; the user-experience surfaces "create a new Professional note" as the explicit path.
   - A Professional-tier note that gets converted (only via "create new and copy" explicit path; not in-place) gets a fresh AI-eligibility evaluation.

4. **Evidence + transparency artifacts**:
   - Every AI invocation emits an `evidence_topic` record (per ADR-0139 foundry-evidence pattern).
   - Outputs labelled "AI-generated" in UI per EU AI Act Art. 50.
   - Tenant-admin sees AI-usage dashboard (calls, model, cost) per month.
   - User can opt-out at any time; opt-out is immediate.

5. **Model drift control**:
   - Eval set per capability (`microservices/notes/capabilities/eval/T1-*-golden.jsonl`) gates production via foundry-eval.
   - Canary rollout per ADR-0139 (1 % / 5 % / 20 % / 100 %).
   - Rollback runbook `runbooks/ai-classifier-rollback-e2e-respect.md`.

## Alternatives Considered

### A. No AI features at all (Standard-Notes-style)
- Pros: simplest privacy posture; zero risk.
- Cons: market parity gap with Notion AI / Mem / Reflect; oyatie loses competitive ground; PRD requires T1 features.
- Rejected: market viability cost too high.

### B. AI over both tiers with runtime opt-in
- Pros: simplest implementation; uniform code path.
- Cons: violates ADR-NOTES-0001 E2E posture; one bug = decryption oracle; one PM-decision = E2E posture broken; relies on policy-not-data-model invariant.
- Rejected.

### C. Structural impossibility on Personal-tier + opt-in on Professional-tier (this ADR's choice)
- Pros: E2E posture invariant by data model; market parity on Professional-tier; aligned with EU AI Act Art. 50; matches industry frontier (Apple Intelligence model).
- Accepted.

### D. Structural impossibility on Personal-tier + AI ON by default on Professional-tier
- Pros: simpler default for tenants who want AI.
- Cons: defaults must be conservative for regulatory + per-pack compliance; tenants must opt-in to consent on behalf of users; matches messenger ADR-MSGR-0003 posture.
- Rejected.

### E. On-device-only AI for both tiers (client SDK runs local LLM; no server AI)
- Pros: server is uninvolved across both tiers; cleanest privacy posture.
- Cons: local LLM at minimum-shippable-tier is impractical for the breadth of capabilities (summarize-of-long-note + tag-suggest with vocab-of-tenant); restricts to small models only; longer-term direction but not minimum-shippable-tier.
- Rejected at minimum-shippable-tier; tracked as direction.

### F. Confidential-compute / TEE on server (oyatie holds plaintext only inside enclave for AI calls)
- Pros: server-side processing with privacy preservation.
- Cons: TEE adds complexity + attack surface + ops cost; vulnerable to side-channel attacks; defeats the structural-impossibility claim (the enclave is still server-side); not battle-tested at this scale.
- Rejected at minimum-shippable-tier; tracked as future research.

## Consequences

### Positive

- E2E posture (ADR-NOTES-0001) preserved as structural invariant; AI surface cannot become a decryption oracle.
- EU AI Act Art. 50 transparency satisfied by output labelling + evidence-topic.
- KR PIPA Art. 29 + Art. 28 satisfied by per-pack gating + opt-in.
- HIPAA-eligibility on pack-us-healthcare not compromised (no AI on PHI without per-channel opt-in + BAA).
- CI lane + runtime metric give defence-in-depth.
- Tenant-admin governance dashboard satisfies the principle of "show, don't hide" — tenants see what AI is doing.

### Negative

- Personal-tier users wanting AI must mark notes Professional (UX must surface this clearly).
- Cost of two paths (T0/T1 only for Professional; nothing for Personal server-side).
- Local-LLM-on-device direction not minimum-shippable-tier; Personal-tier on-device AI requires SDK-feature-build customisation.
- Tenants migrating from Notion / Mem / Reflect lose AI on Personal notes by default; UX should communicate this clearly.

### Operational

- Crate `oya-notes-ai-assist-{kernel,domain,usecase,api,adapter,worker,sdk,app}` enumerated.
- Cedar policy `microservices/notes/policy/ai-assist-scope.cedar`.
- CI lane `oya-check-e2e-ai-refusal` registers BLOCKER on `dev`.
- PrometheusRule alarm on `oya_notes_ai_call_blocked_e2e_total > 0` for 5m.
- Runbook `runbooks/ai-classifier-rollback-e2e-respect.md`.
- Quarterly chaos-test: synthetic Personal+AI call attempt → must return 403 + emit metric + audit-chain.

### Regulatory

- **GDPR Art. 9** — Personal-tier E2E + AI-refusal mitigates special-category risk structurally.
- **GDPR Art. 22** — T2 auto-organize behind per-user opt-in + human-in-loop.
- **GDPR Art. 32** — structural impossibility is stronger than runtime gating.
- **KR PIPA Arts. 22-2, 28, 29** — per-pack overlay enforces.
- **EU AI Act Art. 50** — transparency satisfied.
- **NIST AI RMF 1.0** — trustworthy-AI characteristics: explainability + transparency + safety covered.

## References

- GDPR Arts. 9, 22, 25, 32.
- KR PIPA Arts. 22-2, 28, 29.
- HIPAA 45 CFR §164.502(b).
- EU AI Act Art. 50; Art. 27.
- NIST AI RMF 1.0.
- Apple Intelligence on-device-AI architecture (publicly documented).
- ADR-NOTES-0001 (E2E posture).
- ADR-NOTES-0004 (search architecture).
- ADR-MSGR-0003 (sibling messenger AI bounds; tier-shape pattern).
- ADR-0139 (SLO-gated promotion; foundry-eval).
- `microservices/notes/PRD.md` NFR Security + FR-21 + AC-03.
- `microservices/notes/policy/e2e-personal-tier-default.md`.
- `microservices/notes/capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`.
- `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`.

---
id: ADR-0318
title: "Adopt collar-color and workspace universality doctrine"
status: Superseded
date: 2026-05-20
owner_team: council-architecture
co_owners:
  - council-product
  - council-accessibility
  - council-privacy
  - council-security
  - axis-saas
  - axis-workflow
  - axis-ontology
  - axis-intelligence
  - axis-mobile
  - axis-edge
supersedes: []
superseded_by: [ADR-709]
related:
  - ADR-0215
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0292
  - ADR-0303
  - ADR-0306
  - ADR-0317
related_specs:
  - /specs/root-hub-pointers.json
  - /specs/masterplan.json
  - /specs/platform-architecture.json
  - /specs/microservices/workflow.json
  - /specs/microservices/ontology.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/doc-style.md
  - docs/templates/adr-template-v2.md
  - docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - docs/decisions/ADR-0306-disaster-mode-cell-resilience.md
doc_class: Architecture-Decision-Record
shape: Explanation
length_cap: 2600
authority_tier: 2
purpose: >
  Establish that Oyatie is one universal platform across collar color,
  workspace, device class, tenure, locale, and disability context. The
  doctrine binds every user-facing microservice to declare device-profile
  adapters, workspace adaptations, collar-color UX shells, role projections,
  and accessibility accommodations without forking the product or breaking
  the shared gesture vocabulary introduced by ADR-0317.
enforcement_status: advisory-until-ux-profile-registry-ships-blocker-thereafter
planned_enforcement_ref: oya-governance-workspace-universality
enforced_by:
  - cloud-ci/Rust gate packet ux-device-profile-matrix
  - cloud-ci/Rust gate packet ux-workspace-matrix
  - cloud-ci/Rust gate packet collar-color-shell-coverage
  - cloud-ci/Rust gate packet accessibility-accommodation-coverage
  - cloud-ci/Rust gate packet role-projection-transfer-invariant
naming_justifications:
  - name: oya-ux-universality-kernel
    justification: >
      Kernel traits for device profile, workspace profile, collar-color shell, and accommodation contract.
  - name: oya-ux-device-profile-registry
    justification: >
      Registry artifact that lets every microservice declare supported device classes.
  - name: oya-ux-collar-shell-library
    justification: >
      Shared shell library for collar-color variants without product forks.
  - name: UXProfileResolved
    justification: >
      Audit event emitted when a user surface resolves a profile bundle.
  - name: UXProfileFallbackActivated
    justification: >
      Audit event emitted when the requested device profile falls back safely.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Collar-color / workspace universality

# ADR-0318: Collar-Color and Workspace Universality Doctrine

> Status: Proposed
> Date: 2026-05-20
> Owner: council-architecture
> Binding theme: one platform, many projections, no workforce forks.

## A. Context
Oyatie is not a white-collar SaaS shell with optional field add-ons. It is a universal workspace
substrate that must operate across technical and non-technical workers, office and non-office
environments, every supported device class, and every collar-color segment. The same platform must
serve a nurse at a patient bedside, a warehouse associate scanning a tote, a driver reading a route,
a construction supervisor checking a permit, a farmer logging field evidence, a pilot using a
cockpit checklist, and a finance analyst reviewing an approval queue.
The ecosystem-universality thesis is: one canonical product graph, one tenant model, one policy
engine, one audit chain, one workflow substrate, and one role-projection vocabulary. The UX changes
by projection, not by product fork. A gesture learned in one context remains meaningful in another
context: approve, assign, scan, acknowledge, escalate, comment, sign, route, defer, attach evidence,
and review history keep their semantic identity across laptop, phone, watch, voice, vehicle, and AR
overlay.
ADR-0317 is named by the brief as the authoritative in-flight role-based projection and unified UX
shell. At author time the specified ADR-0317 file was absent from the live repository, so this ADR
cites ADR-0317 by identifier and doctrine while avoiding a broken Markdown link. When ADR-0317
lands, this ADR MUST be patched to replace the in-flight citation with the concrete path and inbound
citation row.
ADR-0292 binds minor-user protection to every B2C surface. That matters here because universality
cannot flatten children, guardians, care workers, students, apprentices, or trainees into a generic
adult office user. Device, collar-color, locale, and tenure profiles MUST carry age and consent
overlays when a surface is consumer-facing or B2B2C.
ADR-0306 binds disaster-mode and offline-first cell resilience. That matters here because field,
warehouse, vehicle, cockpit, outdoors, bedside, and shop-floor workflows routinely operate under
intermittent connectivity. A universal platform that requires uninterrupted network access is an
office-only platform by another name.
Industry precedent converges on profile-based adaptation. Apple uses required device capability
metadata to keep incompatible apps off unsupported hardware. Google Play filters apps by manifest
features and device compatibility. Walmart deploys associate-specific mobile workflows and devices.
UPS embeds route optimization into driver handhelds. Kaiser Permanente unifies care and coverage
workflows in mobile check-in. Amazon Mechanical Turk gates work with worker qualifications. Lyft
presents driver-specific earnings and busy-area tools. RealWear centers voice-driven smartglasses
for frontline work. These are not identical domains, but they share one pattern: the workflow
surface adapts to device, worker state, environment, and eligibility.

### A-1. Universal audience roster
A-1.01. Audience: technical engineer debugging an incident.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.02. Audience: non-technical store associate picking an order.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.03. Audience: office finance analyst reviewing a risk report.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.04. Audience: field technician repairing equipment.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.05. Audience: shop-floor operator responding to an alarm.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.06. Audience: vehicle driver following route and proof-of-delivery flow.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.07. Audience: patient-bedside nurse performing a handoff.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.08. Audience: construction foreman checking permit and safety state.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.09. Audience: kitchen worker validating allergen and batch temperature.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.10. Audience: warehouse picker scanning, staging, and handing off goods.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.11. Audience: home caregiver coordinating family support.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.12. Audience: outdoor worker under sunlight, gloves, and weak connectivity.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.13. Audience: cockpit operator following a mission checklist.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.
A-1.14. Audience: OR worker using sterile voice-only confirmation.
A-1 evidence: the product identity remains the same; only projection density, input mode, and risk gates change.

### A-2. Problem with workforce-specific products
A-2.01. Anti-pattern: A white-collar-only shell hides field exceptions until implementation time.
A-2 decision pressure: universality MUST be an invariant in the product graph, not a later UX retrofit.
A-2.02. Anti-pattern: A blue-collar-only mobile app loses policy and audit semantics from the canonical workspace.
A-2 decision pressure: universality MUST be an invariant in the product graph, not a later UX retrofit.
A-2.03. Anti-pattern: A care-specific app often hard-codes clinical vocabulary without reusable consent or locale overlays.
A-2 decision pressure: universality MUST be an invariant in the product graph, not a later UX retrofit.
A-2.04. Anti-pattern: A driver-specific app tends to fork route evidence and proof capture away from the shared audit chain.
A-2 decision pressure: universality MUST be an invariant in the product graph, not a later UX retrofit.
A-2.05. Anti-pattern: A wearable-only app often degrades to notifications without authority to complete work safely.
A-2 decision pressure: universality MUST be an invariant in the product graph, not a later UX retrofit.
A-2.06. Anti-pattern: A voice-only assistant can become a separate product unless it shares the command vocabulary.
A-2 decision pressure: universality MUST be an invariant in the product graph, not a later UX retrofit.
A-2.07. Anti-pattern: A locale pack can become a translation veneer unless it binds legal, script, and workflow differences.
A-2 decision pressure: universality MUST be an invariant in the product graph, not a later UX retrofit.
A-2.08. Anti-pattern: An accessibility mode can become a degraded mode unless it carries the same capabilities and evidence.
A-2 decision pressure: universality MUST be an invariant in the product graph, not a later UX retrofit.

## B. Decision
Oyatie adopts the Collar-Color and Workspace Universality Doctrine. Every user-facing microservice
MUST declare how its workflows project across four primary axes: skill, workspace, tenure, and
locale. Every such projection MUST also declare device profile support and disability
accommodations. Projections can hide, reorder, summarize, or stage information; they MUST NOT change
authorization, audit semantics, workflow identity, data class, tenant scope, or legal obligation.

### B-1. Universality invariants across four axes
B-1 axis: skill.
B-1 invariant: novice through expert; technical and non-technical workers share the same action vocabulary.
B-1 rule: microservices MUST expose capability semantics before UX shells choose density or input mode.
B-1 anti-pattern: duplicating a workflow because a workforce segment has different jargon.
B-1 acceptance: the same audit event class is emitted from every projection of the same action.
B-1 axis: workspace.
B-1 invariant: office, home, field, shop floor, vehicle, bedside, site, kitchen, warehouse, cockpit, OR, and outdoors share the same workflow identity.
B-1 rule: microservices MUST expose capability semantics before UX shells choose density or input mode.
B-1 anti-pattern: duplicating a workflow because a workforce segment has different jargon.
B-1 acceptance: the same audit event class is emitted from every projection of the same action.
B-1 axis: tenure.
B-1 invariant: day-zero onboarding and 30-year veteran expert mode remain projections of one semantic model.
B-1 rule: microservices MUST expose capability semantics before UX shells choose density or input mode.
B-1 anti-pattern: duplicating a workflow because a workforce segment has different jargon.
B-1 acceptance: the same audit event class is emitted from every projection of the same action.
B-1 axis: locale.
B-1 invariant: language, script, jurisdiction, currency, units, labor rules, and consent overlays are pack overlays, not product forks.
B-1 rule: microservices MUST expose capability semantics before UX shells choose density or input mode.
B-1 anti-pattern: duplicating a workflow because a workforce segment has different jargon.
B-1 acceptance: the same audit event class is emitted from every projection of the same action.

### B-2. Decision statements
B-2.01. Decision: Every user-facing microservice MUST own a device-profile matrix.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.02. Decision: Every user-facing microservice MUST own a workspace-profile matrix.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.03. Decision: Every workflow action MUST keep one canonical gesture/action vocabulary across projections.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.04. Decision: Every collar-color shell MUST be a shared library projection, not a product fork.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.05. Decision: Every supported device profile MUST declare input, output, offline, accessibility, and privacy constraints.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.06. Decision: Voice-first surfaces MUST be intelligence-mediated but policy-bounded and auditable.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.07. Decision: Offline-first surfaces MUST use ADR-0306 CRDT-backed sync or explicitly declare no offline action support.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.08. Decision: Wearable-first surfaces MUST be glanceable, bounded, and escalation-aware.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.09. Decision: Locale-aware surfaces MUST bind language, script, jurisdiction, and legal overlays together.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.10. Decision: Disability accommodations MUST be capability-equivalent unless law or safety forbids completion.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.11. Decision: Minor-affecting surfaces MUST compose ADR-0292 before projecting simplified or guardian-mediated UX.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.
B-2.12. Decision: ADR-0317 unified shell vocabulary is the source for transferable gestures across roles and devices.
B-2 evidence: the decision is verifiable by profile registry rows and audit-event parity tests.
B-2 rejection: shipping an unregistered ad hoc surface violates the doctrine.

## C. Consequences
### C-1. Maintainability
C-1.1. Consequence: shared projection contracts reduce per-workforce forks and concentrate changes in profile adapters.
C-1.2. Positive: future teams add one profile row and one adapter instead of forking a product.
C-1.3. Negative: the first registry and test matrix are larger than a single web shell.
C-1.4. Operational impact: on-call can filter incidents by `ux_profile_id`, `workspace_profile_id`, and `collar_shell_id`.
C-1.5. Quality bar: every profile-sensitive incident gets reproduction fixtures for at least two device classes.
C-1.6. Rollback: disable the new profile row and route to the nearest declared fallback shell.
C-1.7. Multi-region: profile configuration is tenant-scoped and replicated through cell config channels.
C-1.8. Sovereign-cell: locale packs constrain fields, retention, and available actions before rendering.
C-1.9. Versioning: profile schema follows SemVer and deprecates profile fields with ADR-0258 cadence.
C-1.10. Anti-pattern: hard-coding worker type into REST handlers instead of resolving projection context.

### C-2. Observability
C-2.1. Consequence: profile resolution emits metrics, traces, logs, and audit events for every projected action.
C-2.2. Positive: future teams add one profile row and one adapter instead of forking a product.
C-2.3. Negative: the first registry and test matrix are larger than a single web shell.
C-2.4. Operational impact: on-call can filter incidents by `ux_profile_id`, `workspace_profile_id`, and `collar_shell_id`.
C-2.5. Quality bar: every profile-sensitive incident gets reproduction fixtures for at least two device classes.
C-2.6. Rollback: disable the new profile row and route to the nearest declared fallback shell.
C-2.7. Multi-region: profile configuration is tenant-scoped and replicated through cell config channels.
C-2.8. Sovereign-cell: locale packs constrain fields, retention, and available actions before rendering.
C-2.9. Versioning: profile schema follows SemVer and deprecates profile fields with ADR-0258 cadence.
C-2.10. Anti-pattern: hard-coding worker type into REST handlers instead of resolving projection context.

### C-3. Scalability
C-3.1. Consequence: device and workspace variants scale horizontally as stateless projection shells over shared services.
C-3.2. Positive: future teams add one profile row and one adapter instead of forking a product.
C-3.3. Negative: the first registry and test matrix are larger than a single web shell.
C-3.4. Operational impact: on-call can filter incidents by `ux_profile_id`, `workspace_profile_id`, and `collar_shell_id`.
C-3.5. Quality bar: every profile-sensitive incident gets reproduction fixtures for at least two device classes.
C-3.6. Rollback: disable the new profile row and route to the nearest declared fallback shell.
C-3.7. Multi-region: profile configuration is tenant-scoped and replicated through cell config channels.
C-3.8. Sovereign-cell: locale packs constrain fields, retention, and available actions before rendering.
C-3.9. Versioning: profile schema follows SemVer and deprecates profile fields with ADR-0258 cadence.
C-3.10. Anti-pattern: hard-coding worker type into REST handlers instead of resolving projection context.

### C-4. Performance
C-4.1. Consequence: each profile carries latency and interaction budgets appropriate to its environment.
C-4.2. Positive: future teams add one profile row and one adapter instead of forking a product.
C-4.3. Negative: the first registry and test matrix are larger than a single web shell.
C-4.4. Operational impact: on-call can filter incidents by `ux_profile_id`, `workspace_profile_id`, and `collar_shell_id`.
C-4.5. Quality bar: every profile-sensitive incident gets reproduction fixtures for at least two device classes.
C-4.6. Rollback: disable the new profile row and route to the nearest declared fallback shell.
C-4.7. Multi-region: profile configuration is tenant-scoped and replicated through cell config channels.
C-4.8. Sovereign-cell: locale packs constrain fields, retention, and available actions before rendering.
C-4.9. Versioning: profile schema follows SemVer and deprecates profile fields with ADR-0258 cadence.
C-4.10. Anti-pattern: hard-coding worker type into REST handlers instead of resolving projection context.

### C-5. Optimization
C-5.1. Consequence: dense shells, voice shells, wearable shells, and offline shells optimize for different cost frontiers.
C-5.2. Positive: future teams add one profile row and one adapter instead of forking a product.
C-5.3. Negative: the first registry and test matrix are larger than a single web shell.
C-5.4. Operational impact: on-call can filter incidents by `ux_profile_id`, `workspace_profile_id`, and `collar_shell_id`.
C-5.5. Quality bar: every profile-sensitive incident gets reproduction fixtures for at least two device classes.
C-5.6. Rollback: disable the new profile row and route to the nearest declared fallback shell.
C-5.7. Multi-region: profile configuration is tenant-scoped and replicated through cell config channels.
C-5.8. Sovereign-cell: locale packs constrain fields, retention, and available actions before rendering.
C-5.9. Versioning: profile schema follows SemVer and deprecates profile fields with ADR-0258 cadence.
C-5.10. Anti-pattern: hard-coding worker type into REST handlers instead of resolving projection context.

### C-6. Code quality
C-6.1. Consequence: profile adapters get contract tests, fixture matrices, accessibility checks, and semantic parity tests.
C-6.2. Positive: future teams add one profile row and one adapter instead of forking a product.
C-6.3. Negative: the first registry and test matrix are larger than a single web shell.
C-6.4. Operational impact: on-call can filter incidents by `ux_profile_id`, `workspace_profile_id`, and `collar_shell_id`.
C-6.5. Quality bar: every profile-sensitive incident gets reproduction fixtures for at least two device classes.
C-6.6. Rollback: disable the new profile row and route to the nearest declared fallback shell.
C-6.7. Multi-region: profile configuration is tenant-scoped and replicated through cell config channels.
C-6.8. Sovereign-cell: locale packs constrain fields, retention, and available actions before rendering.
C-6.9. Versioning: profile schema follows SemVer and deprecates profile fields with ADR-0258 cadence.
C-6.10. Anti-pattern: hard-coding worker type into REST handlers instead of resolving projection context.

## D. Detailed Mechanics

### D-1. Per-device-profile UX surface
D-1.01. Device profile: laptop.
D-1.01. Examples: MacBook Pro, ThinkPad, Dell Latitude, Chromebook Plus.
D-1.01. Surface: dense keyboard-first shell with command palette, tabs, side nav, audit overlays.
D-1.01. Input model: keyboard, trackpad, pointer, microphone, camera.
D-1.01. Offline model: local encrypted queue plus CRDT merge window per ADR-0306.
D-1.01. Adapter one: ux_device_profile_laptop.
D-1.01. Adapter two: projection_shell_desktop_dense.
D-1.01. Precedent A: Apple Required Device Capabilities.
D-1.01. Precedent B: Google Play filters.
D-1.01. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.01. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.01. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.01. Audit event: UXProfileResolved includes `device_profile=laptop`.
D-1.01. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.01. Anti-pattern: treating laptop as a CSS breakpoint only.
D-1.01. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.01. Migration: every user-facing microservice declares supported=laptop or explicit not-supported reason.

D-1.02. Device profile: desktop.
D-1.02. Examples: Windows workstation, macOS Studio, Linux operator console.
D-1.02. Surface: multi-window, large-monitor, always-on command center with durable panels.
D-1.02. Input model: keyboard, mouse, accessibility switch, dictation.
D-1.02. Offline model: background sync with conflict inspector and region-cell replay.
D-1.02. Adapter one: ux_device_profile_desktop.
D-1.02. Adapter two: projection_shell_command_center.
D-1.02. Precedent A: Apple Required Device Capabilities.
D-1.02. Precedent B: Google Play filters.
D-1.02. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.02. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.02. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.02. Audit event: UXProfileResolved includes `device_profile=desktop`.
D-1.02. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.02. Anti-pattern: treating desktop as a CSS breakpoint only.
D-1.02. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.02. Migration: every user-facing microservice declares supported=desktop or explicit not-supported reason.

D-1.03. Device profile: tablet.
D-1.03. Examples: iPad Pro, Surface Pro, Galaxy Tab Active, bedside clinical tablet.
D-1.03. Surface: touch-first split view with pen, scan, signature, and bedside handoff affordances.
D-1.03. Input model: touch, pen, camera scan, NFC, microphone.
D-1.03. Offline model: case-local CRDT bundle with deferred evidence seal.
D-1.03. Adapter one: ux_device_profile_tablet.
D-1.03. Adapter two: projection_shell_touch_split.
D-1.03. Precedent A: Apple Required Device Capabilities.
D-1.03. Precedent B: Kaiser Permanente mobile app experience.
D-1.03. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.03. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.03. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.03. Audit event: UXProfileResolved includes `device_profile=tablet`.
D-1.03. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.03. Anti-pattern: treating tablet as a CSS breakpoint only.
D-1.03. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.03. Migration: every user-facing microservice declares supported=tablet or explicit not-supported reason.

D-1.04. Device profile: phone.
D-1.04. Examples: iPhone, Android phone, Walmart Samsung Galaxy XCover Pro.
D-1.04. Surface: one-hand task cards, scan, approve, notify, route, and quick capture.
D-1.04. Input model: touch, camera, NFC, GPS, voice, haptics.
D-1.04. Offline model: store-and-forward task queue with per-field merge hints.
D-1.04. Adapter one: ux_device_profile_phone.
D-1.04. Adapter two: projection_shell_mobile_action_card.
D-1.04. Precedent A: Walmart Me@Walmart associate app.
D-1.04. Precedent B: Kaiser Permanente mobile app experience.
D-1.04. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.04. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.04. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.04. Audit event: UXProfileResolved includes `device_profile=phone`.
D-1.04. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.04. Anti-pattern: treating phone as a CSS breakpoint only.
D-1.04. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.04. Migration: every user-facing microservice declares supported=phone or explicit not-supported reason.

D-1.05. Device profile: wearable.
D-1.05. Examples: Apple Watch, Galaxy Watch, Fitbit tracker, Pixel Watch.
D-1.05. Surface: glanceable state, critical notification, simple acknowledge, timer, safety ping.
D-1.05. Input model: tap, crown or bezel, haptic, short voice, health sensor availability.
D-1.05. Offline model: tiny bounded queue synchronized through paired phone or direct cell path.
D-1.05. Adapter one: ux_device_profile_wearable.
D-1.05. Adapter two: projection_shell_glance.
D-1.05. Precedent A: Apple watchOS design guidance.
D-1.05. Precedent B: Samsung Galaxy Watch sensors.
D-1.05. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.05. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.05. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.05. Audit event: UXProfileResolved includes `device_profile=wearable`.
D-1.05. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.05. Anti-pattern: treating wearable as a CSS breakpoint only.
D-1.05. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.05. Migration: every user-facing microservice declares supported=wearable or explicit not-supported reason.

D-1.06. Device profile: ar-overlay.
D-1.06. Examples: Apple Vision Pro, Microsoft HoloLens, RealWear Arc class.
D-1.06. Surface: heads-up contextual overlay anchored to object, procedure, or route.
D-1.06. Input model: gaze, hand, voice, external scanner, remote expert annotation.
D-1.06. Offline model: procedure bundle cached with signed step evidence and later CRDT merge.
D-1.06. Adapter one: ux_device_profile_ar_overlay.
D-1.06. Adapter two: projection_shell_spatial_overlay.
D-1.06. Precedent A: RealWear industrial smartglasses.
D-1.06. Precedent B: Apple Required Device Capabilities.
D-1.06. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.06. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.06. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.06. Audit event: UXProfileResolved includes `device_profile=ar-overlay`.
D-1.06. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.06. Anti-pattern: treating ar-overlay as a CSS breakpoint only.
D-1.06. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.06. Migration: every user-facing microservice declares supported=ar-overlay or explicit not-supported reason.

D-1.07. Device profile: voice-only.
D-1.07. Examples: phone call, headset, vehicle voice, smartglasses, low-vision mode.
D-1.07. Surface: dialogue state machine with readback, confirmation, interrupt, and escalation.
D-1.07. Input model: speech, DTMF, wake phrase, noise-aware push-to-talk.
D-1.07. Offline model: local command buffer with explicit readback before replay.
D-1.07. Adapter one: ux_device_profile_voice_only.
D-1.07. Adapter two: projection_shell_dialogue.
D-1.07. Precedent A: RealWear speech recognizer.
D-1.07. Precedent B: UPS UPSNav / ORION driver device.
D-1.07. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.07. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.07. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.07. Audit event: UXProfileResolved includes `device_profile=voice-only`.
D-1.07. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.07. Anti-pattern: treating voice-only as a CSS breakpoint only.
D-1.07. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.07. Migration: every user-facing microservice declares supported=voice-only or explicit not-supported reason.

D-1.08. Device profile: ruggedized-handheld.
D-1.08. Examples: Zebra TC series, Galaxy XCover, Honeywell scanner.
D-1.08. Surface: scan-first workflow with giant hit targets and glove-safe recovery.
D-1.08. Input model: barcode, RFID, camera, hardware trigger, touch, haptic.
D-1.08. Offline model: warehouse-cell queue with deterministic scan dedupe.
D-1.08. Adapter one: ux_device_profile_ruggedized_handheld.
D-1.08. Adapter two: projection_shell_scan_first.
D-1.08. Precedent A: Walmart Store Assist workflows.
D-1.08. Precedent B: Google Play filters.
D-1.08. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.08. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.08. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.08. Audit event: UXProfileResolved includes `device_profile=ruggedized-handheld`.
D-1.08. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.08. Anti-pattern: treating ruggedized-handheld as a CSS breakpoint only.
D-1.08. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.08. Migration: every user-facing microservice declares supported=ruggedized-handheld or explicit not-supported reason.

D-1.09. Device profile: in-vehicle.
D-1.09. Examples: delivery van console, fleet tablet mount, service truck unit.
D-1.09. Surface: route, stop, checklist, handoff, and safety-first glance surface.
D-1.09. Input model: voice, steering-safe controls, GPS, barcode, camera proof.
D-1.09. Offline model: route packet cached before departure with partial connectivity merge.
D-1.09. Adapter one: ux_device_profile_in_vehicle.
D-1.09. Adapter two: projection_shell_route_console.
D-1.09. Precedent A: UPS UPSNav / ORION driver device.
D-1.09. Precedent B: Lyft driver earnings tools.
D-1.09. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.09. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.09. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.09. Audit event: UXProfileResolved includes `device_profile=in-vehicle`.
D-1.09. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.09. Anti-pattern: treating in-vehicle as a CSS breakpoint only.
D-1.09. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.09. Migration: every user-facing microservice declares supported=in-vehicle or explicit not-supported reason.

D-1.10. Device profile: in-cockpit.
D-1.10. Examples: electronic flight bag, rail cab tablet, marine bridge tablet.
D-1.10. Surface: checklist, incident, route, and compliance overlay with no distraction debt.
D-1.10. Input model: touch, hardware knob, voice readback, offline chart package.
D-1.10. Offline model: mission packet preflight with append-only evidence after landing.
D-1.10. Adapter one: ux_device_profile_in_cockpit.
D-1.10. Adapter two: projection_shell_mission_checklist.
D-1.10. Precedent A: Apple Required Device Capabilities.
D-1.10. Precedent B: Google Play filters.
D-1.10. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.10. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.10. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.10. Audit event: UXProfileResolved includes `device_profile=in-cockpit`.
D-1.10. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.10. Anti-pattern: treating in-cockpit as a CSS breakpoint only.
D-1.10. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.10. Migration: every user-facing microservice declares supported=in-cockpit or explicit not-supported reason.

D-1.11. Device profile: large-font-accessibility.
D-1.11. Examples: 200 percent text, screen reader, single switch, AAC board.
D-1.11. Surface: same workflow semantics with enlarged text, serialized focus, and reduced load.
D-1.11. Input model: screen reader, switch, voice, AAC, keyboard, high contrast pointer.
D-1.11. Offline model: accessibility preferences cached as tenant/user policy overlay.
D-1.11. Adapter one: ux_device_profile_accessibility_large_font.
D-1.11. Adapter two: projection_shell_serial_focus.
D-1.11. Precedent A: Apple Accessibility HIG.
D-1.11. Precedent B: Kaiser Permanente mobile app experience.
D-1.11. Capability gate: profile must declare required sensors, screen size, network, and input fallback.
D-1.11. Accessibility gate: large text, screen reader, switch, and voice alternatives are explicit fields.
D-1.11. Privacy gate: camera, microphone, location, and health data are purpose-bound before use.
D-1.11. Audit event: UXProfileResolved includes `device_profile=large-font-accessibility`.
D-1.11. Fallback event: UXProfileFallbackActivated records source and fallback profile.
D-1.11. Anti-pattern: treating large-font-accessibility as a CSS breakpoint only.
D-1.11. Test: golden projection fixture renders the same action ids as laptop baseline.
D-1.11. Migration: every user-facing microservice declares supported=large-font-accessibility or explicit not-supported reason.

### D-2. Per-workspace UX adaptation
D-2.01. Workspace profile: office.
D-2.01. Physical context: desk, conference room, shared monitor.
D-2.01. UX intent: focus plus collaboration with full keyboard.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=office`.
D-2 anti-pattern: cloning a separate office app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.02. Workspace profile: work-from-home.
D-2.02. Physical context: home office, shared household, variable bandwidth.
D-2.02. UX intent: privacy-preserving notifications and async work.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=work-from-home`.
D-2 anti-pattern: cloning a separate work-from-home app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.03. Workspace profile: field.
D-2.03. Physical context: utility pole, farm row, inspection route.
D-2.03. UX intent: offline inspection, photo evidence, safety checklist.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=field`.
D-2 anti-pattern: cloning a separate field app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.04. Workspace profile: shop-floor.
D-2.04. Physical context: factory cell, CNC station, maintenance bay.
D-2.04. UX intent: scan, alarm, lockout, and procedure mode.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=shop-floor`.
D-2 anti-pattern: cloning a separate shop-floor app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.05. Workspace profile: vehicle.
D-2.05. Physical context: delivery van, service truck, rideshare car.
D-2.05. UX intent: route, stop, proof, and hands-free status.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=vehicle`.
D-2 anti-pattern: cloning a separate vehicle app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.06. Workspace profile: patient-bedside.
D-2.06. Physical context: hospital room, clinic room, home care visit.
D-2.06. UX intent: clinical handoff and low-error confirmation.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=patient-bedside`.
D-2 anti-pattern: cloning a separate patient-bedside app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.07. Workspace profile: construction-site.
D-2.07. Physical context: jobsite, scaffold, trailer, outdoors.
D-2.07. UX intent: rugged checklist, hazard, permit, and crew sync.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=construction-site`.
D-2 anti-pattern: cloning a separate construction-site app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.08. Workspace profile: kitchen.
D-2.08. Physical context: restaurant line, commissary, school kitchen.
D-2.08. UX intent: glove-safe timer, allergen, temp, and batch check.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=kitchen`.
D-2 anti-pattern: cloning a separate kitchen app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.09. Workspace profile: warehouse.
D-2.09. Physical context: aisle, dock, cage, freezer.
D-2.09. UX intent: scan-first pick, stage, pack, and exception capture.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=warehouse`.
D-2 anti-pattern: cloning a separate warehouse app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.10. Workspace profile: on-the-road.
D-2.10. Physical context: courier, sales, home health, rail.
D-2.10. UX intent: mobile route and brief action cards.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=on-the-road`.
D-2 anti-pattern: cloning a separate on-the-road app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.11. Workspace profile: home.
D-2.11. Physical context: consumer account, caregiver, family workspace.
D-2.11. UX intent: low-jargon personal workflows.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=home`.
D-2 anti-pattern: cloning a separate home app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.12. Workspace profile: outdoors.
D-2.12. Physical context: farm, emergency response, site survey.
D-2.12. UX intent: sunlight, offline, GPS, and protective gear.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=outdoors`.
D-2 anti-pattern: cloning a separate outdoors app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.13. Workspace profile: cockpit.
D-2.13. Physical context: flight deck, rail cab, bridge.
D-2.13. UX intent: mission checklist and incident-safe readback.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=cockpit`.
D-2 anti-pattern: cloning a separate cockpit app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.14. Workspace profile: operating-room.
D-2.14. Physical context: OR, sterile field, procedure room.
D-2.14. UX intent: voice, sterile confirmation, and audit trail.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=operating-room`.
D-2 anti-pattern: cloning a separate operating-room app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.15. Workspace profile: retail-floor.
D-2.15. Physical context: store aisle, customer counter, curbside pickup.
D-2.15. UX intent: associate lookup, pick, stage, handoff.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=retail-floor`.
D-2 anti-pattern: cloning a separate retail-floor app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

D-2.16. Workspace profile: classroom.
D-2.16. Physical context: teacher station, student device, accessibility support.
D-2.16. UX intent: attention-aware lesson and admin flow.
D-2 precedent A: Walmart associate and Store Assist workflows for environment-specific work.
D-2 precedent B: UPSNav and ORION for route and stop-specific driver guidance.
D-2 rule: workspace changes density, default sort, hazard prompts, and input mode, not authorization.
D-2 offline: ADR-0306 determines whether work queues locally, refuses, or routes to emergency mode.
D-2 privacy: shared screens and public spaces default to minimum necessary disclosure.
D-2 accessibility: lighting, noise, gloves, motion, and fatigue become profile constraints.
D-2 audit: UXProfileResolved includes `workspace_profile=classroom`.
D-2 anti-pattern: cloning a separate classroom app with its own data model.
D-2 test: role action ids and audit event classes match the canonical workflow fixture.

### D-3. Voice-first surface
D-3.01. Mechanic: Resolve role, tenant, locale, and device profile before accepting a command.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.02. Mechanic: Expose only actions whose Cedar decision can be explained in short readback.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.03. Mechanic: Use wake, intent, slot, confirmation, execution, evidence, and completion states.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.04. Mechanic: Require readback for destructive, regulated, safety, payment, or identity actions.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.05. Mechanic: Provide interrupt, repeat, help, cancel, escalate, and switch-to-visual commands.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.06. Mechanic: Degrade to DTMF or operator handoff when speech confidence is below policy floor.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.07. Mechanic: Emit VoiceIntentParsed, VoiceReadbackConfirmed, and VoiceActionExecuted audit events.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.08. Mechanic: Keep transcripts purpose-scoped and retention-bound per compliance pack.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.09. Mechanic: Use Intelligence as mediator without granting it policy authority.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.
D-3.10. Mechanic: Use ADR-0317 action vocabulary so voice commands transfer to visual gestures.
D-3 precedent A: RealWear speech-driven frontline smartglass command model.
D-3 precedent B: UPS in-vehicle driver navigation constrains attention while moving.
D-3 anti-pattern: voice assistant invents a command not present in the canonical action registry.

### D-4. Offline-first surface
D-4.01. Offline primitive: command queue.
D-4.01. Behavior: records requested action, actor, tenant, profile, and idempotency key.
D-4 precedent A: ADR-0306 disaster-mode and CRDT-backed sync doctrine.
D-4 precedent B: mobile field precedents from UPS and Walmart operate under route/store locality.
D-4 anti-pattern: silent last-write-wins on regulated workflow state.
D-4 test: partition fixture proves deterministic replay or explicit refusal.
D-4.02. Offline primitive: CRDT merge.
D-4.02. Behavior: uses ADR-0306 compatible merge semantics for collaborative fields.
D-4 precedent A: ADR-0306 disaster-mode and CRDT-backed sync doctrine.
D-4 precedent B: mobile field precedents from UPS and Walmart operate under route/store locality.
D-4 anti-pattern: silent last-write-wins on regulated workflow state.
D-4 test: partition fixture proves deterministic replay or explicit refusal.
D-4.03. Offline primitive: conflict inspector.
D-4.03. Behavior: shows human-readable divergence when automatic merge is unsafe.
D-4 precedent A: ADR-0306 disaster-mode and CRDT-backed sync doctrine.
D-4 precedent B: mobile field precedents from UPS and Walmart operate under route/store locality.
D-4 anti-pattern: silent last-write-wins on regulated workflow state.
D-4 test: partition fixture proves deterministic replay or explicit refusal.
D-4.04. Offline primitive: evidence cache.
D-4.04. Behavior: stores photos, scans, signatures, and voice confirmations encrypted at rest.
D-4 precedent A: ADR-0306 disaster-mode and CRDT-backed sync doctrine.
D-4 precedent B: mobile field precedents from UPS and Walmart operate under route/store locality.
D-4 anti-pattern: silent last-write-wins on regulated workflow state.
D-4 test: partition fixture proves deterministic replay or explicit refusal.
D-4.05. Offline primitive: cell replay.
D-4.05. Behavior: replays in order against tenant cell once connectivity returns.
D-4 precedent A: ADR-0306 disaster-mode and CRDT-backed sync doctrine.
D-4 precedent B: mobile field precedents from UPS and Walmart operate under route/store locality.
D-4 anti-pattern: silent last-write-wins on regulated workflow state.
D-4 test: partition fixture proves deterministic replay or explicit refusal.
D-4.06. Offline primitive: emergency bypass.
D-4.06. Behavior: preserves life-safety paths without broad offline privilege.
D-4 precedent A: ADR-0306 disaster-mode and CRDT-backed sync doctrine.
D-4 precedent B: mobile field precedents from UPS and Walmart operate under route/store locality.
D-4 anti-pattern: silent last-write-wins on regulated workflow state.
D-4 test: partition fixture proves deterministic replay or explicit refusal.
D-4.07. Offline primitive: sovereign boundary.
D-4.07. Behavior: refuses offline export when pack forbids local retention.
D-4 precedent A: ADR-0306 disaster-mode and CRDT-backed sync doctrine.
D-4 precedent B: mobile field precedents from UPS and Walmart operate under route/store locality.
D-4 anti-pattern: silent last-write-wins on regulated workflow state.
D-4 test: partition fixture proves deterministic replay or explicit refusal.
D-4.08. Offline primitive: sync telemetry.
D-4.08. Behavior: emits queue depth, conflict count, merge latency, and retry budget.
D-4 precedent A: ADR-0306 disaster-mode and CRDT-backed sync doctrine.
D-4 precedent B: mobile field precedents from UPS and Walmart operate under route/store locality.
D-4 anti-pattern: silent last-write-wins on regulated workflow state.
D-4 test: partition fixture proves deterministic replay or explicit refusal.

### D-5. Wearable-first surface
D-5.01. Wearable class: Apple Watch.
D-5.01. Role: complication, notification, quick approve, haptic escalation.
D-5 precedent A: Apple watchOS guidance favors glanceable, brief, focused interactions.
D-5 precedent B: Samsung Galaxy Watch docs require checking sensor availability per device.
D-5 precedent C: RealWear positions voice-powered smartglasses for frontline workforces.
D-5 rule: wearable actions MUST be bounded, reversible, or explicitly readback-confirmed.
D-5 anti-pattern: full desktop form squeezed into a watch view.
D-5 test: no wearable fixture may require more than three interaction steps for critical ack.
D-5.02. Wearable class: Galaxy Watch.
D-5.02. Role: sensor-aware glance, health/safety check, task acknowledgment.
D-5 precedent A: Apple watchOS guidance favors glanceable, brief, focused interactions.
D-5 precedent B: Samsung Galaxy Watch docs require checking sensor availability per device.
D-5 precedent C: RealWear positions voice-powered smartglasses for frontline workforces.
D-5 rule: wearable actions MUST be bounded, reversible, or explicitly readback-confirmed.
D-5 anti-pattern: full desktop form squeezed into a watch view.
D-5 test: no wearable fixture may require more than three interaction steps for critical ack.
D-5.03. Wearable class: Fitbit class tracker.
D-5.03. Role: wellness signal, shift fatigue hint, simple acknowledgement.
D-5 precedent A: Apple watchOS guidance favors glanceable, brief, focused interactions.
D-5 precedent B: Samsung Galaxy Watch docs require checking sensor availability per device.
D-5 precedent C: RealWear positions voice-powered smartglasses for frontline workforces.
D-5 rule: wearable actions MUST be bounded, reversible, or explicitly readback-confirmed.
D-5 anti-pattern: full desktop form squeezed into a watch view.
D-5 test: no wearable fixture may require more than three interaction steps for critical ack.
D-5.04. Wearable class: smartglasses.
D-5.04. Role: hands-free overlay, voice command, remote expert, procedure step.
D-5 precedent A: Apple watchOS guidance favors glanceable, brief, focused interactions.
D-5 precedent B: Samsung Galaxy Watch docs require checking sensor availability per device.
D-5 precedent C: RealWear positions voice-powered smartglasses for frontline workforces.
D-5 rule: wearable actions MUST be bounded, reversible, or explicitly readback-confirmed.
D-5 anti-pattern: full desktop form squeezed into a watch view.
D-5 test: no wearable fixture may require more than three interaction steps for critical ack.
D-5.05. Wearable class: industrial smartglasses class.
D-5.05. Role: PPE-compatible, noisy-site, object-anchored work.
D-5 precedent A: Apple watchOS guidance favors glanceable, brief, focused interactions.
D-5 precedent B: Samsung Galaxy Watch docs require checking sensor availability per device.
D-5 precedent C: RealWear positions voice-powered smartglasses for frontline workforces.
D-5 rule: wearable actions MUST be bounded, reversible, or explicitly readback-confirmed.
D-5 anti-pattern: full desktop form squeezed into a watch view.
D-5 test: no wearable fixture may require more than three interaction steps for critical ack.

### D-6. Per-collar-color UX library
D-6.01. Collar shell: knowledge-worker.
D-6.01. Segment: white-collar knowledge work.
D-6.01. Primary work: documents, approvals, analytics, workflow authoring.
D-6 precedent A: Amazon Mechanical Turk qualifications show work eligibility as profile metadata.
D-6 precedent B: Walmart associate app shows worker-context-specific mobile workflow without consumer fork.
D-6 rule: collar shell changes vocabulary density, examples, defaults, and workflow grouping.
D-6 rule: collar shell MUST NOT change tenant scope, policy decision, or audit event class.
D-6 data: shell registry row stores collar_shell_id, allowed roles, examples, and fallback shell.
D-6 handoff: role projection can move from collar shell to another without data migration.
D-6 anti-pattern: assuming white-collar document metaphors are universal.
D-6 test: canonical approve/escalate/evidence gestures transfer across collar shell fixtures.

D-6.02. Collar shell: trades.
D-6.02. Segment: blue-collar skilled trade work.
D-6.02. Primary work: inspection, dispatch, repair, materials, safety.
D-6 precedent A: Amazon Mechanical Turk qualifications show work eligibility as profile metadata.
D-6 precedent B: Walmart associate app shows worker-context-specific mobile workflow without consumer fork.
D-6 rule: collar shell changes vocabulary density, examples, defaults, and workflow grouping.
D-6 rule: collar shell MUST NOT change tenant scope, policy decision, or audit event class.
D-6 data: shell registry row stores collar_shell_id, allowed roles, examples, and fallback shell.
D-6 handoff: role projection can move from collar shell to another without data migration.
D-6 anti-pattern: assuming white-collar document metaphors are universal.
D-6 test: canonical approve/escalate/evidence gestures transfer across collar shell fixtures.

D-6.03. Collar shell: care-service.
D-6.03. Segment: pink-collar care and service work.
D-6.03. Primary work: handoff, scheduling, patient/client context, consent.
D-6 precedent A: Amazon Mechanical Turk qualifications show work eligibility as profile metadata.
D-6 precedent B: Walmart associate app shows worker-context-specific mobile workflow without consumer fork.
D-6 rule: collar shell changes vocabulary density, examples, defaults, and workflow grouping.
D-6 rule: collar shell MUST NOT change tenant scope, policy decision, or audit event class.
D-6 data: shell registry row stores collar_shell_id, allowed roles, examples, and fallback shell.
D-6 handoff: role projection can move from collar shell to another without data migration.
D-6 anti-pattern: assuming white-collar document metaphors are universal.
D-6 test: canonical approve/escalate/evidence gestures transfer across collar shell fixtures.

D-6.04. Collar shell: agricultural.
D-6.04. Segment: green-collar agricultural work.
D-6.04. Primary work: field log, weather, equipment, crop/animal workflow.
D-6 precedent A: Amazon Mechanical Turk qualifications show work eligibility as profile metadata.
D-6 precedent B: Walmart associate app shows worker-context-specific mobile workflow without consumer fork.
D-6 rule: collar shell changes vocabulary density, examples, defaults, and workflow grouping.
D-6 rule: collar shell MUST NOT change tenant scope, policy decision, or audit event class.
D-6 data: shell registry row stores collar_shell_id, allowed roles, examples, and fallback shell.
D-6 handoff: role projection can move from collar shell to another without data migration.
D-6 anti-pattern: assuming white-collar document metaphors are universal.
D-6 test: canonical approve/escalate/evidence gestures transfer across collar shell fixtures.

D-6.05. Collar shell: specialized-trade.
D-6.05. Segment: gold-collar specialized-trade work.
D-6.05. Primary work: licensed procedure, calibration, aviation, lab.
D-6 precedent A: Amazon Mechanical Turk qualifications show work eligibility as profile metadata.
D-6 precedent B: Walmart associate app shows worker-context-specific mobile workflow without consumer fork.
D-6 rule: collar shell changes vocabulary density, examples, defaults, and workflow grouping.
D-6 rule: collar shell MUST NOT change tenant scope, policy decision, or audit event class.
D-6 data: shell registry row stores collar_shell_id, allowed roles, examples, and fallback shell.
D-6 handoff: role projection can move from collar shell to another without data migration.
D-6 anti-pattern: assuming white-collar document metaphors are universal.
D-6 test: canonical approve/escalate/evidence gestures transfer across collar shell fixtures.

D-6.06. Collar shell: service-industry.
D-6.06. Segment: retail, hospitality, food, transport service.
D-6.06. Primary work: queue, handoff, shift, task, exception.
D-6 precedent A: Amazon Mechanical Turk qualifications show work eligibility as profile metadata.
D-6 precedent B: Walmart associate app shows worker-context-specific mobile workflow without consumer fork.
D-6 rule: collar shell changes vocabulary density, examples, defaults, and workflow grouping.
D-6 rule: collar shell MUST NOT change tenant scope, policy decision, or audit event class.
D-6 data: shell registry row stores collar_shell_id, allowed roles, examples, and fallback shell.
D-6 handoff: role projection can move from collar shell to another without data migration.
D-6 anti-pattern: assuming white-collar document metaphors are universal.
D-6 test: canonical approve/escalate/evidence gestures transfer across collar shell fixtures.

### D-7. Tenure-arc invariant
D-7.01. Tenure point: day-zero.
D-7.01. UX state: guided first-shift workflow, explicit labels, no hidden state.
D-7 rule: tenure affects explanation density and shortcut exposure, not workflow semantics.
D-7 precedent A: Lyft driver app guidance exposes tools that help drivers plan where and when to work.
D-7 precedent B: Walmart associate app starts with schedule/task simplification and grows with role use.
D-7 onboarding: day-zero teaches action vocabulary used by expert mode.
D-7 veteran: expert shortcuts must remain discoverable, reversible, and auditable.
D-7 anti-pattern: hiding policy-sensitive context because the worker is experienced.
D-7 test: tenure fixtures produce the same command ids for equivalent work.

D-7.02. Tenure point: day-seven.
D-7.02. UX state: shortcuts begin, learned vocabulary reinforced.
D-7 rule: tenure affects explanation density and shortcut exposure, not workflow semantics.
D-7 precedent A: Lyft driver app guidance exposes tools that help drivers plan where and when to work.
D-7 precedent B: Walmart associate app starts with schedule/task simplification and grows with role use.
D-7 onboarding: day-zero teaches action vocabulary used by expert mode.
D-7 veteran: expert shortcuts must remain discoverable, reversible, and auditable.
D-7 anti-pattern: hiding policy-sensitive context because the worker is experienced.
D-7 test: tenure fixtures produce the same command ids for equivalent work.

D-7.03. Tenure point: day-thirty.
D-7.03. UX state: routine work shifts to compact cards and exception-first queues.
D-7 rule: tenure affects explanation density and shortcut exposure, not workflow semantics.
D-7 precedent A: Lyft driver app guidance exposes tools that help drivers plan where and when to work.
D-7 precedent B: Walmart associate app starts with schedule/task simplification and grows with role use.
D-7 onboarding: day-zero teaches action vocabulary used by expert mode.
D-7 veteran: expert shortcuts must remain discoverable, reversible, and auditable.
D-7 anti-pattern: hiding policy-sensitive context because the worker is experienced.
D-7 test: tenure fixtures produce the same command ids for equivalent work.

D-7.04. Tenure point: year-one.
D-7.04. UX state: operator can customize density without changing semantics.
D-7 rule: tenure affects explanation density and shortcut exposure, not workflow semantics.
D-7 precedent A: Lyft driver app guidance exposes tools that help drivers plan where and when to work.
D-7 precedent B: Walmart associate app starts with schedule/task simplification and grows with role use.
D-7 onboarding: day-zero teaches action vocabulary used by expert mode.
D-7 veteran: expert shortcuts must remain discoverable, reversible, and auditable.
D-7 anti-pattern: hiding policy-sensitive context because the worker is experienced.
D-7 test: tenure fixtures produce the same command ids for equivalent work.

D-7.05. Tenure point: year-ten.
D-7.05. UX state: expert mode exposes macros, batch actions, and fast audit filters.
D-7 rule: tenure affects explanation density and shortcut exposure, not workflow semantics.
D-7 precedent A: Lyft driver app guidance exposes tools that help drivers plan where and when to work.
D-7 precedent B: Walmart associate app starts with schedule/task simplification and grows with role use.
D-7 onboarding: day-zero teaches action vocabulary used by expert mode.
D-7 veteran: expert shortcuts must remain discoverable, reversible, and auditable.
D-7 anti-pattern: hiding policy-sensitive context because the worker is experienced.
D-7 test: tenure fixtures produce the same command ids for equivalent work.

D-7.06. Tenure point: year-thirty.
D-7.06. UX state: veteran muscle memory remains valid across new devices and jobs.
D-7 rule: tenure affects explanation density and shortcut exposure, not workflow semantics.
D-7 precedent A: Lyft driver app guidance exposes tools that help drivers plan where and when to work.
D-7 precedent B: Walmart associate app starts with schedule/task simplification and grows with role use.
D-7 onboarding: day-zero teaches action vocabulary used by expert mode.
D-7 veteran: expert shortcuts must remain discoverable, reversible, and auditable.
D-7 anti-pattern: hiding policy-sensitive context because the worker is experienced.
D-7 test: tenure fixtures produce the same command ids for equivalent work.

### D-8. Locale-aware UX per role
D-8.01. Locale pack: US-English.
D-8.01. Binding: LTR, imperial/metric toggle, ADA-first accessibility.
D-8 rule: locale means language plus jurisdiction, script, units, calendar, and legal overlays.
D-8 role: pack overlay modifies examples and required notices before shell rendering.
D-8 precedent A: Google Play distribution and filters account for device and market compatibility.
D-8 precedent B: Kaiser mobile care workflows combine care context and administrative coverage in one app.
D-8 minor: ADR-0292 overlays age thresholds and guardian flows where relevant.
D-8 anti-pattern: translation-only localization with no legal or workflow semantics.
D-8 test: locale fixture validates RTL/LTR, script shaping, date, unit, and policy notice fields.

D-8.02. Locale pack: KR-Korean.
D-8.02. Binding: Hangul, KR-PIPA overlays, honorific-aware templates.
D-8 rule: locale means language plus jurisdiction, script, units, calendar, and legal overlays.
D-8 role: pack overlay modifies examples and required notices before shell rendering.
D-8 precedent A: Google Play distribution and filters account for device and market compatibility.
D-8 precedent B: Kaiser mobile care workflows combine care context and administrative coverage in one app.
D-8 minor: ADR-0292 overlays age thresholds and guardian flows where relevant.
D-8 anti-pattern: translation-only localization with no legal or workflow semantics.
D-8 test: locale fixture validates RTL/LTR, script shaping, date, unit, and policy notice fields.

D-8.03. Locale pack: JP-Japanese.
D-8.03. Binding: kana/kanji density, JP youth and labor overlays.
D-8 rule: locale means language plus jurisdiction, script, units, calendar, and legal overlays.
D-8 role: pack overlay modifies examples and required notices before shell rendering.
D-8 precedent A: Google Play distribution and filters account for device and market compatibility.
D-8 precedent B: Kaiser mobile care workflows combine care context and administrative coverage in one app.
D-8 minor: ADR-0292 overlays age thresholds and guardian flows where relevant.
D-8 anti-pattern: translation-only localization with no legal or workflow semantics.
D-8 test: locale fixture validates RTL/LTR, script shaping, date, unit, and policy notice fields.

D-8.04. Locale pack: EU-Multilingual.
D-8.04. Binding: GDPR, works council, decimal and date localization.
D-8 rule: locale means language plus jurisdiction, script, units, calendar, and legal overlays.
D-8 role: pack overlay modifies examples and required notices before shell rendering.
D-8 precedent A: Google Play distribution and filters account for device and market compatibility.
D-8 precedent B: Kaiser mobile care workflows combine care context and administrative coverage in one app.
D-8 minor: ADR-0292 overlays age thresholds and guardian flows where relevant.
D-8 anti-pattern: translation-only localization with no legal or workflow semantics.
D-8 test: locale fixture validates RTL/LTR, script shaping, date, unit, and policy notice fields.

D-8.05. Locale pack: Arabic-RTL.
D-8.05. Binding: RTL layout, bidirectional forms, local calendar policy.
D-8 rule: locale means language plus jurisdiction, script, units, calendar, and legal overlays.
D-8 role: pack overlay modifies examples and required notices before shell rendering.
D-8 precedent A: Google Play distribution and filters account for device and market compatibility.
D-8 precedent B: Kaiser mobile care workflows combine care context and administrative coverage in one app.
D-8 minor: ADR-0292 overlays age thresholds and guardian flows where relevant.
D-8 anti-pattern: translation-only localization with no legal or workflow semantics.
D-8 test: locale fixture validates RTL/LTR, script shaping, date, unit, and policy notice fields.

D-8.06. Locale pack: Hindi-Indic.
D-8.06. Binding: Indic script shaping, low-bandwidth mobile fallback.
D-8 rule: locale means language plus jurisdiction, script, units, calendar, and legal overlays.
D-8 role: pack overlay modifies examples and required notices before shell rendering.
D-8 precedent A: Google Play distribution and filters account for device and market compatibility.
D-8 precedent B: Kaiser mobile care workflows combine care context and administrative coverage in one app.
D-8 minor: ADR-0292 overlays age thresholds and guardian flows where relevant.
D-8 anti-pattern: translation-only localization with no legal or workflow semantics.
D-8 test: locale fixture validates RTL/LTR, script shaping, date, unit, and policy notice fields.

D-8.07. Locale pack: Spanish-LatAm.
D-8.07. Binding: regional tax/workflow vocabulary and voice variants.
D-8 rule: locale means language plus jurisdiction, script, units, calendar, and legal overlays.
D-8 role: pack overlay modifies examples and required notices before shell rendering.
D-8 precedent A: Google Play distribution and filters account for device and market compatibility.
D-8 precedent B: Kaiser mobile care workflows combine care context and administrative coverage in one app.
D-8 minor: ADR-0292 overlays age thresholds and guardian flows where relevant.
D-8 anti-pattern: translation-only localization with no legal or workflow semantics.
D-8 test: locale fixture validates RTL/LTR, script shaping, date, unit, and policy notice fields.

### D-9. Per-disability accommodation UX
D-9.01. Accommodation profile: voice-only.
D-9.01. Use case: hands unavailable, mobility impairment, sterile field.
D-9 rule: accommodation is a first-class projection profile, not a hidden preference toggle.
D-9 equivalence: action capability remains equivalent unless law, consent, or safety forbids it.
D-9 precedent A: Apple Accessibility guidance centers adaptable and perceivable interfaces.
D-9 precedent B: healthcare mobile flows show why patient and caregiver access must stay clear.
D-9 audit: accommodation resolution emits profile id without exposing sensitive medical detail by default.
D-9 anti-pattern: accessible view that only supports read-only work.
D-9 test: screen-reader and switch fixtures complete the same acceptance criteria as visual fixtures.

D-9.02. Accommodation profile: single-switch.
D-9.02. Use case: motor impairment, rugged gloves, switch control.
D-9 rule: accommodation is a first-class projection profile, not a hidden preference toggle.
D-9 equivalence: action capability remains equivalent unless law, consent, or safety forbids it.
D-9 precedent A: Apple Accessibility guidance centers adaptable and perceivable interfaces.
D-9 precedent B: healthcare mobile flows show why patient and caregiver access must stay clear.
D-9 audit: accommodation resolution emits profile id without exposing sensitive medical detail by default.
D-9 anti-pattern: accessible view that only supports read-only work.
D-9 test: screen-reader and switch fixtures complete the same acceptance criteria as visual fixtures.

D-9.03. Accommodation profile: screen-reader-only.
D-9.03. Use case: blindness, low-vision, temporary display failure.
D-9 rule: accommodation is a first-class projection profile, not a hidden preference toggle.
D-9 equivalence: action capability remains equivalent unless law, consent, or safety forbids it.
D-9 precedent A: Apple Accessibility guidance centers adaptable and perceivable interfaces.
D-9 precedent B: healthcare mobile flows show why patient and caregiver access must stay clear.
D-9 audit: accommodation resolution emits profile id without exposing sensitive medical detail by default.
D-9 anti-pattern: accessible view that only supports read-only work.
D-9 test: screen-reader and switch fixtures complete the same acceptance criteria as visual fixtures.

D-9.04. Accommodation profile: aac.
D-9.04. Use case: speech disability, noisy environment, nonverbal communication.
D-9 rule: accommodation is a first-class projection profile, not a hidden preference toggle.
D-9 equivalence: action capability remains equivalent unless law, consent, or safety forbids it.
D-9 precedent A: Apple Accessibility guidance centers adaptable and perceivable interfaces.
D-9 precedent B: healthcare mobile flows show why patient and caregiver access must stay clear.
D-9 audit: accommodation resolution emits profile id without exposing sensitive medical detail by default.
D-9 anti-pattern: accessible view that only supports read-only work.
D-9 test: screen-reader and switch fixtures complete the same acceptance criteria as visual fixtures.

D-9.05. Accommodation profile: post-stroke.
D-9.05. Use case: aphasia, motor planning, fatigue-aware pacing.
D-9 rule: accommodation is a first-class projection profile, not a hidden preference toggle.
D-9 equivalence: action capability remains equivalent unless law, consent, or safety forbids it.
D-9 precedent A: Apple Accessibility guidance centers adaptable and perceivable interfaces.
D-9 precedent B: healthcare mobile flows show why patient and caregiver access must stay clear.
D-9 audit: accommodation resolution emits profile id without exposing sensitive medical detail by default.
D-9 anti-pattern: accessible view that only supports read-only work.
D-9 test: screen-reader and switch fixtures complete the same acceptance criteria as visual fixtures.

D-9.06. Accommodation profile: post-trauma.
D-9.06. Use case: low-stimulation, panic-safe, predictable interaction.
D-9 rule: accommodation is a first-class projection profile, not a hidden preference toggle.
D-9 equivalence: action capability remains equivalent unless law, consent, or safety forbids it.
D-9 precedent A: Apple Accessibility guidance centers adaptable and perceivable interfaces.
D-9 precedent B: healthcare mobile flows show why patient and caregiver access must stay clear.
D-9 audit: accommodation resolution emits profile id without exposing sensitive medical detail by default.
D-9 anti-pattern: accessible view that only supports read-only work.
D-9 test: screen-reader and switch fixtures complete the same acceptance criteria as visual fixtures.

D-9.07. Accommodation profile: cognitive-impairment.
D-9.07. Use case: memory support, simplified choices, caregiver consent.
D-9 rule: accommodation is a first-class projection profile, not a hidden preference toggle.
D-9 equivalence: action capability remains equivalent unless law, consent, or safety forbids it.
D-9 precedent A: Apple Accessibility guidance centers adaptable and perceivable interfaces.
D-9 precedent B: healthcare mobile flows show why patient and caregiver access must stay clear.
D-9 audit: accommodation resolution emits profile id without exposing sensitive medical detail by default.
D-9 anti-pattern: accessible view that only supports read-only work.
D-9 test: screen-reader and switch fixtures complete the same acceptance criteria as visual fixtures.

### D-10. Same training transfers across all axes
D-10.01. Transfer rule: Action names stay stable: approve, reject, assign, claim, acknowledge, escalate, sign, scan, attach, route, defer, and close.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.02. Transfer rule: Icons, voice phrases, shortcuts, and haptics map to the same action ids.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.03. Transfer rule: Role-specific vocabulary is a label overlay on top of canonical actions.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.04. Transfer rule: Workflow Studio templates inherit the same action ids for no-code and code paths.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.05. Transfer rule: Training modules reference both a default office example and at least one non-office example.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.06. Transfer rule: Every high-risk action has the same confirmation posture across device classes.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.07. Transfer rule: Undo, cancel, help, and escalation gestures are universal escape hatches.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.08. Transfer rule: Profile-specific defaults can reorder tasks but cannot hide required risk context.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.09. Transfer rule: Audit history uses canonical event labels, not shell-specific labels.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.
D-10.10. Transfer rule: ADR-0317 owns the unified shell vocabulary; ADR-0318 owns universality coverage.
D-10 precedent A: Apple and Google platform guidance both make capability and form-factor constraints explicit.
D-10 precedent B: Walmart, UPS, Lyft, Kaiser, and MTurk show role/environment adaptation at scale.
D-10 anti-pattern: training a separate mental model for each workforce segment.

## E. Implementation Footprint
The implementation footprint is registry-first. Every user-facing microservice declares profile
support in machine-readable files and binds runtime adapters to canonical action ids. The doctrine
does not require every microservice to ship every profile on day one; it requires every microservice
to declare supported, unsupported, fallback, and risk status explicitly.

### E-1. Shared packages and registries
E-1.01. Artifact: crates/oya-ux-universality-kernel.
E-1.01. Purpose: profile ids, invariants, action parity traits.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.02. Artifact: crates/oya-ux-projection-domain.
E-1.02. Purpose: role, tenure, locale, workspace projection model.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.03. Artifact: crates/oya-ux-device-profile-registry.
E-1.03. Purpose: device profile schema and validation.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.04. Artifact: crates/oya-ux-collar-shell-library.
E-1.04. Purpose: shared collar-color UX shell descriptors.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.05. Artifact: crates/oya-ux-accessibility-accommodation-kernel.
E-1.05. Purpose: accommodation equivalence checks.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.06. Artifact: registry/ux/device-profiles.json.
E-1.06. Purpose: canonical device profile rows.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.07. Artifact: registry/ux/workspace-profiles.json.
E-1.07. Purpose: canonical workspace profile rows.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.08. Artifact: registry/ux/collar-shells.json.
E-1.08. Purpose: canonical collar shell rows.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.09. Artifact: registry/ux/accommodation-profiles.json.
E-1.09. Purpose: accessibility profile rows.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.
E-1.10. Artifact: registry/ux/action-vocabulary.json.
E-1.10. Purpose: ADR-0317 action ids and aliases.
E-1 contract: artifact carries binding ADR ADR-0318 and related ADR-0317.
E-1 validation: schema parse, SemVer, examples, and golden fixture coverage.
E-1 anti-pattern: local JSON fragment inside a single app with no registry row.

### E-2. Per-microservice adapter footprint
E-2.01. Microservice: analytics.
E-2.01. Required declaration: microservices/analytics/ux/device-profile-matrix.json.
E-2.01. Required declaration: microservices/analytics/ux/workspace-profile-matrix.json.
E-2.01. Required declaration: microservices/analytics/ux/collar-color-shell-matrix.json.
E-2.01. Required declaration: microservices/analytics/ux/accommodation-profile-matrix.json.
E-2.01. Runtime adapter: crates/oya-analytics-ux-profile-adapter when the service renders user-facing work.
E-2.01. Audit: UXProfileResolved includes microservice=analytics.
E-2.01. Test: profile matrix fixture verifies canonical action ids for analytics.

E-2.02. Microservice: api-gateway.
E-2.02. Required declaration: microservices/api-gateway/ux/device-profile-matrix.json.
E-2.02. Required declaration: microservices/api-gateway/ux/workspace-profile-matrix.json.
E-2.02. Required declaration: microservices/api-gateway/ux/collar-color-shell-matrix.json.
E-2.02. Required declaration: microservices/api-gateway/ux/accommodation-profile-matrix.json.
E-2.02. Runtime adapter: crates/oya-api-gateway-ux-profile-adapter when the service renders user-facing work.
E-2.02. Audit: UXProfileResolved includes microservice=api-gateway.
E-2.02. Test: profile matrix fixture verifies canonical action ids for api-gateway.

E-2.03. Microservice: application.
E-2.03. Required declaration: microservices/application/ux/device-profile-matrix.json.
E-2.03. Required declaration: microservices/application/ux/workspace-profile-matrix.json.
E-2.03. Required declaration: microservices/application/ux/collar-color-shell-matrix.json.
E-2.03. Required declaration: microservices/application/ux/accommodation-profile-matrix.json.
E-2.03. Runtime adapter: crates/oya-application-ux-profile-adapter when the service renders user-facing work.
E-2.03. Audit: UXProfileResolved includes microservice=application.
E-2.03. Test: profile matrix fixture verifies canonical action ids for application.

E-2.04. Microservice: audit-chain.
E-2.04. Required declaration: microservices/audit-chain/ux/device-profile-matrix.json.
E-2.04. Required declaration: microservices/audit-chain/ux/workspace-profile-matrix.json.
E-2.04. Required declaration: microservices/audit-chain/ux/collar-color-shell-matrix.json.
E-2.04. Required declaration: microservices/audit-chain/ux/accommodation-profile-matrix.json.
E-2.04. Runtime adapter: crates/oya-audit-chain-ux-profile-adapter when the service renders user-facing work.
E-2.04. Audit: UXProfileResolved includes microservice=audit-chain.
E-2.04. Test: profile matrix fixture verifies canonical action ids for audit-chain.

E-2.05. Microservice: calendar.
E-2.05. Required declaration: microservices/calendar/ux/device-profile-matrix.json.
E-2.05. Required declaration: microservices/calendar/ux/workspace-profile-matrix.json.
E-2.05. Required declaration: microservices/calendar/ux/collar-color-shell-matrix.json.
E-2.05. Required declaration: microservices/calendar/ux/accommodation-profile-matrix.json.
E-2.05. Runtime adapter: crates/oya-calendar-ux-profile-adapter when the service renders user-facing work.
E-2.05. Audit: UXProfileResolved includes microservice=calendar.
E-2.05. Test: profile matrix fixture verifies canonical action ids for calendar.

E-2.06. Microservice: cell.
E-2.06. Required declaration: microservices/api-gateway/ARCHITECTURE.md#cell-aware-routing device profile matrix.
E-2.06. Required declaration: microservices/tenancy/ARCHITECTURE.md#cell-assignment workspace profile matrix.
E-2.06. Required declaration: microservices/tenancy/ARCHITECTURE.md#cell-assignment collar-color shell matrix.
E-2.06. Required declaration: microservices/tenancy/ARCHITECTURE.md#cell-assignment accommodation profile matrix.
E-2.06. Runtime adapter: crates/oya-cell-ux-profile-adapter when the service renders user-facing work.
E-2.06. Audit: UXProfileResolved includes microservice=cell.
E-2.06. Test: profile matrix fixture verifies canonical action ids for cell.

E-2.07. Microservice: cloud-iac.
E-2.07. Required declaration: microservices/cloud-iac/ux/device-profile-matrix.json.
E-2.07. Required declaration: microservices/cloud-iac/ux/workspace-profile-matrix.json.
E-2.07. Required declaration: microservices/cloud-iac/ux/collar-color-shell-matrix.json.
E-2.07. Required declaration: microservices/cloud-iac/ux/accommodation-profile-matrix.json.
E-2.07. Runtime adapter: crates/oya-cloud-iac-ux-profile-adapter when the service renders user-facing work.
E-2.07. Audit: UXProfileResolved includes microservice=cloud-iac.
E-2.07. Test: profile matrix fixture verifies canonical action ids for cloud-iac.

E-2.08. Microservice: cloud-k8s.
E-2.08. Required declaration: microservices/cloud-k8s/ux/device-profile-matrix.json.
E-2.08. Required declaration: microservices/cloud-k8s/ux/workspace-profile-matrix.json.
E-2.08. Required declaration: microservices/cloud-k8s/ux/collar-color-shell-matrix.json.
E-2.08. Required declaration: microservices/cloud-k8s/ux/accommodation-profile-matrix.json.
E-2.08. Runtime adapter: crates/oya-cloud-k8s-ux-profile-adapter when the service renders user-facing work.
E-2.08. Audit: UXProfileResolved includes microservice=cloud-k8s.
E-2.08. Test: profile matrix fixture verifies canonical action ids for cloud-k8s.

E-2.09. Microservice: cloud-secrets.
E-2.09. Required declaration: microservices/cloud-secrets/ux/device-profile-matrix.json.
E-2.09. Required declaration: microservices/cloud-secrets/ux/workspace-profile-matrix.json.
E-2.09. Required declaration: microservices/cloud-secrets/ux/collar-color-shell-matrix.json.
E-2.09. Required declaration: microservices/cloud-secrets/ux/accommodation-profile-matrix.json.
E-2.09. Runtime adapter: crates/oya-cloud-secrets-ux-profile-adapter when the service renders user-facing work.
E-2.09. Audit: UXProfileResolved includes microservice=cloud-secrets.
E-2.09. Test: profile matrix fixture verifies canonical action ids for cloud-secrets.

E-2.10. Microservice: comms-email.
E-2.10. Required declaration: microservices/comms-email/ux/device-profile-matrix.json.
E-2.10. Required declaration: microservices/comms-email/ux/workspace-profile-matrix.json.
E-2.10. Required declaration: microservices/comms-email/ux/collar-color-shell-matrix.json.
E-2.10. Required declaration: microservices/comms-email/ux/accommodation-profile-matrix.json.
E-2.10. Runtime adapter: crates/oya-comms-email-ux-profile-adapter when the service renders user-facing work.
E-2.10. Audit: UXProfileResolved includes microservice=comms-email.
E-2.10. Test: profile matrix fixture verifies canonical action ids for comms-email.

E-2.11. Microservice: community.
E-2.11. Required declaration: microservices/community/ux/device-profile-matrix.json.
E-2.11. Required declaration: microservices/community/ux/workspace-profile-matrix.json.
E-2.11. Required declaration: microservices/community/ux/collar-color-shell-matrix.json.
E-2.11. Required declaration: microservices/community/ux/accommodation-profile-matrix.json.
E-2.11. Runtime adapter: crates/oya-community-ux-profile-adapter when the service renders user-facing work.
E-2.11. Audit: UXProfileResolved includes microservice=community.
E-2.11. Test: profile matrix fixture verifies canonical action ids for community.

E-2.12. Microservice: compliance.
E-2.12. Required declaration: microservices/compliance/ux/device-profile-matrix.json.
E-2.12. Required declaration: microservices/compliance/ux/workspace-profile-matrix.json.
E-2.12. Required declaration: microservices/compliance/ux/collar-color-shell-matrix.json.
E-2.12. Required declaration: microservices/compliance/ux/accommodation-profile-matrix.json.
E-2.12. Runtime adapter: crates/oya-compliance-ux-profile-adapter when the service renders user-facing work.
E-2.12. Audit: UXProfileResolved includes microservice=compliance.
E-2.12. Test: profile matrix fixture verifies canonical action ids for compliance.

E-2.13. Microservice: connect.
E-2.13. Required declaration: microservices/connector/ux/device-profile-matrix.json.
E-2.13. Required declaration: microservices/connector/ux/workspace-profile-matrix.json.
E-2.13. Required declaration: microservices/connector/ux/collar-color-shell-matrix.json.
E-2.13. Required declaration: microservices/connector/ux/accommodation-profile-matrix.json.
E-2.13. Runtime adapter: crates/oya-connect-ux-profile-adapter when the service renders user-facing work.
E-2.13. Audit: UXProfileResolved includes microservice=connect.
E-2.13. Test: profile matrix fixture verifies canonical action ids for connect.

E-2.14. Microservice: consent-graph.
E-2.14. Required declaration: microservices/consent-graph/ux/device-profile-matrix.json.
E-2.14. Required declaration: microservices/consent-graph/ux/workspace-profile-matrix.json.
E-2.14. Required declaration: microservices/consent-graph/ux/collar-color-shell-matrix.json.
E-2.14. Required declaration: microservices/consent-graph/ux/accommodation-profile-matrix.json.
E-2.14. Runtime adapter: crates/oya-consent-graph-ux-profile-adapter when the service renders user-facing work.
E-2.14. Audit: UXProfileResolved includes microservice=consent-graph.
E-2.14. Test: profile matrix fixture verifies canonical action ids for consent-graph.

E-2.15. Microservice: crm.
E-2.15. Required declaration: microservices/crm/ux/device-profile-matrix.json.
E-2.15. Required declaration: microservices/crm/ux/workspace-profile-matrix.json.
E-2.15. Required declaration: microservices/crm/ux/collar-color-shell-matrix.json.
E-2.15. Required declaration: microservices/crm/ux/accommodation-profile-matrix.json.
E-2.15. Runtime adapter: crates/oya-crm-ux-profile-adapter when the service renders user-facing work.
E-2.15. Audit: UXProfileResolved includes microservice=crm.
E-2.15. Test: profile matrix fixture verifies canonical action ids for crm.

E-2.16. Microservice: developer-sdk.
E-2.16. Required declaration: microservices/developer-sdk/ux/device-profile-matrix.json.
E-2.16. Required declaration: microservices/developer-sdk/ux/workspace-profile-matrix.json.
E-2.16. Required declaration: microservices/developer-sdk/ux/collar-color-shell-matrix.json.
E-2.16. Required declaration: microservices/developer-sdk/ux/accommodation-profile-matrix.json.
E-2.16. Runtime adapter: crates/oya-developer-sdk-ux-profile-adapter when the service renders user-facing work.
E-2.16. Audit: UXProfileResolved includes microservice=developer-sdk.
E-2.16. Test: profile matrix fixture verifies canonical action ids for developer-sdk.

E-2.17. Microservice: docs.
E-2.17. Required declaration: microservices/docs/ux/device-profile-matrix.json.
E-2.17. Required declaration: microservices/docs/ux/workspace-profile-matrix.json.
E-2.17. Required declaration: microservices/docs/ux/collar-color-shell-matrix.json.
E-2.17. Required declaration: microservices/docs/ux/accommodation-profile-matrix.json.
E-2.17. Runtime adapter: crates/oya-docs-ux-profile-adapter when the service renders user-facing work.
E-2.17. Audit: UXProfileResolved includes microservice=docs.
E-2.17. Test: profile matrix fixture verifies canonical action ids for docs.

E-2.18. Microservice: drive.
E-2.18. Required declaration: microservices/drive/ux/device-profile-matrix.json.
E-2.18. Required declaration: microservices/drive/ux/workspace-profile-matrix.json.
E-2.18. Required declaration: microservices/drive/ux/collar-color-shell-matrix.json.
E-2.18. Required declaration: microservices/drive/ux/accommodation-profile-matrix.json.
E-2.18. Runtime adapter: crates/oya-drive-ux-profile-adapter when the service renders user-facing work.
E-2.18. Audit: UXProfileResolved includes microservice=drive.
E-2.18. Test: profile matrix fixture verifies canonical action ids for drive.

E-2.19. Microservice: feature-flags.
E-2.19. Required declaration: microservices/feature-flags/ux/device-profile-matrix.json.
E-2.19. Required declaration: microservices/feature-flags/ux/workspace-profile-matrix.json.
E-2.19. Required declaration: microservices/feature-flags/ux/collar-color-shell-matrix.json.
E-2.19. Required declaration: microservices/feature-flags/ux/accommodation-profile-matrix.json.
E-2.19. Runtime adapter: crates/oya-feature-flags-ux-profile-adapter when the service renders user-facing work.
E-2.19. Audit: UXProfileResolved includes microservice=feature-flags.
E-2.19. Test: profile matrix fixture verifies canonical action ids for feature-flags.

E-2.20. Microservice: finops-portal.
E-2.20. Required declaration: microservices/finops-portal/ux/device-profile-matrix.json.
E-2.20. Required declaration: microservices/finops-portal/ux/workspace-profile-matrix.json.
E-2.20. Required declaration: microservices/finops-portal/ux/collar-color-shell-matrix.json.
E-2.20. Required declaration: microservices/finops-portal/ux/accommodation-profile-matrix.json.
E-2.20. Runtime adapter: crates/oya-finops-portal-ux-profile-adapter when the service renders user-facing work.
E-2.20. Audit: UXProfileResolved includes microservice=finops-portal.
E-2.20. Test: profile matrix fixture verifies canonical action ids for finops-portal.

E-2.21. Microservice: forms.
E-2.21. Required declaration: microservices/forms/ux/device-profile-matrix.json.
E-2.21. Required declaration: microservices/forms/ux/workspace-profile-matrix.json.
E-2.21. Required declaration: microservices/forms/ux/collar-color-shell-matrix.json.
E-2.21. Required declaration: microservices/forms/ux/accommodation-profile-matrix.json.
E-2.21. Runtime adapter: crates/oya-forms-ux-profile-adapter when the service renders user-facing work.
E-2.21. Audit: UXProfileResolved includes microservice=forms.
E-2.21. Test: profile matrix fixture verifies canonical action ids for forms.

E-2.22. Microservice: foundry.
E-2.22. Required declaration: microservices/foundry/ux/device-profile-matrix.json.
E-2.22. Required declaration: microservices/foundry/ux/workspace-profile-matrix.json.
E-2.22. Required declaration: microservices/foundry/ux/collar-color-shell-matrix.json.
E-2.22. Required declaration: microservices/foundry/ux/accommodation-profile-matrix.json.
E-2.22. Runtime adapter: crates/oya-intelligence-ux-profile-adapter when the service renders user-facing work.
E-2.22. Audit: UXProfileResolved includes microservice=foundry.
E-2.22. Test: profile matrix fixture verifies canonical action ids for foundry.

E-2.23. Microservice: global-trade.
E-2.23. Required declaration: microservices/global-trade/ux/device-profile-matrix.json.
E-2.23. Required declaration: microservices/global-trade/ux/workspace-profile-matrix.json.
E-2.23. Required declaration: microservices/global-trade/ux/collar-color-shell-matrix.json.
E-2.23. Required declaration: microservices/global-trade/ux/accommodation-profile-matrix.json.
E-2.23. Runtime adapter: crates/oya-global-trade-ux-profile-adapter when the service renders user-facing work.
E-2.23. Audit: UXProfileResolved includes microservice=global-trade.
E-2.23. Test: profile matrix fixture verifies canonical action ids for global-trade.

E-2.24. Microservice: governance.
E-2.24. Required declaration: microservices/governance/ux/device-profile-matrix.json.
E-2.24. Required declaration: microservices/governance/ux/workspace-profile-matrix.json.
E-2.24. Required declaration: microservices/governance/ux/collar-color-shell-matrix.json.
E-2.24. Required declaration: microservices/governance/ux/accommodation-profile-matrix.json.
E-2.24. Runtime adapter: crates/oya-governance-ux-profile-adapter when the service renders user-facing work.
E-2.24. Audit: UXProfileResolved includes microservice=governance.
E-2.24. Test: profile matrix fixture verifies canonical action ids for governance.

E-2.25. Microservice: identity.
E-2.25. Required declaration: microservices/identity/ux/device-profile-matrix.json.
E-2.25. Required declaration: microservices/identity/ux/workspace-profile-matrix.json.
E-2.25. Required declaration: microservices/identity/ux/collar-color-shell-matrix.json.
E-2.25. Required declaration: microservices/identity/ux/accommodation-profile-matrix.json.
E-2.25. Runtime adapter: crates/oya-identity-ux-profile-adapter when the service renders user-facing work.
E-2.25. Audit: UXProfileResolved includes microservice=identity.
E-2.25. Test: profile matrix fixture verifies canonical action ids for identity.

E-2.26. Microservice: intelligence.
E-2.26. Required declaration: microservices/intelligence/ux/device-profile-matrix.json.
E-2.26. Required declaration: microservices/intelligence/ux/workspace-profile-matrix.json.
E-2.26. Required declaration: microservices/intelligence/ux/collar-color-shell-matrix.json.
E-2.26. Required declaration: microservices/intelligence/ux/accommodation-profile-matrix.json.
E-2.26. Runtime adapter: crates/oya-intelligence-ux-profile-adapter when the service renders user-facing work.
E-2.26. Audit: UXProfileResolved includes microservice=intelligence.
E-2.26. Test: profile matrix fixture verifies canonical action ids for intelligence.

E-2.27. Microservice: mail.
E-2.27. Required declaration: microservices/mail/ux/device-profile-matrix.json.
E-2.27. Required declaration: microservices/mail/ux/workspace-profile-matrix.json.
E-2.27. Required declaration: microservices/mail/ux/collar-color-shell-matrix.json.
E-2.27. Required declaration: microservices/mail/ux/accommodation-profile-matrix.json.
E-2.27. Runtime adapter: crates/oya-mail-ux-profile-adapter when the service renders user-facing work.
E-2.27. Audit: UXProfileResolved includes microservice=mail.
E-2.27. Test: profile matrix fixture verifies canonical action ids for mail.

E-2.28. Microservice: marketplace.
E-2.28. Required declaration: microservices/marketplace/ux/device-profile-matrix.json.
E-2.28. Required declaration: microservices/marketplace/ux/workspace-profile-matrix.json.
E-2.28. Required declaration: microservices/marketplace/ux/collar-color-shell-matrix.json.
E-2.28. Required declaration: microservices/marketplace/ux/accommodation-profile-matrix.json.
E-2.28. Runtime adapter: crates/oya-marketplace-ux-profile-adapter when the service renders user-facing work.
E-2.28. Audit: UXProfileResolved includes microservice=marketplace.
E-2.28. Test: profile matrix fixture verifies canonical action ids for marketplace.

E-2.29. Microservice: meet.
E-2.29. Required declaration: microservices/meet/ux/device-profile-matrix.json.
E-2.29. Required declaration: microservices/meet/ux/workspace-profile-matrix.json.
E-2.29. Required declaration: microservices/meet/ux/collar-color-shell-matrix.json.
E-2.29. Required declaration: microservices/meet/ux/accommodation-profile-matrix.json.
E-2.29. Runtime adapter: crates/oya-meet-ux-profile-adapter when the service renders user-facing work.
E-2.29. Audit: UXProfileResolved includes microservice=meet.
E-2.29. Test: profile matrix fixture verifies canonical action ids for meet.

E-2.30. Microservice: messenger.
E-2.30. Required declaration: microservices/messenger/ux/device-profile-matrix.json.
E-2.30. Required declaration: microservices/messenger/ux/workspace-profile-matrix.json.
E-2.30. Required declaration: microservices/messenger/ux/collar-color-shell-matrix.json.
E-2.30. Required declaration: microservices/messenger/ux/accommodation-profile-matrix.json.
E-2.30. Runtime adapter: crates/oya-messenger-ux-profile-adapter when the service renders user-facing work.
E-2.30. Audit: UXProfileResolved includes microservice=messenger.
E-2.30. Test: profile matrix fixture verifies canonical action ids for messenger.

E-2.31. Microservice: network.
E-2.31. Required declaration: microservices/community/ux/device-profile-matrix.json.
E-2.31. Required declaration: microservices/community/ux/workspace-profile-matrix.json.
E-2.31. Required declaration: microservices/community/ux/collar-color-shell-matrix.json.
E-2.31. Required declaration: microservices/community/ux/accommodation-profile-matrix.json.
E-2.31. Runtime adapter: crates/oya-network-ux-profile-adapter when the service renders user-facing work.
E-2.31. Audit: UXProfileResolved includes microservice=network.
E-2.31. Test: profile matrix fixture verifies canonical action ids for network.

E-2.32. Microservice: notes.
E-2.32. Required declaration: microservices/notes/ux/device-profile-matrix.json.
E-2.32. Required declaration: microservices/notes/ux/workspace-profile-matrix.json.
E-2.32. Required declaration: microservices/notes/ux/collar-color-shell-matrix.json.
E-2.32. Required declaration: microservices/notes/ux/accommodation-profile-matrix.json.
E-2.32. Runtime adapter: crates/oya-notes-ux-profile-adapter when the service renders user-facing work.
E-2.32. Audit: UXProfileResolved includes microservice=notes.
E-2.32. Test: profile matrix fixture verifies canonical action ids for notes.

E-2.33. Microservice: observability.
E-2.33. Required declaration: microservices/observability/ux/device-profile-matrix.json.
E-2.33. Required declaration: microservices/observability/ux/workspace-profile-matrix.json.
E-2.33. Required declaration: microservices/observability/ux/collar-color-shell-matrix.json.
E-2.33. Required declaration: microservices/observability/ux/accommodation-profile-matrix.json.
E-2.33. Runtime adapter: crates/oya-observability-ux-profile-adapter when the service renders user-facing work.
E-2.33. Audit: UXProfileResolved includes microservice=observability.
E-2.33. Test: profile matrix fixture verifies canonical action ids for observability.

E-2.34. Microservice: ontology.
E-2.34. Required declaration: microservices/ontology/ux/device-profile-matrix.json.
E-2.34. Required declaration: microservices/ontology/ux/workspace-profile-matrix.json.
E-2.34. Required declaration: microservices/ontology/ux/collar-color-shell-matrix.json.
E-2.34. Required declaration: microservices/ontology/ux/accommodation-profile-matrix.json.
E-2.34. Runtime adapter: crates/oya-ontology-ux-profile-adapter when the service renders user-facing work.
E-2.34. Audit: UXProfileResolved includes microservice=ontology.
E-2.34. Test: profile matrix fixture verifies canonical action ids for ontology.

E-2.35. Microservice: ops-dashboard-control-center.
E-2.35. Required declaration: microservices/ops-dashboard-control-center/ux/device-profile-matrix.json.
E-2.35. Required declaration: microservices/ops-dashboard-control-center/ux/workspace-profile-matrix.json.
E-2.35. Required declaration: microservices/ops-dashboard-control-center/ux/collar-color-shell-matrix.json.
E-2.35. Required declaration: microservices/ops-dashboard-control-center/ux/accommodation-profile-matrix.json.
E-2.35. Runtime adapter: crates/oya-ops-dashboard-control-center-ux-profile-adapter when the service renders user-facing work.
E-2.35. Audit: UXProfileResolved includes microservice=ops-dashboard-control-center.
E-2.35. Test: profile matrix fixture verifies canonical action ids for ops-dashboard-control-center.

E-2.36. Microservice: payments.
E-2.36. Required declaration: microservices/payments/ux/device-profile-matrix.json.
E-2.36. Required declaration: microservices/payments/ux/workspace-profile-matrix.json.
E-2.36. Required declaration: microservices/payments/ux/collar-color-shell-matrix.json.
E-2.36. Required declaration: microservices/payments/ux/accommodation-profile-matrix.json.
E-2.36. Runtime adapter: crates/oya-payments-ux-profile-adapter when the service renders user-facing work.
E-2.36. Audit: UXProfileResolved includes microservice=payments.
E-2.36. Test: profile matrix fixture verifies canonical action ids for payments.

E-2.37. Microservice: plant-maintenance.
E-2.37. Required declaration: microservices/plant-maintenance/ux/device-profile-matrix.json.
E-2.37. Required declaration: microservices/plant-maintenance/ux/workspace-profile-matrix.json.
E-2.37. Required declaration: microservices/plant-maintenance/ux/collar-color-shell-matrix.json.
E-2.37. Required declaration: microservices/plant-maintenance/ux/accommodation-profile-matrix.json.
E-2.37. Runtime adapter: crates/oya-plant-maintenance-ux-profile-adapter when the service renders user-facing work.
E-2.37. Audit: UXProfileResolved includes microservice=plant-maintenance.
E-2.37. Test: profile matrix fixture verifies canonical action ids for plant-maintenance.

E-2.38. Microservice: plugin-app-store.
E-2.38. Required declaration: microservices/plugin-app-store/ux/device-profile-matrix.json.
E-2.38. Required declaration: microservices/plugin-app-store/ux/workspace-profile-matrix.json.
E-2.38. Required declaration: microservices/plugin-app-store/ux/collar-color-shell-matrix.json.
E-2.38. Required declaration: microservices/plugin-app-store/ux/accommodation-profile-matrix.json.
E-2.38. Runtime adapter: crates/oya-plugin-app-store-ux-profile-adapter when the service renders user-facing work.
E-2.38. Audit: UXProfileResolved includes microservice=plugin-app-store.
E-2.38. Test: profile matrix fixture verifies canonical action ids for plugin-app-store.

E-2.39. Microservice: production-planning.
E-2.39. Required declaration: microservices/production-planning/ux/device-profile-matrix.json.
E-2.39. Required declaration: microservices/production-planning/ux/workspace-profile-matrix.json.
E-2.39. Required declaration: microservices/production-planning/ux/collar-color-shell-matrix.json.
E-2.39. Required declaration: microservices/production-planning/ux/accommodation-profile-matrix.json.
E-2.39. Runtime adapter: crates/oya-production-planning-ux-profile-adapter when the service renders user-facing work.
E-2.39. Audit: UXProfileResolved includes microservice=production-planning.
E-2.39. Test: profile matrix fixture verifies canonical action ids for production-planning.

E-2.40. Microservice: quality-management.
E-2.40. Required declaration: microservices/quality-management/ux/device-profile-matrix.json.
E-2.40. Required declaration: microservices/quality-management/ux/workspace-profile-matrix.json.
E-2.40. Required declaration: microservices/quality-management/ux/collar-color-shell-matrix.json.
E-2.40. Required declaration: microservices/quality-management/ux/accommodation-profile-matrix.json.
E-2.40. Runtime adapter: crates/oya-quality-management-ux-profile-adapter when the service renders user-facing work.
E-2.40. Audit: UXProfileResolved includes microservice=quality-management.
E-2.40. Test: profile matrix fixture verifies canonical action ids for quality-management.

E-2.41. Microservice: real-estate.
E-2.41. Required declaration: microservices/real-estate/ux/device-profile-matrix.json.
E-2.41. Required declaration: microservices/real-estate/ux/workspace-profile-matrix.json.
E-2.41. Required declaration: microservices/real-estate/ux/collar-color-shell-matrix.json.
E-2.41. Required declaration: microservices/real-estate/ux/accommodation-profile-matrix.json.
E-2.41. Runtime adapter: crates/oya-real-estate-ux-profile-adapter when the service renders user-facing work.
E-2.41. Audit: UXProfileResolved includes microservice=real-estate.
E-2.41. Test: profile matrix fixture verifies canonical action ids for real-estate.

E-2.42. Microservice: recordings.
E-2.42. Required declaration: microservices/recordings/ux/device-profile-matrix.json.
E-2.42. Required declaration: microservices/recordings/ux/workspace-profile-matrix.json.
E-2.42. Required declaration: microservices/recordings/ux/collar-color-shell-matrix.json.
E-2.42. Required declaration: microservices/recordings/ux/accommodation-profile-matrix.json.
E-2.42. Runtime adapter: crates/oya-recordings-ux-profile-adapter when the service renders user-facing work.
E-2.42. Audit: UXProfileResolved includes microservice=recordings.
E-2.42. Test: profile matrix fixture verifies canonical action ids for recordings.

E-2.43. Microservice: sheets.
E-2.43. Required declaration: microservices/sheets/ux/device-profile-matrix.json.
E-2.43. Required declaration: microservices/sheets/ux/workspace-profile-matrix.json.
E-2.43. Required declaration: microservices/sheets/ux/collar-color-shell-matrix.json.
E-2.43. Required declaration: microservices/sheets/ux/accommodation-profile-matrix.json.
E-2.43. Runtime adapter: crates/oya-sheets-ux-profile-adapter when the service renders user-facing work.
E-2.43. Audit: UXProfileResolved includes microservice=sheets.
E-2.43. Test: profile matrix fixture verifies canonical action ids for sheets.

E-2.44. Microservice: shorts.
E-2.44. Required declaration: microservices/shorts/ux/device-profile-matrix.json.
E-2.44. Required declaration: microservices/shorts/ux/workspace-profile-matrix.json.
E-2.44. Required declaration: microservices/shorts/ux/collar-color-shell-matrix.json.
E-2.44. Required declaration: microservices/shorts/ux/accommodation-profile-matrix.json.
E-2.44. Runtime adapter: crates/oya-shorts-ux-profile-adapter when the service renders user-facing work.
E-2.44. Audit: UXProfileResolved includes microservice=shorts.
E-2.44. Test: profile matrix fixture verifies canonical action ids for shorts.

E-2.45. Microservice: sites.
E-2.45. Required declaration: microservices/sites/ux/device-profile-matrix.json.
E-2.45. Required declaration: microservices/sites/ux/workspace-profile-matrix.json.
E-2.45. Required declaration: microservices/sites/ux/collar-color-shell-matrix.json.
E-2.45. Required declaration: microservices/sites/ux/accommodation-profile-matrix.json.
E-2.45. Runtime adapter: crates/oya-sites-ux-profile-adapter when the service renders user-facing work.
E-2.45. Audit: UXProfileResolved includes microservice=sites.
E-2.45. Test: profile matrix fixture verifies canonical action ids for sites.

E-2.46. Microservice: slides.
E-2.46. Required declaration: microservices/slides/ux/device-profile-matrix.json.
E-2.46. Required declaration: microservices/slides/ux/workspace-profile-matrix.json.
E-2.46. Required declaration: microservices/slides/ux/collar-color-shell-matrix.json.
E-2.46. Required declaration: microservices/slides/ux/accommodation-profile-matrix.json.
E-2.46. Runtime adapter: crates/oya-slides-ux-profile-adapter when the service renders user-facing work.
E-2.46. Audit: UXProfileResolved includes microservice=slides.
E-2.46. Test: profile matrix fixture verifies canonical action ids for slides.

E-2.47. Microservice: social.
E-2.47. Required declaration: microservices/social/ux/device-profile-matrix.json.
E-2.47. Required declaration: microservices/social/ux/workspace-profile-matrix.json.
E-2.47. Required declaration: microservices/social/ux/collar-color-shell-matrix.json.
E-2.47. Required declaration: microservices/social/ux/accommodation-profile-matrix.json.
E-2.47. Runtime adapter: crates/oya-community-social-ux-profile-adapter when the service renders user-facing work.
E-2.47. Audit: UXProfileResolved includes microservice=social.
E-2.47. Test: profile matrix fixture verifies canonical action ids for social.

E-2.48. Microservice: supply-chain-planning.
E-2.48. Required declaration: microservices/supply-chain-planning/ux/device-profile-matrix.json.
E-2.48. Required declaration: microservices/supply-chain-planning/ux/workspace-profile-matrix.json.
E-2.48. Required declaration: microservices/supply-chain-planning/ux/collar-color-shell-matrix.json.
E-2.48. Required declaration: microservices/supply-chain-planning/ux/accommodation-profile-matrix.json.
E-2.48. Runtime adapter: crates/oya-supply-chain-planning-ux-profile-adapter when the service renders user-facing work.
E-2.48. Audit: UXProfileResolved includes microservice=supply-chain-planning.
E-2.48. Test: profile matrix fixture verifies canonical action ids for supply-chain-planning.

E-2.49. Microservice: tasks.
E-2.49. Required declaration: microservices/tasks/ux/device-profile-matrix.json.
E-2.49. Required declaration: microservices/tasks/ux/workspace-profile-matrix.json.
E-2.49. Required declaration: microservices/tasks/ux/collar-color-shell-matrix.json.
E-2.49. Required declaration: microservices/tasks/ux/accommodation-profile-matrix.json.
E-2.49. Runtime adapter: crates/oya-tasks-ux-profile-adapter when the service renders user-facing work.
E-2.49. Audit: UXProfileResolved includes microservice=tasks.
E-2.49. Test: profile matrix fixture verifies canonical action ids for tasks.

E-2.50. Microservice: tenancy.
E-2.50. Required declaration: microservices/tenancy/ux/device-profile-matrix.json.
E-2.50. Required declaration: microservices/tenancy/ux/workspace-profile-matrix.json.
E-2.50. Required declaration: microservices/tenancy/ux/collar-color-shell-matrix.json.
E-2.50. Required declaration: microservices/tenancy/ux/accommodation-profile-matrix.json.
E-2.50. Runtime adapter: crates/oya-tenancy-ux-profile-adapter when the service renders user-facing work.
E-2.50. Audit: UXProfileResolved includes microservice=tenancy.
E-2.50. Test: profile matrix fixture verifies canonical action ids for tenancy.

E-2.51. Microservice: translate.
E-2.51. Required declaration: microservices/translate/ux/device-profile-matrix.json.
E-2.51. Required declaration: microservices/translate/ux/workspace-profile-matrix.json.
E-2.51. Required declaration: microservices/translate/ux/collar-color-shell-matrix.json.
E-2.51. Required declaration: microservices/translate/ux/accommodation-profile-matrix.json.
E-2.51. Runtime adapter: crates/oya-translate-ux-profile-adapter when the service renders user-facing work.
E-2.51. Audit: UXProfileResolved includes microservice=translate.
E-2.51. Test: profile matrix fixture verifies canonical action ids for translate.

E-2.52. Microservice: treasury.
E-2.52. Required declaration: microservices/treasury/ux/device-profile-matrix.json.
E-2.52. Required declaration: microservices/treasury/ux/workspace-profile-matrix.json.
E-2.52. Required declaration: microservices/treasury/ux/collar-color-shell-matrix.json.
E-2.52. Required declaration: microservices/treasury/ux/accommodation-profile-matrix.json.
E-2.52. Runtime adapter: crates/oya-treasury-ux-profile-adapter when the service renders user-facing work.
E-2.52. Audit: UXProfileResolved includes microservice=treasury.
E-2.52. Test: profile matrix fixture verifies canonical action ids for treasury.

E-2.53. Microservice: warehouse.
E-2.53. Required declaration: microservices/warehouse/ux/device-profile-matrix.json.
E-2.53. Required declaration: microservices/warehouse/ux/workspace-profile-matrix.json.
E-2.53. Required declaration: microservices/warehouse/ux/collar-color-shell-matrix.json.
E-2.53. Required declaration: microservices/warehouse/ux/accommodation-profile-matrix.json.
E-2.53. Runtime adapter: crates/oya-warehouse-ux-profile-adapter when the service renders user-facing work.
E-2.53. Audit: UXProfileResolved includes microservice=warehouse.
E-2.53. Test: profile matrix fixture verifies canonical action ids for warehouse.

E-2.54. Microservice: workflow-engine.
E-2.54. Required declaration: microservices/workflow-engine/ux/device-profile-matrix.json.
E-2.54. Required declaration: microservices/workflow-engine/ux/workspace-profile-matrix.json.
E-2.54. Required declaration: microservices/workflow-engine/ux/collar-color-shell-matrix.json.
E-2.54. Required declaration: microservices/workflow-engine/ux/accommodation-profile-matrix.json.
E-2.54. Runtime adapter: crates/oya-workflow-engine-ux-profile-adapter when the service renders user-facing work.
E-2.54. Audit: UXProfileResolved includes microservice=workflow-engine.
E-2.54. Test: profile matrix fixture verifies canonical action ids for workflow-engine.

E-2.55. Microservice: workflow-studio.
E-2.55. Required declaration: microservices/workflow-studio/ux/device-profile-matrix.json.
E-2.55. Required declaration: microservices/workflow-studio/ux/workspace-profile-matrix.json.
E-2.55. Required declaration: microservices/workflow-studio/ux/collar-color-shell-matrix.json.
E-2.55. Required declaration: microservices/workflow-studio/ux/accommodation-profile-matrix.json.
E-2.55. Runtime adapter: crates/oya-workflow-studio-ux-profile-adapter when the service renders user-facing work.
E-2.55. Audit: UXProfileResolved includes microservice=workflow-studio.
E-2.55. Test: profile matrix fixture verifies canonical action ids for workflow-studio.

E-2.56. Microservice: workplace-integration.
E-2.56. Required declaration: microservices/workplace-integration/ux/device-profile-matrix.json.
E-2.56. Required declaration: microservices/workplace-integration/ux/workspace-profile-matrix.json.
E-2.56. Required declaration: microservices/workplace-integration/ux/collar-color-shell-matrix.json.
E-2.56. Required declaration: microservices/workplace-integration/ux/accommodation-profile-matrix.json.
E-2.56. Runtime adapter: crates/oya-workplace-integration-ux-profile-adapter when the service renders user-facing work.
E-2.56. Audit: UXProfileResolved includes microservice=workplace-integration.
E-2.56. Test: profile matrix fixture verifies canonical action ids for workplace-integration.

### E-3. Observability hooks
E-3.01. Hook kind: metric.
E-3.01. Hook name: oya_ux_profile_resolution_total.
E-3.01. Semantics: counter by microservice, device_profile, workspace_profile, collar_shell.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.02. Hook kind: metric.
E-3.02. Hook name: oya_ux_profile_fallback_total.
E-3.02. Semantics: counter by source_profile, fallback_profile, reason.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.03. Hook kind: metric.
E-3.03. Hook name: oya_ux_offline_queue_depth.
E-3.03. Semantics: gauge by tenant, cell, device_profile.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.04. Hook kind: metric.
E-3.04. Hook name: oya_ux_action_parity_violation_total.
E-3.04. Semantics: counter by action_id and profile.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.05. Hook kind: trace.
E-3.05. Hook name: ux.profile.resolve.
E-3.05. Semantics: span around profile bundle resolution.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.06. Hook kind: trace.
E-3.06. Hook name: ux.projection.render.
E-3.06. Semantics: span around shell projection render.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.07. Hook kind: log.
E-3.07. Hook name: ux.profile.fallback.
E-3.07. Semantics: structured warning when fallback activates.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.08. Hook kind: audit.
E-3.08. Hook name: UXProfileResolved.
E-3.08. Semantics: profile bundle selected for user action.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.09. Hook kind: audit.
E-3.09. Hook name: UXProfileFallbackActivated.
E-3.09. Semantics: fallback selected with reason and policy context.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.
E-3.10. Hook kind: audit.
E-3.10. Hook name: UXActionParityViolationBlocked.
E-3.10. Semantics: noncanonical action attempted by shell.
E-3 retention: follows data class and tenant compliance pack.
E-3 anti-pattern: rendering fallback with no machine-readable reason.

## F. Migration
Migration is declaration-first, adapter-second, enforcement-third. Existing microservices first
declare their supported device, workspace, collar-color, tenure, locale, and accommodation matrices.
They then add runtime adapters for supported profiles. The gate remains advisory until the registry
and first parity fixtures ship, then becomes blocker for new user-facing surfaces.

### F-1. Migration waves
F-1.01. Migration wave: wave-0.
F-1.01. Scope: author registry schemas and profile vocabulary; no product behavior changes.
F-1 verification: profile registry validates and changed surfaces pass action parity tests.
F-1 rollback: disable only the new profile row and route to declared fallback.
F-1 anti-pattern: shipping adapter code before declaring registry ownership.
F-1.02. Migration wave: wave-1.
F-1.02. Scope: inventory every user-facing microservice and mark support/fallback/not-supported.
F-1 verification: profile registry validates and changed surfaces pass action parity tests.
F-1 rollback: disable only the new profile row and route to declared fallback.
F-1 anti-pattern: shipping adapter code before declaring registry ownership.
F-1.03. Migration wave: wave-2.
F-1.03. Scope: ship shared projection kernel and action parity fixtures.
F-1 verification: profile registry validates and changed surfaces pass action parity tests.
F-1 rollback: disable only the new profile row and route to declared fallback.
F-1 anti-pattern: shipping adapter code before declaring registry ownership.
F-1.04. Migration wave: wave-3.
F-1.04. Scope: add mobile, tablet, desktop, and accessibility adapters for top-level work surfaces.
F-1 verification: profile registry validates and changed surfaces pass action parity tests.
F-1 rollback: disable only the new profile row and route to declared fallback.
F-1 anti-pattern: shipping adapter code before declaring registry ownership.
F-1.05. Migration wave: wave-4.
F-1.05. Scope: add field, vehicle, warehouse, wearable, and voice-first adapters.
F-1 verification: profile registry validates and changed surfaces pass action parity tests.
F-1 rollback: disable only the new profile row and route to declared fallback.
F-1 anti-pattern: shipping adapter code before declaring registry ownership.
F-1.06. Migration wave: wave-5.
F-1.06. Scope: promote gates from advisory to blocker for new surfaces.
F-1 verification: profile registry validates and changed surfaces pass action parity tests.
F-1 rollback: disable only the new profile row and route to declared fallback.
F-1 anti-pattern: shipping adapter code before declaring registry ownership.
F-1.07. Migration wave: wave-6.
F-1.07. Scope: backfill legacy surfaces and retire unregistered shells.
F-1 verification: profile registry validates and changed surfaces pass action parity tests.
F-1 rollback: disable only the new profile row and route to declared fallback.
F-1 anti-pattern: shipping adapter code before declaring registry ownership.

### F-2. Existing microservice matrix declaration
F-2.01. Microservice: analytics.
F-2.01. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.02. Microservice: api-gateway.
F-2.02. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.03. Microservice: application.
F-2.03. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.04. Microservice: audit-chain.
F-2.04. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.05. Microservice: calendar.
F-2.05. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.06. Microservice: cell.
F-2.06. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.07. Microservice: cloud-iac.
F-2.07. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.08. Microservice: cloud-k8s.
F-2.08. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.09. Microservice: cloud-secrets.
F-2.09. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.10. Microservice: comms-email.
F-2.10. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.11. Microservice: community.
F-2.11. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.12. Microservice: compliance.
F-2.12. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.13. Microservice: connect.
F-2.13. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.14. Microservice: consent-graph.
F-2.14. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.15. Microservice: crm.
F-2.15. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.16. Microservice: developer-sdk.
F-2.16. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.17. Microservice: docs.
F-2.17. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.18. Microservice: drive.
F-2.18. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.19. Microservice: feature-flags.
F-2.19. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.20. Microservice: finops-portal.
F-2.20. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.21. Microservice: forms.
F-2.21. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.22. Microservice: foundry.
F-2.22. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, in-vehicle, voice-only.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.23. Microservice: global-trade.
F-2.23. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, ruggedized-handheld, voice-only, ar-overlay.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.24. Microservice: governance.
F-2.24. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.25. Microservice: identity.
F-2.25. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.26. Microservice: intelligence.
F-2.26. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.27. Microservice: mail.
F-2.27. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.28. Microservice: marketplace.
F-2.28. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.29. Microservice: meet.
F-2.29. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.30. Microservice: messenger.
F-2.30. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.31. Microservice: network.
F-2.31. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.32. Microservice: notes.
F-2.32. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.33. Microservice: observability.
F-2.33. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, in-vehicle, voice-only.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.34. Microservice: ontology.
F-2.34. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.35. Microservice: ops-dashboard-control-center.
F-2.35. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, in-vehicle, voice-only.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.36. Microservice: payments.
F-2.36. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.37. Microservice: plant-maintenance.
F-2.37. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, ruggedized-handheld, voice-only, ar-overlay.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.38. Microservice: plugin-app-store.
F-2.38. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.39. Microservice: production-planning.
F-2.39. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, ruggedized-handheld, voice-only, ar-overlay.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.40. Microservice: quality-management.
F-2.40. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, ruggedized-handheld, voice-only, ar-overlay.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.41. Microservice: real-estate.
F-2.41. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.42. Microservice: recordings.
F-2.42. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.43. Microservice: sheets.
F-2.43. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.44. Microservice: shorts.
F-2.44. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.45. Microservice: sites.
F-2.45. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.46. Microservice: slides.
F-2.46. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.47. Microservice: social.
F-2.47. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.48. Microservice: supply-chain-planning.
F-2.48. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, ruggedized-handheld, voice-only, ar-overlay.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.49. Microservice: tasks.
F-2.49. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, wearable.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.50. Microservice: tenancy.
F-2.50. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.51. Microservice: translate.
F-2.51. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.52. Microservice: treasury.
F-2.52. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.53. Microservice: warehouse.
F-2.53. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, ruggedized-handheld, voice-only, ar-overlay.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.54. Microservice: workflow-engine.
F-2.54. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, in-vehicle, voice-only.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.55. Microservice: workflow-studio.
F-2.55. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility, in-vehicle, voice-only.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

F-2.56. Microservice: workplace-integration.
F-2.56. Initial device declaration: laptop, desktop, tablet, phone, large-font-accessibility.
F-2 workspace declaration: office plus any service-specific non-office contexts listed in product PRD.
F-2 collar declaration: knowledge-worker by default; add trades, care, agriculture, specialized, or service when product stories require it.
F-2 locale declaration: canonical base plus Korea localization pack before FD-001 claim expansion.
F-2 disability declaration: large-font, screen-reader, voice-only, and switch support status explicit.
F-2 migration status: declaration required before new user-facing implementation packets.

## G. References

### G-1. Internal references
G-1.01. docs/standards/documentation-rigor.md.
G-1.02. docs/standards/doc-style.md.
G-1.03. docs/templates/adr-template-v2.md.
G-1.04. docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md.
G-1.05. docs/decisions/ADR-0306-disaster-mode-cell-resilience.md.
G-1.06. docs/decisions/ADR-0317-role-based-projection-unified-ux-shell.md (in-flight authority named by brief; file absent at author time).

### G-2. External references and industry precedents
G-2.01. Apple Required Device Capabilities: https://developer.apple.com/support/required-device-capabilities/
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.02. Apple Accessibility HIG: https://developer.apple.com/design/human-interface-guidelines/accessibility
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.03. Apple watchOS design guidance: https://developer.apple.com/design/human-interface-guidelines/designing-for-watchos
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.04. Android uses-feature manifest element: https://developer.android.com/guide/topics/manifest/uses-feature-element
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.05. Google Play filters: https://developer.android.com/google/play/filters
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.06. Samsung Galaxy Watch sensors: https://developer.samsung.com/tizen/Galaxy-Watch/get-started/using-device-sensors.html
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.07. RealWear industrial smartglasses: https://www.realwear.com/
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.08. RealWear speech recognizer: https://developer.realwear.com/docs/developer-examples/speech-recognizer/
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.09. Walmart Me@Walmart associate app: https://corporate.walmart.com/news/2021/06/03/walmart-unveils-all-in-one-associate-app-me-walmart-and-gives-740-000-associates-a-new-samsung-smartphone
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.10. Walmart Store Assist workflows: https://developer.walmart.com/walmart-commerce-technologies/docs/store-assist-workflows
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.11. UPS UPSNav / ORION driver device: https://about.ups.com/us/en/newsroom/press-releases/innovation-driven/ups-deploys-purpose-built-navigation-for-ups-service-personnel.html
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.12. Kaiser Permanente mobile app experience: https://insider.kaiserpermanente.org/your-care-connected-discover-the-new-kp-mobile-app-experience/
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.13. Amazon Mechanical Turk qualifications: https://docs.aws.amazon.com/AWSMechTurk/latest/AWSMechanicalTurkRequester/SelectingEligibleWorkers.html
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.
G-2.14. Lyft driver earnings tools: https://www.lyft.com/driver/earnings
G-2 use: cited as precedent for explicit device, role, worker, workspace, accessibility, or eligibility adaptation.

### G-3. Precedent mapping by primitive family
G-3.01. Primitive family: device capability filtering.
G-3.01. Precedent A: Apple Required Device Capabilities.
G-3.01. Precedent B: Android uses-feature manifest element.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.02. Primitive family: market/device distribution filtering.
G-3.02. Precedent A: Google Play filters.
G-3.02. Precedent B: Apple Required Device Capabilities.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.03. Primitive family: frontline associate workflow.
G-3.03. Precedent A: Walmart Me@Walmart associate app.
G-3.03. Precedent B: Walmart Store Assist workflows.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.04. Primitive family: driver route workflow.
G-3.04. Precedent A: UPS UPSNav / ORION driver device.
G-3.04. Precedent B: Lyft driver earnings tools.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.05. Primitive family: clinical mobile workflow.
G-3.05. Precedent A: Kaiser Permanente mobile app experience.
G-3.05. Precedent B: Apple Accessibility HIG.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.06. Primitive family: worker eligibility.
G-3.06. Precedent A: Amazon Mechanical Turk qualifications.
G-3.06. Precedent B: ADR-0243 Cedar universal gate.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.07. Primitive family: wearable sensors.
G-3.07. Precedent A: Samsung Galaxy Watch sensors.
G-3.07. Precedent B: Apple watchOS design guidance.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.08. Primitive family: voice smartglasses.
G-3.08. Precedent A: RealWear industrial smartglasses.
G-3.08. Precedent B: RealWear speech recognizer.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.09. Primitive family: offline and disaster.
G-3.09. Precedent A: ADR-0306 disaster-mode cell resilience.
G-3.09. Precedent B: UPS route-local workflow precedent.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

G-3.10. Primitive family: minor and guardian overlays.
G-3.10. Precedent A: ADR-0292 minor-user doctrine.
G-3.10. Precedent B: Kaiser mobile care workflow precedent.
G-3 inference: Oyatie adopts the shared pattern, not vendor-specific product behavior.

## H. Change Log and Naming Justifications

### H-1. Change log
H-1.01. 2026-05-20: Initial ADR authored from /tmp/codex-brief-adr-0318-collar-color.md.
H-1.02. 2026-05-20: ADR-0317 was absent at the specified path; cited as in-flight authority by id.
H-1.03. 2026-05-20: External device, field, clinical, worker, driver, and wearable precedents recorded.
H-1.04. 2026-05-20: Device, workspace, collar, tenure, locale, and disability matrices included.

### H-2. Naming justifications
H-2.00. Name: collar-color.
H-2.00. Justification: Preserves the established labor-market term while binding it to projection, not hierarchy.
H-2 anti-pattern: name that implies separate products for separate worker classes.
H-2.01. Name: workspace-universality.
H-2.01. Justification: Names the invariant that office and non-office work share the same platform.
H-2 anti-pattern: name that implies separate products for separate worker classes.
H-2.02. Name: device-profile.
H-2.02. Justification: Matches Apple and Google capability-filter precedent and keeps hardware facts explicit.
H-2 anti-pattern: name that implies separate products for separate worker classes.
H-2.03. Name: workspace-profile.
H-2.03. Justification: Separates physical and operational context from job role.
H-2 anti-pattern: name that implies separate products for separate worker classes.
H-2.04. Name: collar-shell.
H-2.04. Justification: Makes collar-specific vocabulary a shell, not a forked product.
H-2 anti-pattern: name that implies separate products for separate worker classes.
H-2.05. Name: tenure-arc.
H-2.05. Justification: Captures that onboarding and expert mode are points on one learning curve.
H-2 anti-pattern: name that implies separate products for separate worker classes.
H-2.06. Name: locale-aware.
H-2.06. Justification: Signals language, script, jurisdiction, units, and legal pack overlays together.
H-2 anti-pattern: name that implies separate products for separate worker classes.
H-2.07. Name: accommodation-profile.
H-2.07. Justification: Treats accessibility as first-class capability equivalence.
H-2 anti-pattern: name that implies separate products for separate worker classes.
H-2.08. Name: role-projection-transfer.
H-2.08. Justification: Connects this ADR to ADR-0317 unified shell gesture transfer.
H-2 anti-pattern: name that implies separate products for separate worker classes.

### H-3. Explicit non-goals
H-3.01. Non-goal: This ADR does not require every microservice to implement every adapter immediately.
H-3 boundary: implementation MUST preserve tenant scope, policy gates, and audit semantics.
H-3.02. Non-goal: This ADR does not permit a microservice to omit explicit unsupported-profile declarations.
H-3 boundary: implementation MUST preserve tenant scope, policy gates, and audit semantics.
H-3.03. Non-goal: This ADR does not replace ADR-0317 unified shell action vocabulary.
H-3 boundary: implementation MUST preserve tenant scope, policy gates, and audit semantics.
H-3.04. Non-goal: This ADR does not weaken ADR-0292 minor-user gates for simplified UX.
H-3 boundary: implementation MUST preserve tenant scope, policy gates, and audit semantics.
H-3.05. Non-goal: This ADR does not weaken ADR-0306 offline sync or disaster-mode boundaries.
H-3 boundary: implementation MUST preserve tenant scope, policy gates, and audit semantics.
H-3.06. Non-goal: This ADR does not make accessibility a best-effort mode.
H-3 boundary: implementation MUST preserve tenant scope, policy gates, and audit semantics.
H-3.07. Non-goal: This ADR does not introduce a separate frontline product line.
H-3 boundary: implementation MUST preserve tenant scope, policy gates, and audit semantics.
H-3.08. Non-goal: This ADR does not turn device capability detection into user tracking.
H-3 boundary: implementation MUST preserve tenant scope, policy gates, and audit semantics.

## Appendix A. Universality Acceptance Checklist
Appendix-A.0001. Device laptop with workspace office: declare support, fallback, and action parity.
Appendix-A.0001. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0002. Device laptop with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0002. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0003. Device laptop with workspace field: declare support, fallback, and action parity.
Appendix-A.0003. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0004. Device laptop with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0004. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0005. Device laptop with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0005. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0006. Device laptop with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0006. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0007. Device laptop with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0007. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0008. Device laptop with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0008. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0009. Device laptop with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0009. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0010. Device laptop with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0010. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0011. Device laptop with workspace home: declare support, fallback, and action parity.
Appendix-A.0011. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0012. Device laptop with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0012. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0013. Device laptop with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0013. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0014. Device laptop with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0014. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0015. Device laptop with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0015. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0016. Device laptop with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0016. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0017. Device desktop with workspace office: declare support, fallback, and action parity.
Appendix-A.0017. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0018. Device desktop with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0018. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0019. Device desktop with workspace field: declare support, fallback, and action parity.
Appendix-A.0019. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0020. Device desktop with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0020. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0021. Device desktop with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0021. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0022. Device desktop with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0022. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0023. Device desktop with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0023. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0024. Device desktop with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0024. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0025. Device desktop with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0025. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0026. Device desktop with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0026. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0027. Device desktop with workspace home: declare support, fallback, and action parity.
Appendix-A.0027. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0028. Device desktop with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0028. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0029. Device desktop with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0029. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0030. Device desktop with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0030. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0031. Device desktop with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0031. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0032. Device desktop with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0032. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0033. Device tablet with workspace office: declare support, fallback, and action parity.
Appendix-A.0033. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0034. Device tablet with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0034. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0035. Device tablet with workspace field: declare support, fallback, and action parity.
Appendix-A.0035. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0036. Device tablet with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0036. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0037. Device tablet with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0037. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0038. Device tablet with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0038. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0039. Device tablet with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0039. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0040. Device tablet with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0040. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0041. Device tablet with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0041. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0042. Device tablet with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0042. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0043. Device tablet with workspace home: declare support, fallback, and action parity.
Appendix-A.0043. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0044. Device tablet with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0044. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0045. Device tablet with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0045. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0046. Device tablet with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0046. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0047. Device tablet with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0047. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0048. Device tablet with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0048. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0049. Device phone with workspace office: declare support, fallback, and action parity.
Appendix-A.0049. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0050. Device phone with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0050. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0051. Device phone with workspace field: declare support, fallback, and action parity.
Appendix-A.0051. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0052. Device phone with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0052. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0053. Device phone with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0053. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0054. Device phone with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0054. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0055. Device phone with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0055. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0056. Device phone with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0056. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0057. Device phone with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0057. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0058. Device phone with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0058. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0059. Device phone with workspace home: declare support, fallback, and action parity.
Appendix-A.0059. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0060. Device phone with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0060. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0061. Device phone with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0061. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0062. Device phone with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0062. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0063. Device phone with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0063. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0064. Device phone with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0064. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0065. Device wearable with workspace office: declare support, fallback, and action parity.
Appendix-A.0065. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0066. Device wearable with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0066. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0067. Device wearable with workspace field: declare support, fallback, and action parity.
Appendix-A.0067. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0068. Device wearable with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0068. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0069. Device wearable with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0069. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0070. Device wearable with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0070. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0071. Device wearable with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0071. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0072. Device wearable with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0072. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0073. Device wearable with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0073. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0074. Device wearable with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0074. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0075. Device wearable with workspace home: declare support, fallback, and action parity.
Appendix-A.0075. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0076. Device wearable with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0076. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0077. Device wearable with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0077. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0078. Device wearable with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0078. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0079. Device wearable with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0079. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0080. Device wearable with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0080. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0081. Device ar-overlay with workspace office: declare support, fallback, and action parity.
Appendix-A.0081. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0082. Device ar-overlay with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0082. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0083. Device ar-overlay with workspace field: declare support, fallback, and action parity.
Appendix-A.0083. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0084. Device ar-overlay with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0084. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0085. Device ar-overlay with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0085. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0086. Device ar-overlay with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0086. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0087. Device ar-overlay with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0087. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0088. Device ar-overlay with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0088. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0089. Device ar-overlay with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0089. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0090. Device ar-overlay with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0090. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0091. Device ar-overlay with workspace home: declare support, fallback, and action parity.
Appendix-A.0091. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0092. Device ar-overlay with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0092. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0093. Device ar-overlay with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0093. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0094. Device ar-overlay with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0094. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0095. Device ar-overlay with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0095. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0096. Device ar-overlay with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0096. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0097. Device voice-only with workspace office: declare support, fallback, and action parity.
Appendix-A.0097. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0098. Device voice-only with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0098. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0099. Device voice-only with workspace field: declare support, fallback, and action parity.
Appendix-A.0099. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0100. Device voice-only with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0100. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0101. Device voice-only with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0101. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0102. Device voice-only with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0102. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0103. Device voice-only with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0103. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0104. Device voice-only with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0104. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0105. Device voice-only with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0105. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0106. Device voice-only with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0106. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0107. Device voice-only with workspace home: declare support, fallback, and action parity.
Appendix-A.0107. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0108. Device voice-only with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0108. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0109. Device voice-only with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0109. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0110. Device voice-only with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0110. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0111. Device voice-only with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0111. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0112. Device voice-only with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0112. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0113. Device ruggedized-handheld with workspace office: declare support, fallback, and action parity.
Appendix-A.0113. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0114. Device ruggedized-handheld with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0114. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0115. Device ruggedized-handheld with workspace field: declare support, fallback, and action parity.
Appendix-A.0115. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0116. Device ruggedized-handheld with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0116. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0117. Device ruggedized-handheld with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0117. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0118. Device ruggedized-handheld with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0118. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0119. Device ruggedized-handheld with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0119. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0120. Device ruggedized-handheld with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0120. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0121. Device ruggedized-handheld with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0121. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0122. Device ruggedized-handheld with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0122. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0123. Device ruggedized-handheld with workspace home: declare support, fallback, and action parity.
Appendix-A.0123. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0124. Device ruggedized-handheld with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0124. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0125. Device ruggedized-handheld with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0125. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0126. Device ruggedized-handheld with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0126. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0127. Device ruggedized-handheld with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0127. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0128. Device ruggedized-handheld with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0128. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0129. Device in-vehicle with workspace office: declare support, fallback, and action parity.
Appendix-A.0129. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0130. Device in-vehicle with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0130. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0131. Device in-vehicle with workspace field: declare support, fallback, and action parity.
Appendix-A.0131. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0132. Device in-vehicle with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0132. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0133. Device in-vehicle with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0133. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0134. Device in-vehicle with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0134. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0135. Device in-vehicle with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0135. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0136. Device in-vehicle with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0136. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0137. Device in-vehicle with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0137. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0138. Device in-vehicle with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0138. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0139. Device in-vehicle with workspace home: declare support, fallback, and action parity.
Appendix-A.0139. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0140. Device in-vehicle with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0140. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0141. Device in-vehicle with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0141. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0142. Device in-vehicle with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0142. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0143. Device in-vehicle with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0143. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0144. Device in-vehicle with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0144. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0145. Device in-cockpit with workspace office: declare support, fallback, and action parity.
Appendix-A.0145. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0146. Device in-cockpit with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0146. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0147. Device in-cockpit with workspace field: declare support, fallback, and action parity.
Appendix-A.0147. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0148. Device in-cockpit with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0148. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0149. Device in-cockpit with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0149. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0150. Device in-cockpit with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0150. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0151. Device in-cockpit with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0151. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0152. Device in-cockpit with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0152. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0153. Device in-cockpit with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0153. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0154. Device in-cockpit with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0154. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0155. Device in-cockpit with workspace home: declare support, fallback, and action parity.
Appendix-A.0155. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0156. Device in-cockpit with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0156. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0157. Device in-cockpit with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0157. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0158. Device in-cockpit with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0158. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0159. Device in-cockpit with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0159. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0160. Device in-cockpit with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0160. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0161. Device large-font-accessibility with workspace office: declare support, fallback, and action parity.
Appendix-A.0161. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0162. Device large-font-accessibility with workspace work-from-home: declare support, fallback, and action parity.
Appendix-A.0162. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0163. Device large-font-accessibility with workspace field: declare support, fallback, and action parity.
Appendix-A.0163. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0164. Device large-font-accessibility with workspace shop-floor: declare support, fallback, and action parity.
Appendix-A.0164. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0165. Device large-font-accessibility with workspace vehicle: declare support, fallback, and action parity.
Appendix-A.0165. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0166. Device large-font-accessibility with workspace patient-bedside: declare support, fallback, and action parity.
Appendix-A.0166. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0167. Device large-font-accessibility with workspace construction-site: declare support, fallback, and action parity.
Appendix-A.0167. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0168. Device large-font-accessibility with workspace kitchen: declare support, fallback, and action parity.
Appendix-A.0168. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0169. Device large-font-accessibility with workspace warehouse: declare support, fallback, and action parity.
Appendix-A.0169. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0170. Device large-font-accessibility with workspace on-the-road: declare support, fallback, and action parity.
Appendix-A.0170. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0171. Device large-font-accessibility with workspace home: declare support, fallback, and action parity.
Appendix-A.0171. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0172. Device large-font-accessibility with workspace outdoors: declare support, fallback, and action parity.
Appendix-A.0172. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0173. Device large-font-accessibility with workspace cockpit: declare support, fallback, and action parity.
Appendix-A.0173. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0174. Device large-font-accessibility with workspace operating-room: declare support, fallback, and action parity.
Appendix-A.0174. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0175. Device large-font-accessibility with workspace retail-floor: declare support, fallback, and action parity.
Appendix-A.0175. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.
Appendix-A.0176. Device large-font-accessibility with workspace classroom: declare support, fallback, and action parity.
Appendix-A.0176. Evidence: profile fixture includes ADR-0317 action ids and ADR-0306 offline posture.

## Appendix B. Collar and Tenure Transfer Checklist
Appendix-B.01.day-zero. Collar shell knowledge-worker at tenure day-zero: training vocabulary transfers unchanged.
Appendix-B.01.day-zero. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.01.day-seven. Collar shell knowledge-worker at tenure day-seven: training vocabulary transfers unchanged.
Appendix-B.01.day-seven. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.01.day-thirty. Collar shell knowledge-worker at tenure day-thirty: training vocabulary transfers unchanged.
Appendix-B.01.day-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.01.year-one. Collar shell knowledge-worker at tenure year-one: training vocabulary transfers unchanged.
Appendix-B.01.year-one. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.01.year-ten. Collar shell knowledge-worker at tenure year-ten: training vocabulary transfers unchanged.
Appendix-B.01.year-ten. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.01.year-thirty. Collar shell knowledge-worker at tenure year-thirty: training vocabulary transfers unchanged.
Appendix-B.01.year-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.02.day-zero. Collar shell trades at tenure day-zero: training vocabulary transfers unchanged.
Appendix-B.02.day-zero. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.02.day-seven. Collar shell trades at tenure day-seven: training vocabulary transfers unchanged.
Appendix-B.02.day-seven. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.02.day-thirty. Collar shell trades at tenure day-thirty: training vocabulary transfers unchanged.
Appendix-B.02.day-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.02.year-one. Collar shell trades at tenure year-one: training vocabulary transfers unchanged.
Appendix-B.02.year-one. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.02.year-ten. Collar shell trades at tenure year-ten: training vocabulary transfers unchanged.
Appendix-B.02.year-ten. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.02.year-thirty. Collar shell trades at tenure year-thirty: training vocabulary transfers unchanged.
Appendix-B.02.year-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.03.day-zero. Collar shell care-service at tenure day-zero: training vocabulary transfers unchanged.
Appendix-B.03.day-zero. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.03.day-seven. Collar shell care-service at tenure day-seven: training vocabulary transfers unchanged.
Appendix-B.03.day-seven. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.03.day-thirty. Collar shell care-service at tenure day-thirty: training vocabulary transfers unchanged.
Appendix-B.03.day-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.03.year-one. Collar shell care-service at tenure year-one: training vocabulary transfers unchanged.
Appendix-B.03.year-one. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.03.year-ten. Collar shell care-service at tenure year-ten: training vocabulary transfers unchanged.
Appendix-B.03.year-ten. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.03.year-thirty. Collar shell care-service at tenure year-thirty: training vocabulary transfers unchanged.
Appendix-B.03.year-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.04.day-zero. Collar shell agricultural at tenure day-zero: training vocabulary transfers unchanged.
Appendix-B.04.day-zero. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.04.day-seven. Collar shell agricultural at tenure day-seven: training vocabulary transfers unchanged.
Appendix-B.04.day-seven. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.04.day-thirty. Collar shell agricultural at tenure day-thirty: training vocabulary transfers unchanged.
Appendix-B.04.day-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.04.year-one. Collar shell agricultural at tenure year-one: training vocabulary transfers unchanged.
Appendix-B.04.year-one. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.04.year-ten. Collar shell agricultural at tenure year-ten: training vocabulary transfers unchanged.
Appendix-B.04.year-ten. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.04.year-thirty. Collar shell agricultural at tenure year-thirty: training vocabulary transfers unchanged.
Appendix-B.04.year-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.05.day-zero. Collar shell specialized-trade at tenure day-zero: training vocabulary transfers unchanged.
Appendix-B.05.day-zero. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.05.day-seven. Collar shell specialized-trade at tenure day-seven: training vocabulary transfers unchanged.
Appendix-B.05.day-seven. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.05.day-thirty. Collar shell specialized-trade at tenure day-thirty: training vocabulary transfers unchanged.
Appendix-B.05.day-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.05.year-one. Collar shell specialized-trade at tenure year-one: training vocabulary transfers unchanged.
Appendix-B.05.year-one. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.05.year-ten. Collar shell specialized-trade at tenure year-ten: training vocabulary transfers unchanged.
Appendix-B.05.year-ten. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.05.year-thirty. Collar shell specialized-trade at tenure year-thirty: training vocabulary transfers unchanged.
Appendix-B.05.year-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.06.day-zero. Collar shell service-industry at tenure day-zero: training vocabulary transfers unchanged.
Appendix-B.06.day-zero. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.06.day-seven. Collar shell service-industry at tenure day-seven: training vocabulary transfers unchanged.
Appendix-B.06.day-seven. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.06.day-thirty. Collar shell service-industry at tenure day-thirty: training vocabulary transfers unchanged.
Appendix-B.06.day-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.06.year-one. Collar shell service-industry at tenure year-one: training vocabulary transfers unchanged.
Appendix-B.06.year-one. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.06.year-ten. Collar shell service-industry at tenure year-ten: training vocabulary transfers unchanged.
Appendix-B.06.year-ten. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.
Appendix-B.06.year-thirty. Collar shell service-industry at tenure year-thirty: training vocabulary transfers unchanged.
Appendix-B.06.year-thirty. Evidence: approve, assign, attach, escalate, and close actions retain canonical ids.

## Appendix C. Locale and Accommodation Checklist
Appendix-C.01.voice-only. Locale US-English with accommodation voice-only: render, navigate, confirm, and audit equivalently.
Appendix-C.01.voice-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.01.single-switch. Locale US-English with accommodation single-switch: render, navigate, confirm, and audit equivalently.
Appendix-C.01.single-switch. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.01.screen-reader-only. Locale US-English with accommodation screen-reader-only: render, navigate, confirm, and audit equivalently.
Appendix-C.01.screen-reader-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.01.aac. Locale US-English with accommodation aac: render, navigate, confirm, and audit equivalently.
Appendix-C.01.aac. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.01.post-stroke. Locale US-English with accommodation post-stroke: render, navigate, confirm, and audit equivalently.
Appendix-C.01.post-stroke. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.01.post-trauma. Locale US-English with accommodation post-trauma: render, navigate, confirm, and audit equivalently.
Appendix-C.01.post-trauma. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.01.cognitive-impairment. Locale US-English with accommodation cognitive-impairment: render, navigate, confirm, and audit equivalently.
Appendix-C.01.cognitive-impairment. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.02.voice-only. Locale KR-Korean with accommodation voice-only: render, navigate, confirm, and audit equivalently.
Appendix-C.02.voice-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.02.single-switch. Locale KR-Korean with accommodation single-switch: render, navigate, confirm, and audit equivalently.
Appendix-C.02.single-switch. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.02.screen-reader-only. Locale KR-Korean with accommodation screen-reader-only: render, navigate, confirm, and audit equivalently.
Appendix-C.02.screen-reader-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.02.aac. Locale KR-Korean with accommodation aac: render, navigate, confirm, and audit equivalently.
Appendix-C.02.aac. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.02.post-stroke. Locale KR-Korean with accommodation post-stroke: render, navigate, confirm, and audit equivalently.
Appendix-C.02.post-stroke. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.02.post-trauma. Locale KR-Korean with accommodation post-trauma: render, navigate, confirm, and audit equivalently.
Appendix-C.02.post-trauma. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.02.cognitive-impairment. Locale KR-Korean with accommodation cognitive-impairment: render, navigate, confirm, and audit equivalently.
Appendix-C.02.cognitive-impairment. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.03.voice-only. Locale JP-Japanese with accommodation voice-only: render, navigate, confirm, and audit equivalently.
Appendix-C.03.voice-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.03.single-switch. Locale JP-Japanese with accommodation single-switch: render, navigate, confirm, and audit equivalently.
Appendix-C.03.single-switch. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.03.screen-reader-only. Locale JP-Japanese with accommodation screen-reader-only: render, navigate, confirm, and audit equivalently.
Appendix-C.03.screen-reader-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.03.aac. Locale JP-Japanese with accommodation aac: render, navigate, confirm, and audit equivalently.
Appendix-C.03.aac. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.03.post-stroke. Locale JP-Japanese with accommodation post-stroke: render, navigate, confirm, and audit equivalently.
Appendix-C.03.post-stroke. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.03.post-trauma. Locale JP-Japanese with accommodation post-trauma: render, navigate, confirm, and audit equivalently.
Appendix-C.03.post-trauma. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.03.cognitive-impairment. Locale JP-Japanese with accommodation cognitive-impairment: render, navigate, confirm, and audit equivalently.
Appendix-C.03.cognitive-impairment. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.04.voice-only. Locale EU-Multilingual with accommodation voice-only: render, navigate, confirm, and audit equivalently.
Appendix-C.04.voice-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.04.single-switch. Locale EU-Multilingual with accommodation single-switch: render, navigate, confirm, and audit equivalently.
Appendix-C.04.single-switch. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.04.screen-reader-only. Locale EU-Multilingual with accommodation screen-reader-only: render, navigate, confirm, and audit equivalently.
Appendix-C.04.screen-reader-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.04.aac. Locale EU-Multilingual with accommodation aac: render, navigate, confirm, and audit equivalently.
Appendix-C.04.aac. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.04.post-stroke. Locale EU-Multilingual with accommodation post-stroke: render, navigate, confirm, and audit equivalently.
Appendix-C.04.post-stroke. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.04.post-trauma. Locale EU-Multilingual with accommodation post-trauma: render, navigate, confirm, and audit equivalently.
Appendix-C.04.post-trauma. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.04.cognitive-impairment. Locale EU-Multilingual with accommodation cognitive-impairment: render, navigate, confirm, and audit equivalently.
Appendix-C.04.cognitive-impairment. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.05.voice-only. Locale Arabic-RTL with accommodation voice-only: render, navigate, confirm, and audit equivalently.
Appendix-C.05.voice-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.05.single-switch. Locale Arabic-RTL with accommodation single-switch: render, navigate, confirm, and audit equivalently.
Appendix-C.05.single-switch. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.05.screen-reader-only. Locale Arabic-RTL with accommodation screen-reader-only: render, navigate, confirm, and audit equivalently.
Appendix-C.05.screen-reader-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.05.aac. Locale Arabic-RTL with accommodation aac: render, navigate, confirm, and audit equivalently.
Appendix-C.05.aac. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.05.post-stroke. Locale Arabic-RTL with accommodation post-stroke: render, navigate, confirm, and audit equivalently.
Appendix-C.05.post-stroke. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.05.post-trauma. Locale Arabic-RTL with accommodation post-trauma: render, navigate, confirm, and audit equivalently.
Appendix-C.05.post-trauma. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.05.cognitive-impairment. Locale Arabic-RTL with accommodation cognitive-impairment: render, navigate, confirm, and audit equivalently.
Appendix-C.05.cognitive-impairment. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.06.voice-only. Locale Hindi-Indic with accommodation voice-only: render, navigate, confirm, and audit equivalently.
Appendix-C.06.voice-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.06.single-switch. Locale Hindi-Indic with accommodation single-switch: render, navigate, confirm, and audit equivalently.
Appendix-C.06.single-switch. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.06.screen-reader-only. Locale Hindi-Indic with accommodation screen-reader-only: render, navigate, confirm, and audit equivalently.
Appendix-C.06.screen-reader-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.06.aac. Locale Hindi-Indic with accommodation aac: render, navigate, confirm, and audit equivalently.
Appendix-C.06.aac. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.06.post-stroke. Locale Hindi-Indic with accommodation post-stroke: render, navigate, confirm, and audit equivalently.
Appendix-C.06.post-stroke. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.06.post-trauma. Locale Hindi-Indic with accommodation post-trauma: render, navigate, confirm, and audit equivalently.
Appendix-C.06.post-trauma. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.06.cognitive-impairment. Locale Hindi-Indic with accommodation cognitive-impairment: render, navigate, confirm, and audit equivalently.
Appendix-C.06.cognitive-impairment. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.07.voice-only. Locale Spanish-LatAm with accommodation voice-only: render, navigate, confirm, and audit equivalently.
Appendix-C.07.voice-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.07.single-switch. Locale Spanish-LatAm with accommodation single-switch: render, navigate, confirm, and audit equivalently.
Appendix-C.07.single-switch. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.07.screen-reader-only. Locale Spanish-LatAm with accommodation screen-reader-only: render, navigate, confirm, and audit equivalently.
Appendix-C.07.screen-reader-only. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.07.aac. Locale Spanish-LatAm with accommodation aac: render, navigate, confirm, and audit equivalently.
Appendix-C.07.aac. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.07.post-stroke. Locale Spanish-LatAm with accommodation post-stroke: render, navigate, confirm, and audit equivalently.
Appendix-C.07.post-stroke. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.07.post-trauma. Locale Spanish-LatAm with accommodation post-trauma: render, navigate, confirm, and audit equivalently.
Appendix-C.07.post-trauma. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.
Appendix-C.07.cognitive-impairment. Locale Spanish-LatAm with accommodation cognitive-impairment: render, navigate, confirm, and audit equivalently.
Appendix-C.07.cognitive-impairment. Evidence: screen, voice, and policy fixtures include ADR-0292 where age-sensitive.

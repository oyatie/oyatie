# IP-032 ITSM mobile-itsm

Service: itsm
ChangeSet scope: microservices/itsm/IP-032-mobile-itsm.md
Counterparts displaced: ServiceNow Now Mobile + Now Mobile Agent, Jira Service Management Mobile, Freshservice Mobile
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0263, ADR-0328

## Objective
- O-001: Ship a native ITSM bundle on iOS (Swift) and Android (Kotlin) covering agent + requester roles; no cross-platform JS framework permitted by language policy.
- O-002: Carry push notifications for on-call pages, escalation alerts, ticket-update alerts, and CSAT prompts.
- O-003: Cache runbooks and KB articles offline so a responder can act in a network-degraded SRE scenario.
- O-004: Bind every device to a per-tenant device record so an offboarded device automatically loses access.

## Surface
- S-001: iOS bundle: `frontend/ios/OyatieItsm/` (Swift + SwiftUI; bundle id `dev.oyatie.itsm`).
- S-002: Android bundle: `frontend/android/oyatie-itsm/` (Kotlin + Jetpack Compose; application id `dev.oyatie.itsm`).
- S-003: REST + AsyncAPI endpoints under `/api/v1/itsm/mobile/`.

## Tenant invariants
- T-001: Each device has a `device_id` registered against the tenant + principal at first sign-in.
- T-002: Offboarding the principal revokes the device session within 5 minutes (paid) or end-of-day (demo_trial).
- T-003: Pack overlays gate which fields appear on the device (e.g., HIPAA-protected fields blurred on screenshot).

## Push notification mechanic
- P-001: Apple Push Notification service for iOS; Firebase Cloud Messaging for Android (no Google account requirement for the user, just for transit).
- P-002: Push payloads are content-free; the device fetches the full ticket payload after sign-in.
- P-003: Personal Messenger transport (MLS RFC 9420 per ADR-0246) used for agent-to-agent chat from the device.

## Cedar policy
- C-001: `policy/mobile-itsm-authorization.cedar` default-denies; explicit permits for `mobile.page.acknowledge`, `mobile.ticket.resolve`, `mobile.runbook.read`, `mobile.kiosk.submit`.
- C-002: Permits require `principal.device_id IN tenant.registered_devices` AND `principal.tenant_id == resource.tenant_id`.

## Offline + sync
- OS-001: Runbooks + KB articles cached in encrypted on-device store; SQLite via SQLCipher (iOS Keychain key, Android Keystore key).
- OS-002: Tickets opened offline are queued and synced once connectivity returns; conflict resolution is last-writer-wins with provenance.
- OS-003: Audit events queue and emit when online.

## Tenant-class behavior
- TC-001: demo_trial: mobile API capped at 5000 calls/month per ADR-0331; push delivered best-effort.
- TC-002: paid: per-seat licensing for the agent-mode bundle; requester-mode is free for tenant employees.

## Performance targets
- PT-001: Cold launch under 1.5s on iPhone 15 / Pixel 8.
- PT-002: Push-to-display p95 ≤ 12s end-to-end via APNs/FCM.

## Implementation sequence
- I-001: Author OpenAPI mobile contract slice.
- I-002: Bind Cedar policy; verify device registration flow.
- I-003: Implement iOS bundle (Swift); ship to TestFlight.
- I-004: Implement Android bundle (Kotlin); ship to Internal Track.
- I-005: Wire APNs/FCM via Notifications µservice substrate.
- I-006: Run on-device tests across iPhone 15+, Pixel 8+, Samsung S24+.

## Acceptance evidence
- E-001: XCTest + Espresso suites for both bundles.
- E-002: openslo: mobile_p95_action_latency_ms ≤ 1200 paid.
- E-003: Audit-chain replay for one ack→resolve flow.

## Out of scope
- OoS-001: macOS Catalyst — covered by separate macOS WinUI-equivalent surface.
- OoS-002: Web-based mobile via PWA — out of scope; native only per policy.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: mobile ITSM acknowledgement.
- REST/API anchor: mobile ack route and push callback.
- Policy anchor: policy/mobile-itsm-authorization.cedar.
- SLO/dashboard anchor: mobile ack under 12s.
- Counterpart pressure: ServiceNow ITSM, Jira Service Management, and Freshservice all expose this class of ITSM surface; Oyatie closes the gap with tenant scope, Cedar, audit-chain evidence, and pack overlays.

### Implementation detail that must exist before promotion
- Define the command DTO with tenant_id, principal_id, audience_type, purpose, data_class, and audit_event_class fields.
- Bind the command to a Capability or an adjacent bounded-context action instead of adding a free-form route.
- Evaluate Cedar before any repository write, external provider call, workflow-engine dispatch, or audit success event.
- Emit an ADR-0263 audit event for success and a distinct denial event for policy, budget, residency, or capacity refusal.
- Carry home_cell, jurisdiction_code, and pack ids through the request context before data leaves the home cell.
- Use existing ITSM source files as the first implementation surface: src/domain/mod.rs, src/usecase/mod.rs, src/adapter/mod.rs, and tests/integration.rs.
- Keep source-system identifiers from ServiceNow, Jira, or Freshservice as aliases only; they cannot authorize Oyatie actions.
- Preserve demo_trial and paid behavior from manifest.json; demo caps must be tested separately from paid behavior.
- Add dashboard evidence before calling the feature production-ready.
- Add rollback that disables this capability without disabling incident open, change approval, SLA recompute, or audit publication.

### Acceptance evidence to add
- Unit or integration test proving the clean allow path succeeds for a synthetic tenant.
- Negative test proving cross-tenant access is denied before mutation.
- Negative test proving missing pack/residency context fails closed where the capability touches protected data.
- Contract test or schema validation for the REST/event/RPC surface used by this capability.
- Audit replay check proving one success event is emitted for each successful command.
- Dashboard or OpenSLO check proving latency/error-budget evidence is available.
- Counterpart parity row explaining the ServiceNow/Jira/Freshservice behavior being displaced.
- Residual-risk note if a referenced runtime module, route, or Cedar entity is not yet implemented.

### Counterpart comparison
| Counterpart | Why this IP is not a clone |
|---|---|
| ServiceNow ITSM | mobile incident acknowledgement under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-032-mobile-itsm.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/itsm/IP-032-mobile-itsm.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].

---
id: ADR-CME-001
title: Per-Tenant Signing Key Custody with Rotation Cadence
status: Proposed
date: 2026-05-20
microservice: comms-email
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-comms-email
---

# ADR-CME-001: Per-Tenant Signing Key Custody with Rotation Cadence

## Context

- Comms-email owns transactional send, tenant domain onboarding, DKIM signing, bounce and complaint handling, webhook delivery, suppression lists, inbound receiver, reputation monitoring, and template rendering.
- Existing SVC-ADR-001 pins DKIM rotation cadence for comms-email.
- This ADR expands custody, rotation, selector overlap, emergency revocation, and tenant-level evidence.
- Named pressure CME-P1: tenants need domain-auth evidence for deliverability and audits.
- Named pressure CME-P2: one tenant's signing-key compromise must not affect another tenant.
- Named pressure CME-P3: DNS propagation requires overlap windows rather than instant selector flips.
- Named pressure CME-P4: high-volume send must not call the key store per message if a safe signing lease can serve bursts.
- Named pressure CME-P5: sovereign packs may require signing inside tenant-approved cells.
- Named precedent: AWS SES domain identities isolate sender-domain verification and signing configuration.
- Named precedent: Google Workspace and Microsoft 365 use DKIM selectors with staged activation.
- Named precedent: DKIM RFC 6376 uses DNS-published public keys and message signatures for domain authentication.
- Constraint CME-C1: tenant and sending-domain ownership come from ADR-0244.
- Constraint CME-C2: key creation, selector activation, signing, rotation, revocation, and denial emit evidence per ADR-0263.
- Constraint CME-C3: Cedar gates domain verification, selector activation, signing lease issue, revocation, and replay per ADR-0243.
- Constraint CME-C4: public management APIs follow ADR-0258.
- Constraint CME-C5: private signing keys must not leave OpenBao or HSM-backed custody except through bounded signing leases where permitted.
- Constraint CME-C6: scheduled rotation is annual by default with 14-day overlap.
- Constraint CME-C7: emergency revocation completes within 5 minutes from declaration.
- Constraint CME-C8: signing keys are scoped by tenant, domain, selector, algorithm, and cell.
- Constraint CME-C9: selector records must support DNS TTL and rollback behavior.
- Constraint CME-C10: tenant offboarding must retire keys without deleting audit evidence.
- Mail microservice has a related mail-auth ADR, but comms-email owns transactional-sender custody.
- This ADR is service-local and does not move mailbox mail-auth ownership.
- The decision must preserve the existing annual cadence while adding custody rigor.

## Decision

- Use per-tenant, per-domain DKIM signing keys for comms-email.
- Store signing key custody in OpenBao transit or HSM-backed transit per cell.
- Never store private signing key bytes in application databases.
- Represent each DKIM selector as an immutable selector epoch.
- Use selector format `oya{YYYYMM}{suffix}` for scheduled rotations.
- Keep two active selectors during the 14-day overlap window.
- Use annual scheduled rotation on the first business day of the onboarding anniversary month.
- Use emergency revocation with target completion <= 5 minutes.
- Issue short-lived signing leases to high-volume signing workers only when pack policy permits.
- Keep signing leases at or below 60 seconds.
- Deny signing when no valid selector exists.
- Deny unsigned fallback for all transactional sends.
- Store public DNS records in tenant-domain state.
- Require verified domain ownership before selector activation.
- Require Cedar approval for selector activation.
- Require Cedar approval for signing lease issue.
- Require Cedar approval for emergency revocation.
- Publish DKIM DNS TXT records before selector activation.
- Confirm DNS propagation before switching primary selector.
- Keep retired selectors verifiable for delayed delivery and forensic review.
- Track signing attempts by selector, tenant, and provider adapter.
- Separate transactional sender keys from inbound receiver and webhook keys.
- Separate platform-owned sender domains into a platform tenant.
- Keep tenant-managed external provider adapters mapped to the same custody evidence model.
- Allow algorithm upgrade by selector epoch.
- Default to RSA-2048 for broad receiver compatibility.
- Allow Ed25519 where receiver compatibility and tenant policy permit.
- Keep DMARC alignment checks tied to the selected sender domain.
- Keep bounce and complaint handlers independent from signing key custody.
- Keep suppression list operations independent from signing key custody.
- Make key custody visible in tenant admin diagnostics without exposing secrets.

## Alternatives Considered

### Shared Platform DKIM Key

- Pros: simplest signing path.
- Pros: lowest key-count operations.
- Pros: easiest DNS onboarding for platform-managed domains.
- Cons: one compromise affects many tenants.
- Cons: tenants cannot prove isolated custody.
- Cons: emergency revocation becomes platform-wide.
- Rejected because tenant blast radius is unacceptable.

### Tenant-Uploaded Private Keys

- Pros: tenants can bring existing mail infrastructure posture.
- Pros: migration can be quick for sophisticated tenants.
- Pros: key ownership feels explicit to tenants.
- Cons: uploaded secrets create exfiltration and handling risk.
- Cons: key quality and algorithm support vary.
- Cons: HSM custody evidence is weak for imported material.
- Rejected as default; future external key mode needs separate ADR.

### Provider-Managed Signing Only

- Pros: SES, Mailgun, or Postal can handle signing details.
- Pros: fewer in-house signing components.
- Pros: deliverability tooling may be mature.
- Cons: custody evidence becomes provider-specific.
- Cons: sovereign packs may reject cross-region signing.
- Cons: provider migration changes tenant evidence.
- Rejected because comms-email must own a portable custody model.

### Quarterly Rotation

- Pros: reduces maximum key age.
- Pros: gives more frequent rotation drills.
- Pros: may satisfy stricter tenants.
- Cons: increases DNS churn and support overhead.
- Cons: more selector overlap windows mean more operational risk.
- Cons: security gain is not proportional for default tenants.
- Rejected as default; can be pack-specific by future override.

### Annual Rotation with Emergency Revocation

- Pros: aligns with existing service cadence.
- Pros: keeps DNS churn manageable.
- Pros: emergency path handles compromise quickly.
- Cons: annual keys have longer exposure than quarterly keys.
- Cons: overlap windows require careful DNS and sender coordination.
- Cons: high-volume signing needs lease controls.
- Accepted as default custody and rotation posture.

## Consequences

- Positive: tenant signing-key compromise is isolated.
- Positive: audits can prove selector lifecycle and key custody.
- Positive: annual cadence matches existing service ADR and reduces DNS churn.
- Positive: emergency revocation has a concrete time target.
- Positive: high-volume send can scale through short signing leases.
- Positive: selector overlap preserves deliverability during DNS propagation.
- Positive: sovereign packs can require in-cell signing.
- Positive: tenant diagnostics can show readiness without exposing secrets.
- Negative: key lifecycle operations add operational burden.
- Negative: OpenBao or HSM availability affects send availability.
- Negative: selector misconfiguration can cause deliverability incidents.
- Negative: signing lease bugs could widen key-use blast radius.
- Negative: Ed25519 adoption depends on receiver compatibility.
- Neutral: DMARC and SPF remain separate but related domain-auth controls.
- Neutral: external providers can be adapters if they emit equivalent evidence.
- Neutral: platform domains are modeled as a tenant rather than a global exception.
- Neutral: emergency revocation can temporarily hold outbound mail.
- Neutral: rotation cadence can be amended by compliance pack.

## Implementation Notes

- Data shape `CommsEmailTenantDomain`: `{tenant_id, domain_id, fqdn, verification_state, home_cell, dmarc_alignment_mode, pack_set_hash}`.
- Data shape `CommsEmailDkimSelector`: `{tenant_id, domain_id, selector, algorithm, public_dns_value, custody_ref, state, activates_at, retires_at}`.
- Data shape `SigningLease`: `{tenant_id, domain_id, selector, lease_id, worker_id, issued_at, expires_at, permit_id}`.
- Data shape `DkimRotationPlan`: `{tenant_id, domain_id, from_selector, to_selector, overlap_start, overlap_end, dns_confirmed_at, state}`.
- Data shape `EmergencyRevocation`: `{tenant_id, domain_id, selector, reason, declared_by, declared_at, completed_at, audit_event_id}`.
- Data shape `DkimSigningEvidence`: `{message_id, tenant_id, domain_id, selector, body_hash, header_hash, signed_at, provider_adapter}`.
- OpenBao path `transit/keys/{cell_id}/{tenant_id}/comms-email/dkim/{domain_id}/{selector}`.
- OpenBao path `secret/{tenant_id}/comms-email/dkim/{domain_id}/{selector}/metadata`.
- Kubernetes secret stores only OpenBao references, never private keys.
- REST endpoint `POST /v1/comms-email/domains` creates tenant domain challenge.
- REST endpoint `GET /v1/comms-email/domains/{domain_id}/dns-records` returns DKIM, SPF, and DMARC records.
- REST endpoint `POST /v1/comms-email/domains/{domain_id}/verify` verifies DNS challenge.
- REST endpoint `POST /v1/comms-email/domains/{domain_id}/dkim/selectors` creates selector pair.
- REST endpoint `POST /v1/comms-email/domains/{domain_id}/dkim/selectors/{selector}/activate` activates selector.
- REST endpoint `POST /v1/comms-email/domains/{domain_id}/dkim/rotate` starts scheduled rotation.
- REST endpoint `POST /v1/comms-email/domains/{domain_id}/dkim/revoke` starts emergency revocation.
- REST endpoint `POST /v1/comms-email/signing-leases` issues a short signing lease.
- REST endpoint `GET /v1/comms-email/domains/{domain_id}/dkim/status` returns tenant-safe diagnostics.
- AsyncAPI channel `comms_email.dkim.selector.created.v1` publishes selector creation.
- AsyncAPI channel `comms_email.dkim.selector.activated.v1` publishes activation.
- AsyncAPI channel `comms_email.dkim.rotation.started.v1` publishes rotation start.
- AsyncAPI channel `comms_email.dkim.revoked.v1` publishes emergency revocation.
- AsyncAPI channel `comms_email.dkim.signing.failure.v1` publishes signing failures.
- Cedar permit `comms_email::domain::verify` requires tenant admin and matching tenant id.
- Cedar permit `comms_email::dkim::selector_activate` requires step-up and DNS confirmation.
- Cedar permit `comms_email::dkim::signing_lease_issue` requires service identity, domain readiness, and pack allowance.
- Cedar permit `comms_email::dkim::emergency_revoke` requires incident role and reason code.
- Cedar forbid `comms_email::send::unsigned_fallback` is unconditional.
- Cedar forbid `comms_email::dkim::cross_cell_sign` when pack disallows remote signing.
- Audit event `EVT-CME-DKIM-SELECTOR-CREATED` includes selector and custody ref hash.
- Audit event `EVT-CME-DKIM-SELECTOR-ACTIVATED` includes DNS confirmation evidence.
- Audit event `EVT-CME-DKIM-ROTATION-STARTED` includes overlap window.
- Audit event `EVT-CME-DKIM-EMERGENCY-REVOKED` includes completion duration.
- Audit event `EVT-CME-DKIM-SIGNING-LEASE-ISSUED` includes lease id and expiry.
- Audit event `EVT-CME-DKIM-SIGNING-FAILED` includes reason and selector.
- Metric `comms_email_dkim_sign_latency_ms` tracks signing path.
- Metric `comms_email_dkim_rotation_days_until_due` tracks cadence compliance.
- Metric `comms_email_dkim_dns_propagation_seconds` tracks DNS readiness.
- Metric `comms_email_dkim_emergency_revoke_seconds` tracks revocation target.
- Metric `comms_email_unsigned_fallback_attempt_total` must remain zero.
- Metric `comms_email_signing_lease_active` tracks active leases by tenant tier.
- Trace span `comms_email.dkim.sign` records selector and custody mode.
- Trace span `comms_email.dkim.rotate` records DNS checks and activation.
- Trace span `comms_email.dkim.revoke` records incident id and completion.
- Log schema `CommsEmailDkimDecisionLog` includes tenant hash, domain hash, selector, action, and result.
- SLO target: signing p95 <= 10 ms for lease-backed high-volume send.
- SLO target: signing p95 <= 50 ms for direct transit signing.
- SLO target: scheduled rotation completion within 14-day overlap.
- SLO target: emergency revocation completion <= 5 minutes.
- SLO target: unsigned fallback attempts equals zero.
- Capacity math: 2,000 messages per second at 4 ms lease-backed signing p95 yields 8 in-flight signatures before safety factor.
- Capacity math: 10,000 tenants with two selectors each means 20,000 active or overlapping selector records.
- Capacity math: annual rotation spreads DNS changes; monthly batching prevents global rotation spikes.
- Capacity math: 14-day overlap with daily retry traffic requires both selectors to verify delayed messages during the overlap and grace window.
- Rollback path: revert primary selector pointer to previous active selector during overlap.
- Rollback path: hold outbound send if no selector can sign; do not send unsigned.
- Rollback path: revoke active signing leases on custody incident.
- Multi-region path: sign in tenant home cell and queue remote sends until home-cell signing is available.
- Sovereign-cell path: KR, EU, CN-PIPL, FedRAMP-High, and IL5/6 packs require in-cell signing custody.
- Versioning: selector APIs v1 are additive only.
- Deprecation: retired selectors remain verifiable for at least 30 days after final send unless emergency revocation dictates otherwise.

## Verification

- Unit test `selector_activation_requires_verified_domain` checks DNS precondition.
- Unit test `unsigned_fallback_is_unconditionally_forbidden` checks safety invariant.
- Unit test `signing_lease_expires_within_sixty_seconds` checks lease TTL.
- Unit test `emergency_revocation_revokes_active_leases` checks incident path.
- Unit test `cross_cell_signing_denied_for_sovereign_pack` checks residency.
- Property test `rotation_overlap_always_has_valid_signer` checks planned rotation.
- Property test `selector_names_are_deterministic_and_unique` checks naming.
- Property test `public_dns_value_matches_custody_key` checks key linkage.
- Fuzz test `dns_record_parser_rejects_malformed_dkim` checks onboarding.
- Integration test `domain_onboard_creates_two_selectors_before_send` checks readiness.
- Integration test `scheduled_rotation_keeps_old_selector_valid_for_overlap` checks deliverability.
- Integration test `emergency_revocation_completes_under_five_minutes` checks target.
- Integration test `provider_adapter_maps_signing_evidence` checks portability.
- Load test `dkim_sign_two_thousand_messages_per_second` validates signing SLO.
- Load test `ten_thousand_tenant_rotation_calendar` validates cadence spread.
- Chaos test `openbao_unavailable_holds_send_unsigned_fallback_zero` checks fail-closed.
- Chaos test `dns_propagation_delay_prevents_selector_cutover` checks rollback.
- Metric check: dashboard `comms-email/dkim-rotation` shows due, active, overlap, and revocation panels.
- Metric check: dashboard `comms-email/send-pipeline` shows signing latency and failure reasons.
- Alert check: unsigned fallback attempt above zero pages immediately.
- Audit check: every selector activation emits `EVT-CME-DKIM-SELECTOR-ACTIVATED`.
- Static check: no database column stores private key bytes.
- Contract check: OpenAPI exposes no private-key export endpoint.
- Regression check: SVC-ADR-001 annual cadence remains honored.

## References

- RFC 6376 DomainKeys Identified Mail.
- RFC 7208 Sender Policy Framework.
- RFC 7489 DMARC.
- RFC 8617 Authenticated Received Chain.
- RFC 8461 SMTP MTA Strict Transport Security.
- AWS SES domain identity documentation.
- Google Workspace DKIM documentation.
- Microsoft 365 DKIM selector documentation.
- OpenBao transit secrets engine documentation.
- Cedar policy language documentation.
- SVC-ADR-001 DKIM rotation cadence.
- ADR-0243 Cedar-as-universal-gate.
- ADR-0263 observability-emission-contract.
- microservices/comms-email/PRD.md.
- comms/comms-email/runbooks/dkim-key-rotation.md.
- comms/comms-email/runbooks/reputation-drop-circuit-breaker-engaged.md.

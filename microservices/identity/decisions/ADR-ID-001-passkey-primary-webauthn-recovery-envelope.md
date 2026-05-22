---
id: ADR-ID-001
title: Passkey-Primary Multi-Factor with WebAuthn Level 3 and Recovery-Key Envelope
status: Accepted
date: 2026-05-20
microservice: identity
related_oyatie_adrs:
  - docs/decisions/ADR-0002-tenant-and-identity-kernel.md
  - docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md
  - docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - docs/decisions/ADR-0008-data-use-boundary.md
  - docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md
decision_owner: axis-identity
---

# ADR-ID-001: Passkey-Primary Multi-Factor with WebAuthn Level 3 and Recovery-Key Envelope

## Context

- Identity owns OIDC issuer, WebAuthn relying party, SCIM server, step-up orchestrator, external IdP federation, HRIS adapter, AAGUID refresh, and multi-context principal resolution.
- Existing IPs include `IP-004-webauthn-relying-party-kernel.md`, `IP-005-webauthn-rest.md`, `IP-010-step-up-orchestrator.md`, and `IP-017-multi-context-principal-resolver.md`.
- Existing runbooks include `passkey-reset.md`, `passkey-cross-device-debug.md`, `jwks-rotation.md`, `brute-force-mitigation.md`, and `idp-failover-drill.md`.
- Existing policy files include `cedar-acr-predicates.cedar`, `context-split.cedar`, `dual-context-residency.cedar`, `operator-recovery.cedar`, and `tenant-scope.cedar`.
- Named precedent: Google Advanced Protection and Apple iCloud Keychain make phishing-resistant authenticators the default high-assurance model.
- Named precedent: Microsoft Entra ID passkey rollout treats FIDO2 credentials as first-class authentication methods with device and AAGUID metadata.
- Named precedent: GitHub account recovery combines recovery codes, passkeys, and support verification, but Oyatie needs tenant and pack overlays.
- Constraint ID-C1: `Principal`, `Subject`, `Session`, `Credential`, and tenant scope must conform to ADR-0002.
- Constraint ID-C2: registration, authentication, step-up, recovery, and revocation must emit audit evidence per ADR-0003.
- Constraint ID-C3: Cedar must decide assurance level, recovery authority, operator action, and session issuance per ADR-0007.
- Constraint ID-C4: biometric templates, device attestations, account recovery facts, and identity documents are governed by ADR-0008 data classes.
- Constraint ID-C5: recovery-key envelopes and issuer signing keys must use OpenBao / HSM-backed custody per ADR-0043.
- Constraint ID-C6: passkeys must be primary for human users; passwords are not a first-class primary credential.
- Constraint ID-C7: recovery must be possible without support gaining account access.
- Constraint ID-C8: tenant administrators cannot recover a user's personal tenant context.
- Constraint ID-C9: work-tenant recovery cannot silently bind to personal identity without explicit dual-context proof.
- Constraint ID-C10: emergency, survivor-safety, minor, and cognitive-impairment journeys require recovery flows that are safe but not abusable.
- Constraint ID-C11: external IdP federation must not downgrade passkey assurance for local step-up.
- Constraint ID-C12: authenticator sync passkeys and hardware security keys must be distinguishable for policy.
- Constraint ID-C13: AAGUID trust decisions must be updateable without breaking existing sessions abruptly.
- Constraint ID-C14: account recovery must rotate sessions, credentials, and delegated grants atomically.
- Constraint ID-C15: the service must remain useful when a platform authenticator vendor has an outage.
- The architecture must support WebAuthn Level 3 features, resident credentials, discoverable credentials, user verification, and device-bound attestation policy.
- The architecture must support recovery codes and recovery key envelopes without storing plaintext recovery secrets.
- The architecture must support high-risk step-up for eDiscovery, tenant admin, healthcare break-glass, and cross-tenant authority.
- The architecture must explain exact assurance levels to Cedar and downstream services.

## Decision

- Make passkeys the primary human authentication factor for Identity.
- Use WebAuthn Level 3 as the protocol target for registration, authentication, attestation, and discoverable credentials.
- Accept synced platform passkeys for normal user assurance.
- Require hardware-backed or enterprise-attested passkeys for high-risk administrator, auditor, break-glass, and recovery-authority flows.
- Represent assurance as AAL-like `acr` values: `aal1_observed`, `aal2_passkey_uv`, `aal3_hardware_bound`, and `aal3_recovery_ceremony`.
- Store each passkey as a `CredentialBinding` scoped to tenant, subject, device, AAGUID, credential id hash, and attestation class.
- Store credential public keys only; never store private key material.
- Keep credential id raw bytes encrypted at rest and expose only hashes to audit and support.
- Use recovery-key envelope as a separate factor: a user-held recovery secret wraps a server-generated recovery grant.
- Store only recovery envelope ciphertext, recovery public metadata, and a verifier hash.
- Split recovery into three phases: proof collection, recovery grant issuance, and session rebinding.
- Require Cedar approval before recovery grant issuance.
- Require audit-chain write before any recovered session is issued.
- Rotate all active sessions after successful recovery.
- Revoke delegated tokens and high-risk grants after recovery unless tenant policy explicitly preserves them.
- Use OpenBao to protect issuer signing keys, recovery envelope wrapping keys, and operator break-glass sealing keys.
- Keep recovery envelope paths per tenant and subject: `secret/<tenant_id>/identity/recovery/<subject_id>/<recovery_epoch>`.
- Use AAGUID refresh worker to maintain authenticator trust catalog.
- Treat unknown AAGUID as allowed for low-risk login only if user verification succeeds and tenant policy allows it.
- Treat revoked AAGUID as requiring step-up with a different authenticator.
- Treat external IdP authentication as identity proof input, not as passkey-equivalent assurance unless signed device-bound claims meet policy.
- Issue OIDC tokens only after session assurance and tenant audience are resolved.
- Encode `acr`, `amr`, `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `credential_epoch`, and `recovery_epoch` in signed token claims.
- Make recovery denial as auditable as recovery success.
- Make support operators unable to decrypt recovery envelopes.

## Alternatives Considered

### Password Primary with Optional Passkeys

- Pros: familiar to users.
- Pros: easier migration from legacy SaaS.
- Pros: fewer device compatibility issues.
- Cons: phishing and credential stuffing remain primary account-takeover paths.
- Cons: high-risk tenants need stronger defaults.
- Cons: recovery flows remain support-heavy.
- Rejected because identity is a substrate and must default to phishing-resistant authentication.

### External IdP Only

- Pros: delegates authentication complexity to enterprise IdPs.
- Pros: reduces local credential storage.
- Pros: fits B2B tenants with mature SSO.
- Cons: B2C, personal, emergency, and cross-tenant flows still need local identity.
- Cons: external IdP claims vary and may not encode Oyatie dual-context requirements.
- Cons: tenant IdP compromise would directly compromise Oyatie without local step-up.
- Rejected because external federation is important but cannot be the sole substrate.

### Support-Assisted Recovery with Operator Escrow

- Pros: simplest for users who lose all devices.
- Pros: support can resolve edge cases quickly.
- Pros: easy to explain in enterprise contracts.
- Cons: operators become high-value account-takeover targets.
- Cons: violates personal/work tenant boundary expectations.
- Cons: makes recovery hard to prove cryptographically.
- Rejected because recovery must be user-held and ceremony-bound, not operator-decryptable.

### Hardware Security Keys Only

- Pros: highest phishing-resistant assurance.
- Pros: good enterprise control story.
- Pros: clear lost-device procedures.
- Cons: too expensive and fragile for broad consumer adoption.
- Cons: mobile-first users expect synced passkeys.
- Cons: creates onboarding friction for low-risk contexts.
- Rejected as a universal default; retained as required mode for high-risk assurance.

## Consequences

- Positive: default authentication is phishing-resistant.
- Positive: assurance level is explicit and enforceable by Cedar and downstream services.
- Positive: recovery does not require support to know or decrypt user secrets.
- Positive: work and personal identities can recover independently.
- Positive: high-risk actions can require hardware-bound credentials without blocking normal login.
- Positive: external IdP federation composes with local step-up instead of replacing it.
- Positive: AAGUID catalog changes can be rolled out as policy updates.
- Positive: token claims carry the context needed by tenancy, policy-engine, messenger, mail, drive, and compliance.
- Negative: first-run onboarding is more complex than password-only systems.
- Negative: synced passkeys differ by platform and must be tested across vendors.
- Negative: lost-device recovery has more ceremony than email reset.
- Negative: support cannot bypass the recovery envelope, which increases some user-friction cases.
- Negative: authenticator trust catalog freshness becomes an operational dependency.
- Neutral: password compatibility can exist only as a migration or low-assurance fallback where a tenant explicitly permits it.
- Neutral: external IdP remains supported for workforce SSO and SCIM lifecycle.
- Neutral: tenants can require hardware keys for all users via policy.
- Neutral: recovery codes and recovery keys are separate artifacts with different UX.
- Neutral: session and token rotation after recovery may interrupt active workflows by design.

## Implementation Notes

- Data shape `CredentialBinding`: `{tenant_id, subject_id, credential_id_hash, public_key_cose, aaguid, attestation_class, uv_required, credential_epoch, state}`.
- Data shape `PasskeyRegistrationChallenge`: `{tenant_id, subject_id, challenge_hash, rp_id, origin, created_at, expires_at, policy_hash}`.
- Data shape `AuthenticationCeremony`: `{tenant_id, subject_id, ceremony_id, credential_id_hash, user_verification, sign_count, origin, acr, audit_event_id}`.
- Data shape `RecoveryEnvelope`: `{tenant_id, subject_id, recovery_epoch, envelope_ciphertext, verifier_hash, openbao_wrap_ref, created_at, disabled_at}`.
- Data shape `RecoveryGrant`: `{tenant_id, subject_id, recovery_epoch, grant_id, ceremony_state, expires_at, approved_policy_id, audit_event_id}`.
- OpenBao path: `secret/<tenant_id>/identity/recovery/<subject_id>/<recovery_epoch>`.
- OpenBao path: `transit/keys/<tenant_id>-identity-token-issuer`.
- REST endpoint `POST /v1/identity/webauthn/registration/options` creates a registration challenge.
- REST endpoint `POST /v1/identity/webauthn/registration/verify` binds a credential.
- REST endpoint `POST /v1/identity/webauthn/authentication/options` creates an authentication challenge.
- REST endpoint `POST /v1/identity/webauthn/authentication/verify` creates a session ceremony.
- REST endpoint `POST /v1/identity/recovery/envelopes` creates or rotates a recovery envelope.
- REST endpoint `POST /v1/identity/recovery/ceremonies` starts recovery.
- REST endpoint `POST /v1/identity/recovery/ceremonies/{ceremony_id}/complete` issues a recovery grant and rebinding session.
- REST endpoint `POST /v1/identity/sessions/{session_id}/step-up` raises assurance.
- AsyncAPI channel `identity.passkey.registered.v1` publishes credential binding.
- AsyncAPI channel `identity.passkey.revoked.v1` publishes credential revocation.
- AsyncAPI channel `identity.recovery.started.v1` publishes recovery ceremony start.
- AsyncAPI channel `identity.recovery.completed.v1` publishes successful recovery.
- AsyncAPI channel `identity.session.assurance.changed.v1` publishes ACR changes.
- Cedar permit `identity::passkey::register` requires current session or onboarding proof.
- Cedar permit `identity::session::step_up` requires requested action and accepted authenticator class.
- Cedar permit `identity::recovery::complete` requires valid recovery envelope proof and no active recovery freeze.
- Cedar forbid `identity::recovery::operator_decrypt` is unconditional.
- Cedar forbid `identity::token::issue_high_risk` when `context.acr < "aal3_hardware_bound"`.
- Audit event `EVT-ID-PASSKEY-REGISTERED` includes AAGUID, attestation class, and credential hash.
- Audit event `EVT-ID-RECOVERY-ENVELOPE-ROTATED` includes recovery epoch and policy hash.
- Audit event `EVT-ID-RECOVERY-COMPLETED` includes session rotation count and revoked grant count.
- Audit event `EVT-ID-RECOVERY-DENIED` includes denial reason and policy id.
- Metric `identity_webauthn_verify_latency_ms` tracks authentication and registration.
- Metric `identity_recovery_completion_total` tracks success, denial, and abandonment.
- Metric `identity_unknown_aaguid_login_total` tracks authenticator catalog gaps.
- Metric `identity_high_risk_stepup_success_ratio` tracks administrative workflow health.
- Capacity math: if authentication p95 is 80 ms and peak is 5,000 logins/s, Little's Law gives 400 in-flight ceremonies; provision 4,000 slots for burst and IdP callback jitter.
- Capacity math: AAGUID catalog refresh every 6 hours with 10k entries is small; trust propagation SLO is dominated by policy publish, target below 60 seconds.
- Rollback path: credential policy change rolls back by signed policy pointer, not by deleting credentials.
- Rollback path: failed recovery completion revokes the recovery grant and leaves old sessions revoked only after final audit commit.
- Multi-region path: authentication ceremony runs in subject home cell; token verification keys replicate read-only.
- Sovereign path: recovery envelopes and authentication audit stay in home jurisdiction for regulated packs.
- Versioning: WebAuthn ceremony profile is `identity-webauthn-profile-v1`.
- Deprecation: authenticator class deprecation gets tenant notice unless a critical compromise requires immediate deny.

## Verification

- Unit test `passkey_registration_binds_tenant_subject_and_aaguid` verifies credential scope.
- Unit test `operator_cannot_decrypt_recovery_envelope` verifies no support bypass.
- Unit test `high_risk_token_requires_hardware_bound_acr` covers assurance gates.
- Unit test `unknown_aaguid_low_risk_only` covers authenticator catalog policy.
- Unit test `recovery_rotates_sessions_and_delegated_grants` covers account-takeover containment.
- Property test `webauthn_challenge_cannot_replay_across_tenants` generates challenge contexts.
- Property test `credential_epoch_monotonic_after_recovery` checks revocation ordering.
- Fuzz test `webauthn_attestation_parser_rejects_malformed_cbor` covers hostile clients.
- Integration test `external_idp_requires_local_stepup_for_admin` verifies federation composition.
- Integration test `personal_tenant_recovery_not_admin_recoverable` protects dual-context boundary.
- Integration test `recovery_denial_emits_audit_event` verifies evidence parity.
- Integration test `oidc_claims_include_acr_amr_tenant_and_epoch` validates token shape.
- Load test `webauthn_verify_5000_logins_per_second` keeps p95 below 100 ms.
- Load test `stepup_1000_admin_actions_per_second` keeps p99 below 150 ms.
- Chaos test `openbao_unavailable_blocks_recovery_envelope_rotation` proves fail-closed behavior.
- Chaos test `aaguid_catalog_publish_rollback` proves policy pointer rollback.
- Metric SLO: `identity_webauthn_verify_latency_ms` p95 below 100 ms.
- Metric SLO: `identity_high_risk_stepup_success_ratio` above 99 percent outside active incidents.
- Metric SLO: `identity_unknown_aaguid_login_total` reviewed daily and below tenant policy threshold.
- Audit check: every session issued after recovery has preceding `EVT-ID-RECOVERY-COMPLETED`.
- Audit check: every high-risk token has a recent step-up ceremony id.
- Static check: no endpoint returns recovery plaintext or credential private material.
- Static check: token claims include tenant and credential epoch.
- Contract check: OpenAPI documents passkey primary and recovery envelope semantics.


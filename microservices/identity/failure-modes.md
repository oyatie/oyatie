---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + ops-sre-reliability
---

# Failure Modes — identity µservice

Each failure mode declares: trigger, detection signal, blast radius, automatic mitigation, manual mitigation, recovery procedure.

## FM-01 — Postgres primary failure within a pack

**Trigger**: AZ outage; instance crash; disk failure on primary.
**Detection**: pg_isready probe fails; Patroni leader election triggers.
**Blast radius**: per-pack; all Zitadel write operations stall briefly.
**Automatic mitigation**: Patroni promotes a sync replica (≤30s).
**Manual mitigation**: if Patroni quorum lost, ops-sre-reliability promotes manually per runbook.
**Recovery**: restore failed primary as new replica; allow re-sync; relabel.
**Customer impact**: ≤30s sign-in/SCIM unavailability per pack.

## FM-02 — JWT signing key rotation cascading misverification

**Trigger**: New `kid` published to JWKS but consumer caches stale for >24h.
**Detection**: token verification 401 rate spikes from a specific consumer µservice.
**Blast radius**: per-consumer-µservice; per-pack scope.
**Automatic mitigation**: `oya-shared-oidc-client-kernel` issues JWKS refresh on `kid` not found before failing (already in kernel).
**Manual mitigation**: force JWKS cache flush on the consumer via configmap reload.
**Recovery**: consumer recovers automatically once JWKS refreshes.
**Customer impact**: ≤2min auth blip on specific consumer routes.

## FM-03 — WebAuthn sign-count regression detected (cloned-authenticator alarm)

**Trigger**: Browser presents an assertion with sign_count < stored sign_count.
**Detection**: `WebauthnError::SignCountRegression` emitted; alarm to OnCall.
**Blast radius**: per-credential, per-user.
**Automatic mitigation**: refuse the assertion; mark credential `revoked=true`; emit `IdentityWebAuthnRevoked`.
**Manual mitigation**: ops-security investigates; if confirmed clone, force user step-up to `sensitive` and re-register a new credential.
**Recovery**: user registers a new Passkey.
**Customer impact**: one user temporarily unable to sign in with the cloned credential; alternate credentials remain valid.

## FM-04 — FIDO-MDS3 metadata fetch failure

**Trigger**: FIDO Alliance blob endpoint unreachable; signature verification fails on the blob.
**Detection**: `aaguid-refresh-freshness` SLO degrades.
**Blast radius**: regulated packs (those that enforce AAGUID allowlist).
**Automatic mitigation**: continue serving cached metadata up to 48h.
**Manual mitigation**: ops-security investigates upstream; if outage > 48h, ops-security may temporarily widen AAGUID allowlist with explicit risk-acceptance ticket.
**Recovery**: FIDO Alliance endpoint recovers; cache refreshes automatically.
**Customer impact**: new authenticator-model registrations may be refused in regulated packs until metadata refreshes.

## FM-05 — SCIM bearer leaked (single-tenant breach)

**Trigger**: SCIM bearer appears in pastebin / red-team report / abnormal-IP usage detected.
**Detection**: rate anomaly + geo anomaly on a SCIM bearer.
**Blast radius**: one tenant's user-list (full read), plus mutations until revocation.
**Automatic mitigation**: rate-limit the suspect bearer to 1 rps; alert.
**Manual mitigation**: revoke the bearer immediately; provision a new bearer; coordinate with tenant IT to re-bind upstream SCIM client.
**Recovery**: tenant IT pushes a SCIM full-resync after new bearer is in place.
**Customer impact**: tenant SCIM down for the duration of the rotation (≤4h).

## FM-06 — Zitadel Postgres event-store corruption

**Trigger**: storage-level corruption; bug in Zitadel that writes invalid event ordering.
**Detection**: Zitadel startup fails the event-store consistency check; or runtime errors on token issuance with `event-store inconsistent`.
**Blast radius**: one pack.
**Automatic mitigation**: failover to warm-standby in failover region (PITR-restored, ≤30s RPO).
**Manual mitigation**: ops-sre-reliability triages: if corruption pre-dates RPO, restore from WAL archive in offsite tape.
**Recovery**: rebuild primary from warm-standby; replay since-failover events.
**Customer impact**: ≤5min identity outage per pack; ≤30s of sign-in events potentially lost (replayed from JWKS cache + audit-chain).

## FM-07 — Step-up infinite loop (user bounces between elevated and sensitive)

**Trigger**: Bug in Cedar policy: `acr=sensitive` action immediately requires `acr=critical` which immediately requires `acr=sensitive`, etc.
**Detection**: `IdentityStepUpGranted` rate spike for a single user; >3 grants in 60s.
**Blast radius**: typically one user or one policy; if widespread, one tenant.
**Automatic mitigation**: rate-limit step-up grants to 3 per 60s per user; subsequent attempts return 429.
**Manual mitigation**: identify the offending Cedar policy; emergency-deploy a fix.
**Recovery**: PR through admission gate with the fix.
**Customer impact**: affected user(s) temporarily blocked; alternate auth path (operator-mediated) available.

## FM-08 — HRIS poller drift (terminated user still active)

**Trigger**: HRIS adapter fails to receive a termination event due to vendor API change / vendor outage / silent failure.
**Detection**: daily reconciliation job; compares HRIS active-set vs Zitadel active-set; drift >0.1% alerts.
**Blast radius**: affected tenant(s).
**Automatic mitigation**: reconciliation job applies SCIM PATCH `active=false` to drifted users; sends notice to tenant IT.
**Manual mitigation**: tenant IT confirms HRIS state; either accepts the auto-correction or escalates.
**Recovery**: HRIS adapter restored; reconciliation re-runs.
**Customer impact**: terminated user retains access for up to 24h (window between HRIS event and reconciliation); acceptable per SOC 2 CC6.6 with ≤24h reconciliation cadence.

## FM-09 — DDoS on /oauth/v2/token endpoint

**Trigger**: botnet sends millions of token requests with random / brute-forced credentials.
**Detection**: rate alarm + Coraza WAF deny rate spike.
**Blast radius**: one pack edge; legitimate sign-ins may degrade.
**Automatic mitigation**: Envoy rate-limit + eBPF XDP drop at NIC; Coraza WAF rules trip; per-IP block.
**Manual mitigation**: ops-security tightens geo/ASN deny-list; coordinate with upstream CDN if any.
**Recovery**: attack subsides; edge filters relax back to default.
**Customer impact**: ≤5% legitimate sign-in latency increase during attack window.

## FM-10 — Cedar PDP outage (waypoint ext_authz unreachable)

**Trigger**: Cedar PDP pod crash; ext_authz gRPC unreachable.
**Detection**: Envoy ext_authz failure metric; 500s on every authorised call.
**Blast radius**: per-pack (every consumer µservice through the waypoint).
**Automatic mitigation**: ext_authz `failure_mode_allow: false` (per ADR-0183) → requests denied during outage.
**Manual mitigation**: ops-sre-reliability redeploys PDP pods; verifies Cedar entity store.
**Recovery**: PDP recovers; requests resume.
**Customer impact**: ≤5min full-stack outage per pack (sign-ins, mutations, reads all denied).

## FM-11 — Cross-pack residency violation attempt

**Trigger**: A consumer µservice attempts to authenticate a pack-eu user against pack-us issuer (bug or misconfig).
**Detection**: Cedar deny audit on `principal.iss` mismatch.
**Blast radius**: contained at PDP; no data leak.
**Automatic mitigation**: Cedar denies; audit event emitted with reason `residency-violation`.
**Manual mitigation**: ops-security RCA; council-compliance review.
**Recovery**: the consumer's pack binding is fixed.
**Customer impact**: one request failure for the misconfigured consumer; no data exposure.

## FM-12 — OpenBao SecretReference resolver outage

**Trigger**: cloud-secrets µservice degraded; SecretReference resolution fails.
**Detection**: SecretReference resolve latency p99 > 100ms; resolution-success-rate < 0.999.
**Blast radius**: per-pack; new pod starts fail.
**Automatic mitigation**: in-process cached secrets remain valid; existing pods continue serving until rotation due.
**Manual mitigation**: ops-security investigates cloud-secrets µservice per FM in cloud-secrets failure-modes.
**Recovery**: cloud-secrets recovers; deferred rotations execute.
**Customer impact**: rotation-cadence breach; SLO `key-rotation-correctness` may degrade until recovery.

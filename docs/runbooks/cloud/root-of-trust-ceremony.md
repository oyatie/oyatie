---
doc_class: Runbook
template_id: TPL-RUNBOOK
runbook_id: RB-CLOUD-ROOT-OF-TRUST-CEREMONY
title: "Run the offline root-of-trust ceremony without exposing root secrets"
status: draft
severities_supported: [Sev-1, Sev-2]
owner_team: council-security + axis-cloud + ops-compliance
last_verified: 2026-06-30
last_drilled: null
slo_topic: oya.cloud.bootstrap.root_of_trust
state_data_contract: ../../../specs/root-of-trust-ceremony-contract.json
audit_emission_topic: oya.security.root_of_trust.ceremony
related_runbooks:
  - RB-CLOUD-KMS-EMERGENCY-ROTATION
  - RB-SHAMIR-SHARE-LOSS-OR-COERCION
  - RB-META-TRUST-ROOT-RECOVERY
related_adrs: [ADR-0536, ADR-0537, ADR-0510, ADR-0515]
diataxis_class: how-to
authority_chain_declaration: |
  ADR-0537 step 0 + ADR-0536 D-1/D-8/D-9/D-16 + OQ-005 founder decision packet > this runbook.
  This runbook is specification-only until founder ratifies the OQ-005 custody posture and HSM procurement path.
doc_status: published
---

# Runbook RB-CLOUD-ROOT-OF-TRUST-CEREMONY: Run the offline root-of-trust ceremony without exposing root secrets

## Trigger / symptom

Use this runbook only for a scheduled, witnessed ceremony that establishes or rotates the dogfood bootstrap root of trust from ADR-0537 step 0:

- first internal dogfood bootstrap before Step 1 KMS unseal;
- HSM-backed destination root provisioning after procurement evidence is available;
- Shamir/quorum break-glass share refresh, holder replacement, or coercion response;
- DNS seed snapshot re-signing when the ADR-0537 Step 5 seed must change;
- sealed FIDO2 break-glass credential set replacement.

Do not use this runbook for day-to-day KMS key rotation, tenant BYOK rotation, OpenBao operator unseal, or emergency incident mitigation. Those flows route to the related runbooks above.

## SLO impact

- SLO affected: `oya.cloud.bootstrap.root_of_trust`.
- Ceremony execution is planned change work, not live production mitigation.
- Active compromise of an existing root, a coerced share holder, or unauthorized root material exposure is Sev-1 and must switch to incident response before any new ceremony continues.

## Scope and non-claims

This is a runbook/data-contract change only. It does not authorize production HSM claims, implement ceremony tooling, procure hardware, or ratify OQ-005. The current downstream posture from OQ-005 is:

- destination custody: HSM-backed root domains with non-exportable key material;
- bootstrap/break-glass posture: bounded Shamir/quorum software custody for Tier 1 internal dogfood or break-glass transition only;
- forbidden claim: software/quorum/OpenBao-adjacent custody must not be presented as HSM-backed, regulated/FIPS target custody, or Tier 3 regulated production.

## Roles

- Ceremony lead: council-security member accountable for stop/go decisions.
- HSM custodian: operates the offline HSM or records the approved software-quorum fallback.
- Share custodians: distinct humans holding the approved M-of-N shares; the threshold and share count are non-secret, the share values are secret.
- Witnesses: at least two independent witnesses from council-security/ops-compliance.
- Recorder: produces the redacted evidence packet conforming to `specs/root-of-trust-ceremony-contract.json`.
- Founder approver: required for OQ-005 custody posture ratification before production or regulated claims.

## Pre-checks (stop on any failure)

1. Confirm authorization scope.
   - Expected: ceremony request names `purpose`, `target_environment`, `custody_posture`, and the current OQ-005 approval status.
   - Stop if the request attempts Tier 3 regulated production or HSM-backed claims without HSM evidence and founder ratification.

2. Allocate a ceremony id and empty redacted evidence packet.
   - Expected: `ceremony_id` follows `rotc-YYYYMMDD-<purpose>-<seq>` and the packet path follows `evidence/root-of-trust/ceremonies/<ceremony_id>/manifest.json`.
   - The packet may contain hashes, fingerprints, attestation ids, public keys, certificate chains, and witness signatures; it must not contain private keys, root bytes, unseal keys, Shamir share values, recovery seeds, FIDO2 private attestation material, passwords, or raw tokens.

3. Verify the ceremony room and workstation are offline.
   - Expected: network interfaces disabled or physically absent, removable media inventoried, tamper-evident bag ids recorded, phones and networked devices excluded.
   - Stop if any untrusted device or network path enters the room after pre-check.

4. Verify participant quorum and separation.
   - Expected: participants satisfy the approved M-of-N threshold, no single custodian can reconstruct alone, witnesses are not counted as share custodians unless separately approved.
   - Stop if quorum, independence, or jurisdiction distribution fails the contract.

5. Verify HSM posture or bounded fallback.
   - HSM destination path: HSM attestation, firmware version, serial hash, non-exportable capability, and custody chain are present.
   - Bounded fallback path: founder-approved internal-dogfood/break-glass exception is present, expiry is set, and production/regulated use is explicitly refused.

6. Verify source inputs.
   - Expected: ADR-0537 step 0 inputs are listed: root CA, KMS domain root, sealed FIDO2 break-glass credentials, hand-signed DNS seed snapshot.
   - Stop if any input has an untracked provenance or unredacted secret in the packet.

## Ceremony steps

### C1. Open the evidence packet

Record the ceremony metadata, participant role ids, timestamp window, target environment, and declared custody posture in the manifest. The recorder records only redacted metadata and cryptographic fingerprints; root material and shares remain off-record.

Expected outcome: manifest exists with state `opened`, no secret-bearing fields, and witness ids allocated.

### C2. Establish the root CA

On the offline ceremony workstation/HSM, generate or rotate the root CA according to the approved custody posture:

- HSM destination: key generation occurs inside the HSM with non-exportable private material.
- Hybrid: HSM root is primary; Shamir shares are issued only for documented break-glass recovery.
- Bounded bootstrap: Shamir/quorum software root is allowed only for Tier 1 internal dogfood or break-glass transition and must carry an expiry plus claim ceiling.

Record only the public certificate, certificate fingerprint, HSM attestation reference if available, and witness signature references.

Expected outcome: root CA public artifact is fingerprinted; private key bytes are never written to disk, chat, logs, repo files, or evidence packets.

### C3. Establish the KMS domain root

Create the KMS domain root material for ADR-0536 D-8:

- HSM destination: bind the KMS domain to the non-exportable HSM key/partition and record attestation evidence.
- Hybrid/break-glass: issue Shamir quorum recovery shares with `threshold >= 2`, `share_count >= threshold`, and the approved default for the environment; production-regulated use requires the HSM destination posture.
- Transitional OpenBao/software custody: may appear only as a time-bounded internal dogfood bridge, never as a production HSM substitute.

Record KMS domain id, root id, public wrapping-key/certificate fingerprint, custody posture, threshold/share-count metadata, holder ids, jurisdiction distribution, and evidence refs. Do not record share values, unseal keys, root bytes, DEKs, KEKs, OpenBao recovery keys, or plaintext exports.

Expected outcome: KMS root custody is either HSM-backed or explicitly marked as bounded bootstrap/break-glass with a forbidden-claim ceiling.

### C4. Issue Shamir shares and sealed holder receipts

For each approved share holder:

1. Holder identity and role are verified by two witnesses.
2. The share is placed on approved offline media or hardware security token.
3. The recorder logs holder id, share index, medium type, tamper-evident bag id, jurisdiction, receipt hash, and witness signature ids.
4. The holder signs a receipt that contains no share value.

Expected outcome: evidence proves distribution and custody without exposing any share material.

### C5. Seal FIDO2 break-glass credentials

Create or rotate the sealed FIDO2 break-glass credential set from ADR-0536 D-1:

- record credential set id, relying-party id, public credential ids or hashes, storage medium inventory, and sealed-envelope ids;
- require multi-person access policy and expiry/review cadence;
- forbid recording private credential material, recovery codes, PINs, biometric templates, or authenticator secrets.

Expected outcome: break-glass inventory is auditable and non-secret; credential activation remains offline and quorum-controlled.

### C6. Sign the DNS seed snapshot

Prepare the ADR-0537 Step 5 seed snapshot that the DNS data plane can serve before the DNS control plane exists:

- compile the static seed snapshot offline;
- sign it with the approved ceremony key;
- record snapshot content digest, signing certificate fingerprint, signature artifact digest, zone ids, validity window, and rollback seed digest.

Expected outcome: a hand-signed DNS seed snapshot is referenced by digest and signature, not embedded with private signing material.

### C7. Close the packet and destroy transient material

Before leaving the ceremony room:

- zeroize workstation memory and destroy scratch media;
- verify no private material was written to logs, shell history, repo files, chat, screenshots, or evidence packets;
- move sealed media into custody storage with chain-of-custody signatures;
- mark manifest state `sealed` and collect witness signatures.

Expected outcome: final packet is complete, redacted, and independently reviewable.

### C8. Publish non-secret references to the bootstrap lane

After the ceremony packet is sealed, publish only non-secret references needed by later ADR-0537 steps:

- root CA public certificate and fingerprint;
- KMS root id / domain id / public wrapping-key fingerprint / custody posture;
- DNS seed snapshot digest and signature reference;
- FIDO2 break-glass credential inventory id;
- manifest digest and audit-chain event ids.

Expected outcome: Step 1 KMS unseal and later bootstrap stages can verify provenance without seeing root material.

## Audit artifact contract

The evidence packet must conform to `specs/root-of-trust-ceremony-contract.json` and include these redacted artifact classes:

| Artifact | Required evidence | Forbidden contents |
| --- | --- | --- |
| Ceremony manifest | ceremony id, state, purpose, target environment, OQ-005 status, custody posture, participant role ids, timestamps | private keys, shares, tokens, passwords |
| HSM attestation | vendor/model, firmware digest, serial hash, attestation digest, non-exportable flag | admin PINs, partition passwords, wrapped private key blobs |
| Root CA public proof | certificate PEM or digest, subject, validity, fingerprint, witness signatures | CA private key or seed |
| KMS root proof | root id, domain id, public wrapping-key/cert fingerprint, custody posture, threshold/share-count metadata | KMS root bytes, KEKs, DEKs, OpenBao exports/unseal keys |
| Shamir holder receipts | holder id, share index, jurisdiction, medium id hash, bag id, receipt hash | share values, photos of shares, mnemonic/recovery text |
| FIDO2 break-glass inventory | credential set id, RP id, public credential hash, sealed-envelope ids, quorum policy | private attestation secrets, PINs, recovery codes |
| DNS seed proof | snapshot digest, signature digest, zone ids, validity window, rollback digest | private signing key material |
| Witness attestations | witness id, role, signed statement digest, timestamp | personal documents beyond approved identity-verification references |
| Secret-scan report | scanner id/version, reviewed paths, result summary | scanner raw dumps that include secret material |

## Rollback / abort

Abort before publication if any pre-check fails, the room loses offline integrity, quorum is insufficient, HSM attestation mismatches, a secret enters the evidence packet, or any witness refuses to sign.

If abort occurs before root publication: destroy transient material, mark packet `aborted`, and file a follow-up decision packet. If a root was already published and then found invalid or exposed: open Sev-1, freeze dependent bootstrap, run KMS emergency rotation or root re-ceremony, and publish a superseding manifest that references the failed ceremony id.

## Verification

- [ ] Manifest validates as JSON and passes the required-field checks in `specs/root-of-trust-ceremony-contract.json`.
- [ ] Every referenced public artifact has a digest and every digest resolves to a file or immutable evidence-store URI.
- [ ] The packet contains no fields named or classified as private key, share value, seed phrase, unseal key, token, password, PIN, DEK, KEK, or OpenBao export.
- [ ] Custody posture is one of `hsm_backed_destination`, `hybrid_hsm_shamir_break_glass`, or `bounded_software_quorum_bootstrap` and the claim ceiling matches OQ-005.
- [ ] Two witnesses independently sign the sealed manifest digest.
- [ ] The DNS seed snapshot signature verifies against the recorded public certificate/fingerprint.
- [ ] The root CA and KMS root refs are sufficient for ADR-0537 Step 1/Step 5 consumers without exposing root material.

## Post-ceremony updates

- [ ] Store sealed evidence under `evidence/root-of-trust/ceremonies/<ceremony_id>/` or the approved immutable evidence store.
- [ ] Record audit events: `RootOfTrustCeremonyOpened`, `RootOfTrustRootCaEstablished`, `RootOfTrustKmsDomainEstablished`, `RootOfTrustShamirSharesIssued`, `RootOfTrustFido2BreakGlassSealed`, `RootOfTrustDnsSeedSigned`, `RootOfTrustCeremonySealed`.
- [ ] Update the KMS/bootstrap registry with non-secret refs only.
- [ ] File a follow-up implementation/gate card if the ceremony exposed a missing validator, secret scanner, or custody-policy enforcement gap.

## Sources

- `docs/decisions/ADR-0537-dogfood-bootstrap-order-rust-owned-stack-doctrine.md` §1 Step 0 and §4.
- `docs/decisions/ADR-0536-hyperscaler-grounded-substrate-decision-matrix.md` D-1, D-8, D-9, D-16, OQ-5.
- OQ-005 parent handoff: Hybrid/HSM-backed destination custody with bounded Shamir/software/OpenBao-adjacent bootstrap only for Tier 1/internal dogfood or break-glass pending HSM evidence.
- `docs/runbooks/shamir-share-loss-or-coercion.md`.
- `docs/runbooks/meta-trust-root-recovery.md`.
- `docs/runbooks/cloud/kms-emergency-rotation.md`.

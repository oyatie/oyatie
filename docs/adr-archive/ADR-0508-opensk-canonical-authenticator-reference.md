---
id: ADR-0508
status: Superseded
planning_impact: false
deciders: founder, council-architecture
date: 2026-05-28
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0482, ADR-0507, ADR-0506, ADR-0483, ADR-0484]
door: two-way
---

# ADR-0508 — OpenSK canonical authenticator-side reference (Phase-1) + oya-authn-device Tier-3 bespoke hardware destination

## Status

Accepted (2026-05-28).

## Context

WebAuthn is a two-layer protocol:

- **Relying-party (RP)**: the server that validates authenticator assertions — ADR-0507
  (webauthn-rs Phase-1 + oya-webauthn Tier-2 bespoke destination).
- **Authenticator**: the hardware key (or platform authenticator) that generates and signs
  assertions — **this ADR**.

The hyperscaler pattern is to OWN both layers. Every major hyperscaler ships bespoke
authenticator silicon:

- Google: Titan security key — internal FIDO2 key distributed to all employees; based on
  related work from the OpenSK/OpenTitan family.
- Apple: Secure Enclave — bespoke ARM-codesigned security element; platform authenticator
  for Touch ID / Face ID WebAuthn.
- Microsoft: Pluton — silicon root of trust co-designed with AMD/Intel/Qualcomm; shipped in
  all modern Surface and Windows 11 hardware.

For oyatie's multi-decade kernel + OS + platform ambition (ADR-0482, ADR-0483), owning the
authenticator hardware is the canonical destination. A closed-loop identity stack — where
oyatie controls both the RP validation logic AND the authenticator firmware AND eventually the
authenticator silicon — is the only path consistent with the bespoke-over-oss doctrine and the
kubers Phase-B silicon ownership goal.

OpenSK (`github.com/google/OpenSK`) is the only credible OSS bridge:

- Rust-native FIDO2/CTAP2 authenticator firmware (no C/C++ runtime in the hot path).
- Apache 2.0 — clean; no SSPL/BSL/Commons-Clause.
- Google-maintained; reference platform is Nordic nRF52840 dongle (commodity dev hardware).
- OpenTitan SoC port in progress — aligns with oyatie's eventual open-silicon ownership goal.
- FIDO Alliance certified (CTAP2.1 conformance).

ADR-0507 covers the RP side. This ADR covers the authenticator side. Together they close the
loop: **oya-identity RP + oya-authn-device authenticator = closed-loop oyatie identity stack**.

## Hyperscaler-lens pre-check

| Criterion | Result |
|---|---|
| Active upstream | PASS — Google-maintained; actively developed at github.com/google/OpenSK; OpenTitan SoC port in-progress |
| License clean | PASS — Apache-2.0; OSI-clean; no SSPL/BSL/Commons-Clause |
| Fully self-hostable | PASS — full firmware source published; builds with standard Rust/embedded toolchain; no managed-service dependency |
| Hyperscaler-internal equivalent | PASS — Google Titan keys descend from related work; Apple Secure Enclave, MS Pluton are all bespoke authenticator silicon; all hyperscalers own their authenticator hardware |

## Decision

1. **OpenSK is the canonical Phase-1 authenticator-side reference** for oyatie. It is the
   firmware substrate from which `oya-authn-device` will be forked and eventually replaced.

2. **Initial use (Phase-1, NOW–12mo)**:
   - Declare OpenSK reference metadata in `tools/opensk-vendored/README.md`,
     `tools/opensk-vendored/UPSTREAM-CONFIG.json`, and
     `tools/opensk-vendored/OWNERS` (follow-up implementation lane; OpenSK source
     is NOT vendored in this ADR commit — intent and ownership are declared here only).
   - Ship an `oya-authn-device-firmware` crate that wraps OpenSK for the nRF52840 reference
     dongle (dev/test use only; not yet productized). Deferred IP.
   - Document dev-key provisioning workflow for oyatie engineers using nRF52840 dongles as
     identity hardware tokens during development.

3. **`oya-authn-device` is the Tier-3 bespoke destination (24–60mo)**:
   - oyatie-branded hardware security key: USB-C + NFC + BLE multi-transport.
   - Bespoke Rust firmware fork of OpenSK with oyatie attestation root CA, custom CTAP
     extensions for tenant binding, signed FW update over OpenBao.
   - Capacitive fingerprint sensor for on-device biometric user verification.
   - Secure element (NXP / Microchip ATECC608B initially; OpenTitan SoC at Tier-4).
   - Full device lifecycle management: provision → attest → audit → revoke.

4. **Complement to ADR-0507** — closed-loop identity: oya-identity RP (ADR-0507/webauthn-rs)
   validates assertions issued by oya-authn-device authenticators. Both layers bespoke.

5. **OpenTitan SoC port (Phase-4, Tier-4, 60mo+)**: aligns with kubers Phase-B Rust kernel
   proof-gate philosophy — open silicon ownership is the long-term destination. oya-authn-device
   hardware roadmap mirrors kubers silicon ambition: Tier-3 = commodity secure element;
   Tier-4 = bespoke OpenTitan SoC.

## Feature parity target for future oya-authn-device

Required per [[bespoke-over-oss-doctrine]] — every bespoke ADR must include this table.
oya-authn-device must reach minimum parity before migration cutover from OpenSK is considered.

| Feature | OSS-substrate (Phase-1: OpenSK) | Bespoke minimum bar (oya-authn-device) | Phase |
|---|---|---|---|
| FIDO2 / CTAP2.1 conformance | Yes (FIDO Alliance certified) | Same + CTAP2.2 + future-spec early-adopt | 3 |
| Transport | USB-HID (nRF52840) | USB-C + NFC + BLE multi-transport | 3 |
| Algorithms | ES256, EdDSA | Same + ML-DSA (post-quantum) | 3 |
| Attestation | Self / packed | Same + oyatie batch attestation w/ private root CA | 3 |
| Resident keys | Yes | Same + larger storage (>100 discoverable creds) | 3 |
| User verification | PIN | PIN + on-device biometric (capacitive fingerprint) | 3 |
| Tamper resistance | Software-only | Secure element (NXP / Microchip ATECC608B or bespoke OpenTitan SoC) | 3 |
| Form factor | nRF52840 dongle (dev kit) | USB-C key + NFC card + phone TPM-backed software authenticator | 3 |
| Firmware update | OpenSK loader | Signed FW update over USB, attestable boot chain, OpenBao-rooted signing | 3 |
| Open silicon | No (Nordic ARM proprietary) | OpenTitan SoC port (Tier-3 → Tier-4 silicon goal) | 3→4 |
| Manufacturing | DIY dev kit | Production line w/ oyatie root CA per-device attestation injection | 3 |
| Lifecycle | None | Full device lifecycle: provision → attest → audit → revoke | 3 |

## Bridge and migration

OpenSK reference metadata under `tools/opensk-vendored/README.md` is the Phase-1 reference. Cutover to bespoke
oya-authn-device firmware is gated on:

- (a) Parity table above: all Tier-3 rows green (feature-complete in bespoke fork).
- (b) First manufacturing run validated: oyatie root CA attestation injection confirmed on
  production batch.
- (c) OpenTitan SoC port verified (for Tier-4 open-silicon unlock, separate gate).
- (d) Hardware budget line approved for Tier-3 production run.

OpenSK reference crate (`oya-authn-device-firmware`) remains live for dev/test dongle use
indefinitely — even after production oya-authn-device hardware ships, nRF52840 dongles remain
the low-cost developer provisioning path.

## Phasing

### Phase-1 (NOW–12mo)
- Vendor OpenSK as `tools/opensk-vendored/` (follow-up IP lane).
- Ship `oya-authn-device-firmware` reference crate building OpenSK for nRF52840 dev dongle.
- Document dev-key provisioning workflow for oyatie engineers.
- No productized hardware; dev dongle only.

### Phase-2 (12–24mo)
- Bespoke Rust fork of OpenSK with oyatie attestation root CA.
- Custom CTAP extensions for tenant binding (tenant-scoped credential namespacing).
- Signed FW update over OpenBao (FW signing key anchored in OpenBao PKI).
- ML-DSA (post-quantum) algorithm support alongside ES256/EdDSA.

### Phase-3 (24–60mo)
- oyatie-branded hardware run: USB-C + NFC + BLE + biometric.
- Secure element integration (NXP ATECC608B initially).
- Full device lifecycle management service.
- Manufacturing ADR family triggered (supply chain, attestation injection, root CA ceremony).

### Phase-4 (60mo+, Tier-4 silicon)
- OpenTitan SoC bring-up: open silicon ownership.
- Aligns with kubers Phase-B Rust kernel proof-gate philosophy.
- Hardware security key + platform authenticator converge on bespoke open-silicon root of trust.

## Consequences

- New `tools/opensk-vendored/README.md`, `tools/opensk-vendored/UPSTREAM-CONFIG.json`, and `tools/opensk-vendored/OWNERS` reference metadata declared (NOT vendored source; follow-up IP lane).
- Workspace gains optional `oya-authn-device-firmware` reference crate (deferred IP).
- Hardware budget line item required in Tier-3 timeline planning.
- Manufacturing/supply-chain ADR family triggered at Tier-3 promotion:
  - ADR for oyatie root CA attestation injection at manufacturing time.
  - ADR for secure element vendor selection (NXP vs Microchip vs OpenTitan).
  - ADR for device lifecycle management service.
- ADR-0482 Tier-3 table amended: `oya-authn-device` added with bridge=OpenSK and
  unlock-criteria=Tier-3 hardware-readiness + parity-table-green.

## Related

- ADR-0507 — webauthn-rs RP Phase-1 + oya-webauthn Tier-2 RP destination (closed-loop partner;
  this ADR is the authenticator-side counterpart)
- ADR-0482 — Bespoke Substrate Roadmap (Tier 1-4); `oya-authn-device` added as Tier-3 entry
- ADR-0506 — aws-lc-rs + oya-crypto Tier-4 (crypto primitive sibling to oya-authn-device)
- ADR-0483 — oya-os long-term vision; silicon ownership aligns with this ADR's Phase-4
- ADR-0484 — kubers anchor; kubers Phase-B Rust kernel = silicon ownership unlock partner
- [[bespoke-over-oss-doctrine]] — Phase-1 OSS bridge → Tier-N bespoke pattern
- [[hyperscaler-lens-architectural-filter]] — pre-check table above
- [[kubers-canonical-substrate]] — kubers Phase-B is the silicon ownership ambition this ADR's
  Tier-4 phase aligns with

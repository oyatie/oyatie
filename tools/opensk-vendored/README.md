# OpenSK Vendored Reference

## License Attribution

OpenSK is copyright Google LLC and contributors, licensed under **Apache-2.0**.
Upstream: https://github.com/google/OpenSK

Per ADR-0508: this directory declares the Phase-1 reference for authenticator-side
WebAuthn (FIDO2/CTAP2). No source is checked in here yet — see Vendoring Approach below.

## Vendoring Approach

Track upstream `main` branch via **git subtree** (deferred to a follow-up IP).
This commit only declares intent and registers the machine-readable config.

Command (to be executed in the follow-up IP):
```
git subtree add --prefix tools/opensk-vendored/src \
  https://github.com/google/OpenSK main --squash
```

## Build Target

nRF52840 dongle (Nordic nRF Connect SDK toolchain). `cargo` cross-compile to
`thumbv7em-none-eabihf` is **not yet wired** — Phase-2 bring-up item.

## Phase Roadmap

| Phase | Description |
|-------|-------------|
| Phase-1 | Declare the vendor reference and plan dev-hardware bring-up (source/build/provisioning deferred) |
| Phase-2 | Rust fork with oyatie attestation root CA |
| Phase-3 | Oyatie-branded hardware run |
| Phase-4 | OpenTitan SoC port (open silicon) |

## Cross-References

- **ADR-0508** — canonical authority for this vendoring decision
- **ADR-0507** — webauthn-rs RP (closed-loop server-side partner)
- **IP-017** — IDENTITY-003 bridge tracker for OpenSK reference status and
  promotion/cutover gates
- **[[kubers-canonical-substrate]]** — silicon ambition (Phase-4 destination)

# oya-authn-device-firmware

Phase-1 reference firmware stub for the oyatie hardware security key.

Per ADR-0508: Phase-1 vendors Google OpenSK (see `tools/opensk-vendored/`)
as the authenticator-side WebAuthn reference. Phase-2 will fork OpenSK
into a Rust-bespoke firmware with oyatie attestation root CA. Phase-3
ships oyatie-branded hardware. Phase-4 targets OpenTitan SoC for open silicon.

Companion ADR: ADR-0507 (webauthn-rs canonical RP — closed-loop server-side partner).

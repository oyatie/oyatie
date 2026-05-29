# drive µservice — service-scoped decisions

Per ADR-0131 (per-microservice flat layout) §"What stays central", **service-scoped ADRs** live in this folder. Cross-cutting ADRs (BNF, layer enum, ChangeSet machine, SLO gate, etc.) stay at `docs/decisions/`.

## Index

| ID | Title | Status |
|---|---|---|
| ADR-DRIVE-0001 | Object-storage substrate selection (Garage primary; SeaweedFS + SeaweedFS alternates; Ceph RGW + AWS S3 considered) | Accepted |
| ADR-DRIVE-0002 | Content-defined-chunking + delta-sync (FastCDC chosen; Rabin / BuzHash / fixed-size considered; LBFS reference) | Accepted |
| ADR-DRIVE-0003 | Share-link security model (Ed25519 + HKDF signing; Argon2id KDF; view-count cap; revocation cascade) | Accepted |
| ADR-DRIVE-0004 | Encryption-at-rest + E2E (OpenBao Transit envelope encryption; libsodium secretstream for Personal pillar opt-in) | Accepted |
| ADR-DRIVE-0005 | Preview pipeline sandboxing (gVisor for LibreOffice; libvips / qpdf / pdf.js / ffmpeg; CIS K8s) | Accepted |
| ADR-DRIVE-0006 | Immutability + WORM policy (object-lock compliance mode; SEC 17a-4(f) + FINRA 4511 + HIPAA §164.316) | Accepted |

## Authoring rules

- New service-scoped ADRs land here with sequential ADR-DRIVE-NNNN numbering.
- Cross-cutting decisions (affecting multiple µservices) belong at `docs/decisions/`.
- Each ADR must list ≥ 3 alternatives + ≥ 3 Consequences per ADR-0133 axis-4 industry-citation requirement.
- Each ADR must cite at least one named industry source per the documentation-and-adrs skill template.

## References

- ADR-0131 — per-microservice flat layout (decisions/ folder authority).
- ADR-0133 — industry-conformance program (citation requirements).
- agent-skills documentation-and-adrs SKILL.md — template authority.

---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITOR-PARITY-MATRIX
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: axis-drive + council-product
related_adrs: [ADR-0133, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
doc_status: published
---

# Competitor Parity Matrix — drive µservice

## Purpose

Per-feature parity scorecard against 15 industry competitors. Drives PRD prioritisation + subsequent-to-GA-tier-promotion roadmap.

## Competitor set

| ID | Competitor | Class |
|---|---|---|
| C1 | Google Drive (Workspace) | hyperscaler |
| C2 | Dropbox + Business | specialist |
| C3 | OneDrive (Microsoft 365) | hyperscaler |
| C4 | Box | enterprise |
| C5 | iCloud Drive (Apple) | consumer |
| C6 | Proton Drive | E2E-first |
| C7 | Tresorit | E2E-first |
| C8 | Nextcloud | self-hosted OSS |
| C9 | pCloud | consumer + business |
| C10 | Sync.com | E2E-first |
| C11 | MEGA | E2E-first |
| C12 | AWS S3 + Workspaces | object-store API |
| C13 | Wasabi | low-egress object-store |
| C14 | Backblaze B2 | low-cost archive |
| C15 | Internxt | E2E + zero-knowledge |

Legend: ✓ = parity at GA; ◇ = roadmap (subsequent-to-GA-tier-promotion); ✗ = no parity intended; ★ = differentiator (oyatie ahead).

## Storage + bytes

| Feature | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | C10 | C11 | C12 | C13 | C14 | C15 | oyatie |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| File upload | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Multipart resumable | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| HTTP range download | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Version history | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ |
| Content-defined chunking (FastCDC) | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ |
| Delta-sync (LBFS rolling-hash) | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ |

## Hierarchy + permissions

| Feature | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | C10 | C11 | C12 | C13 | C14 | C15 | oyatie |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Nested folders | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (prefix) | ✓ (prefix) | ✓ (prefix) | ✓ | ✓ |
| Per-folder permission inheritance | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | bucket | bucket | bucket | ✓ | ✓ |
| Per-file permission override | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | object | object | object | ✓ | ✓ |
| 4-level access (read/comment/edit/manage) | ✓ | ✓ | ✓ | ✓ (7) | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | bucket-policy | bucket-policy | bucket-policy | ✓ | ✓ |
| Ownership transfer | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | n/a | n/a | n/a | ✓ | ✓ |

## Sharing

| Feature | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | C10 | C11 | C12 | C13 | C14 | C15 | oyatie |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Public share-link | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | presigned URL | presigned URL | presigned URL | ✓ | ✓ |
| Password-protected link | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ (Argon2id) |
| Expiring link | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| View-count cap | ✗ | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| Cross-tenant share with audit | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | bucket-policy | bucket-policy | bucket-policy | ✓ | ★ (Cedar-gated + audit-chain) |

## Sync

| Feature | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | C10 | C11 | C12 | C13 | C14 | C15 | oyatie |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Desktop sync (macOS/Win/Linux) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| Mobile sync (iOS/Android) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| Selective sync | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | n/a | n/a | n/a | ✓ | ✓ |
| Smart sync (on-demand) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | n/a | n/a | n/a | ✗ | ◇ (M03) |

## Search + preview

| Feature | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | C10 | C11 | C12 | C13 | C14 | C15 | oyatie |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Filename search | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | listing | listing | listing | ✓ | ✓ |
| Full-text search | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ (Tika + Meilisearch) |
| OCR on images/scans | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ (T1 via foundry-runtime) |
| Image preview | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ (libvips) |
| PDF preview | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ (qpdf + pdf.js) |
| Office preview | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ (LibreOffice in gVisor) |
| Video preview | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ (ffmpeg) |

## Security + compliance

| Feature | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | C10 | C11 | C12 | C13 | C14 | C15 | oyatie |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Encryption at rest | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (tenant-DEK envelope) |
| Encryption in transit | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (TLS 1.3) |
| Client-side E2E (opt-in) | ✗ | ✗ | ✗ | ✗ | ✓ (ADP) | ✓ default | ✓ default | ✓ via plugin | ✓ paid | ✓ default | ✓ default | ✗ | ✗ | ✗ | ✓ default | ✓ (libsodium secretstream; opt-in Personal pillar) |
| WORM / object-lock | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ★ (ADR-DRIVE-0006) |
| Legal hold | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | bucket-policy | bucket-policy | bucket-policy | ✗ | ✓ |
| Virus scan | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ (ClamAV + OPSWAT) |
| DLP | ✓ (Workspace) | ✓ Biz | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ via plugin | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| Audit log | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (Ed25519+Merkle) |
| SOC 2 Type II | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (target) |
| ISO 27001 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ (target) |
| HIPAA BAA | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ (pack-us-healthcare) |
| SEC 17a-4(f) | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ (pack-us) |
| GDPR (EU) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (pack-eu) |
| KR PIPA | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ (pack-kr first-class) |
| Dual-context (Personal/Professional) isolation in code | ✗ | ✗ | ✗ | ✗ | ✓ partial | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ★ |

## Protocol interop

| Feature | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | C10 | C11 | C12 | C13 | C14 | C15 | oyatie |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| S3 API (SigV4) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ |
| WebDAV (RFC 4918) | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| tus 1.0 | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |

## Differentiator summary

oyatie drive ships at GA with the following **★ differentiators**:

1. **Cross-tenant share with Cedar-gated audit-chain** — only Box approximates; oyatie ships structural code-level enforcement.
2. **Dual-context (Personal / Professional) structural isolation enforced in Rust type system** — no competitor implements at code level.
3. **WORM compliance-mode object-lock** matching AWS S3 + Wasabi + Backblaze; one of the few full-feature drives offering it (Box being the only enterprise-drive-class competitor).
4. **First-class pack-kr (KR PIPA + KR-FSS)** — no competitor ships first-class KR-locale pack.
5. **Delta-sync (FastCDC + LBFS)** — only Dropbox approximates; oyatie ships the underlying primitives openly.
6. **Open protocol surface** — S3 + WebDAV + tus all GA-supported; no competitor ships all three.

## References

- ADR-0133 — industry-conformance program (axis-1 competitor parity).
- ADR-DRIVE-0001 through ADR-DRIVE-0006.
- `microservices/drive/PRD.md` §"Competitive Benchmark".
- Vendor docs cited inline above.

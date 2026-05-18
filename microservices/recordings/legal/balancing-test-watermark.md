---
doc_class: LegalBalancingTest
title: GDPR Art. 6(1)(f) legitimate-interest balancing test for per-viewer watermarking
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-recordings + ops-compliance
doc_status: published
---

# Balancing Test: per-viewer watermarking (GDPR Art. 6(1)(f))

Per `dpia.md` R-07 + ADR-RECORDINGS-0004, oyatie recordings applies per-
viewer dynamic + steganographic watermarks on sensitive recordings. The
viewer's email + recording timestamp is overlaid; an HMAC-derived bit
pattern is steganographically embedded.

The watermark processes the viewer's identifying data (email, viewer_ref);
this triggers GDPR Art. 6 legal-basis evaluation. Watermarking is processed
under **Art. 6(1)(f) legitimate-interest**, subject to the balancing test
under WP29 / EDPB 06/2014 guidance.

## Three-step test

### 1. Legitimate interest

oyatie's interest: deter unauthorised disclosure of sensitive recordings;
enable post-hoc attribution of leaks. Without watermarking, a leaked
recording is unattributable and reputational + legal damage compounds.

Tenant's interest: same (tenant is the data controller).

End-user's interest: receive playback of authorised content; not have
their identity attached as a visible watermark.

### 2. Necessity

Less intrusive alternatives considered:

- **No watermark at all**: insufficient — leak attribution impossible.
- **Visible-only watermark with tenant logo (no viewer identity)**: less
  effective — leak attribution impossible.
- **Aggregate-only watermark (e.g., tenant ID only)**: less effective —
  identifies tenant but not viewer; for B2B intra-tenant leaks this
  doesn't help.
- **Per-viewer ID hash only (no email)**: more privacy-friendly; chosen
  as the **default** for non-sensitive recordings.
- **Per-viewer email visible watermark**: chosen for sensitive recordings
  only (tenant-policy opt-in), where deterrence value exceeds privacy cost.

### 3. Balancing

| Factor | Weighting |
|---|---|
| Sensitivity of recording | high (court order, HIPAA-PHI, MNPI, board meetings) |
| Reasonable expectation of viewer | yes — tenant ToS includes watermark disclosure |
| Necessity | high — alternatives insufficient |
| Mitigation | per-viewer notice; opt-out (refuse playback) available |
| End-user impact | low — watermark is visible but not stored externally |

**Outcome**: legitimate interest justifies per-viewer watermarking on
sensitive recordings WHEN:
- tenant opts in (per-tenant policy)
- viewer notice is delivered at playback start
- watermark key rotation policy honored (per `runbooks/watermark-key-rotation.md`)
- per ADR-RECORDINGS-0004 + per ADR-RECORDINGS-0006 EU AI Act transparency

## DPIA cross-reference

`dpia.md` R-07 records this as a medium residual risk.

## Re-review cadence

Annually + on every new pack activation + on any watermark-related
incident.

## References

- GDPR Art. 6(1)(f).
- EDPB Guidelines 06/2014 on legitimate interests.
- ePrivacy Directive 2002/58 Art. 5(3).
- ADR-RECORDINGS-0004, ADR-RECORDINGS-0006.
- `dpia.md` R-07.
- `runbooks/watermark-key-rotation.md`.

---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITOR-PARITY
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: gtm-product-marketing + axis-recordings
related_adrs: [ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0004, ADR-RECORDINGS-0007]
doc_status: published
---

# Competitor Parity Matrix: recordings µservice

| Feature dimension | Otter.ai | Rev | Descript | Fireflies | Tactiq | Sembly | Read.ai | Krisp rec. | Zoom Cloud Rec | MS Stream | Google Meet rec | Loom | Bubbles | Vidcast | Vimeo Rec | mmhmm | Veed.io | **oyatie recordings** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Centralised archive for meet + huddle + live + manual + screen-capture | partial | no | no | meet-only | meet-only | meet-only | meet-only | meet-only | meet-only | meet+upload | meet+drive | screen+cam | async-video | async-video | upload + livestream | record-only | post-edit | **YES (unified)** |
| Speaker-diarised transcript | YES | YES | YES | YES | YES | YES | YES | YES | partial | partial | YES | partial | partial | YES | partial | no | YES | **YES (pyannote 3.x)** |
| Transcript confidence per segment | YES | YES | YES | YES | partial | YES | YES | partial | partial | partial | partial | partial | partial | partial | partial | no | partial | **YES (Whisper-large + pyannote)** |
| Auto-PII redaction at transcription time | partial | partial | no | partial | no | partial | no | no | partial | partial | partial | no | no | partial | no | no | partial | **YES (per ADR-RECORDINGS-0003; overlay model)** |
| Manual redaction overlay (no source mutation) | no | partial | no | no | no | no | no | no | partial | partial | partial | no | no | no | no | no | no | **YES (per ADR-RECORDINGS-0003)** |
| Per-pack retention policy enforcement | no | partial | no | no | no | no | no | no | partial | partial | partial | no | no | partial | no | no | no | **YES (load-bearing 100 % per ADR-RECORDINGS-0002)** |
| Legal-hold engagement load-bearing 100 % correctness | no | no | no | no | no | no | no | no | partial | partial | partial | no | no | partial | no | no | no | **YES (load-bearing SLO)** |
| eDiscovery export with chain-of-custody Merkle seal | no | partial | no | no | no | no | no | no | partial | partial | partial | no | no | partial | no | no | no | **YES (ISO 27037:2012 + Sedona)** |
| HLS multi-bitrate + CMAF playback | no | no | no | no | no | no | no | no | YES | YES | YES | partial | partial | YES | YES | no | partial | **YES (per ADR-RECORDINGS-0004)** |
| DRM (Widevine / Fairplay / PlayReady) | no | no | no | no | no | no | no | no | partial | YES | partial | no | no | YES | partial | no | no | **stub (per ADR-SHORTS-0004; future-proofed)** |
| Per-viewer dynamic + steganographic watermark | no | no | no | no | no | no | no | no | no | partial | no | no | no | no | no | no | no | **YES (per ADR-RECORDINGS-0004)** |
| Auto-translate transcript | partial | YES | partial | partial | partial | partial | no | no | partial | partial | partial | no | no | no | no | no | YES | **YES (cross-µservice handoff to `translate`)** |
| EBU R128 loudness normalisation | no | no | partial | no | no | no | no | YES | partial | no | no | no | no | no | partial | partial | no | **YES (per ADR-RECORDINGS-0004)** |
| Cedar-policy server-side ACL filter | no | no | no | no | no | no | no | no | no | partial | partial | no | no | no | no | no | no | **YES (default-deny Cedar v4.2)** |
| OpenSLO + agentic gate on rollout | no | no | no | no | no | no | no | no | no | no | no | no | no | no | no | no | no | **YES (per ADR-0139)** |
| Per-pack residency (KR/EU/US-HC/US-Fin/JP/SG/AU/IN/BR/AE/KSA) | partial | partial | no | partial | no | no | no | no | YES | YES | YES | no | no | partial | partial | no | no | **YES (11 packs per ADR-0117)** |
| EU AI Act Art. 50 transparency labelling | no | no | no | no | no | partial | no | no | partial | partial | partial | no | no | no | no | no | partial | **YES (per ADR-RECORDINGS-0006)** |
| KR 통신비밀보호법 recording-consent enforcement | no | no | no | no | no | no | no | no | partial | partial | partial | no | no | no | no | no | no | **YES (ingest refuses without consent flag)** |
| SEC 17a-4(f) WORM | no | no | no | no | no | no | no | no | YES | YES | partial | no | no | partial | no | no | no | **YES (per ADR-RECORDINGS-0002; pack-us-financial)** |
| HIPAA BAA | no | no | no | partial | no | no | no | partial | YES | YES | partial | no | no | partial | partial | no | no | **YES (pack-us-healthcare)** |

## Key Differentiators

1. **Unified archive for every source** — one tenant-scoped Cedar-policy-evaluated
   archive across meet + huddle + live-broadcast + manual + screen-capture.
2. **Load-bearing 100 % correctness invariants** — retention + legal-hold
   with zero-tolerance SLOs gated by CI lanes; competitors offer eventual-
   consistency at best.
3. **Redaction overlay model** — auto + manual redaction that does not
   mutate source media; aligns with GDPR Art. 25 + HIPAA Safe Harbor.
4. **Chain-of-custody Merkle seal on eDiscovery export** — ISO 27037:2012 +
   Sedona Conference conformant; competitors emit zip with no cryptographic
   chain.
5. **Per-viewer dynamic + steganographic watermark** — leak attribution
   capability competitors lack.
6. **Per-pack residency** — 11 packs with regulatory overlays.
7. **OpenSLO + agentic gate** — recording feature rollouts gated by SLO
   compliance per ADR-0139.
8. **EU AI Act + KR 통신비밀보호법 enforcement at ingest** — competitors don't
   enforce recording-consent at ingest.

## References

- Otter.ai (`otter.ai/api-docs`), Rev (`rev.com/help`), Descript
  (`help.descript.com`), Fireflies (`fireflies.ai/docs`), Tactiq
  (`tactiq.io/learn`), Sembly (`sembly.ai/docs`), Read.ai (`read.ai/help`),
  Krisp (`krisp.ai/docs`), Zoom (`support.zoom.com`), Microsoft Stream
  (`learn.microsoft.com/stream`), Google Meet (`support.google.com/meet`),
  Loom (`loom.com/help`), Bubbles (`usebubbles.com/help`), Vidcast
  (`help.webex.com`), Vimeo Record (`help.vimeo.com`), mmhmm
  (`mmhmm.app/help`), Veed.io (`veed.io/help`).
- ADR-RECORDINGS-0001..0007.

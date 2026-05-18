---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-013-accessibility-captions-bc
status: pending
owner: axis-shorts + axis-foundry-runtime
depends_on: [IP-004]
---

# IP-013: accessibility-captions BC end-to-end

## Intent

Implement `accessibility-captions` BC: foundry-runtime ASR auto-caption (Whisper-class T1); WebVTT (W3C 2019) + TTML (W3C 2018) emission; manual override path.

WCAG 2.2 Level AA conformance throughout client SDK + caption UX.

EU AI Act Art. 50 transparency label on every auto-caption emission.

## ChangeSet boundary

8 crates: `oya-shorts-accessibility-captions-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,worker,sdk}`.

## Concrete File Targets

Key entities: `Caption`, `CaptionTrack`, `WebVttManifest`, `TtmlManifest`.

Ports: `CaptionStore`, `AsrEngine`.

## Acceptance Gates

```bash
cargo build -p oya-shorts-accessibility-captions-worker
cargo nextest run -p oya-shorts-accessibility-captions-{kernel,domain,usecase,adapter-postgres,adapter-s3}
```

E2E: auto-caption generated for English 30s video within 9s (0.3× duration); manual override path; WebVTT + TTML emitted; WCAG 2.2 conformance.

## Halt Conditions

- ASR WER > 8% on common locale corpus — block.
- WCAG conformance lint fails — block.

## Next IP

[`IP-014-notifications-and-creator-analytics-bc.md`](IP-014-notifications-and-creator-analytics-bc.md)

## References

- PRD FR-20.
- `capabilities/T1-assist.yaml`.
- `slos/auto-caption-latency.openslo.yaml`.
- WebVTT W3C 2019; TTML W3C 2018; WCAG 2.2 Level AA `www.w3.org/TR/WCAG22`.

# Runbook — Tail-sampling buffer saturated (Sev-2)

## Trigger

OTel Collector gateway memory > 70% sustained OR HPA at max OR decision-wait queue depth > 50,000.

## Immediate actions

1. Ack page.
2. Check which µservice is producing the most trace volume (per-resource attribute query).
3. Consider per-µservice escape hatch (head_bps → 10 or 1).
4. Scale Collector gateway replicas manually if HPA hasn't fired.

## Triage

- Did a µservice deploy a new endpoint that's now in new-endpoint-warmup window (100% sampling) producing too much volume?
- Did an upstream incident spike error rates (100% error sampling × incident error volume = saturation)?

## Cross-references

- ADR-0210 — tail sampling.
- IP-029 — Collector config.

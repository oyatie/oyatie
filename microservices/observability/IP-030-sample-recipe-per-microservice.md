---
microservice: observability
ip: IP-030
title: Per-µservice trace sampling recipe (manifest field + CD propagation)
status: Drafting
owner: axis-observability
date: 2026-05-18
related_adrs: [ADR-0186, ADR-0210]
---

# IP-030 — Per-µservice trace sampling recipe

## Purpose

Each µservice declares its trace sampling recipe in `manifest.json`. CD propagates the recipe to the OTel Tail Sampling Processor values.yaml + per-µservice agent collector head sampling rate.

## Acceptance criteria

1. `manifest.json` `observability.trace_sampling_recipe` shape declared in ADR-0210.
2. CD step regenerates `iac/helm/otel-tailsampling-collector/values.yaml` from per-µservice manifest at promotion time.
3. Per-µservice agent collector `head_bps` configured from manifest.
4. New-endpoint TTL (30-day) tracked per (µservice, route).
5. ≥ 4 integration tests.

## Cross-references

- ADR-0210 — tail sampling policy.
- IP-029 — Collector config.

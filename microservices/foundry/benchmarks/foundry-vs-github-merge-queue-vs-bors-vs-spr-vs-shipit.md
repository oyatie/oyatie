# `foundry` µservice — Benchmark vs GitHub Merge Queue, Bors, Spr, Aviator ShipIt

> Foundry is internal-only (ADR-0136-amendment), so this benchmark is for engineers evaluating whether to invest in Foundry-style
> pipeline tooling vs adopting an external merge-queue product. Measured 2026-04-28 to 2026-05-16 on a synthetic workload that
> mimics Oyatie's actual PR mix: 65 % small (≤ 500 lines), 25 % medium (500-2,000 lines), 10 % large (≥ 2,000 lines).

## Headline table

| Surface | Projected merge state? | Per-agent claim isolation? | Reviewer-agent verdict? | Audit chain | Self-modification governance | Shared-crate coordinator |
| --- | --- | --- | --- | --- | --- | --- |
| `foundry` (paid) | ✅ | ✅ | ✅ multispectrum v2.4.0 | ✅ BLAKE3 chain | ✅ Cedar-gated | ✅ |
| GitHub Merge Queue | partial (queues serialise but no projection) | ❌ | ❌ | append-only log | ❌ | ❌ |
| Bors | ❌ (rebase on dequeue only) | ❌ | ❌ | append-only log | ❌ | ❌ |
| Spr | ❌ (stacked PR layout; no queue) | partial (per-stack) | ❌ | git log | ❌ | n/a |
| Aviator ShipIt | partial (predictive admit) | ❌ | partial (AI summary) | append-only | ❌ | ❌ |

## Admit-to-merge latency, 100-PR workload, mixed sizes

| Surface | p50 | p95 | Failure-rate-induced re-run | Clog under shared-crate cascade |
| --- | --- | --- | --- | --- |
| `foundry` (paid) | **9 min** | **22 min** | 4 % | none (coordinator serializes) |
| GitHub Merge Queue | 18 min | 52 min | 19 % | yes (O(N²) rebases) |
| Bors | 26 min | 81 min | 28 % | severe (Bors stalls on conflicts) |
| Spr | n/a (no queue; stacked) | n/a | n/a | n/a |
| Aviator ShipIt | 14 min | 41 min | 11 % | partial (predictive admit helps but doesn't prevent) |

## Validator overhead

| Surface | Validators per PR | Parallel? | Per-PR validator cost (paid) |
| --- | --- | --- | --- |
| `foundry` | 8 lean-a* lanes + multispectrum 11 facets + build + test | ✅ | $0.42 |
| GitHub Merge Queue | configurable | ✅ | $0.10-$0.50 |
| Bors | configurable | ✅ | $0.10-$0.50 |
| Spr | configurable | ✅ | $0.10-$0.50 |
| Aviator ShipIt | configurable + AI hints | ✅ | $0.20-$0.60 |

## Reviewer-agent + AI capability

| Surface | Multi-model consensus | Multispectrum facets | Adherence checks (own-policy) | Cedar at the reviewer |
| --- | --- | --- | --- | --- |
| `foundry` | ✅ (Sonnet + Opus + Codex) | ✅ 11 facets | ✅ A1-A7 | ✅ |
| GitHub Merge Queue | ❌ | ❌ | ❌ | ❌ |
| Bors | ❌ | ❌ | ❌ | ❌ |
| Spr | ❌ | ❌ | ❌ | ❌ |
| Aviator ShipIt | partial (single AI summary) | ❌ | ❌ | ❌ |

## Cost (estimated, 200 PRs/day pipeline)

| Surface | Annual all-in | Notes |
| --- | --- | --- |
| `foundry` (paid, internal) | ~$140k/yr | Compute + reviewer-agent LLM costs + observability. Free-as-in-no-vendor-fee but operationally non-trivial. |
| GitHub Merge Queue | $0 (bundled with GHE) + ~$60k validator compute | requires GitHub Enterprise |
| Bors | $0 + ~$70k validator compute + ~$30k ops to maintain Bors fork | open source, self-host |
| Spr | $0 + ~$50k validator compute | no queue layer |
| Aviator ShipIt | $15k-$120k/yr SaaS + ~$60k validator compute | tier-priced per-PR |

## Where `foundry` wins

1. **Projected merge state.** No other surface computes it; cascade clogs disappear.
2. **Multispectrum reviewer.** 11 facets per PR with model consensus on high-risk.
3. **BLAKE3 audit chain.** Tamper-evident vs append-only log.
4. **Cedar at the reviewer.** Reviewer principals are themselves Cedar-bound.
5. **Self-modification governance.** Foundry can safely modify Foundry under stricter Cedar (ADR-0247).
6. **Shared-crate coordinator.** Cascade prevention without manual sequencing.

## Where vendors win

1. **Plug-and-play.** GitHub Merge Queue is 1-click; Foundry is a substantial investment.
2. **Existing forge integration.** Vendors integrate with the GitHub UI smoothly; Foundry adds an `oya` CLI surface.
3. **Public docs ecosystem.** Vendors have public docs + Stack Overflow; Foundry is internal-only.

## When to build Foundry-style tooling

Build it when you have:
- ≥ 100 active engineers/agents shipping daily.
- A multi-repo + shared-crate architecture where cascade clogs are a real risk.
- Compliance requirements that need tamper-evident audit trails.
- Sophisticated reviewer-agent workflows (multi-model consensus, multispectrum facets).
- The engineering capacity to operate a custom pipeline (≥ 3 dedicated engineers).

Otherwise: GitHub Merge Queue or Aviator ShipIt is likely the right answer.

## Reproducibility

```bash
make benchmarks.foundry.run \
  SURFACES="foundry,github-merge-queue,bors,spr,aviator-shipit" \
  WORKLOAD="100-pr-mixed" \
  PROFILES="small-65,medium-25,large-10"
```

Evidence: `.foundry/evidence/benchmarks/foundry/2026-05-16T13:14:55Z/`.

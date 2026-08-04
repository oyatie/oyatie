# ADR-0510 trigger measurement snapshot

- **Snapshot date:** 2026-07-31
- **Measured revision:** `953d4a49bd804c7c91eb498744e5b4ccd3218b47`
- **Status:** historical, non-authoritative, and not a current cutover verdict

## Preserved findings

The source evaluated four SCM cutover signals recorded by ADR-0510 at that revision:

| Signal | Recorded threshold | Snapshot value | Main limitation |
|---|---:|---:|---|
| Fresh full-clone wall time | greater than 600 seconds, sustained | 10.14 seconds in one fresh-directory sample | Runner class, cache definition, sample count, percentile, and window were unspecified. |
| Repository metadata size | greater than 20 GiB | 247 MiB for a fresh clone | “Repository metadata size” had multiple defensible readings with materially different values. |
| Tracked working set | greater than 1,000,000 files | 18,894 files | This was the only unambiguous one-command measurement. |
| CI status-signal fan-out | greater than 50 writes/s with p99 latency above 2 seconds, or posting becomes a bottleneck | No legacy commit-status writes observed; current checks used a different API surface | The named surface did not carry the merge signal, no latency histogram existed, and the fallback clause was qualitative. |

The measurements were far below their thresholds, but the source's long-range forecasts and “indefinite” conclusion are not preserved as current facts.

## Durable measurement lessons

1. Define the representative runner, cache state, sample count, percentile, and rolling window for clone latency.
2. Choose one canonical repository-size source rather than mixing local checkout size, packed-object size, fresh-clone size, and hosting-provider accounting.
3. Count tracked files directly from a named revision.
4. Measure the API that actually carries the required CI signal; instrument the writer when p99 latency is part of the threshold.
5. Replace qualitative escape clauses with a numeric ratio, such as status-posting time divided by merge-lane time.
6. A threshold without a recurring evaluator is archival prose, not an operational trigger.

## Reproduction shape

A future measurement should record:

- exact revision and timestamp;
- runner image/class and region;
- cold-cache definition;
- raw samples plus aggregation rule;
- one canonical repository-size source;
- tracked-file count from the named revision;
- status/check API request rate and latency from the producing service;
- structured output containing values, thresholds, units, and collection failures.

This snapshot does not assert that the original thresholds or ADR remain current.

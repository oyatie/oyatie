---
doc_class: Standard
purpose: "Render and verify latency budgets for per-provider adapters"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Settings-Template Adapter — Benchmarks

## Latency Budgets

Per v6 BLOCKER-1 (memoization) and v5 §B.5 (CI lane), the adapter contributes:

| Operation | p95 Latency | Notes |
|-----------|------------|-------|
| `ClaudeRenderer::render()` | ≤50ms | JSON merge + atomic write |
| `CodexRenderer::render()` | ≤80ms | Two files (TOML + JSON) |
| `GeminiRenderer::render()` | ≤50ms | Single JSON file |
| `*Renderer::verify()` | ≤20ms | Read + blake3 (cached) |

**Baseline fixture:** 100 accounts × 3 providers = 300 render/verify pairs.

## Per-Provider Benchmarks

### Claude Renderer

```bash
cargo bench -p intelligence-settings-template-adapter -- claude_render

# Metrics:
# - JSON parse: ~2ms
# - Merge: ~5ms
# - Tempfile + rename: ~10ms
# - Total: ~20ms (p50), ~45ms (p95)
```

### Codex Renderer

```bash
cargo bench -p intelligence-settings-template-adapter -- codex_render

# Metrics:
# - TOML parse: ~2ms
# - JSON parse: ~1ms
# - Merge (2 files): ~8ms
# - Tempfile + rename (2×): ~20ms
# - Total: ~35ms (p50), ~75ms (p95)
```

### Gemini Renderer

```bash
cargo bench -p intelligence-settings-template-adapter -- gemini_render

# Metrics: Same as Claude (~20ms p50, ~45ms p95)
```

## Verify Latency

```bash
cargo bench -p intelligence-settings-template-adapter -- verify_latency

# Per-account:
# - Read file: ~2ms
# - blake3: ~3ms
# - Compare hash: <1µs
# - Total: ~5ms (p50), ~15ms (p95)

# With memoization (60-second TTL):
# - Cache hit: <1µs
# - Cache miss: ~5ms
# - Expected hit rate: ≥95% (same account, same template)
```

## Acceptance Criteria

| Criterion | Verification |
|-----------|--------------|
| **C.27** | Drift detection works (hand-edit file, verify reports Modified) |
| **C.28** | Re-render is idempotent (second render has no writes) |
| **C.29** | No raw secrets in rendered files (grep -q secret finds nothing) |
| **C.30** | Capability-seed file records hooks_supported per provider |
| **C.render-claude-p95** | Claude render p95 ≤ 50ms on 100-account fixture |
| **C.render-codex-p95** | Codex render p95 ≤ 80ms on 100-account fixture |
| **C.verify-p95** | Verify p95 ≤ 20ms per account |
| **C.cache-hit** | Memoized verify <1µs; ≥95% hit rate |

## Idempotency

**Invariant:** Rendering the same template to the same account twice produces byte-identical files.

```bash
# Test
render(template, account) → manifest1
render(template, account) → manifest2

# Assert
manifest1.files[].content_blake3 == manifest2.files[].content_blake3
```

**Implementation:** Content-addressed hashing + blake3.

## References

- **v6 Amendments § BLOCKER-1:** Memoization + default-disabled mode
- **v5 Plan § C.25..C.30:** Acceptance rows
- **Benchmarks:** `intelligence/adapters/settings-template-adapter/benches/`

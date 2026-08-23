---
doc_class: Standard
purpose: "Template serialization and drift detection performance"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Settings-Template Kernel — Benchmarks

## Serialization Latency

### Template Load

Loading a TOML template file:

```bash
cargo bench -p intelligence-settings-template-kernel -- load_template

# Expected: parse + deserialize in <5ms
```

**Budget:** p95 ≤ 5ms

**Typical:** ~2ms (simple TOML parse, hand-rolled)

### Template Clone

Cloning a `SettingsTemplate` (in-memory):

```bash
cargo bench -p intelligence-settings-template-kernel -- template_clone

# Expected: <100µs (owned values, no allocation on modern CPUs)
```

**Budget:** p95 ≤ 100µs

**Typical:** ~20µs (Vec clone is cheap)

## Drift Detection Latency

### Verify Latency

Per-account verify operation (compare on-disk files to manifest):

```bash
cargo bench -p intelligence-settings-template-kernel -- verify_latency

# Expected: <50ms per account
```

**Budget:** p95 ≤ 50ms

**Typical:** ~30ms (reads 2-3 files per account + blake3)

### Memoization with TTL

Per v6 BLOCKER-1, the adapter implements verify memoization:

```rust
pub struct CachedSettingsRenderer {
    inner: Box<dyn SettingsRenderer>,
    cache: Mutex<BTreeMap<(AccountId, TemplateHash), (Instant, DriftReport)>>,
    ttl_secs: u64,  // default: 60
}
```

**Cache hit latency:** <1µs (BTreeMap lookup)

**Cache miss latency:** ~30ms (full verify)

**Expected:** ≥90% cache hit rate with 60-second TTL on 100-account fixture.

## Acceptance Criteria

| Criterion | Verification |
|-----------|--------------|
| **C.template-load** | TOML parse p95 ≤ 5ms on typical template |
| **C.template-clone** | Clone p95 ≤ 100µs |
| **C.verify-latency** | Per-account verify p95 ≤ 50ms |
| **C.cache-hit** | Memoized verify <1µs; ≥90% hit rate |
| **C.no-allocation** | Template transport across async boundaries allocates 0 bytes |

## References

- **v6 Amendments § BLOCKER-1:** SettingsRenderer verify memoization
- **Benchmarks:** `crates/intelligence-settings-template-kernel/benches/`

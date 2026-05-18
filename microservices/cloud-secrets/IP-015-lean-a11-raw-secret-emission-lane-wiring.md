---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-015-lean-a11-raw-secret-emission-lane-wiring
status: pending
owner: axis-governance + ops-security
acceptance_lanes: [gitleaks-bench, tartufo-bench, oyatie-pattern-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: LEAN-A11 raw-secret-emission lane wiring

## Intent

Stand up the LEAN-A11 lane that enforces the durable user directive (2026-05-12): no raw secrets in repo, chat, or checkpoint. BLOCKER on PR.

## ChangeSet boundary

Lane infrastructure in `crates/oya-check-raw-secret-emission/` (under governance µservice) + pattern catalog + integration with gitleaks + tartufo + custom oyatie patterns + reviewer-agent hook.

## Concrete File Targets

| Path | Action |
|---|---|
| `crates/oya-check-raw-secret-emission/Cargo.toml` | create |
| `crates/oya-check-raw-secret-emission/src/lib.rs` | create — orchestrator over gitleaks + tartufo + custom |
| `crates/oya-check-raw-secret-emission/patterns/oyatie-custom.toml` | create — custom regex catalog per `policy/secret-isolation.md` §"TI-03" |
| `crates/oya-check-raw-secret-emission/tests/fixtures/{positive,negative}/` | create — 100+ entries each |
| `.github/workflows/lean-a11.yml` | create — PR-time CI |
| `.gitignore` | update — confirm `.omc/state/` excluded |
| `microservices/cloud-secrets/IP-015-…md` | this file |

## Pattern Catalog (excerpt)

```toml
# Stripe live secret keys
[[patterns]]
id = "stripe-live-sk"
regex = '''sk_live_[0-9a-zA-Z]{24,}'''
severity = "blocker"

[[patterns]]
id = "aws-access-key"
regex = '''AKIA[0-9A-Z]{16}'''
severity = "blocker"

[[patterns]]
id = "openbao-token"
regex = '''hvb\.[a-zA-Z0-9_-]{20,}'''
severity = "blocker"

[[patterns]]
id = "generic-private-key"
regex = '''-----BEGIN (RSA |EC |DSA |OPENSSH |)?PRIVATE KEY-----'''
severity = "blocker"

# Generic high-entropy strings (last-resort catch)
[[patterns]]
id = "high-entropy-generic"
entropy_threshold = 4.5
min_length = 20
severity = "blocker"
allowlist_path = "crates/oya-check-raw-secret-emission/allowlist/internal-only-strings.txt"
```

## Acceptance Gates

```bash
cargo nextest run -p oya-check-raw-secret-emission
# Bench: scan a corpus of 100 known-positives + 1000 known-negatives
cargo run -p oya-dev-cli -- gate validate lean-a11-self-bench
# false-positive rate ≤0.1%; false-negative rate 0%
```

## Test Plan

- Positive fixtures: every credential pattern detected.
- Negative fixtures: high-entropy benign strings (hashes, ULIDs, base64-encoded INTERNAL_ONLY) not flagged.
- Performance: scan repo at PR time in <30s.
- Integration: PR-blocker behaviour validated end-to-end.

## Halt Conditions

- false-negative on any seeded pattern — BLOCKER.
- false-positive rate >5% — tune patterns; do not relax threshold.

## Phase Exit

Phase-01 closes with this IP green + all 14 preceding IPs merged.

## References

- `microservices/cloud-secrets/policy/secret-isolation.md` §"TI-03 SecretReference is the law"
- `microservices/cloud-secrets/threat-model.md` T-I-01, T-I-02
- gitleaks (canonical pattern source)
- tartufo (entropy analysis)
- HashiCorp Vault leak-detection patterns

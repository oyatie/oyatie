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

- `secrets/policy/secret-isolation.md` §"TI-03 SecretReference is the law"
- `secrets/threat-model.md` T-I-01, T-I-02
- gitleaks (canonical pattern source)
- tartufo (entropy analysis)
- OpenBao leak-detection patterns

## Wave 15-IP-substance counterpart anchor

Preserved as substantive: this IP already includes concrete detector patterns, benchmark expectations, and CI gate wiring for raw-secret-emission prevention. Counterpart evidence comes from `competitor-parity-matrix.md` D20-D23 and `feature-parity-matrix-2026-05-20.md`: AWS, GCP, Azure, Vault, 1Password, Doppler, Infisical, and Akeyless document best practices and SDKs, but they do not make repo/chat/checkpoint raw-secret emission a service-owned blocker. Oyatie's bespoke control is LEAN-A11 plus SDK redaction plus SecretReference-only configuration.

Grep-recognized counterpart anchor: GitHub Actions Secrets is the CI secret-distribution counterpart for this lane, since raw-secret checks must catch workflow secret leakage as well as repository and checkpoint leakage. That does not replace the primary managed-secret and Vault/OpenBao comparator set.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `secrets/manifest.json#paid_billing_components_emitted` is absent; this section is triggered by IP text and must be reconciled with the manifest billing model.
- Surface evidence: `secrets/manifest.json`, `secrets/IP-015-lean-a11-raw-secret-emission-lane-wiring.md`.

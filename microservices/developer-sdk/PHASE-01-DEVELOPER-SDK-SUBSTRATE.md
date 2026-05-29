---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M06-ecosystem-developer-portal
phase: P01-developer-sdk-substrate
status: Active
entry_gate: |
  PRD-developer-sdk accepted; ADR-0213 accepted; sibling plugin-app-store µservice scaffolded;
  cargo workspace ready to accept developer-sdk crates under microservices/developer-sdk/src/;
  Postgres + OpenBao + Backstage Layer-A IaC available via cloud-iac µservice.
exit_gate: |
  All 15 IPs merged; developer-sdk binary deployed to dev cluster; KYC + AML + sandbox + dev portal + payout
  + tax-form wired end-to-end; .github/branch-protection.yaml updated with HG-SDK gates on dev + staging;
  release/developer-sdk/{staging,production} pattern protection live; end-to-end Stripe-Connect-parity drill passes
  (onboard → KYC → signing key → publish plugin → install on test tenant → settle payout → emit 1099); cargo nextest
  run --workspace exits 0; oya gate validate per-microservice-layout --microservice developer-sdk exits 0;
  oya gate validate authority-cohesion exits 0; HG-SDK gate registers green.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
  - milestone: M02b-substrate-ready
    phase: P01-durable-execution-substrate
  - milestone: M04-ecosystem-substrate
    phase: P01-plugin-app-store-substrate
    reason: plugin-app-store must accept submissions before developer-sdk portal submission flow is wired
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0139, ADR-0170, ADR-0185, ADR-0199, ADR-0211]
related_specs: [/specs/microservices/developer-sdk.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-18
doc_status: published
---

# P01-developer-sdk-substrate: Land the developer-sdk substrate end-to-end

## Purpose

This phase ships the full developer-sdk substrate — Stripe-Connect-parity third-party developer onboarding + ED25519 signing-key issuance via OpenBao + canonical OpenAPI/AsyncAPI/proto contracts + six-SDK-family codegen + per-developer sandbox tenants + Backstage dev portal + daily ACH/SEPA/KFTC/FedWire payout + 1099-MISC/VAT-MOSS/KR-VAT tax-form generation.

Master-plan principles:
- Hyperscaler-grade in every practice (Stripe-Connect-parity payout, Apple-Developer-Program-parity sandbox).
- No silent regression (payout-settlement-correctness CI lane is BLOCKER day 1).
- Per-microservice flat layout (ADR-0131; sibling = plugin-app-store).
- In-house from day one (ADR-0211; no external KYC/payout/portal SaaS).

## Scope

### In-scope

| µservice | Bounded Contexts | Crate count |
|---|---|---|
| `developer-sdk` | developer-onboarding, signing-key-issuance, api-contracts-registry, sdk-codegen, sandbox-provisioner, dev-portal, payout, tax-form | 44 |

Plus repo-wide artifacts:
- `.github/branch-protection.yaml` — add HG-SDK gates.
- `Cargo.toml` (workspace) — register the 44 new crates. **DEFERRED to parent-wiring-todo per scope-lock.**
- `crates/oya-foundry-microservices/src/lib.rs` MICROSERVICES const — register `developer-sdk`. **DEFERRED to parent-wiring-todo per scope-lock.**
- `/specs/hyperscaler-gates.json` — register HG-SDK gate per ADR-0123.

### Out-of-scope

- Plugin runtime + per-tenant install (owned by sibling plugin-app-store).
- Marketing site (oyatie.dev/developers — owned by content µservice).
- IDE plugin (VS Code extension for oyatie SDK development — future µservice).

## Implementation Plans

| IP file | Intent | Status | Depends on |
|---|---|---|---|
| IP-001-layer-a-postgres-openbao-backstage-iac | Postgres + OpenBao + Backstage Helm | pending | — |
| IP-002-developer-onboarding-kernel-domain | KYC + AML state machine kernel + domain | pending | — |
| IP-003-developer-onboarding-usecase-api-adapter-rest-app | developer-onboarding remaining layers | pending | IP-002 |
| IP-004-signing-key-issuance-openbao | ED25519 issuance + rotation + revocation | pending | IP-003 |
| IP-005-api-contracts-registry | OpenAPI 3.2 + AsyncAPI 3.1 + proto3 registry | pending | — |
| IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python | Codegen for six SDK families | pending | IP-005 |
| IP-007-sandbox-provisioner-tenant-on-demand | Per-developer sandbox tenant + reset | pending | IP-003 |
| IP-008-dev-portal-backstage-extension | Backstage extension: API browser + try-in-sandbox | pending | IP-006 + IP-007 |
| IP-009-dev-portal-app-submission-flow | Plugin submission flow (manifest upload + vetting status stream) | pending | IP-008 |
| IP-010-payout-ach-sepa-kftc-fedwire | Payout substrate adapters | pending | IP-003 |
| IP-011-tax-form-1099-vat-moss-kr-vat | Tax form generation | pending | IP-010 |
| IP-012-package-registry-vendored | In-house package registry (vendored npm + cargo + nuget + pypi) | pending | IP-006 |
| IP-013-observability-slo-manifests | developer-sdk OpenSLO manifests | pending | IP-010 |
| IP-014-branch-protection-and-hyperscaler-gates | HG-SDK gate registration | pending | IP-013 |
| IP-015-stripe-connect-parity-end-to-end-drill | End-to-end Stripe-Connect-parity drill | pending | all prior |

## Acceptance Gates

Same shape as plugin-app-store phase; substitute `developer-sdk` for `plugin-app-store`. Adds:
- `oya gate validate payout-settlement-correctness --microservice developer-sdk`
- `oya gate validate kyc-pipeline-correctness --microservice developer-sdk`
- `oya gate validate codegen-determinism --microservice developer-sdk`

### End-to-end drill gates

| Scenario | Pass criterion |
|---|---|
| Stripe-parity | onboard developer → KYC pass → bank verified → signing key issued → submit plugin → vetting pass → install on test tenant → settle payout → emit 1099 — all ≤ 24h |
| KYC false-positive | ≤ 2% rate on weekly review |
| Codegen determinism | Regenerate all six SDK families twice → byte-identical |
| Sandbox reset | ≤ 30s p99 |
| Payout settlement correctness | 100% ledger-to-bank match |
| Developer revocation cascade | revoked developer's plugins all → revoked ≤ 30s |

## Clean Architecture Compliance

Per ADR-0105 13-layer enum; same shape as plugin-app-store with additional layers `adapter-openbao`, `adapter-ach`, `adapter-sepa`, `adapter-kftc`.

## References

- ADR-0213; ADR-0131; ADR-0132; ADR-0139; ADR-0170 (Backstage); ADR-0185 (OpenAPI 3.2 codegen); ADR-0199 (per-tenant cost attribution); ADR-0211 (in-house).
- Stripe docs; Apple Developer Program docs; Backstage docs.
- `microservices/developer-sdk/PRD.md`.
- `/specs/per-microservice-flat-layout.json`.

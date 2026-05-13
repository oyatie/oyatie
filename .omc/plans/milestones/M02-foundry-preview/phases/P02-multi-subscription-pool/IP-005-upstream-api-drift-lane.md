---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-005-upstream-api-drift-lane
parent: ./INDEX.md
milestone: M02
phase: P02-multi-subscription-pool
status: pending approval
purpose: |
  Ship the nightly fitness lane `oya-foundry-fitness-upstream-api-drift` that fetches the
  canonical upstream OpenAPI specs for Anthropic / OpenAI / Gemini and diffs them against
  the adapter contracts in `contracts/foundry-compat-{anthropic,openai}-v1.openapi.yaml`.
  Any divergence emits `EVT-UPSTREAM-API-DRIFT-DETECTED` with a severity classification
  (BREAKING / NON-BREAKING / ADDITIVE) and — for BREAKING — opens a follow-up PR via the
  documented `oya-tooling-agent-write open-pr` primitive (or `gh` per Directive 12 with
  rationale logged). ccproxy-api has no automated drift lane; this is net-new and brings
  oyatie to hyperscaler-bar API-stability discipline (Directive 6).
grit_claim_symbols:
  - "crates/oya-foundry-fitness-upstream-api-drift-kernel/src/lib.rs::detect_drift"
  - "crates/oya-foundry-fitness-upstream-api-drift-kernel/src/lib.rs::DriftReport"
  - "crates/oya-foundry-fitness-upstream-api-drift-kernel/src/lib.rs::DriftSeverity"
  - "crates/oya-foundry-fitness-upstream-api-drift-kernel/src/lib.rs::DriftEntry"
  - "tools/oya-foundry-fitness-upstream-api-drift/src/main.rs::run"
agent_prerequisites:
  - .omc/plans/MASTERPLAN.md
  - ./INDEX.md
  - ./IP-002-anthropic-compat-adapter.md
  - ./IP-003-openai-compat-adapter.md
  - docs/AGENTS.md
  - docs/CONSTITUTION.md
final_shape_compliance: true
dependency_additions:
  - { crate: "openapiv3 2.2", lts: true, adr_exception: null }
  - { crate: "oasdiff (subprocess) v1.10", lts: true, adr_exception: null }
  - { crate: "reqwest 0.13 (rustls-tls)", lts: true, adr_exception: null }
  - { crate: "serde_yaml 0.9 (maintenance only; see ADR if/when superseded)", lts: false, adr_exception: "ADR-pending-serde-yaml-replacement" }
  - { crate: "clap 4.5", lts: true, adr_exception: null }
decision_log: |
  Linus good-taste row: eliminated the per-provider drift code paths by representing each
  provider as a row in a static `UpstreamRegistry` table; `detect_drift` is one function
  that iterates the table. Adding Gemini, Mistral, Cohere, or a future provider is a row
  addition, not a code addition.
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---

# IP-005-upstream-api-drift-lane: Nightly upstream-OpenAPI drift detection

## Purpose

Ships the safety net that catches upstream API changes before they reach production traffic.
Anthropic and OpenAI publish OpenAPI surfaces that evolve; ccproxy-api has historically
chased breakage reactively. This IP makes oyatie proactive: a nightly CI job fetches each
upstream OpenAPI, runs `oasdiff` against the pinned adapter contract, classifies each
finding (BREAKING / NON-BREAKING / ADDITIVE), and — for BREAKING — auto-opens a PR with
the suggested contract delta and reviewer-agent handoff (`api-shape-reviewer`).

## Symbols to grit-claim

```
crates/oya-foundry-fitness-upstream-api-drift-kernel/src/lib.rs::detect_drift
crates/oya-foundry-fitness-upstream-api-drift-kernel/src/lib.rs::DriftReport
crates/oya-foundry-fitness-upstream-api-drift-kernel/src/lib.rs::DriftSeverity
crates/oya-foundry-fitness-upstream-api-drift-kernel/src/lib.rs::DriftEntry
crates/oya-foundry-fitness-upstream-api-drift-kernel/src/registry.rs::UpstreamRegistry
tools/oya-foundry-fitness-upstream-api-drift/src/main.rs::run
.github/workflows/upstream-api-drift.yml::oya-foundry-fitness-upstream-api-drift
```

### Cadence + severity matrix

```
Cadence: nightly 02:00 UTC + on every PR that touches contracts/foundry-compat-*.
DriftSeverity::Breaking      → auto-open PR; ping `api-shape-reviewer`; lane HIGH.
DriftSeverity::NonBreaking   → file MISTAKES-LEDGER row; lane NOTE.
DriftSeverity::Additive      → record in CHANGELOG; lane PASS.
```

### Registry shape

```
struct UpstreamSpec {
    provider: ProviderFamily,
    canonical_url: Url,              // e.g., https://github.com/anthropics/anthropic-sdk-python/.../openapi.yaml
    pinned_version: SemVer,
    adapter_contract_path: PathBuf,  // contracts/foundry-compat-<provider>-v1.openapi.yaml
}

static UPSTREAM_REGISTRY: &[UpstreamSpec] = &[
    AnthropicMessagesV1, OpenAIChatCompletionsV1, OpenAIResponsesV1, GeminiGenerateContentV1,
];
```

## Agent prerequisites

<!-- agent-instructions:start -->
Before `grit claim`, the agent **MUST**:
1. `icm recall-context "P02 upstream-api-drift oasdiff fitness lane" --limit 5`.
2. Read `./IP-002-anthropic-compat-adapter.md` and `./IP-003-openai-compat-adapter.md` (the contracts that this lane diffs against).
3. Confirm symbols unclaimed.
4. Read `docs/AGENTS.md §Pre-flight checklist`.
5. Read `.omc/standards/dependency-policy.md §3` (the lane adds `oasdiff` subprocess — flagged with cargo-vet certification row).
6. Read parent INDEX `./INDEX.md` for the BLOCKER vs HIGH lane-severity mapping.
<!-- agent-instructions:end -->

**Human path:** check `.github/workflows/upstream-api-drift.yml` runs nightly; review any auto-opened PRs with label `upstream-api-drift`.

## Acceptance test commands

```
$ cargo nextest run -p oya-foundry-fitness-upstream-api-drift-kernel --all-features # expect: PASS, 0 failures
$ cargo clippy -p oya-foundry-fitness-upstream-api-drift-kernel -- -D warnings      # expect: PASS, 0 warnings
$ cargo deny check                                                                   # expect: PASS
$ oya gate validate oya-foundry-fitness-upstream-api-drift                           # expect: PASS
$ oya-tooling-agent-read run-evidence "scripts/smoke/drift-simulation.sh"            # expect: mutated-upstream-spec → DriftReport with severity=Breaking + correct entry list
$ oya-tooling-agent-read run-evidence "scripts/smoke/drift-no-drift.sh"              # expect: identical-spec → DriftReport empty + lane PASS
```

Drift-simulation test: copy `contracts/foundry-compat-anthropic-v1.openapi.yaml` to a tmp
dir; introduce (a) a deleted field (BREAKING), (b) a new optional field (ADDITIVE), (c) a
renamed enum value (BREAKING); run `detect_drift` against the original; assert classification.

## Done criteria

- [ ] All `grit_claim_symbols` claimed → work → `grit done`.
- [ ] D1-D18 done-definition walked.
- [ ] All acceptance commands PASS.
- [ ] `cargo deny check` clean; serde_yaml exception ADR drafted (or migrated to alternative).
- [ ] `icm store -t context-foundry` emitted.
- [ ] Audit-chain `EVT-UPSTREAM-API-DRIFT-LANE-SHIPPED` emitted.
- [ ] Nightly CI run executes successfully for at least one cycle pre-merge.
- [ ] Auto-PR scaffold validated against a synthetic BREAKING change.

## Rollback procedure

1. Identify rollback boundary: disable the workflow `.github/workflows/upstream-api-drift.yml` via repo-settings toggle.
2. Execute: workflow off; file MISTAKES-LEDGER row noting the duration; revert PR if kernel-level bug.
3. Verify: no auto-PRs opened by lane until re-enabled.
4. Postmortem trigger: Sev-3 (lane is advisory; absence does not break prod).

## Next IP pointer

`IP-006-tos-policy-audit-chain.md`.

## Icm-store-payload

```
icm store \
  -t context-foundry \
  -c "IP-005-upstream-api-drift-lane merged at <git-sha>; grit symbols released: detect_drift, DriftReport, DriftSeverity, UpstreamRegistry; acceptance lanes green: -upstream-api-drift (HIGH), -no-placeholder; next IP: IP-006-tos-policy-audit-chain" \
  -i high \
  -k "M02,P02,IP-005,upstream-api-drift,oasdiff,ccproxy-parity-gap-closure"
```

## Decision log (Linus good-taste row)

Eliminated per-provider drift functions by representing providers as rows in a static
`UpstreamRegistry` table; adding a provider is a row, not a function.

## Cross-references

- Master Plan: `.omc/plans/MASTERPLAN.md` §2 Directives 3, 6, 10.
- Phase INDEX: `./INDEX.md`.
- Adapter contracts: `contracts/foundry-compat-anthropic-v1.openapi.yaml`, `contracts/foundry-compat-openai-v1.openapi.yaml`.
- ADR-0053; progressive-delivery + branch-pipeline composers.
- ccproxy-api gap closure: ccproxy-api has no automated upstream-drift lane; this IP closes that gap.
- `oasdiff`: https://github.com/oasdiff/oasdiff.
- Anthropic OpenAPI source: https://github.com/anthropics/anthropic-sdk-python (vendor-published OpenAPI).
- OpenAI OpenAPI: https://github.com/openai/openai-openapi.
- Gemini API spec: https://ai.google.dev/api.

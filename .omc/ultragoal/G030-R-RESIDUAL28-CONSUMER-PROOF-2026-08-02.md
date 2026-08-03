# G030-R residual-28 consumer proof — corrected 2026-08-02

State: **PLANNING_ONLY — TWENTY-SIX RESIDUAL JSON ROWS GRAPH-WIRED; TWO MARKDOWN ROWS RETAINED POLICY_PROTECTED; NO DELETION/ACTIVATION**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supersedes the earlier same-day k=5 draft after its delayed independent consumer audit exposed the complete `specs/**/*.json` canonicalization contract.  
No residual path, gate policy, PR, GitOps declaration, or cluster state was changed.

## Corrected result

G030-Q left an exact residual of **28** protected focus paths. All 28 exist at the immutable tip. Of these:

- **26 JSON rows** are `GRAPH_WIRED_INPUT` through the Buck2-native canonical-JSON live-corpus gate;
- **2 Markdown rows** remain `POLICY_PROTECTED_MACHINE_ARTIFACT` because that gate selects only `*.json` and no other executable reader was found.

Reconciled totals become **152 `MACHINE_SSOT` + 982 `GRAPH_WIRED_INPUT` + 42 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. Remaining protected queue: 19 fixture + 23 non-fixture. Delete candidates remain 0.

The earlier k=5 result was wrong because it applied the exact semantic-reader test without checking the repo-wide canonical-JSON governed-root consumer. Exact basename grep was insufficient for a recursively selected corpus.

## Complete-corpus executable edge

At the immutable tip:

1. `ci/facade/canonical-json/canonical-json-policy.json` sets `governed_roots` to `specs`.
2. Its only exclusions are suffix `.generated.json` and prefix `specs/fixtures/`.
3. `ci/facade/canonical-json/src/lib.rs::collect_observed` iterates every governed root, calls `walk_json`, and byte-reads each selected path with `fs::read`.
4. `walk_json` recursively traverses nested directories and selects every regular `*.json`.
5. `ci/facade/canonical-json/tests/canonical_json.rs::live_governed_corpus_is_canonical_at_zero_baseline` loads the policy, collects the live corpus, evaluates it, and fails unless findings are empty.
6. `ci/facade/canonical-json/BUCK` declares the executable Buck2 Rust target `ci-canonical-json-gate`.

All 26 residual JSON paths are under `specs/`; none is under `specs/fixtures/`; none ends `.generated.json`. Therefore all 26 are mechanically selected, opened, canonicalized, and evaluated.

This is a real executable graph edge, though its semantic obligation is byte-canonical JSON rather than domain-schema sufficiency.

## Promoted GRAPH_WIRED (26)

```text
specs/audit-event-schema.json
specs/capabilities/canonical-tier-schema.json
specs/capabilities/eu-ai-act-risk-class-registry.json
specs/catalog/canonical-crate-record-schema.json
specs/cloud-observability-slo-evidence-contract.json
specs/cloud-production-quality-kit-evidence-backlog.json
specs/compliance-pack-floors.json
specs/finops-dimensional-model.json
specs/http-stack-policy.json
specs/iac-module-library.json
specs/ip/canonical-frontmatter-schema.json
specs/language-discipline-registry.json
specs/legal-ip-domain-taxonomy.json
specs/microservice-tier-classification.json
specs/microservices/foundry.json
specs/microservices/pii-registry.json
specs/microservices/rpo-rto-targets.json
specs/microservices/scorecards/canonical/aws-well-architected.json
specs/microservices/scorecards/canonical/cis-k8s-benchmark.json
specs/microservices/scorecards/canonical/google-sre-prr.json
specs/microservices/scorecards/canonical/slsa-l3.json
specs/openslo/canonical-envelope-schema.json
specs/policy-ir-benchmark-fixture-suite.json
specs/policy-ir-benchmark-rubric.json
specs/repo-hygiene-automation.json
specs/root-of-trust-ceremony-contract.json
```

### Stronger domain-specific edges retained as supplemental proof

- `specs/http-stack-policy.json`: `marketplace/facade/dev-cli/src/http_stack_gate.rs` supplies the exact default path, `read_http_stack_policy` reads and parses it, and `validate_http_stack_gate` enforces its preferred/forbidden/justified crate policy.
- Four `specs/microservices/scorecards/canonical/*.json` rows: ontology scorecards `FRAMEWORKS` names exactly those four slugs; `canonical_dir(root).join(format!("{slug}.json"))` loads each, with Buck library/bin/unittest targets.
- `specs/microservices/foundry.json`: product-PRD traversal also opens and classifies it as `NonPrd` because it is Retired. That remains retirement semantics, not live PRD authority; canonical-JSON coverage is the promotion edge.

## Retained POLICY_PROTECTED (2)

| Path | Disposition | Reason |
|---|---|---|
| `specs/policy/cedar-scope-schema.md` | `POLICY_PROTECTED_MACHINE_ARTIFACT` | Markdown; excluded by the canonical-JSON extension selector; only a cloud-enforceability-facets citation was found |
| `specs/products/RETIREMENT.md` | `POLICY_PROTECTED_MACHINE_ARTIFACT` | Markdown retirement companion; product-PRD source comment identifies it but the gate scans `specs/microservices/*.json`, not this file |

Neither row is a deletion candidate. Absence of this consumer shape is not proof of absence of all authority.

## Anti-vacuity

Proven:

- exact residual width 28 and immutable membership 28/28;
- exact split 26 JSON + 2 Markdown;
- all 26 JSON rows match the governed root and neither exclusion;
- the Rust collector recursively enumerates and byte-reads them;
- the Buck2 gate evaluates the live corpus at zero baseline;
- no double count against G030-E..Q: these 28 were Q's exact residual;
- the original five domain-reader promotions are included within, not added to, the 26.

Not proven / not claimed:

- domain-schema or business-semantic sufficiency for the 21 JSON rows that have only canonical-JSON wiring;
- protected-context green at the current tip;
- permission to delete either Markdown row;
- that foundry is live PRD authority;
- independent merge/design approval for any later mutation.

## Arithmetic

```text
after Q:     152 + 956 + 68 = 1176
promote k=26: 152 + 982 + 42 = 1176
protected:     19 fixture + 23 non-fixture = 42
delete candidates: 0
```

## Verification and review boundary

Evidence is immutable tip tree membership, canonical-JSON policy, recursive Rust collector, live-corpus test, and Buck target at `b651080374113aeb57500eecbd9d1326f0404e48`.

The delayed independent consumer audit completed and identified the missed gate edge; the coordinator independently confirmed it against tip. This is **independent audit evidence**, not independent APPROVE for a mutation, PR, design, or cluster action. A second residual audit still failed transport and is not approval.

## Non-actions

- No residual path edited, deleted, or re-rooted.
- No move-plan JSON, generated face, or multispectrum evidence surface added.
- No G028 push/apply; G028 remains local-only at `051bc7ec6`.
- No G023 deletion; no #1523 restack push; no cluster mutation; no canonical dirty checkout mutation.
- No independent APPROVE inferred from transport failure or this read-only audit.

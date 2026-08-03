# G030-K design-system residual consumer and catalog-gap proof — 2026-08-02

State: **PLANNING_ONLY — EIGHT ROWS GRAPH-WIRED; NINE CATALOG-ONLY ROWS RETAINED; NO DESIGN MOVE**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-J-ACCOUNTS-REGISTRY-CONSUMER-CONTRACT-DIVERGENCE-PROOF-2026-08-02.md`.  
No design specification, catalog, PRD, gate, implementation, owner assignment, generated face, PR, GitOps declaration, or cluster state was changed.

## Exact residual recovery

The prior G030 count of 17 is correct. The immutable tip contains 32 `specs/design-system/*.json` rows:

- 15 are exact live `current_path` entries in `specs/root-hub-pointers.json` and were already ranked `MACHINE_SSOT` by G030-E precedence;
- the remaining 17 are exactly `tip design-system rows − root-hub design-system rows`.

The earlier 17-versus-32 discrepancy was therefore a precedence partition, not missing files or census drift.

## Result

Eight of the 17 residual rows are executable existence-contract inputs. They are referenced by active PRD-shaped microservice JSON and the Rust `product-prd-json` gate resolves every `frontend_components[].design_system_ref` against the repository filesystem.

| Path | PRD consumer | Disposition |
|---|---|---|
| `specs/design-system/anon-channel-feed.json` | `specs/microservices/anonymous.json` | `GRAPH_WIRED_INPUT — PRD EXISTENCE CONTRACT` |
| `specs/design-system/anon-post-composer.json` | `specs/microservices/anonymous.json` | `GRAPH_WIRED_INPUT — PRD EXISTENCE CONTRACT` |
| `specs/design-system/ar-camera-overlay.json` | `specs/microservices/social.json` | `GRAPH_WIRED_INPUT — PRD EXISTENCE CONTRACT` |
| `specs/design-system/hr-aggregate-insights-dashboard.json` | `specs/microservices/anonymous.json` | `GRAPH_WIRED_INPUT — PRD EXISTENCE CONTRACT` |
| `specs/design-system/salary-benchmark-widget.json` | `specs/microservices/anonymous.json` | `GRAPH_WIRED_INPUT — PRD EXISTENCE CONTRACT` |
| `specs/design-system/social-feed-scroller.json` | `specs/microservices/social.json` | `GRAPH_WIRED_INPUT — PRD EXISTENCE CONTRACT` |
| `specs/design-system/social-post-composer.json` | `specs/microservices/social.json` | `GRAPH_WIRED_INPUT — PRD EXISTENCE CONTRACT` |
| `specs/design-system/stories-ring-bar.json` | `specs/microservices/social.json` | `GRAPH_WIRED_INPUT — PRD EXISTENCE CONTRACT` |

The other nine residual rows appear in the accepted design-system catalog but have no measured semantic reader beyond generic tracked-artifact accounting:

| Path | Measured state | Disposition |
|---|---|---|
| `specs/design-system/job-board-search.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |
| `specs/design-system/network-feed.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |
| `specs/design-system/professional-profile-card.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |
| `specs/design-system/recruiter-pipeline.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |
| `specs/design-system/sales-copilot-panel.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |
| `specs/design-system/shorts-creator-analytics-dashboard.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |
| `specs/design-system/shorts-for-you-feed.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |
| `specs/design-system/shorts-live-viewer.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |
| `specs/design-system/shorts-video-editor.json` | catalog `component_refs` only | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY` |

This promotes eight rows from the protected-only queue. The reconciled totals become **152 `MACHINE_SSOT` + 917 `GRAPH_WIRED_INPUT` + 107 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. The remaining protected queue is 19 fixture residuals plus 88 non-fixture rows.

## Executable PRD consumer contract

`marketplace/facade/dev-cli/src/commands/gate/product_prd_json.rs`:

1. recursively collects JSON rows under `specs/microservices`;
2. treats every non-retired row whose `_meta.spec_id` starts with `PRD-` as active for validation — `Draft` status does not exempt it;
3. requires user-facing PRDs to declare a non-empty `frontend_components` array;
4. requires each component to declare `design_system_ref` using `$ref:<repo-path>` syntax;
5. strips any fragment and requires the referenced repository path to exist;
6. fails if zero active product PRDs were checked.

The anonymous and social PRDs both have `PRD-` identifiers and non-empty frontend component arrays, so their Draft status does not remove them from this executable reader. The dedicated Buck2 Rust test target is:

`root//marketplace/facade/dev-cli:marketplace-dev-cli-product-prd-json`.

This establishes source-graph and Buck-target visibility. It does not establish that the target executed in every protected `oya-ci-required` run at this immutable tip; no affected-set expectation for the target was found.

The gate's contract is existence-only. It does not parse the referenced design-system row or compare its component ID, schema, status, accessibility rules, breakpoints, tokens, or verification commands to the PRD reference.

## Catalog gap

`specs/design-system/catalog.json` is an Accepted owner catalog with 31 `component_refs`, exactly covering every other design-system JSON row. Exact search found no Rust/CI/governance/tool consumer of `component_refs`, `DS-CATALOG`, or the catalog path that resolves those references. The catalog's own verification command list names generic product-PRD, active-artifact, retired-vocabulary, and JSON-format commands, but none was proven to parse `component_refs`.

Therefore catalog membership is retention and owner-intent evidence, not an executable graph edge. The nine catalog-only rows remain protected; this census does not declassify them or infer that their associated product concepts are dead.

## Already-classified 15-row boundary

The 15 design-system paths not listed above are exact root-hub `current_path` entries and were already included in G030-E's `MACHINE_SSOT` bucket. Some also have PRD existence references or shell implementation comment citations, but precedence prevents double counting. This slice changes only the residual 17.

The shell frontend comments naming four design-system rows are implementation provenance, not a parser edge: the Rust modules do not load the JSON files. Those four are already root-hub `MACHINE_SSOT` rows and are outside this residual.

## Anti-vacuity and semantic boundary

Proven:

- tip design-system rows = 32;
- exact root-hub design-system rows = 15;
- exact residual = 17;
- catalog references all 31 non-catalog rows;
- eight residual paths are referenced by PRD-shaped rows the product gate classifies as active;
- the gate checks each such referenced path exists;
- nine residual paths have no non-catalog semantic citation beyond historical corpus-audit prose.

Not proven:

- semantic parsing or schema validation of any design-system row;
- catalog-reference existence or uniqueness validation;
- implementation conformance to the JSON design contracts;
- protected required-context execution of the product-PRD Buck target;
- ownership or destination for any future console/product move.

These are enforcement and implementation gaps, not delete authority.

## Verification boundary

Evidence came from immutable tree enumeration and exact searches at `b651080374113aeb57500eecbd9d1326f0404e48`: all 32 paths, root-hub current paths, catalog refs, microservice PRD refs/statuses, product-PRD gate source, Buck target, shell citations, fixtures, ADRs, and executable-surface searches. No local CLI execution is used as merge authority.

An independent Explore audit retried residual-set recovery and failed with the same encrypted-content transport error. It remains `FAILED_TRANSPORT_NOT_APPROVE`; the mechanical proof is not independent approval.

## Non-actions and non-claims

- No design specification, catalog, PRD, or shell implementation edited.
- No catalog-only row deleted or declassified.
- No claim that catalog membership is executable.
- No claim that existence validation proves semantic conformance.
- No `app/` ownership, exact destination leaf, or move-plan JSON invented.
- No new generated face or multispectrum evidence surface.
- No independent APPROVE; transport failure remains non-approval.

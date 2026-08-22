# Architect r17 (Torvalds-lens) — Active Artifact Contract Scaffold

## Verdict
ITERATE

Not reject: the direction is right. One graph substrate + contributed registries + claim matrix is the correct class of solution. Not approve: the current scaffold is mostly untracked, unenforced metadata. It describes a hyperscaler-grade control plane; it is not one yet.

## Session
019e243d-d60b-7ee0-8be3-e97fd154015b

## Linus-style findings (each: defect, evidence, fix)
1. Defect: the reviewed artifacts are not in HEAD, so verified-claims fail at the repo-state boundary.
   Evidence: `git ls-files -- <7 reviewed paths>` returned 0 rows; `git status --short -- <7 paths>` shows all seven as `??`.
   Fix: land them through the sanctioned grit path, then rerun `git ls-files` and pin the resulting object/commit in provenance.
2. Defect: public contract change lacks a tracked ADR/audit chain.
   Evidence: contract claims v3.0.0 and supersedes v1/v2 at `/specs/active-machine-readable-artifact-contract.json:9-15`, but no ADR for this contract was tracked; `git ls-files` only found `docs/standards/prevention-doctrine.md` among the queried new spec/ADR authorities.
   Fix: author an ADR for the control-plane/graph/registry split, cite migration impact, and attach an audit/evidence bundle before calling this Linus-grade.
3. Defect: the graph schema is not actually a graph instance or enforceable catalog; core node/edge/invariant data is stashed under custom `_canonical_*` keys.
   Evidence: schema requires instance fields `node_types`, `edge_types`, `invariants`, `contributing_registry_protocol` at `knowledge-graph-schema.json:31-36`; the shipped catalogs live under `_canonical_node_types` and `_canonical_invariants` at `120` and `394` onward. Generic JSON Schema validators will ignore those custom keywords.
   Fix: split into `knowledge-graph.schema.json` and `knowledge-graph-catalog.json`, or promote these catalogs into first-class schema-validated data with `$defs` plus validator tests.
4. Defect: no evidence the promised single Rust validator exists.
   Evidence: graph validator is `crates/check-knowledge-graph (planned)` at `knowledge-graph-schema.json:27-28`; contract validator is planned at `active-machine-readable-artifact-contract.json:37-38`; `find crates ...` found many `check-*` crates but not `check-knowledge-graph` or `check-active-artifact-contract`.
   Fix: implement the minimal validator first: load registries, resolve paths against HEAD, fail on unknown refs, duplicate ids, and operational-without-evidence.
5. Defect: enforcement lanes are named but not wired.
   Evidence: artifact registry rows require planned lanes/hooks/crates at `artifact-capabilities-registry.json:20-35` and `54-77`; `rg` found no `lean-a-active-artifact-contract`, `lean-a-knowledge-graph-integrity`, `lean-a-building-block-drift`, `lean-a-attestation-format`, or `lean-a-ops-portal-claim-matrix` in `registry/quality/lanes.yaml` / workflows. Existing tail lanes are unrelated planned quality lanes at `registry/quality/lanes.yaml:435-475`.
   Fix: add one active lane and CI job for the minimal validator. Do not add five fake lanes.
6. Defect: DRY registry says it enforces drift but counts are manual and contradictory.
   Evidence: `consumer_count_actual_today` is 0 for real consumers listed in the same rows, e.g. block has 4 consumers but count 0 at `reusable-building-blocks-registry.json:58-65`; evidence-bundle has 4 consumers but count 0 at `76-83`; optimization status admits drift detection and reuse ranking are planned/manual at `310-316`.
   Fix: delete manual counts or mark them estimated only; compute actual consumers from graph edges and fail when listed consumers do not resolve.
7. Defect: reusable-block rows mix canonical paths with prose, planned state, and non-path pseudo-consumers.
   Evidence: planned crate paths embed prose at `reusable-building-blocks-registry.json:141-177`; consumers include strings like `every future BC...` and runtime concepts, not resolvable refs at `146-150`, `195-199`.
   Fix: separate `canonical_path`, `status`, `planned_reason`, `consumer_selector`, and `consumer_refs`; require refs to resolve or be explicitly typed selectors.
8. Defect: 9-capability contract is too heavy to author manually at scale.
   Evidence: every row carries nine nested capability objects; the current registry has only 10 rows but already 35 KB. Baseline says remaining artifacts will be added incrementally by a planned self-update lane at `artifact-capabilities-registry.json:10`.
   Fix: use defaults/profiles: e.g. `artifact_profile: schema|registry|template|plan-attestation` plus overrides. Generate expanded views for audit, not hand-authored boilerplate.
9. Defect: claim matrix is honest about many gaps, but still over-claims documentation evidence as implementation/verification.
   Evidence: `claims_we_can_make` says pre-mortem has `HG-RELIABILITY evidence_class=implementation+verification` at `ops-portal.json:27` while gate coverage says reliability is planned and missing implementations/tests at `155-158`.
   Fix: downgrade that claim to documentation-only, or attach actual implementation/test evidence.
10. Defect: evidence references ephemeral `/tmp` outputs and an acknowledged protocol violation.
    Evidence: attestation references `/tmp` architect/critic artifacts and says they should be archived at `ops-portal-plan-set-accepted-2026-05-13.json:79-98`; it logs `rtk git commit` protocol violation at `111-120`.
    Fix: archive durable review outputs under `/evidence/audits/consensus/` and make future state transitions grit-sealed before evidence is accepted.

## Scale/Maintainability assessment
- Will this scale past 10k artifacts? evidence: No evidence yet. The schema says no giant graph and future adapter at `knowledge-graph-schema.json:17`, but current state is monolithic manual registries and no storage/index implementation.
- Will validator queries finish in <5s on full graph? evidence: Unknown/FAIL as a claim. Only SQL sketches exist at `knowledge-graph-schema.json:480+`; no crate, dataset, benchmark, or complexity budget.
- DRY enforcement actually prevents drift? evidence: Not today. Drift lane planned; counts manual; consumers non-resolving; invariants are prose at `reusable-building-blocks-registry.json:306-324`.
- 9-capability contract feasible to author for every artifact? evidence: Not manually. 10 rows already produce large boilerplate; no generator exists; all autogen/selfheal/selfupdate are planned.

## User-Mandated-Rule Check
- (i)   honest-claims: FAIL — claim matrix is mostly honest about cannot-claim rows, but reliability is labeled implementation+verification from documentation-only evidence (`ops-portal.json:27` vs `155-158`).
- (ii)  Linus-grade: FAIL — public contract v3.0.0 has no tracked ADR/audit and the artifacts are not in HEAD.
- (iii) verified-claims: FAIL — `git ls-files` found 0/7 reviewed artifacts tracked. For registry rows, only `/templates/evidence-bundle-template.json` was HEAD; the other 9 artifact paths were WT-only.
- (iv)  honest-introspection: PASS-with-defects — gaps are candidly listed (`ops-portal.json:35-50`, `135-194`; attestation `14-20`, `100-120`), but stale “evidence_class=implementation+verification” wording remains.

## Hyperscaler-grade verdict
- AWS Config + AWS Resource Explorer: matches the idea of inventory + compliance state. Falls short on resource normalization, indexed queries, immutable history, remediation execution, and managed rule evidence.
- GCP Asset Inventory + Cloud Asset Graph: matches graph ambition. Falls short on temporal snapshots, IAM integration, ancestor scoping, search language, and exportable delta feeds.
- K8s CRD + admission controllers: matches schema + validator direction. Falls short because no admission path exists; schemas do not yet block writes; custom `_canonical_*` metadata is not enforceable CRD status/spec.
- Cargo workspace.dependencies + Maven BOM: matches centralization goal. Falls short because consumers are free-text, not compiler/package-manager-enforced references.
- Innovation: the 9-capability contract unifies enforcement, provenance, telemetry, and self-maintenance in one control-plane vocabulary. Good idea. Needs generated profiles and hard validators or it becomes YAML cosplay in JSON.

## Missing capabilities (priority order)
1. Minimal `check-knowledge-graph`/`check-active-artifact-contract` validator with HEAD path resolution.
2. ADR + migration policy for v3.0.0 public contract and graph/registry split.
3. Active CI/quality lane that fails on untracked artifact refs, broken refs, duplicate ids, and operational-without-evidence.
4. Durable audit archive for consensus outputs; ban `/tmp` as evidence_ref except as transient input.
5. Generated registry/profile system so 9-capability rows are generated, not manually maintained.
6. Graph storage/query benchmark fixture with 10k+ synthetic artifacts and <5s gate budget.
7. Typed refs/selectors instead of prose path strings.
8. Drift detector with structural hashes for schemas, Cedar, OpenAPI components, traits, lanes.

## Architectural critique (per-file)
- `/specs/active-machine-readable-artifact-contract.json`: strong vocabulary, too much manual payload; planned validators; related paths include untracked/missing specs.
- `/specs/knowledge-graph-schema.json`: right substrate idea; wrong enforceability shape. Catalogs live as ignored custom schema metadata; no storage/query model.
- `/registry/artifact-capabilities-registry.json`: useful control-plane seed; currently 10 rows of promises. Needs generated profiles and a failing lane.
- `/registry/reusable-building-blocks-registry.json`: right DRY intent; current data is polluted by estimates, prose, future paths, and manual counts.
- `.omc/ledger/ops-portal-ledger.json`: useful plan ledger; evidence refs include WT-only and `/tmp`; statuses are plan-level only.
- `/registry/claim-matrix/ops-portal.json`: best artifact in the set for honesty; still contains an over-classified reliability evidence claim.
- `/evidence/ops-portal-plan-set-accepted-2026-05-13.json`: candid about missing cosign/rekor/audit/grit; not durable enough because it cites `/tmp` and untracked artifacts.

## Recommended next-action
Stop expanding registries. Implement one narrow validator + lane: read the artifact-capabilities registry, verify every `artifact_path` is tracked by `git ls-files`, verify every named checker crate/lane either exists or status is not operational, and fail any `claims_we_can_make` item whose evidence class exceeds its gate status. Then add the ADR for v3.0.0 and rerun this review.

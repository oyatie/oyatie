# Ultragoal resume status — 2026-08-02T late

origin/dev: `0c1014b87f0d881a821faa6a872b309deba0cfbf` (#1529 MERGED; declared ARC request 22Gi)

## Hard blockers
- G028 declaration MERGED (#1529 → tip `0c1014b87`); live ARS/ERS still **20Gi** because GitOps reconciler is inert on `admin@oya-talos` (no Argo/CAPI). Prior founder ruling fixes **class B / KEEP_CURRENT_LAB=true**; v1 independent review returned **REQUEST_CHANGES** and staged ARC-only v2 fact collection is in progress. No implementation or helm/CRS/render.sh apply authorized.
- Tip CI run `30767156146` FAILED FULL on `//oya:corpus-yaml-facts` with `Local command returned non-zero exit code <no exit code>`; runner survived to upload artifacts. Classification: child-action no-exit-code (OOM/DiskPressure/eviction **not** proven). Artifact: `G028-TIP-FULL-CORPUS-YAML-FACTS-FORENSICS-2026-08-02.md`.
- #1526 and #1528 remain open/red; no cold FULL rerun while live request is 20Gi. #1528 is runner-loss class, not plan-defect evidence.
- #1523 still on pre-restack head `1c308fa4843c`; local restack rehearsal parked unpushed until live 22Gi + #1526 path healthy.
- #1524 remains draft DO-NOT-MERGE CONFLICTING; preserve only.
- Independent review transport still collapses (encrypted_content decrypt 400) across peers/workflows → never APPROVE.

## Next executable order
1. Complete v2 repository/live-inventory/identity facts, freeze packet digest, and obtain real independent design APPROVE on the exact staged ARC-only class-B packet.
2. Implement minimal class-B declaration/render/tests; exact-head code APPROVE + protected PR.
3. Authorized one-time bootstrap from admitted immutable artifact only.
4. Observe live ARS/ERS/new pod request=22Gi.
5. Re-run #1526 cold FULL; admit corpus repair; observe promoted tip green.
6. Push #1523 restack; independent review; admit; observe.
7. Only then W1/G023 deletion and W0-C/D rebuilds.
8. G024 #1528 re-evaluate only after cold path healthy (currently infra runner-loss class).
9. G025/G026/G030/G036/G037 remain plan-only owner-gated; no invented leaves; no activation.

## Latest planning progress

### G028 class B selected by prior founder ruling (2026-08-02)
- Durable 2026-07-29 ruling: laptop Talos cluster is permanent, has merge authority, and must become declarative
- Mechanical result: `KEEP_CURRENT_LAB=true → class=B`; no contrary later ruling found
- Artifact: `.omc/ultragoal/OWNER-DECISION-G028-ABC-LIVE-22GI-2026-08-02.md`
- Independent architecture review returned `REQUEST_CHANGES`: single-apply CRD race, unsafe global prune blast radius, mutable `dev` adoption, missing content-addressed inputs, incomplete ARC/secret rollback inventory, unnamed principals, unspecified Buck2 fan-in, and unbound packet bytes
- V2 draft narrows G028 to staged CRDs → controllers → ARC-only exact-SHA adoption with sync/prune disabled
- Repository facts now fix the 18-Application blast radius, existing protected `ci/facade` Buck2 fan-in, ARC chart declarations, and absent releaseName/hermetic pins; live Helm/resource/secret inventory, content digests, bootstrap principal, target fingerprint, and rollback owner remain unresolved
- Artifacts: `.omc/ultragoal/G028-CLASS-B-INDEPENDENT-REVIEW-2026-08-02.md`, `.omc/ultragoal/G028-CLASS-B-V2-STAGED-ARC-ADOPTION-DESIGN-2026-08-02.md`, `.omc/ultragoal/G028-BOOTSTRAP-REPOSITORY-FACTS-2026-08-02.md`, `.omc/ultragoal/G028-ARC-ADOPTION-REPOSITORY-FACTS-2026-08-02.md`
- No implementation or fresh v2 re-review is authorized until the remaining fact-dependent fields are frozen; class selection does **not** authorize live mutation

### Direct tip FULL forensics (2026-08-02)
- Workflow `wf_9c2680fb-269` collectors 10/10 FAILED_TRANSPORT; empty-evidence synthesis discarded (not authority)
- Direct GH evidence: fail at 21:16:53Z on `root//oya:corpus-yaml-facts`; signature no-exit-code; ~90 concurrent local actions; upload artifacts success 21:18:16Z; job completed with runner alive
- Tip codepath: single genrule over 4103 oya YAML (6.48 MiB blobs); argv ~0.36 MiB → ARG_MAX refuted; extractor holds full shard in memory and would stderr on normal failure
- #1526 head shards at 256 (~17 faces) — plausible mitigator, not root-cause proof; rerun still unauthorized at live 20Gi
- Live ARS/ERS re-probed: still 20Gi
- Artifact: `.omc/ultragoal/G028-TIP-FULL-CORPUS-YAML-FACTS-FORENSICS-2026-08-02.md`

### G036 tip-authority refresh (2026-08-02)
- Recomputed on tip `0c1014b87`: 56/56 BUCK+rust_test; 8 selected; gap 48 exact match to prior list
- Authority + dep order amended off stale pre-#1529 / “G028 local-only” text; no activation
- Artifact hash note: dual-mirrored to worktree+main `.omc`/`.omx`

### Active owner-packet tip refresh (2026-08-02)
- G030-V residual13 all EXIST on tip; authority → `0c1014b87`
- G037-B authority → `0c1014b87`; hatch package/shell still ABSENT; BIND|RETIRE still owner-gated
- G026 unresolved two-crate packet authority → `0c1014b87`; `app/*` still ABSENT
- Historical census packets retain measurement tip `b651080…` as provenance (not live authority)

### Plan-lane audit transport collapse (2026-08-02)
- Workflow `ultragoal-plan-lane-audit` / `wf_95240de6-acc`: 7/8 agents FAILED_TRANSPORT (encrypted_content decrypt 400); inventory null; synthesis on empty audits discarded
- Not APPROVE; no packet tip-drift verdicts established by that workflow
- Coordinator continues with direct tip evidence + corrected forensic workflow

### G028 gap peer re-review transport failed again (2026-08-02)
- Peer `g028-gap-review` idleReason=failed encrypted_content decrypt 400; not APPROVE
- Corrected class A + ERS path remain dual-mirrored; local tip fact-check already passed
- Independent design APPROVE still outstanding; prior founder ruling now fixes class B / KEEP_CURRENT_LAB=true; no mutation authorized

### Tip CI FULL failed on corpus-yaml-facts (2026-08-02)
- Run `30767156146` head `0c1014b87` conclusion failure; 10 constituents green including buck2; affected-set + oya-ci-required red
- Binding FULL cold-rebuild; action `//oya:corpus-yaml-facts` genrule died with `non-zero exit code <no exit code>` on runner `oya-arm64-lh4ch-runner-9d5zl`
- Same corpus class as open #1526; live ARS/ERS still 20Gi; tip CI red ≠ live 22Gi and is not bootstrap authority
- Class B design packet written; independent design review is pending; prior founder ruling fixes class B / KEEP_CURRENT_LAB=true

### G028 class B permanent-lab design packet (2026-08-02)
- Recommended if KEEP_CURRENT_LAB; uses existing root-app/app-of-apps/22Gi values; rejects CRS/render.sh/cells[] as apply authority
- Requires class-B authority already selected by founder ruling + independent design APPROVE before any implementation or bootstrap act
- Artifact: `.omc/ultragoal/G028-CLASS-B-PERMANENT-LAB-GITOPS-DESIGN-2026-08-02.md`

### #1528 affected-set classified as runner loss (2026-08-02)
- Head `da46906d02408cef255f3a678ff5e047fe8a3d44`; run `30757840152`; job `91523009385`
- Terminal annotation: self-hosted runner lost communication; affected-set alone failed; Buck2 + nine other constituents passed
- Classification: infrastructure runner loss, not evidence of a G024 plan defect; no rerun while live ARC request remains 20Gi
- Independent classifier FAILED_TRANSPORT; not APPROVE

### G028 gap re-review transport failed twice (2026-08-02)
- Independent code-reviewer + verifier both FAILED_TRANSPORT (encrypted_content decrypt 400); not APPROVE
- Local tip fact-check against `0c1014b87` passed: request 22Gi declared; CRS management→workload; cells=[]; bootstrap-sync requires Argo; live ARS/ERS still 20Gi
- Prior founder ruling fixes class B / KEEP_CURRENT_LAB=true; independent design APPROVE remains the unblock for implementation

### G028 gap packet class A corrected (2026-08-02)
- Independent review REQUEST_CHANGES: cells[] cannot adopt admin@oya-talos; CRS is management→workload; ERS probe path fixed
- Class A rewritten as CAPI replacement/migration only; B remains non-CAPI lab bootstrap and is selected by prior founder ruling; independent design APPROVE still required
- Artifact: `.omc/ultragoal/G028-GITOPS-BOOTSTRAP-GAP-2026-08-02.md`

### G028 GitOps bootstrap gap packet (2026-08-02)
- Live cell has no Argo/Flux/CAPI; Helm oya-arm64 v12 still 20Gi while tip declares 22Gi
- Intended chain is CAPI CRS → Argo root-app → infra/gitops oya-arm64; lab is non-CAPI permanent Talos
- Artifact: `.omc/ultragoal/G028-GITOPS-BOOTSTRAP-GAP-2026-08-02.md` (plan-only; no apply)
- #1526/#1523 remain blocked on live 22Gi observation after admitted reconciler path

### G028 live GitOps observation BLOCKED/INERT (2026-08-02)
- origin/dev declares 22Gi after #1529, but live AutoscalingRunnerSet, EphemeralRunnerSet, and 3 pods remain 20Gi
- Helm release oya-arm64 v12 predates merge (2026-07-30); no argocd namespace/controller/CRDs/Applications and no Flux controller
- No manual helm apply performed; #1526 cold FULL and #1523 restack push remain blocked until an admitted reconciler applies declaration and 22Gi is observed
- Promoted-tip CI run 30767156146 queued

### G028 #1529 MERGED (2026-08-02T21:06:55Z)
- Merge commit `0c1014b87f0d881a821faa6a872b309deba0cfbf` on `origin/dev`
- Exact approved head `051bc7ec6`; oya-ci-required SUCCESS 12/12
- Tip declares request=22Gi; GitOps live observe still required before #1526 cold FULL
- Artifact: `.omc/ultragoal/G028-PR-PACKAGE-2026-08-02.md`

### #1526 affected-set failure (2026-08-02) — classification narrowed
- PR #1526 affected-set job 91524468086 annotation: "The self-hosted runner lost communication with the server"
- Failed during step "Materialize merge-base build + test baselines when affected-set needs FULL"
- Binding affected-set step never started — runner-loss / cold-path class; **DiskPressure/eviction not proven** from annotation alone
- Superseding tip FULL evidence on `0c1014b87` (run 30767156146) is child-action no-exit-code with runner survived; see `G028-TIP-FULL-CORPUS-YAML-FACTS-FORENSICS-2026-08-02.md`
- Cold FULL re-run remains blocked until live ARS/ERS request=22Gi via admitted reconciler

### G028 independent APPROVE + PR #1529 (2026-08-02)
- Independent peer lanes APPROVE exact head `051bc7ec603d49b838a400471a01778b966b2b8c`
- Branch pushed; PR https://github.com/jason931225/oyatie/pull/1529 opened against `dev`
- Not merged; live cluster not mutated; GitOps observe remains post-merge
- Artifact: `.omc/ultragoal/G028-PR-PACKAGE-2026-08-02.md`

### G028 independent review still blocked (2026-08-02T~23:50Z)
- Another `agent-skills:code-reviewer` lane FAILED_TRANSPORT (decrypt)
- Still LOCAL ONLY at `051bc7ec6`; not APPROVE; not pushed; not applied

### G026 application unresolved-owner packet (2026-08-02)
- Exact unresolved set reduced to two crates: `oya-application-app`, `oya-cloud-surface-domain`
- Owner must establish named product vs single capability vs split/decompose/keep
- `app/` absent; no generic `app/application`, `app/foundation`, or `app/workspace`
- No move-plan JSON or exact destination leaf before owner + independent review
- Artifact: `.omc/ultragoal/G026-APPLICATION-UNRESOLVED-OWNER-PACKET-2026-08-02.md`

### G030-V residual 13 owner-ruling queue (2026-08-02)
- Exact 13 non-fixture residual paths assigned to accountable owner/ruling classes
- Accounts runtime-directory use separated from schema-byte validation
- Frozen/tombstone retention separated from delete authority
- Owner response requires WIRE|KEEP_PROTECTED|RETIRE + authority + acceptance check
- Delete candidates remain 0; no artifact mutation
- Artifact: `.omc/ultragoal/G030-V-RESIDUAL13-OWNER-RULING-QUEUE-2026-08-02.md`

### G037-B hatch owner disposition packet (2026-08-02)
- Exact 5 hatched active lanes frozen with tip owner/source/check_command
- Hatch keys still 2; package + shell harness ABSENT at tip
- Half-(c) comment names missing id `merge-queue-staging-ref-gc`; live id is `merge-queue-ref-hygiene`
- Planned five remain non-hatch / non-active
- No registry/hatch mutation; owner BIND|RETIRE table required
- Artifact: `.omc/ultragoal/G037-B-HATCH-OWNER-DISPOSITION-PACKET-2026-08-02.md`

### G036 exact 48-kernel protected-context gap (2026-08-02)
- Exact set: 56 kernels − 8 affected-set selected = 48 protected-context-gap candidates
- All 56 have BUCK + rust_test; gap is admission reachability, not missing tests
- Self-conformance still single-root `ci/facade`; workflow has no //governance/check fan-in
- Minimum multi-root design recorded; no activation/policy/baseline edit
- Blocked on independent design review + healthy PR train after G028
- Artifact: `.omc/ultragoal/G036-EXACT-48-KERNEL-PROTECTED-CONTEXT-GAP-2026-08-02.md`

### G028 independent review still blocked (2026-08-02T~23:00Z)
- Second distinct lane (`agent-skills:code-reviewer`) also FAILED_TRANSPORT (decrypt)
- Still LOCAL ONLY at `051bc7ec6`; not APPROVE; not pushed; not applied

### G030-U exact residual fixture inventory (2026-08-02)
- Locked exact 19 fixture residual tip paths: calendar 15 + CRATEADR 3 + DR 1
- Closed residual set with T: 13 non-fixture + 19 fixture = 32 POLICY_PROTECTED
- Owner/blocker classes recorded; no Buck wiring or deletion from census lane
- Totals remain 152 / 992 / 32; delete candidates 0
- Artifact: `.omc/ultragoal/G030-U-EXACT-RESIDUAL-FIXTURE-INVENTORY-2026-08-02.md`

### G028 independent review still blocked (2026-08-02T~22:45Z)
- Latest `oh-my-claudecode:code-reviewer` retry on `051bc7ec6` FAILED_TRANSPORT (decrypt)
- Still LOCAL ONLY; not APPROVE; not pushed; not applied
- Package updated: `.omc/ultragoal/G028-PR-PACKAGE-2026-08-02.md`

### G030-T remaining protected queue inventory + TSV correction (2026-08-02)
- Exact residual closed: 19 fixture + 13 non-fixture = 32 POLICY_PROTECTED
- Non-fixture residual is the closed set of accounts schema/README, 4 vcs frozen companions, foundation README, foundry-supervisor.toml, release supply-chain README, tick-log, claim-matrix ops-portal, cedar-scope md, products RETIREMENT md
- Accounting defect: G030-N counted out-of-universe `registry/release/evidence-packs.tsv` inside the 1,176 focus partition; TSV consumer edge remains true, counters corrected only
- Corrected totals after S+T: MACHINE_SSOT 152; GRAPH_WIRED 992; POLICY_PROTECTED 32
- Delete candidates 0; final-queue independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-T-REMAINING-PROTECTED-QUEUE-INVENTORY-2026-08-02.md`

### G030-S earlier-retained specs JSON canonical-gate correction (2026-08-02)
- 11 earlier-retained specs JSON rows promoted GRAPH_WIRED via the same complete canonical-JSON governed-root contract missed by domain-only probes
- Paths: 9 design-system catalog-only JSON + `specs/reorg/ci-graph-additions.json` + `specs/reorg/kernel-move-plan.BLOCKED.json`
- Domain gaps preserved: catalog-only design semantics; blocked/non-enumerated reorg companions remain non-executable as move plans
- Pre-T arithmetic was 152/993/31; G030-T corrects inherited N TSV inflation to 152/992/32
- Remaining protected queue after T: 19 fixture + 13 non-fixture; delete candidates 0
- Final-queue independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-S-EARLIER-RETAINED-SPECS-JSON-CANONICAL-GATE-CORRECTION-2026-08-02.md`

### G030-R residual-28 consumer proof — corrected (2026-08-02)
- 26 residual JSON rows promoted GRAPH_WIRED via the Buck2 Rust canonical-JSON gate's complete recursive `specs/**/*.json` corpus contract
- 2 Markdown rows remain POLICY_PROTECTED: `specs/policy/cedar-scope-schema.md`; `specs/products/RETIREMENT.md`
- Original k=5 draft superseded: exact-path search missed canonical-json's governed-root reader
- Stronger semantic edges remain for http-stack policy + 4 scorecard canonical files; foundry remains Retired/NonPrd despite byte-canonical gate wiring
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 982; POLICY_PROTECTED 42
- Remaining protected queue: 19 fixture + 23 non-fixture; delete candidates 0
- Delayed independent audit found the edge; coordinator confirmed against immutable tip. Audit evidence is not mutation/PR APPROVE.
- Artifact: `.omc/ultragoal/G030-R-RESIDUAL28-CONSUMER-PROOF-2026-08-02.md`


### G030-Q lifecycle-config all-config consumer (2026-08-02)
- 8 residual lifecycle JSON rows promoted GRAPH_WIRED through the Buck2-native all-config directory consumer
- 2 rows live/baselined (ADR, doc); 6 residual rows explicitly known-broken pending owner re-root-or-delete rulings
- feature-flag lifecycle excluded from this promotion because G030-E already counted its exact execution citation
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 956; POLICY_PROTECTED 68
- Remaining protected queue: 19 fixture + 49 non-fixture; delete candidates 0
- Independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-Q-LIFECYCLE-CONFIG-ALL-CONSUMER-PROOF-2026-08-02.md`

### G030-P singleton registry residual consumers (2026-08-02)
- 8 rows promoted GRAPH_WIRED: inherited-bominal-adrs.yaml; ci-fix-loop-retry-budget.json; dependency-blessed-allowlist.json; graph/architecture-map.json; hyperscaler-scorecards/index.json; merge-queue-admission-log.json; microservices.json; mistakes-ledger.json
- 2 rows remain POLICY_PROTECTED: merge-queue-tick-log.json (prose-only; writer ABSENT); claim-matrix/ops-portal.json (catalog-only; planned checker ABSENT)
- Double-count check clean vs G030-E..O; mistakes-ledger was join target in G030-I, not prior residual promotion
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 948; POLICY_PROTECTED 76
- Remaining protected queue: 19 fixture + 57 non-fixture; delete candidates 0
- Independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-P-SINGLETON-REGISTRY-RESIDUAL-CONSUMER-PROOF-2026-08-02.md`


### G030-O check-empirical existence contract (2026-08-02)
- 13 residual empirical scorecards promoted GRAPH_WIRED via score-cards inventory + is_file existence contract
- 1 row already counted in G030-I (score-card-pre-push-loop-recovery-patterns.json) — not re-promoted
- Closed 14↔14 tip residual / inventory join; delete candidates 0
- Existence wiring ≠ BLOCKER semantic sufficiency
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 940; POLICY_PROTECTED 84
- Remaining protected queue: 19 fixture + 65 non-fixture
- Independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-O-CHECK-EMPIRICAL-EVIDENCE-EXISTENCE-CONTRACT-PROOF-2026-08-02.md`

### G030-N capabilities + release registry consumers (2026-08-02)
- 3 rows promoted GRAPH_WIRED: foundry-internal.json; release/images.yaml; release/evidence-packs.tsv
- 2 rows remain POLICY_PROTECTED: foundry-supervisor.toml (unparsed); release/supply-chain/README.md (extension-skipped)
- Tip release residual width = 3 (G030-G table said 2; tip tree governs)
- Pre-release empty-scope manifests are structural inputs, not release-candidate evidence
- Reconciled totals after N: MACHINE_SSOT 152; GRAPH_WIRED 927; POLICY_PROTECTED 97
- Remaining protected queue after N: 19 fixture + 78 non-fixture; delete candidates 0
- Independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-N-CAPABILITIES-AND-RELEASE-REGISTRY-CONSUMER-PROOF-2026-08-02.md`


### G030-M foundation-bypass + capability-template consumers (2026-08-02)
- 6 YAML rows promoted GRAPH_WIRED: 3 foundation-bypass ledger rows + 1 root capability + 2 nested eval dependents
- 1 README remains POLICY_PROTECTED (extension-skipped by foundation loader)
- Foundation rows are structurally wired but unremediated windows expired (created 2026-05-21; all open past 2026-06-20)
- Capability root-only enumeration + eval_set/eval_run path resolution proven; nested dirs intentionally skipped by schema walk
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 924; POLICY_PROTECTED 100
- Remaining protected queue: 19 fixture + 81 non-fixture; delete candidates 0
- Independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-M-FOUNDATION-BYPASS-AND-CAPABILITY-TEMPLATE-CONSUMER-PROOF-2026-08-02.md`



### G030-L VCS registry live reader + frozen companion (2026-08-02)
- 1 row promoted GRAPH_WIRED: registry/vcs/changeset-event-log.json via live Rust changeset-state gates + catalog lane inputs
- 4 rows remain POLICY_PROTECTED frozen historical companions (concurrent-safe-paths, event-router, webhook-delivery-log, README)
- ADR-0363 freezes family as historical evidence; retired VCS implementations ABSENT; committed logs empty
- Authority/readership tension recorded: frozen-not-active AND currently gate-read; no delete/reactivation
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 918; POLICY_PROTECTED 106
- Remaining protected queue: 19 fixture + 87 non-fixture; delete candidates 0
- Independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-L-VCS-REGISTRY-LIVE-READER-FROZEN-COMPANION-PROOF-2026-08-02.md`



### G030-K design-system residual consumer + catalog gap (2026-08-02)
- Exact residual recovered: tip 32 − root-hub 15 = 17 (prior 17-vs-32 discrepancy closed)
- 8 residual rows promoted GRAPH_WIRED via product-prd-json existence contract on PRD-ANONYMOUS/PRD-SOCIAL
- 9 residual rows remain POLICY_PROTECTED catalog-only (no component_refs validator)
- Draft status does not exempt PRD- rows from the gate
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 917; POLICY_PROTECTED 107
- Remaining protected queue: 19 fixture + 88 non-fixture; delete candidates 0
- Independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-K-DESIGN-SYSTEM-RESIDUAL-CONSUMER-CATALOG-GAP-PROOF-2026-08-02.md`



### G030-J accounts registry consumer + contract divergence (2026-08-02)
- 3 example TOML rows promoted to GRAPH_WIRED_INPUT via supervisor FileAccountSnapshotProvider directory enumeration
- schema.json + README.md remain POLICY_PROTECTED (unwired schema / contract documentation)
- Contract divergence recorded: schema fields/providers ≠ runtime/examples; parser_ref path ABSENT; --validate-accounts flag ABSENT outside README
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 909; POLICY_PROTECTED 115
- Remaining protected queue: 19 fixture + 96 non-fixture; delete candidates 0
- Independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-J-ACCOUNTS-REGISTRY-CONSUMER-CONTRACT-DIVERGENCE-PROOF-2026-08-02.md`



### G030-I loop-recovery pattern + empirical scorecard proof (2026-08-02)
- 3 pattern rows + 1 empirical scorecard promoted to GRAPH_WIRED_INPUT
- Rust gate enumerates dir, joins score-card IDs + mistakes ledger, enforces active blockers and empirical path existence
- Buck target marketplace-dev-cli-loop-recovery-patterns + affected-set expectation present
- Detector-query equivalence and evidence-path resolution remain structural-only gaps
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 906; POLICY_PROTECTED 118
- Delete candidates 0; independent audit FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-I-LOOP-RECOVERY-PATTERN-EMPIRICAL-CONSUMER-PROOF-2026-08-02.md`



### G030-H reorg-plan consumer/landed/retention proof (2026-08-02)
- 8 ordinary `*-move-plan.json` rows machine-enumerated + mechanically landed (old absent / new present)
- `ci-graph-additions.json` retained as ADR-0563 Cargo.lock companion (not a move-plan)
- `kernel-move-plan.BLOCKED.json` outside active glob; unapplied blocked design retained
- Reconciled totals: MACHINE_SSOT 152; GRAPH_WIRED 902; POLICY_PROTECTED 122
- Delete candidates 0; no plan edited/activated
- Independent audit retry FAILED_TRANSPORT_NOT_APPROVE
- Artifact: `.omc/ultragoal/G030-H-REORG-PLAN-CONSUMER-LANDED-RETENTION-PROOF-2026-08-02.md`


### G026 console-shell + marketplace-plugin placement proof (2026-08-02)
- portal shell → console/facade class; distinct from ops workspace-shell implementation
- plugin invocation engine → marketplace/core class; suffix `-app` does not override engine semantics
- Shell path locks mapped; plugin has one live external importer (billing saas-bench Cargo + 2 Buck labels)
- Exact leaf spelling remains owner/review decision; no move-plan JSON
- Artifact: `.omc/ultragoal/G026-CONSOLE-SHELL-MARKETPLACE-PLUGIN-PLACEMENT-PROOF-2026-08-02.md`


### G026 workspace API facade collision/importer proof (2026-08-02)
- Four APIs independently movable as capability facades; not app/workspace
- Exact destination path free for chat/meet/drive/forms under owning faces
- Zero external Cargo/Buck importers; rewrite set = self + membership + authz frozen paths
- Exact leaf spelling still owner-approved; no move-plan JSON
- Artifact: `.omc/ultragoal/G026-WORKSPACE-API-FACADE-COLLISION-PROOF-2026-08-02.md`


### G030-G reconciled semantic census (2026-08-02)
- Corrected non-overlapping partition: MACHINE_SSOT 152; GRAPH_WIRED_INPUT 894; POLICY_PROTECTED_ONLY 130
- Fixture accounting closed: 6 already-graph + 112 promoted + 19 residual = 137
- Remaining protected queue: 19 fixture residual + 111 non-fixture; not a delete queue
- Next bounded family: 10 `specs/reorg/*` plans consumer/retention proof
- Artifact: `.omc/ultragoal/G030-G-RECONCILED-SEMANTIC-CENSUS-2026-08-02.md`


### G026 application cone owner-split (2026-08-02)
- 8 crates / 4 ownership classes; hard rule remains: do not birth `app/application`
- shell → console substrate; plugin → marketplace; chat/meet → comms facade; drive → storage facade; forms → workflow facade
- `oya-application-app` = true 2+ composition but product name OWNER_RULING_REQUIRED
- `oya-cloud-surface-domain` = boundary unresolved; KEEP until owner proves capability or named product
- workspace APIs are NOT `app/workspace`; common data-boundary helper ≠ 2nd capability
- No move-plan JSON; capability-lane serial moves only after independent APPROVE + collision proof
- Artifact: `.omc/ultragoal/G026-APPLICATION-CONE-OWNER-SPLIT-2026-08-02.md`


### G030-F fixture contract expansion (2026-08-02)
- Immutable fixture tree = 137 paths / 19 families
- 118 paths / 16 families have exact or family-directory machine consumers → GRAPH_WIRED_INPUT
- Residual 19: calendar 15 have a colocated but non-Buck Python replay consumer; CRATEADR 3 are ADR-retained with no measured machine consumer; DR 1 has a transitional non-Buck Python bridge
- Residuals remain POLICY_PROTECTED_MACHINE_ARTIFACT; deletion candidates 0
- G030-E 131 was a conservative first-pass bucket; next reconciliation must recompute all 1,176 rows once, not arithmetically add and double-count
- Artifact: `.omc/ultragoal/G030-F-FIXTURE-CONTRACT-EXPANSION-2026-08-02.md`

### G030-E specs/registry semantic-consumer census (2026-08-02)
- 1,176 protected focus rows partitioned conservatively: MACHINE_SSOT 152; GRAPH_WIRED_INPUT 782; POLICY_PROTECTED_ONLY 242
- Corrected exact-literal blind spot: producer glob expands all 748 `registry/catalog/*.yaml` rows into required catalog/SLO/liveness gates
- Largest unresolved family = 131 `specs/fixtures/**`; next is gate-specific fixture contract expansion, not deletion
- No path in specs/registry is a delete candidate; protected:true unchanged
- Artifact: `.omc/ultragoal/G030-E-SPECS-REGISTRY-SEMANTIC-CONSUMER-CENSUS-2026-08-02.md`


### G026 product app face-birth design (2026-08-02)
- 90 crates / 30 tops: refuse 90-row face map (59 role=other)
- Product birth = empty `app/<product>/` roots (+ office/healthcare contexts), not capability faces
- `app/` ABSENT on tip; first slice P0 meta root only, then P1 named roots, moves only after 2+ capability proof
- `oya/application` is NOT `app/application`: shell→console substrate; multi-cap app needs owner product name; saas-plugin→marketplace facade candidate
- Office cone groups 27 crates (office+7 context tops); translate owner-pick
- Zero move-plan JSON; activation after design APPROVE + preferred #1526/#1523 health
- Artifact: `.omc/ultragoal/G026-PRODUCT-APP-FACE-BIRTH-DESIGN-2026-08-02.md`


### G026 CI runtime face-birth design (2026-08-02)
- 12 crates mapped to non-colliding ci/{core,adapters,facade} leaves; ports deferred (traits stay in kernels)
- Apps → facade/{controller,tide,webhook-gateway}; gate facade names remain disjoint
- No move-plan JSON; activation after independent design APPROVE + preferred #1526/#1523 health
- Artifact: `.omc/ultragoal/G026-CI-RUNTIME-FACE-BIRTH-DESIGN-2026-08-02.md`


### G030-D ephemeral dual-proof (2026-08-02)
- 11 focus + 2 sibling jsonl nested `docs/**/.omc/state/**` tracked on tip
- Consumer proof NEGATIVE (no exact path seed/literal in ci/governance/specs/registry/.github/libs)
- Authority proof UNRESOLVED (under docs/audit + docs/decisions; root gitignore is root-scoped only)
- Disposition: EPHEMERAL_COMMITTED_RUNTIME_LEAK — DELETE_CANDIDATE_PENDING_OWNER_RULING
- NOT DARK_BUREAUCRACY; deletion not authorized
- Artifact: `.omc/ultragoal/G030-D-EPHEMERAL-DUAL-PROOF-2026-08-02.md`


### G030-C root/SSOT citation audit (2026-08-02)
- ROOT_AUTHORITY proven for README/CLAUDE/AGENTS/HANDOFF via markdown-retirement + root-hub + repo-root-hygiene
- specs/ 360 and registry/ 816 focus rows are POLICY_PROTECTED (unit-class protected:true)
- Exact affected-set semantic seeds: specs 7, registry 3 — not full-tree MACHINE_SSOT
- masterplan.json and capability-registry.json authority-live but not exact synthetic seeds
- Artifact: `.omc/ultragoal/G030-C-ROOT-SSOT-CITATION-AUDIT-2026-08-02.md`
- Next: dual-proof probe on 11 focus ephemeral docs paths; semantic-consumer census for protected trees


### G030-B unit-class histogram (2026-08-02)
- Producer-equivalent first-match on committed unit-class-policy.json (no scm-facts run)
- Focus 13,959 → husk 5,798 (41.5%); doc 5,721; protected registry/build_config/spec/vendor 2,230; scratch 0
- oya/ husk focus 4,729 = G026 non-code/product shells, not bulk-delete
- Smallest candidate class: 11 focus ephemeral session-state JSON under docs/audit + docs/decisions (authority proof likely fails)
- Artifact: `.omc/ultragoal/G030-B-UNIT-CLASS-HISTOGRAM-2026-08-02.md`
- State: PLANNING_ONLY — next G030-C SSOT citations; no deletion


### G030 non-code corpus baseline (2026-08-02T later)
- Authority: `origin/dev` `b651080374113aeb57500eecbd9d1326f0404e48`
- Focus family md+yaml/yml+json+toml = **13,959** / tracked 18,886
- Prefix focus: oya 6,521; docs 3,137; cloud 1,230; registry 816; specs 360; evidence 189; root authority md 4
- Controllers mapped; generated faces remain decommitted; dual-proof + anti-vacuity rules recorded
- Artifact: `.omc/ultragoal/G030-NONCODE-CORPUS-BASELINE-2026-08-02.md`
- State: PLANNING_ONLY — next G030-B read-only producer histogram; no deletion/activation
- Independent review: transport still fused; no APPROVE inferred


- G026 CI/product destination-face proof written: capability `ci` owns the 12 runtime crates; leaf destinations absent; gate plan remains 0-overlap; product 90 crates are APP_FACE_BIRTH_REQUIRED (`app/` absent); 48 non-code shells are not DELETE candidates.
- G028 #1529 MERGED tip `0c1014b87` declares 22Gi; live ARS/ERS still 20Gi (reconciler inert). Prior founder ruling fixes class B / KEEP_CURRENT_LAB=true; independent design APPROVE pending; no helm apply.
- PR train: #1526 affected-set FAIL (corpus class; no cold FULL rerun at 20Gi); #1528 runner-loss class; #1523 buck2 FAIL restack unpushed; #1524 draft DO-NOT-MERGE CONFLICTING.
- G023 remains LIVE 170-file pending deletion after #1523; G025 KEEP/REFACTOR only.


# G030-M foundation-bypass and capability-template consumer proof — 2026-08-02

State: **PLANNING_ONLY — SIX YAML ROWS GRAPH-WIRED; ONE README RETAINED; NO EXCEPTION/TEMPLATE EDIT**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-L-VCS-REGISTRY-LIVE-READER-FROZEN-COMPANION-PROOF-2026-08-02.md`.  
No bypass ledger, capability template, gate, policy, PR, GitOps declaration, or cluster state was changed.

## Result

Two residual families share the same shape: a directory-default Rust governance gate enumerates or resolves committed YAML under an exact registry path. That is executable graph wiring. It is not proof that every current row still passes window/publish checks, and it is not delete authority.

### Family A — `registry/foundation-bypasses/*` (4)

| Path | Measured consumer/retention evidence | Disposition |
|---|---|---|
| `registry/foundation-bypasses/byp_adr_0346_oya_verify_ci_mirror.yaml` | foundation-bypass gate defaults to this directory; non-recursive root `*.yaml` enumeration; required foundation-bypass fields present; domain build accepts `byp_` IDs | `GRAPH_WIRED_INPUT — LEDGER DIRECTORY ENUMERATION` |
| `registry/foundation-bypasses/byp_adr_0347_governance_bulk_rename.yaml` | same directory consumer and field shape | `GRAPH_WIRED_INPUT — LEDGER DIRECTORY ENUMERATION` |
| `registry/foundation-bypasses/byp_adr_0348_sharding_automation.yaml` | same directory consumer and field shape | `GRAPH_WIRED_INPUT — LEDGER DIRECTORY ENUMERATION` |
| `registry/foundation-bypasses/README.md` | owner contract for empty-vs-missing ledger and validation command; skipped by the gate's `.yaml` extension filter | `POLICY_PROTECTED_MACHINE_ARTIFACT — CONTRACT DOCUMENTATION` |

### Family B — `registry/capability-templates/*` (3)

| Path | Measured consumer/retention evidence | Disposition |
|---|---|---|
| `registry/capability-templates/cap.demo.readiness.yaml` | root capability YAML read by both foundry-capability-schema and foundry-eval gates; declares eval_set/eval_run relative paths | `GRAPH_WIRED_INPUT — ROOT CAPABILITY RECORD` |
| `registry/capability-templates/eval-sets/cap.demo.readiness.yaml` | resolved from the root record's `eval_set` field and parsed by foundry-eval | `GRAPH_WIRED_INPUT — RESOLVED EVAL SET` |
| `registry/capability-templates/eval-runs/cap.demo.readiness.yaml` | resolved from the root record's `eval_run` field and parsed by foundry-eval | `GRAPH_WIRED_INPUT — RESOLVED EVAL RUN` |

This promotes six rows from the protected-only queue. The reconciled totals become **152 `MACHINE_SSOT` + 924 `GRAPH_WIRED_INPUT` + 100 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. The remaining protected queue is 19 fixture residuals plus 81 non-fixture rows. Delete candidates remain 0.

## Foundation-bypass consumer proof

`marketplace/facade/dev-cli/src/foundation_audit_gates.rs`:

1. defaults `ledger_dir` to exactly `registry/foundation-bypasses`;
2. `read_dir`s that directory non-recursively;
3. accepts only files whose extension is exactly `yaml`;
4. parses scalar `key: value` fields;
5. defaults `entry_class` to `foundation-bypass` when absent;
6. requires `id`, `pr_ref`, `crate_ref`, `gate_bypassed`, `bypassing_actor`, `rationale`, `regression_window_days`, and `created_at_epoch_days`;
7. builds domain records through `oya-intelligence-bypass-domain`;
8. validates open/remediated windows against `now_epoch_days`.

Dispatch and catalog evidence:

- CLI dispatch arm: `oya gate validate foundation-bypass` in `marketplace/facade/dev-cli/src/commands/gate/mod.rs`;
- catalog lane ID `foundation-bypass` with `LaneInputs::Global`.

The three committed YAML rows all carry the default foundation-bypass field set and `byp_*` IDs. The domain package accepts `byp_` prefixes, `oya-` crate refs, and `svc_` actors. Therefore the rows are structural inputs to the live loader.

### Expiry / fail-closed note — not unwired

Computed epoch-day windows at tip:

| id | created | window days | expires | status as of 2026-08-02 (day 20667) |
|---|---:|---:|---|---|
| `byp_adr_0346_oya_verify_ci_mirror` | 20594 (2026-05-21) | 14 | 20608 (2026-06-04) | open and expired |
| `byp_adr_0347_governance_bulk_rename` | 20594 | 30 | 20624 (2026-06-20) | open and expired |
| `byp_adr_0348_sharding_automation` | 20594 | 30 | 20624 (2026-06-20) | open and expired |

None of the three rows set `remediated_at_epoch_days`. A current-day foundation-bypass validation is therefore expected to fail closed on expiry. That is an enforcement/remediation defect for the owning migration/exception lifecycle, not evidence that the files are unwired or deletable. The README remains protected documentation because the loader never reads it.

## Capability-template consumer proof

Two gates default to the same directory:

### foundry-capability-schema

`marketplace/facade/dev-cli/src/foundry_capability_schema_gates.rs`:

1. defaults `capabilities_dir` to `registry/capability-templates`;
2. non-recursively enumerates root entries;
3. skips directories and non-`.yaml` files;
4. parses the root capability record into a domain `Capability` (id, namespace, autonomy tier, data classes, evidence topic, cost profile, MCP description/schemas);
5. fails if zero root capability YAML rows exist;
6. also reads `registry/capabilities/foundry-internal.json` for cross-id uniqueness (outside this residual family).

At tip the only root YAML is `cap.demo.readiness.yaml`. Nested `eval-sets/` and `eval-runs/` directories are intentionally skipped by this gate's root-only walk.

### foundry-eval

`marketplace/facade/dev-cli/src/foundry_eval_gates.rs`:

1. defaults to the same capabilities directory;
2. uses the same root-only YAML enumeration;
3. requires each root capability `status` to be `published`;
4. reads `eval_set` and `eval_run` scalars from the root record;
5. resolves both as paths under the capabilities directory;
6. parses the eval set (including cases) and eval run;
7. checks capability IDs match;
8. registers the set, records the run, and asserts publish readiness.

The committed root row declares:

- `eval_set: eval-sets/cap.demo.readiness.yaml`
- `eval_run: eval-runs/cap.demo.readiness.yaml`

Those exact nested paths exist and contain the fields the eval parsers require. Therefore all three family rows are executable graph inputs even though only the root row is directory-enumerated; the nested rows are path-resolved dependents.

Dispatch and catalog evidence:

- CLI dispatch arms for `foundry-capability-schema` and `foundry-eval`;
- catalog lane IDs with broad `registry/**` + `specs/**` globs;
- artifact-capabilities registry names all three exact paths as intended evidence inputs.

No dedicated Buck/affected-set expectation for these exact residual paths was required for this promotion. Source-graph readers and catalog registration establish wiring; protected required-context execution remains separately unproven here.

## Anti-vacuity and semantic boundary

Proven:

- foundation residual = 4; capability residual = 3;
- foundation gate defaults to the exact ledger directory and parses root YAML;
- three foundation YAML rows match the default foundation-bypass field contract;
- their unremediated windows are expired by 2026-08-02;
- capability schema and eval gates default to the exact templates directory;
- only one root capability YAML exists and is parsed by both gates;
- eval gate resolves and parses both nested YAML dependents;
- README is extension-skipped by the foundation loader.

Not proven:

- successful current-day foundation-bypass validation (expected fail on expiry);
- protected required-context execution of either family in every `oya-ci-required` run;
- semantic equivalence between artifact-capabilities declared commands and live cloud-ci producers;
- owner remediation or declassification of expired bypasses;
- any additional capability template beyond the bootstrap demo row.

These are enforcement and lifecycle gaps, not G030 delete authority.

## Verification boundary

Evidence came from immutable tree enumeration, exact directory defaults, loader traversal, field/path resolution, domain ID rules, epoch-day window arithmetic, gate dispatch/catalog registration, and artifact-registry citations at `b651080374113aeb57500eecbd9d1326f0404e48`. No local CLI execution is used as merge authority.

An independent verifier lane retried this partition and failed with the same encrypted-content transport error. It remains `FAILED_TRANSPORT_NOT_APPROVE`; the mechanical proof is not independent approval.

## Non-actions and non-claims

- No bypass YAML remediated, extended, or deleted.
- No capability template or nested eval fixture edited.
- No claim that expired open bypasses are currently valid exceptions.
- No claim that publish-ready bootstrap demo proves a full capability product surface.
- No move-plan JSON, generated face, or multispectrum evidence surface added.
- No independent APPROVE inferred from transport failure.

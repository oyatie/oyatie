# G030-N capabilities and release registry consumer proof — 2026-08-02

State: **PLANNING_ONLY — THREE ROWS GRAPH-WIRED; TWO COMPANIONS RETAINED; NO REGISTRY EDIT**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-M-FOUNDATION-BYPASS-AND-CAPABILITY-TEMPLATE-CONSUMER-PROOF-2026-08-02.md`.  
No capability registry, release manifest, gate, policy, PR, GitOps declaration, or cluster state was changed.

## Result

Two residual families share the same decision rule used since G030-H: promote only on an exact executable reader edge; retain companions that are directory-contract docs or unparsed historical configuration.

### Family A — `registry/capabilities/*` (2)

| Path | Measured consumer/retention evidence | Disposition |
|---|---|---|
| `registry/capabilities/foundry-internal.json` | exact default `--internal-registry` for foundry-capability-schema; rich-row validator reads and parses the JSON array; capability-registry-app seed parser + integration test ascend to this exact path and publish ≥50 capabilities | `GRAPH_WIRED_INPUT — LIVE INTERNAL CAPABILITY REGISTRY` |
| `registry/capabilities/foundry-supervisor.toml` | three `[[driver]]` rows present; exact path/basename machine consumers outside historical prose/audit maps = 0; no TOML loader defaults to this path | `POLICY_PROTECTED_MACHINE_ARTIFACT — UNPARSED SUPERVISOR DRIVER SPEC` |

### Family B — `registry/release/*` (3 tip files)

| Path | Measured consumer/retention evidence | Disposition |
|---|---|---|
| `registry/release/images.yaml` | exact default for release-supply-chain and supply-chain empty-scope declaration; parser reads release_state/empty_scope_rationale comments and `images: []` | `GRAPH_WIRED_INPUT — PRE-RELEASE EMPTY-SCOPE MANIFEST` |
| `registry/release/evidence-packs.tsv` | exact default manifest for release-evidence-pack; parser requires regulator TSV header and empty-scope comments; zero data rows accepted in pre-release | `GRAPH_WIRED_INPUT — PRE-RELEASE EMPTY EVIDENCE-PACK MANIFEST` |
| `registry/release/supply-chain/README.md` | only tip child under default evidence_dir; evidence walk accepts only `.yaml`/`.yml` and skips this file; directory existence is the contract, not the README body | `POLICY_PROTECTED_MACHINE_ARTIFACT — EVIDENCE DIRECTORY CONTRACT DOCUMENTATION` |

This promotes three rows from the protected-only queue. The reconciled totals become **152 `MACHINE_SSOT` + 927 `GRAPH_WIRED_INPUT` + 97 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. The remaining protected queue is 19 fixture residuals plus 78 non-fixture rows. Delete candidates remain 0.

Note on residual width: G030-G's earlier non-fixture table listed `registry/release/*` as 2. Immutable tip enumeration is three paths (two root manifests + nested supply-chain README). This proof uses the tip tree, not the earlier width estimate.

## Capabilities consumer proof

### foundry-internal.json

`marketplace/facade/dev-cli/src/foundry_capability_schema_gates.rs`:

1. defaults `internal_registry_path` to exactly `registry/capabilities/foundry-internal.json`;
2. `validate_foundry_capability_schema_gate` reads that path through `read_foundry_internal_registry_records`;
3. `validate_foundry_internal_registry_value` requires a non-empty JSON array of objects;
4. each row must carry `id` in `foundry.*`, matching `namespace`, plus `name`, `owner_team`, `status`, autonomy/evidence fields;
5. IDs are uniqueness-joined against root capability-template YAML IDs.

`oya/intelligence/crates/oya-intelligence-capability-registry-app/src/lib.rs` separately:

1. documents the seed schema as exactly this file;
2. provides `parse_seed_json` for the array shape;
3. integration-tests by ascending from `CARGO_MANIFEST_DIR` until `registry/capabilities/foundry-internal.json` exists;
4. asserts ≥50 publishable capabilities and zero default T4Actuate seeds.

At tip the file contains 57 rich capability rows; the required field set used by the gate is present. Therefore the row is an executable graph input for both the governance gate and the capability-registry app seed path.

CLI/catalog evidence:

- usage text exposes `--internal-registry <registry/capabilities/foundry-internal.json>`;
- foundry-capability-schema remains a catalogued gate lane with `registry/**` inputs.

### foundry-supervisor.toml — not promoted

Immutable content is three driver tables (`claude-driver`, `codex-driver`, `gemini-driver`) with `provider_family` / `autonomy_tier` scalars. Exact-path and basename searches outside the file itself hit only historical scorecard/audit prose. Provider-family enums and CLI session drivers exist elsewhere in the intelligence tree, but none default-read or parse this TOML path. Shared-directory membership with the live JSON seed is not a consumer edge. Retain as policy-protected configuration residue; no delete authority.

## Release consumer proof

### images.yaml

`marketplace/facade/dev-cli/src/supply_chain_gates.rs`:

1. defaults `release_images_path` to `registry/release/images.yaml` for both supply-chain ADR-0039 empty-scope detection and release-supply-chain validation;
2. `read_release_artifact_manifest` parses comment scalars `release_state` and `empty_scope_rationale`, skips `images:` / `images: []`, and collects artifact refs;
3. `empty_scope_declared` is true only when artifacts are empty, `release_state == pre-release`, and rationale is non-empty;
4. pre-release phase accepts that empty declared scope; release phase requires per-artifact evidence.

Committed tip content matches the empty-scope contract exactly. Dispatch exists as `oya gate validate release-supply-chain` and run-all includes `release-supply-chain --phase pre-release`. Catalog retains the broader `supply-chain` lane and release-evidence-pack globs over `registry/**`.

### evidence-packs.tsv

Same module:

1. defaults `manifest_path` to `registry/release/evidence-packs.tsv`;
2. requires the regulator TSV header;
3. parses optional `release_version` / `empty_scope_rationale` comments;
4. accepts zero data rows when records are not required (default compliance-matrix SLA).

Committed tip content is header-only with pre-release empty-scope comments. Dispatch: `oya gate validate release-evidence-pack`. Catalog lane ID `release-evidence-pack` with `evidence/**` + `registry/**` globs.

### supply-chain/README.md — not promoted

Default evidence_dir is `registry/release/supply-chain`. The directory walker:

1. requires the directory to exist in release phase;
2. enumerates only non-directory `.yaml`/`.yml` children;
3. therefore skips README.md by extension.

Tip has no YAML evidence children. Pre-release may accept zero evidence records; the README is human contract text for that intentional emptiness, not a machine-read attestation. Directory presence is structural; the README body is not. Retain protected.

`registry/release/image-promotions` is a defaulted promotion_dir with **no tip residual files**; absence is recorded, not invented as a protected row.

## Anti-vacuity and semantic boundary

Proven:

- capabilities residual = 2; release tip residual = 3;
- exact default path + parser for foundry-internal.json;
- capability-registry-app seed integration path to the same file;
- tip internal registry has 57 foundry.* rows with required fields;
- no exact current machine reader for foundry-supervisor.toml;
- exact defaults and parsers for images.yaml and evidence-packs.tsv;
- both manifests declare pre-release empty scope and parse as such;
- supply-chain evidence walk skips non-YAML, including README;
- CLI dispatch arms exist for release-supply-chain and release-evidence-pack.

Not proven:

- protected required-context execution of every release/capability lane in each `oya-ci-required` run;
- that empty pre-release manifests prove a release candidate readiness product;
- any runtime supervisor loading of the TOML driver table;
- owner approval to rename, move, or delete the unparsed supervisor TOML;
- existence of image-promotion residual files (directory default only; tip ABSENT).

## Verification boundary

Evidence came from immutable tree enumeration, exact path/basename searches, JSON/TOML/YAML/TSV shape inspection, Rust default paths and parsers, CLI dispatch/run-all registration, and gate-catalog citations at `b651080374113aeb57500eecbd9d1326f0404e48`. No local CLI execution is used as merge authority. No independent APPROVE; transport remains fused and is not treated as approval.

## Non-actions and non-claims

- No capability or release registry row edited or deleted.
- No supervisor TOML promoted by shared-directory inference.
- No claim that empty pre-release manifests are release-blocking evidence packs.
- No image-promotions rows invented for an absent directory.
- No move-plan JSON, generated face, or multispectrum evidence surface added.
- No independent APPROVE inferred from transport failure.

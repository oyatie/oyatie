# G030-J accounts-registry consumer and contract-divergence proof — 2026-08-02

State: **PLANNING_ONLY — THREE TOML ROWS GRAPH-WIRED; SCHEMA/README RETAINED; CONTRACT DIVERGENCE RECORDED**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-I-LOOP-RECOVERY-PATTERN-EMPIRICAL-CONSUMER-PROOF-2026-08-02.md`.  
No account row, schema, documentation, parser, gate, policy, PR, GitOps declaration, or cluster state was changed.

## Result

The five-row `registry/accounts/*` residual is not a deletion queue. The runtime supervisor enumerates every `.toml` file in this exact directory, so all three committed examples are executable graph inputs. The README and JSON Schema are protected owner contracts, but no executable consumer of either exact path was found.

| Path | Measured consumer/retention evidence | Disposition |
|---|---|---|
| `registry/accounts/claude.example.toml` | supervisor defaults `FileAccountSnapshotProvider` to `registry/accounts`; enumerates every `.toml`; parses this row into a live `SupervisorAccount` snapshot | `GRAPH_WIRED_INPUT — RUNTIME DIRECTORY ENUMERATION` |
| `registry/accounts/codex.example.toml` | same runtime directory consumer | `GRAPH_WIRED_INPUT — RUNTIME DIRECTORY ENUMERATION` |
| `registry/accounts/gemini.example.toml` | same runtime directory consumer | `GRAPH_WIRED_INPUT — RUNTIME DIRECTORY ENUMERATION` |
| `registry/accounts/schema.json` | draft machine-readable owner contract; artifact registry and README claim schema validation, but no reader of this exact path was found and its `parser_ref` is stale | `POLICY_PROTECTED_MACHINE_ARTIFACT — UNWIRED SCHEMA` |
| `registry/accounts/README.md` | owner documentation and declared lifecycle/validation contract; no machine consumer; documented validation CLI does not exist at tip | `POLICY_PROTECTED_MACHINE_ARTIFACT — CONTRACT DOCUMENTATION` |

This promotes three rows from the protected-only queue. The reconciled totals become **152 `MACHINE_SSOT` + 909 `GRAPH_WIRED_INPUT` + 115 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. The remaining protected queue is 19 fixture residuals plus 96 non-fixture rows.

## Runtime consumer proof

`oya/intelligence/crates/oya-intelligence-supervisor-app/src/main.rs` constructs:

`FileAccountSnapshotProvider::new("registry/accounts")`.

Its `snapshot` implementation:

1. calls `fs::read_dir` on that directory;
2. accepts every entry whose extension is exactly `toml`;
3. reads each accepted file as text;
4. extracts `id`, `provider_family`, and `secret_ref` with line-prefix/string-split parsing;
5. maps the three example provider values to runtime variants;
6. constructs one `SupervisorAccount` per readable row with a syntactically accepted secret reference.

The directory currently contains no non-example TOML files. Therefore the three examples are the complete immutable runtime input set at this tip. Absence of their exact filenames in Rust is not negative consumer proof: the executable edge is extension-filtered directory enumeration.

The committed artifact-capabilities registry separately names `registry/accounts/claude*.toml`, `codex*.toml`, and `gemini*.toml` as intended settings-renderer inputs. That is corroborating declared graph intent, not evidence that those documented commands execute or validate these rows in protected CI.

## Contract divergence

The executable loader, examples, schema, and README do not share one row contract:

- Runtime/examples use `id`, `subscription_id`, `state`, `secret_ref`, and provider values `Claude`, `OpenAIOrCodex`, and `Gemini`.
- The schema/README require `account_id`, `display_name`, `autonomy_tier_ceiling`, `cost_ceiling_tokens`, `priority`, and provider values `Anthropic`, `OpenAI`, and `Google`.
- The schema says it is parsed by `microservices/intelligence/crates/oya-intelligence-settings-template-kernel/src/account_kernel.rs:48`; that path does not exist at the immutable tip.
- The current `intelligence/core/account-kernel` defines identity/reference value types and provider parsing, not a TOML or JSON-Schema loader.
- The README documents `oya-intelligence-supervisor-app -- --validate-accounts`; exact search finds that flag only in the README. The supervisor main has no validation argument parser.
- The artifact registry claims settings-template schema verification, while the measured `SettingsRenderer` interface accepts an already-typed `ProviderAccount`; no registry-account TOML or `schema.json` reader was found in that kernel.

Accordingly, the three TOML rows are **graph-wired runtime inputs**, but this does not certify schema conformance. The schema and README remain policy-protected because they record an owner contract and intended validation lifecycle; they are not promoted to graph-wired based on unimplemented claims.

## Fail-closed and anti-vacuity boundary

Proven:

- the supervisor's default runtime path is the exact accounts directory;
- all three committed TOML files match its extension filter;
- all three expose the line fields the minimal loader reads;
- all three provider values hit explicit runtime match arms;
- all three secret references satisfy the runtime's broad `sref://` check.

Not proven, and in several cases contradicted:

- JSON-Schema validation;
- README validation-command existence;
- exact schema/runtime field or provider-value agreement;
- minimum account count of ten (the directory has three rows, and the claimed threshold mechanism is shell prose rather than a measured owned gate);
- fail-closed parsing: unreadable entries are skipped; absent fields receive defaults; unknown providers default to Claude; there is no duplicate-ID or exact row-shape check;
- protected required-context execution of a settings-drift or account-schema Buck target.

These are enforcement defects for the owning intelligence/account lifecycle, not authority for G030 to rewrite or delete the files.

## Verification boundary

Evidence came from immutable source and exact searches at `b651080374113aeb57500eecbd9d1326f0404e48`: the five registry files, supervisor runtime construction and loader, current account/settings-template kernels, artifact-capabilities declarations, path-existence probes, and repository-wide exact flag/path searches. No local CLI execution is used as merge authority.

An independent Explore audit retried this family and failed with the same encrypted-content transport error. It remains `FAILED_TRANSPORT_NOT_APPROVE`; the mechanical proof is not independent approval.

## Non-actions and non-claims

- No account row, schema, README, or runtime loader edited.
- No claim that the examples conform to `schema.json`.
- No claim that declared settings-drift commands execute.
- No claim that the current permissive loader is production-safe.
- No delete or declassification candidate.
- No new generated face, move-plan JSON, or multispectrum evidence surface.
- No independent APPROVE; transport failure remains non-approval.

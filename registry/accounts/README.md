# registry/accounts/ — Foundry Supervisor Account Schema

Per-driver account list for `intelligence-supervisor`. Each `.toml` file declares one or more provider accounts for a single driver family (Claude / Codex / Gemini).

## Schema

Defined at [`schema.json`](./schema.json) (JSON Schema 2020-12). The canonical parser is `crates/intelligence-settings-template-kernel/src/account_kernel.rs:48`.

### Row shape (TOML)

```toml
[[account]]
account_id = "acct-001"
provider_family = "Anthropic"       # CamelCase — case-sensitive per account-kernel:48
display_name = "Default Claude account"
secret_ref = "sref://openbao/foundry/claude/acct-001"   # NO raw keys — ADR-0067
autonomy_tier_ceiling = "T2"        # T1=read/suggest; T2=write+human-approval; T3=auto-write; T4=disabled
cost_ceiling_tokens = 100000        # rolling input+output budget; enforced by UsageEnforcement::check_limit
priority = 1                        # lower = higher priority; supervisor selects lowest with capacity
```

### Required fields

| Field | Type | Constraint |
|---|---|---|
| `account_id` | string | `acct-NNN` format; unique across workspace |
| `provider_family` | string | One of `Anthropic`, `OpenAI`, `Google` (CamelCase) |
| `display_name` | string | 1–128 chars |
| `secret_ref` | string | Must start with `sref://openbao/` |
| `autonomy_tier_ceiling` | string | One of `T1`, `T2`, `T3`, `T4` |
| `cost_ceiling_tokens` | integer | ≥ 0 |
| `priority` | integer | ≥ 1 |

### Optional fields

| Field | Type | Description |
|---|---|---|
| `model_hint` | string | Override preferred model for this account |
| `tags` | string[] | Free-form grouping tags for dashboards |

## Secret references

All secrets use OpenBao `sref://` references per **ADR-0067** (`secret-reference-openbao-pattern`). The supervisor resolves them at runtime via `SecretStorePort` (implemented in `intelligence-account-adapter-openbao`). **Never commit raw API keys.**

## Autonomy tier ceilings

Enforced at runtime by `docs/policies/autonomy-ceiling.cedar` + `docs/policies/foundry-supervisor.cedar` loaded together by `autonomy-ceiling-app::enforce_for_tenant`.

| Tier | Meaning |
|---|---|
| T1 | Read + suggest only |
| T2 | Write with human approval |
| T3 | Automated write |
| T4 | Disabled by default (Cedar blanket-forbid) |

## N-threshold for drift lane activation

The **`lean-settings-drift`** CI lane is feature-flagged **off in production** until the total row count across all `registry/accounts/*.toml` files reaches **N ≥ 10**.

- **Before threshold:** lane runs against fixture files only in CI — passes vacuously for an empty fleet. This is intentional, not a false positive (per v5 §A.1.2 + §C.26).
- **At/after threshold:** lane runs against real account files and enforces schema + drift detection.

Threshold check at lane start:

```sh
wc -l registry/accounts/*.toml | tail -1 | awk '{print $1}'
```

## Validation

```sh
# Validate a single file (exit 0 = valid)
cargo run -p intelligence-supervisor-app -- --validate-accounts registry/accounts/claude.example.toml

# Validate all
for f in registry/accounts/*.toml; do
  cargo run -p intelligence-supervisor-app -- --validate-accounts "$f"
done
```

## Files

| File | Provider |
|---|---|
| `claude.example.toml` | Anthropic (Claude) |
| `codex.example.toml` | OpenAI (Codex) |
| `gemini.example.toml` | Google (Gemini) |

## Ownership

`axis-foundry` — M02-P06-IP (foundry-supervisor). Schema governed by `account-kernel:48` parser invariants.

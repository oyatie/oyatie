# Spec: gate run-all --affected [--base <ref>]

## Objective

Extend `oya gate run-all` with `--affected [--base <ref>]` so presubmit runs
execute only the governance lanes triggered by the diff against a base ref,
mirroring the affected-scope selection already used by `oya verify --affected`
(ADR-0360 O1). `--ci-required` remains the authoritative whole-workspace trunk
backstop and always forces the full lane set.

## Crate boundary

All changes are confined to `crates/dev-cli` — no new workspace member, no
changes to `Cargo.toml` at the repo root. This is consistent with the flat
clean-arch doctrine (ADR-0509).

## Mod layout (flat clean-arch per ADR-0509)

```
crates/dev-cli/src/commands/gate/run_all.rs   ← sole change site
```

No new modules, no new abstractions beyond the two new fields on `RunAllArgs`.

## Contracts / standards

* Not an HTTP or gRPC service; no OpenAPI/AsyncAPI/proto3 contract surface.
* No OTel spans added (gate aggregator is a local CLI tool, not a long-lived
  service; OTel is already handled by the individual gate handlers).
* No new SLO required (CLI tool, not a µservice).
* Reuses existing `verify_affected::changed_files()` (git subprocess, already
  audited) and `governance_gate_catalog_domain::lanes_for_changed()` (pure
  domain function, no I/O).

## CLI surface

```
oya gate run-all [--include-deferred] [--ci-required] [--affected [--base <ref>]]
```

| Flag | Semantics |
|------|-----------|
| `--affected` | Run only lanes triggered by the diff vs `<base>` (default `origin/dev`). |
| `--base <ref>` | The git ref to diff against; only valid with `--affected`. Defaults to `origin/dev` when `--affected` is used without `--base`. |
| `--ci-required` | Trunk backstop: forces the full lane set; overrides `--affected` narrowing. |
| `--include-deferred` | Unchanged semantics. |

## Selection algorithm

1. When `--affected` is absent: existing behaviour (full `AGGREGATED_VALIDATE_LANES`).
2. When `--ci-required` is present: full `AGGREGATED_VALIDATE_LANES` regardless
   of `--affected` (trunk backstop wins).
3. When `--affected` is present and `--ci-required` is absent:
   a. Call `verify_affected::changed_files(repo_root, base)`.
   b. Convert `Vec<String>` to `Vec<&str>`.
   c. Call `governance_gate_catalog_domain::lanes_for_changed(&changed_refs)`.
   d. Iterate the returned subset in catalog order; dispatch only those lanes.
   e. Log `[gate run-all] affected mode: {selected}/{total} lanes selected`.
4. If `changed_files()` returns an error (e.g. git unavailable), emit a warning
   and fall back to the full lane set (fail-safe / conservative).

## Testing strategy

Unit tests (no subprocess, no filesystem):

1. `ci_required_selects_full_lane_set` — parse `["--ci-required"]`, assert
   `RunAllArgs::ci_required == true` and that the resolved lanes equal
   `AGGREGATED_VALIDATE_LANES` (full set).
2. `affected_flag_parses_default_base` — parse `["--affected"]`, assert
   `RunAllArgs::affected == true` and `base == "origin/dev"`.
3. `affected_flag_with_explicit_base` — parse `["--affected", "--base", "main"]`,
   assert `base == "main"`.
4. `affected_narrows_lanes_for_sample_diff` — construct a sample changed-files
   list (`["docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md"]`), call `lanes_for_changed` directly,
   assert the result is smaller than the full set and contains only expected
   lanes (e.g. `adr-citation`, `adr-supersession-consistency`).
5. `affected_flag_unknown_adjacent_flag_rejected` — verify unknown flags are
   still rejected.

## Observability / SLO

No SLO change required; this is a CLI tool with no hosted-service surface.

## Security

`--base <ref>` is passed directly to `git merge-base HEAD <ref>` inside
`changed_files()`. That function already exists and is validated; no new
subprocess injection vector is introduced here.

## Cloud-native readiness

The flag is consumed by the local CLI only. Foundry pipeline callers that pass
`--ci-required` are unaffected; the trunk backstop remains unconditional.

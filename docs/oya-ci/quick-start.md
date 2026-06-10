# oya-ci quick-start / adoption

Adopting the oya-ci conformance floor in a repo is three steps: drop a config, run the producer,
wire the CI lanes. With no config you still get a valid, GREEN, empty-but-present baseline.

## 1. Drop an `oya-ci.toml` at the repo root

Start from the [config reference](./config-reference.md). The minimum useful config enables the
language-agnostic gates; a Rust/Cargo repo also enables `bnf-layer-suffix`,
`manifest-hygiene`, `cargo-prefix`, and `workspace-glob-coverage`. Repos with catalog records
also enable `slo-coverage`. oyatie's own `oya-ci.toml` (at this repo's root) is a worked,
complete example.

Zero-config is valid: with no `oya-ci.toml` present the producer materializes the compiled-in
bundled default (language-agnostic gates on, empty policy tables, repo root = `.`), so a fresh repo
gets a non-erroring baseline that names zero foreign paths.

## 2. Generate the faces

```sh
cargo run -p oya-cloud-ci-accounting-registry-app -- --repo-root .
```

This writes the generated faces under
`cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/`, including
`gate-baseline.generated.json` — the frozen accepted-violation set the firewall ratchets against.
Commit the faces. (Generated faces carry no wall-clock, so `committed == regenerated` holds
byte-for-byte and registry-drift can byte-diff them.)

### The new-file settle (two commits)

When you first add tracked files (the config crate, the `oya-ci.toml`, new source), the producer
accounts them, so the baseline grows by their accounting keys. The registry also records each
non-generated file's `last_touch_commit`. Because a brand-new file has no commit yet at the moment
you regenerate, freeze it in **two commits**:

1. commit the source + the regenerated faces;
2. re-run the producer (now the files have a commit) and commit the regenerated
   `accounting-registry.generated.json` only.

A third regen then yields zero face delta — convergence. (The `gate-baseline.generated.json` face
carries no `last_touch`, so it is stable across the settle.)

## 3. Wire the CI lanes

The required check is a single fan-in job, `oya-ci-required`, that depends on every gate lane. The
homogeneous gates run as a `strategy.matrix` of `cargo test -p <crate>` lanes; the producer-regen,
`registry-drift`, and `cloud-ci-firewall` lanes are bespoke jobs.

> **Caveat (do NOT use a reusable `workflow_call`):** a called workflow renames published
> check-runs to `<caller> / <job>`, which breaks the required-context name `oya-ci-required`. For
> external repos, ship a **composite action** (`uses:`-able — it does not rename check-runs) plus a
> copy-in matrix template, and keep the `oya-ci-required` fan-in job in the consumer's own
> workflow. See [the firewall model](./firewall-model.md) for why the fan-in is the gate.

## What "GREEN" means

- registry-drift: the committed faces equal a fresh regen (no hand-edits, no stale faces).
- firewall: no NEW key beyond the committed baseline for any `baseline-block-on-new` code, and no
  un-signed-off baseline GROWTH on regen.

To accept new pre-existing debt into the baseline (rare, audited), a founder adds the key to
`gate-baseline.signoff.json` — the one-way door. Everything else only ever shrinks.

# oya-repo-hooks-bootstrap

Hermetic, zero-manual-step activation for the born-accounting faces merge driver + hooks
(task #125 v1, ADR-0572).

## The bootstrap problem this fixes

`.gitattributes <faces-glob> merge=oya-faces` NAMES a merge driver; it does NOT DEFINE one. A driver
is active only if `merge.oya-faces.driver` exists in *that clone's* git config — per-clone local state
the repo cannot carry, and absent on fresh CI runners. The existing `oya-cargo-lock-merge-driver` and
`oya-friction-ledger-merge-driver` READMEs document a manual `git config merge.* …` line; a repo-wide
grep shows nothing in tracked source sets it. The bootstrap problem is currently UNSOLVED for those
drivers too. The founder bar is AUTOMATED, not flag-only — activation must be a binary, not a README
line.

## What it does (idempotent)

`oya-repo-hooks-bootstrap [--repo-root <path>] [--driver-bin <path>]`:

1. `git config --local merge.oya-faces.{name,driver}` (driver = `<abs> driver %O %A %B %P`).
2. `git config --local core.hooksPath .githooks`.
3. installs `.githooks/{post-merge,post-rewrite,post-checkout}` 2-line shims — post-merge/post-rewrite
   run `oya-faces-merge-driver settle` (the authoritative post-commit regeneration); post-checkout
   re-runs this idempotent bootstrap so activation stays fresh.
4. regenerates the `.gitattributes` generated glob block from the control-plane (`gitattributes`
   subcommand; `gitattributes --check` is the CI drift guard, exit 1 on stale).

Re-running is a no-op. Driver-path resolution: `--driver-bin` > `OYA_FACES_MERGE_DRIVER` env > the
bare name `oya-faces-merge-driver` (git resolves it from PATH at merge time).

## Universality

The faces-glob set is read from `registry/generated-artifact-control-plane.json` (via
`oya_faces_merge_driver_app::ControlPlane`), so `.gitattributes` cannot drift from the declared
policy. A repo adopting oya-ci ships its own manifest and gets the same activation.

## Activation lanes (zero manual step)

The `.githooks/` shims + the git config are per-clone RUNTIME activation state, not committed
artifacts (gitignored, like the merge-driver git config and the ADR-0551/0552 per-checkout derived
baselines). The bootstrap binary installs them idempotently:

- **CI runners:** one `oya-repo-hooks-bootstrap .` step before any merge-driver-consuming gate (the
  follow-up CI-yml wiring). Cheap, hermetic, idempotent.
- **Local dev:** one explicit `buck2 run //tools/oya-repo-hooks-bootstrap-app:oya-repo-hooks-bootstrap`
  on onboarding installs `core.hooksPath` + the shims; the installed `post-checkout` shim then re-runs
  the idempotent bootstrap after each checkout so activation stays fresh.
- **Merge queue (v2):** the tide worker runs the same bootstrap in its workspace.

Why an explicit command rather than a buck2 genrule side-effect (ADR-0572 / design §9.5): coupling a
git-config side-effect into the build graph is mildly unclean (builds should be pure); one documented
idempotent command is less magic and is the doctrine-endorsed alternative.

Why `--local`, not `--global`: `--global` pollutes the developer's machine and is not hermetic;
`--local` written by an idempotent checked-in binary is the per-repo hermetic answer.

## Irreducible-glue ledger

The `git config --local` writes are subprocesses (git is the config store). The `.githooks` shims are
2-line POSIX shells git spawns (git mandates hooks be executables). Both are the minimal
git-integration glue — no other shell. The Rust binary IS the auto-fix; the shim is the git-required
entrypoint.

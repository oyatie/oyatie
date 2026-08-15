# Agent Security Model

Runtime-managed security guidance for the Oyatie contributor environment. OS-enforced controls
are the real gate; repository hooks are optional local adapters only. Oyatie does not track agent
runtime configuration, so this document cannot claim a project-wide sandbox is active.

## Sandbox (OS-enforced, macOS Seatbelt)

Operators using Claude Code should enable its macOS Seatbelt sandbox in managed or user-level
settings. Repository-local `.claude/` is ignored and untracked; the project does not activate or
verify that sandbox on the operator's behalf.

### Filesystem restrictions

A managed sandbox profile should deny credential paths at the OS level:

```
~/.ssh/**            ~/.aws/**           ~/.config/gcloud/**
**/secrets/**        **/*.pem            **/*.key
**/id_rsa            **/id_ed25519       **/*.p12
**/*.pfx             **/*.kubeconfig
```

The same managed profile should deny writes to environment files. These are recommended operator
controls, not guarantees made by the repository:

```
.env    **/.env    **/.env.*
```

### Network allowlist

A managed profile should restrict subprocess egress to the domains needed for the build and source
workflow, for example:

```
localhost / 127.0.0.1 / *.local / *.internal
github.com / *.github.com / *.githubusercontent.com
crates.io / static.crates.io / index.crates.io
static.rust-lang.org / *.rust-lang.org
api.anthropic.com / *.anthropic.com
```

When that profile is installed, other egress should be blocked and a probe such as
`curl https://example.com` should fail. The repository neither installs nor verifies this
allowlist, so operators must not infer it from a clean checkout.

Note: `sandbox.network` is a proxy/hostname allowlist, not TLS-inspected. Domain-fronting
is a residual risk; the allowlist is the primary control.

## Secret handling

There is **no** Claude Code subprocess env-scrub mechanism (`settings.json` `env` only
*adds* vars; it cannot strip inherited ones). So this control is operational, not config:
**keep secrets out of the agent's shell environment** — never `export` `GITHUB_ADMIN_TOKEN`,
`OPENBAO_ROOT_TOKEN`, the OpenBao unseal keys, etc. session-wide. Source them just-in-time
inside the one command that needs them (e.g. the masked GitHub push reads the token via
`awk` from the gitignored `.env` and uses it only in that single push URL).

## Permissions

`permissions.deny` in managed agent-runtime settings can block Docker tool invocations (Docker
Inc. tooling is forbidden per ADR-0381) using the tool-family deny pattern, not
arg-validation regex. It **also** denies `Read(...)` of credential paths (`~/.ssh/**`,
`~/.aws/**`, `**/*.pem`, `**/*.kubeconfig`, `**/secrets/**`, …): the sandbox
`filesystem.denyRead` covers only the **Bash** tool and its children, so without these
`Read`-tool deny rules an agent could read those files via the `Read`/`Edit` tools. The
two layers merge into the final boundary when a runtime supports both; neither is configured by
this repository.

> Scope note: project-scope `allowRead`/`allowedDomains` are merge-additive (loosenable by
> user/local settings). For a hard guarantee, `denyRead` + `allowedDomains` +
> `disableBypassPermissionsMode` belong in **managed** settings, and
> `sandbox.failIfUnavailable` should be `true` in CI/managed scope so a missing sandbox
> fails loud rather than silently running unconfined.

`disableBypassPermissionsMode: "disable"` prevents `--dangerously-skip-permissions`
from being used to circumvent the permission model.

## Runtime adapters

Repository-local hook adapters are retired and are **not** a security gate. Operators may install
runtime-managed safeguards outside Git, but their presence or behavior is not asserted by this
repository. Protected CI and runtime enforcement remain the project backstops.

The `exfil-guard.sh` and `no-secret-leak.sh` regex hooks were removed in the
2026-05-29 security redesign because regex-based PreToolUse hooks are bypassable
(shell quoting, eval, heredoc, subshell indirection). The OS sandbox replaced them
as the authoritative control.

## Codex configuration (recommended)

Do not edit `~/.codex/config.toml` automatically — contributors must opt in. The
recommended settings:

```toml
sandbox_mode = "workspace-write"   # confines writes to the workspace
approval_policy = "on-request"     # agent asks before shell side-effects
```

## Configuration verification

After an operator installs or changes managed agent security settings and restarts the runtime,
verify that specific external profile with manual checks such as these (the repository does not
claim the results in advance):

```sh
# 1. .env read works (GitHub token fetch must not be blocked)
cat .env

# 2. git fetch works (github.com is in the allowlist)
git fetch origin --dry-run

# 3. cargo deny check works (crates.io + index.crates.io are in the allowlist)
cargo deny check

# 4. External egress is blocked (MUST fail with network error, not connection)
curl https://example.com
# expected: connection refused / network sandbox block
```

If step 4 succeeds (returns HTTP 200), the sandbox is not active — confirm you restarted
Claude Code after the settings change and that your macOS version supports Seatbelt for the Claude
Code process.

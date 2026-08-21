# Agent Security Model

CONFIG-PRIMARY security for the oyatie contributor environment. OS-enforced controls
are the real gate; hooks are advisory/monitoring only.

## Sandbox (OS-enforced, macOS Seatbelt)

`sandbox.enabled = true` in `.claude/settings.json` activates macOS Seatbelt on every
bash subprocess spawned by Claude Code. This is enforced by the OS — it cannot be
bypassed by prompt injection or by modifying hook scripts.

### Filesystem restrictions

`denyRead` blocks credential paths at the OS level:

```
~/.ssh/**            ~/.aws/**           ~/.config/gcloud/**
**/secrets/**        **/*.pem            **/*.key
**/id_rsa            **/id_ed25519       **/*.p12
**/*.pfx             **/*.kubeconfig
```

`denyWrite` (anti-clobber) — the repo-root `.env` is readable (legitimate awk reads
for the GitHub token work), but write is blocked:

```
.env    **/.env    **/.env.*
```

### Network allowlist

Only the following domains are reachable from subprocesses:

```
localhost / 127.0.0.1 / *.local / *.internal
github.com / *.github.com / *.githubusercontent.com
crates.io / static.crates.io / index.crates.io
static.rust-lang.org / *.rust-lang.org
api.anthropic.com / *.anthropic.com
```

All other egress is blocked. This covers buck2/cargo/reindeer/git fetches and the
local GitHub push (localhost:3000). `curl https://example.com` will fail — that is
the intended behaviour.

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

`permissions.deny` in `.claude/settings.json` blocks Docker tool invocations (Docker
Inc. tooling is forbidden per ADR-0381) using the tool-family deny pattern, not
arg-validation regex. It **also** denies `Read(...)` of credential paths (`~/.ssh/**`,
`~/.aws/**`, `**/*.pem`, `**/*.kubeconfig`, `**/secrets/**`, …): the sandbox
`filesystem.denyRead` covers only the **Bash** tool and its children, so without these
`Read`-tool deny rules an agent could read those files via the `Read`/`Edit` tools. The
two layers MERGE into the final boundary — both are required.

> Scope note: project-scope `allowRead`/`allowedDomains` are merge-additive (loosenable by
> user/local settings). For a hard guarantee, `denyRead` + `allowedDomains` +
> `disableBypassPermissionsMode` belong in **managed** settings, and
> `sandbox.failIfUnavailable` should be `true` in CI/managed scope so a missing sandbox
> fails loud rather than silently running unconfined.

`disableBypassPermissionsMode: "disable"` prevents `--dangerously-skip-permissions`
from being used to circumvent the permission model.

## Hooks (advisory/monitoring only)

Hooks are **not** the security gate. The former Cargo-blocking hook was retired by
ADR-0716 when Cargo became the merge path. The remaining hooks provide:

- `injection-content-scanner.sh` (PostToolUse) — advisory OWASP LLM01 scanner;
  always exits 0, never blocks.

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

After changing agent security settings and restarting Claude Code, verify the sandbox is active with these manual checks (sandbox only takes effect after restart):

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

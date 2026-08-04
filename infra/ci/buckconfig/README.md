# Buck2 CI cache/RE configuration

This directory stores opt-in Buck2 CI overlay templates for NativeLink cache and
cache-only execution.

## Use cases

- `warm-cache-ro.buckconfig`
  - Read-only warm cache endpoint (`allow_cache_uploads = false`).
  - Safe default posture for first-stage throughput experiments.

- `warm-cache-rw.buckconfig`
  - Read/write cache endpoint (`allow_cache_uploads = true`).
  - Use only after evidence that cache integrity and multi-tenant isolation are
    safe.

## Recommended pipeline wiring

`infra/ci/warm-buck2-cache.sh` is the canonical CI-only helper. It writes
`.buckconfig.local` from environment variables (or safe defaults) and is designed
to be a no-op when cache/re is not enabled.

When enabled, the helper does:

1. resolves RE mode (`ro` / `rw` / `off`),
2. writes an overlay with `[buck2_re_client]`, `[oya_cache]`, and
   `toolchains//cache:cache-platform`,
3. supports TLS mode and optional mTLS cert path.

### Environment variables

- `OYA_CI_RE_CACHE_MODE` (`off` | `ro` | `rw`) — `off` removes overlay.
- `OYA_CI_RE_ENDPOINT` — override endpoint URI (defaults to reader/writer
  service names for `ro`/`rw`).
- `OYA_CI_RE_INSTANCE_NAME` — RE instance name, default `main`.
- `OYA_CI_RE_TLS` — `true`/`false`, default `true`.
- `OYA_CI_RE_TLS_CLIENT_CERT` — optional mTLS cert path.
- `OYA_CI_RE_REMOTE_CACHE_ENABLED` — explicit `true`/`false` override.
- `OYA_CI_RE_ALLOW_UPLOADS` — explicit `true`/`false` override.

Repository-level default in this commit enables CAS read-cache (`ro`) for GitHub
Actions via `OYA_CI_RE_CACHE_MODE` unless explicitly overridden via CI vars.

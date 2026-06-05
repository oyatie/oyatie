# SDK Engineer — First Week on `developer-sdk`

Audience: a software engineer with multi-language SDK experience (familiar with at least one of: openapi-generator, swagger-codegen,
protoc, Stripe-style SDKs, AWS SDK v3) joining the `oya-developer-sdk-*` lane.

## Day 1 — required reading

- `docs/decisions/ADR-0220-developer-sdk-multi-language.md` — binding scope.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md` — gRPC over HTTP/3 + 3 invariants.
- `docs/decisions/ADR-0253-http3-quic-default-protocol.md` — protocol-level expectations.
- `oya/developer-sdk/feature-parity-matrix-2026-05-20.md` — language and ecosystem capability coverage.
- OpenAPI 3.2.0 spec (the one that landed Nov 2025; required reading especially for `webhooks` + `pathItems` updates).
- gRPC HTTP/3 RFC draft `draft-ietf-grpc-http3-04`.

Clone:
```bash
git fetch origin dev
git worktree add -b onboarding/$USER-sdk-week1 .worktrees/$USER-sdk-week1 origin/dev
```

## Day 2 — walk the generator end-to-end

Bring up the `sdk-loopback-1` dev cell through the registered Buck2/Prow dev-cell harness with `PROFILE=developer-sdk-dev`.

Trigger regeneration through the registered Buck2 SDK projection target from the loopback OpenAPI bundle. Keep generated TypeScript/Python SDK
artifacts outside the monorepo unless a lane-owned registry fixture explicitly requires them; Buck2 owns the in-repo verification evidence.

Look at the output tree:
```
.gen/loopback-1/
  rust/        # cargo-publishable
  typescript/  # npm-publishable
  python/      # PyPI-publishable (wheel + sdist)
```

Read `oya/developer-sdk/reference-implementations/multi-language-canary-rust-sdk.md` and the generator shard named by the active implementation
plan to see how the Rust prelude is wired (this is the part that makes generated SDKs feel hand-written: `Result<T, OyatieError>`, `Tenant`
newtype, `tracing::instrument` on every fn, etc).

## Day 3 — build a sample app against the generated SDK

```bash
cd .gen/loopback-1/rust
buck2 build <registered-rust-sdk-smoke-target>
cd ../../..
```
Then run the registered Rust SDK sample fixture. Keep any Cargo manifest edits outside the repo unless they are part of Buck2/Reindeer metadata
maintenance.

For non-Rust SDK packages, run the registry fixture smoke through the SDK lane's Buck2 target rather than adding package-manager state to the repo.

## Day 4 — author a feature flag override

Pick a language extension from `oya/developer-sdk/implementation-plans/` or `oya/developer-sdk/sdk-plan.md`. Implement it in the lane-owned
generator shard recorded by that plan. Example: add per-call retry-with-jitter to the Go template.

Authoring path:
1. Edit the template file under `templates/`.
2. Add a regen-fixture under `fixtures/`.
3. Add a snapshot test under `tests/snapshot.rs`.
4. Run the registered Buck2 test target for the language generator shard.
5. Ensure Prow publishes green `lean-a5-doc-coverage` + `lean-a9-template-substance` evidence.

## Day 5 — release a dev-channel build through native SCM/Prow

Open a short-lived PR from your isolated worktree branch. Merge only after reviewer approval and the trusted Prow/Kubernetes-native
`oya-ci-required` context:
```bash
gh pr merge --auto --squash
```

The post-merge workflow regenerates all language SDKs, cosign-signs them, and publishes to the `dev` channel of each registry.
Verify registry publication through lane-owned smoke fixtures and attach those logs to PR/release evidence. Package-manager commands are
ecosystem compatibility checks only; they are not monorepo merge authority.

## Done with week 1

- [ ] You can recite the language availability matrix.
- [ ] You regenerated all SDKs from a canonical spec on dev-cell.
- [ ] You built + ran a Rust sample against the generated SDK.
- [ ] You shipped a template change through native SCM/Prow and saw it publish to dev channels.
- [ ] You read ADR-0220 + ADR-0145 + ADR-0253 + the OpenAPI 3.2.0 changes.

## Rookie traps

1. **Hand-editing generated code.** Forbidden. If a generated SDK has a wart, fix the template + add a snapshot test.
2. **Skipping cosign.** Every publish must be signed; unsigned publishes fail at the registry-side webhook (we own the webhook).
3. **Bumping major without ADR.** Breaks `no_silent_regression`; CI lane refuses.
4. **Forgetting `Tenant`.** Every SDK call carries a tenant; tenant-less calls fail compile in Rust, fail TS strict-mode, etc.

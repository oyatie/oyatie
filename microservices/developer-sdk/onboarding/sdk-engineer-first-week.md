# SDK Engineer — First Week on `developer-sdk`

Audience: a software engineer with multi-language SDK experience (familiar with at least one of: openapi-generator, swagger-codegen,
protoc, Stripe-style SDKs, AWS SDK v3) joining the `oya-developer-sdk-*` lane.

## Day 1 — required reading

- `docs/decisions/ADR-0220-developer-sdk-multi-language.md` — binding scope.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md` — gRPC over HTTP/3 + 3 invariants.
- `docs/decisions/ADR-0253-http3-quic-default-protocol.md` — protocol-level expectations.
- `microservices/developer-sdk/capability-ladders/tier-matrix.md` — language coverage by tenant_class.
- OpenAPI 3.2.0 spec (the one that landed Nov 2025; required reading especially for `webhooks` + `pathItems` updates).
- gRPC HTTP/3 RFC draft `draft-ietf-grpc-http3-04`.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-sdk-week1 .worktrees/$USER-sdk-week1
```

## Day 2 — walk the generator end-to-end

```bash
make dev-cell.up CELL=sdk-loopback-1 PROFILE=developer-sdk-dev
```

Trigger a regeneration of the Rust + TypeScript + Python SDKs from the loopback OpenAPI bundle:
```bash
./bin/oya sdk regen \
  --languages rust,typescript,python \
  --openapi crates/oya-openapi-canonical-v2026-05/spec/v1.yaml \
  --output-dir .gen/loopback-1
```

Look at the output tree:
```
.gen/loopback-1/
  rust/        # cargo-publishable
  typescript/  # npm-publishable
  python/      # PyPI-publishable (wheel + sdist)
```

Read `crates/oya-developer-sdk-codegen-rust/templates/` to see how the Rust prelude is wired (this is the part that makes generated
SDKs feel hand-written: `Result<T, OyatieError>`, `Tenant` newtype, `tracing::instrument` on every fn, etc).

## Day 3 — build a sample app against the generated SDK

```bash
cd .gen/loopback-1/rust
cargo build
cd ../../..
mkdir hello-sdk && cd hello-sdk && cargo init
echo 'oya-canonical-sdk = { path = "../.gen/loopback-1/rust" }' >> Cargo.toml
cat > src/main.rs <<'EOF'
use oya_canonical_sdk::{Client, Tenant};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new(
        "https://loopback.api-gateway.oyatie.local",
        Tenant::parse("oyatie.community.dev-sample")?,
    )?;
    let me = client.identity().who_am_i().send().await?;
    println!("hello, {}", me.principal_id);
    Ok(())
}
EOF
cargo run
```

Now do the same in TypeScript:
```bash
cd .gen/loopback-1/typescript && npm install && npm pack
```

## Day 4 — author a feature flag override

Pick a language extension from `microservices/developer-sdk/backlog/starter-features.md`. Implement under
`crates/oya-developer-sdk-codegen-<lang>/`. Example: add per-call retry-with-jitter to the Go template.

Authoring path:
1. Edit the template file under `templates/`.
2. Add a regen-fixture under `fixtures/`.
3. Add a snapshot test under `tests/snapshot.rs`.
4. Run `cargo test -p oya-developer-sdk-codegen-go`.
5. Ensure `lean-a5-doc-coverage` + `lean-a9-template-substance` lanes are green.

## Day 5 — release a dev-channel build through Foundry

```bash
./bin/oya vcs claim \
  --agent sdk-eng-$USER \
  --intent sdk-feature-go-retry-jitter \
  crates/oya-developer-sdk-codegen-go microservices/developer-sdk
```

Open PR; once admitted + reviewer-agent APPROVE:
```bash
gh pr merge --auto --squash
```

The post-merge workflow regenerates all language SDKs, cosign-signs them, and publishes to the `dev` channel of each registry.
You can verify:
```bash
cargo install --version 0.0.0-dev-<git-sha> oya-canonical-sdk
npm view @oyatie/sdk@dev
pip install --index-url https://pypi.oyatie.dev/simple oya-canonical-sdk==0.0.0.dev<sha>
```

## Done with week 1

- [ ] You can recite the language availability matrix.
- [ ] You regenerated all SDKs from a canonical spec on dev-cell.
- [ ] You built + ran a Rust sample against the generated SDK.
- [ ] You shipped a template change through Foundry and saw it publish to dev channels.
- [ ] You read ADR-0220 + ADR-0145 + ADR-0253 + the OpenAPI 3.2.0 changes.

## Rookie traps

1. **Hand-editing generated code.** Forbidden. If a generated SDK has a wart, fix the template + add a snapshot test.
2. **Skipping cosign.** Every publish must be signed; unsigned publishes fail at the registry-side webhook (we own the webhook).
3. **Bumping major without ADR.** Breaks `no_silent_regression`; CI lane refuses.
4. **Forgetting `Tenant`.** Every SDK call carries a tenant; tenant-less calls fail compile in Rust, fail TS strict-mode, etc.

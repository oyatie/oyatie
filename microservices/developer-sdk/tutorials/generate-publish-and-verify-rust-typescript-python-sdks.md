# Tutorial — Generate, publish, and verify Rust/TypeScript/Python SDKs from a canonical OpenAPI spec

Goal: take the canonical OpenAPI 3.2.0 spec, regenerate three language SDKs, publish each to a dev registry, install into a sample
app, and verify cosign signatures end-to-end.

Pre-reqs:
- Loopback dev cell: `make dev-cell.up CELL=sdk-loopback-1 PROFILE=developer-sdk-dev`
- A pinned commit of `crates/oya-openapi-canonical-v2026-05`
- Cosign installed (`brew install cosign` or `apt install cosign`)
- A tenant: `make dev-tenant.create T=oyatie.community.dev-sample tenant_class=demo_trial`

## Step 1 — inspect the canonical spec

```bash
ls crates/oya-openapi-canonical-v2026-05/spec/
# v1.yaml v1-aggregator-report.json
```

`v1.yaml` is the aggregated OpenAPI 3.2.0 spec (~6,400 lines). `v1-aggregator-report.json` shows per-µservice slice contributions.

## Step 2 — regenerate

```bash
./bin/oya sdk regen \
  --languages rust,typescript,python \
  --openapi crates/oya-openapi-canonical-v2026-05/spec/v1.yaml \
  --output-dir .gen/sdk-week1
```

Expected output (trimmed):
```
[rust]
  templates: 142 used, 0 fallback to upstream openapi-generator
  files     : 287 generated
  cargo build : OK in 47 s
[typescript]
  files     : 196 generated
  npm install : OK in 22 s
  tsc       : OK
[python]
  files     : 178 generated
  poetry build: wheel + sdist OK
```

## Step 3 — publish to dev registries

```bash
./bin/oya sdk publish \
  --languages rust,typescript,python \
  --channel dev \
  --source .gen/sdk-week1
```

Expected:
```
[rust]      published oya-canonical-sdk 0.0.0-dev-9f3c4a7 → crates-dev.oyatie.io
[typescript] published @oyatie/sdk 0.0.0-dev-9f3c4a7 → registry-dev.oyatie.io/npm
[python]    published oya-canonical-sdk 0.0.0.dev<sha> → pypi-dev.oyatie.io
[cosign]    signed 3 artifacts; attestations at https://attestations-dev.oyatie.io/9f3c4a7
```

## Step 4 — install + use the Rust SDK

```bash
cargo new sdk-tutorial-rust && cd sdk-tutorial-rust
cargo add oya-canonical-sdk --registry oyatie-dev
```

`src/main.rs`:
```rust
use oya_canonical_sdk::{Client, Tenant};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new(
        "https://loopback.api-gateway.oyatie.local",
        Tenant::parse("oyatie.community.dev-sample")?,
    )?;
    let me = client.identity().who_am_i().send().await?;
    println!("rust: {}", me.principal_id);
    Ok(())
}
```

```bash
cargo run
```

Expected: `rust: oyatie.community.dev-sample::User::"dev-sample-bot"`.

## Step 5 — install + use the TypeScript SDK

```bash
mkdir sdk-tutorial-ts && cd sdk-tutorial-ts && npm init -y
npm config set @oyatie:registry https://registry-dev.oyatie.io/npm
npm install @oyatie/sdk@dev
```

`index.mjs`:
```javascript
import { Client, Tenant } from '@oyatie/sdk';

const client = new Client({
  endpoint: 'https://loopback.api-gateway.oyatie.local',
  tenant: Tenant.parse('oyatie.community.dev-sample'),
});
const me = await client.identity.whoAmI();
console.log('typescript:', me.principalId);
```

```bash
node index.mjs
```

Expected: `typescript: oyatie.community.dev-sample::User::"dev-sample-bot"`.

## Step 6 — install + use the Python SDK

```bash
mkdir sdk-tutorial-py && cd sdk-tutorial-py
python -m venv .venv && source .venv/bin/activate
pip install --index-url https://pypi-dev.oyatie.io/simple oya-canonical-sdk
```

`main.py`:
```python
import asyncio
from oya_canonical_sdk import Client, Tenant

async def main():
    client = Client(
        endpoint="https://loopback.api-gateway.oyatie.local",
        tenant=Tenant.parse("oyatie.community.dev-sample"),
    )
    me = await client.identity.who_am_i()
    print(f"python: {me.principal_id}")

asyncio.run(main())
```

```bash
python main.py
```

Expected: `python: oyatie.community.dev-sample::User::"dev-sample-bot"`.

## Step 7 — verify cosign signature on each artifact

Rust:
```bash
cargo download oya-canonical-sdk --registry oyatie-dev --version 0.0.0-dev-9f3c4a7 -o /tmp/crate.tgz
cosign verify-blob --bundle https://attestations-dev.oyatie.io/9f3c4a7/rust/oya-canonical-sdk-0.0.0-dev-9f3c4a7.bundle /tmp/crate.tgz
```

TypeScript:
```bash
npm pack @oyatie/sdk@dev --pack-destination /tmp
cosign verify-blob --bundle https://attestations-dev.oyatie.io/9f3c4a7/typescript/sdk-0.0.0-dev-9f3c4a7.tgz.bundle /tmp/oyatie-sdk-0.0.0-dev-9f3c4a7.tgz
```

Python:
```bash
pip download --no-deps --dest /tmp oya-canonical-sdk==0.0.0.dev9f3c4a7 --index-url https://pypi-dev.oyatie.io/simple
cosign verify-blob --bundle https://attestations-dev.oyatie.io/9f3c4a7/python/oya_canonical_sdk-0.0.0.dev9f3c4a7-py3-none-any.whl.bundle /tmp/oya_canonical_sdk-0.0.0.dev9f3c4a7-py3-none-any.whl
```

Each verify call should return `Verified OK`.

## Step 8 — break a signature and confirm verify fails

Tamper:
```bash
echo "0" >> /tmp/oya_canonical_sdk-0.0.0.dev9f3c4a7-py3-none-any.whl
cosign verify-blob --bundle https://attestations-dev.oyatie.io/9f3c4a7/python/.../.bundle /tmp/oya_canonical_sdk-0.0.0.dev9f3c4a7-py3-none-any.whl
# Error: signature validation failed
```

This confirms the chain works.

## What you proved

- Three languages, one spec, one regen command.
- Publish targets dev registries with cosign signatures.
- Sample apps install + run against the loopback API.
- Tamper detection is real — modified artifacts fail cosign verify.

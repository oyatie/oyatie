# Laptop CAS — Cloudflare Tunnel + Access / mTLS

**BAN:** anonymous public gRPC to NativeLink. Tunnel without Access/mTLS is incomplete.

## Posture

```
GHA / local Buck2  --mTLS client cert-->  Cloudflare Access  --Tunnel-->  127.0.0.1:50051|50052  (NativeLink mTLS)
```

Belt = Cloudflare Access (Service Auth / device posture).  
Suspenders = NativeLink client CA verify on writer/reader listeners (same ADR-0560 seam as in-cluster).

Trusted workflows only hold **writer** client material. Forks / `untrusted-author-presubmit` get **nothing**.

## Example config (committed)

See `docs/runbooks/assets/laptop-cas/cloudflared/config.example.yml` and the live copy under `~/oyatie-cas/cloudflared/config.example.yml`.

Suggested hostnames (placeholder zone):

| Hostname | Origin |
|----------|--------|
| `cas-writer.lab.oyatie.dev` | `https://127.0.0.1:50051` |
| `cas-reader.lab.oyatie.dev` | `https://127.0.0.1:50052` |

Do **not** publish `:50061` ops via tunnel.

## Founder secrets (blocked until provided)

| Secret | Where it lives | Notes |
|--------|----------------|-------|
| Cloudflare Tunnel token / credentials JSON | Keychain / OpenBao — **never git** | `cloudflared tunnel create oyatie-cas-lab` |
| Access Service Auth client id/secret | GHA org/repo secrets for trusted classes only | Forks excluded |
| Production client CAs (writer/reader) | OpenBao `oya/ci/nativelink-cas-tls` pattern | Lab currently uses self-signed under `~/oyatie-cas/tls/` |
| Writer/reader client keys for GHA | GHA secrets → mode-0600 `.buckconfig.local` | Resolver pattern in `infra/ci/buckconfig/warm-cache-*.buckconfig` |

## Bring-up steps (once secrets exist)

1. `brew install cloudflare/cloudflare/cloudflared`
2. `cloudflared tunnel create oyatie-cas-lab`
3. Copy UUID + credentials into `~/oyatie-cas/cloudflared/` (gitignored locally)
4. Adapt `config.example.yml` → `config.yml`
5. Create Access applications for both hostnames (Service Auth)
6. `cloudflared tunnel --config ~/oyatie-cas/cloudflared/config.yml run` (launchd/KeepAlive recommended)
7. Point lab Buck2 overlays at `grpcs://cas-writer.lab.oyatie.dev:443` (or Access TCP pattern if L4)

## Precedent

In-cluster connector pattern: `infra/cloudflare/cloudflared.yaml` (k8s apiserver tunnel). Same outbound-only model; different origin service.

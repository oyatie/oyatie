# Reorg disposition — the unreferenced oya/ + cloud/ crates (2026-07-27)

24 agents · 1,631,401 subagent tokens · 468 tool calls. Every DELETE verdict
adversarially challenged before being accepted.

## Headline: the mechanical signal was wrong 58% of the time

| Stage | Result |
|---|---|
| Named crates in `oya/` + `cloud/` | 251 |
| Zero external references (raw) | 107 (43%) |
| …less 35 `[[bin]]` entrypoints, 11 CI-referenced | **61 truly unreferenced**, 35,774 src lines |
| Crates dispositioned (incl. context crates) | **80** |
| DELETE verdicts issued | 23 |
| DELETE verdicts challenged | 19 |
| **DELETE verdicts OVERTURNED** | **11 (58%)** |
| DELETE upheld | 8 |

**Final tally:** MOVE 32 · DELETE 23 (→ ~12 net after overturns) · CONSOLIDATE 13 ·
WIRE-UP 10 · REFACTOR 2.

Overturned verdicts revised to: CONSOLIDATE 6 · REFACTOR 2 · MOVE 2 · WIRE-UP 1.

**Conclusion: "zero references" identifies UNWIRED, not DEAD.** A codemod-driven reorg
keyed on reachability would have deleted roughly half of what it flagged, including the
highest-value code in the set. This is the founder's
MOVE/REFACTOR/REWRITE/DELETE quadrichotomy confirmed empirically.

## CORRECTION to the anti-pattern sweep

The sweep claimed — and I repeated — that **"ADR-0020 mandates one crate
`oya-intelligence-adapter-kernel` which does not exist."** That is **FALSE**.

It exists as **`oya-intelligence-adapter-domain`** (961 src lines, 973 test lines —
a 1.01:1 ratio, the best in the cluster). It carries the exact ADR-0020 surface:
`pub trait ProviderAdapter` (lib.rs:209), `ProviderAuth` (58), `InvocationPolicy` (88),
`ProviderEvent` (163), `ProviderRoute`/`ProviderRoutePreference`, `CostCeiling`,
`ProviderCallReceipt`, `PromptEnvelope`, `ToolSchemaSet`, and `SubscriptionBinding` +
`SubscriptionBindingRegistry` (the ADR's "Resolved item 1" tenant-attribution registry).
It is also **not unreferenced** — `oya/application/crates/oya-application-app` depends on
it by path.

**Revised remedy:** not "create the missing crate" but **MOVE + rename**
`oya/intelligence/crates/oya-intelligence-adapter-domain` →
`intelligence/core/adapter-kernel` (crate `intelligence-adapter-kernel`, the ADR-0020
name), then fold the six duplicated auth kernels into `intelligence/core/account-kernel`.
Nothing is wrong with the code — only its address and its `-domain` suffix.

## The intelligence adapter island — resolved

| Crate | Disposition | Note |
|---|---|---|
| `*-anthropic-subscription-adapter` | **WIRE-UP** | 1,852 src + 463-line hyper OAuth integration test. Singleflight refresh lock, BinaryHeap refresh ticker, PKCE enrollment, token state machine, terminal-vs-transient error classification, persists to `CredentialStorePort` BEFORE mutating in-memory state (correct crash ordering), `#![forbid(unsafe_code)]`. Last touched 2026-06-30 (PQC hybrid TLS #1037) — **active**. Consumer should be `oya-intelligence-provider-pool-app`, which already talks `api.anthropic.com` but has **no OAuth refresh path**. |
| `*-openai-subscription-adapter` | **WIRE-UP** + RENAME | 970 src + 264-line integration test. **MISNOMER**: this is API-KEY auth, not subscription — its own docs say "OpenAI API keys do NOT expire (distinct from Anthropic OAuth tokens)". Destination `intelligence/adapters/openai-apikey-pool`. Genuine OpenAI subscription OAuth already lives at `intelligence/adapters/codex-adapter` (981 LOC, Sign-in-with-ChatGPT). **Keep both — no overlap.** |
| `*-adapter-domain` | **MOVE** + rename | The ADR-0020 contract crate (see correction above) |
| 6 × `*-{provider}-{mode}-kernel` | **CONSOLIDATE** | 169 lines each; `diff` between any two = **3 changed lines** (doc comment, one const, one test assert). All six already `use intelligence_account_kernel::{ProviderFamily, SecretReference}` — the destination is proven, not speculative. ⚠ `anthropic-subscription-kernel` has a LIVE consumer (the 1,852-line OAuth adapter) so it needs an import swap, not a delete. ⚠ Carry over the `auth_token_debug_is_redacted` and AuthError Display-distinctness tests — do not drop them. |
| 4 × mock adapters | **DELETE** | 133 lines each, byte-identical modulo provider name, headers literally read "In-memory mock impl of ProviderAuthPort" |

## Execution ordering

1. **MOVE + rename `adapter-domain` → `intelligence/core/adapter-kernel`** — unblocks
   everything else in the cluster; pure relocation, no restructure.
2. **CONSOLIDATE the 6 auth kernels** into `account-kernel` as one parameterised port,
   atomically with **DELETE of the 4 mock adapters** (each kernel's only build-graph
   consumer is its paired mock).
3. **WIRE-UP the two live adapters** — highest product value in the whole set; this is
   the subscription-OAuth-pooling vertical.
4. The remaining 32 MOVE / 12 net DELETE / 2 REFACTOR proceed per the serial
   one-move-per-PR strangler playbook.

## Process lesson to keep

Two independent near-misses in one session, same shape:
- The sweep proposed "delete all 12 intelligence adapter crates" — would have destroyed
  3,549 LOC of live OAuth.
- My own reachability metric ranked the 1,852-line OAuth adapter as "truly dead."

Both were caught only by an adversarial second pass. **On this repo, unreferenced code
is disproportionately unfinished-valuable rather than abandoned**, because the reorg
itself is what left it unwired. Any automated cleanup MUST default to
should_delete=false under uncertainty.

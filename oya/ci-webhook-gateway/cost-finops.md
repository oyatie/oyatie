# Cost / FinOps — CI Webhook Gateway

See also `operational-boundaries.md` for the capacity model this cost view
derives from.

## Cost shape

- **Compute**: one small stateless pod (request ~50m CPU / 64Mi memory; limit
  ~250m / 128Mi). Per-delivery work is one HMAC + one JSON parse + one bounded
  TCP POST — negligible.
- **Storage**: none (stateless).
- **Egress**: only the in-cluster Jenkins kick (mesh-local; no public egress).

## FinOps attribution (ADR-0344 dimensional model)

- Cost center: `oya-substrate` (the change-coordination substrate), NOT any
  tenant. The gateway processes no tenant data and bills to no tenant.
- The expensive part of the gated pipeline is the **Jenkins CI run**, governed
  by the CI-farm cost program (ADR-0349/0360) and the per-changeset cost
  budgets (ADR-0113, carried forward) — not this gateway.

## Cost guardrails

- Horizontal scale is bounded (PR volume is low); one replica is the default.
- The per-changeset webhook fan-in cap (ADR-0112 carried-forward, 1000
  events/24h) bounds runaway kick loops, capping downstream CI spend.

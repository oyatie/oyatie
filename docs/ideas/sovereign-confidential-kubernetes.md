# Sovereign Confidential Kubernetes

> Status: **idea, not ratified.** Produced 2026-08-08 via `/idea-refine`. Nothing here
> supersedes an ADR. Where it disagrees with a ratified decision, that disagreement is
> named explicitly rather than assumed resolved.

## Problem Statement

How might we run other people's code on infrastructure we own, at a safety bar that
survives a real adversary — and give buyers a property the hyperscalers structurally
cannot offer: **the operator cannot read their memory**.

## Recommended Direction

Managed Kubernetes on the owned Rust substrate, with **hardware-attested
confidentiality** as the wedge, sold to KYC-verified enterprise accounts.

We will never beat EKS/GKE/AKS on price, features, or region count. The only durable
answer to *"why not just use EKS"* is a property they cannot match while remaining US
hyperscalers: attested confidentiality plus EU data residency. That reframes the Rust
substrate from a purity argument into a **small auditable TCB** argument — which is the
actual security case, and the one a customer's auditor understands.

It ships on **AMD SEV-SNP / Intel TDX** — commodity EPYC and Xeon, with Cloud Hypervisor
already carrying experimental TDX support. HyperEnclave's vendor-decoupled (TPM-rooted)
attestation is a **phase-2 upgrade, not a prerequisite**; treating it as one would put a
23-commit research repo with no ARM support on the critical path.

### The threat model inverts, and that IS the product

With KYC-verified enterprise tenants, the adversary is no longer the tenant. It is the
**operator** — us, our staff, anyone who compromises our control plane, and anyone who
serves us a subpoena. Tenant-vs-tenant risk drops sharply with KYC; operator-vs-tenant
risk becomes the entire value proposition, and SEV-SNP/TDX is precisely the answer to it.

Consequence: the isolation story must survive an **insider-threat question from a bank's
security team**, not a kernel researcher's fuzzer. Those demand different evidence.

### "Most are KYC'd" is not a security property

The bar is set by the **worst tenant accepted, not the average one**. If 5% are anonymous
card signups, that 5% sets the abuse posture and the isolation requirement, and the full
cost is paid for a rounding error of revenue.

So make it a gate, not a tendency. **One bar, two friction levels:** the self-serve trial
is KYC'd too, just lighter — card + business email + domain verification, with a low spend
ceiling until full verification. Anonymous signup is the thing to refuse outright.

## Key Assumptions to Validate

- [ ] **Buyers pay a premium for attested confidentiality.**
      → 5 discovery calls with regulated/EU buyers BEFORE writing code.
      Kill criterion: fewer than 2 would pay >20% premium.
- [ ] **Attestation UX is bearable.** This is where confidential computing usually dies —
      the customer receives a quote and does not know what to do with it.
      → Prototype the verification flow end to end, and walk an auditor through it.
      This is now the TOP risk, ahead of any isolation question.
- [ ] **SEV-SNP or TDX works on our substrate at acceptable overhead.**
      → Boot one Talos node as a confidential VM under Cloud Hypervisor and measure.
      Published figures suggest 2–10%; measure ours, do not cite theirs.
- [ ] **Enterprise procurement can be cleared in the runway available.**
      → SOC 2 Type II, DPAs, pen-test report, insurance certificates. 6–12 months of
      calendar time. Start in parallel today; it is the real critical path.
- [ ] **ARM is not required for v1.**
      → Asterinas arm64 is not on main and HyperEnclave has no ARM at all. If buyers
      require Graviton-class economics, the plan changes.

## MVP Scope

**IN:** one region, one AZ. Managed Kubernetes only. Confidential worker nodes on SEV-SNP.
An attestation verification flow a customer's auditor can actually follow. Three
contracted design partners, invoiced. Upstream Linux, KVM, containerd and Kata as-is.
VAP/CEL for tier admission.

**OUT:** anonymous self-serve, IaaS, functions, multi-region, autoscaling, the port
engine, the owned kernel, our own SDN controller.

The MVP tests exactly one thing: **will an enterprise pay for attested confidentiality,
and can their auditor verify it.** Everything else waits on that answer.

## Not Doing (and Why)

- **Free tier** — removes the dominant abuse vector for the cost of a signup form, and
  free-tier users are not the confidential-compute buyer.
- **Anonymous self-serve** — the security bar is set by the worst tenant accepted. One
  bar, two friction levels.
- **HyperEnclave in v1** — 23 commits, no ARM, requires VMX + IOMMU on bare metal. It is
  the phase-2 "do not trust the CPU vendor either" upgrade, not a dependency.
- **Owned kernel / OSDK greenfield on the critical path** — ADR-0338 Tier 0 runs tenant
  code whose syscall surface is unknowable, so the full Asterinas ABI is required there.
  Minimal-kernel work belongs to Tier 1, which is not on this path.
- **The Go→Rust port engine** — real and ratified (ADR-0637/0638), but it does not move
  the "will someone pay" needle. It runs on its own track.
- **Our own SDN controller** — modern datacenter fabrics use BGP/EVPN distributed control.
  A centralised controller is the architecture the industry moved away from, and it is a
  cross-cell coordination point against our own blast-radius doctrine.
- **All three product shapes at once** — it is ONE VM substrate with three packagings, not
  three products. Ship the substrate and one packaging.
- **Side-channel-aware placement before launch** — real, but with KYC'd enterprise tenants
  it is hardening backlog rather than launch-gating.

## Open Questions

- **Admission is stale and webhook-shaped.** ADR-0183 chose Kyverno; ADR-0379 superseded
  it with Kubewarden; ADR-0338 still cites Kyverno in 44 places — and it is the ADR that
  enforces tenant isolation. Both are webhooks: a failure domain in the isolation path,
  where failing open silently stops enforcing. VAP/CEL is GA in every version we run
  (1.30–1.38) and evaluates in-process, with `paramKind` for the tier map. Rust belongs in
  the **controller that projects manifest tiers into the cluster**, not in the admission
  hook. Filed as its own bead.
- **Does ADR-0338's `kata-clh` survive?** The Asterinas kata fork ships Dragonball and
  upstream Kata now defaults to it. Evidence lane pending.
- **Which is the funnel and which is the business** — self-serve trial vs contracted
  accounts. Leaning: trial is the on-ramp, contracts are the revenue.
- **Who owns abuse response**, even at ~1 FTE plus on-call? It is small under KYC, not zero.

---
doc_class: Idea-Onepager
doc_status: drafted
date: 2026-08-08
---

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

We will never beat EKS/GKE/AKS on price, features, or region count — and **confidential compute
is not the gap either.** This repo's own ADR-0147 hyperscaler table already records Confidential
GKE and Azure Confidential Containers on AMD SEV-SNP; AWS sells Nitro Enclaves; all three run EU
regions and sovereign/residency programmes. Any thesis resting on "they structurally cannot
offer attested confidentiality" is false on arrival, and an earlier draft of this page rested on
exactly that.

The residual property is narrower, and stating it exactly is the difference between a wedge and
a slogan. On EKS/GKE/AKS the **operator of the confidential boundary is a US-domiciled company**:
the enclave shields tenant memory from that operator's infrastructure, but the attestation root,
the key-release service, and the legal entity answering process are all still theirs. Our claim
is a different one and it has three parts — an **EU-domiciled operator**, a **TCB small enough
that a customer's auditor can enumerate it**, and attestation against **tenant-approved
measurements**, so that trusting us is not a precondition of the guarantee. That is a
jurisdiction-plus-TCB argument rather than a hardware-feature argument, and it is what turns the
Rust substrate from a purity preference into the thing that makes the TCB auditable at all.

Treat it as a claim to falsify, not a premise: the discovery calls below must show buyers
distinguish that residual property from Confidential GKE. If they do not, the wedge is wrong and
no amount of substrate work fixes it.

It ships on **AMD SEV-SNP / Intel TDX** — commodity EPYC and Xeon, with Cloud Hypervisor
already carrying experimental TDX support. HyperEnclave's vendor-decoupled (TPM-rooted)
attestation is a **phase-2 upgrade, not a prerequisite**; treating it as one would put a
23-commit research repo with no ARM support on the critical path.

### The threat model GAINS an adversary — it does not lose one

With KYC-verified enterprise tenants the **operator** enters the model as a first-class
adversary: us, our staff, anyone who compromises our control plane, and anyone who serves us a
subpoena. Operator-vs-tenant becomes the value proposition, and SEV-SNP/TDX is precisely the
answer to it. That is the new half.

The tenant does **not** leave. KYC establishes *attribution*, not trustworthiness — a malicious
employee inside a verified customer, a compromised enterprise account, and a customer who is
simply hostile all run code we agreed to execute on hardware we own. Attribution changes the
*response* (we know whom to bill, sue, and disconnect); it changes nothing about what the code
can attempt while it runs. An earlier draft of this page said the adversary "is no longer the
tenant"; that is wrong for a product whose entire premise is running other people's code, and
it also contradicted this page's own "worst tenant accepted" rule two sections down.

So both adversaries are in the launch model, and the tenant-vs-tenant isolation controls are
launch-gated rather than deferred to hardening.

Consequence: the isolation story must survive an **insider-threat question from a bank's
security team** *and* a kernel researcher's fuzzer. Those demand different evidence, and the
launch needs both kinds.

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
An attestation verification flow a customer's auditor can actually follow, with **secret
release gated on that verification** (see below). Dedicated hosts, so no two tenants share a
physical machine. Three contracted design partners, invoiced. Upstream Linux, KVM, containerd
and Kata **as transitional adapters behind stable interfaces**, in the sense ADR-0510 already
establishes for upstream Kubernetes and Talos. VAP/CEL for tier admission.

**OUT:** anonymous self-serve, IaaS, functions, multi-region, autoscaling, the port
engine, the owned kernel, our own SDN controller.

### Every upstream component enters with a destination and an exit criterion

"As-is" was the wrong phrasing in the first draft: unqualified, it turns a staged experiment
into a second, permanent, unowned stack contract that competes with the owned-Rust destination.
Each upstream piece therefore enters named as a transient adapter behind a seam, with the thing
that replaces it and the parity test that authorises the swap:

| Upstream in MVP | Seam it sits behind | Owned destination | Deletion criterion |
| --- | --- | --- | --- |
| containerd + Kata | CRI + RuntimeClass | owned runtime | tier-0 parity on the ADR-0338 tier contract |
| KVM | VMM interface Cloud Hypervisor already occupies | owned VMM | boot + attest a confidential guest at parity overhead |
| Linux (guest kernel) | guest ABI | Asterinas track | full ABI coverage for tier-0 syscall surface |

This is why the owned kernel is **out of this MVP rather than absent from the plan** — it has a
track, not an exemption. Anything that cannot name its seam, its destination and its parity test
does not enter the MVP at all.

The MVP tests exactly one thing: **will an enterprise pay for attested confidentiality,
and can their auditor verify it.** Everything else waits on that answer.

### Attestation must gate secret release, or the MVP does not test its own claim

A quote the auditor can read is a *report*. The promised property — the operator cannot read
tenant memory — needs a *control*. If the operator provisions tenant secrets to a guest whose
measurement was never checked, then the operator can boot an altered guest, receive the
plaintext, and still show the auditor a perfectly valid quote taken from a different one. The
central confidentiality claim would ship untested.

So the MVP ships the binding, not just the report: measurements are **approved by the tenant**
(or by a verifier the tenant designates — not by us), the key-release service **refuses** any
guest whose quote fails verification or whose measurement is unapproved, and secret material
reaches a guest only after that refusal point. This is also the standing repository invariant
`INV-CONFIDENTIAL-COMPUTE` in `specs/hyperscaler-architecture-invariants.json`: attestation of
the enclave is verified *before* secrets are provisioned.

Its acceptance test is the negative one, and it is launch-gating: boot a deliberately altered
guest, and a guest whose quote cannot be verified, and prove each receives **no** secret
material. A launch that cannot demonstrate that has not demonstrated confidentiality.

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
- **The Go→Rust port engine** — real, though its ADR-0637/0638 records are archived and the
  live authority is now ADR-0704, and in any case it does not move
  the "will someone pay" needle. It runs on its own track.
- **Our own SDN controller** — modern datacenter fabrics use BGP/EVPN distributed control.
  A centralised controller is the architecture the industry moved away from, and it is a
  cross-cell coordination point against our own blast-radius doctrine.
- **All three product shapes at once** — it is ONE VM substrate with three packagings, not
  three products. Ship the substrate and one packaging.
- **Exotic side-channel research before launch** — covert-channel and microarchitectural
  research is hardening backlog. The *basic* guarantee it is often confused with — that two
  tenants never share a physical host — is launch-gating and is met by dedicated hosts in the
  MVP scope above. An earlier draft deferred the whole topic on the strength of KYC; that
  reasoning died with the adversary-model correction, since a verified tenant is still an
  adversary.

## Open Questions

- **Admission is stale and webhook-shaped.** ADR-0183 chose Kyverno; ADR-0379 superseded
  it with Kubewarden; ADR-0338 still cites Kyverno in 44 places — and it is the ADR that
  enforces tenant isolation. Both are webhooks: a failure domain in the isolation path,
  where failing open silently stops enforcing. VAP/CEL is GA since Kubernetes 1.30 and the
  cluster we actually pin is `v1.36.1` (`infra/capi/clusters/values.yaml`), so it is available
  everywhere we run; an earlier draft of this page quoted a "1.30–1.38" range that was really an
  Istio chart revision and a BusyBox tag. It evaluates in-process, with `paramKind` for the tier
  map. Rust belongs in the **controller that projects manifest tiers into the cluster**, not in
  the admission hook. Now carried by ADR-0710, which is Proposed pending the workload-boundary
  evidence — so this question is open, not answered.
- **Does ADR-0338's `kata-clh` survive?** The Asterinas kata fork ships Dragonball and
  upstream Kata now defaults to it. Evidence lane pending.
- **Which is the funnel and which is the business** — self-serve trial vs contracted
  accounts. Leaning: trial is the on-ramp, contracts are the revenue.
- **Who owns abuse response**, even at ~1 FTE plus on-call? It is small under KYC, not zero.

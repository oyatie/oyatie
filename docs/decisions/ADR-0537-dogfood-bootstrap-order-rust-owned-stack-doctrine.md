---
id: ADR-0537
title: "Dogfood bootstrap order + Rust-owned stack doctrine — the circular-dependency-free ten-step bring-up, the buck2 tier-dependency lint, the kuberos→cloud-os→cloud-k8s→services→products ladder, and all-CLI retirement"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-09
door: one-way
owner: founder
supersedes: []
superseded_by: []
depends_on: [ADR-0536, ADR-0510, ADR-0520]
amends: [ADR-0520]
related: [ADR-0131, ADR-0132, ADR-0243, ADR-0328, ADR-0341, ADR-0348, ADR-0393, ADR-0476, ADR-0510, ADR-0513, ADR-0515, ADR-0516, ADR-0517, ADR-0518, ADR-0519, ADR-0520, ADR-0521, ADR-0522, ADR-0523, ADR-0524, ADR-0525, ADR-0526, ADR-0527, ADR-0528, ADR-0529, ADR-0530, ADR-0531, ADR-0532, ADR-0533, ADR-0534, ADR-0535, ADR-0536]
related_specs:
  - /specs/masterplan.json
  - /specs/master-plan-sequencing.json
milestone: W0
---

# ADR-0537: Dogfood bootstrap order + Rust-owned stack doctrine

## Status

**Proposed — 2026-06-09 (authored for founder sign-off; door: one-way).**

Companion to ADR-0536 (the substrate decision matrix): ADR-0536 decides WHAT each substrate domain
is; this ADR decides the ORDER the substrates come up in, the lint that keeps that order honest, the
owned-stack ladder beneath them, and the operational doctrine (all-CLI retirement, K8s-native
operation) that binds the whole estate. ADR-0328 remains the canonical sequence authority.

## Context

The substrate has circular dependencies at first boot: oya-data encrypts with KMS keys while a naive
KMS would store its state in oya-data; DNS names the control plane while the control plane would
configure DNS; the CAS stores the policy bundles and zone snapshots that other services need before
the CAS's own dependencies exist; audit wants messaging while messaging wants audit. Hyperscaler
precedent resolves first-boot circularity with explicit dependency tiers and named recursion breaks
(AWS static-stability doctrine in the Builders' Library: the recovery path's dependency graph is a
DAG ordered by tier; Google's production layering practice concurs). Separately, the founder
directives of 2026-06-09 bind two estate-wide doctrines that need a durable decision record: the
Rust-owned stack ladder from kernel to products (extending the ADR-0520 owned-substrate doctrine
downward to the kernel), and the retirement of ALL operator CLIs in favor of cloud-ci gates for
authority and console + API for operations (ADR-0536 D-4).

## Decision

### §1 The circular-dependency-free bring-up (step 0 ceremony + steps 1–10)

- **Step 0 — Root ceremony (offline, witnessed, recorded).** Generate the root CA, the KMS domain
  root material, the sealed FIDO2 break-glass credential set (ADR-0536 D-1), and the hand-signed DNS
  seed snapshot (ADR-0536 D-9). This is the only manual step in the estate's life.
- **Step 1 — KMS unseal.** cloud-kms boots on its OWN local Raft quorum — explicitly NOT oya-data.
  Recursion break #1: oya-data encrypts with KMS keys, so the KMS cannot keep its state in oya-data;
  the crypto root carries zero dependency on the data substrate.
- **Step 2 — Secrets + workload identity.** SPIFFE certificates are issued at pod admission;
  fetch-fail = deploy-fail. A pod that cannot obtain its identity does not start — there is no
  identityless degraded mode.
- **Step 3 — IdP.** The identity provider (ADR-0536 D-1) comes up with the primordial operator
  domain seeded from the step-0 ceremony.
- **Step 4 — Embedded Cedar PDP.** Signed content-addressed policy bundles distribute to every
  service's in-process PDP (ADR-0536 D-2); every subsequent service boots with authorization local.
- **Step 5 — Network/DNS from the hand-signed seed snapshot.** The DNS data plane serves the step-0
  seed before its own control plane exists (ADR-0536 D-9) — static stability by construction.
- **Step 6 — Persistence.** oya-data (ADR-0536 D-10) comes up. Recursion break #2: the bootstrap
  metadata that locates oya-data's own ranges lives in a separate single-Raft bootstrap metastore,
  not in oya-data itself.
- **Step 7 — CAS.** Object storage (ADR-0536 D-11) boots from a static-config seed. Recursion
  break #3: the CAS stores policy bundles, zone snapshots, and audit anchors for everyone else, but
  discovers itself from static configuration, never from systems that depend on it.
- **Step 8 — Messaging.** The Pulsar-backed substrate behind the thin owned client (ADR-0536 D-13).
- **Step 9 — Audit.** With messaging and CAS WORM available, the audit pipeline (ADR-0536 D-16)
  anchors its signed digest chain from the first admin event onward.
- **Step 10 — Commercial.** Metering/billing (ADR-0536 D-14) with internal chargeback from day one:
  the platform meters itself and internal teams are tenant #1 — the dogfood loop closes.

### §2 The buck2 tier-dependency lint

Every service declares its `bootstrap_tier` (its §1 step number). A buck2 lint rule refuses any
Tier-N service that links a live client of a Tier>N service: at runtime a service may depend only on
tiers at or below its own. Build-time/codegen dependencies and test-only dependencies are exempt.
The lint ships born-advisory and promotes shadow → warn → enforce per ADR-0536 D-7. Precedent: the
AWS dependency-tier discipline (static stability — the recovery path is a tier-ordered DAG) and
Google's production layering.

### §3 The Rust-owned stack ladder (amends ADR-0520)

The owned-substrate doctrine of ADR-0520 extends downward to the kernel. The ladder is:
**kuberos kernel (`cloud/cloud-kernel`) → Talos-class cloud-os (`cloud/cloud-os`) → bespoke Rust
cloud-k8s substrate (`cloud/cloud-k8s`) → Rust cloud services (`cloud/<service>`) → Rust oyatie
products (`oya/<service>`)**. Upstream Kubernetes, containerd, and Talos serve as ADR-0510
transitional implementations behind stable interfaces; every rung is cutover-gated per ADR-0510
trigger discipline, so no rung blocks delivery above it. ADR-0520 remains in force; this section
names the rungs beneath the substrates it already governs.

### §4 All-CLI retirement + cloud-native K8s-native operation (binding doctrine)

Per founder directives 2026-06-09: ALL operator CLIs are retired. Authority lives in cloud-ci gates;
operations live in the console (ADR-0536 D-4) + API. Operation of the estate is K8s-native — CRDs +
operators + reconcilers + GitOps, zero imperative ops. Any capability that would have been a CLI
verb is authored as a CRD + reconciler or as a console/API action instead. Break-glass is the sealed
FIDO2 ceremony path of ADR-0536 D-1, not a CLI.

## Drivers

- First-boot circularity is real and already visible in the decided matrix (KMS ↔ data, DNS ↔
  control plane, CAS ↔ everything, audit ↔ messaging); named recursion breaks are the only
  alternative to undocumented manual bring-up steps.
- AWS static-stability doctrine: recovery and bring-up paths only work when the dependency graph is
  a tier-ordered DAG, and only stay that way when a machine checks it (§2).
- Founder directives 2026-06-09: whole stack owned in Rust down to the kernel; all-CLI retirement;
  cloud-native K8s-native operation; dogfood with internal chargeback.

## Alternatives considered

- **Boot KMS on oya-data** — rejected: circular (oya-data encrypts with KMS keys); the crypto root
  carries its own local Raft quorum instead.
- **Bring identity up later and bootstrap with shared secrets** — rejected: identityless pods are an
  un-auditable bring-up surface; SPIFFE-at-admission with fetch-fail = deploy-fail is step 2.
- **Big-bang bring-up with manual sequencing runbooks** — rejected: undocumented operator knowledge
  is the anti-pattern the tier lint exists to remove; the order is data (tier numbers) checked in CI.
- **Keep a minimal break-glass operator CLI** — rejected: break-glass is the sealed FIDO2 ceremony
  path (ADR-0536 D-1); a retained CLI re-opens the imperative-ops surface the doctrine closes.
- **Tier checking by convention/review only** — rejected: ADR-0530 enforced-excellence posture;
  conventions drift, lints do not.

## Open questions (founder decision points)

Carried jointly with ADR-0536 (same numbering): **OQ-1** cell-boundary GTM; **OQ-2** audit retention
posture; **OQ-3** v1 pricing dimensions; **OQ-4** W5 numeric cutover triggers for oya-data;
**OQ-5** HSM procurement; **OQ-6** Firecracker adopt-vs-reimplement confirmation; **OQ-7**
Policy-Zones logging-vs-enforce for messenger/mail at KR launch. None blocks bring-up sequencing:
steps degrade to their ADR-0510 transitional posture until ruled.

## Consequences

On founder sign-off (door: one-way) the ten-step order becomes the sequencing skeleton for the
FD-001 + cloud-substrate lanes under ADR-0328 batch discipline, and the tier numbers become data
that the §2 lint checks on every change. ADR-0520 is amended (not superseded): its
transitional-behind-interface doctrine now explicitly spans kernel → OS → K8s substrate → services →
products. The all-CLI retirement (§4) is estate-wide and one-way; together with ADR-0536 D-4 it
fixes the operator surface as console + API + GitOps. Step 10's internal chargeback makes the
platform its own first customer, so the D-14 billing pipeline accrues production evidence before any
external tenant onboards. The §2 lint lands born-advisory; its promotion follows the ADR-0536 D-7
shadow → warn → enforce ladder, sequenced under ADR-0328.

## Governed surfaces added by trust/root/IaC port

The cloud trust/root/IaC drift port (retired external agent harness Kanban t_3e97188f / t_8360046d) adds the following
governed surfaces under this ADR's step-0 root ceremony and Rust-owned cloud-os ladder:

- `os/core/trustd-domain/src/persistence.rs` — trustd sealed-state
  persistence with full-payload authenticated state and monotonic anti-replay checkpointing.
- `docs/runbooks/cloud/OWNERS` — ownership marker for the cloud runbook subtree that carries the
  step-0 root ceremony operator-facing draft.
- `docs/runbooks/cloud/root-of-trust-ceremony.md` — draft runbook for redacted offline
  root-of-trust ceremony evidence and custody posture checks.
- `specs/root-of-trust-ceremony-contract.json` — machine-readable root-of-trust ceremony evidence
  packet contract for step-0 metadata, with root secret values explicitly excluded.

---
*Proposed 2026-06-09 (authored for founder sign-off; door:one-way). Companion: ADR-0536 (substrate
decision matrix). Amends ADR-0520 (ladder extended to the kernel). Sequence authority: ADR-0328.
Founder directives 2026-06-09: Rust-owned stack, all-CLI retirement, K8s-native operation.*

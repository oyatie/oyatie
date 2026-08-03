---
doc_class: Decision-Packet
doc_status: published
date: 2026-08-03
owner: council-architecture
subject_adrs: [ADR-0044, ADR-0148, ADR-0157, ADR-0179, ADR-0182, ADR-0184]
ratifies: none
measured_against: origin/dev@d11567a1a
---

# Storage-HA and service-mesh ADR corrections (Lane C3 + C4)

ROLE: Lane C3 and C4 deliverable. C3 drafts the amendment that ADR-0184's Postgres HA
substrate needs before Lane D2 deploys anything. C4 was asked to record that ADR-0044 is
Proposed so that nothing commits us to rented mesh infra by citation drift; **that premise is
refuted below, with evidence** — the mesh implementation is committed by three *Accepted* ADRs,
and ADR-0044's status is irrelevant to the commitment.

This packet **ratifies nothing**. No ADR `status` field is changed, no `amends` / `amended_by`
edge is written, no gate is created. Every ADR named here keeps the status it has on
`origin/dev`. Promotion of §1.4 to an ADR is the founder's call (§4).

Deliberately **not** an `ADR-*.md` file. `decision_md_file_names`
(`ci/facade/cross-artifact-agreement/tests/cross_artifact_agreement.rs:2838-2845`) selects
`docs/decisions/` entries by `name.starts_with("ADR-") && name.ends_with(".md")`, and
`adr_index_projection_parity_is_advisory_clean_on_live_tree` (same file, `:3147`) ratchets
`adr_index_projection_stale` against a born-empty baseline. An `ADR-`-prefixed filename here
would enter `source_adr_ids`, miss `record_ids`, and turn that required-gate test red without
regenerating `docs/ADR-INDEX.md` + `docs/machine-readable/decisions.json`. The generated ADR
index is untouched by this packet, by construction.

---

## 0. Method

Every claim below is a `git show origin/dev:<path>` / `git grep -n origin/dev` read against
`origin/dev@d11567a1a`. The canonical checkout's working tree is a stale preserve branch and was
not used as evidence.

---

## 1. C3 — ADR-0184 pins Citus + Patroni; the only live Postgres config is CNPG

### 1.1 The contradiction, at path:line

**ADR-0184 (`status: Accepted`, 2026-05-18) — `docs/decisions/ADR-0184-storage-tier-layering.md`:**

| line | text |
|---:|---|
| `:40` | `- Citus 14.0 for logical sharding by tenant where multi-tenant scale demands it` |
| `:41` | `- Connection pooling via pgcat (per ADR-0179-postgres-connection-pooling-pgcat).` |
| `:43` | `- Patroni 4.x for HA / leader election.` |
| `:50` | `- HA failover: streaming replica promotes to primary via Patroni if Tier 1 primary fails.` |
| `:145` | ``- `postgres/` — Postgres 18.4 + Patroni 4.x + pgcat.`` |
| `:171` | **Patroni 4.x** \| KEEP (Apache 2.0; broad community) \| THE standard Postgres HA leader-election operator. \| None planned. |

**The live configuration — `infra/arc/runner-scale-set-arm64-values.yaml`:**

| line | text |
|---:|---|
| `:61` | `# k8s Secret -> pod env -> gate; the Secret is projected from the CNPG cluster` |
| `:62` | `# in oya-data, never copied into git.` |
| `:65` | `value: oya-pg-rw.oya-data.svc.cluster.local` |
| `:68` | `secretKeyRef: { name: oya-pg-superuser, key: username }` |
| `:71` | `secretKeyRef: { name: oya-pg-superuser, key: password }` |

`oya-pg-rw` is CloudNativePG's `<cluster>-rw` read-write Service naming convention. Patroni
publishes no such Service. The hostname alone commits the runner to a CNPG-shaped cluster.

**The required workflow agrees — `.github/workflows/oya-ci-required.yml`:**

| line | text |
|---:|---|
| `:1007` | ``# Flipping it without first rewiring `services:` to the CNPG cluster in oya-data`` |
| `:1117` | ``# Flipping it without first rewiring `services:` to the CNPG cluster in oya-data`` |

Those two comments sit on `gate-live-postgres-adapters` (`:1003`) and
`gate-live-postgres-facades` (`:1113`), both still `runs-on: ubuntu-latest` (`:1009`, `:1119`) —
the rented-runner state Lane B3 exists to remove.

So the contradiction is not a doc nit. **An Accepted ADR names Patroni; the runner values, the
required workflow, and Lane B3's exit condition all name CNPG.** Deploying D2 first would make
the deployment the de-facto authority and leave the Accepted ADR describing a substrate that
does not exist — which is exactly the "amend before, not after" instruction.

### 1.2 The fact that reframes the amendment: neither substrate exists

```
$ git grep -rln 'postgresql.cnpg.io' origin/dev | wc -l
0
$ git grep -rln 'patroni' origin/dev -- infra/ | wc -l
0
$ git grep -rln 'oya-data' origin/dev -- infra/
origin/dev:infra/arc/runner-scale-set-arm64-values.yaml
```

There is **no CNPG `Cluster` manifest anywhere in the repo, no Patroni manifest under `infra/`,
and exactly one file that mentions the `oya-data` namespace** — the ARC values that consume it.
ADR-0184's entire Tier 1 is unimplemented. `oya-pg-rw.oya-data.svc.cluster.local` and the
`oya-pg-superuser` Secret are dangling references to a cluster nothing in the repo creates.

This bounds the amendment correctly. The live requirement is **one Postgres for two CI gates**,
not a production storage tier. An amendment that re-architects Tier 1 would be inventing scope;
an amendment that unblocks D2 and leaves production sharding open is the honest size.

### 1.3 Why the ADR picked Patroni, and why that reasoning no longer decides it

ADR-0184`:171` justifies Patroni as "THE standard Postgres HA leader-election operator". That
was a substrate-maturity argument, not a workload argument, and it is now contested on its own
terms: CloudNativePG entered the CNCF Sandbox in 2025 and is the operator the repo's own IaC
module library already names —

- `docs/decisions/ADR-0339-shared-iac-module-library.md:567` —
  `pg-cluster` → "Self-managed PostgreSQL cluster (CloudNativePG operator)"
- `specs/iac-module-library.json:216` —
  `"purpose": "Self-hosted PostgreSQL cluster (CloudNativePG operator)"`

ADR-0339 is `status: Proposed`, so it is not authority — but it means CNPG is not an unreviewed
import from the ARC work. Two independent surfaces reached for it, and neither reconciled with
ADR-0184.

CNPG is Apache-2.0, which satisfies ADR-0184`:130`'s all-permissive-license invariant as well as
Patroni's Apache-2.0 does. No license argument distinguishes them.

### 1.4 DRAFT AMENDMENT — for promotion to `ADR-0635-amendment-…` on ratification

> **Promotion note (mechanical).** `max(ADR on origin/dev)` is **ADR-0634**; re-derive against
> every in-flight branch before allocating, per the off-by-one collision recorded in
> `auth005-wave3-launch-plan-2026-06-24.md`. Unlike this packet, an `ADR-`-prefixed file **does**
> enter `source_adr_ids`: `decision_md_file_names` drops an `-amendment-` file only when a
> non-amendment file shares its 8-char id, and no `ADR-0635-*.md` base exists. So promoting §1.4
> **requires regenerating `docs/ADR-INDEX.md` + `docs/machine-readable/decisions.json` through
> `oya doc adr-index`** (`marketplace/facade/dev-cli/src/commands/doc/adr_index.rs`) in the same
> PR, or `adr_index_projection_parity` goes red. Do not hand-edit either face.

> **Amends:** ADR-0184 (Tier 1 only). **Status on landing: Proposed.**
>
> **A1 — HA substrate.** Replace "Patroni 4.x for HA / leader election" (ADR-0184`:43`, `:50`,
> `:145`, `:171`) with **CloudNativePG (Apache-2.0)** as the canonical PostgreSQL HA and
> leader-election operator. Rationale: it is the operator the live ARC configuration and both
> required Postgres gates already assume, it is the operator ADR-0339 / `iac-module-library.json`
> already name, and it is license-equivalent to Patroni under ADR-0184`:130`. Patroni is recorded
> as **rejected-in-place**, not deleted: ADR-0184's Patroni rows stay in the ledger per
> `docs/decisions/README.md` ("do not delete or rewrite them to make the ledger look clean").
>
> **A2 — Scope of A1.** A1 governs the **single-instance and streaming-replica** Tier-1/Tier-2
> shapes (ADR-0184 §Tier 1, §Tier 2). It does **not** decide the sharded shape — see A3.
>
> **A3 — Citus is severed from A1, not decided by it.** CNPG manages one PostgreSQL cluster; it
> does not orchestrate a Citus coordinator/worker topology. ADR-0184`:40` (Citus 14.0, opt-in
> per-µservice) and `:170` (Citus KEEP) are **unchanged and unreached** by this amendment. No
> µservice runs Citus today. The sharded-tenant substrate is deferred to the first µservice that
> demonstrates the scale that triggers ADR-0184`:40`'s "where multi-tenant scale demands it".
> Recording this explicitly is the point: an amendment that silently dropped Citus would be a
> second, unratified decision riding along.
>
> **A4 — Pooling is a dependent decision, flagged not ruled.** ADR-0179 (`status: Accepted`,
> 2026-05-18) pins **pgcat**; CNPG ships a first-class `Pooler` CRD backed by PgBouncer. Adopting
> CNPG does not by itself retire pgcat — CNPG can front a cluster with an external pooler. But
> the two are alternatives for the same seam, and D2 will be forced to pick one at deploy time.
> **This amendment does not amend ADR-0179.** It records that A1 makes ADR-0179 a live question,
> and that D2 must either deploy pgcat in front of the CNPG cluster (ADR-0179 stands, no
> amendment needed) or raise a separate ADR-0179 amendment. It must not resolve it by
> deployment.
>
> **A5 — What A1 unblocks, precisely.** One CNPG `Cluster` in namespace `oya-data` publishing
> `oya-pg-rw` and a `oya-pg-superuser` Secret, satisfying the two dangling references at
> `infra/arc/runner-scale-set-arm64-values.yaml:65,68,71`. That is the whole of D2's requirement.
> Production Tier-1 topology, per-cell placement (ADR-0009), backup (ADR-0197, still Accepted and
> unimplemented — Lane D1) and residency are **out of scope** and remain governed by ADR-0184 as
> written.
>
> **A6 — Non-goal.** This amendment does not make CNPG the owned-Rust destination. Per the
> founder's owned-stack directive and ADR-0510, CNPG is a transitional operator behind the
> storage port seam, on the same footing Patroni held. It buys D2; it does not settle the
> destination.

### 1.5 Sequencing

The spine's "amend before deploying D2, not after" holds and is now sharper: because **zero**
Postgres manifests exist, D2 is a greenfield deploy with no migration cost either way. The
amendment is cheap *now* and expensive after a cluster exists and gates depend on it.

---

## 2. C4 — the premise is REFUTED; the correction runs the other way

### 2.1 What is true

`docs/decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md:3` — `status: proposed`.
The body header agrees: `> **Status:** Proposed`. Citing ADR-0044 as ratified doctrine was
wrong, and the general rule the spine drew from it — check `status`, `superseded_by`, and newer
contradictions before citing any ADR — is correct and worth keeping.

### 2.2 What is false

> "The gateway TIER (ADR-0157/0182) IS ratified; the IMPLEMENTATION never was, so nothing
> commits us to rented mesh infra."

The implementation **was** ratified — three times, by three **Accepted** ADRs, none of which
depends on ADR-0044:

| ADR | status | line | what it pins |
|---|---|---:|---|
| ADR-0148 | **Accepted** 2026-05-18 | `:3`, title | Cilium 1.19.x (pin 1.19.4) L3/L4 + **Istio Ambient** L7 (ztunnel + waypoint), "layered globally; zero overlap" |
| ADR-0157 | **Accepted** 2026-05-18 | `:11` | `architectural_authority: ADR-0182 (gateway-vs-mesh separation principle; **this ADR picks the implementation**)` |
| ADR-0157 | " | `:79`, `:80`, `:82`, `:155` | `Data plane: Envoy 1.30 LTS` · `Control plane: Envoy Gateway 1.1` · `WAF: Coraza` · `Helm chart iac/helm/api-gateway/ ships with Envoy Gateway 1.1 + Coraza + ratelimit-redis sidecar` |
| ADR-0182 | **Accepted** 2026-05-18 | `:59-61` | `The canonical north-south substrate is **Envoy Gateway 1.8.0**` |
| ADR-0182 | " | `:157-159`, `:164` | `KEEP` rulings on Envoy Gateway, the Envoy data plane and Coraza; an explicit **"Why no in-house gateway"** section |

ADR-0157`:11` is decisive on its own: the Accepted gateway ADR states in its own frontmatter that
it *picks the implementation*. The tier/implementation split the spine relied on does not exist
in the corpus.

**Consequence:** de-citing ADR-0044 changes nothing about what the repo is committed to. Anyone
who reads only "ADR-0044 is Proposed" and concludes the mesh is uncommitted has inherited the
inverse of the original error. **Istio Ambient, Cilium, Envoy Gateway and Coraza are committed by
Accepted ADRs, and remain committed after ADR-0044 is de-cited.** Decommitting requires amending
ADR-0148, ADR-0157 and ADR-0182 — a founder decision that nobody has been asked for, and one this
packet does not propose.

### 2.3 Relationship to the C2 packet (`PROPOSED-CITATION-TRIAGE-2026-08-02.md`)

C2 already rules ADR-0044 **DE-CITE**, repointing to ADR-0148 + ADR-0182, and correctly observes
that ADR-0148 "REWRITES the prior framing" while declaring no `supersedes` edge. That ruling is
sound and this packet does not disturb it. §2.2 is the missing half: the ruling is about *which
ADR to cite*, and says nothing about *what we are committed to*. Both halves have to travel
together or the DE-CITE row reads as a decommitment. **The two packets are independent files and
may land in either order.**

### 2.4 The real defect, which nobody has recorded

ADR-0148 (Accepted) rewrites ADR-0044 (Proposed) on the same subject and declares
`supersedes: []` (`ADR-0148…:7`); ADR-0044 declares `Superseded-by: -`. Two live ADRs, one
subject, no edge between them. The generic `supersession_half_edge` invariant
(`ci/facade/cross-artifact-agreement/src/lib.rs:649-675`) reads only `supersedes` /
`superseded_by`, so a *missing* pair — as opposed to a one-sided pair — is invisible to it. This
is the mechanism that let ADR-0044 sit "Proposed looking live" for months. Recording it; not
fixing it here (it needs the status-lifecycle blocker C2 §3 B1 documents).

---

## 3. Adjacent defects found while verifying, ruled on by nobody

Reported, not fixed. Each is a distinct lane's work.

1. **Two Accepted ADRs pin two different Envoy Gateway versions.** ADR-0157`:80` pins
   **Envoy Gateway 1.1**; ADR-0182`:59-61`, `:146`, `:157` pin **Envoy Gateway 1.8.0**. Both are
   `status: Accepted`, both dated 2026-05-18, and ADR-0157`:11` names ADR-0182 as its
   `architectural_authority`. A `conflicting_accepted_pair` on the substrate D3 would deploy.
2. **Two Accepted ADRs disagree on the rate-limit cache, on license grounds.** ADR-0157`:71`,
   `:81`, `:155` put the Envoy rate-limit service on **Redis**; ADR-0184 Tier 3 assigns exactly
   that workload ("Rate-limit counters … backing Envoy Gateway's rate limit") to **Valkey 8.1**
   and *rejects Redis 7.4+ by name* on RSALv2/SSPLv1 grounds (ADR-0184 §Tier 3, `:130`). Same
   date, same council, opposite substrates — and the losing side is the one the license doctrine
   forbids.
3. **Dead ADR path anchors in ADR-0253's `related:`** — `ADR-0044-service-mesh-and-mtls.md` and
   `ADR-0149-api-gateway-vs-service-mesh-separation.md` resolve to nothing; the live files are
   `ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md` and `ADR-0149` is
   *idempotency-keys-canonical*, an unrelated decision. ADR-0339`:20` cites
   `ADR-0211-in-house-tech-stack-preference.md`; the live file is
   `ADR-0211-in-house-tech-stack-policy.md`. Lane C1 / E4 class.
4. **`amends` / `amended_by` reciprocity is enforced only by hand-written per-pair tests**
   (`cross_artifact_agreement.rs:375`, `:407`, `:470`; `product_protocol_policy.rs:1022-1066`).
   ADR-0253 carries `amended_by: [ADR-0565]` but not `ADR-0354`, whose frontmatter says
   `amends: ADR-0253` — a live half-edge no gate can see. Lane E class.

---

## 4. What the founder is being asked to decide

1. **C3 / A1** — promote §1.4 to `ADR-0635-amendment-…` (Proposed → Accepted): CloudNativePG
   replaces Patroni as the Tier-1 HA operator, scoped to the single-instance and streaming-replica
   shapes. **This is the only item that blocks D2.**
2. **C3 / A4** — confirm that D2 deploys pgcat in front of the CNPG cluster (ADR-0179 stands), or
   direct a separate ADR-0179 amendment. Either answer is fine; deploying without one is not.
3. **C4** — accept that the mesh/gateway implementation is committed by Accepted ADRs and that no
   decommitment is proposed. If the owned-stack directive means it *should* be decommitted, that
   is an amendment to ADR-0148 / ADR-0157 / ADR-0182 and needs its own lane.
4. **§3.1 and §3.2** — route the two `conflicting_accepted_pair` findings to a lane. §3.2 in
   particular puts an Accepted ADR in conflict with the repo's own permissive-license doctrine.

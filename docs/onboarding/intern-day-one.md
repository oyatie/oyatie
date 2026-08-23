---
doc_class: Onboarding-Runbook
doc_id: ONB-INTERN-DAY-ONE
status: Published
date: 2026-05-20
owner_team: council-engineering + axis-devrel + axis-identity + axis-tenancy + axis-policy-engine + axis-audit-chain + axis-cell
audience:
  - new-hire-engineer
  - intern
  - external-contributor
  - new-agent-persona
companion_doc: /docs/architecture/keystone-bundle-reading-order.md
related_adrs:
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0246-policy-engine-substrate-promotion
  - ADR-0247-self-hosting-self-modification-doctrine
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0250-build-ahead-of-certification-doctrine
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0252-time-coordination-distributed-consistency
  - ADR-0253-network-topology-edge-service-mesh
  - ADR-0254-deployment-model-spectrum
  - ADR-0255-intelligence-as-two-layer-ai-substrate
related_specs:
  - /specs/platform-architecture.json
  - /specs/tenant-model.json
  - /specs/cedar-fragment-schema.json
  - /specs/microservices/identity.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/cell.json
keystone_bundle: 2026-05-20-foundational-doctrine
---

# Intern Day-One Runbook

> A step-by-step runbook for the first eight hours of an intern (or
> external contributor) at oyatie. By the end of day one you will
> have a working local cluster running the five bootstrap µservices,
> the `oyatie` tenant created via bootstrap migration, a sandbox
> tenant under your engineer ID, a first Cedar-gated call exercised,
> and a first PR opened against `dev` for multispectrum review.

---

## 1. Welcome

Welcome to oyatie.

You are joining a platform that is built on a 14-ADR foundational
doctrine bundle (the `2026-05-20-foundational-doctrine` keystone
bundle) and a large library of supporting ADRs, specs, PRDs, and
standards. This runbook walks you through your first day of hands-on
work. It assumes you have read Phase 1 of the reading-order doc
(`/docs/architecture/keystone-bundle-reading-order.md` §3) — the
four foundational doctrines (ADR-0242, ADR-0243, ADR-0244,
ADR-0245). If you have not, stop here, read Phase 1, then return.

Your role on day one is **bootstrap engineer**. You are not yet
shipping features. You are getting a working local platform stand-
up, exercising the five bootstrap µservices, and submitting your
first PR. Day one is intentionally shaped to expose you to the
five most load-bearing µservices in the platform — `identity`,
`tenancy`, `policy-engine`, `audit-chain`, and `cell` — because
every later µservice you build will depend on at least one of these.

If anything in this runbook breaks, write the error down. The
"Common errors + recovery" section (§7) covers the top 15 likely
breakages with diagnosis and fix. If your error is not in §7, ask
your mentor and propose adding it to the list.

By the end of the day you will have:

- A local Kubernetes cluster running on your laptop.
- The five bootstrap µservices deployed and healthy.
- The `oyatie` org tenant created via the canonical bootstrap
  migration.
- A personal sandbox tenant (`oyatie.dev.<your-id>`) created and
  inheriting the doctrine.
- A first Cedar-gated call exercised against the policy engine.
- A first PR open against `dev` for multispectrum review.

---

## 2. What you will build day-one

You are deploying a working local copy of the platform's bootstrap
substrate. This is not a full deploy of the entire platform — the
full platform has dozens of µservices — but it is enough to
exercise every load-bearing doctrine and to give you a sense of how
real engineering work happens in this codebase.

### 2.1 Scope expectations

In scope for day one:

- Clone the repo.
- Install the Rust toolchain.
- Install `kind` (local Kubernetes) or `k3d`, plus `kubectl` and
  `helm`.
- Deploy the five bootstrap µservices to your local cluster.
- Run the canonical `0001_create_self_tenant.sql` migration.
- Verify reserved-namespace protection works (i.e., that you cannot
  create a tenant named `bad`).
- Create your personal sandbox tenant.
- Exercise a Cedar-gated call (the call goes through the evaluator
  and you see the audit event land).
- Submit a documentation-only PR adding your name to the contributor
  ledger.

Out of scope for day one:

- Building any product surface (Messenger, Mail, Community,
  Workplace Integration).
- Deploying any compliance pack (HIPAA, PCI DSS, etc.).
- Federating with another cluster.
- Touching the deployment-control-plane µservice.
- Shipping any production code change.

Out-of-scope items are day-two-plus work. Do not push past day one
without explicit mentor sign-off.

### 2.2 Tools you will install

- `rustup` and a current Rust toolchain (stable).
- `cargo` (comes with rustup).
- `docker` (or `podman` with `docker` shim).
- `kind` (Kubernetes-in-Docker) — preferred — or `k3d`.
- `kubectl`.
- `helm`.
- `cosign` (for signature verification of policy fragments).
- `jq` and `yq` (for JSON / YAML CLI).
- A code editor with `rust-analyzer` (VS Code or your preferred
  editor).

If your laptop is fresh, expect 60-90 minutes to install everything.

---

## 3. The five bootstrap µservices

The platform's bootstrap substrate is five µservices. They must
come up in this order; each depends on its predecessors.

### 3.1 `identity` (Zitadel deployment)

Path: `/microservices/identity/`

**What it does.** Issues OIDC tokens for every principal on the
platform. Backed by Zitadel as the IdP implementation. Handles
SAML SSO, OIDC, SCIM provisioning, MFA, passkeys, federated
identity. Every other µservice trusts tokens issued by `identity`.

**Why it is first.** Without an identity provider, no other
µservice can authenticate any caller. The chicken-and-egg loop is
broken by a self-signed Tier-1 bootstrap token used to provision
the first `oyatie.platform-ops` admin; from then on every token
is issued by `identity`.

**Day-one expectations.** You will deploy Zitadel with default
config, log in as the bootstrap admin, and issue your first token
for the `oyatie.platform-ops.sre` sub-scope.

### 3.2 `tenancy` (with oyatie tenant migration)

Path: `/microservices/tenancy/`

**What it does.** Owns the tenants table. Every tenant row carries:
`tenant_id`, `audience_type`, `parent_tenant_id`,
`jurisdiction_primary`, `data_residency_allowed`,
`sovereign_cloud_pack`, `finops_cost_center`, `merchant_status`,
`payout_method`, `dsar_response_sla_days`, `audit_streams`,
`locked`. The schema is in `/specs/tenant-model.json`.

**Why it is second.** Every other µservice scopes its work by
`tenant_id`. Without the tenants table, the policy engine cannot
resolve a sub-scope to a tenant, the audit chain cannot pick a
stream, and the cell cannot pin a tenant to a home cell.

**Day-one expectations.** You will run the canonical bootstrap
migration `migrations/0001_create_self_tenant.sql`, which inserts
the `oyatie` row with the locked canonical-row content from
`platform-architecture.json` §`platform.tenancy.canonical_oyatie_tenant_row`.

### 3.3 `policy-engine` (with bootstrap Cedar fragments signed by org root key)

Path: `/microservices/policy-engine/`

**What it does.** Loads, verifies, hot-reloads, and evaluates Cedar
fragments. Every code path that asks a question of the form "may
I?", "should I?", "to whom?", "where?" goes through the evaluator.
Fragments live in five scopes: baseline, pack, overlay, reserved,
tenant. Composition: permits union, forbids override permits,
deny-wins.

**Why it is third.** Once tenants exist, every subsequent decision
about a tenant (where to route, what they may do, which audit
stream to emit on) needs policy evaluation.

**Day-one expectations.** You will load the bootstrap fragment
bundle from `microservices/policy-engine/fragments/bootstrap/`,
verify the Ed25519 signatures with `cosign` against the org root
key, and confirm that the reserved-namespace fragment denies
attempts to register `bad` as a tenant.

### 3.4 `audit-chain` (per-stream provisioning)

Path: `/microservices/audit-chain/`

**What it does.** Owns the append-only, Merkle-sealed audit log.
Every state-changing action emits one or more audit events. Events
are routed to streams by Cedar policy. Streams are per-tenant
plus optional roll-up streams. Sealed checkpoints are signed daily
and posted to a public transparency log per ADR-0028.

**Why it is fourth.** Once tenants and policy exist, every action
must be auditable. Day-one you provision the five canonical streams
for the `oyatie` tenant: `oyatie.root`, `oyatie.foundry`,
`oyatie.security`, `oyatie.finance`, `oyatie.platform-ops`.

**Day-one expectations.** You will run the per-stream provisioning
command, then verify that an event you emit lands in the right
stream by Cedar policy.

### 3.5 `cell` (cell registration)

Cell pattern ownership: `/microservices/tenancy/ARCHITECTURE.md#cell-assignment`, `/microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning`, `/microservices/observability/ARCHITECTURE.md#cell-health`, `/microservices/api-gateway/ARCHITECTURE.md#cell-aware-routing`, and `/microservices/audit-chain/ARCHITECTURE.md#cell-scoped-audit`.

**What it does.** Owns the cell registry. Each cell row carries
its tier, region, certification levels (set of compliance packs it
can host), home-tenant assignments, and DR tier. Cells are the
universal blast-radius primitive per ADR-0248.

**Why it is fifth.** Once a tenant exists, it must be pinned to a
cell. Once a cell exists, the policy engine knows which evaluator
replicas to consult and the audit chain knows which sealed log to
write to.

**Day-one expectations.** You will register one cell in your local
cluster — `local-cell-0` — with tier T1-bootstrap, region
`local-dev`, certifications `[baseline]`, and pin the `oyatie`
tenant plus your sandbox tenant to it.

---

## 4. Step-by-step day-one tasks

The day is structured as ten tasks. Allocate roughly 45-60 minutes
per task. If a task takes longer than 90 minutes, ask your mentor.

### 4.1 Step 1: Clone repo + set up dev environment

**Goal:** A working laptop with the repo, the toolchain, and a
local Kubernetes cluster.

**Procedure:**

1. Clone the repo:
   ```
   git clone git@github.com:oyatie/oyatie.git
   cd oyatie
   git checkout dev
   ```

2. Install Rust:
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   rustup default stable
   rustup component add rustfmt clippy rust-analyzer
   ```

3. Install Docker (or Podman with the Docker shim). Verify:
   ```
   docker info
   ```

4. Install `kind` (preferred for local K8s):
   ```
   # macOS
   brew install kind

   # Linux
   curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-linux-amd64
   chmod +x ./kind && sudo mv ./kind /usr/local/bin/kind
   ```

5. Install `kubectl` and `helm`:
   ```
   # macOS
   brew install kubernetes-cli helm

   # Linux
   curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
   sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
   curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
   ```

6. Install `cosign`, `jq`, `yq`:
   ```
   # macOS
   brew install cosign jq yq

   # Linux
   # Follow your distro's package manager
   ```

7. Create the kind cluster:
   ```
   kind create cluster --name oyatie-dev --config tools/local-dev/kind-config.yaml
   kubectl cluster-info --context kind-oyatie-dev
   ```

**Acceptance:** `kubectl get nodes` shows at least one Ready node.

**Common gotchas:** Docker not running; insufficient laptop
resources (kind needs 4GB free RAM minimum); on macOS Apple Silicon
you may need `--platform linux/arm64` flags.

### 4.2 Step 2: Read Phase 1 of reading-order doc

**Goal:** Foundational doctrine in your head before you touch code.

**Procedure:** Open
`/docs/architecture/keystone-bundle-reading-order.md` §3 and read
the four ADRs in order: ADR-0242, ADR-0243, ADR-0244, ADR-0245.
Spend 30-45 minutes on each. Answer the four Phase-1 self-check
questions in your own words.

**Acceptance:** You can answer (without looking) these four
questions:

- Why does `oyatie` need to be a row in the tenants table?
- Why is Cedar's scope wider than "authorization"?
- What does `acme.marketing.q4` decompose to in terms of tenant
  and sub-scopes, and what depth is it?
- What are the four µservice tiers?

**Common gotchas:** Skipping ahead. Do not start Step 3 until you
can answer the four questions.

### 4.3 Step 3: Run `cargo check` on first µservice

**Goal:** Confirm the toolchain works against the first bootstrap
µservice.

**Procedure:**

1. Navigate to the identity µservice:
   ```
   cd microservices/identity
   ls
   ```

2. Read the README at `microservices/identity/README.md`.

3. Run a check:
   ```
   cargo check --workspace
   ```

4. Run the unit tests:
   ```
   cargo test --workspace
   ```

5. Run the formatter:
   ```
   cargo fmt --all --check
   ```

6. Run the linter:
   ```
   cargo clippy --workspace --all-targets -- -D warnings
   ```

**Acceptance:** All four commands exit zero.

**Common gotchas:** Out-of-date Rust toolchain (`rustup update`);
missing system libraries (`pkg-config`, `libssl-dev`,
`build-essential`); platform-specific deps on Apple Silicon.

### 4.4 Step 4: Walk through Cedar fragment authoring exercise

**Goal:** Familiarity with the Cedar fragment format and the
signing chain.

**Procedure:**

1. Open `microservices/policy-engine/fragments/bootstrap/genesis.cedar`
   in your editor.

2. Read the fragment. Note its three parts: a permit declaration
   (who is allowed to do what), a forbid declaration (the explicit
   default-deny), and a metadata header (id, scope, owner,
   version, depends_on, gate_category).

3. Open `microservices/policy-engine/fragments/baseline/reserved-tenant-namespace.cedar`.
   Read it. Note that it forbids the registration of any tenant
   whose normalised name starts with `oyatie`, `oya`, `oyat`, or
   `oyati`. Note the normalisation chain: NFKC + lowercase +
   diacritic-strip + UTS#39 confusable removal.

4. Verify the signature:
   ```
   cosign verify-blob \
     --key microservices/policy-engine/keys/org-baseline.pub \
     --signature microservices/policy-engine/fragments/baseline/reserved-tenant-namespace.cedar.sig \
     microservices/policy-engine/fragments/baseline/reserved-tenant-namespace.cedar
   ```

5. Walk through `microservices/policy-engine/fragments/README.md`
   to understand the lifecycle states (authored, reviewed, signed,
   published, activated, in-force, sunset, tombstoned).

**Acceptance:** You can articulate (out loud, to a rubber duck) what
the reserved-namespace fragment does, why it has a default-deny
clause, and what the signing chain proves.

**Common gotchas:** Cosign signature verification failing because
the key is wrong (use the right pub key per scope); reading the
fragment without reading the metadata header.

### 4.5 Step 5: Deploy tenancy µservice to local kind cluster

**Goal:** A running `tenancy` µservice in the kind cluster, with
its database provisioned.

**Procedure:**

1. Pull the canonical Helm chart for `tenancy`:
   ```
   ls microservices/tenancy/charts/
   ```

2. Apply the chart with a local-dev values override:
   ```
   helm install tenancy microservices/tenancy/charts/tenancy \
     -f microservices/tenancy/charts/tenancy/values.local-dev.yaml \
     --namespace tenancy --create-namespace
   ```

3. Wait for the deployment to roll out:
   ```
   kubectl rollout status deployment/tenancy --namespace tenancy --timeout=5m
   ```

4. Port-forward the gRPC endpoint:
   ```
   kubectl port-forward svc/tenancy 5000:5000 --namespace tenancy &
   ```

5. Hit the health endpoint:
   ```
   curl http://localhost:5000/health
   ```

**Acceptance:** `/health` returns `200 OK` with a JSON body listing
the µservice version, the schema-revision, and the bootstrap
state.

**Common gotchas:** Image pull errors (load images into the kind
cluster via `kind load docker-image`); persistent volume claim
failing on macOS (check `kind-config.yaml` mount paths); deployment
stuck in `CrashLoopBackOff` because the database container is not
ready (check `kubectl logs`).

### 4.6 Step 6: Create the `oyatie` tenant via bootstrap migration

**Goal:** The canonical `oyatie` tenant row in the tenants table.

**Procedure:**

1. Locate the canonical migration:
   ```
   ls microservices/tenancy/migrations/
   ```

   You should see `0001_create_self_tenant.sql`.

2. Open the migration and read it. Note that it inserts a single
   row with `tenant_id = 'oyatie'`, `audience_type = 'PLATFORM_OWNER'`,
   `parent_tenant_id = null`, `jurisdiction_primary = 'US-DE'`,
   `data_residency_allowed = ['US', 'EU', 'KR']`, `merchant_status =
   'platform_facilitator'`, `payout_method = 'internal'`, and
   `locked = true`. The `locked = true` flag means this row cannot
   be updated except by the bootstrap migration suite.

3. Run the migration:
   ```
   kubectl exec -n tenancy deployment/tenancy -- \
     /app/bin/tenancy-migrate up --to 0001_create_self_tenant
   ```

4. Verify:
   ```
   grpcurl -plaintext localhost:5000 \
     tenancy.v1.TenantService/GetTenant \
     -d '{"tenant_id":"oyatie"}'
   ```

**Acceptance:** The grpcurl call returns the canonical oyatie row.

**Common gotchas:** Migration partially-applied (clear with a
fresh database volume); grpcurl not installed (install via brew
or pkg manager); endpoint requires a token (set
`-H "authorization: Bearer ${BOOTSTRAP_TOKEN}"`).

### 4.7 Step 7: Verify reserved namespace check works

**Goal:** Confirm that the Cedar reserved-namespace fragment
actually denies attempts to register a reserved name.

**Procedure:**

1. Deploy the `policy-engine` µservice (same procedure as `tenancy`):
   ```
   helm install policy-engine microservices/policy-engine/charts/policy-engine \
     -f microservices/policy-engine/charts/policy-engine/values.local-dev.yaml \
     --namespace policy-engine --create-namespace
   kubectl rollout status deployment/policy-engine --namespace policy-engine --timeout=5m
   ```

2. Verify the policy-engine has loaded the bootstrap fragments:
   ```
   kubectl logs deployment/policy-engine --namespace policy-engine | grep "fragments_loaded"
   ```

3. Configure `tenancy` to consult `policy-engine` for the
   reserved-namespace gate (per
   `microservices/tenancy/charts/tenancy/values.local-dev.yaml`,
   the `policy_engine_endpoint` should point at the in-cluster
   service).

4. Attempt to register a reserved tenant:
   ```
   grpcurl -plaintext localhost:5000 \
     tenancy.v1.TenantService/CreateTenant \
     -d '{"tenant_id":"bad","audience_type":"B2B_TENANT"}'
   ```

5. Confirm the call fails with a `PERMISSION_DENIED` status and
   the audit log carries a `reserved-namespace-deny` event.

6. Repeat with `oyatie-shadow`, `oyat-test`, `Oyatié` (case +
   accent variant), and `oуatie` (Cyrillic confusable). All should
   be denied.

**Acceptance:** All five reserved-namespace attempts return
`PERMISSION_DENIED`; all five produce an audit event with the right
fragment id (`baseline/reserved-tenant-namespace`).

**Common gotchas:** Policy-engine not reachable from tenancy (check
service DNS); fragments not loaded (check the cosign verification
log); normalisation not applied (check that the call site
normalises before policy evaluation).

### 4.8 Step 8: Create test sandbox tenant `oyatie.dev.intern-<id>`

**Goal:** A personal sandbox tenant scoped under the canonical
`oyatie.dev.*` pattern.

**Procedure:**

1. Pick your sandbox id. Use your GitHub handle, lowercase, no
   special characters. Example: `oyatie.dev.intern-alice`.

2. Create the sandbox tenant:
   ```
   grpcurl -plaintext localhost:5000 \
     tenancy.v1.TenantService/CreateTenant \
     -d '{
       "tenant_id":"oyatie.dev.intern-alice",
       "audience_type":"SANDBOX",
       "parent_tenant_id":"oyatie",
       "jurisdiction_primary":"US-DE"
     }'
   ```

3. Verify the sandbox inherits jurisdiction, capability flags,
   and Cedar policy from the parent:
   ```
   grpcurl -plaintext localhost:5000 \
     tenancy.v1.TenantService/GetTenant \
     -d '{"tenant_id":"oyatie.dev.intern-alice","resolve_inherited":true}'
   ```

4. Note the `lifetime: 24h-inactivity-teardown` flag — this sandbox
   is auto-torn-down after 24 hours of inactivity.

**Acceptance:** The sandbox tenant is created; inheritance resolves
the parent's `jurisdiction_primary` and `cedar_policy`.

**Common gotchas:** Parent tenant not yet created (Step 6 must
complete first); naming pattern mismatch (`oyatie.dev.<id>` requires
exactly one sub-scope after `dev`); audience_type not `SANDBOX`.

### 4.9 Step 9: Make first Cedar-gated call

**Goal:** Exercise the end-to-end policy + audit flow under your
sandbox tenant.

**Procedure:**

1. Issue a token for your sandbox sub-scope:
   ```
   # Using the identity µservice's bootstrap admin
   IDENTITY_TOKEN=$(grpcurl -plaintext localhost:5001 \
     identity.v1.TokenService/IssueToken \
     -d '{
       "principal_id":"intern-alice",
       "tenant_id":"oyatie.dev.intern-alice",
       "scopes":["tenancy.read","ontology.write"]
     }' | jq -r .access_token)
   ```

2. Make a Cedar-gated call. For day one, use the canonical
   "create ontology object" path:
   ```
   grpcurl -plaintext \
     -H "authorization: Bearer ${IDENTITY_TOKEN}" \
     localhost:5002 \
     ontology.v1.ObjectService/CreateObject \
     -d '{
       "tenant_id":"oyatie.dev.intern-alice",
       "object_type":"hello",
       "payload":{"text":"hello world"}
     }'
   ```

3. The call goes through this flow:
   - **Identity:** Validates the token; resolves the principal.
   - **Tenancy:** Resolves the tenant; loads inherited attributes.
   - **Policy-engine:** Evaluates the Cedar fragments under the
     sandbox tenant's scope; permits the action (because the
     baseline + sandbox fragments permit `ontology.write` for
     `SANDBOX` audience).
   - **Audit-chain:** Emits a `ontology.object.created` event to
     the `oyatie.dev.intern-alice.audit` stream.
   - **Ontology:** Persists the object.

4. Verify the audit event landed:
   ```
   grpcurl -plaintext \
     -H "authorization: Bearer ${IDENTITY_TOKEN}" \
     localhost:5003 \
     audit_chain.v1.QueryService/QueryEvents \
     -d '{
       "tenant_id":"oyatie.dev.intern-alice",
       "stream":"oyatie.dev.intern-alice.audit",
       "limit":10
     }'
   ```

**Acceptance:** The CreateObject call returns success with an
object id; the audit-chain query returns at least one event whose
`event_type` is `ontology.object.created`.

**Common gotchas:** Token issued under wrong sub-scope (must match
the tenant_id); ontology µservice not deployed (deploy it like
step 5); audit-chain stream not provisioned (provisioning happens
automatically on first event but may lag by 1-2 seconds).

### 4.10 Step 10: Submit first PR with multispectrum review

**Goal:** A documentation-only PR that exercises the contribution
workflow.

**Procedure:**

1. Create an isolated worktree branch (per the CLAUDE.md
   manual Wave-B bootstrap note):
   ```
   git worktree add ../oyatie-day-one-pr -b intern/<your-id>/day-one
   cd ../oyatie-day-one-pr
   ```

2. Make a small documentation change. Suggested: add yourself to
   `docs/contributors/ledger.md` with one line:
   `2026-05-20: <your-name> <your-github-handle> — joined as intern`.

3. Commit using the canonical commit message format:
   ```
   git add docs/contributors/ledger.md
   git commit -m "docs(onboarding): add <your-handle> to contributor ledger

   Day-one PR per /docs/onboarding/intern-day-one.md §4.10.

   Co-Authored-By: <your-name> <your-email>"
   ```

4. Push to remote:
   ```
   git push -u origin intern/<your-id>/day-one
   ```

5. Open a PR against `dev` using `gh`:
   ```
   gh pr create --base dev \
     --title "docs(onboarding): add <your-handle> to contributor ledger" \
     --body "$(cat <<'EOF'
   ## Summary
   - Day-one PR per `/docs/onboarding/intern-day-one.md` §4.10.
   - Adds intern entry to `docs/contributors/ledger.md`.

   ## Test plan
   - [x] `cargo fmt --check` passes (no code touched).
   - [x] `cargo clippy` passes (no code touched).
   - [ ] Multispectrum review facets F1, F2, A1, A6 pass.
   - [ ] reviewer-agent APPROVE.

   ## Keystone references
   - ADR-0242 — oyatie is a tenant doctrine (intern operates under
     `oyatie.dev.intern-<id>` sandbox)
   - ADR-0245 — substrate-vs-product layering (this PR touches docs
     only, no tier change)
   EOF
   )"
   ```

6. The Foundry pipeline (per the CLAUDE.md `coordination_surface:
   governance_pipeline`) picks up the PR, runs the admission gate,
   triggers multispectrum review with the facets called out in the
   body, and posts the verdict.

7. Wait for reviewer-agent APPROVE plus CI green; the merge queue
   admits the PR automatically.

**Acceptance:** PR is merged to `dev`. You see your name in the
contributor ledger on the `dev` branch.

**Common gotchas:** Wrong branch base (must be `dev`, not `main`);
commit message missing `Co-Authored-By` trailer; PR body missing
the keystone references; pushing without `-u` (so the upstream is
not tracked).

---

## 5. Day-one acceptance criteria

By the end of the day you should have completed all ten of these
testable items. Tick them off; if any are not done, ask your mentor
before going home.

1. Kind cluster running locally with at least one Ready node.
2. `cargo check`, `cargo test`, `cargo fmt --check`, `cargo clippy`
   all pass on the `identity` µservice.
3. Phase 1 of the reading-order doc complete; four self-check
   questions answered out loud.
4. Reserved-namespace fragment signature verified with `cosign`.
5. `tenancy` µservice deployed and `/health` returns 200.
6. `0001_create_self_tenant.sql` migration applied; `oyatie` row
   exists with `locked: true`.
7. Reserved namespace check denies all five variant attempts
   (bad, oyatie-shadow, oyat-test, Oyatié, oуatie).
8. Sandbox tenant `oyatie.dev.intern-<your-id>` created and
   resolves inherited attributes from the parent.
9. First Cedar-gated call (CreateObject on Ontology) succeeds; the
   audit-chain query returns the corresponding event.
10. First PR opened against `dev` with the canonical body format;
    multispectrum review triggered.

---

## 6. Companion reading

Once day one is complete, continue with Phase 2 of the
reading-order doc (`/docs/architecture/keystone-bundle-reading-order.md`
§4). Phase 2 covers the infrastructure ADRs and prepares you for
day two, which begins building product surfaces. Day two is also
when you take on your first feature ticket from your axis.

---

## 7. Common errors + recovery

This section covers the top 15 likely first-day errors with
diagnosis and fix. If your error is not here, ask your mentor and
propose adding it.

### 7.1 `cargo check` fails with "linker `cc` not found"

**Diagnosis:** Missing system build essentials.

**Fix:**
- Linux: `sudo apt install build-essential pkg-config libssl-dev`.
- macOS: `xcode-select --install`.

### 7.2 `kind create cluster` fails with "could not get Docker info"

**Diagnosis:** Docker daemon is not running.

**Fix:** Start Docker Desktop (macOS / Windows) or `sudo systemctl
start docker` (Linux). Verify with `docker info`.

### 7.3 `kind create cluster` hangs at "ensuring node image"

**Diagnosis:** Slow image pull or out-of-disk.

**Fix:** Pre-pull the node image: `docker pull kindest/node:v1.29.0`.
Verify disk: `df -h`. If disk is full, prune: `docker system prune
-a`.

### 7.4 `helm install` fails with "context deadline exceeded"

**Diagnosis:** The kind cluster's API server is slow on first
startup.

**Fix:** Wait 30-60 seconds and retry. If persistent, check
`kubectl cluster-info` and `kubectl get nodes`.

### 7.5 `kubectl rollout status` shows pods stuck in `ImagePullBackOff`

**Diagnosis:** Local images not loaded into the kind cluster.

**Fix:**
```
docker pull <image>
kind load docker-image <image> --name oyatie-dev
kubectl rollout restart deployment/<name>
```

### 7.6 `kubectl rollout status` shows pods stuck in `CrashLoopBackOff`

**Diagnosis:** µservice failed to start. Could be missing config,
missing database, or unmigrated schema.

**Fix:** Check the logs:
```
kubectl logs deployment/<name> --namespace <ns> --tail 100
```
Look for the first error. Most common: missing
`policy_engine_endpoint` config; missing database secret;
unmigrated schema (run the migration first).

### 7.7 `cosign verify-blob` fails with "no matching signatures"

**Diagnosis:** Wrong key file for the fragment scope.

**Fix:** Each fragment scope (baseline, pack, overlay, tenant) is
signed by a different key. The path is in
`/specs/platform-architecture.json` §`platform.policy.signing.intermediate_keys_per_scope`.
Use the right pub key for the scope.

### 7.8 Migration fails with "relation 'tenants' does not exist"

**Diagnosis:** Earlier migrations not applied.

**Fix:** Run all migrations from the start:
```
kubectl exec -n tenancy deployment/tenancy -- \
  /app/bin/tenancy-migrate up --to head
```

### 7.9 `grpcurl` fails with "transport: Error while dialing dial tcp [::1]:5000: connect: connection refused"

**Diagnosis:** Port-forward not active or terminated.

**Fix:** Re-run the port-forward in a separate terminal:
```
kubectl port-forward svc/tenancy 5000:5000 --namespace tenancy
```

### 7.10 `CreateTenant` fails with `PERMISSION_DENIED` for a non-reserved name

**Diagnosis:** Token is missing the `tenancy.write` scope, or the
caller is not under the `oyatie.platform-ops` sub-scope.

**Fix:** Issue a token with the right scope:
```
IDENTITY_TOKEN=$(grpcurl -plaintext localhost:5001 \
  identity.v1.TokenService/IssueToken \
  -d '{
    "principal_id":"bootstrap-admin",
    "tenant_id":"oyatie.platform-ops.sre",
    "scopes":["tenancy.write","tenancy.read"]
  }' | jq -r .access_token)
```

### 7.11 Reserved namespace check passes for `oyatie-shadow` (does not deny)

**Diagnosis:** Normalisation chain incomplete (likely the UTS#39
confusable removal step is missing).

**Fix:** Check the policy-engine logs for the normalisation
function used. The canonical chain is NFKC + lowercase + diacritic-
strip + UTS#39 confusable removal. Configure all four. The chain
is defined in `/specs/platform-architecture.json`
§`platform.tenancy.reserved_namespace_normalisation`.

### 7.12 Audit-chain query returns no events

**Diagnosis:** Stream lag; or stream not yet provisioned; or query
sub-scope mismatch.

**Fix:**
- Wait 2-5 seconds for the stream to lag-catch.
- Verify the stream exists:
  ```
  grpcurl -plaintext localhost:5003 \
    audit_chain.v1.AdminService/ListStreams \
    -d '{"tenant_id":"oyatie.dev.intern-<id>"}'
  ```
- Make sure the query sub-scope matches the emit sub-scope exactly.

### 7.13 PR opens against `main` instead of `dev`

**Diagnosis:** Default base branch is `main` (deprecated).

**Fix:** Re-base the PR:
```
gh pr edit --base dev
```
Or close it and re-open with `--base dev`. Per `[Branch pipeline
implemented]` memory entry, `dev` is the default; `main` is
deprecated and unprotected.

### 7.14 Multispectrum review fails with "no F-family verdict"

**Diagnosis:** The PR body is missing the keystone references
or the relevant axis tag.

**Fix:** Edit the PR body to add the references; the body format
is in §4.10 above. The Foundry pipeline retries on edit.

### 7.15 `cargo clippy -D warnings` fails on code you did not touch

**Diagnosis:** Pre-existing clippy warning in a sibling crate; CI
is stricter than the unmodified workspace.

**Fix:** Run clippy only on the crate you touched:
```
cargo clippy -p <crate> --all-targets -- -D warnings
```
If a sibling crate is broken, file an issue and tag the axis that
owns it. Do not fix it yourself in your day-one PR.

---

## 8. Who to ask for help

The escalation chain. Always start with your mentor; only escalate
upward when the lower tier is unavailable or has signed off on
escalation.

### 8.1 Tier 1: Your mentor

Your mentor is your day-one buddy. Their job is to unblock you on
laptop setup, tooling, and small workflow questions. Mentor is
assigned at orientation; if you do not have one, ask in
`#onboarding` (the canonical onboarding messenger channel).

### 8.2 Tier 2: Axis lead

If your mentor is unavailable for more than an hour, escalate to
the axis lead for the axis you are joining. Axis leads are listed
in `/docs/teams/axis-leads.md`. They are the technical owners of
the µservices under their axis.

### 8.3 Tier 3: Council

If the axis lead is unavailable, escalate to the relevant council.
Councils are the policy owners (council-architecture,
council-product, council-privacy, council-security,
council-engineering, council-legal, council-design-system).
Membership is in `/docs/teams/councils.md`. Council escalations
should be rare and reserved for cross-axis blockers.

### 8.4 Tier 4: Ops on-call

For production-impacting issues (you should not encounter any on
day one, but if you do), escalate to `ops-sre-reliability` on-call.
Pager rotation is in `/ops/runbooks/oncall-rotation.md`. Day-one
work is local-only; production should never be touched.

---

## 9. After day-one

Day two starts with Phase 2 of the reading-order doc and your first
feature ticket from your axis. Day three onward you should be
operating independently with mentor check-ins twice a day.

Within your first two weeks, you should:

- Complete all nine phases of the reading-order doc.
- Land at least three PRs against `dev`.
- Be added to the on-call shadow rotation for your axis.
- Author your first ADR (or comment substantively on an existing
  in-flight ADR).
- Pair-program with at least two engineers from different axes.

Within your first month, you should be carrying a feature ticket
end-to-end from spec to PR to merge, with multispectrum review
APPROVE from the reviewer-agent.

---

## 10. Reading-order doc link

For the full corpus walkthrough, refer to the companion document:
`/docs/architecture/keystone-bundle-reading-order.md`. That
document covers all 14 keystone ADRs, the supporting specs, the
product PRDs, the standards documents, the user-story compendia,
and the analysis documents — a total of 21-32 hours of reading
organised into nine phases. Phase 1 must be complete before you
start hands-on work in §4 of this runbook; Phases 2-9 can
interleave with coding.

---

## 11. Sign-off

Before you leave for the day, complete the day-one sign-off
checklist in `/docs/onboarding/day-one-signoff.md`. The checklist
captures:

- All ten acceptance criteria ticked.
- Mentor sign-off on each criterion.
- Self-assessment of any criteria you struggled with.
- Open questions for day two.

The sign-off doc is the gating artefact: without it your day-two
ticket cannot be assigned. The doc is reviewed at your day-two
morning sync.

Welcome to the team. The doctrine is dense; the platform is
ambitious; the support is real. Ask questions early and often. You
are not expected to know everything on day one; you are expected
to read carefully, follow the runbook, and surface confusion fast.

---

## 12. Glossary

A quick glossary of terms used in this runbook. Each term links
to the keystone ADR or spec section that defines it.

- **audit-chain.** The Merkle-sealed append-only audit substrate.
  ADR-0028 + ADR-0250.
- **audience type.** The eight-value enum on the tenant row
  (PLATFORM_OWNER, B2B_TENANT, B2C_CONSUMER, DEVELOPER, SANDBOX,
  PREVIEW, PARTNER_AGENCY, RESELLER). ADR-0244.
- **bootstrap migration.** A schema migration that runs once at
  µservice first-deploy; for `tenancy`, the canonical bootstrap
  migration creates the `oyatie` row.
- **Cedar.** The policy language. ADR-0150 + ADR-0243 + ADR-0246.
- **cell.** A K8s cluster scoped per (tenant, region) plus
  tier-classified peer cells. ADR-0248.
- **compliance pack.** A versioned signed bundle wrapping a single
  regulation. ADR-0251.
- **dev tools cell.** A peer service cell that hosts internal
  workflow definitions (formerly Foundry). ADR-0247.
- **DSAR.** Data-Subject-Access-Request. The pull-all-personal-data
  flow that every tenant must support.
- **Foundry pipeline.** The agentic dev pipeline. The
  `coordination_surface` per CLAUDE.md.
- **fragment.** A Cedar policy file. Per ADR-0246 the fragment
  lifecycle has eight states.
- **gate.** A code path that consults Cedar for a
  decision-with-policy-implication. ADR-0243 §D-3 enumerates 13
  gate categories.
- **idempotency key.** A caller-supplied unique key per (tenant,
  action, hour) that lets the platform deduplicate retries.
  ADR-0252.
- **identity.** The µservice that issues OIDC tokens. Backed by
  Zitadel.
- **kind.** Kubernetes-in-Docker, the canonical local development
  cluster.
- **kyverno.** The admission-tier policy engine. ADR-0183
  separates it from Cedar.
- **MLS.** Messaging Layer Security, RFC 9420. The E2E group-key
  agreement. ADR-MSGR-0002.
- **multispectrum review.** The canonical PR review process with
  facets F1..F11 + M1+M2 + A1..A7. Memory:
  feedback_multispectrum_review_v22 +
  feedback_multispectrum_adherence_facets.
- **policy-engine.** The peer substrate µservice that runs the
  Cedar evaluator. ADR-0246.
- **product.** A µservice tier — tenant-scoped surface. ADR-0245.
- **reserved namespace.** A tenant id prefix that no customer can
  register. The five prefixes are `oyatie`, `oya`, `oyat`,
  `oyati`. ADR-0244 + ADR-0242.
- **sandbox tenant.** An ephemeral tenant under `oyatie.dev.<id>`
  with a 24-hour-inactivity teardown.
  /specs/platform-architecture.json §ephemeral_tenant_classes.
- **shuffle sharding.** AWS-canonical two-shard placement of
  tenants across data-plane cells. ADR-0248.
- **SPIFFE / SPIRE.** The workload-identity substrate; every
  workload has an SVID. ADR-0253.
- **static stability.** The cell-internal property that the cell
  must operate without the control plane for a window long enough
  to tolerate cross-region failure. ADR-0248.
- **sub-scope.** A dotted-hierarchical scope under a tenant, depth
  ≤4. ADR-0244.
- **substrate.** A µservice tier — audience-neutral capability.
  ADR-0245.
- **tenancy.** The µservice that owns the tenants table.
- **tenant.** A row in the tenants table. Every workload runs
  under a tenant. ADR-0244.
- **tenant-model.json.** The canonical spec for the tenant row
  schema. /specs/tenant-model.json.
- **tier (cell).** Tier 0 (external) / Tier 1 (bootstrap) / Tier 2
  (control plane) / Tier 3 (data plane) / Tier 4 (reserved for
  post-cert). ADR-0248.
- **tier (µservice).** substrate / product / service-cell /
  reserved. ADR-0245.
- **worktree.** A `git worktree` directory used for parallel
  branch work without disturbing the primary checkout.

---

## 13. Document maintenance

This runbook is owned by
`council-engineering + axis-devrel + axis-identity + axis-tenancy +
axis-policy-engine + axis-audit-chain + axis-cell`. When a
bootstrap µservice changes its deployment shape, the relevant Step
section is updated, the version field is bumped, and a CHANGELOG
entry is added below.

CHANGELOG:

- 2026-05-20: Initial publication. Tracks the 14-ADR
  `2026-05-20-foundational-doctrine` bundle and the five bootstrap
  µservices (identity, tenancy, policy-engine, audit-chain, cell).

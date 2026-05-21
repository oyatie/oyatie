---
doc_class: FAQ
microservice: workflow-studio
persona: no-code-builder + tenant-workflow-author + automation-engineer
date: 2026-05-20
doc_status: published
---

# No-Code Builder FAQ — workflow-studio

## Why do we build our own studio instead of integrating n8n or partnering with Zapier?

Per ADR-XXX-workflow-studio-rationale. Three drivers:

1. **Native oyatie µservice integration**: n8n + Zapier don't have first-class nodes for our µservices (`payments`, `intelligence`, `ontology`, `audit-chain`). Building integrations from scratch on n8n means duplicating the API surface in n8n's node format; we'd be maintaining two integration paths. Building studio-native nodes from our OpenAPI/gRPC contracts is one-step.
2. **Per-tenant isolation**: n8n is single-tenant per instance. To offer per-tenant studios via n8n we'd run one n8n per tenant — expensive. Our studio is multi-tenant native with Cedar-enforced isolation.
3. **Audit-chain integration**: every editor action emits to audit-chain for paid tenant_class regulated-pack overlays. n8n doesn't have an audit primitive.

The trade-off: smaller node catalog at launch (~ 400 vs n8n's 600+). We mitigate via the custom-node SDK (paid tenant_class) + auto-generated nodes from every µservice's contract.

## What's the difference between this µservice and `workflow-engine`?

- `workflow-engine`: the EXECUTION substrate. Headless. Runs the workflow.
- `workflow-studio`: the VISUAL AUTHORING surface. UI-first. Outputs definitions for the engine.

A tenant can use the engine without the studio (write YAML or Rust SDK directly). The studio is value-add for non-technical authors.

## How does collaborative editing work without conflicts?

Per ADR-XXX-workflow-studio-collab. We use Yjs (https://yjs.dev/) — a CRDT (Conflict-free Replicated Data Type) library. Each workflow's structure (nodes, edges, parameters) is encoded as a Yjs document; every editor edit is a CRDT operation that's commutative + associative.

Cursor + selection state is also CRDT-tracked + broadcast via WebSocket.

When two users edit the same node parameter simultaneously, the CRDT resolves to "last-write-wins per-key" — both edits land, the later one is visible. For text fields, the CRDT preserves both insertions as if both users typed (Yjs text type).

Mostly the conflict surface is small because users typically edit different nodes simultaneously. For the rare same-node case, the resolution is acceptable + auditable.

## Why React 19 and not Vue / Svelte / SolidJS?

Per ADR-XXX-frontend-stack. React 19 (released 2024-12) has:

- The largest enterprise ecosystem (we use Apollo Client, Auth0 React SDK, MUI X, React Hook Form, etc.).
- React 19's `use()` hook + Server Components simplify the canvas's data fetching.
- React Concurrent rendering helps with 5k+ node canvases.
- React Native + React DOM share the same component model — when we ship a mobile studio later, code-share is automatic.

Vue + Svelte are competitive but have smaller ecosystems for the integrations we need (Apollo, MUI X). SolidJS is fast but its ecosystem is too new for a 5-year bet.

## How does the visual canvas scale to 5 000 nodes?

Per ADR-XXX-workflow-studio-canvas-scale. The naive approach (one DOM element per node) breaks at ~ 500 nodes (browser repaint becomes janky). We use:

- React Flow 12.x with virtualization: only render nodes within the viewport.
- Off-canvas serialization: nodes not in viewport are stored in Yjs but not rendered.
- Worker-thread CRDT operations: Yjs ops run in a Web Worker; main thread stays responsive.
- WebGL canvas for the edge rendering (PixiJS-backed at 1000+ nodes).

At 5 000 nodes the editor remains 60 fps on a MacBook M3 Pro with viewport panning. Lower-end hardware (4 GiB RAM Chromebook) caps at ~ 2 000 nodes.

## What's the workflow definition output format? Can I move workflows out of oyatie?

The output is `workflow-engine`'s YAML definition format — the same format you'd write by hand. Workflows are PORTABLE:

```sh
oya workflow-studio export \
    --tenant acme-corp \
    --workflow-id customer-onboarding \
    --version latest \
    --output ./customer-onboarding.yaml
```

The exported YAML can be:

- Re-imported to another oyatie cell.
- Imported into Temporal (with manual rewrite to Temporal's SDK).
- Inspected + reviewed in git.

There's no vendor lock-in at the workflow-definition level. The lock-in is at the node catalog level: a workflow that uses `payments.charge_create` requires the `payments` µservice to run. Tenants who want full portability use generic HTTP/database nodes instead of oyatie-specific ones.

## How does AI-assisted workflow generation work?

Per ADR-XXX-workflow-studio-ai-assist. The flow:

1. Tenant types a goal in natural language ("When a customer signs up, validate their email, send a welcome email, charge $1, and create them in Stripe").
2. The studio's AI-assist routes to `intelligence` µservice (typically Claude 3.5 Sonnet for reasoning + Llama 3.3 70B fine-tuned for workflow-DSL generation).
3. The LLM is prompted with the full node catalog + the goal + few-shot examples of (goal → workflow definition).
4. The LLM outputs a workflow definition in YAML.
5. The studio validates the YAML against the workflow schema; if valid, renders to the canvas.
6. If invalid (LLM hallucinated a node), the studio surfaces the error + asks the LLM to fix.

Typical generation: 5-15 node workflows in ~ 20-30 s. The user then refines manually.

Limitations:

- The LLM doesn't know tenant-specific custom nodes; generates only built-in nodes.
- Complex workflows (50+ nodes) usually need multiple iterations.
- The LLM sometimes hallucinates parameter names; validation catches.

## Can a tenant publish a workflow that doesn't pass Cedar?

No. The publish flow is gated by Cedar:

- `workflow_studio::workflow::publish` permission required.
- Each handler referenced in the workflow must be Cedar-permitted for the tenant.

If a tenant author drags a node that the tenant lacks permission for (e.g., a `payments.payout_create` node when the tenant's plan doesn't include payouts), the publish fails with a Cedar denial. The studio surfaces the denied permission with a "request access" link.

## How do I version-control workflows? Can I git-blame the canvas?

Per ADR-XXX-workflow-studio-versioning. Every workflow is git-versioned in a per-tenant repo managed by the `vcs` µservice. Each publish creates a new git commit with:

- Commit author: the publishing user.
- Commit message: the version note + auto-generated diff summary.
- Commit timestamp: the publish timestamp.

In the studio UI, the **History** panel shows each commit + the diff between commits. You can:

- View the diff between two versions (added/removed/modified nodes).
- Roll back to a previous version (creates a new commit).
- Branch + merge workflows (rare; useful for A/B testing two workflow variants).

`git blame`-equivalent: hover any node + click **History** → shows when it was added + by whom.

## What's the simulation / dry-run mode?

Per ADR-XXX-workflow-studio-simulation. Simulation runs a workflow against synthetic inputs WITHOUT dispatching real side-effects. Useful for:

- Testing a new workflow before publishing.
- Validating that a workflow handles all paths (e.g., the if-branch + the else-branch).
- Debugging a workflow that failed in production by re-running with the same inputs.

Simulation:

1. Captures the workflow definition.
2. Replaces every external-side-effect node with a stub (e.g., `payments.charge_create` returns a mock charge ID without actually charging).
3. Runs the workflow event-by-event in a sandboxed simulator.
4. Returns the full event log + variable state at each step.

Simulation latency: ~ 1-5 s for a 100-node workflow. Doesn't emit to audit-chain (it's a dry-run).

## What's time-travel debugging?

Per ADR-XXX-workflow-studio-time-travel-debug. For workflows that completed (or failed) in production, the studio can replay the workflow event-by-event:

1. Load the workflow instance ID into the studio.
2. The studio fetches the event log from `workflow-engine`.
3. Step through events using **Next** / **Previous** buttons.
4. At each step, inspect the variable state, the node parameters, the step output.

Equivalent to a step-through debugger but for workflow execution rather than code execution. Very useful for debugging "why did this workflow take the failure branch?"

## Can tenants offer workflow authoring to THEIR OWN customers (white-label)?

Yes — this is the B2B2C use case enabled by paid tenant_class. A tenant can embed the studio in their own product via:

- iFrame embed (basic; cross-origin auth).
- React component embed (`@oyatie/workflow-studio-embed`; SSO + theme customization).
- API-only (the tenant builds their own UI on top of the studio's API).

Per-tenant white-label config:

```sh
oya workflow-studio config \
    --tenant acme-corp \
    --white-label-config '{
      "logo_url": "https://acme-corp.com/logo.png",
      "primary_color": "#0066CC",
      "node_catalog_filter": ["http", "email", "slack", "tenant.acme-corp.*"],
      "ai_assist_enabled": true,
      "branding_show": false
    }'
```

The end-customer sees ACME's branded studio, not oyatie's. The underlying compute is shared (per-cell capacity); per-customer rate-limits + audit-chain isolation maintained.

## Why does the studio require a separate µservice from `workflow-engine`?

Per ADR-XXX-workflow-studio-vs-engine-separation. Three reasons:

1. **Different deployment cadence**: the studio's front-end changes frequently (UI iterations); the engine's runtime changes slowly (durability semantics).
2. **Different scaling shape**: the studio is read-heavy (canvas loads); the engine is write-heavy (event log).
3. **Different security boundary**: the studio's attack surface is the front-end (XSS, CSRF); the engine's attack surface is the workflow execution (RCE via custom code nodes).

Keeping them separate lets each evolve independently + lets us scale them independently.

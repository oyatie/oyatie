---
doc_class: Onboarding
microservice: workflow-studio
persona: no-code-builder + tenant-workflow-author + automation-engineer
related_adrs: [ADR-0263, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# No-Code Builder onboarding — first 5 working days on `workflow-studio`

Audience: a new no-code builder, tenant workflow author, or automation engineer joining the `workflow-studio` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, authored a 5-node workflow visually, published it to the engine, exercised collaborative editing with a peer, generated a workflow via AI assist, and walked the workflow-publish-failure runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 30 min). Note the n8n-class positioning + the dual role (substrate + hero product).
2. Read `ARCHITECTURE.md` § frontend-stack + § collaborative-edit-CRDT + § publish-pipeline (∼ 60 min).
3. Open the studio UI at `https://studio.<cell>.oyatie.local/`. Click through:
   - The canvas (workflow editor)
   - The node catalog sidebar
   - The version history panel
   - The simulation runner
4. Open the Grafana folder `workflow-studio`. primary boards: `studio-editor-load`, `studio-publish-rate`, `studio-collab-session-active`, `studio-ai-assist-latency`, `studio-template-instantiation-rate`.
5. Walk `runbooks/README.md`. The on-call runbooks: `publish-failure.md`, `collab-sync-stuck.md`, `editor-load-slow.md`, `template-instantiation-failed.md`, `ai-assist-timeout.md`, `custom-node-build-failed.md`, `version-conflict-merge.md`.
6. Sit in on the Wednesday studio-substrate handoff. Watch the outgoing rotation review the past-week publish-failure-rate + collab-session counts.

Acceptance: you can sketch the publish path: studio canvas → save → workflow definition serialised → submitted to `workflow-engine register` API → version published in vcs → audit-chain emit.

## Day 2 — demo_trial studio cell bootstrap + first workflow

```sh
cargo run -p oya-dev-cli -- workflow-studio bootstrap \
    --tenant-class demo_trial \
    --cell drill-syd-1 \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/workflow_engine \
    --valkey-endpoint valkey://drill-valkey-syd-1:6379 \
    --workflow-engine-endpoint http://drill-workflow-engine-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 8 min. Verify:

```sh
curl https://studio.drill-syd-1.oyatie.local/health
# Expected: {"status":"up","components":{"frontend":"ok","api":"ok","collab-ws":"ok","workflow-engine-connection":"ok"}}
```

Open the studio in the browser at `https://studio.drill-syd-1.oyatie.local/`. Log in as a tenant author.

Author your first workflow visually:

1. Click **New Workflow**.
2. Name it "First Workflow".
3. Drag a **HTTP Webhook Trigger** node onto the canvas.
4. Drag a **Set Variable** node → connect from webhook → set `greeting = "Hello, " + body.name`.
5. Drag a **HTTP Request** node → POST to `https://httpbin.org/post` with body `{"message": "{{greeting}}"}`.
6. Drag a **Conditional** node → if `responseStatus == 200` → success branch; else → failure branch.
7. On success branch: **Email Send** node → recipient `{{body.email}}`, subject "Greeted", body "{{greeting}}".
8. On failure branch: **Slack Message** node → channel `#ops-alerts`, message "Failed to greet {{body.name}}".
9. Click **Save** (Cmd-S / Ctrl-S).
10. Click **Publish** → enter version note "First test".

The publish:

1. Validates the workflow (no orphan nodes, no cycles, all required fields set).
2. Serializes to YAML.
3. POSTs to `workflow-engine` register API.
4. Commits the version to the per-tenant vcs repo.
5. Emits `workflow_studio.workflow.published` to audit-chain.

Test it:

```sh
oya workflow-engine workflow start \
    --tenant drill-acme \
    --workflow-id first-workflow \
    --version 1 \
    --inputs '{"body":{"name":"Alice","email":"alice@drill.test"}}'
```

Expected: workflow runs end-to-end; email sent; audit-chain has the events.

Acceptance: studio bootstrap; visual workflow authoring; publish + execute round-trip.

## Day 3 — Real-time collaborative editing

Real-time collab uses paid tenant_class capacity policy; demo_trial keeps last-write-wins. Set up a paid tenant_class shadow:

```sh
cargo run -p oya-dev-cli -- workflow-studio bootstrap \
    --tenant-class paid \
    --profile shadow \
    --cell drill-syd-1
```

Open the studio in TWO browser tabs (or with a peer). Both load the same workflow. Each cursor + selection appears live in the other tab.

Try:

1. Tab A drags a node to a new position; Tab B sees it move in real time.
2. Tab A edits a node's parameter; Tab B sees the parameter update.
3. Both tabs edit different nodes simultaneously; both edits land without conflict.
4. Both tabs edit the SAME node's same parameter simultaneously; CRDT resolves to last-write-wins per-key (typically not noticeable since the actual conflict surface is small).

Inspect the collab-session metadata:

```sh
oya workflow-studio collab session-show \
    --tenant drill-acme \
    --workflow-id first-workflow
# Output: { active_editors: 2, last_save: ..., crdt_ops_total: 42 }
```

Acceptance: collab editing verified end-to-end; you understand the Yjs CRDT model.

## Day 4 — AI-assisted workflow generation + template instantiation

AI-assisted generation uses paid tenant_class with per_usage billing; demo_trial can use templates.

Browse templates:

```sh
oya workflow-studio template list --category consumer
# Output:
#   - id: welcome-flow-email-only
#   - id: welcome-flow-email-plus-slack
#   - id: customer-onboarding-multi-step
#   - id: ...
```

Instantiate a template:

1. In the studio UI, click **Templates** → **Customer Onboarding Multi-step**.
2. Click **Use Template**.
3. Studio creates a new workflow pre-populated with the template's nodes.
4. Customize: change the email template, add a Slack notification, change the payment amount.
5. Publish.

For AI-assisted generation (paid tenant_class preview):

```sh
oya workflow-studio ai-generate \
    --tenant drill-acme \
    --prompt "When a customer signs up via webhook, validate their email is corporate (not gmail/yahoo), if yes send a personalized welcome and create them in Stripe, if no send a generic welcome." \
    --output workflow-draft.yaml
```

Expected: studio returns a workflow draft (5-10 nodes typical). Open in the studio UI and refine.

Acceptance: template instantiation verified; AI-assist draft inspected.

## Day 5 — Custom node SDK + publish-failure runbook

Custom node SDK uses paid tenant_class; preview under demo_trial with a local TS toolchain.

Author a custom node:

```typescript
// my-custom-node/index.ts
import { defineNode, NodeInputs, NodeOutputs } from "@oyatie/workflow-studio-node-sdk";

export default defineNode({
  id: "tenant.acme-corp.greet-vip",
  name: "Greet VIP Customer",
  category: "Customer",
  inputs: {
    customer_name: { type: "string", required: true },
    customer_segment: { type: "enum", values: ["standard", "priority", "vip"], required: true },
  },
  outputs: {
    greeting: { type: "string" },
    is_vip: { type: "boolean" },
  },
  async handler(inputs: NodeInputs): Promise<NodeOutputs> {
    const isVip = inputs.customer_segment === "vip";
    const greeting = isVip
      ? `🎉 Welcome to the VIP lounge, ${inputs.customer_name}! Your dedicated concierge is waiting.`
      : `Hello, ${inputs.customer_name}! Thanks for signing up.`;
    return { greeting, is_vip: isVip };
  },
});
```

Register the node:

```sh
oya workflow-studio node-publish \
    --tenant drill-acme \
    --node-dir ./my-custom-node/ \
    --visibility tenant-only
# Output: published custom node tenant.acme-corp.greet-vip v1
```

The node appears in the tenant's studio catalog. Drag onto canvas like any built-in node.

Now walk the publish-failure runbook. Read `runbooks/publish-failure.md`. Common causes:

1. **Orphan nodes**: nodes with no incoming connection (except triggers). Studio validation catches.
2. **Cycles**: studio validation catches.
3. **Missing required parameters**: studio inline-validation surfaces the missing fields.
4. **Cedar permit deny**: tenant author lacks `workflow_studio::workflow::publish`. Resolution: assign role.
5. **Workflow-engine register failure**: the engine rejected the definition (e.g., references a handler not registered). Resolution: register the handler first.

Simulate a failure:

```sh
oya workflow-studio publish-simulate \
    --tenant drill-acme \
    --workflow-file ./bad-workflow.yaml
# Output: validation_errors=[{"node":"step-3","issue":"missing required parameter 'recipient'"}]
```

Acceptance: custom node authored + published; publish-failure runbook walked.

## What you've learned

- demo_trial studio bootstrap + first visual workflow authoring + publish.
- Real-time collaborative editing (paid tenant_class shadow).
- Template instantiation + AI-assisted generation (paid tenant_class preview).
- Custom node SDK + publish-failure runbook.

Next week: paid tenant_class capacity review (full node catalog rollout + collab-edit at production scale), paid tenant_class tour (AI-assisted generation + custom-node SDK + time-travel debugging), regulated-pack overlay tour (sovereign-pack node allowlists + regulator-attestation), and your first production shadow.

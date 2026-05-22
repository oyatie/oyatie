---
doc_class: Tutorial
microservice: workflow-studio
persona: no-code-builder + tenant-workflow-author
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a customer-onboarding flow visually in the studio

You will: open the studio UI, drag-and-drop an 8-node customer-onboarding workflow, configure each node, simulate the workflow, publish it to the engine, execute one instance, and time-travel-debug the completed instance. Total time ≤ 60 minutes.

## Pre-requisites

- A tenant on demo_trial or paid tenant_class (ADR-0330 + ADR-0331).
- Studio access at `https://studio.<your-cell>.oyatie.local/`.
- Cedar principal in the `workflow_author` role.
- (Optional) A Slack webhook URL + a test email destination.

## Step 1 — Open the studio + create a new workflow (≤ 5 min)

1. Navigate to `https://studio.prod-syd-1.oyatie.local/`.
2. Sign in (SSO via your tenant IDP).
3. Click **New Workflow** → name "Customer Onboarding v1" → click **Create**.

The empty canvas opens. The **Node Catalog** is on the left sidebar; the **Inspector** is on the right.

## Step 2 — Drop the trigger node (≤ 3 min)

The workflow starts on a customer-signup webhook.

1. From **Triggers** → drag **HTTP Webhook** onto the canvas.
2. Click the node to open the Inspector.
3. Configure:
   - **Path**: `/v1/webhooks/customer-onboarding`
   - **Method**: POST
   - **Expected schema**: paste the JSON schema:
     ```json
     {
       "type": "object",
       "required": ["customer_id", "email", "company", "tenant_class", "amount_minor_units"],
       "properties": {
         "customer_id": {"type": "string"},
         "email":       {"type": "string", "format": "email"},
         "company":     {"type": "string"},
         "tenant_class":{"type": "string", "enum": ["demo_trial","paid"]},
         "amount_minor_units": {"type": "integer", "minimum": 100}
       }
     }
     ```

## Step 3 — Validate email is corporate (≤ 5 min)

1. Drag **Conditional** node → connect from the webhook trigger.
2. Configure:
   - **Condition expression**: `!email.match(/@(gmail|yahoo|hotmail|outlook|live)\.com$/i)`
   - **Branch label (true)**: `is_corporate`
   - **Branch label (false)**: `is_personal`

## Step 4 — Create the customer in IAM + Stripe (corporate path) (≤ 10 min)

On the `is_corporate` branch:

1. Drag **IAM Principal Create** node → connect from conditional's `is_corporate` branch.
2. Configure:
   - **Email**: `{{webhook.body.email}}`
   - **Organization**: `{{webhook.body.company}}`
   - **Role**: `tenant_user`
3. Drag **Payments Charge Create** node → connect from IAM.
4. Configure:
   - **Customer ID**: `{{iam_principal_create.principal_id}}`
   - **Amount minor units**: `{{webhook.body.amount_minor_units}}`
   - **Currency**: `USD`
   - **Description**: `First invoice on {{webhook.body.plan}} plan`
   - **Idempotency key**: `onboard-{{webhook.body.customer_id}}-charge`
5. Drag **Mail Send Template** node → connect from Payments.
6. Configure:
   - **Template ID**: `welcome_corporate`
   - **Recipient**: `{{webhook.body.email}}`
   - **Template data**:
     ```json
     {
       "company": "{{webhook.body.company}}",
       "plan": "{{webhook.body.plan}}",
       "first_charge_id": "{{payments_charge_create.charge_id}}"
     }
     ```

## Step 5 — Send a generic welcome (personal path) (≤ 3 min)

On the `is_personal` branch:

1. Drag **Mail Send Template** node → connect from conditional's `is_personal` branch.
2. Configure:
   - **Template ID**: `welcome_personal`
   - **Recipient**: `{{webhook.body.email}}`

## Step 6 — Slack notification on completion (both paths) (≤ 5 min)

After Mail (corporate) AND Mail (personal):

1. Drag a **Merge** node → connect both Mail nodes to it.
2. Drag a **Slack Webhook** node → connect from Merge.
3. Configure:
   - **Webhook URL**: `{{secrets.slack_ops_alerts_webhook}}`
   - **Message**: `Onboarded {{webhook.body.email}} ({{webhook.body.company}}) on plan {{webhook.body.plan}}`

## Step 7 — Audit-chain emit (final step) (≤ 3 min)

After Slack:

1. Drag **Audit Chain Emit** node.
2. Configure:
   - **Event class**: `customer.onboarded`
   - **Tenant ID**: `{{tenant.id}}`
   - **Payload**:
     ```json
     {
       "customer_id": "{{webhook.body.customer_id}}",
       "plan": "{{webhook.body.plan}}",
       "is_corporate_email": "{{conditional.result}}",
       "first_charge_id": "{{payments_charge_create.charge_id}}"
     }
     ```

## Step 8 — Save + validate the workflow (≤ 5 min)

1. Click **Save** (Cmd-S / Ctrl-S).
2. Click **Validate** in the top bar.
3. Expected: green check mark. If red, the Inspector shows which nodes have errors.

Common validation errors:

- Orphan node (a node not connected to the trigger).
- Missing required parameter (the Inspector shows which field).
- Cycle (a connection that creates a loop).

## Step 9 — Simulate the workflow (≤ 10 min)

Before publishing, simulate with synthetic inputs.

1. Click **Simulate** in the top bar.
2. Provide test inputs:
   ```json
   {
     "customer_id": "test-001",
     "email": "alice@example.corporate",
     "company": "Example Corp",
     "tenant_class": "paid",
     "amount_minor_units": 12500
   }
   ```
3. Click **Run Simulation**.
4. The canvas highlights each node as it executes; the right-side **Simulation Output** panel shows:
   - Each step's status (succeeded / failed / skipped).
   - Each step's output value.
   - The final workflow output.
5. Expected: all corporate-path nodes execute; personal-path nodes are skipped (the conditional's `is_personal` branch).

Try the personal-email path:

```json
{
  "customer_id": "test-002",
  "email": "alice@gmail.com",
  "company": "Example Personal",
  "tenant_class": "demo_trial",
  "amount_minor_units": 500
}
```

Expected: personal-path nodes execute; corporate-path nodes (IAM + Payments) are skipped.

## Step 10 — Publish (≤ 3 min)

1. Click **Publish** in the top bar.
2. Add a version note: "Initial customer-onboarding flow with corporate/personal split."
3. Click **Confirm**.

The publish:

- Validates the workflow.
- Registers the definition with `workflow-engine`.
- Commits to the per-tenant vcs repo.
- Emits `workflow_studio.workflow.published` to audit-chain.

The published URL is now live:

```sh
oya workflow-engine workflow get \
    --tenant acme-corp \
    --workflow-id customer-onboarding-v1 \
    --version 1
```

## Step 11 — Trigger a real instance via the webhook (≤ 5 min)

```sh
curl -X POST https://workflow-engine.prod-syd-1.oyatie.local/v1/webhooks/customer-onboarding \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "X-Oya-Webhook-Secret: $WEBHOOK_SECRET" \
    -H "Content-Type: application/json" \
    -d '{
        "customer_id":"prod-cust-001",
        "email":"alice@realcorp.com",
        "company":"Real Corp",
        "tenant_class":"paid",
        "amount_minor_units":12500
    }'
```

The webhook returns a `workflow_instance_id` immediately:

```json
{"workflow_instance_id": "wf_01HZX9..."}
```

The workflow runs in the background; the user gets a 200 OK.

Watch progress in the studio: navigate to **Instances** → `wf_01HZX9...`. The canvas re-renders with each node lit up as it completes.

## Step 12 — Time-travel debugging (≤ 7 min)

After the workflow completes (~ 5-10 s), step through the events:

1. In the **Instances** view, click on `wf_01HZX9...`.
2. Click **Time Travel**.
3. The canvas shows the workflow at time T=0 (workflow start).
4. Click **Next** to step through each event.
5. At each step, the **Variables** panel shows the current state (all variables, all step outputs).

Useful for debugging:

- "Why did this workflow take the personal-email branch?" → step to the conditional + see the email value.
- "Why did the payment fail?" → step to the payment node + see the error response.
- "Why did Slack not get notified?" → step to the Slack node + see if it was reached.

## What you've learned

- Visual workflow authoring with 8+ nodes.
- Parameter templating via `{{...}}` syntax.
- Branching via conditional + merging.
- Workflow simulation (dry-run before publish).
- Real publish + execution.
- Time-travel debugging for completed instances.

Next tutorial: `tutorials/author-custom-node.md` — author a tenant-specific custom node in TypeScript + register it in the studio catalog.

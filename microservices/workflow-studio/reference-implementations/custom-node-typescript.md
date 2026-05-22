---
doc_class: ReferenceImplementation
microservice: workflow-studio
language: TypeScript
date: 2026-05-20
doc_status: published
---

# Reference implementation — Author a custom tenant node in TypeScript for workflow-studio

A runnable example that:

1. Authors a custom node (`tenant.acme-corp.lead-enrich`) that calls an external API + transforms the response.
2. Defines typed inputs/outputs with JSON Schema.
3. Implements retry + error handling.
4. Publishes the node to the tenant's studio catalog.
5. Tests via the simulation harness.

## Project structure

```
my-tenant-nodes/
├── package.json
├── tsconfig.json
├── nodes/
│   └── lead-enrich/
│       ├── index.ts          # Node definition
│       ├── handler.ts        # Runtime handler
│       └── test.spec.ts      # Unit tests
└── README.md
```

## package.json

```json
{
  "name": "@acme-corp/oyatie-workflow-nodes",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "build": "tsc",
    "test": "vitest run",
    "publish-nodes": "oya workflow-studio node-publish --tenant acme-corp --node-dir ./dist/"
  },
  "dependencies": {
    "@oyatie/workflow-studio-node-sdk": "^1.18.0",
    "@oyatie/cedar-client": "^1.18.0",
    "@oyatie/audit-chain-client": "^1.18.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vitest": "^2.1.0",
    "@types/node": "^22.7.0"
  }
}
```

## tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "moduleResolution": "Bundler",
    "strict": true,
    "esModuleInterop": true,
    "outDir": "./dist",
    "declaration": true,
    "skipLibCheck": true
  },
  "include": ["nodes/**/*.ts"]
}
```

## nodes/lead-enrich/index.ts

```typescript
import { defineNode, NodeDefinition } from "@oyatie/workflow-studio-node-sdk";
import { leadEnrichHandler } from "./handler";

const definition: NodeDefinition = defineNode({
  id: "tenant.acme-corp.lead-enrich",
  name: "Enrich Lead",
  category: "Custom > ACME CRM",
  description: "Enrich a lead with firmographic + technographic data via the ACME enrichment API.",
  icon: "🔍",
  version: 1,

  inputs: {
    email: {
      type: "string",
      format: "email",
      required: true,
      description: "Lead email; used as the enrichment lookup key",
    },
    company_domain: {
      type: "string",
      required: false,
      description: "Company domain (optional; derived from email if not provided)",
    },
    enrichment_depth: {
      type: "enum",
      values: ["basic", "standard", "deep"],
      default: "standard",
      description: "Depth of enrichment data to retrieve",
    },
    api_key: {
      type: "secret",
      secret_namespace: "acme_enrichment_api_key",
      required: true,
      description: "ACME Enrichment API key (stored in oyatie secrets)",
    },
  },

  outputs: {
    enriched: {
      type: "object",
      schema: {
        type: "object",
        properties: {
          person: {
            type: "object",
            properties: {
              full_name: { type: "string" },
              title: { type: "string" },
              linkedin_url: { type: "string", format: "uri" },
            },
          },
          company: {
            type: "object",
            properties: {
              name: { type: "string" },
              industry: { type: "string" },
              employee_count: { type: "integer" },
              annual_revenue_usd: { type: "integer" },
              technologies: { type: "array", items: { type: "string" } },
            },
          },
          enrichment_confidence: { type: "number", minimum: 0, maximum: 1 },
        },
      },
      description: "Enriched lead data",
    },
    confidence_below_threshold: {
      type: "boolean",
      description: "True if enrichment confidence < 0.6",
    },
  },

  config: {
    retry: {
      max_attempts: 3,
      backoff: "exponential",
      initial_delay_seconds: 2,
      max_delay_seconds: 30,
      retryable_errors: ["transient", "rate_limited", "timeout"],
      non_retryable_errors: ["invalid_api_key", "lead_not_found"],
    },
    timeout_seconds: 30,
    rate_limit: {
      per_second_per_tenant: 10,
    },
    audit_chain: {
      emit_on_success: "intelligence.enrichment.completed",
      emit_on_failure: "intelligence.enrichment.failed",
      emit_payload: ["email", "enrichment_depth", "enriched.enrichment_confidence"],
    },
  },

  handler: leadEnrichHandler,
});

export default definition;
```

## nodes/lead-enrich/handler.ts

```typescript
import { NodeInputs, NodeOutputs, NodeHandlerContext, NodeError } from "@oyatie/workflow-studio-node-sdk";

interface EnrichmentApiResponse {
  person: { full_name: string; title: string; linkedin_url: string };
  company: {
    name: string;
    industry: string;
    employee_count: number;
    annual_revenue_usd: number;
    technologies: string[];
  };
  confidence: number;
}

export async function leadEnrichHandler(
  inputs: NodeInputs,
  context: NodeHandlerContext
): Promise<NodeOutputs> {
  const { email, company_domain, enrichment_depth, api_key } = inputs;

  // Derive company domain from email if not provided
  const domain = company_domain || email.split("@")[1];

  // Call the ACME enrichment API
  const response = await context.fetch(
    `https://api.acme-enrichment.com/v1/lookup?depth=${enrichment_depth}`,
    {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${api_key}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ email, company_domain: domain }),
      timeout_ms: 25_000,
    }
  );

  if (!response.ok) {
    if (response.status === 401) {
      throw new NodeError("invalid_api_key", "ACME enrichment API key is invalid");
    } else if (response.status === 404) {
      throw new NodeError("lead_not_found", `No enrichment data found for ${email}`);
    } else if (response.status === 429) {
      throw new NodeError("rate_limited", "ACME enrichment API rate limit exceeded");
    } else if (response.status >= 500) {
      throw new NodeError("transient", `ACME API ${response.status}: ${await response.text()}`);
    } else {
      throw new NodeError("unknown", `ACME API ${response.status}: ${await response.text()}`);
    }
  }

  const data: EnrichmentApiResponse = await response.json();

  return {
    enriched: {
      person: data.person,
      company: data.company,
      enrichment_confidence: data.confidence,
    },
    confidence_below_threshold: data.confidence < 0.6,
  };
}
```

## nodes/lead-enrich/test.spec.ts

```typescript
import { describe, it, expect, beforeEach } from "vitest";
import { createNodeTestHarness } from "@oyatie/workflow-studio-node-sdk/testing";
import leadEnrichNode from "./index";

describe("tenant.acme-corp.lead-enrich", () => {
  let harness: ReturnType<typeof createNodeTestHarness>;

  beforeEach(() => {
    harness = createNodeTestHarness(leadEnrichNode);
  });

  it("returns enriched data on successful API response", async () => {
    harness.mockFetch({
      url: /api\.acme-enrichment\.com/,
      response: {
        status: 200,
        body: {
          person: { full_name: "Alice Example", title: "VP Engineering", linkedin_url: "https://linkedin.com/in/alice" },
          company: { name: "Real Corp", industry: "SaaS", employee_count: 240, annual_revenue_usd: 24_000_000, technologies: ["postgres","kubernetes","rust"] },
          confidence: 0.92,
        },
      },
    });

    const result = await harness.run({
      email: "alice@realcorp.com",
      enrichment_depth: "standard",
      api_key: "test-api-key",
    });

    expect(result.enriched.person.full_name).toBe("Alice Example");
    expect(result.enriched.company.employee_count).toBe(240);
    expect(result.confidence_below_threshold).toBe(false);
  });

  it("retries on transient errors and eventually fails", async () => {
    harness.mockFetch({
      url: /api\.acme-enrichment\.com/,
      response: { status: 503, body: "service unavailable" },
    });

    await expect(
      harness.run({
        email: "alice@realcorp.com",
        enrichment_depth: "basic",
        api_key: "test-api-key",
      })
    ).rejects.toThrow("transient");

    expect(harness.getFetchCallCount()).toBe(3);  // 3 retries
  });

  it("fails fast on invalid API key (non-retryable)", async () => {
    harness.mockFetch({
      url: /api\.acme-enrichment\.com/,
      response: { status: 401, body: "invalid key" },
    });

    await expect(
      harness.run({
        email: "alice@realcorp.com",
        enrichment_depth: "basic",
        api_key: "bad-key",
      })
    ).rejects.toThrow("invalid_api_key");

    expect(harness.getFetchCallCount()).toBe(1);  // No retries on non-retryable error
  });

  it("flags low confidence", async () => {
    harness.mockFetch({
      url: /api\.acme-enrichment\.com/,
      response: {
        status: 200,
        body: {
          person: { full_name: "Bob Unknown", title: "?", linkedin_url: "" },
          company: { name: "Unknown Co", industry: "?", employee_count: 0, annual_revenue_usd: 0, technologies: [] },
          confidence: 0.42,
        },
      },
    });

    const result = await harness.run({
      email: "bob@unknown.example",
      enrichment_depth: "basic",
      api_key: "test-api-key",
    });

    expect(result.confidence_below_threshold).toBe(true);
  });
});
```

## Build + publish

```sh
# Build the node
npm run build

# Run tests
npm test

# Publish to tenant catalog
npm run publish-nodes
# Output:
#   Published custom node: tenant.acme-corp.lead-enrich v1
#   Visibility: tenant-only
#   Available in studio: yes (refresh the studio UI to see it)
```

## Use the node in a workflow

1. In the studio UI, open the node catalog → **Custom > ACME CRM** → drag **Enrich Lead** onto the canvas.
2. Configure:
   - **Email**: `{{webhook.body.lead_email}}`
   - **Enrichment depth**: `standard`
   - **API key**: select from secrets namespace `acme_enrichment_api_key`
3. Connect the output to downstream nodes (e.g., a Slack alert if confidence is low; a Salesforce upsert if confidence is high).

The studio enforces:

- The retry policy is applied automatically.
- Audit-chain emission on every invocation.
- Cedar check on every invocation (`tenant.acme-corp.lead-enrich::invoke`).

## Where this file lives

`microservices/workflow-studio/reference-implementations/custom-node-typescript.md` (this file). The runnable example project ships at `microservices/workflow-studio/reference-implementations/custom-node-example/` once the SDK is published.

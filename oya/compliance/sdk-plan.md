---
microservice: compliance
doc: SdkPlan
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [axis-frontend]
date: 2026-05-18
related_adrs: [ADR-0209]
---

# Compliance — SDK Plan

## SDKs

| Language | SDK package | Audience |
|---|---|---|
| Rust | `oya-compliance-client` | Internal µservices emitting evidence + DSAR requests |
| TypeScript | `@oya/compliance-client` | Backstage auditor portal + tenant admin UI |
| Python | `oya_compliance_client` | Data engineers running ad-hoc evidence collection |
| Go | `github.com/oyadev/oya-compliance-client-go` | Operator-cluster integrations |

## Surface

```rust
// Rust SDK example
use oya_compliance_client::{Client, EvidenceArtifactKind, ComplianceFramework};

let client = Client::new("https://compliance.oya.svc").with_spiffe_id();

client.emit_artifact(
    EvidenceArtifactKind::CiArtifactHash,
    ComplianceFramework::Soc2TypeII,
    "tenant_a",
    "evt_ci_build_42",
    seal_hex,
).await?;
```

```typescript
// TypeScript SDK example
import { ComplianceClient } from '@oya/compliance-client';

const client = new ComplianceClient({ baseUrl: '/api/v1' });
const coverage = await client.getCoverage({ framework: 'soc2-type-2', tenant: 'tenant_a' });
```

## Versioning

SemVer per ADR-0145 inter-µservice API. Breaking changes require ADR + sunset (per [feedback_no_silent_regression]).

## Authentication

- Internal: SPIFFE-ID via service mesh.
- External (auditor portal): Zitadel OIDC token.

## References

- ADR-0209 — substrate authority.
- ADR-0145 — inter-microservice communication.

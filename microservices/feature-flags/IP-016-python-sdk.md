# IP-016 — Python SDK

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: python-sdk
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0245, ADR-0248, ADR-0253, ADR-0258
**companion_ips**: IP-013, IP-014, IP-015
**references**: contracts/openfeature-sdk-contract.md; sdk-plan.md

## Scope

Python SDK implementing the OpenFeature `Provider` interface. asyncio-first; `httpx` + HTTP/3; `asyncio.Task` for SSE stream; thread-safe `dict` cache with `asyncio.Lock`. Used by ML pipelines and data-science µservices.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `OyatieProvider` class | Implements `openfeature-sdk` `AbstractProvider`; `resolve_boolean_details`, `resolve_string_details`, `resolve_integer_details`, `resolve_float_details`, `resolve_object_details` |
| 2 | `httpx` transport | `httpx.AsyncClient` with HTTP/3 (`h3` extra); TLS 1.3; connection pool reuse |
| 3 | Cache | `dict[(tenant_id, flag_key), CachedFlag]`; `asyncio.Lock`; TTL 30s; LKG: `json.dump` to `~/.cache/oya-ff/lkg.json` |
| 4 | SSE invalidation | `httpx-sse` `aconnect_sse` to `/api/v1/flags/stream`; `asyncio.Task` background; exponential backoff on error |
| 5 | Sync wrapper | `resolve_boolean_details_sync()` via `asyncio.run()` for Jupyter/script callers |
| 6 | Type hints | Fully typed; `py.typed` marker; passes `mypy --strict` |
| 7 | Packaging | `pyproject.toml`; `pip install oyatie-feature-flags`; optional `[grpc]` extra for gRPC transport |
| 8 | Tests | `pytest` + `pytest-asyncio`; 90%+ coverage; `httpx.MockTransport` for unit tests; OpenFeature conformance |

## Usage

```python
from openfeature import api
from oyatie_feature_flags import OyatieProvider, OyatieEvaluationContext

await api.set_provider(OyatieProvider(
    endpoint="https://feature-flags.internal",
    tenant_id="tenant_abc",
))

client = api.get_client()
ctx = OyatieEvaluationContext(
    targeting_key="user_xyz",
    audience_type="B2B",
)
enabled: bool = await client.get_boolean_value("my-flag", False, ctx)
```

## Definition of Done

- `pytest` green
- `mypy --strict` zero errors
- SSE stream: background task survives 24h without leak (asyncio task leak detector)
- OpenFeature Python conformance suite passes
- Sync wrapper: `resolve_boolean_details_sync()` callable from non-async context

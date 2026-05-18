# API Gateway Failure Modes

| Failure | Designed response | Evidence emitted |
|---|---|---|
| JWKS refresh unavailable | Use unexpired cached keys, then fail closed when stale | `oya.api_gateway.request.denied` |
| WAF rule pack regression | Roll back to previous signed bundle | `oya.api_gateway.waf.triggered` |
| Rate-limit backend unavailable | Fall back to per-replica limiter and mark degraded | `oya.api_gateway.request.admitted` with degraded flag |
| Cross-cell route requested | Deny before workload dispatch | `oya.api_gateway.request.denied` |

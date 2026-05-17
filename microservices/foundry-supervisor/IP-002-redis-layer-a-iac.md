---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-002-redis-layer-a-iac
status: pending
execution_unit: ChangeSet
owner: ops-sre-reliability
acceptance_lanes: [helm-lint, helm-install-smoke, oya-check-redis-acl-enforced]
---

# IP-002: Redis Cluster Layer-A IaC

## Intent

Helm chart for Redis Cluster (3 shards × 2 replicas per pack region); kill-switch state cache + supervision-event-bus Redis Streams; per-pod ACL tokens.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/foundry-supervisor/iac/helm/redis/Chart.yaml` | create |
| `microservices/foundry-supervisor/iac/helm/redis/values.yaml` | create |
| `microservices/foundry-supervisor/iac/kustomize/base/kustomization.yaml` | update |

## Substrate selections

- Redis 7.x LTS (cite redis.io/docs/management/scaling/).
- Cluster mode with 3 shards × 2 replicas.
- AOF every-second.
- Per-user ACL with pattern-bounded key access.
- OpenBao-issued ACL tokens (rotated 30d).

## Acceptance Gates

```bash
helm lint microservices/foundry-supervisor/iac/helm/redis
helm install --dry-run --debug -n foundry-supervisor redis microservices/foundry-supervisor/iac/helm/redis
cargo run -p oya-dev-cli -- gate validate redis-acl-enforced --microservice foundry-supervisor
```

## Test Plan

| Test | Verifies |
|---|---|
| Helm lint | chart syntactic |
| Helm install smoke (kind cluster) | cluster forms 3 shards × 2 replicas |
| AOF every-second | verified via `redis-cli config get appendfsync` |
| ACL pattern test | per-tenant token cannot read other-tenant keys |

## Halt Conditions

- ACL `default` user has any non-default access.
- AOF disabled.

## Next IP

[`IP-003-k8s-operator-iac.md`](IP-003-k8s-operator-iac.md)

## References

- `policy/supervisor-isolation.md` TI-R-*.
- Redis Cluster — `redis.io/docs/management/scaling/`.
- Redis ACL — `redis.io/docs/management/security/acl/`.
- `capacity-model.md` §"Redis Cluster Sizing".

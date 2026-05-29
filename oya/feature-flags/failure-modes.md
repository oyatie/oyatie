# Feature Flags Failure Modes

| Failure | Designed response | Evidence emitted |
|---|---|---|
| Flag store unavailable | SDK returns last-known variant until TTL, then default | `oya.feature_flags.flag.evaluated` with degraded flag |
| Cedar predicate error | Return default variant and mark predicate error | `oya.feature_flags.flag.evaluated` |
| Audit-chain unavailable | Queue definition/evaluation events for replay | `oya.feature_flags.flag.changed` |
| Kill-switch stale | Prefer disabled variant when freshness cannot be proven | `oya.feature_flags.kill_switch.invoked` |

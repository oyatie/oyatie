# Intelligence Failure Modes

| Failure | Designed response | Evidence emitted |
|---|---|---|
| Consent grant missing | Refuse request before model invocation | `oya.intelligence.policy.refused` |
| Retrieval citation out of scope | Refuse citation and omit result | `oya.intelligence.retrieval.context-bound` |
| Model adapter unavailable | Return no-draft response with retry guidance | `oya.intelligence.assist-draft.completed` |
| Budget exhausted | Refuse request and preserve deterministic builder state | `oya.intelligence.policy.refused` |

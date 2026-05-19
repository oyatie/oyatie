# Feature Flags Data Residency

Flag definitions are scoped to tenant residency packs. Evaluation context carries tenant, persona, and cohort metadata but does not move user payloads across cells.

Runtime proof of residency is outside this design/spec gate.

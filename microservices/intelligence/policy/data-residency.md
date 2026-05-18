---
doc_kind: policy
owner_team: axis-intelligence
related_adrs: [ADR-0215, ADR-0220]
---

# Intelligence Data Residency

Intelligence stores prompt history, retrieval citations, refusal evidence, and cost attribution inside the active context's residency pack. Cross-context retrieval requires a consent-graph grant, and the emitted audit event records both the active context and the grant id.

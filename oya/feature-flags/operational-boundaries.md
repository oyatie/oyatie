# Feature Flags Operational Boundaries

The feature-flags team owns flag definition APIs, evaluation semantics, SDK cache contract, lifecycle metadata, and audit-required event emission.

Application teams own flag usage in code paths and must supply sunset dates, kill-switch intent, and fallback behavior.

This boundary is design/spec evidence and does not claim production readiness.

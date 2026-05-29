# API Gateway Operational Boundaries

The API gateway team owns north-south admission, TLS termination, WAF policy, coarse tenant and cell scoping, route bundle rollout, and edge telemetry.

Workload teams own domain authorization, resource policy, business validation, and data mutations after a request is admitted.

This document defines ownership and incident boundaries. It does not assert production readiness or observed SLO achievement.

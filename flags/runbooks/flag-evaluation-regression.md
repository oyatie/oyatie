# Flag Evaluation Regression Runbook

## Trigger

Flag evaluations return default variants unexpectedly, audit-required evaluations stop emitting, or kill-switch evaluation latency exceeds the design envelope.

## Checks

1. Compare the current flag definition bundle with the previous signed bundle.
2. Verify Cedar predicate syntax and tenant scope.
3. Confirm client SDK cache TTL has not exceeded the design target.
4. Revert the flag bundle if kill-switch behavior is incorrect.

# Connect Retirement Failure Modes

| Failure | Designed response | Evidence emitted |
|---|---|---|
| Sub-service status drift | Reconcile against sub-service folders and retirement plan | `oya.connect.retirement.status_changed` |
| New runtime scope proposed | Reject under policy and redirect to sub-service owner | `oya.connect.retirement.readiness_checked` |
| Deletion criteria ambiguous | Keep umbrella present and update retirement evidence | `oya.connect.retirement.status_changed` |

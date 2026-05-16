---
doc_status: published
---

# Foundry Supervisor Operations

## Daemon Lifecycle
The supervisor runs as a long-lived process that heartbeats at a configurable interval.

### Tick Sequence
1. **Peek Inbox:** Lock the next message in the JSONL inbox.
2. **Route:** Select the best available account using `RoutePolicy`.
3. **Verify Settings:** Check for drift against canonical templates.
4. **Enforce:** Check usage limits and cost ceilings.
5. **Spawn:** Invoke the provider CLI as a subprocess.
6. **Settle:** Capture output, commit to outbox, and release the inbox lock.

## Signals
- **SIGTERM/SIGINT:** Graceful shutdown; wait for in-flight sessions to complete or reach watchdog timeout.
- **SIGKILL:** Immediate termination; recovery logic handles orphan locks and partial writes on restart.

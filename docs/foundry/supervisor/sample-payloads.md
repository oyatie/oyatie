# Foundry Supervisor Sample Payloads

## Inbox Line (JSONL)
```json
{"state":"Queued","id":"msg-001","body":"base64-encoded-body"}
```

## Outbox Line (JSONL)
```json
{"id":"msg-001","response":"base64-encoded-response","status":"Committed"}
```

## Audit Row (Audit-Chain)
```json
{
  "event_id": "01H... ",
  "event_class": "foundry_supervisor_tick_spawned",
  "principal": "acct-claude-01",
  "capability": "foundry.supervisor.spawn",
  "autonomy_tier": "T3PropAct"
}
```

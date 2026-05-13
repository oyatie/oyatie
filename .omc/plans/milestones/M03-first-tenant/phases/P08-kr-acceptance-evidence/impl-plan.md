# P08 — KR Acceptance Evidence: Implementation Plan

## Metadata
- phase: P08-kr-acceptance-evidence
- milestone: M03-first-tenant
- depends_on: [P01-hr, P02-payroll, P03-accounting, P04-connect-pro-mail, P05-connect-pro-messenger, P06-application-b2b-live, P07-workflow-studio-editor]
- parallel_with: []
- grit_claim_symbols: [m03.p08.evidence.load-tests, m03.p08.evidence.slo, m03.p08.evidence.corpus, m03.p08.evidence.restore-drill, m03.p08.evidence.bundle]
- icm_topics: [context-oyatie, decisions-oyatie, errors-resolved]
- icm_keywords: [acceptance,evidence,kr,payroll,edi,year-end,legal-hold,slo,restore-drill,corpus]

---

## 0. Deliverable Overview

P08 is the acceptance-evidence harness for ADR-0210 M3 closure. It does not introduce new
µservice domain logic; it wires together the P01-P07 subsystems under one verifiable
evidence bundle that proves the M3 closure criteria are met for one paying KR-group tenant.

```
tests/acceptance/
  k6/                    # Load test scripts (3k-person payroll shape + all SLOs)
  playwright/            # E2E browser smoke tests
  runbooks/              # Restore drill runbook (markdown; operator-executable)
  gates/                 # oya gate validate corpus-citations + audit-chain scripts
  evidence/              # Evidence bundle JSON structure + generation script
  monitoring/            # Prometheus rules + Grafana dashboard JSON
```

---

## 1. k6 Load Test Scripts

### 1.1 Payroll Run — 3k Person Shape

```javascript
// tests/acceptance/k6/payroll-run-3k.k6.js
// ADR-0210 gate: full gross-to-net run for 3,000 employees ≤ 30s wall-clock.

import http from 'k6/http';
import { check } from 'k6';

export const options = {
  scenarios: {
    payroll_run: {
      executor: 'per-vu-iterations',
      vus: 1,             // Payroll run is a single orchestrated batch
      iterations: 1,
      maxDuration: '35s', // 5s buffer; fail if > 35s
    },
  },
  thresholds: {
    // Wall-clock for the full run completion (polling until status=completed)
    'http_req_duration{name:poll_run_complete}': ['max<30000'],
    'http_req_failed': ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'https://api.oyatie.local';
const TENANT_TOKEN = __ENV.TENANT_TOKEN;
const TENANT_ID = __ENV.TENANT_ID;
const PAYROLL_PERIOD_ID = __ENV.PAYROLL_PERIOD_ID;  // pre-seeded 3k-employee period

const headers = {
  'Authorization': `Bearer ${TENANT_TOKEN}`,
  'Content-Type': 'application/json',
  'X-Oyatie-Tenant-Id': TENANT_ID,
};

export default function () {
  // 1. Trigger payroll run
  const triggerRes = http.post(
    `${BASE_URL}/api/payroll/runs`,
    JSON.stringify({ period_id: PAYROLL_PERIOD_ID }),
    { headers, tags: { name: 'trigger_run' } }
  );
  check(triggerRes, { 'trigger 202': (r) => r.status === 202 });

  const runId = JSON.parse(triggerRes.body).run_id;
  const startMs = Date.now();

  // 2. Poll until completed or 30s timeout
  let completed = false;
  while (!completed && Date.now() - startMs < 30000) {
    const pollRes = http.get(
      `${BASE_URL}/api/payroll/runs/${runId}`,
      { headers, tags: { name: 'poll_run_complete' } }
    );
    const body = JSON.parse(pollRes.body);
    if (body.status === 'completed') {
      completed = true;
      check(pollRes, {
        'run completed within 30s': () => (Date.now() - startMs) < 30000,
        'entry count is 3000': () => body.entry_count === 3000,
        'audit_hash present': () => typeof body.audit_hash === 'string' && body.audit_hash.length > 0,
      });
    } else if (body.status === 'failed') {
      check(pollRes, { 'run must not fail': () => false });
      break;
    }
    // 500ms poll interval
    import { sleep } from 'k6';
    sleep(0.5);
  }
}
```

### 1.2 Payslip Read — p99 ≤ 50ms

```javascript
// tests/acceptance/k6/payslip-read.k6.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    payslip_read: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 500 },
        { duration: '3m', target: 1000 },
        { duration: '1m', target: 0 },
      ],
    },
  },
  thresholds: {
    'http_req_duration{name:get_payslip}': ['p(99)<50'],
    'http_req_failed': ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'https://api.oyatie.local';
// PAYSLIP_IDS: newline-delimited file of 3000 payslip UUIDs from the completed run
import { SharedArray } from 'k6/data';
const payslipIds = new SharedArray('payslipIds', function () {
  return open(__ENV.PAYSLIP_IDS_FILE).split('\n').filter(Boolean);
});

export default function () {
  const id = payslipIds[Math.floor(Math.random() * payslipIds.length)];
  const res = http.get(
    `${BASE_URL}/api/payroll/payslips/${id}`,
    {
      headers: {
        'Authorization': `Bearer ${__ENV.TENANT_TOKEN}`,
        'X-Oyatie-Tenant-Id': __ENV.TENANT_ID,
      },
      tags: { name: 'get_payslip' },
    }
  );
  check(res, { 'payslip 200': (r) => r.status === 200 });
  sleep(0.05);
}
```

### 1.3 Shell Frame — p99 ≤ 100ms at 10k Sessions

```javascript
// tests/acceptance/k6/shell-frame-10k.k6.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    shell_sessions: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 5000 },
        { duration: '5m', target: 10000 },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    'http_req_duration{name:shell_frame}': ['p(99)<100'],
    'http_req_failed': ['rate<0.001'],
  },
};

export default function () {
  const res = http.get(
    `${__ENV.BASE_URL || 'https://app.oyatie.local'}/dashboard`,
    {
      headers: {
        'Cookie': `__Host-oyatie-session=${__ENV.SESSION_COOKIE}`,
        'Accept': 'text/html',
      },
      tags: { name: 'shell_frame' },
    }
  );
  check(res, {
    'shell 200': (r) => r.status === 200,
    'shell has nav': (r) => r.body.includes('oyatie-nav'),
  });
  sleep(0.1);
}
```

### 1.4 4대보험 EDI Submission — p99 ≤ 200ms

```javascript
// tests/acceptance/k6/edi-submission.k6.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    edi_submission: {
      executor: 'constant-vus',
      vus: 50,
      duration: '5m',
    },
  },
  thresholds: {
    'http_req_duration{name:submit_edi}': ['p(99)<200'],
    'http_req_failed': ['rate<0.001'],
  },
};

export default function () {
  const res = http.post(
    `${__ENV.BASE_URL}/api/payroll/edi/submissions`,
    JSON.stringify({
      period_id: __ENV.PAYROLL_PERIOD_ID,
      agency: 'nps',           // nps | nhis | moel
      submission_kind: 'acquisition',
    }),
    {
      headers: {
        'Authorization': `Bearer ${__ENV.TENANT_TOKEN}`,
        'Content-Type': 'application/json',
        'X-Oyatie-Tenant-Id': __ENV.TENANT_ID,
      },
      tags: { name: 'submit_edi' },
    }
  );
  check(res, { 'edi 202': (r) => r.status === 202 });
  sleep(1);
}
```

### 1.5 Legal Hold Verification — p99 ≤ 300ms

```javascript
// tests/acceptance/k6/legal-hold.k6.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    legal_hold_ops: {
      executor: 'constant-vus',
      vus: 20,
      duration: '3m',
    },
  },
  thresholds: {
    'http_req_duration{name:place_hold}': ['p(99)<300'],
    'http_req_duration{name:verify_hold}': ['p(99)<100'],
    'http_req_failed': ['rate<0.001'],
  },
};

export default function () {
  const placeRes = http.post(
    `${__ENV.BASE_URL}/api/connect/legal-holds`,
    JSON.stringify({
      mailbox_id: __ENV.TEST_MAILBOX_ID,
      reason: 'M3 acceptance evidence test',
      initiated_by: __ENV.LEGAL_ADMIN_USER_ID,
    }),
    {
      headers: {
        'Authorization': `Bearer ${__ENV.TENANT_TOKEN}`,
        'Content-Type': 'application/json',
        'X-Oyatie-Tenant-Id': __ENV.TENANT_ID,
      },
      tags: { name: 'place_hold' },
    }
  );
  check(placeRes, { 'hold placed 201': (r) => r.status === 201 });

  if (placeRes.status === 201) {
    const holdId = JSON.parse(placeRes.body).hold_id;
    const verifyRes = http.get(
      `${__ENV.BASE_URL}/api/connect/legal-holds/${holdId}`,
      {
        headers: { 'Authorization': `Bearer ${__ENV.TENANT_TOKEN}`, 'X-Oyatie-Tenant-Id': __ENV.TENANT_ID },
        tags: { name: 'verify_hold' },
      }
    );
    check(verifyRes, {
      'hold active': (r) => JSON.parse(r.body).status === 'active',
      'hold has audit trail': (r) => JSON.parse(r.body).audit_entries?.length > 0,
    });
  }
  sleep(1);
}
```

---

## 2. SLO Burn-Rate Monitoring

### 2.1 Prometheus Recording Rules

```yaml
# monitoring/prometheus/rules/oyatie-slo.yaml
groups:
  - name: oyatie_slo_windows
    interval: 30s
    rules:

      # ── Availability: 99.9% (43.8 min/month budget) ──────────────────
      - record: job:oyatie_request_error_rate:5m
        expr: |
          sum(rate(http_requests_total{job=~"oya-.*",status=~"5.."}[5m]))
          / sum(rate(http_requests_total{job=~"oya-.*"}[5m]))

      - record: job:oyatie_request_error_rate:1h
        expr: |
          sum(rate(http_requests_total{job=~"oya-.*",status=~"5.."}[1h]))
          / sum(rate(http_requests_total{job=~"oya-.*"}[1h]))

      # Burn-rate alert: 14.4x in 1h window (fast burn) = 2% budget in 5min
      - alert: OyatieSloFastBurn
        expr: job:oyatie_request_error_rate:5m > (14.4 * 0.001)
        for: 2m
        labels:
          severity: page
        annotations:
          summary: "Oyatie error rate fast-burning SLO budget"
          runbook: "https://wiki.oyatie.internal/runbooks/slo-burn"

      # Burn-rate alert: 6x in 6h window (slow burn) = 5% budget in 1h
      - alert: OyatieSloSlowBurn
        expr: job:oyatie_request_error_rate:1h > (6 * 0.001)
        for: 15m
        labels:
          severity: ticket
        annotations:
          summary: "Oyatie error rate slow-burning SLO budget"

      # ── Payroll run latency: ≤ 30s for 3k-person batch ───────────────
      - alert: PayrollRunLatencySloBreached
        expr: |
          histogram_quantile(0.99,
            sum(rate(payroll_run_duration_seconds_bucket[5m])) by (le)
          ) > 30
        for: 1m
        labels:
          severity: page
        annotations:
          summary: "Payroll run p99 exceeds 30s SLO"

      # ── Payslip read latency: p99 ≤ 50ms ─────────────────────────────
      - alert: PayslipReadLatencySloBreached
        expr: |
          histogram_quantile(0.99,
            sum(rate(http_request_duration_seconds_bucket{handler="/api/payroll/payslips/:id"}[5m])) by (le)
          ) > 0.05
        for: 2m
        labels:
          severity: ticket
        annotations:
          summary: "Payslip read p99 exceeds 50ms SLO"

      # ── Shell frame: p99 ≤ 100ms ──────────────────────────────────────
      - alert: ShellFrameLatencySloBreached
        expr: |
          histogram_quantile(0.99,
            sum(rate(http_request_duration_seconds_bucket{handler="/dashboard"}[5m])) by (le)
          ) > 0.1
        for: 2m
        labels:
          severity: ticket
        annotations:
          summary: "Shell frame p99 exceeds 100ms SLO"

      # ── 7-day SLO availability window ─────────────────────────────────
      - record: oyatie:availability_7d
        expr: |
          1 - (
            sum(increase(http_requests_total{job=~"oya-.*",status=~"5.."}[7d]))
            / sum(increase(http_requests_total{job=~"oya-.*"}[7d]))
          )

      - alert: OyatieAvailabilityBelow999
        expr: oyatie:availability_7d < 0.999
        for: 5m
        labels:
          severity: page
        annotations:
          summary: "Oyatie 7-day availability below 99.9%"
          description: "Current 7d availability: {{ $value | humanizePercentage }}"
```

### 2.2 Grafana Dashboard (abridged JSON)

```json
{
  "title": "Oyatie M3 SLO Dashboard",
  "uid": "oyatie-m3-slo",
  "panels": [
    {
      "title": "7-day Availability",
      "type": "stat",
      "targets": [{ "expr": "oyatie:availability_7d * 100" }],
      "thresholds": { "steps": [
        { "color": "red", "value": 0 },
        { "color": "yellow", "value": 99.5 },
        { "color": "green", "value": 99.9 }
      ]}
    },
    {
      "title": "Payroll Run Duration p99 (s)",
      "type": "timeseries",
      "targets": [{ "expr": "histogram_quantile(0.99, sum(rate(payroll_run_duration_seconds_bucket[5m])) by (le))" }]
    },
    {
      "title": "Payslip Read p99 (ms)",
      "type": "timeseries",
      "targets": [{ "expr": "histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket{handler='/api/payroll/payslips/:id'}[5m])) by (le)) * 1000" }]
    },
    {
      "title": "Shell Frame p99 (ms)",
      "type": "timeseries",
      "targets": [{ "expr": "histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket{handler='/dashboard'}[5m])) by (le)) * 1000" }]
    },
    {
      "title": "EDI Submission Error Rate",
      "type": "timeseries",
      "targets": [{ "expr": "sum(rate(edi_submission_errors_total[5m])) by (agency)" }]
    },
    {
      "title": "Error Budget Burn Rate (5m)",
      "type": "gauge",
      "targets": [{ "expr": "job:oyatie_request_error_rate:5m / 0.001" }]
    }
  ]
}
```

---

## 3. Restore Drill Runbook

```markdown
# Restore Drill Runbook — Oyatie M3 KR Acceptance

**Purpose**: Verify that Oyatie can restore tenant data from backup within the RTO/RPO
commitments stated in ADR-0210 (RTO ≤ 4h, RPO ≤ 1h).

**Frequency**: Must be executed and evidence captured before M3 acceptance sign-off.
**Prerequisites**: Access to OCI Object Storage backup bucket; Citus admin credentials;
  Kafka KRaft admin; OpenBao operator token.

## Step 1 — Identify Point-in-Time

1. Record `NOW` timestamp: `date -u +%Y-%m-%dT%H:%M:%SZ`
2. Note most-recent WAL-E/pgBackRest backup older than NOW: 
   `pgbackrest --stanza=oyatie info`
3. Calculate RPO gap: `NOW - backup_stop_time`. Must be ≤ 1h.

## Step 2 — Restore Postgres (Citus)

```bash
# On coordinator node:
pgbackrest --stanza=oyatie --type=time \
  --target="$RESTORE_TARGET_TIME" \
  --target-action=promote restore

# Verify shard count matches pre-restore:
psql -c "SELECT count(*) FROM pg_dist_shard;"

# Verify RLS policies still active:
psql -c "SELECT tablename, rowsecurity FROM pg_tables WHERE schemaname IN ('hr','payroll','accounting','workflow','connect_pro','connect_personal','application') ORDER BY tablename;"
```

## Step 3 — Replay WAL to RPO

```bash
# WAL replay runs automatically after restore.
# Wait for recovery to complete:
until psql -c "SELECT pg_is_in_recovery();" | grep -q 'f'; do sleep 5; done
echo "Recovery complete at: $(date -u)"
```

## Step 4 — Validate Tenant Data Integrity

```bash
# Run acceptance data integrity checks:
cargo test --test restore_drill_integrity -- --nocapture

# Checks performed:
# - Row counts match pre-restore snapshot for test tenant
# - Outbox tables have no published_at < restore_point that are unpublished
# - RLS: queries without oyatie.tenant_id setting return 0 rows on all tables
# - Ed25519 audit chain: verify last 100 events per µservice
```

## Step 5 — Verify Kafka Replay

```bash
# Confirm Kafka consumer groups reset to restore-point offset:
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BROKERS \
  --group oya-hr-consumer --describe

# Re-publish any outbox events after restore point:
cargo run --bin outbox-redelivery -- \
  --since "$RESTORE_TARGET_TIME" \
  --tenant-id "$TEST_TENANT_ID"
```

## Step 6 — Verify OpenBao Key Access

```bash
# Confirm tenant DEKs accessible (no key rotation needed post-restore):
bao kv get secret/tenants/$TEST_TENANT_ID/dek
# Must return HTTP 200 with key material

# Verify ratchet session keys still valid:
bao kv get secret/tenants/$TEST_TENANT_ID/ratchet-sessions
```

## Step 7 — Smoke Test

```bash
# Run minimal smoke test against restored environment:
cargo test --test smoke -- --nocapture \
  --test-threads=1 \
  -- restore_smoke
```

## Step 8 — Record Evidence

```bash
# Capture restore duration:
echo "RTO: $(date -u) - $RESTORE_START_TIME"

# Export evidence:
cargo run --bin evidence-generator -- \
  --phase restore-drill \
  --output tests/acceptance/evidence/restore-drill-$(date +%Y%m%d).json
```

## Acceptance Criteria
- RTO: full restore completes within 4 hours
- RPO: data loss window ≤ 1 hour
- RLS: all tenant isolation policies active post-restore
- Audit chain: no gaps in Ed25519 event seals post-restore
- Kafka: all outbox events after restore point re-delivered
```

---

## 4. Corpus Citation Audit Gate

```rust
// tests/acceptance/gates/corpus_citation_audit.rs
// `oya gate validate corpus-citations`
// Scans all Rust source files in crates/ for LegalCitation { article_id, corpus_sha }
// usages and cross-references them against corpus.lock.
// Fails if any corpus_sha does not match corpus.lock or if corpus.lock is stale.

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
struct CorpusLock {
    version: String,
    entries: Vec<CorpusEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct CorpusEntry {
    article_id: String,
    corpus_sha: String,
    statute: String,
    effective_date: String,
}

#[derive(Debug)]
struct CitationUsage {
    file: PathBuf,
    line: usize,
    article_id: String,
    corpus_sha: String,
}

pub fn validate_corpus_citations(
    corpus_lock_path: &std::path::Path,
    crates_root: &std::path::Path,
) -> Result<ValidationReport, ValidationError> {
    let lock_content = std::fs::read_to_string(corpus_lock_path)
        .map_err(|e| ValidationError::LockFileNotFound(e.to_string()))?;
    let corpus_lock: CorpusLock = toml::from_str(&lock_content)
        .map_err(|e| ValidationError::LockFileInvalid(e.to_string()))?;

    let lock_index: HashMap<String, &CorpusEntry> = corpus_lock.entries.iter()
        .map(|e| (e.article_id.clone(), e))
        .collect();

    let usages = scan_citation_usages(crates_root)?;
    let mut violations = vec![];

    for usage in &usages {
        match lock_index.get(&usage.article_id) {
            None => violations.push(ValidationViolation::UnknownArticle {
                file: usage.file.clone(),
                line: usage.line,
                article_id: usage.article_id.clone(),
            }),
            Some(entry) => {
                if entry.corpus_sha != usage.corpus_sha {
                    violations.push(ValidationViolation::StaleSha {
                        file: usage.file.clone(),
                        line: usage.line,
                        article_id: usage.article_id.clone(),
                        found_sha: usage.corpus_sha.clone(),
                        expected_sha: entry.corpus_sha.clone(),
                    });
                }
            }
        }
    }

    Ok(ValidationReport {
        total_citations: usages.len(),
        violations,
        corpus_lock_version: corpus_lock.version,
    })
}

fn scan_citation_usages(
    root: &std::path::Path,
) -> Result<Vec<CitationUsage>, ValidationError> {
    // Walk all .rs files under root.
    // Regex: LegalCitation\s*\{\s*article_id:\s*"([^"]+)"\s*,\s*corpus_sha:\s*"([^"]+)"
    use regex::Regex;
    let re = Regex::new(
        r#"LegalCitation\s*\{\s*article_id:\s*"([^"]+)"\s*,\s*corpus_sha:\s*"([^"]+)""#
    ).unwrap();

    let mut usages = vec![];
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = std::fs::read_to_string(entry.path())
            .map_err(|e| ValidationError::IoError(e.to_string()))?;
        for (lineno, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                usages.push(CitationUsage {
                    file: entry.path().to_path_buf(),
                    line: lineno + 1,
                    article_id: caps[1].to_string(),
                    corpus_sha: caps[2].to_string(),
                });
            }
        }
    }
    Ok(usages)
}

#[derive(Debug)]
pub struct ValidationReport {
    pub total_citations: usize,
    pub violations: Vec<ValidationViolation>,
    pub corpus_lock_version: String,
}

#[derive(Debug)]
pub enum ValidationViolation {
    UnknownArticle { file: PathBuf, line: usize, article_id: String },
    StaleSha { file: PathBuf, line: usize, article_id: String, found_sha: String, expected_sha: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("corpus.lock not found: {0}")]
    LockFileNotFound(String),
    #[error("corpus.lock invalid TOML: {0}")]
    LockFileInvalid(String),
    #[error("IO error: {0}")]
    IoError(String),
}

// CLI entry point for `oya gate validate corpus-citations`
fn main() {
    let corpus_lock = std::path::Path::new("corpus.lock");
    let crates_root = std::path::Path::new("crates");
    match validate_corpus_citations(corpus_lock, crates_root) {
        Ok(report) if report.violations.is_empty() => {
            println!(
                "corpus-citations: OK — {} citations, all match corpus.lock v{}",
                report.total_citations, report.corpus_lock_version
            );
            std::process::exit(0);
        }
        Ok(report) => {
            eprintln!("corpus-citations: FAIL — {} violations:", report.violations.len());
            for v in &report.violations {
                eprintln!("  {:?}", v);
            }
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("corpus-citations: ERROR — {e}");
            std::process::exit(2);
        }
    }
}
```

---

## 5. Audit Chain Verification Script

```rust
// tests/acceptance/gates/audit_chain_verify.rs
// Verifies Ed25519 seals on all append-only log tables across all µservices.
// Tables checked: hr.employee_audit_log, payroll.run_transitions (via audit_hash),
//   accounting.journal_entries, connect_pro.message_audit_log,
//   application.shell_audit_log, workflow.transitions

use ed25519_dalek::{VerifyingKey, Signature, Verifier};

pub struct AuditChainVerifyConfig {
    pub pg_url: String,
    pub verifying_key_hex: String,     // tenant Ed25519 verifying key (hex)
    pub tenant_id: uuid::Uuid,
    pub sample_size: usize,            // how many tail events to verify per table
}

pub async fn verify_all_audit_chains(
    config: &AuditChainVerifyConfig,
) -> Result<AuditChainReport, AuditChainError> {
    let pool = sqlx::PgPool::connect(&config.pg_url).await
        .map_err(|e| AuditChainError::Database(e.to_string()))?;
    let vk_bytes = hex::decode(&config.verifying_key_hex)
        .map_err(|e| AuditChainError::KeyDecode(e.to_string()))?;
    let verifying_key = VerifyingKey::from_bytes(
        vk_bytes.as_slice().try_into().map_err(|_| AuditChainError::KeyDecode("bad length".into()))?
    ).map_err(|e| AuditChainError::KeyDecode(e.to_string()))?;

    let tables = [
        ("application", "shell_audit_log", "event_hash", "payload"),
        ("workflow", "transitions", "event_hash", "output"),
        ("connect_pro", "message_audit_log", "event_hash", "payload"),
    ];

    let mut results = vec![];
    for (schema, table, hash_col, payload_col) in &tables {
        let count = verify_table(
            &pool,
            &verifying_key,
            schema,
            table,
            hash_col,
            payload_col,
            config.tenant_id,
            config.sample_size,
        ).await?;
        results.push(TableVerifyResult {
            table: format!("{}.{}", schema, table),
            verified_count: count,
            violations: 0,
        });
    }

    Ok(AuditChainReport { tables: results })
}

async fn verify_table(
    pool: &sqlx::PgPool,
    vk: &VerifyingKey,
    schema: &str,
    table: &str,
    hash_col: &str,
    payload_col: &str,
    tenant_id: uuid::Uuid,
    limit: usize,
) -> Result<usize, AuditChainError> {
    let sql = format!(
        "SELECT {hash_col}, {payload_col}::text FROM {schema}.{table}
         WHERE tenant_id = $1
         ORDER BY created_at DESC LIMIT $2"
    );
    let rows = sqlx::query(&sql)
        .bind(tenant_id)
        .bind(limit as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| AuditChainError::Database(e.to_string()))?;

    for row in &rows {
        use sqlx::Row;
        let sig_bytes: Vec<u8> = row.try_get(0)
            .map_err(|e| AuditChainError::Database(e.to_string()))?;
        let payload: String = row.try_get(1)
            .map_err(|e| AuditChainError::Database(e.to_string()))?;
        let sig = Signature::from_slice(&sig_bytes)
            .map_err(|e| AuditChainError::SignatureDecode(e.to_string()))?;
        vk.verify(payload.as_bytes(), &sig)
            .map_err(|_| AuditChainError::SignatureInvalid {
                table: format!("{schema}.{table}"),
            })?;
    }
    Ok(rows.len())
}

#[derive(Debug, serde::Serialize)]
pub struct AuditChainReport {
    pub tables: Vec<TableVerifyResult>,
}

#[derive(Debug, serde::Serialize)]
pub struct TableVerifyResult {
    pub table: String,
    pub verified_count: usize,
    pub violations: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum AuditChainError {
    #[error("database: {0}")]
    Database(String),
    #[error("key decode: {0}")]
    KeyDecode(String),
    #[error("signature decode: {0}")]
    SignatureDecode(String),
    #[error("signature invalid on table {table}")]
    SignatureInvalid { table: String },
}
```

---

## 6. Evidence Bundle Structure + Generator

```rust
// tests/acceptance/evidence/generate_evidence_bundle.rs
// Generates evidence/m3-acceptance-bundle-YYYYMMDD.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct M3EvidenceBundle {
    pub generated_at: String,
    pub bundle_version: String,
    pub tenant_id: String,
    pub adr_0210_closure: Adr0210Closure,
    pub load_test_results: HashMap<String, LoadTestResult>,
    pub edi_evidence: EdiEvidence,
    pub year_end_evidence: YearEndEvidence,
    pub legal_hold_evidence: LegalHoldEvidence,
    pub slo_evidence: SloEvidence,
    pub corpus_citation_evidence: CorpusCitationEvidence,
    pub audit_chain_evidence: AuditChainEvidence,
    pub restore_drill_evidence: Option<RestoreDrillEvidence>,
}

#[derive(Debug, Serialize)]
pub struct Adr0210Closure {
    /// One paying KR group tenant active and running payroll.
    pub paying_tenant_active: bool,
    pub tenant_activated_at: Option<String>,
    /// 4대보험 EDI green (NPS + NHIS + MOEL all submitted, no rejections).
    pub edi_all_green: bool,
    pub edi_last_submission_at: Option<String>,
    /// 연말정산 sealed (all 21-category deductions computed, PDF generated, submitted).
    pub year_end_settlement_sealed: bool,
    pub year_end_settlement_tax_year: Option<i32>,
    /// Legal hold verified (hold placed, four-eyes release gate exercised).
    pub legal_hold_verified: bool,
    /// 7-day SLO evidence window captured.
    pub slo_7day_window_captured: bool,
    pub slo_availability_7d: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct LoadTestResult {
    pub scenario: String,
    pub p99_ms: f64,
    pub threshold_ms: f64,
    pub passed: bool,
    pub run_at: String,
    pub k6_summary_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EdiEvidence {
    pub nps_submission_id: String,
    pub nhis_submission_id: String,
    pub moel_submission_id: String,
    pub all_accepted: bool,
    pub rejection_count: u32,
    pub submitted_at: String,
}

#[derive(Debug, Serialize)]
pub struct YearEndEvidence {
    pub tax_year: i32,
    pub settlement_id: String,
    pub employee_count: u32,
    pub total_refund_amount: i64,      // KRW
    pub total_additional_tax: i64,     // KRW
    pub pdf_object_key: String,
    pub sealed_at: String,
    pub audit_hash: String,
}

#[derive(Debug, Serialize)]
pub struct LegalHoldEvidence {
    pub hold_id: String,
    pub placed_at: String,
    pub release_attempted_same_user: bool,  // Must be false (four-eyes)
    pub release_approved_by: String,
    pub released_at: String,
    pub messages_preserved_count: u64,
    pub export_verified: bool,
}

#[derive(Debug, Serialize)]
pub struct SloEvidence {
    pub window_start: String,
    pub window_end: String,
    pub availability_pct: f64,
    pub payroll_run_p99_s: f64,
    pub payslip_read_p99_ms: f64,
    pub shell_frame_p99_ms: f64,
    pub edi_submission_p99_ms: f64,
    pub all_slos_met: bool,
}

#[derive(Debug, Serialize)]
pub struct CorpusCitationEvidence {
    pub corpus_lock_version: String,
    pub total_citations: usize,
    pub violations: usize,
    pub gate_passed: bool,
    pub validated_at: String,
}

#[derive(Debug, Serialize)]
pub struct AuditChainEvidence {
    pub tables_verified: Vec<String>,
    pub total_events_verified: usize,
    pub violations: usize,
    pub gate_passed: bool,
    pub verified_at: String,
}

#[derive(Debug, Serialize)]
pub struct RestoreDrillEvidence {
    pub drill_date: String,
    pub backup_point: String,
    pub restore_completed_at: String,
    pub rto_minutes: u32,
    pub rpo_minutes: u32,
    pub rto_met: bool,  // ≤ 240 min
    pub rpo_met: bool,  // ≤ 60 min
    pub data_integrity_checks_passed: bool,
    pub kafka_replay_verified: bool,
}
```

---

## 7. ADR-0210 M3 Closure Checklist

```
M3 CLOSURE CHECKLIST (ADR-0210)

[ ] PAYING-01: At least 1 KR group tenant has status = 'active' in application.tenant_shell
[ ] PAYING-02: Tenant has processed ≥ 1 payroll run with ≥ 1 employee
[ ] PAYING-03: Tenant billing_plan != 'trial' (paid subscription confirmed)

[ ] EDI-01: NPS acquisition EDI submitted and accepted (no rejection code)
[ ] EDI-02: NHIS acquisition EDI submitted and accepted
[ ] EDI-03: MOEL acquisition EDI submitted and accepted
[ ] EDI-04: EDI format validates against 더존 iCUBE reference schema (edi_format_validator test)
[ ] EDI-05: All EDI submissions stored as immutable rows (no UPDATE on edi_submissions)

[ ] YEAR-END-01: 연말정산 run completed for current/most-recent tax year
[ ] YEAR-END-02: All 21 deduction categories computed (소득공제 14 + 세액공제 7)
[ ] YEAR-END-03: Settlement PDF generated and stored (payslips.pdf_object_key populated)
[ ] YEAR-END-04: audit_hash sealed with Ed25519 on settlement record

[ ] LEGAL-HOLD-01: Legal hold placed via POST /api/connect/legal-holds
[ ] LEGAL-HOLD-02: Held messages cannot be deleted (prevent_held_message_deletion trigger verified)
[ ] LEGAL-HOLD-03: Four-eyes release: release approved by user ≠ initiator ≠ primary approver
[ ] LEGAL-HOLD-04: PST/MBOX export generated for held messages
[ ] LEGAL-HOLD-05: Legal hold audit trail stored in connect_pro.message_audit_log

[ ] SLO-01: 7-day availability window captured: ≥ 99.9%
[ ] SLO-02: Payroll run p99 ≤ 30s for 3k-person shape (k6 evidence)
[ ] SLO-03: Payslip read p99 ≤ 50ms (k6 evidence)
[ ] SLO-04: Shell frame p99 ≤ 100ms at 10k concurrent sessions (k6 evidence)
[ ] SLO-05: EDI submission p99 ≤ 200ms (k6 evidence)

[ ] CORPUS-01: corpus-citations gate passes: 0 violations
[ ] CORPUS-02: All KR statute citations pinned to corpus.lock (ADR-0190)

[ ] AUDIT-01: Ed25519 audit chain verified on all append-only log tables
[ ] AUDIT-02: Transition log append-only rule tested (UPDATE/DELETE return 0 rows)

[ ] RESTORE-01: Restore drill executed; RTO ≤ 4h evidence captured
[ ] RESTORE-02: RPO ≤ 1h verified against backup timestamps
[ ] RESTORE-03: Tenant data integrity checks pass post-restore

[ ] ONBOARD-01: Sub-5-minute tenant activation documented (Playwright E2E screenshot)
[ ] ONBOARD-02: EmployeeHired → TenantUserProvisioned idempotency test passes
```

---

## 8. k6 Composite Runner Script

```bash
#!/usr/bin/env bash
# tests/acceptance/run-all.sh
# Execute all acceptance load tests in sequence; collect results into evidence bundle.

set -euo pipefail

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_DIR="tests/acceptance/evidence/results-${TIMESTAMP}"
mkdir -p "$RESULTS_DIR"

BASE_URL="${BASE_URL:-https://api.oyatie.local}"
APP_URL="${APP_URL:-https://app.oyatie.local}"
TENANT_TOKEN="${TENANT_TOKEN}"
TENANT_ID="${TENANT_ID}"
PAYROLL_PERIOD_ID="${PAYROLL_PERIOD_ID}"

echo "=== Oyatie M3 Acceptance Load Tests ==="
echo "Tenant: $TENANT_ID"
echo "Results: $RESULTS_DIR"
echo ""

run_k6() {
  local name="$1"
  local script="$2"
  shift 2
  echo ">>> Running: $name"
  k6 run \
    --out "json=${RESULTS_DIR}/${name}.json" \
    --env BASE_URL="$BASE_URL" \
    --env TENANT_TOKEN="$TENANT_TOKEN" \
    --env TENANT_ID="$TENANT_ID" \
    "$@" \
    "$script" \
    && echo "    PASSED: $name" \
    || { echo "    FAILED: $name"; exit 1; }
}

run_k6 "payroll-run-3k" \
  "tests/acceptance/k6/payroll-run-3k.k6.js" \
  --env PAYROLL_PERIOD_ID="$PAYROLL_PERIOD_ID"

run_k6 "payslip-read" \
  "tests/acceptance/k6/payslip-read.k6.js" \
  --env PAYSLIP_IDS_FILE="$RESULTS_DIR/payslip_ids.txt"

run_k6 "shell-frame-10k" \
  "tests/acceptance/k6/shell-frame-10k.k6.js" \
  --env BASE_URL="$APP_URL" \
  --env SESSION_COOKIE="$SESSION_COOKIE"

run_k6 "edi-submission" \
  "tests/acceptance/k6/edi-submission.k6.js" \
  --env PAYROLL_PERIOD_ID="$PAYROLL_PERIOD_ID"

run_k6 "legal-hold" \
  "tests/acceptance/k6/legal-hold.k6.js" \
  --env TEST_MAILBOX_ID="$TEST_MAILBOX_ID" \
  --env LEGAL_ADMIN_USER_ID="$LEGAL_ADMIN_USER_ID"

echo ""
echo "=== Corpus Citation Gate ==="
cargo run --bin oya-gate -- validate corpus-citations \
  && echo "    PASSED: corpus-citations" \
  || { echo "    FAILED: corpus-citations"; exit 1; }

echo ""
echo "=== Audit Chain Verification ==="
cargo test --test audit_chain_verify -- --nocapture \
  && echo "    PASSED: audit-chain" \
  || { echo "    FAILED: audit-chain"; exit 1; }

echo ""
echo "=== Generating Evidence Bundle ==="
cargo run --bin evidence-generator -- \
  --results-dir "$RESULTS_DIR" \
  --tenant-id "$TENANT_ID" \
  --output "tests/acceptance/evidence/m3-acceptance-bundle-${TIMESTAMP}.json"

echo ""
echo "=== ALL ACCEPTANCE GATES PASSED ==="
echo "Evidence bundle: tests/acceptance/evidence/m3-acceptance-bundle-${TIMESTAMP}.json"
```

---

## 9. Acceptance Gates

```
GATE M3-01: Payroll run for 3k employees completes ≤ 30s (k6 payroll-run-3k)
GATE M3-02: Payslip read p99 ≤ 50ms at 1k RPS (k6 payslip-read)
GATE M3-03: Shell frame p99 ≤ 100ms at 10k concurrent sessions (k6 shell-frame-10k)
GATE M3-04: EDI submission p99 ≤ 200ms (k6 edi-submission)
GATE M3-05: Legal hold placed + verified + four-eyes release passes (k6 legal-hold)
GATE M3-06: 4대보험 EDI format validates against 더존 iCUBE reference (unit test)
GATE M3-07: 연말정산 21-category deductions computed; PDF generated; audit_hash sealed
GATE M3-08: corpus-citations gate: 0 violations (oya gate validate corpus-citations)
GATE M3-09: Ed25519 audit chain verified across all append-only log tables
GATE M3-10: 7-day SLO availability ≥ 99.9% (Prometheus query over 7d window)
GATE M3-11: Restore drill: RTO ≤ 4h AND RPO ≤ 1h (runbook executed, evidence JSON captured)
GATE M3-12: Evidence bundle JSON generated and all passed fields = true
GATE M3-13: Grit symbols all claimed and grit done ceremony complete
```

---

## 10. Grit Claim Symbols + Done Ceremony

```
grit session start m03-p08-evidence-2026-05-13
grit claim m03.p08.evidence.load-tests
grit claim m03.p08.evidence.slo
grit claim m03.p08.evidence.corpus
grit claim m03.p08.evidence.restore-drill
grit claim m03.p08.evidence.bundle

# After all 8 phases implemented and evidence collected:
grit claim m03.milestone.complete
grit done --agent m03-p08-evidence-2026-05-13

# M3 milestone done ceremony:
grit session start m03-milestone-close-2026-05-13
grit claim m03.milestone.kr-group-tenant-live
grit claim m03.milestone.edi-green
grit claim m03.milestone.year-end-sealed
grit claim m03.milestone.legal-hold-verified
grit claim m03.milestone.slo-7day-captured
grit done --agent m03-milestone-close-2026-05-13
```

---

## 11. ICM Payload

```bash
icm store \
  -t context-oyatie \
  -c "M03-P08 KR acceptance evidence impl-plan complete: k6 load tests for payroll-run-3k (≤30s), payslip-read (p99≤50ms), shell-frame-10k (p99≤100ms), edi-submission (p99≤200ms), legal-hold; Prometheus SLO burn-rate rules + Grafana M3 dashboard; restore drill runbook (RTO≤4h, RPO≤1h); corpus-citation audit gate (Rust scanner against corpus.lock); Ed25519 audit-chain verifier; evidence bundle JSON structure; ADR-0210 M3 closure checklist; grit done ceremony" \
  -i high \
  -k "acceptance,evidence,kr,payroll,edi,year-end,legal-hold,slo,restore-drill,corpus,audit-chain,m3-closure"
```

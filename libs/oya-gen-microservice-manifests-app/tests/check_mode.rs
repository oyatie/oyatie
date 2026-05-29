#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use oya_gen_microservice_manifests_app::MICROSERVICES;
use serde_json::Value;

fn temp_repo() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "oya-gen-microservice-manifests-check-mode-{}-{nonce}",
        std::process::id()
    ))
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write fixture");
}

#[test]
fn check_mode_rejects_ops_capability_and_slo_source_drift() {
    let repo = temp_repo();
    fs::create_dir_all(repo.join("docs/decisions")).expect("create docs decisions");
    fs::create_dir_all(repo.join("specs/microservices")).expect("create specs");
    for ms in MICROSERVICES {
        fs::create_dir_all(repo.join(format!("microservices/{ms}"))).expect("create ms dir");
    }

    write(
        &repo.join("microservices/ops-dashboard-control-center/capabilities/rollback-execute.yaml"),
        "schema_version: \"1.0\"\nname: rollback-execute\ntier: T3\neu_ai_act_risk_class: high\n",
    );
    write(
        &repo.join("microservices/ops-dashboard-control-center/slos/rollback.openslo.yaml"),
        r#"apiVersion: openslo/v1
kind: SLO
metadata:
  name: oya-ops-rollback
spec:
  objective:
    target: "0.95"
  indicator:
    ratioMetric:
      good:
        metricSource:
          spec:
            query: 'histogram_quantile(0.95, sum(rate(oya_ops_rollback_bucket[5m])) by (le))'
"#,
    );

    let bin = env!("CARGO_BIN_EXE_oya-gen-microservice-manifests");
    let seed = Command::new(bin)
        .arg("--repo-root")
        .arg(&repo)
        .output()
        .expect("run seed");
    assert!(
        seed.status.success(),
        "{}",
        String::from_utf8_lossy(&seed.stderr)
    );

    let manifest_path = repo.join("microservices/ops-dashboard-control-center/manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["capabilities"][0]["tier"] = Value::String("T1".to_string());
    manifest["capabilities"][0]["eu_ai_act_risk_class"] = Value::String("minimal".to_string());
    manifest["slos"][0]["sli"] = Value::String("sum(rate(wrong_total[5m]))".to_string());
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap() + "\n",
    )
    .unwrap();

    let check = Command::new(bin)
        .arg("--repo-root")
        .arg(&repo)
        .arg("--check")
        .output()
        .expect("run check");
    assert!(
        !check.status.success(),
        "check unexpectedly passed: stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(stderr.contains("capabilities.microservices/ops-dashboard-control-center/capabilities/rollback-execute.yaml.tier mismatch"), "{stderr}");
    assert!(stderr.contains("capabilities.microservices/ops-dashboard-control-center/capabilities/rollback-execute.yaml.eu_ai_act_risk_class mismatch"), "{stderr}");
    assert!(stderr.contains("slos.microservices/ops-dashboard-control-center/slos/rollback.openslo.yaml.sli mismatch"), "{stderr}");

    fs::remove_dir_all(repo).ok();
}

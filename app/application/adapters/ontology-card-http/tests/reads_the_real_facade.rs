//! The wedge-serving proof: the real `foundry-ontology-app` listener, booted
//! in-process the way its own suites boot it, answers the adapter.

use std::path::PathBuf;

use application_ontology_card::{OntologyCardError, OntologyCardSource};
use application_ontology_card_http::HttpOntologyCardSource;
use foundry_ontology_app::{Config, OperatorCredential, compose, router};

const TENANT: &str = "ten_acme";
const TOKEN: &str = "operator-token-for-the-console";

struct Files(Vec<PathBuf>);

impl Files {
    fn new() -> Self {
        let stamp = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the epoch")
                .as_nanos()
        );
        let temp = std::env::temp_dir();
        let files = Self(
            ["action", "denial", "store"]
                .iter()
                .map(|slot| temp.join(format!("console-ontology-card-{slot}-{stamp}.sqlite")))
                .collect(),
        );
        files.remove();
        files
    }

    fn remove(&self) {
        for path in &self.0 {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
            }
        }
    }

    fn config(&self) -> Config {
        Config {
            listen_addr: "127.0.0.1:0".into(),
            action_log: self.0[0].clone(),
            denial_log: self.0[1].clone(),
            projection_store: self.0[2].clone(),
            tenants: vec![TENANT.into()],
            operators: vec![OperatorCredential {
                token: TOKEN.into(),
                tenant_id: TENANT.into(),
                principal_id: "prn_console".into(),
                roles: vec!["foundry-operator".into()],
            }],
        }
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        self.remove();
    }
}

async fn serve(files: &Files) -> String {
    let state = compose(&files.config()).expect("the facade boots");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("loopback binds");
    let base = format!("http://{}", listener.local_addr().expect("bound address"));
    tokio::spawn(async move {
        let _ = axum::serve(listener, router(state)).await;
    });
    base
}

async fn read(
    base: &str,
    token: &str,
) -> Result<application_ontology_card::OntologyCardFacts, OntologyCardError> {
    let source = HttpOntologyCardSource::new(base, token).expect("a loopback url and a token");
    tokio::task::spawn_blocking(move || source.ontology_card_facts())
        .await
        .expect("the read does not panic")
}

#[tokio::test]
async fn the_adapter_answers_what_the_facade_serves_on_statusz() {
    let files = Files::new();
    let base = serve(&files).await;
    let served: serde_json::Value = reqwest::Client::new()
        .get(format!("{base}/statusz"))
        .bearer_auth(TOKEN)
        .send()
        .await
        .expect("the facade answers")
        .json()
        .await
        .expect("statusz is json");

    let facts = read(&base, TOKEN)
        .await
        .expect("the adapter reads the facade");

    assert_eq!(
        Some(facts.policy_version.as_str()),
        served["policy_version"].as_str()
    );
    assert_eq!(
        Some(facts.served_tenants),
        served["served_tenants"].as_u64()
    );
    assert_eq!(
        Some(facts.projection_lag),
        served["projection_lag"].as_u64()
    );
    assert_eq!(
        Some(facts.poisoned_entries),
        served["poisoned_entries"].as_u64()
    );
    assert_eq!(facts.served_tenants, 1, "{served}");
    assert!(!facts.policy_version.is_empty(), "{served}");
}

#[tokio::test]
async fn a_credential_the_facade_does_not_recognize_is_unavailable() {
    let files = Files::new();
    let base = serve(&files).await;

    let refusal = read(&base, "not-an-operator").await;

    match refusal {
        Err(OntologyCardError::Unavailable(reason)) => assert!(reason.contains("401"), "{reason}"),
        Ok(facts) => panic!("a stranger read {facts:?}"),
    }
}

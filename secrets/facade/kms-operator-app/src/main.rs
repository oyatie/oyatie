#![forbid(unsafe_code)]

use std::path::Path;
use std::process::ExitCode;

use secrets_kms_operator_app::{OperatorStateStoreConfig, SystemClock, default_operator_backoff};
use secrets_kms_operator_k8s::{
    DomainKmsOperatorActuator, KubeOperatorRuntime, PersistentCloudKmsDirectory,
};
use tracing::error;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_or("RUST_LOG", "info"))
        .init();

    let namespace = env_or("OYATIE_KMS_OPERATOR_NAMESPACE", "cloud-kms");
    let mtls = OperatorMtlsFiles::from_env();
    if let Err(message) = mtls.validate() {
        error!(error = %message, "cloud-kms operator mTLS startup validation failed");
        return ExitCode::FAILURE;
    }
    let state_store = match OperatorStateStoreConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            error!(error = %error, "cloud-kms operator state startup validation failed");
            return ExitCode::FAILURE;
        }
    };

    let client = match kube::Client::try_default().await {
        Ok(client) => client,
        Err(error) => {
            error!(error = %error, "failed to build kube client");
            return ExitCode::FAILURE;
        }
    };
    let repo = match PersistentCloudKmsDirectory::open(&state_store.path) {
        Ok(repo) => repo,
        Err(error) => {
            error!(error = %error, "failed to open cloud-kms operator state store");
            return ExitCode::FAILURE;
        }
    };
    let actuator = DomainKmsOperatorActuator::new(repo);
    let runtime = KubeOperatorRuntime::new(
        client,
        namespace,
        actuator,
        SystemClock,
        default_operator_backoff(),
    );

    runtime.run().await;
    ExitCode::SUCCESS
}

struct OperatorMtlsFiles {
    ca_path: String,
    cert_path: String,
    key_path: String,
}

impl OperatorMtlsFiles {
    fn from_env() -> Self {
        let cert_dir = env_or(
            "OYATIE_KMS_OPERATOR_MTLS_CERT_DIR",
            "/etc/secrets-kms-operator/tls",
        );
        let ca_default = format!("{cert_dir}/ca.crt");
        let cert_default = format!("{cert_dir}/tls.crt");
        let key_default = format!("{cert_dir}/tls.key");
        Self {
            ca_path: env_or("OYATIE_KMS_OPERATOR_MTLS_CA_PATH", &ca_default),
            cert_path: env_or("OYATIE_KMS_OPERATOR_MTLS_CERT_PATH", &cert_default),
            key_path: env_or("OYATIE_KMS_OPERATOR_MTLS_KEY_PATH", &key_default),
        }
    }

    fn validate(&self) -> Result<(), String> {
        for path in [&self.ca_path, &self.cert_path, &self.key_path] {
            if !Path::new(path).is_file() {
                return Err(format!("required mTLS file missing: {path}"));
            }
        }
        Ok(())
    }
}

fn env_or(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(value) if !value.is_empty() => value,
        _ => default.to_owned(),
    }
}

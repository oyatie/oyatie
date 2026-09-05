use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use shared_audit_digest_adapter_awslc::Ed25519ChainSigner;
use shared_audit_event_kernel::{ChainSigner, DigestChainError, encode_hex};

use super::*;

struct Fixture {
    directory: PathBuf,
    store: FilePolicyBundleStore,
    signer: Ed25519ChainSigner,
}

impl Fixture {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "policy-bundle-publication-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let trust = directory.join("trust");
        fs::create_dir(&trust).unwrap();
        let signer = Ed25519ChainSigner::generate("publication-key").unwrap();
        fs::write(
            trust.join("publication-key.pub"),
            encode_hex(&signer.public_key_bytes()),
        )
        .unwrap();
        let store = FilePolicyBundleStore::new(directory.join("bundle.json"), trust);
        fs::write(store.path(), b"previous bundle bytes").unwrap();
        Self {
            directory,
            store,
            signer,
        }
    }

    fn assert_previous_unchanged(&self) {
        assert_eq!(
            fs::read(self.store.path()).unwrap(),
            b"previous bundle bytes"
        );
        self.assert_no_staging();
    }

    fn assert_no_staging(&self) {
        let mut entries = fs::read_dir(&self.directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        entries.sort();
        assert_eq!(entries, vec!["bundle.json", "trust"]);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn bundle() -> PolicyBundle {
    PolicyBundle {
        version: PolicyVersion::new("publication-content").unwrap(),
        schema_src: "authored schema\n".to_owned(),
        policies_src: "authored policy with \"quotes\"\n".to_owned(),
        tenant_policies: BTreeMap::from([("tenant".to_owned(), "overlay\n".to_owned())]),
        templates: vec![],
        template_links: vec![],
        action_map: BTreeMap::new(),
    }
}

struct RecordingSigner<'a> {
    inner: &'a Ed25519ChainSigner,
    message: Mutex<Vec<u8>>,
}

impl ChainSigner for RecordingSigner<'_> {
    fn key_id(&self) -> &str {
        self.inner.key_id()
    }

    fn sign_hex(&self, message: &[u8]) -> Result<String, DigestChainError> {
        *self.message.lock().unwrap() = message.to_vec();
        self.inner.sign_hex(message)
    }
}

struct RefusingSigner;

impl ChainSigner for RefusingSigner {
    fn key_id(&self) -> &str {
        "publication-key"
    }

    fn sign_hex(&self, _: &[u8]) -> Result<String, DigestChainError> {
        Err(DigestChainError::SigningFailed(
            "custody unavailable".to_owned(),
        ))
    }
}

#[test]
fn published_bundle_loads_through_existing_verified_reader() {
    let fixture = Fixture::new();
    fixture
        .store
        .write_signed_bundle(
            &bundle(),
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    assert_eq!(fixture.store.load().unwrap(), bundle());
    fixture.assert_no_staging();
}

#[test]
fn publication_preserves_the_exact_bytes_given_to_signer() {
    let fixture = Fixture::new();
    let signer = RecordingSigner {
        inner: &fixture.signer,
        message: Mutex::new(Vec::new()),
    };
    fixture
        .store
        .write_signed_bundle(&bundle(), &signer, &fixture.signer.public_key_bytes())
        .unwrap();
    let doc: SignedPolicyBundleDoc =
        serde_json::from_slice(&fs::read(fixture.store.path()).unwrap()).unwrap();
    assert_eq!(
        doc.bundle.as_bytes(),
        signer.message.lock().unwrap().as_slice()
    );
    assert_eq!(fixture.store.load().unwrap(), bundle());
}

#[test]
fn signing_refusal_preserves_previous_bundle_without_staging() {
    let fixture = Fixture::new();
    let result = fixture.store.write_signed_bundle(
        &bundle(),
        &RefusingSigner,
        &fixture.signer.public_key_bytes(),
    );
    assert!(matches!(result, Err(BundlePublishError::Signing(_))));
    fixture.assert_previous_unchanged();
}

#[test]
fn untrusted_signer_preserves_previous_bundle_without_staging() {
    let fixture = Fixture::new();
    let untrusted = Ed25519ChainSigner::generate("unregistered-key").unwrap();
    let result =
        fixture
            .store
            .write_signed_bundle(&bundle(), &untrusted, &untrusted.public_key_bytes());
    assert!(matches!(result, Err(BundlePublishError::Verification(_))));
    fixture.assert_previous_unchanged();
}

#[test]
fn missing_trust_anchor_preserves_previous_bundle_without_staging() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.store.trust_dir().join("publication-key.pub")).unwrap();
    let result = fixture.store.write_signed_bundle(
        &bundle(),
        &fixture.signer,
        &fixture.signer.public_key_bytes(),
    );
    assert!(matches!(result, Err(BundlePublishError::Verification(_))));
    fixture.assert_previous_unchanged();
}

#[test]
fn staging_sync_failure_preserves_previous_bytes_and_cleans_staging() {
    let fixture = Fixture::new();
    let result = publication::replace_atomically(fixture.store.path(), b"replacement", |_| {
        Err(io::Error::other("injected staging sync refusal"))
    });
    assert!(matches!(
        result,
        Err(BundlePublishError::BeforeCommit { .. })
    ));
    fixture.assert_previous_unchanged();
}

#[test]
fn directory_sync_failure_reports_commit_without_claiming_rollback() {
    let fixture = Fixture::new();
    let result = publication::replace_atomically(fixture.store.path(), b"replacement", |file| {
        if file.metadata()?.is_dir() {
            Err(io::Error::other("injected directory sync refusal"))
        } else {
            File::sync_all(file)
        }
    });
    assert!(matches!(
        result,
        Err(BundlePublishError::CommittedButDurabilityUnknown { .. })
    ));
    assert_eq!(fs::read(fixture.store.path()).unwrap(), b"replacement");
    fixture.assert_no_staging();
}

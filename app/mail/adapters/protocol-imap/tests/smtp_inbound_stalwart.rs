#![cfg(feature = "upstream-tests")]
// Unchanged pinned upstream SMTP inbound suites compile only into this
// harness; their `#[tokio::test]` functions register as this binary's tests.
// Crate aliases resolve the upstream `smtp`, `common`, `registry` and
// `mail_auth` paths onto the shim; `assert!`/`assert_eq!` are shadowed so
// every predicate — wire or session-state — is recorded and judged against
// the pinned waivers instead of aborting the suite.
extern crate self as common;
extern crate self as mail_auth;
extern crate self as registry;
extern crate self as smtp;
#[path = "stalwart/registry.rs"]
mod registry_shim;
#[path = "stalwart/smtp_shim.rs"]
mod shim;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

const REVISION: &str = "474dd0229cb20cf513036619781ed97bd8073c3f";
/// `(suite, predicate)` refused by decision. None yet: both bound suites
/// pass on the wire and on the mirrored session state.
const WAIVED: &[(&str, &str)] = &[];

static OUTCOMES: Mutex<Vec<serde_json::Value>> = Mutex::new(Vec::new());
thread_local! {
    static SUITE: std::cell::RefCell<&'static str> = const { std::cell::RefCell::new("") };
}

/// Every predicate lands here: printed, kept for the summary, and fatal
/// only when it fails unwaived or passes while waived.
pub fn record(predicate: &str, passed: bool, detail: String) {
    let suite = SUITE.with(|s| *s.borrow());
    let waived = WAIVED.contains(&(suite, predicate));
    let value = json!({"suite":suite,"predicate":predicate,"passed":passed,"waived":waived,"detail":detail});
    println!("{value}");
    OUTCOMES.lock().unwrap().push(value);
    std::assert!(
        passed || waived,
        "{suite}: unwaived predicate failed: {predicate}: {detail}"
    );
    std::assert!(
        !(passed && waived),
        "{suite}: waived predicate passed; remove it from WAIVED: {predicate}"
    );
}

/// The suite a predicate belongs to: named by the upstream builder.
pub fn enter(suite: &'static str) {
    SUITE.with(|s| *s.borrow_mut() = suite);
}

macro_rules! assert {
    ($cond:expr $(,)?) => {
        $crate::record(stringify!($cond), $cond, String::new())
    };
    ($cond:expr, $($arg:tt)+) => {
        $crate::record(stringify!($cond), $cond, format!($($arg)+))
    };
}
macro_rules! assert_eq {
    ($left:expr, $right:expr $(,)?) => {{
        let (l, r) = (&$left, &$right);
        $crate::record(
            concat!(stringify!($left), " == ", stringify!($right)),
            *l == *r,
            format!("{l:?} vs {r:?}"),
        )
    }};
    ($left:expr, $right:expr, $($arg:tt)+) => {{
        let (l, r) = (&$left, &$right);
        $crate::record(
            concat!(stringify!($left), " == ", stringify!($right)),
            *l == *r,
            format!($($arg)+),
        )
    }};
}

/// Upstream's response assertions, recorded instead of panicking.
pub trait VerifyResponse {
    fn assert_code(self, expected_code: &str) -> Self;
    fn assert_contains(self, expected_text: &str) -> Self;
    fn assert_not_contains(self, expected_text: &str) -> Self;
    fn assert_count(self, text: &str, occurrences: usize) -> Self;
}
impl VerifyResponse for Vec<String> {
    fn assert_code(self, expected_code: &str) -> Self {
        let passed = self.last().is_some_and(|l| l.starts_with(expected_code));
        record(&format!("code {expected_code}"), passed, self.join(" | "));
        self
    }
    fn assert_contains(self, expected_text: &str) -> Self {
        let passed = self.iter().any(|l| l.contains(expected_text));
        record(
            &format!("contains {expected_text}"),
            passed,
            self.join(" | "),
        );
        self
    }
    fn assert_not_contains(self, expected_text: &str) -> Self {
        let passed = !self.iter().any(|l| l.contains(expected_text));
        record(
            &format!("not contains {expected_text}"),
            passed,
            self.join(" | "),
        );
        self
    }
    fn assert_count(self, text: &str, occurrences: usize) -> Self {
        let count = self.iter().filter(|l| l.contains(text)).count();
        record(
            &format!("count {text} == {occurrences}"),
            count == occurrences,
            format!("{count}"),
        );
        self
    }
}

/// Upstream's `TestSession` methods the bound suites call, on the shim.
pub trait TestSession {
    fn response(&mut self) -> Vec<String>;
    fn write_rx(&mut self, data: &str);
    async fn cmd(&mut self, cmd: &str, expected_code: &str) -> Vec<String>;
    async fn ehlo(&mut self, host: &str) -> Vec<String>;
}
impl TestSession for shim::Session {
    fn response(&mut self) -> Vec<String> {
        shim::Session::response(self)
    }
    fn write_rx(&mut self, data: &str) {
        shim::Session::write_rx(self, data);
    }
    async fn cmd(&mut self, cmd: &str, expected_code: &str) -> Vec<String> {
        let _ = self.ingest(format!("{cmd}\r\n").as_bytes()).await;
        self.response().assert_code(expected_code)
    }
    async fn ehlo(&mut self, host: &str) -> Vec<String> {
        let _ = self.ingest(format!("EHLO {host}\r\n").as_bytes()).await;
        self.response().assert_code("250")
    }
}

// Upstream paths onto the shim.
pub mod auth {
    pub use crate::shim::{AccountCache, AccountInfo};
}
pub use shim::SpfOutput;
pub mod core {
    pub use crate::shim::SessionAddress;
}
pub mod session {
    pub use crate::{TestSession, VerifyResponse};
}
pub mod schema {
    pub mod structs {
        pub use crate::registry_shim::{Expression, ExpressionMatch, MtaInboundSession};
    }
}
pub mod types {
    pub mod list {
        pub use crate::registry_shim::List;
    }
}
pub mod utils {
    pub mod server {
        pub use crate::fixture::TestServerBuilder;
    }
}

/// One store and service per suite; the registry holds what the suite
/// configures and `new_mta_session` evaluates it per session.
pub mod fixture {
    use crate::{
        registry_shim::{Registry, RegistryObject},
        shim::Session,
    };
    use mail_kernel::Account;
    use mail_service::{MailService, OwnerPolicy};
    use mail_sqlite_store::SqliteStore;
    use std::sync::Arc;

    pub struct TestServerBuilder;
    impl TestServerBuilder {
        pub async fn new(name: &str) -> Self {
            crate::enter(match name {
                "smtp_basic_test" => "basic",
                "smtp_inbound_limits_test" => "limits",
                _ => "unknown",
            });
            Self
        }
        pub async fn with_http_listener(self, _port: u16) -> Self {
            self
        }
        pub fn disable_services(self) -> Self {
            self
        }
        pub async fn build(self) -> TestServer {
            let db = Arc::new(SqliteStore::open(":memory:").unwrap());
            db.provision(
                Account::new("victim", "foobar", "victim", "victim@foobar.org").unwrap(),
                &"v".repeat(32),
            )
            .unwrap();
            TestServer {
                service: Arc::new(MailService {
                    outbound: None,
                    queue: db.clone(),
                    store: db.clone(),
                    identity: db,
                    policy: Arc::new(OwnerPolicy),
                }),
                registry: Arc::new(Registry::default()),
            }
        }
    }

    pub struct TestServer {
        service: Arc<MailService>,
        registry: Arc<Registry>,
    }
    pub struct AccountRef<'a>(&'a TestServer);
    impl TestServer {
        pub fn account(&self, _name: &str) -> AccountRef<'_> {
            AccountRef(self)
        }
        pub fn reload_core(&mut self) {}
        pub fn new_mta_session(&self) -> Session {
            Session::new(self.service.clone(), self.registry.clone())
        }
        pub fn new_mta_session_with_shutdown(&self) -> (Session, tokio::sync::watch::Sender<bool>) {
            (self.new_mta_session(), tokio::sync::watch::channel(false).0)
        }
    }
    impl AccountRef<'_> {
        pub async fn mta_no_auth(&self) {}
        pub async fn registry_create_object(&self, object: impl RegistryObject) {
            object.install(&self.0.registry);
        }
        pub async fn reload_settings(&self) {}
    }
}

fn digest(source: &[u8], expected: &str, suite: &str) {
    std::assert_eq!(
        format!("{:x}", Sha256::digest(source)),
        expected,
        "{suite} changed upstream; review before updating the baseline"
    );
}

mod basic {
    include!(env!("STALWART_SMTP_BASIC_SOURCE"));
}
mod limits {
    include!(env!("STALWART_SMTP_LIMITS_SOURCE"));
}

/// The pinned sources are the ones the harness was written against.
#[test]
fn upstream_sources_are_pinned() {
    digest(
        include_bytes!(env!("STALWART_SMTP_BASIC_SOURCE")),
        "c41542ca51103f5ea9d9eae8b3dfbddf0c23aaff8ce321524a58ea1098142dc4",
        "smtp/inbound/basic",
    );
    digest(
        include_bytes!(env!("STALWART_SMTP_LIMITS_SOURCE")),
        "878aff8d862e523b6ac6e2e1092766fe4add41ca01f31aeb949b53c7cb256262",
        "smtp/inbound/limits",
    );
}

/// The recorded predicates and the waived set, checked once every suite has
/// run (libtest orders by name, and this one sorts after them).
#[test]
fn waived_set_fired_exactly_and_totals_are_printed() {
    let outcomes = OUTCOMES.lock().unwrap();
    let passed = outcomes.iter().filter(|o| o["passed"] == true).count();
    let waived: Vec<_> = outcomes.iter().filter(|o| o["waived"] == true).collect();
    println!(
        "{}",
        json!({"upstream_revision":REVISION,"suites":["tests/src/smtp/inbound/basic.rs","tests/src/smtp/inbound/limits.rs"],"transport":"duplex SMTP wire","predicates":outcomes.len(),"passed":passed,"waived":waived.len()})
    );
    std::assert_eq!(waived.len(), WAIVED.len(), "every waiver must have fired");
}

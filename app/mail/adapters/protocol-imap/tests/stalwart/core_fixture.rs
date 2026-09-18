use super::client::{ImapConnection, Outcomes};
use mail_api::MetadataStore;
use mail_kernel::{Account, Command, MailboxProperties};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use std::sync::Arc;

pub const TOKEN: &str = "0123456789abcdef0123456789abcdef";
pub const JANE_TOKEN: &str = "fedcba9876543210fedcba9876543210";
pub const JDOE: &str = "jdoe@example.com";
pub const JANE: &str = "jane.smith@example.com";

fn special(name: &str, role: &str) -> Command {
    let mut properties = MailboxProperties::named(name.into());
    properties.role = Some(role.into());
    properties.is_subscribed = false;
    Command::SetMailbox {
        id: None,
        properties,
    }
}

/// Mirrors the upstream fixture: `jdoe` keeps INBOX and the Trash folder after
/// the suite deletes Drafts/Junk Mail/Sent Items; `jane` keeps every default.
pub fn open() -> (Arc<SqliteStore>, Arc<MailService>) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(Account::new("a", "t", "jdoe", JDOE).unwrap(), TOKEN)
        .unwrap();
    db.provision(Account::new("j", "t", "jane", JANE).unwrap(), JANE_TOKEN)
        .unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(0),
        vec![special("Deleted Items", "trash")],
    )
    .unwrap();
    db.execute(
        "j",
        mail_api::Precondition::Observed(0),
        vec![
            special("Deleted Items", "trash"),
            special("Drafts", "drafts"),
            special("Junk Mail", "junk"),
            special("Sent Items", "sent"),
        ],
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    (db, service)
}

pub struct TestServer {
    pub db: Arc<SqliteStore>,
    pub service: Arc<MailService>,
    pub outcomes: Outcomes,
}
pub struct AccountRef<'a> {
    server: &'a TestServer,
    address: &'static str,
    token: &'static str,
}
impl TestServer {
    pub fn account(&self, name: &str) -> AccountRef<'_> {
        let (address, token) = match name {
            JDOE => (JDOE, TOKEN),
            JANE => (JANE, JANE_TOKEN),
            other => panic!("unknown fixture account {other}"),
        };
        AccountRef {
            server: self,
            address,
            token,
        }
    }
    pub async fn wait_for_tasks(&self) {}
    pub async fn connect(&self, name: &str, tag: &'static str) -> ImapConnection {
        let account = self.account(name);
        ImapConnection::connect(
            self.service.clone(),
            self.outcomes.clone(),
            Some((account.address, account.token)),
            tag,
        )
        .await
    }
    pub async fn unauthenticated(&self, tag: &'static str) -> ImapConnection {
        ImapConnection::connect(self.service.clone(), self.outcomes.clone(), None, tag).await
    }
}
impl AccountRef<'_> {
    pub fn name(&self) -> &str {
        self.address
    }
    pub fn secret(&self) -> &str {
        self.token
    }
    /// Extra upstream connections (`other_conn`, `imap_jane`) tag with `_z `.
    pub async fn imap_client(&self) -> ImapConnection {
        ImapConnection::connect(
            self.server.service.clone(),
            self.server.outcomes.clone(),
            Some((self.address, self.token)),
            "_z ",
        )
        .await
    }
}

/// The upstream IDLE suite ingests through LMTP; local delivery into the
/// store is the equivalent for the in-process duplex transport.
pub struct SmtpConnection {
    db: Arc<SqliteStore>,
}
thread_local! {
    static INGEST: std::cell::RefCell<Option<Arc<SqliteStore>>> = const { std::cell::RefCell::new(None) };
}
impl SmtpConnection {
    pub fn install(db: &Arc<SqliteStore>) {
        INGEST.with(|cell| *cell.borrow_mut() = Some(db.clone()));
    }
    pub async fn connect() -> Self {
        let db = INGEST
            .with(|cell| cell.borrow().clone())
            .expect("fixture installed");
        Self { db }
    }
    pub async fn ingest(&mut self, _from: &str, recipients: &[&str], body: &str) {
        let recipients: Vec<String> = recipients.iter().map(|r| (*r).to_owned()).collect();
        self.db.deliver(&recipients, body.as_bytes()).unwrap();
    }
}

/// OAUTHBEARER (RFC 7628) client responses decoded by the server's SASL layer.
#[derive(Debug, PartialEq, Eq)]
pub enum Credentials {
    Bearer {
        token: String,
        username: Option<String>,
    },
}
impl Credentials {
    pub fn decode_sasl_challenge_oauth(challenge: &[u8]) -> Option<Self> {
        let (token, username) = mail_protocol_imap::oauth_bearer(challenge)?;
        Some(Self::Bearer { token, username })
    }
}

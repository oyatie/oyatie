//! Upstream configures its server through registry objects whose fields are
//! expressions over the session (`remote_ip = '10.0.0.1'` → value). The
//! shim keeps them and evaluates them into `SmtpParams` per session.
use mail_protocol_imap::SmtpParams;
use std::{sync::Mutex, time::Duration};

#[derive(Clone, Debug, Default)]
pub struct ExpressionMatch {
    pub if_: String,
    pub then: String,
}
#[derive(Clone, Debug, Default)]
pub struct List<T>(pub Vec<T>);
impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
#[derive(Clone, Debug, Default)]
pub struct Expression {
    pub match_: List<ExpressionMatch>,
    pub else_: String,
}
impl Expression {
    /// The first `if_` of the form `remote_ip = '<ip>'` that names the peer
    /// decides; `else_` otherwise. Upstream's expression language is far
    /// wider; only this form appears in the bound suites.
    fn eval(&self, remote_ip: &str) -> &str {
        self.match_
            .0
            .iter()
            .find(|m| {
                let named = m
                    .if_
                    .strip_prefix("remote_ip = '")
                    .and_then(|r| r.strip_suffix('\''))
                    // A wider expression would silently run on the defaults:
                    // refuse it so the suite that needs it is not fooled.
                    .unwrap_or_else(|| panic!("expression the shim cannot evaluate: {}", m.if_));
                named == remote_ip
            })
            .map_or(self.else_.as_str(), |m| m.then.as_str())
    }
}

/// `mta.inbound.session`: lifetime, idle timeout and transfer quota.
#[derive(Clone, Debug, Default)]
pub struct MtaInboundSession {
    pub max_duration: Expression,
    pub timeout: Expression,
    pub transfer_limit: Expression,
}

pub trait RegistryObject {
    fn install(self, registry: &Registry);
}
impl RegistryObject for MtaInboundSession {
    fn install(self, registry: &Registry) {
        *registry.session.lock().unwrap() = Some(self);
    }
}

#[derive(Default)]
pub struct Registry {
    session: Mutex<Option<MtaInboundSession>>,
}
impl Registry {
    pub fn params(&self, remote_ip: &str) -> SmtpParams {
        let mut params = SmtpParams::default();
        if let Some(session) = self.session.lock().unwrap().as_ref() {
            params.max_duration = duration(session.max_duration.eval(remote_ip));
            params.idle_timeout = duration(session.timeout.eval(remote_ip));
            params.transfer_bytes = session
                .transfer_limit
                .eval(remote_ip)
                .parse()
                .unwrap_or(usize::MAX);
        }
        params
    }
}

/// `500ms`, `30m`, `60m`, `1h`, `5s`.
fn duration(text: &str) -> Duration {
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    let n: u64 = digits.parse().unwrap_or(0);
    match &text[digits.len()..] {
        "ms" => Duration::from_millis(n),
        "s" => Duration::from_secs(n),
        "m" => Duration::from_secs(n * 60),
        "h" => Duration::from_secs(n * 3600),
        _ => Duration::from_secs(n),
    }
}

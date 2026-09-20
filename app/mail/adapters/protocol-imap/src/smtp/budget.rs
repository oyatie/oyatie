//! How much verification one session may ask for, and what it already knows.
//!
//! Splitting this from the checks themselves keeps the session loop free of
//! bookkeeping: it asks for a verdict and is handed one, or a refusal to send.
use super::{limits::Refusal, verify::Authentication};
use std::net::IpAddr;

/// A greeting is repeated legitimately across a STARTTLS upgrade, not
/// endlessly, and a reverse path is reissued as freely as a greeting.
const MAX_VERIFICATIONS: u8 = 4;

/// What a session has already decided, and how much more deciding it may ask
/// for. An EHLO is eight bytes and a verification is a chain of lookups, so
/// without a budget a session is a DNS amplifier aimed at our own resolver.
pub(super) struct Budget {
    /// The domain last decided and the verdict reached. The verdict is kept,
    /// not merely the fact of it: remembering only that a domain had been
    /// decided would skip the check on a repeat and let a refused host in by
    /// asking twice.
    decided: Option<(String, Result<(), &'static str>)>,
    spent: u8,
}

impl Budget {
    pub(super) fn new() -> Self {
        Self {
            decided: None,
            spent: 0,
        }
    }

    fn charge(&mut self) -> Result<(), Refusal> {
        self.spent += 1;
        if self.spent > MAX_VERIFICATIONS {
            return Err(Refusal {
                reply: "421 4.7.0 Too many verification requests\r\n".to_owned(),
                close: true,
            });
        }
        Ok(())
    }

    /// The verdict on this greeting, reached once per domain.
    pub(super) async fn ehlo(
        &mut self,
        authentication: &Authentication,
        peer: IpAddr,
        helo: &str,
        host: &str,
    ) -> Result<(), Refusal> {
        if !authentication.spf_ehlo.evaluates() {
            return Ok(());
        }
        let verdict = match &self.decided {
            Some((domain, verdict)) if domain.eq_ignore_ascii_case(helo) => *verdict,
            _ => {
                self.charge()?;
                let verdict = authentication.verify_ehlo(peer, helo, host).await;
                self.decided = Some((helo.to_owned(), verdict));
                verdict
            }
        };
        verdict.map_err(|reply| Refusal {
            reply: reply.to_owned(),
            close: false,
        })
    }

    /// The verdict on this reverse path. Not memoized: a session may offer a
    /// different sender each time, and each is its own question.
    pub(super) async fn mail_from(
        &mut self,
        authentication: &Authentication,
        peer: IpAddr,
        helo: &str,
        host: &str,
        sender: &str,
    ) -> Result<(), Refusal> {
        if !authentication.spf_mail_from.evaluates() {
            return Ok(());
        }
        self.charge()?;
        authentication
            .verify_mail_from(peer, helo, host, sender)
            .await
            .map_err(|reply| Refusal {
                reply: reply.to_owned(),
                close: false,
            })
    }
}

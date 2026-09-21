//! How much verification one session may ask for, and what it already knows.
//!
//! A greeting and a reverse path are not reissued at the same rate, so they
//! do not share a ceiling: a client greets about twice, while a connection
//! legitimately carries as many reverse paths as it carries messages. Only
//! reverse paths that never became a message are bounded.
use super::{limits::Refusal, verify::Authentication};
use std::net::IpAddr;

/// A greeting is repeated legitimately across a STARTTLS upgrade, not
/// endlessly.
const MAX_GREETINGS: u8 = 4;

/// An EHLO is eight bytes and a verification is a chain of lookups, so without
/// a budget a session is a DNS amplifier aimed at our own resolver.
pub(super) struct Budget {
    /// The verdict is kept, not merely the fact that the domain was decided:
    /// otherwise a repeat would skip the check and let a refused host in by
    /// asking twice.
    decided: Option<(String, Result<(), &'static str>)>,
    greetings: u8,
    /// Reverse paths offered since the last message this session delivered.
    abandoned: u8,
    max_abandoned: u8,
}

impl Budget {
    pub(super) fn new(max_abandoned: u8) -> Self {
        Self {
            decided: None,
            greetings: 0,
            abandoned: 0,
            max_abandoned,
        }
    }

    /// Without this, a connection could deliver only as many messages as its
    /// ceiling allowed verifications.
    pub(super) fn delivered(&mut self) {
        self.abandoned = 0;
    }

    fn charge(spent: &mut u8, ceiling: u8) -> Result<(), Refusal> {
        *spent = spent.saturating_add(1);
        if *spent > ceiling {
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
                Self::charge(&mut self.greetings, MAX_GREETINGS)?;
                let verdict = authentication.verify_ehlo(peer, helo, host).await;
                self.decided = Some((helo.to_owned(), verdict));
                verdict
            }
        };
        verdict.map_err(refuse)
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
        Self::charge(&mut self.abandoned, self.max_abandoned)?;
        authentication
            .verify_mail_from(peer, helo, host, sender)
            .await
            .map_err(refuse)
    }
}

fn refuse(reply: &'static str) -> Refusal {
    Refusal {
        reply: reply.to_owned(),
        close: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smtp::verify::Verify;

    fn watching() -> Authentication {
        // No verifier, so no lookup happens: these cover the accounting only.
        Authentication {
            spf_ehlo: Verify::Relaxed,
            spf_mail_from: Verify::Relaxed,
            ..Authentication::default()
        }
    }

    fn peer() -> IpAddr {
        IpAddr::from([10, 0, 0, 1])
    }

    #[tokio::test]
    async fn a_repeated_greeting_costs_nothing() {
        let mut budget = Budget::new(4);
        for _ in 0..20 {
            budget
                .ehlo(&watching(), peer(), "mx1.example.org", "localhost")
                .await
                .expect("a decision already reached must not be bought again");
        }
        assert_eq!(budget.greetings, 1);
    }

    #[tokio::test]
    async fn each_new_greeting_costs_one_and_the_ceiling_ends_the_session() {
        let mut budget = Budget::new(4);
        for n in 0..MAX_GREETINGS {
            budget
                .ehlo(
                    &watching(),
                    peer(),
                    &format!("mx{n}.example.org"),
                    "localhost",
                )
                .await
                .unwrap();
        }
        let refusal = budget
            .ehlo(&watching(), peer(), "one.too.many.example.org", "localhost")
            .await
            .expect_err("the ceiling must be reached");
        assert!(refusal.close);
        assert!(refusal.reply.starts_with("421 4.7.0"));
    }

    #[tokio::test]
    async fn a_delivered_message_does_not_spend_the_next_one() {
        let mut budget = Budget::new(4);
        for _ in 0..50 {
            budget
                .mail_from(
                    &watching(),
                    peer(),
                    "mx1.example.org",
                    "localhost",
                    "s@example.net",
                )
                .await
                .expect("delivering many messages over one connection is ordinary");
            budget.delivered();
        }
        assert_eq!(budget.abandoned, 0);
    }

    #[tokio::test]
    async fn abandoned_reverse_paths_are_still_bounded() {
        let mut budget = Budget::new(4);
        for _ in 0..4 {
            budget
                .mail_from(
                    &watching(),
                    peer(),
                    "mx1.example.org",
                    "localhost",
                    "s@example.net",
                )
                .await
                .unwrap();
        }
        assert!(
            budget
                .mail_from(
                    &watching(),
                    peer(),
                    "mx1.example.org",
                    "localhost",
                    "s@example.net"
                )
                .await
                .is_err_and(|refusal| refusal.close)
        );
    }
}

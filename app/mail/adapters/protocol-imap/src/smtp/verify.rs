//! What the sender's own DNS says about the peer: SPF on both identities the
//! envelope offers, and the reverse lookup of the address it connected from.
//!
//! A verdict never refuses on its own. `Verify` decides whether a failure is
//! recorded or answered, so an operator can watch a policy before it starts
//! rejecting mail — turning SPF on in one step is how a deployment loses
//! legitimate mail it cannot get back.
use super::dns::MailDns;
use mail_auth::{
    IprevOutput, MessageAuthenticator, Parameters, SpfOutput, SpfResult, spf::verify::SpfParameters,
};
use std::{net::IpAddr, sync::Arc};

/// How far a failed check may go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Verify {
    /// Not evaluated at all: no lookup, no verdict.
    #[default]
    Disabled,
    /// Evaluated and recorded; the session continues whatever the answer.
    Relaxed,
    /// A `Fail` is answered with a refusal.
    Strict,
}

impl Verify {
    /// The words an operator writes, and what an unknown word means. An
    /// unreadable policy must not silently become the permissive one.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "disable" | "disabled" | "false" => Some(Self::Disabled),
            "relaxed" => Some(Self::Relaxed),
            "strict" => Some(Self::Strict),
            _ => None,
        }
    }

    fn evaluates(self) -> bool {
        self != Self::Disabled
    }
}

/// The resolver these checks verify against.
///
/// `MessageAuthenticator` owns a live resolver and has no `Debug`, so this
/// carries one for `SmtpParams` without widening what a session prints.
#[derive(Clone)]
pub struct Verifier(pub Arc<MessageAuthenticator>);

impl std::fmt::Debug for Verifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Verifier")
    }
}

/// Everything a session needs to reach an opinion about its peer.
#[derive(Clone, Default)]
pub struct Authentication {
    pub verifier: Option<Verifier>,
    /// Answers consulted before the resolver. Sealed, it answers every miss
    /// itself, which is what keeps a conformance run off the network.
    pub dns: Option<Arc<MailDns>>,
    pub spf_ehlo: Verify,
    pub spf_mail_from: Verify,
}

impl std::fmt::Debug for Authentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Authentication")
            .field("spf_ehlo", &self.spf_ehlo)
            .field("spf_mail_from", &self.spf_mail_from)
            .finish_non_exhaustive()
    }
}

impl Authentication {
    fn resolver(&self) -> Option<(&MessageAuthenticator, &MailDns)> {
        // Both halves or neither: a verifier without a cache would resolve
        // against the real internet inside a sealed run.
        Some((&self.verifier.as_ref()?.0, self.dns.as_ref()?))
    }

    /// `Err(reply)` when policy refuses the session outright.
    pub(super) async fn verify_ehlo(
        &self,
        peer: IpAddr,
        helo: &str,
        host: &str,
    ) -> Result<(), &'static str> {
        if !self.spf_ehlo.evaluates() {
            return Ok(());
        }
        let Some((authenticator, dns)) = self.resolver() else {
            return Ok(());
        };
        let output = authenticator
            .verify_spf(
                Parameters::new(SpfParameters::verify_ehlo(peer, helo, host))
                    .with_txt_cache(dns)
                    .with_ipv4_cache(dns)
                    .with_ipv6_cache(dns)
                    .with_mx_cache(dns)
                    .with_ptr_cache(dns),
            )
            .await;
        refuse(self.spf_ehlo, output.result(), EHLO_REFUSED)
    }

    /// `Err(reply)` when policy refuses this reverse path.
    pub(super) async fn verify_mail_from(
        &self,
        peer: IpAddr,
        helo: &str,
        host: &str,
        sender: &str,
    ) -> Result<(), &'static str> {
        if !self.spf_mail_from.evaluates() {
            return Ok(());
        }
        let Some((authenticator, dns)) = self.resolver() else {
            return Ok(());
        };
        let output = authenticator
            .verify_spf(
                Parameters::new(SpfParameters::verify_mail_from(peer, helo, host, sender))
                    .with_txt_cache(dns)
                    .with_ipv4_cache(dns)
                    .with_ipv6_cache(dns)
                    .with_mx_cache(dns)
                    .with_ptr_cache(dns),
            )
            .await;
        refuse(self.spf_mail_from, output.result(), MAIL_FROM_REFUSED)
    }
}

const EHLO_REFUSED: &str = "550 5.7.23 SPF does not authorize this host for that domain\r\n";
const MAIL_FROM_REFUSED: &str = "550 5.7.23 SPF does not authorize this host for that sender\r\n";
const UNDECIDED: &str = "451 4.4.3 SPF could not be evaluated; try again later\r\n";

/// `Fail` is the domain saying no, and `Strict` answers it. `TempError` is
/// the domain saying nothing yet, and `Strict` answers *that* with a
/// temporary refusal rather than a decision: 451 is not "no", the client
/// retries, and "could not tell" stays distinct from "said yes". Accepting on
/// TempError would decide permissively on absent evidence, which is how a
/// resolver outage becomes an open door.
///
/// `SoftFail`, `Neutral`, `None` and `PermError` are not refusals: the domain
/// either declined to assert anything or published something unusable, and
/// neither is a statement that this host is forged.
fn refuse(policy: Verify, result: SpfResult, reply: &'static str) -> Result<(), &'static str> {
    if policy != Verify::Strict {
        return Ok(());
    }
    match result {
        SpfResult::Fail => Err(reply),
        SpfResult::TempError => Err(UNDECIDED),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_refuses_a_fail_and_defers_a_temperror_and_relaxed_never_answers() {
        for result in [
            SpfResult::Fail,
            SpfResult::SoftFail,
            SpfResult::Neutral,
            SpfResult::None,
            SpfResult::TempError,
            SpfResult::PermError,
            SpfResult::Pass,
        ] {
            assert!(
                refuse(Verify::Relaxed, result, EHLO_REFUSED).is_ok(),
                "relaxed must never refuse: {result:?}"
            );
            assert_eq!(
                refuse(Verify::Strict, result, EHLO_REFUSED).err(),
                match result {
                    SpfResult::Fail => Some(EHLO_REFUSED),
                    // Absent evidence gets a retry, not a verdict.
                    SpfResult::TempError => Some(UNDECIDED),
                    _ => None,
                },
                "strict answers Fail and TempError, nothing else: {result:?}"
            );
        }
    }

    #[test]
    fn an_unreadable_policy_word_is_not_silently_permissive() {
        assert_eq!(Verify::parse("strict"), Some(Verify::Strict));
        assert_eq!(Verify::parse("relaxed"), Some(Verify::Relaxed));
        assert_eq!(Verify::parse("disable"), Some(Verify::Disabled));
        assert_eq!(Verify::parse("Strict"), None, "case is not a policy word");
        assert_eq!(Verify::parse(""), None);
    }

    #[test]
    fn either_half_missing_disables_evaluation() {
        // Half-configured must not fall through to the real internet, and it
        // is half-configured in both directions.
        let cache_only = Authentication {
            dns: Some(Arc::new(MailDns::sealed())),
            spf_ehlo: Verify::Strict,
            ..Authentication::default()
        };
        assert!(
            cache_only.resolver().is_none(),
            "a cache without a verifier"
        );
        let verifier_only = Authentication {
            verifier: Verifier(Arc::new(
                mail_auth::MessageAuthenticator::new_cloudflare().unwrap(),
            ))
            .into(),
            spf_ehlo: Verify::Strict,
            ..Authentication::default()
        };
        assert!(
            verifier_only.resolver().is_none(),
            "a verifier without its cache"
        );
    }
}

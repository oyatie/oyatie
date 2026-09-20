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

/// The identity a verdict was reached about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// The domain the peer gave in EHLO.
    Ehlo,
    /// The domain of the reverse path in MAIL FROM.
    MailFrom,
}

/// Where a session's authentication verdicts go.
///
/// A receiver records what it decided: an operator reads it to explain a
/// refusal, and P4-3 reads the same verdicts to write the
/// `Authentication-Results` header onto the delivered message. Without a sink
/// the checks still run and still refuse; only the record is dropped.
pub trait AuthenticationLog: Send + Sync {
    fn spf(&self, stage: Stage, output: &SpfOutput);
    fn iprev(&self, output: &IprevOutput);
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
    /// Where verdicts are recorded. `None` still checks and still refuses.
    pub log: Option<Arc<dyn AuthenticationLog>>,
    /// Answers consulted before the resolver. Sealed, it answers every miss
    /// itself, which is what keeps a conformance run off the network.
    pub dns: Option<Arc<MailDns>>,
    pub spf_ehlo: Verify,
    pub spf_mail_from: Verify,
    pub iprev: Verify,
}

impl std::fmt::Debug for Authentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Authentication")
            .field("spf_ehlo", &self.spf_ehlo)
            .field("spf_mail_from", &self.spf_mail_from)
            .field("iprev", &self.iprev)
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
        if let Some(log) = &self.log {
            log.spf(Stage::Ehlo, &output);
        }
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
        if let Some(log) = &self.log {
            log.spf(Stage::MailFrom, &output);
        }
        refuse(self.spf_mail_from, output.result(), MAIL_FROM_REFUSED)
    }

    /// The reverse lookup never refuses: it is evidence for a later decision,
    /// and a missing PTR is too common to reject on by itself.
    pub(super) async fn verify_iprev(&self, peer: IpAddr) {
        if !self.iprev.evaluates() {
            return;
        }
        let Some((authenticator, dns)) = self.resolver() else {
            return;
        };
        let output = authenticator
            .verify_iprev(
                Parameters::new(peer)
                    .with_txt_cache(dns)
                    .with_ipv4_cache(dns)
                    .with_ipv6_cache(dns)
                    .with_mx_cache(dns)
                    .with_ptr_cache(dns),
            )
            .await;
        if let Some(log) = &self.log {
            log.iprev(&output);
        }
    }
}

const EHLO_REFUSED: &str = "550 5.7.23 SPF does not authorize this host for that domain\r\n";
const MAIL_FROM_REFUSED: &str = "550 5.7.23 SPF does not authorize this host for that sender\r\n";

/// Only an outright `Fail` refuses. `SoftFail`, `Neutral`, `None` and the two
/// error results are deliberately not refusals: they mean the domain did not
/// say no, and treating "could not tell" as "no" loses mail on a DNS outage.
fn refuse(policy: Verify, result: SpfResult, reply: &'static str) -> Result<(), &'static str> {
    if policy == Verify::Strict && result == SpfResult::Fail {
        return Err(reply);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_explicit_fail_refuses_and_only_under_strict() {
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
                refuse(Verify::Strict, result, EHLO_REFUSED).is_err(),
                result == SpfResult::Fail,
                "strict refuses exactly Fail, not {result:?}"
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
    fn a_verifier_without_its_cache_does_not_evaluate() {
        // Half-configured must not fall through to the real internet.
        let half = Authentication {
            dns: Some(Arc::new(MailDns::sealed())),
            spf_ehlo: Verify::Strict,
            ..Authentication::default()
        };
        assert!(half.resolver().is_none());
    }
}

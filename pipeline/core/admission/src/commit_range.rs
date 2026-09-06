//! Commit-range admission: the delivery contract's per-commit clauses.
//!
//! Every other gate in this crate judges a candidate's FILES. This module
//! judges its COMMITS, which are a different input class: who signed a commit
//! survives no tree diff and is invisible to a path set. Signing is the first
//! clause to land here; the module is shaped to hold the contract's other
//! per-commit clauses rather than to be re-invented per clause.
//!
//! The rule this makes visible was already enforced, by a protected-branch
//! ruleset that emits no check run. A lane carrying one unsigned commit shows
//! every check green and sits `BLOCKED` indefinitely, with nothing on the page
//! naming the cause; diagnosing it takes a GraphQL query and a prior suspicion.
//! An enforcement mechanism that cannot say why it refused is a trap rather
//! than a gate, and this module exists to convert one into the other.

/// What verification concluded about one commit's signature.
///
/// `Unverifiable` is a REFUSAL, not an abstention. A gate whose instrument is
/// missing must report red: reporting green would make the absence of a
/// verifier indistinguishable from the presence of a valid signature, which is
/// the one confusion a signing gate exists to prevent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignatureState {
    /// Verified against a principal the protected source names.
    SignedBy(String),
    /// The commit object carries no signature header at all.
    Unsigned,
    /// A signature that verified against no principal this repository trusts.
    ///
    /// Held apart from `Unsigned` because the remediation differs and the
    /// operator cannot infer which they are in: one re-signs the lane, the
    /// other enrolls a key through review. A single "bad signature" verdict
    /// would send half of its readers down the wrong repair.
    UntrustedKey,
    /// Signed, but not with SSH — a PGP signature, which this contract does
    /// not accept from an author.
    ///
    /// Held apart from `Unverifiable` because it is the likeliest real trip
    /// and the two read completely differently to whoever hit it. GitHub's
    /// "Update branch" button forges a merge commit signed with its own PGP
    /// key; it is not an ancestor of the base, so it lands squarely in the
    /// lane's range. Reporting that as a broken verifier tells the author to
    /// go looking at CI, when the fix is to rebase.
    NotSshSigned,
    /// Verification could not be performed, and why.
    Unverifiable(String),
}

/// One commit's admissible facts. Deliberately not the whole commit: this
/// carries what a clause may judge, so that adding a clause is a field here
/// rather than a second gate elsewhere.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitFact {
    pub sha: String,
    pub signature: SignatureState,
}

impl CommitFact {
    /// The short form operators paste back into `git`.
    fn short(&self) -> &str {
        let end = self.sha.len().min(9);
        &self.sha[..end]
    }
}

/// Refuse a range in which any commit is not signed by a trusted principal.
///
/// The range is the LANE's own commits and nothing else. A gate asserting
/// that no unsigned commit exists anywhere in history would be red forever
/// with no legal edit available to the author who tripped it, which is a
/// broken gate however true its claim.
pub fn signing_violations(range: &[CommitFact]) -> Vec<String> {
    range
        .iter()
        .filter_map(|fact| match &fact.signature {
            SignatureState::SignedBy(_) => None,
            SignatureState::Unsigned => Some(format!(
                "{}: unsigned; the delivery contract requires an SSH-signed commit",
                fact.short()
            )),
            SignatureState::UntrustedKey => Some(format!(
                "{}: signed by a key this repository does not trust; enroll the \
                 key in the protected signing authority before signing with it",
                fact.short()
            )),
            SignatureState::NotSshSigned => Some(format!(
                "{}: signed, but not with SSH — a PGP signature, which is what \
                 GitHub's \"Update branch\" forges; rebase the lane onto its \
                 base instead of merging into it",
                fact.short()
            )),
            SignatureState::Unverifiable(reason) => Some(format!(
                "{}: signature could not be verified ({reason}); refusing closed \
                 rather than admitting an unverified commit",
                fact.short()
            )),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(sha: &str, signature: SignatureState) -> CommitFact {
        CommitFact {
            sha: sha.to_owned(),
            signature,
        }
    }

    #[test]
    fn a_range_signed_throughout_is_admitted() {
        let range = [
            fact("aaaaaaaaaaaa", SignatureState::SignedBy("someone".into())),
            fact("bbbbbbbbbbbb", SignatureState::SignedBy("someone".into())),
        ];
        assert_eq!(signing_violations(&range), Vec::<String>::new());
    }

    /// The failure that motivated the module: a lane where MOST commits are
    /// signed. Reporting only a count, or stopping at the first, would leave
    /// an author re-signing one commit and hitting the same wall.
    #[test]
    fn every_unsigned_commit_is_named_not_just_the_first() {
        let range = [
            fact("111111111aaa", SignatureState::Unsigned),
            fact("222222222bbb", SignatureState::SignedBy("someone".into())),
            fact("333333333ccc", SignatureState::Unsigned),
        ];

        let violations = signing_violations(&range);

        assert_eq!(violations.len(), 2, "{violations:?}");
        assert!(violations[0].starts_with("111111111"), "{violations:?}");
        assert!(violations[1].starts_with("333333333"), "{violations:?}");
    }

    /// THREE STATES, THREE REPAIRS, THREE MESSAGES.
    ///
    /// Re-signing a lane, enrolling a key, and rebasing off a web-flow merge
    /// are entirely different actions, and an operator picks one by reading
    /// the line. Asserting each is merely refused cannot see two of them
    /// collapsing into one text — which is a regression that leaves the gate
    /// correct and the instruction wrong.
    #[test]
    fn each_refusal_names_its_own_repair() {
        let said = |state| {
            let found = signing_violations(&[fact("aaaaaaaaa", state)]);
            assert_eq!(found.len(), 1, "{found:?}");
            found[0].clone()
        };
        let unsigned = said(SignatureState::Unsigned);
        let untrusted = said(SignatureState::UntrustedKey);
        let pgp = said(SignatureState::NotSshSigned);

        assert!(unsigned.contains("unsigned"), "{unsigned}");
        assert!(untrusted.contains("enroll"), "{untrusted}");
        assert!(pgp.contains("rebase"), "{pgp}");
        assert_ne!(unsigned, untrusted);
        assert_ne!(unsigned, pgp);
        assert_ne!(untrusted, pgp);
    }

    /// An untrusted key and no key are different repairs, so they must be
    /// different messages. Asserting only that both are refused would pass
    /// against an implementation that collapses them.
    #[test]
    fn an_untrusted_key_reads_differently_from_no_signature() {
        let unsigned = signing_violations(&[fact("aaaaaaaaa", SignatureState::Unsigned)]);
        let untrusted = signing_violations(&[fact("bbbbbbbbb", SignatureState::UntrustedKey)]);

        assert_eq!(unsigned.len(), 1, "{unsigned:?}");
        assert_eq!(untrusted.len(), 1, "{untrusted:?}");
        assert!(unsigned[0].contains("unsigned"), "{unsigned:?}");
        assert!(untrusted[0].contains("enroll"), "{untrusted:?}");
        assert_ne!(unsigned[0], untrusted[0]);
    }

    /// FAIL CLOSED. A missing verifier is the case a naive gate reports green
    /// for, and green there is worse than no gate at all: it certifies exactly
    /// what it failed to check.
    #[test]
    fn a_signature_that_could_not_be_checked_is_refused_and_says_why() {
        let violations = signing_violations(&[fact(
            "ccccccccc",
            SignatureState::Unverifiable("ssh-keygen not found".into()),
        )]);

        assert_eq!(violations.len(), 1, "{violations:?}");
        assert!(
            violations[0].contains("ssh-keygen not found"),
            "the operator must learn which instrument was missing: {violations:?}"
        );
    }

    /// A range with nothing in it is not a finding. The gate reports on what a
    /// lane carries; asserting a non-empty range would go red exactly when a
    /// lane is rebased down to nothing.
    #[test]
    fn an_empty_range_is_not_a_violation() {
        assert_eq!(signing_violations(&[]), Vec::<String>::new());
    }
}

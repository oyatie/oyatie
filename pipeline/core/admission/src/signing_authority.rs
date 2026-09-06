//! Who may sign a commit on this repository.
//!
//! CODE, not configuration, and forced rather than chosen. The gate that reads
//! this is built from the PROTECTED source (`github.workflow_sha`), never from
//! the candidate tree — the same two-checkout discipline every other gate here
//! uses. A checked-in key file in the candidate would defeat the gate
//! completely: a lane could enroll its own key in the commit the gate is
//! judging and authorize itself. Enrolling a key is therefore a reviewed
//! change landed on `dev`, on exactly the same footing as a change to the
//! gate's own source.
//!
//! The rendering below is the ONE source of truth. The facade emits it for
//! `gpg.ssh.allowedSignersFile` so a developer verifies locally against the
//! same authority CI enforces, rather than against an empty configuration
//! that makes every commit report as unverifiable.

/// A principal permitted to sign commits, and the public key it signs with.
///
/// The key is public by construction — it is the half published at
/// `github.com/<user>.keys` — and carries no secret.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SigningPrincipal {
    pub principal: &'static str,
    pub key: &'static str,
}

/// Every principal this repository trusts to sign a commit.
/// EVERY key the account signs with, not merely the one on the machine that
/// wrote this. The fleet commits from several worktrees and hosts; an
/// authority holding one key refuses lanes the protected ruleset merges, and
/// that false refusal stays invisible until someone pushes from elsewhere.
pub const SIGNING_AUTHORITY: &[SigningPrincipal] = &[
    SigningPrincipal {
        principal: "56489493+jason931225@users.noreply.github.com",
        key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIM0J/JHnhbCxDruhmjhUOTEnZD4i8OsK05OcfLhAWl00",
    },
    SigningPrincipal {
        principal: "56489493+jason931225@users.noreply.github.com",
        key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJ6dcjCZ33c4wU1XaGXLhvDjdabGAQ1YZelM5L37AUwP",
    },
    SigningPrincipal {
        principal: "56489493+jason931225@users.noreply.github.com",
        key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAgMAp8vHS9V/9UQQVTa5FtmS9Q9fdB8I520DsZMMDTR",
    },
];

/// Render the authority as OpenSSH `allowed_signers` bytes.
///
/// The `git` namespace is stated explicitly rather than left to default: a
/// signature made for another namespace must not verify here, or a signing key
/// reused elsewhere would authorize commits it never approved.
pub fn allowed_signers() -> String {
    let mut rendered = String::new();
    for entry in SIGNING_AUTHORITY {
        rendered.push_str(entry.principal);
        rendered.push_str(" namespaces=\"git\" ");
        rendered.push_str(entry.key);
        rendered.push('\n');
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EVERY key the account signs with, pinned by identity and by count.
    ///
    /// "Not empty" was the whole test, and it could not see the failure this
    /// module's own doc warns about: dropping two of the three keys left the
    /// suite green while the gate refused every lane pushed from the fleet's
    /// other hosts. That refusal is invisible until someone pushes from one,
    /// and a gate that refuses good lanes is reverted rather than fixed.
    ///
    /// The keys are asserted by their trailing bytes rather than in full so
    /// the assertion stays readable; enrolling or retiring one is a
    /// deliberate edit here, which is the point.
    #[test]
    fn the_authority_holds_every_key_the_account_signs_with() {
        let keys: Vec<&str> = SIGNING_AUTHORITY.iter().map(|entry| entry.key).collect();
        assert_eq!(
            keys.len(),
            3,
            "three signing keys are registered on the account; an authority \
             holding fewer refuses lanes the protected ruleset merges"
        );
        for tail in ["AWl00", "AUwP", "MDTR"] {
            assert!(
                keys.iter().any(|key| key.ends_with(tail)),
                "the key ending {tail} is not enrolled: {keys:?}"
            );
        }
    }

    #[test]
    fn the_rendering_is_one_allowed_signers_line_per_principal() {
        let rendered = allowed_signers();
        let lines: Vec<&str> = rendered.lines().collect();

        assert_eq!(lines.len(), SIGNING_AUTHORITY.len(), "{rendered}");
        for (line, entry) in lines.iter().zip(SIGNING_AUTHORITY) {
            assert!(line.starts_with(entry.principal), "{line}");
            assert!(line.ends_with(entry.key), "{line}");
        }
    }

    /// The namespace is the clause that stops a key reused for another purpose
    /// from authorizing commits. Asserting the line merely parses would admit
    /// a rendering that dropped it.
    #[test]
    fn every_line_binds_the_signature_to_the_git_namespace() {
        for line in allowed_signers().lines() {
            assert!(line.contains(" namespaces=\"git\" "), "{line}");
        }
    }

    /// Keys are public, but a PRIVATE key pasted here would be catastrophic
    /// and would look almost identical in review.
    #[test]
    fn no_entry_carries_private_key_material() {
        for entry in SIGNING_AUTHORITY {
            assert!(entry.key.starts_with("ssh-"), "{}", entry.key);
            assert!(!entry.key.contains("PRIVATE"), "{}", entry.key);
        }
    }
}

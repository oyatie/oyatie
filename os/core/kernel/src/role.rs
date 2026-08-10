//! RBAC roles, mirroring Talos `pkg/machinery/role`.
//!
//! The authoritative spec is `pkg/machinery/role/role.go`. Upstream models a
//! [`Role`] as a string newtype whose value is used verbatim as the
//! Organization (`O`) value of a Talos client certificate, as the value of a
//! `talosctl` flag, and so on. The built-in roles all share the `os:` prefix:
//!
//! - `os:admin`
//! - `os:operator`
//! - `os:reader`
//! - `os:etcd:backup`
//! - `os:image:verifier`
//! - `os:meta:writer`
//! - `os:impersonator`
//!
//! Upstream [`Set`](RoleSet) semantics are pure *membership*: parsing keeps
//! every role it sees (including roles unknown to this version, for forward
//! compatibility) and reports the unknown ones separately. Empty/whitespace
//! organizations are skipped (older Talos certs carried one empty O).
//!
//! This crate keeps the upstream-faithful surface (`PREFIX`, the role
//! constants, `RoleSet::make_set`/`all`/`zero`/`parse`/`strings`/`includes`/
//! `includes_any`/`intersect`) and additionally exposes a few operating-system-specific
//! convenience helpers (capability predicates, an internal `Os` role) that the
//! rest of the workspace builds on.

use crate::error::{Error, Result};
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// Prefix shared by all built-in Talos roles (upstream `role.Prefix`).
pub const PREFIX: &str = "os:";

/// A Talos API RBAC role. These appear as `os:<role>` Organization values in
/// client certificates and gate access to the apid/machined APIs.
///
/// The variants mirror the upstream `role` constants exactly. `Os` is a
/// operating-system-internal pseudo-role (not present upstream) used to mark machined's
/// own internal calls; it has no canonical certificate string of its own and is
/// never produced by [`RoleSet::parse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    /// Internal full operating-system level access (machined internal calls).
    ///
    /// operating-system extension — not an upstream role. Never parsed from a cert.
    Os,
    /// `os:admin` — every API is available.
    Admin,
    /// `os:operator` — Reader plus management APIs that do not expose secrets
    /// (e.g. rebooting a node).
    Operator,
    /// `os:reader` — read-only APIs that do not expose secrets.
    Reader,
    /// `os:etcd:backup` — allows making etcd backups.
    EtcdBackup,
    /// `os:image:verifier` — allows verifying images.
    ImageVerifier,
    /// `os:meta:writer` — allows mutating META values (write and delete).
    MetaWriter,
    /// `os:impersonator` — impersonate another user (and their role). Used
    /// internally, but may also be granted to a user.
    Impersonator,
}

impl Role {
    /// The canonical string form including the `os:` prefix, as embedded in the
    /// certificate Organization value. Matches upstream role strings exactly.
    ///
    /// [`Role::Os`] has no upstream string; it borrows `os:admin` so existing
    /// callers that round-trip it keep working.
    pub fn as_ou(self) -> &'static str {
        match self {
            // `Os` is an internal pseudo-role with no upstream string; it shares
            // `os:admin` with `Admin` so existing round-trips keep working.
            Role::Os | Role::Admin => "os:admin",
            Role::Operator => "os:operator",
            Role::Reader => "os:reader",
            Role::EtcdBackup => "os:etcd:backup",
            Role::ImageVerifier => "os:image:verifier",
            Role::MetaWriter => "os:meta:writer",
            Role::Impersonator => "os:impersonator",
        }
    }

    /// Short identifier without the `os:` prefix.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Os => "os",
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Reader => "reader",
            Role::EtcdBackup => "etcd:backup",
            Role::ImageVerifier => "image:verifier",
            Role::MetaWriter => "meta:writer",
            Role::Impersonator => "impersonator",
        }
    }

    /// All roles that can be granted to users, in `Ord` order.
    ///
    /// Mirrors upstream `role.All` (which excludes the operating-system-internal `Os`).
    pub fn all() -> &'static [Role] {
        &[
            Role::Admin,
            Role::Operator,
            Role::Reader,
            Role::EtcdBackup,
            Role::ImageVerifier,
            Role::MetaWriter,
            Role::Impersonator,
        ]
    }

    /// Whether holding this role permits triggering/downloading etcd snapshots.
    pub fn can_etcd_backup(self) -> bool {
        matches!(self, Role::Os | Role::Admin | Role::EtcdBackup)
    }

    /// Whether holding this role implies read access.
    pub fn can_read(self) -> bool {
        matches!(self, Role::Os | Role::Admin | Role::Operator | Role::Reader)
    }

    /// Whether holding this role implies write/mutating access.
    pub fn can_write(self) -> bool {
        matches!(self, Role::Os | Role::Admin)
    }

    /// Whether this role may impersonate other roles.
    pub fn can_impersonate(self) -> bool {
        matches!(self, Role::Os | Role::Admin | Role::Impersonator)
    }

    /// Parse a single role identifier, with or without the `os:` prefix.
    ///
    /// This is a operating-system convenience that maps a string onto a known [`Role`]
    /// variant and errors on anything unknown. Upstream `Parse` instead keeps
    /// unknown roles verbatim — see [`RoleSet::parse`] for the upstream-faithful
    /// behavior. Recognizes the exact upstream strings (e.g. `os:etcd:backup`)
    /// and a few historical aliases.
    pub fn parse(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        let stripped = trimmed.strip_prefix(PREFIX).unwrap_or(trimmed);
        match stripped {
            "os" => Ok(Role::Os),
            "admin" => Ok(Role::Admin),
            "operator" => Ok(Role::Operator),
            "reader" => Ok(Role::Reader),
            // Canonical upstream string plus historical operating-system aliases.
            "etcd:backup" | "etcd-backup" | "etcdbackup" => Ok(Role::EtcdBackup),
            "image:verifier" => Ok(Role::ImageVerifier),
            "meta:writer" => Ok(Role::MetaWriter),
            "impersonator" => Ok(Role::Impersonator),
            other => Err(Error::parse(alloc::format!("unknown role '{other}'"))),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::str::FromStr for Role {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Role::parse(s)
    }
}

/// A set of roles, mirroring upstream `role.Set`.
///
/// Upstream semantics are pure membership over the role strings. A set may hold
/// roles that are unknown to this version of operating-system (forward compatibility):
/// those are preserved as their raw string and reported by [`RoleSet::parse`].
///
/// In addition to the upstream `Set` API, this type keeps the operating-system
/// capability predicates (`can_read`, `can_write`, ...) used elsewhere in the
/// workspace; those only consider *known* roles.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoleSet {
    /// Known roles.
    roles: BTreeSet<Role>,
    /// Roles unknown to this version, preserved verbatim for compatibility.
    unknown: BTreeSet<String>,
}

impl RoleSet {
    /// An empty role set (no permissions). Upstream `Zero`.
    pub fn new() -> Self {
        RoleSet {
            roles: BTreeSet::new(),
            unknown: BTreeSet::new(),
        }
    }

    /// An empty role set, named to match upstream `role.Zero`.
    pub fn zero() -> Self {
        Self::new()
    }

    /// Make a set of roles from constants. Upstream `MakeSet`.
    ///
    /// Use [`RoleSet::parse`] when the input comes from strings.
    pub fn make_set(iter: impl IntoIterator<Item = Role>) -> Self {
        Self::from_roles(iter)
    }

    /// Build a role set from an iterator of roles.
    pub fn from_roles(iter: impl IntoIterator<Item = Role>) -> Self {
        RoleSet {
            roles: iter.into_iter().collect(),
            unknown: BTreeSet::new(),
        }
    }

    /// The set of all roles that can be granted to users. Upstream `role.All`.
    pub fn all() -> Self {
        Self::from_roles(Role::all().iter().copied())
    }

    /// Parse a set of roles from strings (e.g. certificate Organization values).
    ///
    /// Upstream-faithful `Parse`: leading/trailing whitespace is trimmed, empty
    /// strings are skipped (older Talos certs carried one empty Organization),
    /// and every remaining role is added to the set — including roles unknown to
    /// this version. The returned [`Vec`] lists the unknown role strings (in
    /// input order, with duplicates preserved, matching upstream).
    pub fn parse<'a, I>(strs: I) -> (Self, Vec<String>)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut set = RoleSet::new();
        let mut unknown_roles = Vec::new();

        for raw in strs {
            let r = raw.trim();

            // Client certificates from previous Talos versions contained one
            // empty Organization; skip it.
            if r.is_empty() {
                continue;
            }

            match Role::parse(r) {
                // Only treat exact upstream-form strings as "known". `Os` is a
                // operating-system-internal pseudo-role and the historical etcd aliases
                // are not upstream strings, so they count as unknown here to
                // keep `parse` faithful to upstream `Parse`.
                Ok(role) if is_canonical(r, role) => {
                    set.roles.insert(role);
                }
                _ => {
                    unknown_roles.push(r.to_string());
                    set.unknown.insert(r.to_string());
                }
            }
        }

        (set, unknown_roles)
    }

    /// Parse a role set from Organization/OU strings, dropping anything that is
    /// not a recognized role.
    ///
    /// This is the lenient operating-system helper used across the workspace: unlike
    /// [`RoleSet::parse`] it does not retain unknown roles and it accepts the
    /// historical aliases handled by [`Role::parse`].
    pub fn parse_ous<'a>(ous: impl IntoIterator<Item = &'a str>) -> Self {
        let mut roles = BTreeSet::new();
        for ou in ous {
            if let Ok(role) = Role::parse(ou) {
                // `Os` is operating-system-internal: it has no upstream string, so it
                // never counts as a canonical/known parse (see `is_canonical`).
                if role != Role::Os {
                    roles.insert(role);
                }
            }
        }
        RoleSet {
            roles,
            unknown: BTreeSet::new(),
        }
    }

    /// Insert a role.
    pub fn insert(&mut self, role: Role) {
        self.roles.insert(role);
    }

    /// Whether the set contains the given role explicitly. Upstream `Includes`.
    pub fn contains(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }

    /// Whether the given role is present in the set. Upstream `Set.Includes`.
    pub fn includes(&self, role: Role) -> bool {
        self.contains(role)
    }

    /// Whether there is a non-empty intersection between the two sets.
    ///
    /// Returns false if either set is empty. Upstream `Set.IncludesAny`.
    pub fn includes_any(&self, other: &RoleSet) -> bool {
        self.roles.intersection(&other.roles).next().is_some()
            || self.unknown.intersection(&other.unknown).next().is_some()
    }

    /// A new set that is the intersection of the two sets. Upstream `Intersect`.
    pub fn intersect(&self, other: &RoleSet) -> RoleSet {
        RoleSet {
            roles: self.roles.intersection(&other.roles).copied().collect(),
            unknown: self.unknown.intersection(&other.unknown).cloned().collect(),
        }
    }

    /// The set as a sorted slice of role strings. Upstream `Set.Strings`.
    ///
    /// Includes any unknown roles, sorted lexically alongside the known ones,
    /// matching upstream which sorts the raw string keys.
    pub fn strings(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .roles
            .iter()
            .map(|r| r.as_ou().to_string())
            .chain(self.unknown.iter().cloned())
            .collect();
        out.sort();
        out
    }

    /// Whether any constituent role grants read access.
    pub fn can_read(&self) -> bool {
        self.roles.iter().any(|r| r.can_read())
    }

    /// Whether any constituent role grants write access.
    pub fn can_write(&self) -> bool {
        self.roles.iter().any(|r| r.can_write())
    }

    /// Whether any constituent role permits etcd snapshot/backup operations.
    pub fn can_etcd_backup(&self) -> bool {
        self.roles.iter().any(|r| r.can_etcd_backup())
    }

    /// Whether any constituent role may impersonate other roles.
    pub fn can_impersonate(&self) -> bool {
        self.roles.iter().any(|r| r.can_impersonate())
    }

    /// Iterate over the contained *known* roles in `Ord` order.
    pub fn iter(&self) -> impl Iterator<Item = Role> + '_ {
        self.roles.iter().copied()
    }

    /// The canonical OU strings (`os:<role>`) for the contained known roles.
    pub fn to_ou_list(&self) -> Vec<&'static str> {
        self.roles.iter().map(|r| r.as_ou()).collect()
    }

    /// Number of known roles in the set.
    pub fn len(&self) -> usize {
        self.roles.len()
    }

    /// Whether the set has no known roles.
    pub fn is_empty(&self) -> bool {
        self.roles.is_empty()
    }

    /// Render the known roles as a stable, comma-separated list of short names.
    pub fn to_string_list(&self) -> String {
        let parts: Vec<&str> = self.roles.iter().map(|r| r.as_str()).collect();
        parts.join(",")
    }
}

/// Whether `raw` is the exact upstream certificate string for `role`.
///
/// Used by [`RoleSet::parse`] to decide whether a parsed role counts as "known"
/// (upstream form) or should be retained as an unknown string.
fn is_canonical(raw: &str, role: Role) -> bool {
    // `Os` is operating-system-internal: it has no upstream string, so it never counts
    // as a canonical/known parse.
    role != Role::Os && raw == role.as_ou()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_matrix() {
        assert!(Role::Reader.can_read());
        assert!(!Role::Reader.can_write());
        assert!(Role::Admin.can_write());
        assert!(Role::Impersonator.can_impersonate());
        assert!(!Role::Reader.can_impersonate());
        // operator reads but does not write.
        assert!(Role::Operator.can_read());
        assert!(!Role::Operator.can_write());
    }

    #[test]
    fn parse_with_and_without_prefix() {
        assert_eq!(Role::parse("os:admin").unwrap(), Role::Admin);
        assert_eq!(Role::parse("reader").unwrap(), Role::Reader);
        assert!(Role::parse("os:wizard").is_err());
    }

    #[test]
    fn upstream_role_strings_are_exact() {
        // Match pkg/machinery/role/role.go exactly.
        assert_eq!(PREFIX, "os:");
        assert_eq!(Role::Admin.as_ou(), "os:admin");
        assert_eq!(Role::Operator.as_ou(), "os:operator");
        assert_eq!(Role::Reader.as_ou(), "os:reader");
        assert_eq!(Role::EtcdBackup.as_ou(), "os:etcd:backup");
        assert_eq!(Role::ImageVerifier.as_ou(), "os:image:verifier");
        assert_eq!(Role::MetaWriter.as_ou(), "os:meta:writer");
        assert_eq!(Role::Impersonator.as_ou(), "os:impersonator");
    }

    #[test]
    fn all_matches_upstream_membership() {
        // role.All = {admin, operator, reader, etcd:backup, image:verifier,
        // meta:writer, impersonator} — and excludes the internal Os.
        let all = RoleSet::all();
        assert_eq!(all.len(), 7);
        assert!(all.contains(Role::Admin));
        assert!(all.contains(Role::Operator));
        assert!(all.contains(Role::Reader));
        assert!(all.contains(Role::EtcdBackup));
        assert!(all.contains(Role::ImageVerifier));
        assert!(all.contains(Role::MetaWriter));
        assert!(all.contains(Role::Impersonator));
        assert!(!all.contains(Role::Os));
    }

    #[test]
    fn roleset_unions_permissions() {
        let set = RoleSet::parse_ous(["os:reader", "os:impersonator", "garbage"]);
        assert_eq!(set.len(), 2);
        assert!(set.can_read());
        assert!(!set.can_write());
        assert!(set.contains(Role::Reader));
    }

    #[test]
    fn parse_ous_rejects_internal_os_role() {
        assert!(RoleSet::parse_ous(["os", "os:os"]).is_empty());
    }

    #[test]
    fn roleset_string_is_sorted_and_stable() {
        let set = RoleSet::from_roles([Role::Reader, Role::Admin, Role::Reader]);
        // Ord puts Admin before Reader.
        assert_eq!(set.to_string_list(), "admin,reader");
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn etcd_backup_role_parses_and_authorizes() {
        assert_eq!(Role::parse("os:etcd:backup").unwrap(), Role::EtcdBackup);
        // historical aliases still parse leniently.
        assert_eq!(Role::parse("os:etcd-backup").unwrap(), Role::EtcdBackup);
        assert_eq!(Role::parse("etcdbackup").unwrap(), Role::EtcdBackup);
        assert_eq!(Role::EtcdBackup.as_ou(), "os:etcd:backup");
        assert_eq!(Role::EtcdBackup.as_str(), "etcd:backup");

        // etcd-backup grants only the snapshot capability, not generic read/write.
        assert!(Role::EtcdBackup.can_etcd_backup());
        assert!(!Role::EtcdBackup.can_read());
        assert!(!Role::EtcdBackup.can_write());

        // admin and os imply etcd-backup.
        assert!(Role::Admin.can_etcd_backup());
        assert!(Role::Os.can_etcd_backup());
        assert!(!Role::Reader.can_etcd_backup());
    }

    #[test]
    fn all_roles_roundtrip_through_ou() {
        for &r in Role::all() {
            // Each user-grantable role's OU parses back to itself exactly.
            let parsed = Role::parse(r.as_ou()).unwrap();
            assert_eq!(parsed, r);
        }
    }

    // --- Upstream Set semantics ---------------------------------------------

    #[test]
    fn parse_keeps_unknown_roles_and_reports_them() {
        // Upstream Parse: trim, skip empties, keep all (incl. unknown).
        let (set, unknown) = RoleSet::parse([" os:admin ", "", "os:reader", "os:future", "   "]);
        assert!(set.contains(Role::Admin));
        assert!(set.contains(Role::Reader));
        assert_eq!(unknown, alloc::vec!["os:future".to_string()]);
        // unknown roles surface in Strings(), sorted with the known ones.
        assert_eq!(
            set.strings(),
            alloc::vec![
                "os:admin".to_string(),
                "os:future".to_string(),
                "os:reader".to_string(),
            ]
        );
    }

    #[test]
    fn parse_empty_input_yields_zero() {
        let (set, unknown) = RoleSet::parse(["", "  ", ""]);
        assert!(unknown.is_empty());
        assert_eq!(set, RoleSet::zero());
        assert!(set.is_empty());
    }

    #[test]
    fn includes_and_includes_any() {
        let set = RoleSet::make_set([Role::Admin, Role::Reader]);
        assert!(set.includes(Role::Admin));
        assert!(!set.includes(Role::Operator));

        // IncludesAny: non-empty intersection.
        assert!(set.includes_any(&RoleSet::make_set([Role::Operator, Role::Reader])));
        assert!(!set.includes_any(&RoleSet::make_set([Role::Operator])));
        // Empty set never intersects.
        assert!(!set.includes_any(&RoleSet::zero()));
        assert!(!RoleSet::zero().includes_any(&set));
    }

    #[test]
    fn intersect_known_and_unknown() {
        let (a, _) = RoleSet::parse(["os:admin", "os:reader", "os:future"]);
        let (b, _) = RoleSet::parse(["os:reader", "os:future", "os:operator"]);
        let i = a.intersect(&b);
        assert!(i.contains(Role::Reader));
        assert!(!i.contains(Role::Admin));
        assert!(!i.contains(Role::Operator));
        // unknown "os:future" is in both, so survives the intersection.
        assert_eq!(
            i.strings(),
            alloc::vec!["os:future".to_string(), "os:reader".to_string()]
        );
    }

    #[test]
    fn roleset_ou_list_and_iter() {
        let set = RoleSet::from_roles([Role::Reader, Role::EtcdBackup]);
        let ous = set.to_ou_list();
        assert_eq!(ous, alloc::vec!["os:reader", "os:etcd:backup"]);
        let collected: alloc::vec::Vec<Role> = set.iter().collect();
        assert_eq!(collected, alloc::vec![Role::Reader, Role::EtcdBackup]);
        assert!(set.can_etcd_backup());
        assert!(set.can_read());
        assert!(!set.can_write());
    }
}

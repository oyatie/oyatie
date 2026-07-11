//! Foundry Cargo prefix fitness kernel.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoPrefixMember {
    pub member_path: String,  // data_class: INTERNAL_ONLY
    pub package_name: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CargoPrefixReport {
    pub members_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CargoPrefixError {
    EmptyPrefix,
    NoWorkspaceMembers,
    MemberPathMissingCrateId {
        member_path: String,
    },
    MemberPathPrefixViolation {
        member_path: String,
        crate_id: String,
        expected_prefix: String,
    },
    PackageNamePrefixViolation {
        member_path: String,
        package_name: String,
        expected_prefix: String,
    },
    PackageNamePathMismatch {
        member_path: String,
        crate_id: String,
        package_name: String,
    },
}

pub fn validate_cargo_prefix<M>(
    members: M,
    expected_prefix: &str,
) -> Result<CargoPrefixReport, CargoPrefixError>
where
    M: IntoIterator<Item = CargoPrefixMember>,
{
    let expected_prefix = expected_prefix.trim();
    if expected_prefix.is_empty() {
        return Err(CargoPrefixError::EmptyPrefix);
    }

    let mut checked = 0usize;
    for member in members {
        checked += 1;
        let crate_id = crate_id_from_member_path(&member.member_path).ok_or_else(|| {
            CargoPrefixError::MemberPathMissingCrateId {
                member_path: member.member_path.clone(),
            }
        })?;
        if !crate_id.starts_with(expected_prefix) {
            return Err(CargoPrefixError::MemberPathPrefixViolation {
                member_path: member.member_path,
                crate_id,
                expected_prefix: expected_prefix.to_string(),
            });
        }
        if !member.package_name.starts_with(expected_prefix) {
            return Err(CargoPrefixError::PackageNamePrefixViolation {
                member_path: member.member_path,
                package_name: member.package_name,
                expected_prefix: expected_prefix.to_string(),
            });
        }
        if crate_id != member.package_name {
            return Err(CargoPrefixError::PackageNamePathMismatch {
                member_path: member.member_path,
                crate_id,
                package_name: member.package_name,
            });
        }
    }

    if checked == 0 {
        Err(CargoPrefixError::NoWorkspaceMembers)
    } else {
        Ok(CargoPrefixReport {
            members_checked: checked,
        })
    }
}

fn crate_id_from_member_path(member_path: &str) -> Option<String> {
    member_path
        .trim()
        .trim_matches('/')
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .map(str::to_string)
        .filter(|segment| !segment.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_oya_prefixed_member_paths_and_package_names() {
        assert_eq!(
            validate_cargo_prefix(
                [
                    CargoPrefixMember {
                        member_path: "crates/oya-intelligence-capability-kernel".into(),
                        package_name: "oya-intelligence-capability-kernel".into(),
                    },
                    CargoPrefixMember {
                        member_path: "crates/oya-dev-cli".into(),
                        package_name: "oya-dev-cli".into(),
                    },
                ],
                "oya-",
            ),
            Ok(CargoPrefixReport { members_checked: 2 })
        );
    }

    #[test]
    fn rejects_unprefixed_member_path() {
        assert_eq!(
            validate_cargo_prefix(
                [CargoPrefixMember {
                    member_path: "crates/foundry-capability-kernel".into(),
                    package_name: "oya-intelligence-capability-kernel".into(),
                }],
                "oya-",
            ),
            Err(CargoPrefixError::MemberPathPrefixViolation {
                member_path: "crates/foundry-capability-kernel".into(),
                crate_id: "foundry-capability-kernel".into(),
                expected_prefix: "oya-".into(),
            })
        );
    }

    #[test]
    fn rejects_unprefixed_package_name() {
        assert_eq!(
            validate_cargo_prefix(
                [CargoPrefixMember {
                    member_path: "crates/oya-intelligence-capability-kernel".into(),
                    package_name: "foundry-capability-kernel".into(),
                }],
                "oya-",
            ),
            Err(CargoPrefixError::PackageNamePrefixViolation {
                member_path: "crates/oya-intelligence-capability-kernel".into(),
                package_name: "foundry-capability-kernel".into(),
                expected_prefix: "oya-".into(),
            })
        );
    }

    #[test]
    fn rejects_path_package_name_mismatch() {
        assert_eq!(
            validate_cargo_prefix(
                [CargoPrefixMember {
                    member_path: "crates/oya-intelligence-capability-kernel".into(),
                    package_name: "oya-intelligence-policy-kernel".into(),
                }],
                "oya-",
            ),
            Err(CargoPrefixError::PackageNamePathMismatch {
                member_path: "crates/oya-intelligence-capability-kernel".into(),
                crate_id: "oya-intelligence-capability-kernel".into(),
                package_name: "oya-intelligence-policy-kernel".into(),
            })
        );
    }

    #[test]
    fn rejects_empty_prefix_and_empty_member_set() {
        assert_eq!(
            validate_cargo_prefix(
                [CargoPrefixMember {
                    member_path: "crates/oya-intelligence-capability-kernel".into(),
                    package_name: "oya-intelligence-capability-kernel".into(),
                }],
                " ",
            ),
            Err(CargoPrefixError::EmptyPrefix)
        );
        assert_eq!(
            validate_cargo_prefix([], "oya-"),
            Err(CargoPrefixError::NoWorkspaceMembers)
        );
    }
}

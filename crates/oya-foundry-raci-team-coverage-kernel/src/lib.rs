//! Foundry RACI team coverage fitness kernel.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaciTeamCoverageReport {
    pub teams_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaciTeamCoverageError {
    NoTeams,
    EmptyTeamId,
    DuplicateTeamId { team_id: String },
    MissingRaciCoverage { team_id: String },
    MissingCodeownersCoverage { team_id: String },
}

pub fn validate_raci_team_coverage<T, R, C>(
    team_ids: T,
    raci_team_ids: R,
    codeowners_team_ids: C,
) -> Result<RaciTeamCoverageReport, RaciTeamCoverageError>
where
    T: IntoIterator,
    T::Item: AsRef<str>,
    R: IntoIterator,
    R::Item: AsRef<str>,
    C: IntoIterator,
    C::Item: AsRef<str>,
{
    let mut teams = BTreeSet::new();
    for team_id in team_ids {
        let team_id = team_id.as_ref().trim();
        if team_id.is_empty() {
            return Err(RaciTeamCoverageError::EmptyTeamId);
        }
        if !teams.insert(team_id.to_string()) {
            return Err(RaciTeamCoverageError::DuplicateTeamId {
                team_id: team_id.to_string(),
            });
        }
    }
    if teams.is_empty() {
        return Err(RaciTeamCoverageError::NoTeams);
    }

    let raci_team_ids = raci_team_ids
        .into_iter()
        .map(|team_id| team_id.as_ref().trim().to_string())
        .collect::<BTreeSet<_>>();
    let codeowners_team_ids = codeowners_team_ids
        .into_iter()
        .map(|team_id| team_id.as_ref().trim().to_string())
        .collect::<BTreeSet<_>>();

    for team_id in &teams {
        if !raci_team_ids.contains(team_id) {
            return Err(RaciTeamCoverageError::MissingRaciCoverage {
                team_id: team_id.clone(),
            });
        }
        if !codeowners_team_ids.contains(team_id) {
            return Err(RaciTeamCoverageError::MissingCodeownersCoverage {
                team_id: team_id.clone(),
            });
        }
    }

    Ok(RaciTeamCoverageReport {
        teams_checked: teams.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_team_missing_raci_row() {
        assert_eq!(
            validate_raci_team_coverage(
                ["axis-foundry", "axis-cloud"],
                ["axis-foundry"],
                ["axis-foundry", "axis-cloud"],
            ),
            Err(RaciTeamCoverageError::MissingRaciCoverage {
                team_id: "axis-cloud".into()
            })
        );
    }

    #[test]
    fn rejects_team_missing_codeowners_owner() {
        assert_eq!(
            validate_raci_team_coverage(
                ["axis-foundry", "axis-cloud"],
                ["axis-foundry", "axis-cloud"],
                ["axis-foundry"],
            ),
            Err(RaciTeamCoverageError::MissingCodeownersCoverage {
                team_id: "axis-cloud".into()
            })
        );
    }

    #[test]
    fn rejects_duplicate_team_ids() {
        assert_eq!(
            validate_raci_team_coverage(
                ["axis-foundry", "axis-foundry"],
                ["axis-foundry"],
                ["axis-foundry"],
            ),
            Err(RaciTeamCoverageError::DuplicateTeamId {
                team_id: "axis-foundry".into()
            })
        );
    }

    #[test]
    fn accepts_raci_and_codeowners_covered_teams() {
        assert_eq!(
            validate_raci_team_coverage(
                ["axis-foundry", "axis-cloud"],
                ["axis-foundry", "axis-cloud"],
                ["axis-foundry", "axis-cloud"],
            ),
            Ok(RaciTeamCoverageReport { teams_checked: 2 })
        );
    }
}

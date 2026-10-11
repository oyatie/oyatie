use super::*;

pub(super) fn eval_user(u: &User, expr: &FilterExpr) -> bool {
    match expr {
        FilterExpr::And(a, b) => eval_user(u, a) && eval_user(u, b),
        FilterExpr::Or(a, b) => eval_user(u, a) || eval_user(u, b),
        FilterExpr::Not(inner) => !eval_user(u, inner),
        FilterExpr::Pr(attr) => attr_present_user(u, attr),
        FilterExpr::Eq(attr, v) => {
            attr_value_user(u, attr).is_some_and(|s| s.eq_ignore_ascii_case(v))
        }
        FilterExpr::Ne(attr, v) => {
            attr_value_user(u, attr).is_none_or(|s| !s.eq_ignore_ascii_case(v))
        }
        FilterExpr::Co(attr, v) => attr_value_user(u, attr)
            .is_some_and(|s| s.to_ascii_lowercase().contains(&v.to_ascii_lowercase())),
        FilterExpr::Sw(attr, v) => attr_value_user(u, attr)
            .is_some_and(|s| s.to_ascii_lowercase().starts_with(&v.to_ascii_lowercase())),
        FilterExpr::Ew(attr, v) => attr_value_user(u, attr)
            .is_some_and(|s| s.to_ascii_lowercase().ends_with(&v.to_ascii_lowercase())),
    }
}

pub(super) fn attr_value_user(u: &User, attr: &str) -> Option<String> {
    match attr {
        "userName" | "username" => Some(u.user_name.clone()),
        "id" => Some(u.id.0.clone()),
        "externalId" => u.external_id.clone(),
        "displayName" => u.display_name.clone(),
        "active" => Some(if u.active {
            "true".to_owned()
        } else {
            "false".to_owned()
        }),
        "emails.value" => u.emails.first().map(|e| e.value.clone()),
        "name.givenName" => u.name.as_ref().and_then(|n| n.given_name.clone()),
        "name.familyName" => u.name.as_ref().and_then(|n| n.family_name.clone()),
        _ => None,
    }
}

pub(super) fn attr_present_user(u: &User, attr: &str) -> bool {
    attr_value_user(u, attr).is_some_and(|s| !s.is_empty())
}

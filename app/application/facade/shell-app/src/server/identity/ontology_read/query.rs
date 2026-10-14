use application_ontology_read::{
    DEFAULT_PAGE_LIMIT, FilterValue, ObjectListQuery, ObjectPointQuery, OntologyReadError,
    PropertyFilter,
};
use std::collections::BTreeMap;

fn parameters(
    raw: Option<&str>,
    admitted: &[&str],
) -> Result<BTreeMap<String, String>, OntologyReadError> {
    let url = reqwest::Url::parse(&format!(
        "https://query.invalid/?{}",
        raw.unwrap_or_default()
    ))
    .map_err(|_| OntologyReadError::InvalidQuery)?;
    let mut values = BTreeMap::new();
    for (key, value) in url.query_pairs() {
        if !admitted.contains(&key.as_ref())
            || values
                .insert(key.into_owned(), value.into_owned())
                .is_some()
        {
            return Err(OntologyReadError::InvalidQuery);
        }
    }
    Ok(values)
}
fn required(values: &BTreeMap<String, String>, key: &str) -> Result<String, OntologyReadError> {
    values
        .get(key)
        .cloned()
        .ok_or(OntologyReadError::InvalidQuery)
}
fn number<T: std::str::FromStr>(value: &str) -> Result<T, OntologyReadError> {
    value.parse().map_err(|_| OntologyReadError::InvalidQuery)
}
fn literal(raw: &str) -> Result<FilterValue, OntologyReadError> {
    let (kind, value) = raw.split_once(':').ok_or(OntologyReadError::InvalidQuery)?;
    match kind {
        "text" => Ok(FilterValue::Text(value.into())),
        "int" => number(value).map(FilterValue::Integer),
        "bool" => match value {
            "true" => Ok(FilterValue::Boolean(true)),
            "false" => Ok(FilterValue::Boolean(false)),
            _ => Err(OntologyReadError::InvalidQuery),
        },
        _ => Err(OntologyReadError::InvalidQuery),
    }
}
pub(super) fn point(
    object_ref: String,
    raw: Option<&str>,
) -> Result<ObjectPointQuery, OntologyReadError> {
    let raw = raw.ok_or(OntologyReadError::InvalidQuery)?;
    if raw.len() > 4096 {
        return Err(OntologyReadError::InvalidQuery);
    }
    let parsed = reqwest::Url::parse(&format!("https://query.invalid/?{raw}"))
        .map_err(|_| OntologyReadError::InvalidQuery)?;
    let pairs: Vec<_> = parsed.query_pairs().into_owned().collect();
    let mut canonical = reqwest::Url::parse("https://query.invalid/")
        .map_err(|_| OntologyReadError::InvalidQuery)?;
    canonical.query_pairs_mut().extend_pairs(&pairs);
    if canonical.query() != Some(raw) {
        return Err(OntologyReadError::InvalidQuery);
    }
    let mut values = BTreeMap::new();
    for (key, value) in pairs {
        if !matches!(key.as_str(), "type" | "revision") || values.insert(key, value).is_some() {
            return Err(OntologyReadError::InvalidQuery);
        }
    }
    let revision = required(&values, "revision")?;
    if revision.is_empty() || !revision.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(OntologyReadError::InvalidQuery);
    }
    let query = ObjectPointQuery {
        entity_type: required(&values, "type")?,
        object_ref,
        revision: number(&revision)?,
    };
    query.validate()?;
    Ok(query)
}
pub(super) fn listing(raw: Option<&str>) -> Result<ObjectListQuery, OntologyReadError> {
    let values = parameters(
        raw,
        &[
            "type", "revision", "limit", "cursor", "property", "equals", "from", "to",
        ],
    )?;
    let filter = match (
        values.get("property"),
        values.get("equals"),
        values.get("from"),
        values.get("to"),
    ) {
        (None, None, None, None) => None,
        (Some(property), Some(value), None, None) => Some(PropertyFilter::Equals {
            property: property.clone(),
            value: literal(value)?,
        }),
        (Some(property), None, Some(from), Some(to)) => Some(PropertyFilter::Range {
            property: property.clone(),
            from: literal(from)?,
            to: literal(to)?,
        }),
        _ => return Err(OntologyReadError::InvalidQuery),
    };
    let query = ObjectListQuery {
        entity_type: required(&values, "type")?,
        revision: number(&required(&values, "revision")?)?,
        limit: values
            .get("limit")
            .map_or(Ok(DEFAULT_PAGE_LIMIT), |v| number(v))?,
        cursor: values.get("cursor").cloned(),
        filter,
    };
    query.validate()?;
    Ok(query)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_form_values_decode_once_before_the_native_literal_transport() {
        let q = listing(Some(
            "type=ety_record&revision=2&property=name&equals=text%3AAda%2B%2520%3D%3A%3F",
        ))
        .unwrap();
        assert_eq!(
            q.filter,
            Some(PropertyFilter::Equals {
                property: "name".into(),
                value: FilterValue::Text("Ada+%20=:?".into())
            })
        );
        assert_eq!(q.limit, 100);
    }
    #[test]
    fn duplicate_unknown_incomplete_and_invalid_queries_refuse() {
        for raw in [
            "type=ety_record&revision=1&revision=2",
            "type=x&revision=1&unknown=x",
            "type=x&revision=1&property=name",
            "type=x&revision=1&from=int:1&to=int:2",
            "type=x&revision=1&property=x&equals=int:1&from=int:1&to=int:2",
            "type=x&revision=1&property=x&from=int:2&to=int:1",
            "type=x&revision=1&limit=0",
            "type=x&revision=abc",
        ] {
            assert!(listing(Some(raw)).is_err(), "{raw}");
        }
    }

    #[test]
    fn point_query_binds_exact_form_decoded_type_reference_and_pin() {
        for entity_type in [
            "ety_record",
            "ety_+%&=#/?",
            "ety_가",
            "ety_a b",
            "ety_\u{feff}",
            &format!("ety_{}", "가".repeat(84)),
        ] {
            for revision in [0, u32::MAX] {
                for reversed in [false, true] {
                    let mut url = reqwest::Url::parse("https://query.invalid/").unwrap();
                    let revision = revision.to_string();
                    let pairs = [("type", entity_type), ("revision", revision.as_str())];
                    if reversed {
                        url.query_pairs_mut().extend_pairs(pairs.into_iter().rev());
                    } else {
                        url.query_pairs_mut().extend_pairs(pairs);
                    }
                    let result = point("ent_+%/가\u{feff}".into(), url.query()).unwrap();
                    assert_eq!(result.entity_type, entity_type);
                    assert_eq!(result.object_ref, "ent_+%/가\u{feff}");
                    assert_eq!(result.revision.to_string(), revision);
                }
            }
        }
    }

    #[test]
    fn point_query_refuses_ambiguous_lossy_noncanonical_and_unbounded_intent() {
        for raw in [
            "",
            "revision=1",
            "type=ety_record",
            "type=&revision=1",
            "type=ety_&revision=1",
            "type=ety_record&revision=",
            "type=ety_record&revision=-1",
            "type=ety_record&revision=%2B1",
            "type=ety_record&revision=1.0",
            "type=ety_record&revision=4294967296",
            "type=ety_record&revision=1&revision=2",
            "type=ety_a&type=ety_b&revision=1",
            "type=ety_a&revision=1&unknown=x",
            "type=ety_a&revision=1&",
            "&type=ety_a&revision=1",
            "%74ype=ety_a&revision=1",
            "type=ety_%61&revision=1",
            "type=ety_%2b&revision=1",
            "type=ety_%&revision=1",
            "type=ety_%FF&revision=1",
            "type=ety_%C0%AF&revision=1",
            "type=ety_a%20b&revision=1",
            "type=ety_a\tb&revision=1",
            "type=ety_a&revision=1#x",
            "type=ety_a%0A&revision=1",
            "type=ety_a%C2%85&revision=1",
            "type=ety_a+&revision=1",
        ] {
            assert_eq!(
                point("ent_a".into(), Some(raw)),
                Err(OntologyReadError::InvalidQuery),
                "{raw:?}"
            );
        }
        assert_eq!(
            point("ent_a".into(), None),
            Err(OntologyReadError::InvalidQuery)
        );
        assert_eq!(
            point("wrong".into(), Some("type=ety_a&revision=1")),
            Err(OntologyReadError::InvalidQuery)
        );
        assert_eq!(
            point("ent_a".into(), Some(&"x".repeat(4097))),
            Err(OntologyReadError::InvalidQuery)
        );
    }
}

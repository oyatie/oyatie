use std::fs;
use std::path::Path;

use check_cohesion::CrossAxisContract;

use crate::{
    extract_json_array_for_key, find_matching_json_delimiter, parse_json_string_array_field,
    parse_json_string_field,
};

pub(crate) fn read_cross_axis_contracts(path: &Path) -> Result<Vec<CrossAxisContract>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("contracts registry unreadable {}: {error}", path.display()))?;
    let array = extract_json_array_for_key(&contents, "cross_axis_contracts")
        .ok_or_else(|| "contracts registry missing cross_axis_contracts array".to_string())?;
    let mut contracts = Vec::new();
    for object in extract_contract_objects(array)? {
        let id = parse_json_string_field(object, "id")
            .ok_or_else(|| "contract row missing id".to_string())?;
        let owner_axis = parse_json_string_field(object, "owner_axis")
            .ok_or_else(|| format!("contract {id} missing owner_axis"))?;
        let consumer_axes = parse_json_string_array_field(object, "consumer_axes")
            .ok_or_else(|| format!("contract {id} missing consumer_axes"))?;
        let location = parse_json_string_field(object, "location")
            .ok_or_else(|| format!("contract {id} missing location"))?;
        let change_review = parse_json_string_field(object, "change_review")
            .ok_or_else(|| format!("contract {id} missing change_review"))?;
        let source_crate_ids = parse_source_crate_ids(&location);
        contracts.push(CrossAxisContract {
            id,
            owner_axis,
            consumer_axes,
            location,
            change_review,
            source_crate_ids,
        });
    }
    Ok(contracts)
}

fn extract_contract_objects(array: &str) -> Result<Vec<&str>, String> {
    let mut objects = Vec::new();
    let mut offset = 0usize;
    let mut expecting_value = true;
    let mut saw_value = false;
    while offset < array.len() {
        let rest = &array[offset..];
        let trimmed = rest.trim_start();
        offset += rest.len() - trimmed.len();
        if offset >= array.len() {
            break;
        }
        if expecting_value {
            if array[offset..].starts_with(',') {
                return Err(
                    "contracts registry cross_axis_contracts contains empty entry".to_string(),
                );
            }
            if !array[offset..].starts_with('{') {
                return Err(
                    "contracts registry cross_axis_contracts contains non-object entry".to_string(),
                );
            }
            let object = &array[offset..];
            let object_end = find_matching_json_delimiter(object, '{', '}').ok_or_else(|| {
                "contracts registry cross_axis_contracts contains malformed object".to_string()
            })?;
            objects.push(&object[..=object_end]);
            offset += object_end + 1;
            expecting_value = false;
            saw_value = true;
        } else if array[offset..].starts_with(',') {
            offset += 1;
            expecting_value = true;
        } else {
            return Err(
                "contracts registry cross_axis_contracts entries must be comma-separated"
                    .to_string(),
            );
        }
    }
    if expecting_value && saw_value {
        return Err("contracts registry cross_axis_contracts contains trailing comma".to_string());
    }
    Ok(objects)
}

fn parse_source_crate_ids(location: &str) -> Vec<String> {
    location
        .split("crates/")
        .skip(1)
        .filter_map(|rest| {
            let crate_id = rest
                .chars()
                .take_while(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '*')
                })
                .collect::<String>();
            if crate_id.is_empty() {
                None
            } else {
                Some(crate_id)
            }
        })
        .collect()
}

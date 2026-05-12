pub(crate) fn clean_yaml_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

pub(crate) fn parse_yaml_inline_values(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(clean_yaml_value)
            .filter(|item| !item.is_empty())
            .collect()
    } else if trimmed.is_empty() {
        Vec::new()
    } else {
        vec![trimmed.to_string()]
    }
}

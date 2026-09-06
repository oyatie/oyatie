use super::*;

pub(super) fn array_field_contents<'a>(
    input: &'a str,
    field: &'static str,
) -> Result<&'a str, CloudIacReleaseIndexError> {
    let field_position = find_field_position(input, field)
        .ok_or(CloudIacReleaseIndexError::MissingField { field })?;
    let token = quoted_field_token(field);
    let after_field = field_position + token.len();
    let colon_position = find_next_non_string_char(input, after_field, ':').ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} is missing ':' separator"),
        }
    })?;
    let array_start = first_non_whitespace_byte(input, colon_position + 1).ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} is missing an array value"),
        }
    })?;
    if input.as_bytes().get(array_start) != Some(&b'[') {
        return Err(CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} must be an array"),
        });
    }
    let array_end = matching_delimiter(input, array_start, '[', ']').ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} array is not closed"),
        }
    })?;
    Ok(&input[array_start + 1..array_end])
}

pub(super) fn top_level_object_slices(input: &str) -> Result<Vec<&str>, CloudIacReleaseIndexError> {
    let mut objects = Vec::new();
    let mut depth = 0_usize;
    let mut object_start = None;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    object_start = Some(index);
                }
                depth = depth.saturating_add(1);
            }
            '}' => {
                if depth == 0 {
                    return Err(CloudIacReleaseIndexError::MalformedJson {
                        reason: "module array contains an unmatched object close".to_string(),
                    });
                }
                depth -= 1;
                if depth == 0 {
                    let start =
                        object_start.ok_or_else(|| CloudIacReleaseIndexError::MalformedJson {
                            reason: "module object close has no start".to_string(),
                        })?;
                    objects.push(&input[start..index + ch.len_utf8()]);
                    object_start = None;
                }
            }
            ',' | ' ' | '\n' | '\r' | '\t' if depth == 0 => {}
            _ if depth == 0 => {
                return Err(CloudIacReleaseIndexError::MalformedJson {
                    reason: "modules array must contain JSON objects only".to_string(),
                });
            }
            _ => {}
        }
    }

    if in_string || depth != 0 {
        return Err(CloudIacReleaseIndexError::MalformedJson {
            reason: "module array has an unterminated string or object".to_string(),
        });
    }
    Ok(objects)
}

pub(super) fn required_string_field(
    object: &str,
    field: &'static str,
) -> Result<String, CloudIacReleaseIndexError> {
    let field_position = find_field_position(object, field)
        .ok_or(CloudIacReleaseIndexError::MissingField { field })?;
    let token = quoted_field_token(field);
    let after_field = field_position + token.len();
    let colon_position = find_next_non_string_char(object, after_field, ':').ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} is missing ':' separator"),
        }
    })?;
    let value_start = first_non_whitespace_byte(object, colon_position + 1).ok_or_else(|| {
        CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} is missing a value"),
        }
    })?;
    if object.as_bytes().get(value_start) != Some(&b'"') {
        return Err(CloudIacReleaseIndexError::MalformedJson {
            reason: format!("field {field} must be a JSON string"),
        });
    }
    parse_json_string(object, value_start)
}

pub(super) fn optional_string_field(
    object: &str,
    field: &'static str,
) -> Result<Option<String>, CloudIacReleaseIndexError> {
    if find_field_position(object, field).is_none() {
        return Ok(None);
    }
    required_string_field(object, field).map(Some)
}

pub(super) fn quoted_field_token(field: &str) -> String {
    format!("\"{field}\"")
}

pub(super) fn find_field_position(input: &str, field: &str) -> Option<usize> {
    input.find(&quoted_field_token(field))
}

pub(super) fn first_non_whitespace_byte(input: &str, start: usize) -> Option<usize> {
    input[start..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(start + offset))
}

pub(super) fn find_next_non_string_char(input: &str, start: usize, target: char) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in input[start..].char_indices() {
        let index = start + offset;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            ch if ch == target => return Some(index),
            _ => {}
        }
    }
    None
}

pub(super) fn matching_delimiter(
    input: &str,
    open_at: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in input[open_at..].char_indices() {
        let index = open_at + offset;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth = depth.saturating_add(1);
        } else if ch == close {
            if depth == 0 {
                return None;
            }
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(super) fn parse_json_string(
    input: &str,
    quote_at: usize,
) -> Result<String, CloudIacReleaseIndexError> {
    let mut output = String::new();
    let mut escaped = false;
    for (offset, ch) in input[quote_at + 1..].char_indices() {
        let index = quote_at + 1 + offset;
        if escaped {
            match ch {
                '"' | '\\' | '/' => output.push(ch),
                'b' => output.push('\u{0008}'),
                'f' => output.push('\u{000c}'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'u' => {
                    return Err(CloudIacReleaseIndexError::MalformedJson {
                        reason:
                            "unicode escapes are outside the local release-index loader contract"
                                .to_string(),
                    });
                }
                _ => {
                    return Err(CloudIacReleaseIndexError::MalformedJson {
                        reason: "invalid JSON string escape".to_string(),
                    });
                }
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Ok(output),
            ch if ch.is_control() => {
                return Err(CloudIacReleaseIndexError::MalformedJson {
                    reason: "JSON string contains an unescaped control character".to_string(),
                });
            }
            _ => output.push(ch),
        }

        if index >= input.len() {
            break;
        }
    }
    Err(CloudIacReleaseIndexError::MalformedJson {
        reason: "JSON string is not closed".to_string(),
    })
}

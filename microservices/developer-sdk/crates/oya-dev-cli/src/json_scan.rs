pub(crate) fn extract_json_objects(array: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut start = None;
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in array.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0
                    && let Some(start) = start.take()
                {
                    objects.push(&array[start..=index]);
                }
            }
            _ => {}
        }
    }
    objects
}

pub(crate) fn parse_json_string_field(object: &str, key: &str) -> Option<String> {
    let key_index = object.find(&format!("\"{key}\""))?;
    let after_key = &object[key_index..];
    let colon_index = after_key.find(':')?;
    parse_json_string_value(&after_key[colon_index + 1..])
}

pub(crate) fn parse_json_string_array_field(object: &str, key: &str) -> Option<Vec<String>> {
    let key_index = object.find(&format!("\"{key}\""))?;
    let after_key = &object[key_index..];
    let colon_index = after_key.find(':')?;
    let after_colon = after_key[colon_index + 1..].trim_start();
    if !after_colon.starts_with('[') {
        return None;
    }
    let array_end = find_matching_json_delimiter(after_colon, '[', ']')?;
    let array = &after_colon[1..array_end];
    let mut values = Vec::new();
    let mut rest = array;
    while let Some(quote_index) = rest.find('"') {
        let value_start = &rest[quote_index..];
        let value = parse_json_string_value(value_start)?;
        values.push(value.clone());
        let consumed = quoted_json_len(value_start)?;
        rest = &value_start[consumed..];
    }
    Some(values)
}

pub(crate) fn parse_json_string_value(value: &str) -> Option<String> {
    let value = value.trim_start();
    if !value.starts_with('"') {
        return None;
    }
    let mut output = String::new();
    let mut characters = value[1..].chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            match characters.next()? {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                '/' => output.push('/'),
                'b' => output.push('\u{0008}'),
                'f' => output.push('\u{000c}'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'u' => output.push(decode_json_unicode_escape(&mut characters)?),
                _ => return None,
            }
        } else if character == '"' {
            return Some(output);
        } else {
            output.push(character);
        }
    }
    None
}

fn decode_json_unicode_escape(characters: &mut impl Iterator<Item = char>) -> Option<char> {
    let mut value = 0_u32;
    for _ in 0..4 {
        value = (value << 4) + characters.next()?.to_digit(16)?;
    }
    char::from_u32(value)
}

pub(crate) fn quoted_json_len(value: &str) -> Option<usize> {
    let mut escaped = false;
    for (index, character) in value.char_indices().skip(1) {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some(index + 1);
        }
    }
    None
}

pub(crate) fn find_matching_json_delimiter(value: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            character if character == open => depth += 1,
            character if character == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn extract_json_object_for_key<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    let key_index = contents.find(&format!("\"{key}\""))?;
    let after_key = &contents[key_index..];
    let object_start = after_key.find('{')?;
    let object = &after_key[object_start..];
    let object_end = find_matching_json_delimiter(object, '{', '}')?;
    Some(&object[1..object_end])
}

pub(crate) fn extract_json_array_for_key<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    let key_index = contents.find(&format!("\"{key}\""))?;
    let after_key = &contents[key_index..];
    let array_start = after_key.find('[')?;
    let array = &after_key[array_start..];
    let array_end = find_matching_json_delimiter(array, '[', ']')?;
    Some(&array[1..array_end])
}

pub(crate) fn extract_json_object_entries(object: &str) -> Vec<(String, &str)> {
    let mut entries = Vec::new();
    let mut rest = object;
    while let Some(key_quote_index) = rest.find('"') {
        let key_start = &rest[key_quote_index..];
        let Some(key) = parse_json_string_value(key_start) else {
            break;
        };
        let Some(key_len) = quoted_json_len(key_start) else {
            break;
        };
        let after_key = &key_start[key_len..];
        let Some(object_start) = after_key.find('{') else {
            break;
        };
        let entry_object = &after_key[object_start..];
        let Some(object_end) = find_matching_json_delimiter(entry_object, '{', '}') else {
            break;
        };
        entries.push((key, &entry_object[..=object_end]));
        rest = &entry_object[object_end + 1..];
    }
    entries
}

pub(crate) fn json_field_has_non_empty_value(object: &str, key: &str) -> bool {
    let Some(key_index) = object.find(&format!("\"{key}\"")) else {
        return false;
    };
    let after_key = &object[key_index..];
    let Some(colon_index) = after_key.find(':') else {
        return false;
    };
    let value = after_key[colon_index + 1..].trim_start();
    if value.starts_with('"') {
        return parse_json_string_value(value)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
    }
    if value.starts_with('[') {
        return !matches!(
            find_matching_json_delimiter(value, '[', ']'),
            Some(1) | None
        );
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_string_value_decodes_json_escapes() {
        assert_eq!(
            parse_json_string_value(r#""\uacf5\uacf5\uc815\ubcf4\ubc95""#),
            Some("공공정보법".into())
        );
        assert_eq!(
            parse_json_string_value(r#""quote: \" slash: \\ newline:\n""#),
            Some("quote: \" slash: \\ newline:\n".into())
        );
    }
}

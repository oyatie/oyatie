use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};

pub(super) fn decode_command(parts: &mut [String], utf8: bool) -> Option<()> {
    let verb = parts.get(1)?.to_ascii_uppercase();
    // LIST and LSUB decode their reference and patterns after tokenising.
    let positions: &[usize] = match verb.as_str() {
        "RENAME" => &[2, 3],
        "CREATE" | "DELETE" | "SELECT" | "EXAMINE" | "STATUS" | "SUBSCRIBE" | "UNSUBSCRIBE"
        | "APPEND" => &[2],
        "COPY" | "MOVE" => &[3],
        "UID"
            if parts.get(2).is_some_and(|p| {
                p.eq_ignore_ascii_case("COPY") || p.eq_ignore_ascii_case("MOVE")
            }) =>
        {
            &[4]
        }
        _ => &[],
    };
    for &index in positions {
        let value = parts.get_mut(index)?;
        if !utf8 {
            *value = decode(value)?;
        }
    }
    Some(())
}

pub(super) fn quote(value: &str, utf8: bool) -> String {
    let value = if utf8 {
        value.to_owned()
    } else {
        encode(value)
    };
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn encode(value: &str) -> String {
    let mut output = String::new();
    let mut encoded = Vec::new();
    let flush = |encoded: &mut Vec<u8>, output: &mut String| {
        if !encoded.is_empty() {
            output.push('&');
            output.push_str(&STANDARD_NO_PAD.encode(&*encoded).replace('/', ","));
            output.push('-');
            encoded.clear();
        }
    };
    for c in value.chars() {
        if c.is_ascii() {
            flush(&mut encoded, &mut output);
            output.push(c);
            if c == '&' {
                output.push('-');
            }
        } else {
            for unit in c.encode_utf16(&mut [0; 2]) {
                encoded.extend_from_slice(&unit.to_be_bytes());
            }
        }
    }
    flush(&mut encoded, &mut output);
    output
}

pub(super) fn decode(mut input: &str) -> Option<String> {
    if !input.is_ascii() {
        return None;
    }
    let original = input;
    let mut output = String::new();
    while let Some((plain, rest)) = input.split_once('&') {
        output.push_str(plain);
        let (encoded, rest) = rest.split_once('-')?;
        if encoded.is_empty() {
            output.push('&');
        } else {
            if encoded
                .bytes()
                .any(|b| !b.is_ascii_alphanumeric() && !b"+,".contains(&b))
            {
                return None;
            }
            let bytes = STANDARD_NO_PAD.decode(encoded.replace(',', "/")).ok()?;
            let (pairs, remainder) = bytes.as_chunks::<2>();
            if !remainder.is_empty() {
                return None;
            }
            let units = pairs
                .iter()
                .map(|p| u16::from_be_bytes(*p))
                .collect::<Vec<_>>();
            output.push_str(&String::from_utf16(&units).ok()?);
        }
        input = rest;
    }
    output.push_str(input);
    (encode(&output) == original && !output.chars().any(char::is_control)).then_some(output)
}

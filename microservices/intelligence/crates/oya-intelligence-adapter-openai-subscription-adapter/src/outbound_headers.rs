//! Outbound request header construction for OpenAI API calls.
//! OpenAI uses `Authorization: Bearer <key>` — NOT `x-api-key`.
// data_class: INTERNAL_ONLY throughout this module.

/// Build the outbound `Authorization: Bearer` header for an OpenAI API call.
///
/// Returns a `Vec<(name, value)>` suitable for injection on proxy calls.
/// SECURITY: The returned vector contains the raw key; callers must not log it.
pub fn openai_auth_headers(api_key: &str) -> Vec<(String, String)> {
    vec![(
        "authorization".to_owned(),
        format!("Bearer {api_key}"),
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_scheme_used() {
        let hdrs = openai_auth_headers("sk-test-key-123");
        let map: std::collections::BTreeMap<_, _> = hdrs.into_iter().collect();
        assert!(
            map["authorization"].starts_with("Bearer "),
            "must use Bearer scheme, got: {}",
            map["authorization"]
        );
        assert_eq!(map["authorization"], "Bearer sk-test-key-123");
    }

    #[test]
    fn no_x_api_key_header() {
        let hdrs = openai_auth_headers("sk-test");
        let map: std::collections::BTreeMap<_, _> = hdrs.into_iter().collect();
        assert!(
            !map.contains_key("x-api-key"),
            "must NOT use x-api-key header"
        );
    }

    #[test]
    fn no_anthropic_version_header() {
        let hdrs = openai_auth_headers("sk-test");
        let map: std::collections::BTreeMap<_, _> = hdrs.into_iter().collect();
        assert!(!map.contains_key("anthropic-version"));
        assert!(!map.contains_key("anthropic-beta"));
    }

    #[test]
    fn exactly_one_header() {
        let hdrs = openai_auth_headers("sk-test");
        assert_eq!(hdrs.len(), 1);
    }
}

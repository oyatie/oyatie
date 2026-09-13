pub fn bearer_token(header: Option<&str>) -> Option<&str> {
    header?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

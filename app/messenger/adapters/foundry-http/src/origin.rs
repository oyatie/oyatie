use messenger_domain::Error;

pub(crate) fn service_origin(value: &str) -> Result<reqwest::Url, Error> {
    let url =
        reqwest::Url::parse(value).map_err(|_| Error::Invalid("invalid server URL".into()))?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(Error::Invalid(
            "use an HTTPS server origin (HTTP only on loopback)".into(),
        ));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::service_origin;

    #[test]
    fn remote_http_and_non_origins_are_refused() {
        for url in [
            "http://example.org",
            "https://u:p@example.org",
            "https://example.org/a",
            "file:///tmp/a",
        ] {
            assert!(service_origin(url).is_err(), "{url}");
        }
        for url in [
            "https://foundry.example",
            "http://127.0.0.1:8008",
            "http://[::1]:8008",
        ] {
            assert!(service_origin(url).is_ok(), "{url}");
        }
    }
}

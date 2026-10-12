use std::net::IpAddr;

#[derive(Clone, Debug)]
pub struct HumanSessionConfig {
    pub(super) base: reqwest::Url, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanSessionConfigError;

impl std::fmt::Display for HumanSessionConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            "IAM transport requires an exact HTTPS origin or explicitly enabled loopback HTTP",
        )
    }
}

impl std::error::Error for HumanSessionConfigError {}

impl HumanSessionConfig {
    pub fn https(base: &str) -> Result<Self, HumanSessionConfigError> {
        Self::parse(base, false)
    }

    /// Explicit local test/development transport; never admits non-loopback HTTP.
    pub fn loopback_http(base: &str) -> Result<Self, HumanSessionConfigError> {
        Self::parse(base, true)
    }

    fn parse(base: &str, allow_loopback_http: bool) -> Result<Self, HumanSessionConfigError> {
        let (_, remainder) = base.split_once("://").ok_or(HumanSessionConfigError)?;
        let authority_end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
        let (authority, suffix) = remainder.split_at(authority_end);
        if base.trim() != base
            || base.contains('\\')
            || authority.contains('@')
            || !matches!(suffix, "" | "/")
        {
            return Err(HumanSessionConfigError);
        }
        let url = reqwest::Url::parse(base).map_err(|_| HumanSessionConfigError)?;
        let loopback = url.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .trim_matches(['[', ']'])
                    .parse::<IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
        if !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
            || url.host_str().is_none()
            || !(url.scheme() == "https"
                || (allow_loopback_http && url.scheme() == "http" && loopback))
        {
            return Err(HumanSessionConfigError);
        }
        Ok(Self { base: url })
    }

    /// Absent configuration keeps the human route disabled. Malformed explicit
    /// configuration refuses startup rather than silently enabling a fallback.
    pub fn from_env() -> Result<Option<Self>, HumanSessionConfigError> {
        Self::from_values(
            std::env::var("OYATIE_FOUNDRY_IAM_BASE_URL")
                .map(Some)
                .or_else(|e| match e {
                    std::env::VarError::NotPresent => Ok(None),
                    _ => Err(HumanSessionConfigError),
                })?
                .as_deref(),
            std::env::var("OYATIE_FOUNDRY_IAM_ALLOW_LOOPBACK_HTTP")
                .map(Some)
                .or_else(|e| match e {
                    std::env::VarError::NotPresent => Ok(None),
                    _ => Err(HumanSessionConfigError),
                })?
                .as_deref(),
        )
    }

    fn from_values(
        base: Option<&str>,
        local: Option<&str>,
    ) -> Result<Option<Self>, HumanSessionConfigError> {
        let allow_local = match local {
            None | Some("false") => false,
            Some("true") => true,
            _ => return Err(HumanSessionConfigError),
        };
        match base {
            None if !allow_local => Ok(None),
            None => Err(HumanSessionConfigError),
            Some(base) => Self::parse(base, allow_local).map(Some),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_configuration_is_exact_and_missing_configuration_disables_human_access() {
        assert!(
            HumanSessionConfig::from_values(None, None)
                .unwrap()
                .is_none()
        );
        assert!(HumanSessionConfig::from_values(None, Some("true")).is_err());
        assert!(HumanSessionConfig::from_values(Some("https://iam.example"), Some("yes")).is_err());
        assert!(HumanSessionConfig::https("https://iam.example").is_ok());
        assert!(HumanSessionConfig::loopback_http("http://127.0.0.1:8080").is_ok());
        assert!(HumanSessionConfig::loopback_http("http://[::1]:8080").is_ok());
        for base in [
            "http://iam.example",
            "http://127.0.0.1:8080",
            "https://user@iam.example",
            "https://iam.example/identity",
            "https://iam.example?x=1",
            "https://iam.example#x",
            "ftp://iam.example",
            "https://@iam.example",
            "https://iam.example/foo/..",
            "https://iam.example ",
            "https://iam.example\\",
        ] {
            assert!(HumanSessionConfig::https(base).is_err(), "{base}");
        }
        assert!(HumanSessionConfig::loopback_http("http://iam.example").is_err());
    }
}

use super::*;

struct SecretYaml(serde_yaml::Value);
impl Drop for SecretYaml {
    fn drop(&mut self) {
        fn wipe(v: &mut serde_yaml::Value) {
            match v {
                serde_yaml::Value::String(s) => s.zeroize(),
                serde_yaml::Value::Sequence(items) => items.iter_mut().for_each(wipe),
                serde_yaml::Value::Mapping(items) => items.values_mut().for_each(wipe),
                _ => (),
            }
        }
        wipe(&mut self.0);
    }
}

fn decode(value: &serde_yaml::Value) -> Result<Zeroizing<Vec<u8>>, AccessError> {
    STANDARD
        .decode(value.as_str().ok_or(AccessError::InvalidCredentials)?)
        .map(Zeroizing::new)
        .map_err(|_| AccessError::InvalidCredentials)
}

pub(super) fn check_certificate(
    ca_pem: &[u8],
    crt_pem: &[u8],
    expected_ca: &str,
) -> Result<(), AccessError> {
    if !verified_digest(ca_pem, expected_ca) {
        return Err(AccessError::InvalidCredentials);
    }
    let (_, ca) = parse_x509_pem(ca_pem).map_err(|_| AccessError::InvalidCredentials)?;
    let (_, crt) = parse_x509_pem(crt_pem).map_err(|_| AccessError::InvalidCredentials)?;
    let ca = ca
        .parse_x509()
        .map_err(|_| AccessError::InvalidCredentials)?;
    let crt = crt
        .parse_x509()
        .map_err(|_| AccessError::InvalidCredentials)?;
    if !ca.validity().is_valid() || !crt.validity().is_valid() {
        return Err(AccessError::CertificateExpired);
    }
    crt.verify_signature(Some(ca.public_key()))
        .map_err(|_| AccessError::InvalidCredentials)
}

pub(super) struct Credentials {
    pub(super) talos: Zeroizing<Vec<u8>>,
    pub(super) kube: Zeroizing<Vec<u8>>,
}

pub(super) fn credentials(profile: &Profile, archive: &[u8]) -> Result<Credentials, AccessError> {
    let talos = run(
        "tar",
        &strings(&["-xOf", "-", &profile.talos_member]),
        archive,
        false,
    )?;
    let kube = run(
        "tar",
        &strings(&["-xOf", "-", &profile.kube_member]),
        archive,
        false,
    )?;
    let t =
        SecretYaml(serde_yaml::from_slice(&talos).map_err(|_| AccessError::InvalidCredentials)?);
    if t.0["context"].as_str() != Some(&profile.talos_context) {
        return Err(AccessError::InvalidCredentials);
    }
    let context = &t.0["contexts"][profile.talos_context.as_str()];
    if context.as_mapping().is_none_or(|m| {
        m.keys().any(|k| {
            !matches!(
                k.as_str(),
                Some("ca" | "crt" | "key" | "endpoints" | "nodes")
            )
        })
    }) {
        return Err(AccessError::InvalidCredentials);
    }
    check_certificate(
        &decode(&context["ca"])?,
        &decode(&context["crt"])?,
        &profile.talos_ca_sha256,
    )?;
    if context["key"].as_str().is_none_or(str::is_empty) {
        return Err(AccessError::InvalidCredentials);
    }
    let k = SecretYaml(serde_yaml::from_slice(&kube).map_err(|_| AccessError::InvalidCredentials)?);
    let current = k.0["current-context"]
        .as_str()
        .ok_or(AccessError::InvalidCredentials)?;
    let contexts = k.0["contexts"]
        .as_sequence()
        .ok_or(AccessError::InvalidCredentials)?;
    let matches: Vec<_> = contexts
        .iter()
        .filter(|v| v["name"].as_str() == Some(current))
        .collect();
    if matches.len() != 1 {
        return Err(AccessError::InvalidCredentials);
    }
    let cluster_name = matches[0]["context"]["cluster"]
        .as_str()
        .ok_or(AccessError::InvalidCredentials)?;
    let user_name = matches[0]["context"]["user"]
        .as_str()
        .ok_or(AccessError::InvalidCredentials)?;
    let clusters = k.0["clusters"]
        .as_sequence()
        .ok_or(AccessError::InvalidCredentials)?;
    let users = k.0["users"]
        .as_sequence()
        .ok_or(AccessError::InvalidCredentials)?;
    if clusters.len() != 1
        || users.len() != 1
        || clusters[0]["name"].as_str() != Some(cluster_name)
        || users[0]["name"].as_str() != Some(user_name)
    {
        return Err(AccessError::InvalidCredentials);
    }
    let cluster = &clusters[0]["cluster"];
    let user = &users[0]["user"];
    if cluster["server"].as_str() != Some(&format!("https://{}:6443", profile.private_ip))
        || cluster["insecure-skip-tls-verify"].as_bool() == Some(true)
        || !cluster["proxy-url"].is_null()
        || user.as_mapping().is_none_or(|m| m.len() != 2)
        || user["client-key-data"].as_str().is_none_or(str::is_empty)
    {
        return Err(AccessError::InvalidCredentials);
    }
    check_certificate(
        &decode(&cluster["certificate-authority-data"])?,
        &decode(&user["client-certificate-data"])?,
        &profile.kube_ca_sha256,
    )?;
    Ok(Credentials { talos, kube })
}

use rustls::{ServerConfig, ServerConnection, crypto::aws_lc_rs, pki_types::PrivatePkcs8KeyDer};
use std::sync::Arc;

// Exercise public TLS record ingestion using the same provider as the mail
// listeners/relay. The malformed retry cases follow rustls0.23.45's maintained
// server regressions, encoded here without private rustls message APIs.
const AES128: u16 = 0x1301;
const CHACHA20: u16 = 0x1303;
const TLS12_ECDSA_AES128: u16 = 0xc02b;
const HRR_RANDOM: [u8; 32] = [
    0xcf, 0x21, 0xad, 0x74, 0xe5, 0x9a, 0x61, 0x11, 0xbe, 0x1d, 0x8c, 0x02, 0x1e, 0x65, 0xb8, 0x91,
    0xc2, 0xa2, 0x11, 0x16, 0x7a, 0xbb, 0x8c, 0x5e, 0x07, 0x9e, 0x09, 0xe2, 0xc8, 0xa8, 0x33, 0x9c,
];

fn server() -> ServerConnection {
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let config = ServerConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider()))
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.der().to_vec().into()],
            PrivatePkcs8KeyDer::from(signing_key.serialize_der()).into(),
        )
        .unwrap();
    ServerConnection::new(Arc::new(config)).unwrap()
}

fn vector(bytes: &[u8]) -> Vec<u8> {
    let mut encoded = u16::try_from(bytes.len()).unwrap().to_be_bytes().to_vec();
    encoded.extend_from_slice(bytes);
    encoded
}

fn extension(output: &mut Vec<u8>, kind: u16, bytes: &[u8]) {
    output.extend_from_slice(&kind.to_be_bytes());
    output.extend_from_slice(&vector(bytes));
}

fn hello(suite: u16, key_share: bool, tls13: bool, psk: bool) -> Vec<u8> {
    let mut payload = vec![3, 3]; // legacy_version TLS1.2
    payload.extend_from_slice(&[0x42; 32]);
    payload.push(0); // empty legacy_session_id
    payload.extend_from_slice(&vector(&suite.to_be_bytes()));
    payload.extend_from_slice(&[1, 0]); // only null compression
    let mut extensions = Vec::new();
    extension(
        &mut extensions,
        43,
        if tls13 { &[4, 3, 4, 3, 3] } else { &[2, 3, 3] },
    );
    extension(&mut extensions, 10, &vector(&[0, 0x1d, 0, 0x17])); // X25519,P256
    extension(&mut extensions, 13, &vector(&[4, 3, 8, 4])); // ECDSA/RSA SHA256
    extension(&mut extensions, 23, &[]); // extended_master_secret (TLS1.2)
    let mut shares = Vec::new();
    if key_share {
        shares.extend_from_slice(&[0, 0x1d]);
        shares.extend_from_slice(&vector(&[0xab; 32]));
    }
    extension(&mut extensions, 51, &vector(&shares));
    extension(&mut extensions, 45, &[1, 1]); // PSK with DHE only
    if psk {
        let mut identity = vector(&[0; 16]);
        identity.extend_from_slice(&0u32.to_be_bytes());
        let mut offer = vector(&identity);
        let mut binder = vec![32];
        binder.extend_from_slice(&[0; 32]);
        offer.extend_from_slice(&vector(&binder));
        extension(&mut extensions, 41, &offer); // PSK must be the final extension
    }
    payload.extend_from_slice(&vector(&extensions));
    let mut handshake = vec![1]; // ClientHello
    handshake.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes()[1..]);
    handshake.extend_from_slice(&payload);
    let mut record = vec![22, 3, 3]; // plaintext handshake record
    record.extend_from_slice(&vector(&handshake));
    record
}

fn retry_server(psk: bool) -> ServerConnection {
    let mut server = server();
    server
        .read_tls(&mut hello(AES128, false, true, psk).as_slice())
        .unwrap();
    server
        .process_new_packets()
        .expect("first hello must request a supported key share");
    let mut response = Vec::new();
    server.write_tls(&mut response).unwrap();
    assert!(
        response.windows(32).any(|bytes| bytes == HRR_RANDOM),
        "server must emit HelloRetryRequest"
    );
    server
}

fn reject_retry(mut server: ServerConnection, hello: Vec<u8>, expected: &str) {
    server.read_tls(&mut hello.as_slice()).unwrap();
    let result = server.process_new_packets();
    assert!(
        result.is_err(),
        "changed retry must be rejected before the server flight"
    );
    assert_eq!(format!("{:?}", result.unwrap_err()), expected);
    let mut alert = Vec::new();
    server.write_tls(&mut alert).unwrap();
    assert_eq!(alert.first(), Some(&21), "refusal must emit a TLS alert");
    assert_eq!(alert.get(5), Some(&2), "retry violation must be fatal");
}

#[test]
fn valid_hello_retry_keeps_the_negotiated_cipher_suite() {
    let mut server = retry_server(false);
    server
        .read_tls(&mut hello(AES128, true, true, false).as_slice())
        .unwrap();
    server
        .process_new_packets()
        .expect("a valid retry must progress to the server handshake flight");
    assert_eq!(
        server.negotiated_cipher_suite().unwrap().suite(),
        rustls::CipherSuite::TLS13_AES_128_GCM_SHA256
    );
    assert_eq!(
        server.protocol_version(),
        Some(rustls::ProtocolVersion::TLSv1_3)
    );
}

#[test]
fn hello_retry_cannot_change_the_negotiated_cipher_suite() {
    reject_retry(
        retry_server(false),
        hello(CHACHA20, true, true, false),
        "PeerMisbehaved(CipherSuiteDifferedOnRetry)",
    );
}

#[test]
fn hello_retry_cannot_downgrade_to_tls12() {
    reject_retry(
        retry_server(false),
        hello(TLS12_ECDSA_AES128, true, false, false),
        "PeerIncompatible(Tls12NotOfferedOrEnabled)",
    );
}

#[test]
fn hello_retry_cannot_withdraw_the_initial_psk_offer() {
    reject_retry(
        retry_server(true),
        hello(AES128, true, true, false),
        "PeerMisbehaved(MissingPskExtensionInSecondClientHello)",
    );
}

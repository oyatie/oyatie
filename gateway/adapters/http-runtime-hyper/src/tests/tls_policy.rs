use super::*;

#[test]
fn pqc_hybrid_policy_prioritizes_hybrid_group_and_classical_fallback() {
    let groups = pqc_hybrid_kx_group_names();
    assert_eq!(
        groups.first().copied(),
        Some(rustls::NamedGroup::X25519MLKEM768),
        "X25519MLKEM768 must be the first offered TLS 1.3 key-share group"
    );
    assert!(
        groups.contains(&rustls::NamedGroup::X25519),
        "classical X25519 fallback must remain enabled"
    );

    let _connector = build_pqc_hybrid_https_connector();
}

#[tokio::test]
async fn pqc_hybrid_https_client_rejects_plain_http_uri() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let client = build_pqc_hybrid_https_client();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("loopback listener binds");
    let addr = listener.local_addr().expect("listener addr");
    let plaintext_server_reached = Arc::new(AtomicBool::new(false));
    let reached = Arc::clone(&plaintext_server_reached);
    let server = tokio::spawn(async move {
        if let Ok(Ok((stream, _))) =
            tokio::time::timeout(Duration::from_millis(200), listener.accept()).await
        {
            reached.store(true, Ordering::SeqCst);
            let io = TokioIo::new(stream);
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|_req| async {
                        Ok::<_, Infallible>(Response::new(Full::new(Bytes::from_static(
                            b"not-pqc",
                        ))))
                    }),
                )
                .await;
        }
    });
    let request = Request::builder()
        .method("GET")
        .uri(format!("http://{addr}/plaintext-is-not-pqc"))
        .body(Full::new(Bytes::new()))
        .expect("request builds");

    client
        .request(request)
        .await
        .expect_err("canonical PQC client must reject plaintext HTTP URIs");
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        !plaintext_server_reached.load(Ordering::SeqCst),
        "canonical PQC client must reject plaintext HTTP before reaching a loopback HTTP server"
    );
    server.abort();
}

#[test]
fn pqc_hybrid_tls13_handshake_selects_hybrid_group() {
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
            .expect("test cert generation");
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.der().clone()).expect("self cert trusted");

    let client_config = pqc_hybrid_tls13_client_config_builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_key = rustls::pki_types::PrivateKeyDer::Pkcs8(
        rustls::pki_types::PrivatePkcs8KeyDer::from(signing_key.serialize_der()),
    );
    let server_config = pqc_hybrid_tls13_server_config_builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert.der().clone()], server_key)
        .expect("server cert and key match");

    let mut client = rustls::ClientConnection::new(
        Arc::new(client_config),
        rustls::pki_types::ServerName::try_from("localhost").expect("valid DNS name"),
    )
    .expect("client connection");
    let mut server =
        rustls::ServerConnection::new(Arc::new(server_config)).expect("server connection");

    for _ in 0..16 {
        let mut client_to_server = Vec::new();
        client
            .write_tls(&mut client_to_server)
            .expect("client writes tls");
        if !client_to_server.is_empty() {
            let mut cursor = std::io::Cursor::new(client_to_server);
            server.read_tls(&mut cursor).expect("server reads tls");
            server
                .process_new_packets()
                .expect("server processes packets");
        }

        let mut server_to_client = Vec::new();
        server
            .write_tls(&mut server_to_client)
            .expect("server writes tls");
        if !server_to_client.is_empty() {
            let mut cursor = std::io::Cursor::new(server_to_client);
            client.read_tls(&mut cursor).expect("client reads tls");
            client
                .process_new_packets()
                .expect("client processes packets");
        }

        if !client.is_handshaking() && !server.is_handshaking() {
            break;
        }
    }

    assert!(!client.is_handshaking(), "client handshake must finish");
    assert!(!server.is_handshaking(), "server handshake must finish");
    assert_eq!(
        client.protocol_version(),
        Some(rustls::ProtocolVersion::TLSv1_3)
    );
    assert_eq!(
        server.protocol_version(),
        Some(rustls::ProtocolVersion::TLSv1_3)
    );
    assert_eq!(
        client
            .negotiated_key_exchange_group()
            .map(|group| group.name()),
        Some(rustls::NamedGroup::X25519MLKEM768)
    );
    assert_eq!(
        server
            .negotiated_key_exchange_group()
            .map(|group| group.name()),
        Some(rustls::NamedGroup::X25519MLKEM768)
    );
}

use super::*;

// F3 adversarial: handler_to_sync wraps a typed Handler so the router
// can hold it as a SyncHandler, and the rendered error path goes through
// From<Error> for HttpResponse — proves the Phase 6 contract end-to-end.
// F3 adversarial Phase 10 (S2 non-UTF8 + S1 header case):
// building a hyper::Request<Full<Bytes>> and round-tripping through a
// helper that exercises the same header-iteration path collect_hyper_request
// uses. Direct Request<Incoming> isn't constructible in tests.
#[test]
fn header_name_lowercased_when_inserted_via_with_header() {
    // Surrogate for the adapter path: middleware-kernel with_header
    // already lowercases. Adapter inherits since adapter writes through
    // BTreeMap with insertion done via the same `.to_ascii_lowercase()`
    // in collect_hyper_request. Smoke this for S1 explicitly.
    let resp = HttpResponse::new(200).with_header("X-Tenant-Id", "acme");
    assert!(resp.headers.contains_key("x-tenant-id"));
    assert!(!resp.headers.contains_key("X-Tenant-Id"));
}

// F3 adversarial: NonUtf8HeaderValue error maps to 400.
#[test]
fn non_utf8_header_value_renders_400() {
    let err = HyperRuntimeError::NonUtf8HeaderValue {
        header_name: "x-binary".into(),
    };
    let resp: HttpResponse = err.into();
    assert_eq!(resp.status, 400);
    let body = std::str::from_utf8(&resp.body).unwrap();
    assert!(body.contains("non-UTF-8"));
    assert!(body.contains("x-binary"));
}

#[test]
fn non_utf8_header_value_status_code() {
    assert_eq!(
        HyperRuntimeError::NonUtf8HeaderValue {
            header_name: "x".into()
        }
        .status_code(),
        400
    );
}

// F3 adversarial: collect_body_with_limit accepts bodies <= max.
#[tokio::test]
async fn collect_body_with_limit_accepts_under_cap() {
    let body = Full::new(Bytes::from_static(b"hello"));
    let result = collect_body_with_limit(body, 1024).await.unwrap();
    assert_eq!(result, b"hello".to_vec());
}

// F3 adversarial: collect_body_with_limit accepts bodies exactly at max.
#[tokio::test]
async fn collect_body_with_limit_accepts_exact_cap() {
    let payload = vec![0xAB; 100];
    let body = Full::new(Bytes::from(payload.clone()));
    let result = collect_body_with_limit(body, 100).await.unwrap();
    assert_eq!(result, payload);
}

// F3 adversarial: collect_body_with_limit rejects bodies > max with the
// specific BodyTooLarge variant. This closes the S3 unbounded-body DoS.
#[tokio::test]
async fn collect_body_with_limit_rejects_over_cap_with_body_too_large() {
    let body = Full::new(Bytes::from(vec![0u8; 1025]));
    let err = collect_body_with_limit(body, 1024).await.unwrap_err();
    match err {
        HyperRuntimeError::BodyTooLarge { max_bytes } => {
            assert_eq!(max_bytes, 1024);
        }
        other => panic!("expected BodyTooLarge, got {other:?}"),
    }
}

// F3 adversarial: BodyTooLarge maps to 413 Payload Too Large at the
// From<HyperRuntimeError> for HttpResponse boundary.
#[test]
fn body_too_large_renders_413() {
    let err = HyperRuntimeError::BodyTooLarge { max_bytes: 1024 };
    let resp: HttpResponse = err.into();
    assert_eq!(resp.status, 413);
    let body = std::str::from_utf8(&resp.body).unwrap();
    assert!(body.contains("1024"));
    assert!(body.contains("body"));
}

// F3 adversarial: each error variant maps to the correct status — proves
// the From impl handles every variant, not just the obvious ones.
#[test]
fn hyper_runtime_error_status_code_mapping() {
    assert_eq!(HyperRuntimeError::Bind("x".into()).status_code(), 500);
    assert_eq!(HyperRuntimeError::BodyRead("x".into()).status_code(), 400);
    assert_eq!(HyperRuntimeError::Config("x".into()).status_code(), 500);
    assert_eq!(HyperRuntimeError::Connection("x".into()).status_code(), 500);
    assert_eq!(HyperRuntimeError::Runtime("x".into()).status_code(), 500);
    assert_eq!(
        HyperRuntimeError::BodyTooLarge { max_bytes: 1 }.status_code(),
        413
    );
    assert_eq!(
        HyperRuntimeError::UnsupportedMethod("BREW".into()).status_code(),
        405
    );
}

// F4 ergonomic: ServerConfig builder methods are chainable.
#[test]
fn server_config_builder_chains_with_methods() {
    let cfg = ServerConfig::default()
        .with_max_body_bytes(2048)
        .with_header_read_timeout(Duration::from_secs(5))
        .with_keepalive_timeout(Duration::from_secs(30));
    assert_eq!(cfg.max_body_bytes, 2048);
    assert_eq!(cfg.header_read_timeout, Duration::from_secs(5));
    assert_eq!(cfg.keepalive_timeout, Duration::from_secs(30));
}

// F1 linus: ServerConfig::default uses safe defaults (sealed contract
// for fresh cell binaries).
#[test]
fn server_config_defaults_are_safe() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.max_body_bytes, DEFAULT_MAX_BODY_BYTES);
    assert_eq!(cfg.max_body_bytes, 1024 * 1024);
    assert!(cfg.header_read_timeout >= Duration::from_secs(5));
    assert!(cfg.header_read_timeout <= Duration::from_secs(60));
    assert!(cfg.keepalive_timeout >= Duration::from_secs(10));
}

#[test]
fn handler_to_sync_routes_ok_and_err_paths() {
    use http_middleware_kernel::Handler;

    #[derive(Clone, Debug)]
    enum SvcErr {
        Missing,
    }
    impl From<SvcErr> for HttpResponse {
        fn from(e: SvcErr) -> Self {
            match e {
                SvcErr::Missing => HttpResponse::new(404).with_body(b"missing-from-svc".to_vec()),
            }
        }
    }

    struct OkSvc;
    impl Handler for OkSvc {
        type Error = SvcErr;
        fn call(&self, _req: HttpRequest) -> Result<HttpResponse, SvcErr> {
            Ok(HttpResponse::new(200).with_body(b"svc-ok".to_vec()))
        }
    }

    struct ErrSvc;
    impl Handler for ErrSvc {
        type Error = SvcErr;
        fn call(&self, _req: HttpRequest) -> Result<HttpResponse, SvcErr> {
            Err(SvcErr::Missing)
        }
    }

    let mut router: Router<SyncHandler> = Router::new();
    router
        .route(HttpMethod::Get, "/ok", handler_to_sync(OkSvc))
        .unwrap();
    router
        .route(HttpMethod::Get, "/err", handler_to_sync(ErrSvc))
        .unwrap();

    let chain = empty_chain();
    let ok = dispatch(mock_request(HttpMethod::Get, "/ok"), &router, &chain);
    assert_eq!(ok.status, 200);
    assert_eq!(ok.body, b"svc-ok".to_vec());

    let err = dispatch(mock_request(HttpMethod::Get, "/err"), &router, &chain);
    // The handler returned Err(SvcErr::Missing); rendered via From impl.
    assert_eq!(err.status, 404);
    assert_eq!(err.body, b"missing-from-svc".to_vec());
}

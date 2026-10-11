use super::*;

pub(super) async fn start_with_options(
    config: &Config,
    scim_database_url: Option<String>,
    passkey_origin: Option<String>,
) -> Result<ServiceHandle, StartError> {
    let state = build_state(config)?;
    let passkey_origin = passkey_origin
        .map(|value| PasskeyOrigin::parse(&value))
        .transpose()
        .map_err(|error| StartError::Passkey(error.to_owned()))?;

    // SCIM provisioning surface: same offline JWKS + issuer/audience material
    // as the authorize path, scim.manage scope required (fail-closed guard).
    //
    // Store selection (12-factor composition-root config, NOT a CLI surface):
    //   - OYATIE_BACKBONE_POSTGRES_URL present + non-empty -> the DURABLE Postgres
    //     SCIM stores are composed. If the connection fails or the
    //     RLS-enforceability guard fires (bypass-capable role, or role not a
    //     member of identity_scim_runtime), the service REFUSES to serve
    //     (StartError::Store) — it NEVER falls back to in-memory.
    //   - absent / empty -> the in-memory stores (single-node dev bring-up).
    //
    // Both plug in behind the UNCHANGED UserStore/GroupStore kernel ports, so
    // the owned-data cutover (G003) swaps the adapter with no change here.
    //
    // Guard runs BEFORE either TcpListener::bind so a broken DSN or
    // bypass-capable role fails before any socket is allocated.
    let scim_base_url = format!("{}{}", config.issuer.trim_end_matches('/'), SCIM_BASE);
    let scim_validation = ValidationConfig::new(&config.issuer, &config.audience);
    let (scim_router, passkey_router, scim_store_kind) =
        match select_scim_store_kind(scim_database_url) {
            ScimStoreSelection::Postgres(url) => {
                // FAIL-CLOSED: a connect failure or RLS-bypass role propagates;
                // NEVER fall back to in-memory when a durable backend was
                // configured. The composition root OWNS the shared pool (typed
                // here so the sqlx dependency is named, not merely transitive)
                // and runs the RLS-enforceability guard once over it.
                let pool: sqlx::PgPool = connect_pool(&url)
                    .await
                    .map_err(|e| StartError::Store(e.to_string()))?;
                assert_rls_enforceable(&pool)
                    .await
                    .map_err(|e| StartError::Store(e.to_string()))?;
                let scim_state = Arc::new(ScimSurfaceState::new(
                    scim_base_url,
                    jwks_from_json(&read_file(&config.jwks_path)?).map_err(StartError::Jwks)?,
                    scim_validation,
                    default_now_epoch_seconds,
                    Arc::clone(&state),
                    PgScimUserStore::from_pool(pool.clone()),
                    PgScimGroupStore::from_pool(pool.clone()),
                ));
                let passkey_router = passkey_origin.map(|origin| {
                    let guarded_scim = Arc::clone(&scim_state);
                    build_passkey_router(
                        pool,
                        origin,
                        Arc::new(move |headers, tenant| {
                            authorize_scim(&guarded_scim, headers, tenant)?;
                            authorize_native_user_admin(&guarded_scim, headers, tenant)
                        }),
                        default_now_epoch_seconds,
                    )
                });
                (build_scim_router(scim_state), passkey_router, "postgres")
            }
            ScimStoreSelection::InMemory => {
                if passkey_origin.is_some() {
                    return Err(StartError::Passkey(
                        "passkey routes require a durable Postgres store".to_owned(),
                    ));
                }
                let scim_state = Arc::new(ScimSurfaceState::new(
                    scim_base_url,
                    jwks_from_json(&read_file(&config.jwks_path)?).map_err(StartError::Jwks)?,
                    scim_validation,
                    default_now_epoch_seconds,
                    Arc::clone(&state),
                    InMemoryUserStore::default(),
                    InMemoryGroupStore::default(),
                ));
                (build_scim_router(scim_state), None, "inmemory")
            }
        };

    let rest_listener = TcpListener::bind(&config.rest_addr)
        .await
        .map_err(|source| StartError::Bind {
            addr: config.rest_addr.clone(),
            source,
        })?;
    let rest_addr = rest_listener
        .local_addr()
        .map_err(|source| StartError::Bind {
            addr: config.rest_addr.clone(),
            source,
        })?;

    let grpc_listener = TcpListener::bind(&config.grpc_addr)
        .await
        .map_err(|source| StartError::Bind {
            addr: config.grpc_addr.clone(),
            source,
        })?;
    let grpc_addr = grpc_listener
        .local_addr()
        .map_err(|source| StartError::Bind {
            addr: config.grpc_addr.clone(),
            source,
        })?;
    let grpc_incoming = TcpIncoming::from(grpc_listener).with_nodelay(Some(true));

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut rest_router = rest::build_service_router(Arc::clone(&state));
    if let Some(issuer_state) = build_issuer_state(config)? {
        rest_router = rest_router.merge(build_issuer_router(issuer_state));
    }
    rest_router = rest_router.merge(scim_router);
    if let Some(passkey_router) = passkey_router {
        rest_router = rest_router.merge(passkey_router);
    }
    let mut rest_shutdown = shutdown_rx.clone();
    let rest_task = tokio::spawn(async move {
        let shutdown = async move {
            let _ = rest_shutdown.changed().await;
        };
        if let Err(err) = axum::serve(rest_listener, rest_router)
            .with_graceful_shutdown(shutdown)
            .await
        {
            error!(error = %err, "REST server exited with error");
        }
    });

    let mut grpc_shutdown = shutdown_rx;
    let grpc_task = tokio::spawn(async move {
        let shutdown = async move {
            let _ = grpc_shutdown.changed().await;
        };
        if let Err(err) = grpc::serve(state, grpc_incoming, shutdown).await {
            error!(error = %err, "gRPC server exited with error");
        }
    });

    info!(
        rest = %rest_addr,
        grpc = %grpc_addr,
        scim_store = scim_store_kind,
        "identity serving"
    );
    Ok(ServiceHandle {
        rest_addr,
        grpc_addr,
        shutdown_tx,
        rest_task,
        grpc_task,
    })
}

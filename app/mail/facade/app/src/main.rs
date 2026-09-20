#![forbid(unsafe_code)]
mod convert;
mod delivery;
mod https;
mod outbound;
mod outbound_config;
mod session;
mod sweep;
use mail_api::DeliveryQueue;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Semaphore};
use tokio_rustls::TlsAcceptor;

async fn serve(
    database: &str,
    certificate: &str,
    private_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // The schema refusal (below-version, converting, above-version) is decided
    // before any TLS material is read or any listener is bound.
    let db = Arc::new(SqliteStore::open(database)?);
    let certificates =
        CertificateDer::pem_file_iter(certificate)?.collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(private_key)?;
    let tls = Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates, key)?,
    );
    let tls = TlsAcceptor::from(tls);
    let relay = outbound_config::configured()?;
    let service = Arc::new(MailService {
        outbound: relay
            .as_ref()
            .map(|_| db.clone() as Arc<dyn mail_api::SubmissionQueue>),
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    });
    let smtp = bind("MAIL_SMTP_LISTEN", "127.0.0.1:2525").await?;
    let submission = bind("MAIL_SUBMISSION_LISTEN", "127.0.0.1:2465").await?;
    let submission_starttls = bind("MAIL_SUBMISSION_STARTTLS_LISTEN", "127.0.0.1:2587").await?;
    let imap = bind("MAIL_IMAP_LISTEN", "127.0.0.1:1993").await?;
    let imap_starttls = bind("MAIL_IMAP_STARTTLS_LISTEN", "127.0.0.1:2143").await?;
    let https = bind("MAIL_HTTP_LISTEN", "127.0.0.1:8443").await?;
    let pop = bind("MAIL_POP_LISTEN", "127.0.0.1:1995").await?;
    let pop_starttls = bind("MAIL_POP_STARTTLS_LISTEN", "127.0.0.1:2110").await?;
    let public_url = setting(
        "MAIL_PUBLIC_URL",
        &format!("https://localhost:{}", https.local_addr()?.port()),
    )?;
    let public_url = public_url.trim_end_matches('/');
    let uri: axum::http::Uri = public_url.parse()?;
    if uri.scheme_str() != Some("https")
        || uri.host().is_none()
        || uri.path() != "/"
        || uri.query().is_some()
        || uri.authority().is_some_and(|a| a.as_str().contains('@'))
    {
        return Err(
            "MAIL_PUBLIC_URL must be an HTTPS origin without credentials, path or query".into(),
        );
    }
    let capacity = Arc::new(Semaphore::new(128));
    let mut sessions = tokio::task::JoinSet::new();
    let app = mail_protocol_imap::jmap_router(service.clone(), public_url.into());
    eprintln!(
        "mail-app: SMTP {}; IMAPS {}; JMAP {public_url}; SUBMISSIONS {}; IMAP STARTTLS {}; SUBMISSION STARTTLS {}; POPS {}; POP STARTTLS {}",
        smtp.local_addr()?,
        imap.local_addr()?,
        submission.local_addr()?,
        imap_starttls.local_addr()?,
        submission_starttls.local_addr()?,
        pop.local_addr()?,
        pop_starttls.local_addr()?
    );
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let (delivery_stop, delivery_stopped) = tokio::sync::oneshot::channel();
    let mut delivery_task = tokio::spawn(delivery::run(service.clone(), delivery_stopped));
    let (sweep_stop, sweep_stopped) = tokio::sync::oneshot::channel();
    let mut sweep_task = tokio::spawn(sweep::run(service.clone(), sweep_stopped));
    let (outbound_stop, outbound_stopped) = tokio::sync::watch::channel(false);
    let mut outbound_workers = tokio::task::JoinSet::new();
    if let (Some(queue), Some(transport)) = (&service.outbound, relay) {
        for _ in 0..8 {
            outbound_workers.spawn(outbound::run(
                queue.clone(),
                transport.clone(),
                outbound_stopped.clone(),
            ));
        }
    }
    let http_tls = tls.clone();
    let mut http_task = tokio::spawn(async move {
        axum::serve(https::HttpsListener::new(https, http_tls), app)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
    });
    let shutdown = shutdown();
    tokio::pin!(shutdown);
    loop {
        // Completed tasks release their semaphore permits before their join
        // results are removed. Reap first so connection churn cannot retain
        // those results while randomized accepts keep reusing the permits.
        while sessions.try_join_next().is_some() {}
        tokio::select! {
            signal = &mut shutdown => { signal?; break; }
            result = &mut http_task => { result??; return Err("HTTP listener stopped unexpectedly".into()); }
            result = &mut delivery_task => { result?; return Err("Delivery worker stopped unexpectedly".into()); }
            result = &mut sweep_task => { result?; return Err("Blob sweep stopped unexpectedly".into()); }
            Some(result) = outbound_workers.join_next(), if !outbound_workers.is_empty() => {
                result?; return Err("Outbound worker stopped unexpectedly".into());
            }
            connection = smtp.accept() => {
                session::spawn(&mut sessions, connection?.0, &service, &tls, &capacity, session::Protocol::Smtp);
            }
            connection = imap.accept() => {
                session::spawn(&mut sessions, connection?.0, &service, &tls, &capacity, session::Protocol::Imap);
            }
            connection = submission.accept() => {
                session::spawn(&mut sessions, connection?.0, &service, &tls, &capacity, session::Protocol::Submission);
            }
            connection = imap_starttls.accept() => {
                session::spawn(&mut sessions, connection?.0, &service, &tls, &capacity, session::Protocol::ImapStartTls);
            }
            connection = submission_starttls.accept() => {
                session::spawn(&mut sessions, connection?.0, &service, &tls, &capacity, session::Protocol::SubmissionStartTls);
            }
            connection = pop.accept() => {
                session::spawn(&mut sessions, connection?.0, &service, &tls, &capacity, session::Protocol::Pop);
            }
            connection = pop_starttls.accept() => {
                session::spawn(&mut sessions, connection?.0, &service, &tls, &capacity, session::Protocol::PopStartTls);
            }
            Some(_) = sessions.join_next(), if !sessions.is_empty() => {}
        }
    }
    let _ = stop.send(());
    let _ = outbound_stop.send(true);
    // Accepted envelopes and content have committed. Incomplete SMTP DATA has no
    // success response and may be retried by its sender after the drain deadline.
    let drain = tokio::time::Instant::now() + Duration::from_secs(30);
    if tokio::time::timeout_at(drain, async {
        while sessions.join_next().await.is_some() {}
    })
    .await
    .is_err()
    {
        sessions.abort_all();
        while sessions.join_next().await.is_some() {}
    }
    if let Ok(result) = tokio::time::timeout_at(drain, &mut http_task).await {
        result??;
    } else {
        http_task.abort();
        let _ = http_task.await;
    }
    let _ = delivery_stop.send(());
    let _ = sweep_stop.send(());
    if tokio::time::timeout_at(drain, async {
        while outbound_workers.join_next().await.is_some() {}
    })
    .await
    .is_err()
    {
        outbound_workers.abort_all();
        while outbound_workers.join_next().await.is_some() {}
    }
    if tokio::time::timeout_at(drain, &mut delivery_task)
        .await
        .is_err()
    {
        delivery_task.abort();
        let _ = delivery_task.await;
    }
    Ok(())
}

fn setting(name: &str, default: &str) -> Result<String, std::env::VarError> {
    match std::env::var(name) {
        Ok(value) => Ok(value),
        Err(std::env::VarError::NotPresent) => Ok(default.into()),
        Err(error) => Err(error),
    }
}

async fn bind(name: &str, default: &str) -> Result<TcpListener, Box<dyn std::error::Error>> {
    let address: std::net::SocketAddr = setting(name, default)?.parse()?;
    Ok(TcpListener::bind(address).await?)
}

async fn shutdown() -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c().await
}

#[tokio::main]
async fn main() {
    // Refusals reach the operator as their Display text, not a Debug dump.
    if let Err(error) = run().await {
        eprintln!("mail-app: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("provision") if args.len()==7 => {
            let token=std::env::var("MAIL_TOKEN").map_err(|_| "MAIL_TOKEN is required (at least 32 bytes)")?;
            SqliteStore::open(&args[2])?.provision(Account::new(&args[4],&args[3],&args[5],&args[6])?,&token)?;
            Ok(())
        }
        Some("failed") if args.len()==4 => {
            for failure in SqliteStore::open(&args[2])?.failed_deliveries(&args[3], 1000)? {
                println!("{} {} {}", failure.message, failure.failed_at, failure.reason);
            }
            Ok(())
        }
        Some("retry") if args.len()==5 => {
            SqliteStore::open(&args[2])?.retry_failed_delivery(&args[3], &args[4])?;
            Ok(())
        }
        Some("serve") if args.len()==5 => serve(&args[2],&args[3],&args[4]).await,
        Some("convert") if args.len()==5 => convert::run(&args[2],&args[3],&args[4]),
        _ => Err("usage: mail-app failed DATABASE ACCOUNT; mail-app retry DATABASE ACCOUNT MESSAGE; mail-app provision DATABASE TENANT ACCOUNT OWNER ADDRESS (MAIL_TOKEN env); mail-app convert DATABASE --backup-verified BACKUP_PATH | --backup-into BACKUP_PATH (stop `serve` first; a database below this binary's schema version is refused by every other command until converted); mail-app serve DATABASE CERTIFICATE_PEM PRIVATE_KEY_PEM (optional MAIL_SMTP_LISTEN, MAIL_SUBMISSION_LISTEN, MAIL_SUBMISSION_STARTTLS_LISTEN, MAIL_IMAP_LISTEN, MAIL_IMAP_STARTTLS_LISTEN, MAIL_HTTP_LISTEN, MAIL_POP_LISTEN, MAIL_POP_STARTTLS_LISTEN, MAIL_PUBLIC_URL; outbound MAIL_RELAY_HOST, MAIL_RELAY_PORT, MAIL_RELAY_HELO, MAIL_RELAY_STARTTLS, MAIL_RELAY_CA, MAIL_RELAY_USERNAME, MAIL_RELAY_PASSWORD; direct delivery MAIL_MX_DNS_SERVERS, MAIL_MX_HELO, MAIL_MX_PORT, MAIL_MX_CA, MAIL_MX_REQUIRE_TLS env)".into()),
    }
}

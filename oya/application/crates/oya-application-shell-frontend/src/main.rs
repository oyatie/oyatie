#[cfg(target_arch = "wasm32")]
fn main() {
    oya_application_shell_frontend::mount_app();
}

#[cfg(all(not(target_arch = "wasm32"), feature = "ssr"))]
fn main() -> std::io::Result<()> {
    dev_server::run()
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "ssr")))]
fn main() {
    println!(
        "Oyatie console shell. Run `cargo leptos watch` from the crate directory for the local dev server, or build the WASM target from the workspace root."
    );
}

#[cfg(all(not(target_arch = "wasm32"), feature = "ssr"))]
mod dev_server {
    use std::{
        env, fs,
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        path::{Component, Path, PathBuf},
        thread,
        time::Duration,
    };

    use oya_application_shell_frontend::{render_envelope_json, static_dashboard_html};

    pub fn run() -> std::io::Result<()> {
        let addr = env::var("LEPTOS_SITE_ADDR")
            .or_else(|_| env::var("SITE_ADDR"))
            .unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        let listener = TcpListener::bind(&addr)?;
        println!("Oyatie console shell local dev server listening on http://{addr}");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        if let Err(error) = handle(stream) {
                            eprintln!("console shell dev server request failed: {error}");
                        }
                    });
                }
                Err(error) => eprintln!("console shell dev server connection failed: {error}"),
            }
        }

        Ok(())
    }

    fn handle(mut stream: TcpStream) -> std::io::Result<()> {
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        let mut buffer = [0_u8; 2048];
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/");

        match path {
            "/" | "/index.html" => write_response(
                &mut stream,
                "200 OK",
                "text/html; charset=utf-8",
                root_html().as_bytes(),
            ),
            "/style/tokens.css" => write_file(
                &mut stream,
                &crate_path("style/tokens.css"),
                "text/css; charset=utf-8",
            ),
            "/style/app.css" => write_file(
                &mut stream,
                &crate_path("style/app.css"),
                "text/css; charset=utf-8",
            ),
            path if path.starts_with("/api/render-envelope/") => {
                let context_id = path.trim_start_matches("/api/render-envelope/");
                if let Some(json) = render_envelope_json(context_id) {
                    write_response(
                        &mut stream,
                        "200 OK",
                        "application/json; charset=utf-8",
                        json.as_bytes(),
                    )
                } else {
                    write_response(
                        &mut stream,
                        "404 Not Found",
                        "application/json; charset=utf-8",
                        br#"{"error":"unknown render envelope"}"#,
                    )
                }
            }
            path if path.starts_with("/pkg/") => {
                let site_pkg = Path::new("target/site");
                if let Some(safe_path) = safe_join(site_pkg, path.trim_start_matches('/')) {
                    let content_type = content_type(&safe_path);
                    write_file(&mut stream, &safe_path, content_type)
                } else {
                    write_response(
                        &mut stream,
                        "400 Bad Request",
                        "text/plain; charset=utf-8",
                        b"bad request",
                    )
                }
            }
            _ => write_response(
                &mut stream,
                "404 Not Found",
                "text/plain; charset=utf-8",
                b"not found",
            ),
        }
    }

    fn root_html() -> String {
        let body = static_dashboard_html();
        let wasm_package_available =
            Path::new("target/site/pkg/oya_application_shell_frontend.js").exists();
        format!(
            r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Oyatie Cloud/Tenant Control Center</title>
    <link rel="stylesheet" href="/style/tokens.css">
    <link rel="stylesheet" href="/style/app.css">
  </head>
  <body>
    <noscript>The Oyatie console needs WebAssembly enabled for island hydration; the server-rendered shell remains visible.</noscript>
    {body}
    <script type="module">
      const wasmPackageAvailable = {wasm_package_available};
      async function mountDashboardIsland() {{
        if (!wasmPackageAvailable) {{
          console.info('Oyatie console: WASM island package missing from /pkg; serving the server-rendered shell only.');
          return;
        }}
        const wasm = await import('/pkg/oya_application_shell_frontend.js');
        await wasm.default();
        wasm.mount_dashboard_islands();
      }}
      mountDashboardIsland();
    </script>
  </body>
</html>
"#
        )
    }

    fn crate_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
    }

    fn write_file(stream: &mut TcpStream, path: &Path, content_type: &str) -> std::io::Result<()> {
        match fs::read(path) {
            Ok(bytes) => write_response(stream, "200 OK", content_type, &bytes),
            Err(_) => write_response(
                stream,
                "404 Not Found",
                "text/plain; charset=utf-8",
                b"not found",
            ),
        }
    }

    fn write_response(
        stream: &mut TcpStream,
        status: &str,
        content_type: &str,
        body: &[u8],
    ) -> std::io::Result<()> {
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )?;
        stream.write_all(body)
    }

    fn safe_join(base: &Path, relative: &str) -> Option<PathBuf> {
        let mut path = PathBuf::from(base);
        for component in Path::new(relative).components() {
            match component {
                Component::Normal(part) => path.push(part),
                _ => return None,
            }
        }
        Some(path)
    }

    fn content_type(path: &Path) -> &'static str {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("js") => "text/javascript; charset=utf-8",
            Some("wasm") => "application/wasm",
            Some("css") => "text/css; charset=utf-8",
            _ => "application/octet-stream",
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use oya_application_shell_frontend::render_envelope::{
            OperatorContext, TenantRenderEnvelope,
        };

        #[test]
        fn render_envelope_endpoint_serializes_one_permitted_context_deny_by_default() {
            let text =
                render_envelope_json("tenant-admin").expect("valid render-envelope response");
            let envelope: TenantRenderEnvelope =
                serde_json::from_str(&text).expect("typed render envelope json");

            assert_eq!(envelope.context, OperatorContext::TenantAdmin);
            assert!(text.contains("\"tenant-admin\""));
            assert!(
                !text.contains("Clinical Home"),
                "unaccredited context must not receive healthcare capabilities"
            );
        }

        #[test]
        fn render_envelope_endpoint_rejects_unknown_context() {
            assert!(render_envelope_json("unknown").is_none());
            assert!(render_envelope_json("/style/app.css").is_none());
        }
    }
}

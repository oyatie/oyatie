//! Contract tests for the scripted HTTP test server itself.
//!
//! A test helper with no tests is a liability: every port that lands on top of this
//! crate inherits its framing, its recording and its failure modes. These exercise the
//! helper against a REAL HTTP client (reqwest, the client every consumer uses), not
//! against a hand-written byte stream, because the properties that matter — keep-alive
//! suppression, `Content-Length` framing, chunked delivery — are exactly the ones a
//! hand-written client would not check.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use futures::StreamExt as _;
use scripted_http_server::{Chunk, ScriptedResponse, ScriptedServer};
use serde_json::json;

#[test]
fn records_method_path_query_headers_and_body() {
    let server = ScriptedServer::start(vec![ScriptedResponse::ok().json(&json!({"ok": true}))]);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(server.url("/repos/o/r/statuses/abc?per_page=100&page=1"))
        .header("Authorization", "Bearer test-token")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&json!({"state": "success", "context": "pr-review"}))
        .send()
        .expect("send");
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response.json::<serde_json::Value>().expect("json"),
        json!({"ok": true})
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 1, "requests: {:?}", server.request_lines());
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path(), "/repos/o/r/statuses/abc");
    assert_eq!(request.query_param("per_page").as_deref(), Some("100"));
    assert_eq!(request.query_param("page").as_deref(), Some("1"));
    assert_eq!(request.query_param("absent"), None);
    // Header recording is the capability the original `scripted_http_server` lacked:
    // it parsed headers only to find content-length and threw them away.
    assert_eq!(request.header("authorization"), Some("Bearer test-token"));
    assert_eq!(request.header("AUTHORIZATION"), Some("Bearer test-token"));
    assert_eq!(request.header("x-github-api-version"), Some("2022-11-28"));
    assert!(request.has_header("content-length"));
    assert_eq!(
        request.json(),
        json!({"state": "success", "context": "pr-review"})
    );
}

#[test]
fn frames_a_body_larger_than_any_fixed_read_buffer() {
    // `recording_multi_request_server` read the request with ONE 16 KiB read and so
    // truncated anything larger. 512 KiB is well past that.
    let payload = "x".repeat(512 * 1024);
    let server = ScriptedServer::start(vec![ScriptedResponse::ok()]);

    reqwest::blocking::Client::new()
        .post(server.url("/big"))
        .body(payload.clone())
        .send()
        .expect("send");

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].body.len(),
        payload.len(),
        "body was truncated: got {} of {} bytes",
        requests[0].body.len(),
        payload.len()
    );
    assert_eq!(requests[0].body_string(), payload);
}

#[test]
fn frames_a_chunked_request_body() {
    let server = ScriptedServer::start(vec![ScriptedResponse::ok()]);

    // No Content-Length: reqwest sends a streaming body as Transfer-Encoding: chunked.
    // The original helper read such a body as zero bytes.
    let stream = futures::stream::iter(vec![
        Ok::<_, std::io::Error>("first-"),
        Ok("second-"),
        Ok("third"),
    ]);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        reqwest::Client::new()
            .post(server.url("/streamed"))
            .body(reqwest::Body::wrap_stream(stream))
            .send()
            .await
            .expect("send");
    });

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].header("transfer-encoding"), Some("chunked"));
    assert_eq!(requests[0].body_string(), "first-second-third");
}

#[test]
fn serves_the_script_positionally_and_records_every_request() {
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok().json(&json!({"n": 1})),
        ScriptedResponse::status(404).json(&json!({"n": 2})),
        ScriptedResponse::status(503).text("n=3"),
    ]);

    let client = reqwest::blocking::Client::new();
    let statuses: Vec<u16> = ["/one", "/two", "/three"]
        .into_iter()
        .map(|path| {
            client
                .get(server.url(path))
                .send()
                .expect("send")
                .status()
                .as_u16()
        })
        .collect();

    assert_eq!(statuses, vec![200, 404, 503]);
    assert_eq!(
        server.request_lines(),
        vec!["GET /one", "GET /two", "GET /three"]
    );
}

#[test]
fn a_request_past_the_end_of_the_script_is_recorded_and_answered_500() {
    // The original helper scripted exactly N connections then exited its thread, so an
    // unexpected (N+1)th call was refused and recorded NOWHERE. Here it is visible.
    let server = ScriptedServer::start(vec![ScriptedResponse::ok()]);

    let client = reqwest::blocking::Client::new();
    assert_eq!(
        client
            .get(server.url("/scripted"))
            .send()
            .expect("send")
            .status()
            .as_u16(),
        200
    );
    let overrun = client.get(server.url("/unexpected")).send().expect("send");
    assert_eq!(overrun.status().as_u16(), 500);
    assert!(
        overrun.text().expect("text").contains("script exhausted"),
        "over-run response must name itself"
    );

    assert_eq!(
        server.request_lines(),
        vec!["GET /scripted", "GET /unexpected"],
        "the unexpected call must appear in the trace"
    );
}

#[test]
fn fewer_requests_than_scripted_does_not_hang() {
    // The original helper's `join()` blocked forever in this case, turning a failed
    // assertion into a hung test. Nothing here joins the server.
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok(),
        ScriptedResponse::ok(),
        ScriptedResponse::ok(),
    ]);
    reqwest::blocking::Client::new()
        .get(server.url("/only-one"))
        .send()
        .expect("send");
    assert_eq!(server.request_count(), 1);
    drop(server);
}

#[test]
fn content_routing_selects_a_response_by_request_content() {
    let server = ScriptedServer::start_with(|request| match request.path() {
        "/token" if request.body_string().contains("refresh-1") => {
            ScriptedResponse::ok().json(&json!({"access_token": "access-1"}))
        }
        "/token" => ScriptedResponse::status(401).json(&json!({"error": "stale refresh token"})),
        _ => ScriptedResponse::status(404),
    });

    let client = reqwest::blocking::Client::new();
    let good = client
        .post(server.url("/token"))
        .body("grant_type=refresh_token&refresh_token=refresh-1")
        .send()
        .expect("send");
    assert_eq!(good.status().as_u16(), 200);

    let stale = client
        .post(server.url("/token"))
        .body("grant_type=refresh_token&refresh_token=refresh-9")
        .send()
        .expect("send");
    assert_eq!(stale.status().as_u16(), 401);

    assert_eq!(
        client
            .get(server.url("/nope"))
            .send()
            .expect("send")
            .status(),
        404
    );
    assert_eq!(server.request_count(), 3);
}

#[test]
fn serves_concurrent_connections_simultaneously() {
    // A single sequential accept loop cannot do this: with a 300ms delay on each
    // response, eight sequential connections take >= 2.4s. One thread per connection
    // keeps the total near a single delay.
    let concurrent = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let observed_concurrent = Arc::clone(&concurrent);
    let observed_peak = Arc::clone(&peak);

    let server = ScriptedServer::start_with(move |_| {
        let now = observed_concurrent.fetch_add(1, Ordering::SeqCst) + 1;
        observed_peak.fetch_max(now, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(300));
        observed_concurrent.fetch_sub(1, Ordering::SeqCst);
        ScriptedResponse::ok().text("done")
    });

    let started = Instant::now();
    let base = server.base_url().to_owned();
    let workers: Vec<_> = (0..8)
        .map(|index| {
            let url = format!("{base}/caller-{index}");
            std::thread::spawn(move || {
                reqwest::blocking::Client::new()
                    .get(url)
                    .send()
                    .expect("send")
                    .status()
                    .as_u16()
            })
        })
        .collect();
    let statuses: Vec<u16> = workers
        .into_iter()
        .map(|worker| worker.join().expect("join"))
        .collect();
    let elapsed = started.elapsed();

    assert_eq!(statuses, vec![200; 8]);
    assert_eq!(server.request_count(), 8);
    assert!(
        peak.load(Ordering::SeqCst) > 1,
        "connections were serialised: peak in-flight was {}",
        peak.load(Ordering::SeqCst)
    );
    assert!(
        elapsed < Duration::from_millis(2_400),
        "8 x 300ms served in {elapsed:?} — that is sequential, not concurrent"
    );
}

#[tokio::test]
async fn chunked_responses_arrive_incrementally() {
    // The property an SSE test depends on: frame N is readable before frame N+1 is
    // written. If the server buffered the whole body, all frames would land at once.
    let server = ScriptedServer::start(vec![ScriptedResponse::ok().sse(vec![
        Chunk::new("event: start\ndata: {\"n\":0}\n\n"),
        Chunk::after(
            Duration::from_millis(150),
            "event: delta\ndata: {\"n\":1}\n\n",
        ),
        Chunk::after(Duration::from_millis(150), "event: done\ndata: [DONE]\n\n"),
    ])]);

    let response = reqwest::Client::new()
        .get(server.url("/v1/messages"))
        .send()
        .await
        .expect("send");
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let started = Instant::now();
    let mut arrivals = Vec::new();
    let mut body = String::new();
    let mut stream = response.bytes_stream();
    while let Some(frame) = stream.next().await {
        let frame = frame.expect("frame");
        arrivals.push(started.elapsed());
        body.push_str(&String::from_utf8_lossy(&frame));
    }

    assert!(
        body.contains("event: start") && body.contains("event: delta") && body.contains("[DONE]"),
        "body: {body}"
    );
    assert!(
        arrivals.len() >= 3,
        "expected at least 3 separately-delivered frames, got {}: {arrivals:?}",
        arrivals.len()
    );
    assert!(
        arrivals[0] < Duration::from_millis(140),
        "first frame waited for the whole body: {arrivals:?}"
    );
    assert!(
        arrivals[arrivals.len() - 1] >= Duration::from_millis(280),
        "frames were not spaced by the scripted delays: {arrivals:?}"
    );
}

#[test]
fn raw_emits_bytes_verbatim_and_hangup_closes_without_responding() {
    let server = ScriptedServer::start(vec![
        ScriptedResponse::raw(
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nhi",
        ),
        ScriptedResponse::hangup(),
    ]);

    let client = reqwest::blocking::Client::new();
    let response = client.get(server.url("/raw")).send().expect("send");
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.text().expect("text"), "hi");

    let hung_up = client.get(server.url("/hangup")).send();
    assert!(hung_up.is_err(), "hangup must not produce a response");
    assert_eq!(
        server.request_lines(),
        vec!["GET /raw", "GET /hangup"],
        "a hung-up request is still recorded"
    );
}

#[test]
fn repeated_response_headers_are_all_sent() {
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok()
            .header("Set-Cookie", "a=1")
            .header("Set-Cookie", "b=2")
            .text("body"),
    ]);

    let response = reqwest::blocking::Client::new()
        .get(server.url("/cookies"))
        .send()
        .expect("send");
    let cookies: Vec<&str> = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect();
    assert_eq!(cookies, vec!["a=1", "b=2"]);
}

#[test]
fn repeated_request_headers_are_all_recorded() {
    let server = ScriptedServer::start(vec![ScriptedResponse::ok()]);
    reqwest::blocking::Client::new()
        .get(server.url("/multi"))
        .header("X-Trace", "one")
        .header("X-Trace", "two")
        .send()
        .expect("send");

    let requests = server.requests();
    assert_eq!(requests[0].header_values("x-trace"), vec!["one", "two"]);
}

#[test]
fn delay_holds_the_response_before_the_first_byte() {
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok()
            .delay(Duration::from_millis(250))
            .text("late"),
    ]);
    let started = Instant::now();
    let response = reqwest::blocking::Client::new()
        .get(server.url("/slow"))
        .send()
        .expect("send");
    assert_eq!(response.text().expect("text"), "late");
    assert!(
        started.elapsed() >= Duration::from_millis(240),
        "delay was not honoured: {:?}",
        started.elapsed()
    );
}

#[test]
fn dropping_the_server_closes_the_port() {
    let server = ScriptedServer::start_always(ScriptedResponse::ok());
    let url = server.url("/alive");
    assert!(reqwest::blocking::Client::new().get(&url).send().is_ok());
    drop(server);

    // Give the accept loop a moment to observe the shutdown flag and drop the listener.
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if reqwest::blocking::Client::new()
            .get(&url)
            .timeout(Duration::from_millis(200))
            .send()
            .is_err()
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("port stayed open after the server was dropped");
}

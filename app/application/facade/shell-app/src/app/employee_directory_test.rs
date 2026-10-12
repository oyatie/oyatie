use super::*;

#[test]
fn sample_directory_has_truthful_totals_and_matching_native_inspections() {
    let html = sample_employee_directory().into_view().to_html();
    assert_eq!(html.matches("data-employee-row=\"true\"").count(), 8);
    assert_eq!(
        EMPLOYEES
            .iter()
            .filter(|e| e.status == SampleStatus::Active)
            .count(),
        6
    );
    assert_eq!(
        EMPLOYEES
            .iter()
            .filter(|e| e.status == SampleStatus::Probation)
            .count(),
        1
    );
    assert_eq!(
        EMPLOYEES
            .iter()
            .filter(|e| e.status == SampleStatus::OnLeave)
            .count(),
        1
    );
    assert!(html.contains("8 matching sample employees · 8 of 8 shown"));
    assert!(html.contains("Employee readiness unavailable"));
    assert!(html.contains("no employee or invitation is created here"));
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(!html.contains("118"));
    assert!(!html.contains("109"));
    for employee in &EMPLOYEES {
        let marker = format!("<details data-employee-inspection=\"{}\">", employee.id);
        let detail = html
            .split_once(&marker)
            .expect("native sample inspection")
            .1
            .split_once("</details>")
            .expect("complete details")
            .0;
        for value in [
            employee.name,
            employee.english_name,
            employee.id,
            employee.team_label,
            employee.status.label(),
        ] {
            assert!(detail.contains(value), "matching sample field {value}");
        }
        assert!(detail.contains("Unavailable · no checklist or evidence evaluated"));
    }
    let compatibility = crate::static_dashboard_html();
    assert!(compatibility.contains(&html));
    let native_workforce = workforce_anchor_board().into_view().to_html();
    for preview in [&native_workforce, &compatibility] {
        assert!(preview.contains(
            "<span data-workforce-anchor-status=\"true\">Sample workforce preview · employee readiness unavailable.</span>"
        ));
        assert!(!preview.contains("Employees ready"));
        assert!(!preview.contains("Sample tenant admission:"));
    }
    assert!(compatibility.contains("3 of 9 sample steps complete · 33%"));
    assert!(compatibility.contains(
        "No tenant admission, go-live, employee readiness or legal/bank verification occurs."
    ));
}

#[tokio::test]
async fn served_hr_preview_contains_shared_samples_and_directory_controller() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(crate::server::serve_until_shutdown(listener, async {
            let _ = stopped.await;
        }));
        let mut stream = tokio::net::TcpStream::connect(address)
            .await
            .expect("native SSR connection");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .expect("HTTP request");
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .await
            .expect("served SSR response");
        let _ = stop.send(());
        server.await.expect("server task").expect("server shutdown");
        let html = String::from_utf8(bytes).expect("SSR UTF-8");
        assert!(html.starts_with("HTTP/1.1 200"));
        assert!(html.contains("data-employee-source=\"eight-row-sample-v1\""));
        assert_eq!(html.matches("data-employee-row=\"true\"").count(), 8);
        assert!(html.contains("8 matching sample employees · 8 of 8 shown"));
        assert!(html.contains("function restoreDirectoryFragment()"));
        assert!(html.contains("No live HR source or verified employee access is connected"));
        assert!(html.contains(
            "<span data-workforce-anchor-status=\"true\">Sample workforce preview · employee readiness unavailable.</span>"
        ));
        assert!(!html.contains("Employees ready"));
        assert!(!html.contains("Sample tenant admission:"));
        assert!(html.contains("3 of 9 sample steps complete · 33%"));
        assert!(html.contains(
            "No tenant admission, go-live, employee readiness or legal/bank verification occurs."
        ));
        assert!(html.contains("data-employee-close=\"true\""));
    })
    .await
    .expect("native SSR proof keeps its two-second deadline");
}

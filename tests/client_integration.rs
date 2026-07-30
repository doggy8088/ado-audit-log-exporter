use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use ado_audit_log_exporter::{AuditClient, AuditQuery, Authentication, RetryPolicy};
use chrono::{TimeZone, Utc};

#[tokio::test]
async fn follows_continuation_token_without_exposing_authentication() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let address = listener.local_addr().expect("mock server address");
    let server = thread::spawn(move || serve_two_pages(listener));

    let authentication =
        Authentication::personal_access_token("test-pat").expect("valid authentication");
    let client = AuditClient::from_endpoint(format!("http://{address}/audit"), authentication)
        .expect("valid client");
    let start = Utc
        .with_ymd_and_hms(2026, 7, 1, 0, 0, 0)
        .single()
        .expect("valid time");
    let end = Utc
        .with_ymd_and_hms(2026, 7, 2, 0, 0, 0)
        .single()
        .expect("valid time");
    let mut pager = client.pager(AuditQuery::new(start, end).expect("valid query"));

    let first = pager
        .next_page()
        .await
        .expect("first page")
        .expect("some page");
    let second = pager
        .next_page()
        .await
        .expect("second page")
        .expect("some page");

    assert_eq!(first.entries[0].id.as_deref(), Some("event-1"));
    assert_eq!(second.entries[0].id.as_deref(), Some("event-2"));
    assert!(pager.next_page().await.expect("finished pager").is_none());
    server.join().expect("mock server");
}

#[tokio::test]
async fn retries_when_a_success_response_body_is_truncated() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let address = listener.local_addr().expect("mock server address");
    let server = thread::spawn(move || serve_truncated_then_complete(listener));

    let authentication =
        Authentication::personal_access_token("test-pat").expect("valid authentication");
    let client = AuditClient::from_endpoint(format!("http://{address}/audit"), authentication)
        .expect("valid client")
        .with_retry_policy(RetryPolicy {
            max_retries: 1,
            max_delay: Duration::ZERO,
        });
    let start = Utc
        .with_ymd_and_hms(2026, 7, 1, 0, 0, 0)
        .single()
        .expect("valid time");
    let end = Utc
        .with_ymd_and_hms(2026, 7, 2, 0, 0, 0)
        .single()
        .expect("valid time");
    let mut pager = client.pager(AuditQuery::new(start, end).expect("valid query"));

    let page = pager
        .next_page()
        .await
        .expect("body transfer should be retried")
        .expect("some page");

    assert_eq!(page.entries[0].id.as_deref(), Some("event-after-retry"));
    server.join().expect("mock server");
}

fn serve_two_pages(listener: TcpListener) {
    for request_number in 0..2 {
        let (mut stream, _) = listener.accept().expect("accept request");
        let request = read_headers(&mut stream);
        assert!(request.starts_with("GET /audit?"));
        assert!(
            request
                .to_ascii_lowercase()
                .contains("\r\nauthorization: basic onrlc3qtcgf0\r\n")
        );

        let body = if request_number == 0 {
            assert!(!request.contains("continuationToken"));
            r#"{"decoratedAuditLogEntries":[{"id":"event-1"}],"continuationToken":"next-page","hasMore":true}"#
        } else {
            assert!(request.contains("continuationToken=next-page"));
            r#"{"decoratedAuditLogEntries":[{"id":"event-2"}],"hasMore":false}"#
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    }
}

fn serve_truncated_then_complete(listener: TcpListener) {
    for request_number in 0..2 {
        let (mut stream, _) = listener.accept().expect("accept request");
        let request = read_headers(&mut stream);
        assert!(request.starts_with("GET /audit?"));

        if request_number == 0 {
            let partial_body = r#"{"decoratedAuditLogEntries":["#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 128\r\nConnection: close\r\n\r\n{partial_body}"
            );
            stream
                .write_all(response.as_bytes())
                .expect("write truncated response");
        } else {
            let body =
                r#"{"decoratedAuditLogEntries":[{"id":"event-after-retry"}],"hasMore":false}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write complete response");
        }
    }
}

fn read_headers(stream: &mut impl Read) -> String {
    let mut bytes = Vec::new();
    let mut byte = [0_u8; 1];
    while !bytes.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut byte).expect("read request");
        assert_ne!(count, 0, "request ended before headers");
        bytes.push(byte[0]);
        assert!(bytes.len() <= 64 * 1024, "request headers too large");
    }
    String::from_utf8(bytes).expect("request headers are UTF-8")
}

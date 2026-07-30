use ado_audit_log_exporter::{AuditClient, AuditQuery, Authentication};
use chrono::{TimeZone, Utc};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path, query_param},
};

#[tokio::test]
async fn follows_continuation_token_without_exposing_authentication() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audit"))
        .and(header("authorization", "Basic OnRlc3QtcGF0"))
        .and(query_param("continuationToken", "next-page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "decoratedAuditLogEntries": [{"id": "event-2"}],
            "hasMore": false
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/audit"))
        .and(header("authorization", "Basic OnRlc3QtcGF0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "decoratedAuditLogEntries": [{"id": "event-1"}],
            "continuationToken": "next-page",
            "hasMore": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    let authentication =
        Authentication::personal_access_token("test-pat").expect("valid authentication");
    let client = AuditClient::from_endpoint(format!("{}/audit", server.uri()), authentication)
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
}

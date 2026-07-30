use std::{collections::HashSet, fmt, time::Duration};

use reqwest::{
    Client, StatusCode, Url,
    header::{ACCEPT, AUTHORIZATION, HeaderMap, USER_AGENT},
};
use serde_json::{Map, Value};
use tokio::time::sleep;

use crate::{AuditError, AuditLogEntry, AuditPage, AuditQuery, Authentication};

/// 暫時性錯誤的重試設定。
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    /// 首次要求失敗後最多重試次數。
    pub max_retries: u32,
    /// 指數退避上限。
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 4,
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Azure DevOps Audit Log REST API client。
#[derive(Clone)]
pub struct AuditClient {
    http: Client,
    endpoint: Url,
    headers: HeaderMap,
    retry_policy: RetryPolicy,
}

impl AuditClient {
    /// 以 Azure DevOps Services 組織名稱建立 client。
    pub fn new(
        organization: impl AsRef<str>,
        authentication: Authentication,
    ) -> Result<Self, AuditError> {
        let organization = organization.as_ref().trim();
        if organization.is_empty()
            || organization.contains('/')
            || organization.contains('\\')
            || organization.chars().any(char::is_whitespace)
        {
            return Err(AuditError::InvalidOrganization(organization.to_owned()));
        }
        let mut endpoint = Url::parse("https://auditservice.dev.azure.com/")
            .map_err(|error| AuditError::InvalidEndpoint(error.to_string()))?;
        endpoint
            .path_segments_mut()
            .map_err(|_| AuditError::InvalidEndpoint("endpoint 無法設定 path".to_owned()))?
            .extend([organization, "_apis", "audit", "auditlog"]);
        Self::from_url(endpoint, authentication)
    }

    /// 以完整 Audit Log API endpoint 建立 client。
    ///
    /// 此方法可供測試替身或相容服務使用；一般 Azure DevOps Services 使用者應使用
    /// [`Self::new`]。
    pub fn from_endpoint(
        endpoint: impl AsRef<str>,
        authentication: Authentication,
    ) -> Result<Self, AuditError> {
        let endpoint = Url::parse(endpoint.as_ref())
            .map_err(|error| AuditError::InvalidEndpoint(error.to_string()))?;
        Self::from_url(endpoint, authentication)
    }

    fn from_url(endpoint: Url, authentication: Authentication) -> Result<Self, AuditError> {
        if !matches!(endpoint.scheme(), "http" | "https") {
            return Err(AuditError::InvalidEndpoint(
                "endpoint 必須使用 http 或 https".to_owned(),
            ));
        }
        if !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err(AuditError::InvalidEndpoint(
                "endpoint 不可包含 credentials、query 或 fragment".to_owned(),
            ));
        }

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authentication.header_value()?);
        headers.insert(
            ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            USER_AGENT,
            reqwest::header::HeaderValue::from_static(concat!(
                "ado-audit-log-exporter/",
                env!("CARGO_PKG_VERSION")
            )),
        );

        let http = Client::builder()
            .default_headers(headers.clone())
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(AuditError::ClientBuild)?;

        Ok(Self {
            http,
            endpoint,
            headers,
            retry_policy: RetryPolicy::default(),
        })
    }

    /// 設定每次 HTTP 要求的逾時時間。
    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, AuditError> {
        if timeout.is_zero() {
            return Err(AuditError::InvalidQuery("timeout 必須大於零".to_owned()));
        }
        self.http = Client::builder()
            .default_headers(self.headers.clone())
            .timeout(timeout)
            .build()
            .map_err(AuditError::ClientBuild)?;
        Ok(self)
    }

    /// 設定重試策略。
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    /// 建立獨立追蹤分頁狀態的 pager。
    pub fn pager(&self, query: AuditQuery) -> AuditLogPager<'_> {
        AuditLogPager {
            client: self,
            query,
            continuation_token: None,
            seen_tokens: HashSet::new(),
            finished: false,
        }
    }

    async fn fetch_page(
        &self,
        query: &AuditQuery,
        continuation_token: Option<&str>,
    ) -> Result<AuditPage, AuditError> {
        for attempt in 0..=self.retry_policy.max_retries {
            let result = self
                .http
                .get(self.endpoint.clone())
                .query(&query.parameters(continuation_token))
                .send()
                .await;

            match result {
                Ok(response) if response.status().is_success() => {
                    let bytes = match response.bytes().await {
                        Ok(bytes) => bytes,
                        Err(_) if attempt < self.retry_policy.max_retries => {
                            sleep(self.exponential_delay(attempt)).await;
                            continue;
                        }
                        Err(error) => return Err(AuditError::Transport(error)),
                    };
                    let payload =
                        serde_json::from_slice::<Value>(&bytes).map_err(AuditError::InvalidJson)?;
                    return parse_page(payload);
                }
                Ok(response) => {
                    let status = response.status();
                    let retry_after = parse_retry_after(response.headers().get("retry-after"));
                    if is_retryable_status(status) && attempt < self.retry_policy.max_retries {
                        sleep(retry_after.unwrap_or_else(|| self.exponential_delay(attempt))).await;
                        continue;
                    }
                    let mut message = response_message(response).await;
                    if status == StatusCode::UNAUTHORIZED {
                        message.push_str("；請確認 token 有效，且 PAT 包含 vso.auditlog scope");
                    } else if status == StatusCode::FORBIDDEN {
                        message
                            .push_str("；token 擁有者必須具備 Azure DevOps 的 View audit log 權限");
                    }
                    return Err(AuditError::HttpStatus { status, message });
                }
                Err(error) => {
                    if (error.is_timeout() || error.is_connect())
                        && attempt < self.retry_policy.max_retries
                    {
                        sleep(self.exponential_delay(attempt)).await;
                        continue;
                    }
                    return Err(AuditError::Transport(error));
                }
            }
        }

        unreachable!("bounded retry loop always returns")
    }

    fn exponential_delay(&self, attempt: u32) -> Duration {
        let seconds = 1_u64.checked_shl(attempt.min(30)).unwrap_or(u64::MAX);
        Duration::from_secs(seconds).min(self.retry_policy.max_delay)
    }
}

impl fmt::Debug for AuditClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuditClient")
            .field("endpoint", &self.endpoint)
            .field("retry_policy", &self.retry_policy)
            .finish_non_exhaustive()
    }
}

/// 保留 continuation token 狀態的非同步分頁器。
pub struct AuditLogPager<'a> {
    client: &'a AuditClient,
    query: AuditQuery,
    continuation_token: Option<String>,
    seen_tokens: HashSet<String>,
    finished: bool,
}

impl AuditLogPager<'_> {
    /// 讀取下一頁。全部讀取完畢後固定回傳 `Ok(None)`。
    pub async fn next_page(&mut self) -> Result<Option<AuditPage>, AuditError> {
        if self.finished {
            return Ok(None);
        }

        let page = self
            .client
            .fetch_page(&self.query, self.continuation_token.as_deref())
            .await?;

        if page.has_more {
            let token = page.continuation_token.clone().ok_or_else(|| {
                AuditError::InvalidResponse(
                    "hasMore 為 true，但回應沒有 continuationToken".to_owned(),
                )
            })?;
            if token.is_empty() {
                return Err(AuditError::InvalidResponse(
                    "continuationToken 不可為空字串".to_owned(),
                ));
            }
            if !self.seen_tokens.insert(token.clone()) {
                return Err(AuditError::RepeatedContinuationToken);
            }
            self.continuation_token = Some(token);
        } else {
            self.finished = true;
        }

        Ok(Some(page))
    }
}

fn is_retryable_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn parse_retry_after(value: Option<&reqwest::header::HeaderValue>) -> Option<Duration> {
    value?
        .to_str()
        .ok()?
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

async fn response_message(response: reqwest::Response) -> String {
    let fallback = response
        .status()
        .canonical_reason()
        .unwrap_or("REST API 要求失敗")
        .to_owned();
    let Ok(bytes) = response.bytes().await else {
        return fallback;
    };
    let text = String::from_utf8_lossy(&bytes);
    let message = serde_json::from_slice::<Value>(&bytes)
        .ok()
        .and_then(|value| {
            value
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| text.trim().to_owned());
    if message.is_empty() {
        fallback
    } else {
        message.chars().take(1_000).collect()
    }
}

fn parse_page(payload: Value) -> Result<AuditPage, AuditError> {
    let root = payload
        .as_object()
        .ok_or_else(|| AuditError::InvalidResponse("最外層 JSON 必須是 object".to_owned()))?;
    let result = result_object(root)?;
    let entries_value = result
        .get("decoratedAuditLogEntries")
        .or_else(|| result.get("auditLogEntries"))
        .or_else(|| result.get("value").filter(|value| value.is_array()))
        .ok_or_else(|| {
            AuditError::InvalidResponse(
                "找不到 decoratedAuditLogEntries、auditLogEntries 或 array 型別的 value".to_owned(),
            )
        })?;
    let entries = serde_json::from_value::<Vec<AuditLogEntry>>(entries_value.clone())
        .map_err(AuditError::InvalidJson)?;
    let continuation_token = result.get("continuationToken").and_then(value_to_token);
    let has_more = result
        .get("hasMore")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| continuation_token.is_some());

    Ok(AuditPage {
        entries,
        continuation_token,
        has_more,
    })
}

fn result_object(root: &Map<String, Value>) -> Result<&Map<String, Value>, AuditError> {
    if root.contains_key("decoratedAuditLogEntries")
        || root.contains_key("auditLogEntries")
        || root.get("value").is_some_and(Value::is_array)
    {
        return Ok(root);
    }
    if let Some(wrapped) = root.get("value").and_then(Value::as_object) {
        return Ok(wrapped);
    }

    let mut keys = root.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    Err(AuditError::InvalidResponse(format!(
        "不支援的回應結構；最外層欄位：{}",
        if keys.is_empty() {
            "(無)".to_owned()
        } else {
            keys.join(", ")
        }
    )))
}

fn value_to_token(value: &Value) -> Option<String> {
    match value {
        Value::String(token) if !token.is_empty() => Some(token.clone()),
        Value::Number(token) => Some(token.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::parse_page;

    #[test]
    fn parses_direct_response_and_preserves_unknown_fields() {
        let page = parse_page(json!({
            "decoratedAuditLogEntries": [{
                "id": "event-1",
                "actionId": "Git.Push",
                "futureField": {"enabled": true}
            }],
            "continuationToken": "next",
            "hasMore": true
        }))
        .expect("valid page");

        assert_eq!(page.entries[0].id.as_deref(), Some("event-1"));
        assert_eq!(page.entries[0].extra_fields["futureField"]["enabled"], true);
        assert_eq!(page.continuation_token.as_deref(), Some("next"));
    }

    #[test]
    fn parses_value_wrapped_response() {
        let page = parse_page(json!({
            "value": {
                "decoratedAuditLogEntries": [],
                "continuationToken": null,
                "hasMore": false
            }
        }))
        .expect("valid page");

        assert!(page.entries.is_empty());
        assert!(!page.has_more);
    }

    #[test]
    fn parses_array_value_response() {
        let page = parse_page(json!({
            "value": [{"id": "event-1"}]
        }))
        .expect("valid page");

        assert_eq!(page.entries.len(), 1);
    }
}

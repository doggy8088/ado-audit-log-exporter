use chrono::{DateTime, SecondsFormat, Utc};

use crate::AuditError;

/// Azure DevOps 稽核記錄查詢條件。
#[derive(Clone, Debug)]
pub struct AuditQuery {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    batch_size: u16,
    skip_aggregation: bool,
}

impl AuditQuery {
    /// 建立指定 UTC 時間範圍的查詢。
    pub fn new(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Self, AuditError> {
        if start_time >= end_time {
            return Err(AuditError::InvalidQuery(
                "start_time 必須早於 end_time".to_owned(),
            ));
        }

        Ok(Self {
            start_time,
            end_time,
            batch_size: 200,
            skip_aggregation: true,
        })
    }

    /// 設定每頁筆數。Azure DevOps Audit Log API 接受 1 到 200。
    pub fn with_batch_size(mut self, batch_size: u16) -> Result<Self, AuditError> {
        if !(1..=200).contains(&batch_size) {
            return Err(AuditError::InvalidQuery(
                "batch_size 必須介於 1 與 200".to_owned(),
            ));
        }
        self.batch_size = batch_size;
        Ok(self)
    }

    /// 設定是否略過 access log aggregation。
    pub fn with_skip_aggregation(mut self, skip_aggregation: bool) -> Self {
        self.skip_aggregation = skip_aggregation;
        self
    }

    pub(crate) fn parameters(&self, continuation_token: Option<&str>) -> Vec<(&str, String)> {
        let mut parameters = vec![
            (
                "startTime",
                self.start_time.to_rfc3339_opts(SecondsFormat::Millis, true),
            ),
            (
                "endTime",
                self.end_time.to_rfc3339_opts(SecondsFormat::Millis, true),
            ),
            ("batchSize", self.batch_size.to_string()),
            ("skipAggregation", self.skip_aggregation.to_string()),
            ("api-version", crate::API_VERSION.to_owned()),
        ];
        if let Some(token) = continuation_token {
            parameters.push(("continuationToken", token.to_owned()));
        }
        parameters
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::AuditQuery;

    #[test]
    fn rejects_reversed_range() {
        let instant = Utc
            .with_ymd_and_hms(2026, 7, 1, 0, 0, 0)
            .single()
            .expect("valid time");

        let error = AuditQuery::new(instant, instant).expect_err("must reject empty range");

        assert!(error.to_string().contains("start_time"));
    }

    #[test]
    fn formats_api_parameters() {
        let start = Utc
            .with_ymd_and_hms(2026, 7, 1, 0, 0, 0)
            .single()
            .expect("valid time");
        let end = Utc
            .with_ymd_and_hms(2026, 7, 2, 0, 0, 0)
            .single()
            .expect("valid time");
        let query = AuditQuery::new(start, end).expect("valid query");

        let parameters = query.parameters(Some("next"));

        assert!(parameters.contains(&("startTime", "2026-07-01T00:00:00.000Z".to_owned())));
        assert!(parameters.contains(&("continuationToken", "next".to_owned())));
    }
}

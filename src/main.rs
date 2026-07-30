use std::{
    collections::BTreeMap,
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Duration as StdDuration,
};

use ado_audit_log_exporter::{AuditClient, AuditLogEntry, AuditQuery, Authentication, RetryPolicy};
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Duration, Utc};
use clap::{Parser, ValueEnum};
use serde_json::Value;
use tempfile::NamedTempFile;

const CSV_FIELDS: [&str; 25] = [
    "id",
    "correlationId",
    "activityId",
    "timestamp",
    "actionId",
    "area",
    "category",
    "categoryDisplayName",
    "details",
    "actorCUID",
    "actorClientId",
    "actorUserId",
    "actorUPN",
    "actorDisplayName",
    "actorImageUrl",
    "authenticationMechanism",
    "ipAddress",
    "userAgent",
    "scopeType",
    "scopeDisplayName",
    "scopeId",
    "projectId",
    "projectName",
    "data",
    "extraFields",
];

#[derive(Debug, Parser)]
#[command(
    name = "ado-audit-log-exporter",
    version,
    about = "透過 REST API 匯出 Azure DevOps 稽核記錄"
)]
struct Cli {
    /// Azure DevOps 組織名稱
    #[arg(long, default_value = "miniasp")]
    organization: String,

    /// 開始時間，必須是含時區的 RFC 3339；預設為目前時間前 30 天
    #[arg(long, value_parser = parse_datetime)]
    start_time: Option<DateTime<Utc>>,

    /// 結束時間，必須是含時區的 RFC 3339；預設為目前時間
    #[arg(long, value_parser = parse_datetime)]
    end_time: Option<DateTime<Utc>>,

    /// 輸出格式
    #[arg(long, value_enum, default_value_t = OutputFormat::Jsonl)]
    format: OutputFormat,

    /// 輸出檔案；未設定時依格式使用 ado-audit.json、.jsonl 或 .csv
    #[arg(long)]
    output: Option<PathBuf>,

    /// 每頁筆數，範圍為 1 到 200
    #[arg(long, default_value_t = 200, value_parser = clap::value_parser!(u16).range(1..=200))]
    batch_size: u16,

    /// 每次 HTTP 要求的逾時秒數
    #[arg(long, default_value = "30", value_parser = parse_positive_duration)]
    timeout: StdDuration,

    /// 暫時性錯誤的重試次數
    #[arg(long, default_value_t = 4)]
    retries: u32,

    /// 保留 Azure DevOps 聚合的 access log
    #[arg(long)]
    aggregate_access_log: bool,

    /// 覆寫既有輸出檔
    #[arg(long)]
    overwrite: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Json,
    Jsonl,
    Csv,
}

impl OutputFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Jsonl => "jsonl",
            Self::Csv => "csv",
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let now = Utc::now();
    let start_time = cli.start_time.unwrap_or(now - Duration::days(30));
    let end_time = cli.end_time.unwrap_or(now);
    let output = cli
        .output
        .unwrap_or_else(|| PathBuf::from(format!("ado-audit.{}", cli.format.extension())));

    let query = AuditQuery::new(start_time, end_time)?
        .with_batch_size(cli.batch_size)?
        .with_skip_aggregation(!cli.aggregate_access_log);
    let authentication = Authentication::from_env()?;
    let client = AuditClient::new(&cli.organization, authentication)?
        .with_timeout(cli.timeout)?
        .with_retry_policy(RetryPolicy {
            max_retries: cli.retries,
            ..RetryPolicy::default()
        });

    export(&client, query, cli.format, &output, cli.overwrite).await
}

async fn export(
    client: &AuditClient,
    query: AuditQuery,
    format: OutputFormat,
    output: &Path,
    overwrite: bool,
) -> Result<()> {
    if output.exists() && !overwrite {
        bail!(
            "輸出檔案已存在：{}；如要覆寫請加上 --overwrite",
            output.display()
        );
    }
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("無法建立輸出目錄：{}", parent.display()))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .with_context(|| format!("無法在 {} 建立暫存檔", parent.display()))?;

    let total = {
        let buffer = BufWriter::new(temporary.as_file_mut());
        let mut writer = EventWriter::new(format, buffer)?;
        let mut pager = client.pager(query);
        let mut total = 0_u64;

        while let Some(page) = pager.next_page().await? {
            for entry in &page.entries {
                writer.write_entry(entry)?;
                total += 1;
            }
            eprintln!("已讀取 {total} 筆稽核記錄");
        }
        writer.finish()?;
        total
    };

    if overwrite {
        temporary
            .persist(output)
            .map_err(|error| error.error)
            .with_context(|| format!("無法寫入輸出檔：{}", output.display()))?;
    } else {
        temporary
            .persist_noclobber(output)
            .map_err(|error| error.error)
            .with_context(|| format!("輸出檔已存在或無法寫入：{}", output.display()))?;
    }

    eprintln!("已匯出 {total} 筆至 {}", output.display());
    Ok(())
}

enum EventWriter<W: Write> {
    Json { writer: W, first: bool },
    Jsonl(W),
    Csv(Box<csv::Writer<W>>),
}

impl<W: Write> EventWriter<W> {
    fn new(format: OutputFormat, mut writer: W) -> Result<Self> {
        match format {
            OutputFormat::Json => {
                writer.write_all(b"[\n")?;
                Ok(Self::Json {
                    writer,
                    first: true,
                })
            }
            OutputFormat::Jsonl => Ok(Self::Jsonl(writer)),
            OutputFormat::Csv => {
                let mut csv = csv::WriterBuilder::new().from_writer(writer);
                csv.write_record(CSV_FIELDS)?;
                Ok(Self::Csv(Box::new(csv)))
            }
        }
    }

    fn write_entry(&mut self, entry: &AuditLogEntry) -> Result<()> {
        match self {
            Self::Json { writer, first } => {
                if !*first {
                    writer.write_all(b",\n")?;
                }
                serde_json::to_writer(&mut *writer, entry)?;
                *first = false;
            }
            Self::Jsonl(writer) => {
                serde_json::to_writer(&mut *writer, entry)?;
                writer.write_all(b"\n")?;
            }
            Self::Csv(writer) => writer.write_record(csv_record(entry))?,
        }
        Ok(())
    }

    fn finish(mut self) -> Result<()> {
        match &mut self {
            Self::Json { writer, .. } => {
                writer.write_all(b"\n]\n")?;
                writer.flush()?;
            }
            Self::Jsonl(writer) => writer.flush()?,
            Self::Csv(writer) => writer.flush()?,
        }
        Ok(())
    }
}

fn csv_record(entry: &AuditLogEntry) -> Vec<String> {
    let extra_fields = entry
        .extra_fields
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    [
        optional_text(&entry.id),
        optional_text(&entry.correlation_id),
        optional_text(&entry.activity_id),
        optional_text(&entry.timestamp),
        optional_text(&entry.action_id),
        optional_text(&entry.area),
        optional_text(&entry.category),
        optional_text(&entry.category_display_name),
        optional_json(&entry.details),
        optional_text(&entry.actor_cuid),
        optional_text(&entry.actor_client_id),
        optional_text(&entry.actor_user_id),
        optional_text(&entry.actor_upn),
        optional_text(&entry.actor_display_name),
        optional_text(&entry.actor_image_url),
        optional_text(&entry.authentication_mechanism),
        optional_text(&entry.ip_address),
        optional_text(&entry.user_agent),
        optional_text(&entry.scope_type),
        optional_text(&entry.scope_display_name),
        optional_text(&entry.scope_id),
        optional_text(&entry.project_id),
        optional_text(&entry.project_name),
        optional_json(&entry.data),
        serde_json::to_string(&extra_fields).unwrap_or_else(|_| "{}".to_owned()),
    ]
    .into_iter()
    .map(neutralize_csv_cell)
    .collect()
}

fn neutralize_csv_cell(value: String) -> String {
    if matches!(value.chars().next(), Some('=' | '+' | '-' | '@')) {
        format!("'{value}")
    } else {
        value
    }
}

fn optional_text(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

fn optional_json(value: &Option<Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(value)) => value.clone(),
        Some(value) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, String> {
    let parsed = DateTime::parse_from_rfc3339(value)
        .map_err(|_| format!("無效的 RFC 3339 時間：{value}"))?;
    Ok(parsed.with_timezone(&Utc))
}

fn parse_positive_duration(value: &str) -> Result<StdDuration, String> {
    let seconds = value
        .parse::<f64>()
        .map_err(|_| format!("必須是數字：{value}"))?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err("數值必須大於零".to_owned());
    }
    let duration = StdDuration::try_from_secs_f64(seconds)
        .map_err(|_| "數值超出 timeout 可表示範圍".to_owned())?;
    if duration.is_zero() {
        return Err("數值必須至少為 1 奈秒".to_owned());
    }
    Ok(duration)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, time::Duration as StdDuration};

    use ado_audit_log_exporter::AuditLogEntry;
    use serde_json::json;

    use super::{EventWriter, OutputFormat, parse_positive_duration};

    #[test]
    fn csv_output_quotes_structured_values() {
        let entry = AuditLogEntry {
            id: Some("event-1".to_owned()),
            data: Some(json!({"repository": "sample"})),
            ..AuditLogEntry::default()
        };
        let mut output = Vec::new();
        let mut writer = EventWriter::new(OutputFormat::Csv, &mut output).expect("writer");

        writer.write_entry(&entry).expect("write");
        writer.finish().expect("finish");

        let text = String::from_utf8(output).expect("UTF-8");
        assert!(text.contains("event-1"));
        assert!(text.contains("{\"\"repository\"\":\"\"sample\"\"}"));
    }

    #[test]
    fn csv_output_neutralizes_formula_prefixes() {
        let entry = AuditLogEntry {
            details: Some(json!("@SUM(1,1)")),
            actor_display_name: Some("-2+3".to_owned()),
            user_agent: Some("=WEBSERVICE(\"https://example.invalid\")".to_owned()),
            project_name: Some("+cmd".to_owned()),
            ..AuditLogEntry::default()
        };
        let mut output = Vec::new();
        let mut writer = EventWriter::new(OutputFormat::Csv, &mut output).expect("writer");

        writer.write_entry(&entry).expect("write");
        writer.finish().expect("finish");

        let mut reader = csv::Reader::from_reader(output.as_slice());
        let record = reader
            .records()
            .next()
            .expect("one record")
            .expect("valid CSV");
        assert_eq!(&record[8], "'@SUM(1,1)");
        assert_eq!(&record[13], "'-2+3");
        assert_eq!(&record[17], "'=WEBSERVICE(\"https://example.invalid\")");
        assert_eq!(&record[22], "'+cmd");
    }

    #[test]
    fn jsonl_keeps_unknown_fields() {
        let mut extra_fields = serde_json::Map::new();
        extra_fields.insert("futureField".to_owned(), json!(true));
        let entry = AuditLogEntry {
            id: Some("event-1".to_owned()),
            extra_fields,
            ..AuditLogEntry::default()
        };
        let mut output = Vec::new();
        let mut writer = EventWriter::new(OutputFormat::Jsonl, &mut output).expect("writer");

        writer.write_entry(&entry).expect("write");
        writer.finish().expect("finish");

        let value: BTreeMap<String, serde_json::Value> =
            serde_json::from_slice(&output).expect("JSON");
        assert_eq!(value["futureField"], true);
    }

    #[test]
    fn jsonl_preserves_formula_prefixed_values() {
        let entry = AuditLogEntry {
            user_agent: Some("=WEBSERVICE(\"https://example.invalid\")".to_owned()),
            ..AuditLogEntry::default()
        };
        let mut output = Vec::new();
        let mut writer = EventWriter::new(OutputFormat::Jsonl, &mut output).expect("writer");

        writer.write_entry(&entry).expect("write");
        writer.finish().expect("finish");

        let value: serde_json::Value = serde_json::from_slice(&output).expect("JSON");
        assert_eq!(
            value["userAgent"],
            "=WEBSERVICE(\"https://example.invalid\")"
        );
    }

    #[test]
    fn timeout_parser_rejects_unrepresentable_and_effectively_zero_values() {
        assert!(parse_positive_duration("1e300").is_err());
        assert!(parse_positive_duration("1e-300").is_err());
    }

    #[test]
    fn timeout_parser_accepts_positive_fractional_seconds() {
        assert_eq!(
            parse_positive_duration("0.25").expect("valid timeout"),
            StdDuration::from_millis(250)
        );
    }
}

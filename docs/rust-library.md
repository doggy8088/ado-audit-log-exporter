# Rust library 使用說明

crate 名稱是 `ado-audit-log-exporter`，Rust module 名稱是 `ado_audit_log_exporter`。

* * *

## 加入相依套件

發布至 crates.io 後：

```toml
[dependencies]
ado-audit-log-exporter = "0.1"
chrono = "0.4"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

發布前可直接引用 Git：

```toml
[dependencies]
ado-audit-log-exporter = {
  git = "https://github.com/doggy8088/ado-audit-log-exporter.git",
  rev = "要固定的提交 SHA"
}
```

正式專案應固定 tag 或 commit，不要長期追蹤可變動的 branch。

* * *

## 基本分頁

```rust
use ado_audit_log_exporter::{
    AuditClient, AuditError, AuditQuery, Authentication,
};
use chrono::{Duration, Utc};

#[tokio::main]
async fn main() -> Result<(), AuditError> {
    let authentication = Authentication::from_env()?;
    let client = AuditClient::new("miniasp", authentication)?;
    let query = AuditQuery::new(
        Utc::now() - Duration::days(7),
        Utc::now(),
    )?
    .with_batch_size(200)?
    .with_skip_aggregation(true);

    let mut pager = client.pager(query);
    while let Some(page) = pager.next_page().await? {
        for entry in page.entries {
            println!(
                "{}\t{}",
                entry.timestamp.as_deref().unwrap_or(""),
                entry.action_id.as_deref().unwrap_or("")
            );
        }
    }

    Ok(())
}
```

* * *

## 明確提供驗證

PAT：

```rust
let authentication =
    Authentication::personal_access_token(pat_from_secret_store)?;
```

Bearer token：

```rust
let authentication =
    Authentication::bearer_token(access_token_from_identity_provider)?;
```

`Authentication` 不實作 `Clone`，`Debug` 只會顯示遮罩值。呼叫端仍應避免自行記錄 token 字串。

* * *

## Timeout 與重試

```rust
use std::time::Duration;
use ado_audit_log_exporter::{AuditClient, RetryPolicy};

let client = AuditClient::new("miniasp", authentication)?
    .with_timeout(Duration::from_secs(60))?
    .with_retry_policy(RetryPolicy {
        max_retries: 6,
        max_delay: Duration::from_secs(30),
    });
```

* * *

## 公開資料模型

`AuditLogEntry` 提供已知欄位的 `Option<T>`：

```rust
if let Some(actor) = &entry.actor_upn {
    println!("actor: {actor}");
}
```

事件專屬或未來新增欄位保留在：

```rust
for (name, value) in &entry.extra_fields {
    println!("{name}: {value}");
}
```

`details` 與 `data` 使用 `Option<serde_json::Value>`，因為不同事件的資料形狀不一致。

* * *

## 錯誤處理

library 回傳 `AuditError`，主要分類包括：

- 缺少或衝突的驗證
- 無效組織、endpoint 或查詢
- HTTP client 建立失敗
- 傳輸錯誤
- 非成功 HTTP 狀態
- JSON 或回應結構錯誤
- continuation token 重複

範例：

```rust
use ado_audit_log_exporter::AuditError;

match pager.next_page().await {
    Ok(Some(page)) => println!("{} entries", page.entries.len()),
    Ok(None) => println!("done"),
    Err(AuditError::HttpStatus { status, message }) => {
        eprintln!("Azure DevOps returned {status}: {message}");
    }
    Err(error) => return Err(error),
}
```

* * *

## 自訂 endpoint

一般使用 `AuditClient::new`。測試替身或相容服務可用：

```rust
let client = AuditClient::from_endpoint(
    "http://127.0.0.1:8080/audit",
    authentication,
)?;
```

此方法接受 `http` 是為了本機測試；正式 Azure DevOps Services endpoint 應使用 `https`。

* * *

## API 穩定性

版本 `0.1.x` 仍屬初期 API。語意版本規則會套用於後續發布，但 `1.0.0` 前的公開 API 仍可能在次版本調整。新增 Azure DevOps 欄位時，既有 `extra_fields` 可先保留資料，再於後續版本加入具名欄位。

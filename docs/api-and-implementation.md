# REST API 與實作架構

* * *

## REST API

使用 endpoint：

```text
GET https://auditservice.dev.azure.com/{organization}/_apis/audit/auditlog
```

API 版本：

```text
7.1-preview.1
```

要求參數：

| 參數 | 用途 |
|---|---|
| `startTime` | 開始時間 |
| `endTime` | 結束時間 |
| `batchSize` | 每頁筆數 |
| `continuationToken` | 下一頁 token |
| `skipAggregation` | 是否略過 access log 聚合 |
| `api-version` | API 版本 |

本工具不存取 `https://miniasp.visualstudio.com/_settings/audit` 網頁，也不自動化瀏覽器。

* * *

## 回應結構相容性

Microsoft 官方範例是：

```json
{
  "value": {
    "decoratedAuditLogEntries": [],
    "continuationToken": null,
    "hasMore": false
  }
}
```

client 也接受直接結果：

```json
{
  "decoratedAuditLogEntries": [],
  "continuationToken": null,
  "hasMore": false
}
```

另外相容：

- `auditLogEntries`
- array 型別的最外層 `value`
- 字串或數字型別的 `continuationToken`

**回應解析不再假定最外層一定存在 object 型別的 `value`。** 若 Azure DevOps 回傳無法辨識的結構，錯誤訊息會列出最外層欄位，但不列出憑證。

* * *

## Rust 模組

```text
src/
├── auth.rs    環境變數與 Authorization header
├── client.rs  HTTP、重試、分頁與回應解析
├── error.rs   thiserror 公開錯誤型別
├── lib.rs     library 公開 API
├── main.rs    clap CLI 與輸出 writer
├── model.rs   稽核事件與分頁模型
└── query.rs   查詢驗證與 query string
```

CLI 使用 `anyhow` 組合使用者情境；library 使用具名 `AuditError`，讓呼叫端可程式化處理。

* * *

## 資料流

```text
環境變數憑證
    ↓
AuditClient
    ↓
AuditQuery
    ↓
AuditLogPager.next_page
    ↓
AuditPage
    ↓
JSON／JSON Lines／CSV writer
    ↓
同目錄暫存檔
    ↓
原子持久化為目標檔
```

每頁最多 200 筆。client 不會把所有頁面收集到記憶體後才回傳，因此 library 呼叫端與 CLI 都能逐頁處理。

* * *

## HTTP 行為

- 使用 `reqwest` 非同步 client
- 預設逾時 30 秒
- 預設重試四次
- 重試連線錯誤、timeout、`429` 與 `5xx`
- 遵守整數秒數 `Retry-After`
- 其他情況採上限 30 秒的指數退避
- 最多保留伺服器錯誤訊息前 1,000 個字元

* * *

## 測試範圍

- PAT `Debug` 遮罩
- 時間區間驗證
- query string 時間格式
- 直接結果回應
- `value` 包裝回應
- array `value` 回應
- 未知欄位保留
- continuation token 分頁
- Authorization header 產生
- JSON Lines 未知欄位輸出
- CSV 複合值跳脫
- npm 平台對應
- SHA-256 檢查
- Release 資產完整性檢查

整合測試使用本機 HTTP mock server，不呼叫 Azure DevOps，也不需要真實 PAT。

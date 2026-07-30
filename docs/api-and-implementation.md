# API 與實作

本工具直接呼叫 Azure DevOps Services 的 Audit REST API，不使用 Chrome、其他瀏覽器、網頁擷取或 Azure DevOps SDK。

* * *

## REST 端點

```http
GET https://auditservice.dev.azure.com/{organization}/_apis/audit/auditlog
```

固定 API 版本：

```text
7.1-preview.1
```

完整要求概念：

```http
GET /{organization}/_apis/audit/auditlog
    ?startTime={RFC3339}
    &endTime={RFC3339}
    &batchSize={integer}
    &continuationToken={token}
    &skipAggregation={boolean}
    &api-version=7.1-preview.1
```

官方端點：[Audit Log Query REST API](https://learn.microsoft.com/en-us/rest/api/azure/devops/audit/audit-log/query?view=azure-devops-rest-7.1)

* * *

## 要求標頭

共同標頭：

```http
Accept: application/json
User-Agent: ado-audit-log-exporter/1.0
```

Microsoft Entra：

```http
Authorization: Bearer {access-token}
```

PAT：

```http
Authorization: Basic {base64-encoded-colon-and-pat}
```

權杖不會放在 URL、查詢參數或輸出檔。

* * *

## 查詢參數

| 參數 | 工具行為 |
|---|---|
| `startTime` | 一律傳送；預設為啟動時刻往前 90 天 |
| `endTime` | 一律傳送；預設為啟動時刻 |
| `batchSize` | 一律傳送；預設為 200 |
| `continuationToken` | 第一頁省略，後續分頁使用前一頁回傳值 |
| `skipAggregation` | 一律傳送；預設為 `true` |
| `api-version` | 固定為 `7.1-preview.1` |

所有參數使用 URL encoding。組織名稱也會以 path segment 安全編碼。

* * *

## 時間處理

程式啟動時只擷取一次目前 UTC 時間，避免分別計算預設開始與結束造成不一致。

有效輸入：

```text
2026-06-01T00:00:00Z
2026-06-01T08:00:00+08:00
```

無效輸入：

```text
2026-06-01T00:00:00
```

缺少時區的時間會被拒絕。有效時間會正規化成：

```text
2026-06-01T00:00:00.000Z
```

開始時間不得晚於結束時間。

* * *

## 分頁流程

Azure DevOps 回應 `AuditLogQueryResult`：

```json
{
  "decoratedAuditLogEntries": [],
  "continuationToken": "opaque-token",
  "hasMore": true
}
```

流程：

1. 送出不含 `continuationToken` 的第一個要求。
2. 輸出 `decoratedAuditLogEntries`。
3. 若 `hasMore` 為 false 或缺少，結束。
4. 若 `hasMore` 為 true，要求必須有非空字串 `continuationToken`。
5. 將 token URL encode 後要求下一頁。
6. 若服務重複已見過的 token，停止並回報錯誤，避免無限迴圈。

`continuationToken` 是 opaque 值，工具不解析其內容。

* * *

## 回應結構相容性

實際 Azure DevOps 服務與官方 SDK 將 `AuditLogQueryResult` 直接放在最上層；Microsoft Learn 的部分回應範例則使用 `value` 包裝：

```json
{
  "value": {
    "decoratedAuditLogEntries": [],
    "continuationToken": null,
    "hasMore": false
  }
}
```

工具依下列順序判斷：

1. 最上層具有 `decoratedAuditLogEntries` 時，直接視為 `AuditLogQueryResult`。
2. 否則，若 `value` 是物件，使用 `value`。
3. 其他結構回報安全的結構摘要，只顯示頂層鍵名與 `value` 類型，不輸出完整回應內容。

這項相容性避免因服務與文件範例的包裝差異造成匯出失敗。

參考：

- [Microsoft Learn：AuditLogQueryResult 定義](https://learn.microsoft.com/en-us/rest/api/azure/devops/audit/audit-log/query?view=azure-devops-rest-7.1)
- [Microsoft Azure DevOps Go API：AuditLogQueryResult](https://pkg.go.dev/github.com/microsoft/azure-devops-go-api/azuredevops/audit#AuditLogQueryResult)

* * *

## 重試策略

可重試：

- HTTP 429。
- HTTP 500 至 599。
- `URLError` 所代表的連線、DNS、TLS 或其他網路錯誤。

不重試：

- HTTP 400 類型錯誤，但 429 除外。
- HTTP 401。
- HTTP 403。
- 無效 JSON。
- JSON 結構不符合預期。
- 本機輸出錯誤。

等待時間：

1. 若 HTTP 回應含可解析為秒數的 `Retry-After`，採用該值。
2. 否則採用指數退避：1、2、4、8 秒。
3. 單次退避上限為 30 秒。

預設 `--retries 4` 代表最多嘗試五次。

* * *

## 輸出串流

每一頁回傳後，事件立即序列化，不將 90 天完整資料累積在記憶體。

| 格式 | 寫入策略 |
|---|---|
| JSON | 先寫 `[`，逐筆加逗號，最後寫 `]` |
| JSON Lines | 每筆事件後加入換行 |
| CSV | 啟動時寫標頭，逐筆寫入資料列 |

每完成一頁，標準錯誤會顯示累計頁數與筆數。事件內容只進入指定輸出。

* * *

## 檔案原子性

指定 `--output PATH` 時使用與目標相同目錄的 `mkstemp` 暫存檔，確保最終 `os.replace` 不跨檔案系統。

成功：

```text
建立暫存檔 → 寫入所有事件 → flush → close → os.replace
```

失敗：

```text
建立暫存檔 → 部分寫入 → 發生錯誤 → 關閉並刪除暫存檔
```

在 POSIX 平台，`mkstemp` 建立的檔案權限是只有擁有者可讀寫。最終權限仍應由實際部署環境與檔案系統驗證。

* * *

## 實作限制

- API 仍標示為 preview，結構與行為可能改變。
- `Retry-After` 目前只解析數字秒數，不解析 HTTP-date。
- 工具不平行要求分頁，因為下一頁依賴上一頁的 continuation token。
- 工具不自動切割超過 90 天的時間範圍。
- 工具不對事件進行去重；重疊時間窗應由下游依 `id` 去重。
- 工具不驗證 `batchSize` 的服務端上限，只確保值大於零。

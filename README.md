# Azure DevOps 稽核記錄匯出工具

`ado-audit-log-exporter` 是一個只使用 Python 標準函式庫的命令列工具，透過 Azure DevOps Audit REST API 匯出組織層級稽核事件。

**工具不使用 Chrome、其他瀏覽器或網頁自動化，所有事件都直接從 Azure DevOps REST API 取得。**

預設設定：

| 項目 | 預設值 |
|---|---|
| Azure DevOps 組織 | `miniasp` |
| 查詢期間 | 執行時刻往前 90 天 |
| API 版本 | `7.1-preview.1` |
| 每頁事件數 | 200 |
| 輸出格式 | JSON Lines |
| 存取事件聚合 | 關閉，保留個別 `AuditLog.AccessLog` |
| HTTP 逾時 | 30 秒 |
| 重試次數 | 4 |

* * *

## 功能

- 直接呼叫 `auditservice.dev.azure.com`。
- 支援 Microsoft Entra Bearer 權杖。
- 預設從 `AZURE_DEVOPS_EXT_PAT` 讀取 PAT。
- 保留 `ADO_PAT` 相容性。
- 自動處理 `continuationToken`，取得所有分頁。
- 支援 JSON、JSON Lines 與 CSV。
- JSON 與 JSON Lines 保留完整 API 事件。
- CSV 以 `extraFields` 保存 API 未知欄位。
- HTTP 429、HTTP 5xx 與網路錯誤自動重試。
- 偵測 continuation token 重複，避免無限迴圈。
- 檔案輸出先寫入安全暫存檔，成功後再原子替換。
- 既有檔案預設不覆寫。
- 進度與資料分開輸出，方便命令列管線處理。
- 同時相容最上層與 `value` 包裝的 `AuditLogQueryResult`。

* * *

## 系統需求

- Python 3.9 以上。
- `make`，若只直接執行 Python 可省略。
- 可連線至 `https://auditservice.dev.azure.com`。
- Azure DevOps Services 組織。
- 權杖代表的身分具有組織層級 `View audit log` 權限。
- PAT 使用者需授予 `Read Audit Log`，API 範圍名稱為 `vso.auditlog`。

工具沒有第三方 Python 套件相依性。

檢查環境：

```sh
python3 --version
make --version
```

* * *

## 快速開始

### 使用 PAT

將 PAT 放入 Azure DevOps CLI 慣用的環境變數：

```sh
export AZURE_DEVOPS_EXT_PAT='replace-with-your-pat'
```

執行預設 JSON Lines 匯出：

```sh
make export
```

預設產生：

```text
ado-audit.jsonl
```

### 使用 Microsoft Entra

先登入 Azure CLI：

```sh
az login
```

由 Makefile 取得短效 Azure DevOps 權杖並匯出：

```sh
make export-entra
```

這個目標只在子程序環境內設定 `ADO_ACCESS_TOKEN`，不將權杖寫入專案檔案。

* * *

## 驗證方式與優先順序

支援的環境變數：

| 環境變數 | 類型 | 說明 |
|---|---|---|
| `AZURE_DEVOPS_EXT_PAT` | PAT | 預設 PAT 來源 |
| `ADO_PAT` | PAT | 相容性備援 |
| `ADO_ACCESS_TOKEN` | Microsoft Entra | Bearer 存取權杖 |

PAT 選擇順序：

1. `AZURE_DEVOPS_EXT_PAT`
2. `ADO_PAT`

若兩個 PAT 變數同時存在，使用 `AZURE_DEVOPS_EXT_PAT`。若任何 PAT 與 `ADO_ACCESS_TOKEN` 同時存在，工具會停止，避免驗證身分不明確。

手動取得 Entra 權杖：

```sh
export ADO_ACCESS_TOKEN="$(
  az account get-access-token \
    --resource 499b84ac-1321-427f-aa17-267ca6975798 \
    --query accessToken \
    --output tsv
)"
```

**權杖只應存在於環境或祕密管理系統，不要放入命令列參數、原始碼、Git 或輸出檔。**

詳細說明：[驗證與權限](docs/authentication.md)。

* * *

## 常用匯出方式

### 匯出 JSON Lines

```sh
make export-jsonl
```

### 匯出 JSON

```sh
make export-json
```

### 匯出 CSV

```sh
make export-csv
```

### 匯出指定月份

以下命令匯出 2026 年 6 月：

```sh
make export-csv \
  START_TIME=2026-06-01T00:00:00Z \
  END_TIME=2026-07-01T00:00:00Z \
  OUTPUT=miniasp-ado-audit-june.csv
```

### 允許覆寫

```sh
make export-csv \
  OUTPUT=miniasp-ado-audit-june.csv \
  OVERWRITE=1
```

### 指定其他組織

```sh
make export \
  ORGANIZATION=contoso \
  OUTPUT=contoso-ado-audit.jsonl
```

### 聚合存取事件

Azure DevOps 可聚合 `AuditLog.AccessLog`。工具預設保留個別事件；若要採用服務端聚合：

```sh
make export AGGREGATE_ACCESS_LOG=1
```

* * *

## Makefile

列出全部目標與變數：

```sh
make help
```

目標：

| 目標 | 說明 |
|---|---|
| `make export` | 使用環境權杖與指定格式匯出 |
| `make export-entra` | 由 Azure CLI 取得短效權杖後匯出 |
| `make export-json` | 匯出 JSON |
| `make export-jsonl` | 匯出 JSON Lines |
| `make export-csv` | 匯出 CSV |
| `make test` | 執行單元測試 |
| `make check` | 執行編譯檢查與單元測試 |
| `make help` | 顯示說明 |

常用變數：

| 變數 | 預設值 |
|---|---|
| `ORGANIZATION` | `miniasp` |
| `FORMAT` | `jsonl` |
| `OUTPUT` | `ado-audit.$(FORMAT)` |
| `START_TIME` | 空白，使用 90 天前 |
| `END_TIME` | 空白，使用目前時間 |
| `BATCH_SIZE` | `200` |
| `TIMEOUT` | `30` |
| `RETRIES` | `4` |
| `AGGREGATE_ACCESS_LOG` | `0` |
| `OVERWRITE` | `0` |
| `PYTHON` | `python3` |

完整參考：[命令參考](docs/command-reference.md)。

* * *

## 直接執行 Python

基本用法：

```sh
python3 export_ado_audit_logs.py \
  --organization miniasp \
  --format jsonl \
  --output ado-audit.jsonl
```

指定完整參數：

```sh
python3 export_ado_audit_logs.py \
  --organization miniasp \
  --start-time 2026-06-01T00:00:00Z \
  --end-time 2026-07-01T00:00:00Z \
  --format csv \
  --output miniasp-ado-audit-june.csv \
  --batch-size 200 \
  --timeout 60 \
  --retries 6
```

查看即時參數：

```sh
python3 export_ado_audit_logs.py --help
```

選項概要：

| 選項 | 說明 |
|---|---|
| `--organization` | Azure DevOps 組織名稱 |
| `--start-time` | RFC 3339 開始時間 |
| `--end-time` | RFC 3339 結束時間 |
| `--format` | `json`、`jsonl` 或 `csv` |
| `--output` | 檔案路徑或 `-` |
| `--batch-size` | 每頁要求事件數 |
| `--aggregate-access-log` | 允許服務聚合存取事件 |
| `--timeout` | HTTP 要求逾時秒數 |
| `--retries` | 可重試錯誤的重試次數 |
| `--overwrite` | 允許替換既有檔案 |

* * *

## 輸出格式

### JSON

單一 JSON 陣列，保留事件全部欄位。

```json
[
  {
    "id": "event-id",
    "timestamp": "2026-06-01T12:34:56.789Z",
    "actionId": "Project.CreateCompleted",
    "data": {
      "ProjectName": "sample"
    }
  }
]
```

### JSON Lines

每行一個完整事件物件，適合大量資料與串流處理。

```jsonl
{"id":"event-1","actionId":"Project.CreateCompleted","data":{"ProjectName":"sample"}}
{"id":"event-2","actionId":"Git.CreateRepo","data":{"RepoName":"sample-repo"}}
```

### CSV

CSV 固定輸出 25 欄。主要欄位：

- 識別：`id`、`correlationId`、`activityId`
- 時間與動作：`timestamp`、`actionId`、`area`、`category`
- 操作者：`actorCUID`、`actorClientId`、`actorUserId`、`actorUPN`
- 連線：`authenticationMechanism`、`ipAddress`、`userAgent`
- 範圍：`scopeType`、`scopeId`、`projectId`、`projectName`
- 動作資料：`data`
- 未知欄位：`extraFields`

`data` 與 `extraFields` 在 CSV 中保存為緊湊 JSON 字串。不同 `actionId` 的 `data` 結構不同。

完整欄位表：[輸出格式](docs/output-formats.md)。

* * *

## API 行為

工具呼叫：

```text
GET https://auditservice.dev.azure.com/{organization}/_apis/audit/auditlog
```

查詢參數：

- `startTime`
- `endTime`
- `batchSize`
- `continuationToken`
- `skipAggregation`
- `api-version=7.1-preview.1`

Azure DevOps 回傳 `hasMore=true` 時，工具將 `continuationToken` URL encode 後要求下一頁，直到 `hasMore` 為 false。

工具支援兩種實際可見的回應結構：

1. 最上層 `AuditLogQueryResult`。
2. `value` 包裝的 `AuditLogQueryResult`。

完整流程：[API 與實作](docs/api-and-implementation.md)。

* * *

## 檔案安全與失敗處理

指定 `--output PATH` 時：

1. 在目標目錄建立隨機暫存檔。
2. 將所有分頁逐筆寫入。
3. 全部成功後以原子替換移至目標路徑。
4. 失敗時刪除暫存檔。

因此 API 中途失敗不會留下表面正常但內容不完整的目標檔。標準輸出模式不具備此保護。

名稱包含 `ado-audit` 的 JSON、JSON Lines 與 CSV 已列入 `.gitignore`。提交前仍應檢查：

```sh
git status --short --ignored
git diff --cached --name-only
```

* * *

## 測試

執行全部單元測試：

```sh
make test
```

執行編譯檢查與測試：

```sh
make check
```

測試範圍包括：

- PAT 與 Bearer 標頭。
- `AZURE_DEVOPS_EXT_PAT` 優先順序。
- 驗證類型衝突。
- RFC 3339 與 UTC 正規化。
- 直接與 `value` 包裝回應。
- continuation token 分頁與 URL encoding。
- HTTP 429 重試。
- 非預期回應的安全錯誤摘要。
- JSON Lines 巢狀資料。
- CSV `data` 與未知欄位。

測試不需真實 Azure DevOps 權杖，也不會連線至 Azure DevOps。

* * *

## 限制

- Azure DevOps Audit API 目前仍是 preview。
- Azure DevOps Services 稽核事件只保留 90 天。
- Auditing 僅適用於以 Microsoft Entra ID 支援的 Azure DevOps Services 組織。
- Azure DevOps 不記錄 Microsoft Entra 登入事件。
- Microsoft Entra 群組內部的成員異動不一定出現在 Azure DevOps 稽核記錄。
- 工具不自動切割超過 90 天的時間範圍。
- 工具不自動去重；重疊時間窗應依事件 `id` 去重。
- 工具不建立 Audit Streaming。
- 工具不負責長期保存、SIEM 匯入或權杖輪替。

正式維運建議：[安全與維運](docs/security-and-operations.md)。

* * *

## 專案結構

```text
.
├── .gitignore
├── Makefile
├── README.md
├── docs/
│   ├── api-and-implementation.md
│   ├── authentication.md
│   ├── command-reference.md
│   ├── getting-started.md
│   ├── index.md
│   ├── output-formats.md
│   ├── references.md
│   ├── security-and-operations.md
│   └── troubleshooting.md
├── export_ado_audit_logs.py
└── test_export_ado_audit_logs.py
```

* * *

## 文件

| 文件 | 主題 |
|---|---|
| [文件中心](docs/index.md) | 文件導覽與工具範圍 |
| [快速開始](docs/getting-started.md) | 第一次匯出 |
| [驗證與權限](docs/authentication.md) | PAT、Entra 與 Azure DevOps 權限 |
| [命令參考](docs/command-reference.md) | Makefile 與 Python 選項 |
| [輸出格式](docs/output-formats.md) | 格式與欄位 |
| [API 與實作](docs/api-and-implementation.md) | REST、分頁、重試與寫入 |
| [疑難排解](docs/troubleshooting.md) | 錯誤處理 |
| [安全與維運](docs/security-and-operations.md) | 保存、排程與安全 |
| [參考資料](docs/references.md) | 官方原始連結 |

* * *

## 官方參考資料

- [Audit Log Query REST API](https://learn.microsoft.com/en-us/rest/api/azure/devops/audit/audit-log/query?view=azure-devops-rest-7.1)
- [存取、匯出與篩選 Azure DevOps 稽核記錄](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/azure-devops-auditing?view=azure-devops)
- [Azure DevOps 稽核事件清單](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/auditing-events?view=azure-devops)
- [Azure DevOps 驗證方式指引](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/authentication-guidance?view=azure-devops)
- [使用 Azure CLI 發行 Microsoft Entra 權杖](https://learn.microsoft.com/en-us/azure/devops/cli/entra-tokens?view=azure-devops)
- [使用 Azure DevOps PAT](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops)
- [使用 `AZURE_DEVOPS_EXT_PAT`](https://learn.microsoft.com/en-us/azure/devops/cli/log-in-via-pat?view=azure-devops)
- [建立 Azure DevOps Audit Streaming](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/auditing-streaming?view=azure-devops)

完整清單與用途：[參考資料](docs/references.md)。

# 輸出格式

Azure DevOps Audit Log Query REST API 回傳 JSON。工具抽出每一頁的 `decoratedAuditLogEntries`，再序列化為 JSON、JSON Lines 或 CSV。

**JSON 與 JSON Lines 保留事件的全部 API 欄位；CSV 將固定欄位展開，並以 `extraFields` 保存未識別欄位。**

* * *

## 格式比較

| 格式 | 結構 | 記憶體特性 | 適合用途 |
|---|---|---|---|
| JSON | 單一事件陣列 | 逐筆寫入，不在程式中累積完整陣列 | API 交換、一次載入 |
| JSON Lines | 每行一個事件物件 | 逐筆寫入 | 大量資料、串流、SIEM、命令列處理 |
| CSV | 標頭列加事件列 | 逐筆寫入 | Excel、試算表、表格工具 |

所有文字檔：

- 使用 UTF-8。
- 不加入 UTF-8 BOM。
- 保留非 ASCII 字元，不轉成 `\uXXXX`。
- JSON 採緊湊格式，減少輸出大小。

* * *

## JSON

選擇：

```sh
make export-json
```

結構：

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

實際檔案是緊湊 JSON，不加入示範中的縮排。

特性：

- 根節點固定為陣列。
- 空結果輸出 `[]`。
- 保留 `data` 巢狀物件。
- 保留服務未來新增的欄位。

* * *

## JSON Lines

選擇：

```sh
make export-jsonl
```

結構：

```jsonl
{"id":"event-1","actionId":"Project.CreateCompleted","data":{"ProjectName":"sample"}}
{"id":"event-2","actionId":"Git.CreateRepo","data":{"RepoName":"sample-repo"}}
```

每一行都是獨立且有效的 JSON 物件。JSON Lines 適合：

- 逐行壓縮或傳輸。
- 使用 `jq -c`、Logstash、Fluent Bit 或其他串流工具。
- 單筆失敗隔離。
- 不需讀取完整檔案即可開始處理。

JSON Lines 不是 Audit REST API 的原生媒體類型，而是本工具提供的本機序列化格式。

* * *

## CSV

選擇：

```sh
make export-csv
```

CSV 固定輸出 25 欄：

| 欄位 | 來源 | 說明 |
|---|---|---|
| `id` | API | 事件唯一識別碼 |
| `correlationId` | API | 關聯同一連鎖操作的識別碼 |
| `activityId` | API | 活動識別碼 |
| `timestamp` | API | UTC 事件時間 |
| `actionId` | API | 動作種類，例如 `Git.CreateRepo` |
| `area` | API | Azure DevOps 產品區域 |
| `category` | API | `unknown`、`modify`、`remove`、`create`、`access` 或 `execute` |
| `categoryDisplayName` | API | 類別顯示名稱 |
| `details` | API | 已修飾的人類可讀事件描述 |
| `actorCUID` | API | 操作者 CUID |
| `actorClientId` | API | 操作者為 service principal 時的 client ID |
| `actorUserId` | API | 操作者為使用者時的 Azure DevOps user ID |
| `actorUPN` | API | 操作者 UPN |
| `actorDisplayName` | API | 操作者顯示名稱 |
| `actorImageUrl` | API | 操作者頭像 URL |
| `authenticationMechanism` | API | 操作者採用的驗證機制 |
| `ipAddress` | API | 事件來源 IP 位址 |
| `userAgent` | API | 請求 User-Agent |
| `scopeType` | API | 事件範圍類型 |
| `scopeDisplayName` | API | 範圍顯示名稱 |
| `scopeId` | API | 組織或範圍識別碼 |
| `projectId` | API | 關聯專案識別碼 |
| `projectName` | API | 關聯專案名稱 |
| `data` | API | 動作特定資料，序列化為緊湊 JSON 字串 |
| `extraFields` | 工具 | 未列入固定欄位的 API 欄位，序列化為緊湊 JSON 字串 |

CSV 內的 `data` 範例：

```text
{"ProjectId":"...","ProjectName":"sample","ProjectVisibility":"Private"}
```

不同 `actionId` 的 `data` 結構不同，不能假設所有事件具有相同子欄位。

* * *

## 欄位空值

Azure DevOps 可省略不適用的欄位或回傳 `null`。

| 格式 | 空值表示 |
|---|---|
| JSON | 保留 API 原始 `null` 或欄位缺少狀態 |
| JSON Lines | 與 JSON 相同 |
| CSV | 空字串 |

CSV 無法區分 API 原始 `null` 與欄位不存在。需要保留這項差異時，使用 JSON 或 JSON Lines。

* * *

## 未知欄位相容性

Audit API 是 preview 版本，服務可能新增欄位。

- JSON 與 JSON Lines：完整保存未知欄位。
- CSV：未知欄位集中保存於 `extraFields`。

`extraFields` 本身若由未來 API 提供，會作為未知來源欄位包含在工具產生的 `extraFields` JSON 物件中，不會覆寫工具欄位。

* * *

## 存取事件聚合

Azure DevOps 預設可能聚合 `AuditLog.AccessLog`。工具為取得個別記錄，預設送出：

```text
skipAggregation=true
```

允許服務聚合：

```sh
make export AGGREGATE_ACCESS_LOG=1
```

或：

```sh
python3 export_ado_audit_logs.py --aggregate-access-log
```

聚合後的事件可能在 `data` 中包含摘要資訊，事件筆數也會下降。這不代表其他類型事件遺失，而是 `AuditLog.AccessLog` 的服務端表示方式改變。

* * *

## 檔案寫入保證

指定檔案路徑時：

1. 工具在目標檔所在目錄建立隨機暫存檔。
2. 所有 API 分頁與輸出成功後關閉暫存檔。
3. 使用作業系統原子替換將暫存檔移至目標路徑。
4. 中途失敗時刪除暫存檔，不留下不完整的目標檔。

若目標檔已存在且未指定 `--overwrite`，工具在發出 API 請求前停止。

標準輸出模式沒有上述檔案保證。將輸出導向檔案時，shell 會先建立或截斷目標檔，因此若需要失敗保護，應使用 `--output PATH`，不要使用 shell 的 `>`。

* * *

## 敏感性

稽核輸出可能包含：

- 使用者名稱、UPN 與識別碼。
- IP 位址。
- User-Agent。
- 專案、儲存庫、群組、權限與資源名稱。
- PAT、權限或組織設定變更的事件描述。

**輸出檔不含本工具使用的 PAT 或 Bearer 權杖，但事件本身仍應視為敏感安全資料。**

儲存與保留建議請參閱 [安全與維運](security-and-operations.md)。

* * *

## 官方結構參考

- [Audit Log Query REST API 與 `DecoratedAuditLogEntry` 定義](https://learn.microsoft.com/en-us/rest/api/azure/devops/audit/audit-log/query?view=azure-devops-rest-7.1)
- [Azure DevOps 稽核事件清單](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/auditing-events?view=azure-devops)

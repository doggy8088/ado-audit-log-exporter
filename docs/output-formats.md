# 輸出格式

工具支援 JSON、JSON Lines 與 CSV。所有文字輸出都是 UTF-8。

* * *

## JSON

使用：

```sh
ado-audit-log-exporter --format json --output audit.json
```

結構是一個 JSON array：

```json
[
  {
    "id": "event-id",
    "timestamp": "2026-07-01T00:00:00Z",
    "actionId": "Git.Push",
    "area": "Git",
    "category": "modify",
    "details": "Pushed updates",
    "data": {
      "RepositoryName": "sample"
    }
  }
]
```

適合一次載入完整資料、交換或匯入接受 JSON array 的系統。

* * *

## JSON Lines

使用：

```sh
ado-audit-log-exporter --format jsonl --output audit.jsonl
```

每行是一個完整 JSON object：

```jsonl
{"id":"event-1","actionId":"Git.Push"}
{"id":"event-2","actionId":"Security.ModifyPermission"}
```

適合：

- 大型匯出
- 串流處理
- 逐行失敗復原
- `jq`、Logstash 或其他日誌管線

查詢特定 action：

```sh
jq -c 'select(.actionId == "Git.Push")' audit.jsonl
```

* * *

## CSV

使用：

```sh
ado-audit-log-exporter --format csv --output audit.csv
```

固定欄位順序：

| 順序 | 欄位 |
|---:|---|
| 1–4 | `id`、`correlationId`、`activityId`、`timestamp` |
| 5–9 | `actionId`、`area`、`category`、`categoryDisplayName`、`details` |
| 10–16 | `actorCUID`、`actorClientId`、`actorUserId`、`actorUPN`、`actorDisplayName`、`actorImageUrl`、`authenticationMechanism` |
| 17–20 | `ipAddress`、`userAgent`、`scopeType`、`scopeDisplayName` |
| 21–24 | `scopeId`、`projectId`、`projectName`、`data` |
| 25 | `extraFields` |

`details`、`data` 或未知複合欄位若是 object 或 array，會以壓縮 JSON 寫入單一 CSV 儲存格。CSV writer 會正確處理逗號、引號與換行。

CSV 儲存格若以 `=`、`+`、`-` 或 `@` 開頭，工具會加上單引號，避免 Excel 或試算表將稽核內容解讀成公式。這項中和只套用於 CSV；JSON 與 JSON Lines 保留服務端傳回的原始值。

* * *

## 未知欄位

Azure DevOps 可能新增事件專屬欄位。Rust 模型使用 Serde flatten 保留未知欄位：

- JSON：未知欄位仍位於原本 object 層級
- JSON Lines：未知欄位仍位於原本 object 層級
- CSV：未知欄位合併為 `extraFields` JSON object

**這項相容設計避免 API 演進時靜默遺失資料，但不代表新欄位的語意已被本工具驗證。**

* * *

## 資料注意事項

- `timestamp` 保留 Azure DevOps 傳回的字串精度
- 未提供欄位在 JSON 中省略，在 CSV 中為空白
- JSON object 的欄位順序不應視為契約
- `data` 的結構依 `actionId` 改變
- `details` 可能是字串或其他 JSON 值
- 同一 `correlationId` 可用來關聯多筆事件

稽核記錄可能含個資與安全資訊，輸出格式不會自行匿名化或遮罩事件內容。

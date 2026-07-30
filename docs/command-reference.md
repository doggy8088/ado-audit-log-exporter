# 命令參考

工具可透過 Makefile 或直接執行 Python。Makefile 提供常用預設值；Python 命令列提供完整控制。

* * *

## Makefile 目標

| 目標 | 說明 |
|---|---|
| `make` | 顯示說明，等同 `make help` |
| `make help` | 列出目標與可覆寫變數 |
| `make export` | 使用環境中的權杖，以 `FORMAT` 匯出 |
| `make export-entra` | 透過 Azure CLI 取得短效 Entra 權杖後執行 `export` |
| `make export-json` | 將 `FORMAT` 設為 `json` 後執行 `export` |
| `make export-jsonl` | 將 `FORMAT` 設為 `jsonl` 後執行 `export` |
| `make export-csv` | 將 `FORMAT` 設為 `csv` 後執行 `export` |
| `make test` | 執行單元測試 |
| `make check` | 執行 Python 編譯檢查與單元測試 |

基本命令：

```sh
make export
make export-json
make export-jsonl
make export-csv
```

* * *

## Makefile 變數

| 變數 | 預設值 | 傳給 Python 的選項 |
|---|---|---|
| `PYTHON` | `python3` | Python 執行檔 |
| `ORGANIZATION` | `miniasp` | `--organization` |
| `FORMAT` | `jsonl` | `--format` |
| `OUTPUT` | `ado-audit.$(FORMAT)` | `--output` |
| `START_TIME` | 空白 | `--start-time`，空白時由 Python 使用 90 天前 |
| `END_TIME` | 空白 | `--end-time`，空白時由 Python 使用目前時間 |
| `BATCH_SIZE` | `200` | `--batch-size` |
| `TIMEOUT` | `30` | `--timeout` |
| `RETRIES` | `4` | `--retries` |
| `AGGREGATE_ACCESS_LOG` | `0` | 設為 `1` 時加入 `--aggregate-access-log` |
| `OVERWRITE` | `0` | 設為 `1` 時加入 `--overwrite` |

覆寫多個變數：

```sh
make export-csv \
  ORGANIZATION=miniasp \
  START_TIME=2026-06-01T00:00:00Z \
  END_TIME=2026-07-01T00:00:00Z \
  OUTPUT=miniasp-ado-audit-june.csv \
  BATCH_SIZE=200 \
  TIMEOUT=60 \
  RETRIES=6
```

若輸出名稱沒有明確指定，會依格式產生：

| 目標 | 預設輸出 |
|---|---|
| `make export-json` | `ado-audit.json` |
| `make export-jsonl` | `ado-audit.jsonl` |
| `make export-csv` | `ado-audit.csv` |

* * *

## Python 命令列概要

查看即時說明：

```sh
python3 export_ado_audit_logs.py --help
```

語法：

```text
export_ado_audit_logs.py
  [--organization ORGANIZATION]
  [--start-time RFC3339]
  [--end-time RFC3339]
  [--format {json,jsonl,csv}]
  [--output PATH]
  [--batch-size BATCH_SIZE]
  [--aggregate-access-log]
  [--timeout SECONDS]
  [--retries RETRIES]
  [--overwrite]
```

* * *

## Python 選項

### `--organization`

Azure DevOps 組織名稱。

- 預設：`miniasp`
- 不要傳入完整 URL。

正確：

```sh
python3 export_ado_audit_logs.py --organization miniasp
```

### `--start-time`

查詢開始時間。

- 格式：RFC 3339。
- 必須包含 `Z` 或 UTC offset。
- 預設：程式啟動時刻往前 90 天。

```sh
--start-time 2026-06-01T00:00:00Z
--start-time 2026-06-01T08:00:00+08:00
```

### `--end-time`

查詢結束時間。

- 格式：RFC 3339。
- 必須包含 `Z` 或 UTC offset。
- 預設：程式啟動時刻。
- 不得早於 `--start-time`。

### `--format`

輸出格式。

- 可用值：`json`、`jsonl`、`csv`
- 預設：`jsonl`

### `--output`

輸出位置。

- 預設：`-`
- `-` 表示標準輸出。
- 指定檔案時，父目錄不存在會自動建立。
- 既有檔案預設不覆寫。

寫到檔案：

```sh
--output audit/miniasp.jsonl
```

寫到標準輸出：

```sh
--output -
```

### `--batch-size`

每個 REST API 分頁要求的最大事件數。

- 預設：`200`
- 必須大於零。
- 服務端仍可回傳少於要求數量的事件。

### `--aggregate-access-log`

允許 Azure DevOps 聚合 `AuditLog.AccessLog` 事件。

- 未指定時，工具送出 `skipAggregation=true`。
- 指定時，工具送出 `skipAggregation=false`。

此選項只影響 Azure DevOps 官方指定可聚合的 `AuditLog.AccessLog` 類型。

### `--timeout`

單一 HTTP 請求逾時秒數。

- 預設：`30`
- 可使用小數。
- 必須大於零。

### `--retries`

網路錯誤、HTTP 429 與 HTTP 5xx 的重試次數。

- 預設：`4`
- `0` 表示不重試。
- 不可為負數。

總嘗試次數等於 `1 + retries`。

### `--overwrite`

允許成功輸出取代既有目標檔。

未指定此選項時，發現目標檔存在會立即停止，不會發出 REST API 請求。

* * *

## 標準輸出與標準錯誤

事件資料寫入：

- 指定檔案，或
- `--output -` 所代表的標準輸出。

進度與錯誤寫入標準錯誤，因此可以安全管線處理資料：

```sh
python3 export_ado_audit_logs.py --format jsonl --output - \
  | gzip > ado-audit.jsonl.gz
```

注意：標準輸出不具備檔案輸出的原子替換保護。若中途失敗，管線接收端可能保留部分資料。

* * *

## 結束狀態

| 狀態 | 意義 |
|---|---|
| `0` | 匯出成功，或管線接收端正常關閉造成 Broken Pipe |
| `1` | Python 參數、驗證、網路、API 或輸出錯誤 |
| `2` | Make 配方失敗時由 Make 回傳的常見狀態 |

Make 的非零狀態不一定等於 Python 的原始狀態；診斷時應閱讀前一行錯誤訊息。

* * *

## 日期標準

時間字串依 [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) 解析。所有有效輸入都會轉換為 UTC，並以毫秒精度與 `Z` 後綴傳給 Azure DevOps。

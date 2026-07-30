# CLI 與 Makefile 參考

* * *

## CLI 語法

```text
ado-audit-log-exporter [OPTIONS]
```

| 選項 | 預設值 | 說明 |
|---|---|---|
| `--organization <NAME>` | `miniasp` | Azure DevOps 組織名稱 |
| `--start-time <RFC3339>` | 執行時間前 30 天 | 查詢開始時間 |
| `--end-time <RFC3339>` | 執行時間 | 查詢結束時間 |
| `--format <FORMAT>` | `jsonl` | `json`、`jsonl` 或 `csv` |
| `--output <PATH>` | `ado-audit.<format>` | 輸出檔 |
| `--batch-size <1..200>` | `200` | 每頁最多筆數 |
| `--timeout <SECONDS>` | `30` | 每次 HTTP 要求逾時秒數 |
| `--retries <COUNT>` | `4` | 連線錯誤、逾時、`429` 或 `5xx` 的重試次數 |
| `--aggregate-access-log` | 關閉 | 保留 Azure DevOps 聚合的 access log |
| `--overwrite` | 關閉 | 覆寫既有輸出檔 |
| `--help` | 無 | 顯示說明 |
| `--version` | 無 | 顯示版本 |

**開始時間必須早於結束時間。** RFC 3339 必須包含 `Z` 或明確 UTC offset。

* * *

## 查詢與分頁

工具固定使用 API `7.1-preview.1`，送出：

- `startTime`
- `endTime`
- `batchSize`
- `skipAggregation`
- `continuationToken`
- `api-version`

每一頁完成後使用服務端的 `continuationToken` 讀取下一頁。若服務端重複傳回相同 token，工具會中止以避免無限迴圈。

* * *

## 重試

預設首次要求後最多再重試四次。以下情況會重試：

- TCP 連線失敗
- HTTP timeout
- HTTP `429`
- HTTP `5xx`

若 `Retry-After` 是整數秒數，工具會依照它等待；否則採 1、2、4、8 秒的指數退避，單次最長 30 秒。

`400`、`401`、`403` 與其他非暫時性 `4xx` 不會重試。

* * *

## 寫檔行為

輸出流程：

1. 在目標檔同一個目錄建立暫存檔
2. 串流寫入每一頁資料
3. 完成格式結尾並 flush
4. 將暫存檔持久化為目標檔

API 或寫檔失敗時，不會留下看似完整的目標檔。預設拒絕覆寫現有檔案；只有 `--overwrite` 會允許替換。

* * *

## Makefile 變數

| 變數 | 預設值 |
|---|---|
| `CARGO` | `cargo` |
| `NPM` | `npm` |
| `ORGANIZATION` | `miniasp` |
| `FORMAT` | `jsonl` |
| `OUTPUT` | `ado-audit.$(FORMAT)` |
| `START_TIME` | 空白 |
| `END_TIME` | 空白 |
| `BATCH_SIZE` | `200` |
| `TIMEOUT` | `30` |
| `RETRIES` | `4` |
| `AGGREGATE_ACCESS_LOG` | `0` |
| `OVERWRITE` | `0` |

布林變數使用 `0` 或 `1`。

### 範例

```sh
make export-json \
  ORGANIZATION=miniasp \
  START_TIME=2026-07-01T00:00:00Z \
  END_TIME=2026-07-02T00:00:00Z \
  OUTPUT=audit.json \
  OVERWRITE=1
```

```sh
make export-jsonl \
  ORGANIZATION=miniasp \
  BATCH_SIZE=100 \
  TIMEOUT=60 \
  RETRIES=6
```

* * *

## 開發目標

| 目標 | 執行內容 |
|---|---|
| `make build` | `cargo build --release --locked` |
| `make install` | `cargo install --path . --locked` |
| `make test` | Rust 與 Node 測試 |
| `make npm-check` | npm 安裝、測試、dry-run 封裝 |
| `make check` | fmt、Clippy、Rust 測試、npm 測試、封裝 |
| `make release-asset-check` | 檢查同版本 GitHub Release 資產 |
| `make clean` | 移除 Cargo 與 npm 本機執行檔產物 |

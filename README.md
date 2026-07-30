# Azure DevOps 稽核記錄匯出工具

透過 Azure DevOps Audit Log REST API 匯出組織層級的稽核記錄。專案同時提供：

- Rust 原生 CLI
- 可供 Rust 專案引用的非同步 library
- 從 GitHub Releases 下載原生執行檔的 npm 薄包裝
- JSON、JSON Lines 與 CSV 三種輸出格式
- macOS、Linux 與 Windows 的 x64 或 ARM64 執行檔

**工具只透過 REST API 讀取稽核記錄，不使用 Chrome，也不會把 PAT 寫入命令列參數、輸出檔或日誌。**

目前準備發布版本為 `0.1.1`。**GitHub Release 與 npm 的 `0.1.0` 已公開；crates.io crate 尚未發布。**

* * *

## 支援平台

| 作業系統 | CPU | Rust target | GitHub Release 資產 |
|---|---|---|---|
| macOS | Apple Silicon | `aarch64-apple-darwin` | `.tar.xz` 與 `.sha256` |
| macOS | Intel x64 | `x86_64-apple-darwin` | `.tar.xz` 與 `.sha256` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `.tar.xz` 與 `.sha256` |
| Linux | x64 | `x86_64-unknown-linux-gnu` | `.tar.xz` 與 `.sha256` |
| Windows | x64 | `x86_64-pc-windows-msvc` | `.zip` 與 `.sha256` |

npm 套件本身不包含原生執行檔。安裝時會依平台從同版本的 GitHub Release 下載壓縮檔，驗證 SHA-256 後再安裝，因此 npm 套件維持精簡。

* * *

## 必要條件

- Azure DevOps Services 組織已啟用 Auditing
- 呼叫身分具有 `View audit log` 權限
- 使用 PAT 時，PAT 具有 `vso.auditlog` scope
- 稽核資料仍在 Azure DevOps 保留期限內

Microsoft 文件指出 Azure DevOps 稽核記錄保留 90 天，而且 Auditing 只適用於連接 Microsoft Entra ID 的 Azure DevOps Services 組織，不適用於內部部署 Azure DevOps Server。長期保存應將匯出檔存入受控儲存空間，或採用 Audit Streaming。

* * *

## 安裝

### npm

已可從 npm 安裝：

```sh
npm install --global ado-audit-log-exporter
ado-audit-log-exporter --version
```

需要 Node.js `22.14.0` 以上。npm 安裝程序另需能連線至 GitHub Releases，且系統具有：

- macOS 或 Linux：`tar` 與 xz 支援
- Windows：PowerShell `Expand-Archive`

### Cargo

發布至 crates.io 後可執行：

```sh
cargo install ado-audit-log-exporter --locked
```

從目前原始碼安裝：

```sh
git clone https://github.com/doggy8088/ado-audit-log-exporter.git
cd ado-audit-log-exporter
cargo install --path . --locked
```

### GitHub Release

可直接從 GitHub Release 下載對應平台的壓縮檔與 `.sha256`。解壓縮後得到單一 `ado-audit-log-exporter` 執行檔；Windows 檔名為 `ado-audit-log-exporter.exe`。

* * *

## 驗證

**預設讀取 `AZURE_DEVOPS_EXT_PAT` 環境變數。** 工具支援以下順序：

1. `AZURE_DEVOPS_EXT_PAT`
2. `ADO_PAT`
3. `ADO_ACCESS_TOKEN`

前兩項是 PAT；第三項是 Microsoft Entra access token。若同時設定任一 PAT 與 `ADO_ACCESS_TOKEN`，工具會中止，避免使用到非預期身分。

macOS 或 Linux：

```sh
export AZURE_DEVOPS_EXT_PAT='你的 PAT'
```

PowerShell：

```powershell
$env:AZURE_DEVOPS_EXT_PAT = '你的 PAT'
```

請勿把 token 寫入 Makefile、shell 歷程、`.env`、Git 設定或命令列參數。完整說明見 [驗證文件](docs/authentication.md)。

* * *

## 快速使用

匯出 `miniasp` 組織在指定時間範圍內的 CSV：

```sh
ado-audit-log-exporter \
  --organization miniasp \
  --start-time 2026-06-01T00:00:00Z \
  --end-time 2026-07-01T00:00:00Z \
  --format csv \
  --output miniasp-ado-audit-june.csv
```

使用 Makefile：

```sh
make export-csv \
  START_TIME=2026-06-01T00:00:00Z \
  END_TIME=2026-07-01T00:00:00Z \
  OUTPUT=miniasp-ado-audit-june.csv
```

沒有指定時間時，預設查詢執行當下之前 30 天到執行當下。沒有指定格式時，預設輸出 `ado-audit.jsonl`。

既有輸出檔不會被覆寫。如需明確覆寫：

```sh
ado-audit-log-exporter \
  --organization miniasp \
  --format jsonl \
  --output ado-audit.jsonl \
  --overwrite
```

* * *

## 可取得的記錄格式

| 格式 | CLI 值 | 結構 | 適用情境 |
|---|---|---|---|
| JSON | `json` | 一個 JSON array | 完整交換、API 或程式載入 |
| JSON Lines | `jsonl` | 每行一個 JSON object | 串流處理、大型資料集、`jq` |
| CSV | `csv` | 固定欄位表格 | Excel、試算表、SIEM 前處理 |

Azure DevOps 常見欄位包括：

- 識別：`id`、`correlationId`、`activityId`
- 時間與動作：`timestamp`、`actionId`、`area`、`category`
- 行為者：`actorUserId`、`actorUPN`、`actorDisplayName`
- 來源：`ipAddress`、`userAgent`、`authenticationMechanism`
- 範圍：`scopeType`、`scopeDisplayName`、`scopeId`
- 詳細資料：`details`、`data`

**Azure DevOps 日後新增的未知欄位不會被丟棄。** JSON 與 JSON Lines 會保留原欄位；CSV 會將未知欄位寫入 `extraFields` JSON 欄。

REST API 官方範例將 `AuditLogQueryResult` 包在最外層 `value` object 中，但實際服務也可能直接回傳結果 object。client 同時接受這兩種結構，另外相容 array 型別的 `value`，避免因固定要求 `value` 為 object 而匯出失敗。

完整欄位與資料型別說明見 [輸出格式](docs/output-formats.md)。

* * *

## Rust library

`ado_audit_log_exporter` 提供非同步分頁 API：

```rust
use ado_audit_log_exporter::{
    AuditClient, AuditQuery, Authentication,
};
use chrono::{Duration, Utc};

# async fn export() -> Result<(), ado_audit_log_exporter::AuditError> {
let authentication = Authentication::from_env()?;
let client = AuditClient::new("miniasp", authentication)?;
let query = AuditQuery::new(
    Utc::now() - Duration::days(1),
    Utc::now(),
)?;
let mut pager = client.pager(query);

while let Some(page) = pager.next_page().await? {
    for entry in page.entries {
        println!("{:?}", entry.action_id);
    }
}
# Ok(())
# }
```

公開 API 包含：

- `Authentication`
- `AuditClient`
- `AuditQuery`
- `AuditLogPager`
- `AuditPage`
- `AuditLogEntry`
- `RetryPolicy`
- `AuditError`

詳細整合方式見 [Rust library 使用說明](docs/rust-library.md)。

* * *

## Makefile 目標

| 目標 | 用途 |
|---|---|
| `make build` | 建置 release 原生執行檔 |
| `make install` | 透過 Cargo 安裝本機 CLI |
| `make export` | 匯出指定格式 |
| `make export-entra` | 透過 Azure CLI 取得暫時 access token |
| `make export-json` | 匯出 JSON |
| `make export-jsonl` | 匯出 JSON Lines |
| `make export-csv` | 匯出 CSV |
| `make test` | 執行 Rust 與 npm 測試 |
| `make check` | 執行完整本機品質檢查 |
| `make release-asset-check` | 確認目前版本的 Release 資產齊全 |

可用 `make help` 查看變數。

* * *

## CI/CD 與發布

GitHub Actions 包含三個工作流程：

- `ci.yml`：格式、Clippy、Rust 測試、npm 測試與封裝檢查
- `release.yml`：由 `v*.*.*` 標籤建置五個平台，產生 SHA-256 並建立 GitHub Release
- `npm-publish.yml`：由 GitHub Release 事件透過 npm trusted publishing 發布

**工作流程不使用 `NPM_TOKEN` 或 `NODE_AUTH_TOKEN`。** npm 發布透過 GitHub Actions OIDC 取得短效憑證，且 npm 會自動產生 provenance。

`0.1.0` 已由維護者完成人工 bootstrap。從 `0.1.1` 起，GitHub Release 建立後由 `npm-publish.yml` 透過 trusted publishing 發布。完整流程見 [發布手冊](docs/releasing.md)。

* * *

## 安全設計

- 憑證只從環境變數讀取
- `Authentication` 的 Rust `Debug` 輸出會遮罩 token
- HTTP client 的 `Debug` 不輸出驗證標頭
- 匯出檔以同目錄暫存檔寫入，成功後才原子替換
- 預設拒絕覆寫現有檔案
- npm 安裝強制核對 SHA-256
- npm 發布工作流程採 OIDC，儲存庫不需要長效 npm token
- `.gitignore` 排除匯出記錄、本機 binary、`.env` 與封裝產物

稽核記錄本身可能含帳號、IP、User-Agent、資源名稱與管理操作內容，應視為敏感資料。見 [安全與操作](docs/security-and-operations.md)。

* * *

## 開發與驗證

最低支援 Rust 版本為 `1.85`，npm 發布要求 Node.js `22.14.0` 以上。

```sh
make check
```

等同執行主要檢查：

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --locked
npm ci --ignore-scripts
npm test
npm pack --dry-run
```

在同版本 GitHub Release 尚未建立前，`make release-asset-check` 預期失敗；這項失敗表示 npm 發布防護正在阻止缺少原生執行檔的版本上架。

* * *

## 文件

- [文件索引](docs/index.md)
- [快速開始](docs/getting-started.md)
- [驗證與權限](docs/authentication.md)
- [CLI 與 Makefile 參考](docs/command-reference.md)
- [輸出格式](docs/output-formats.md)
- [Rust library 使用說明](docs/rust-library.md)
- [npm 跨平台封裝](docs/npm-distribution.md)
- [CI/CD](docs/ci-cd.md)
- [發布流程](docs/releasing.md)
- [安全與操作](docs/security-and-operations.md)
- [疑難排解](docs/troubleshooting.md)
- [實作架構](docs/api-and-implementation.md)
- [官方參考資料](docs/references.md)

* * *

## 授權

本專案採用 [MIT License](LICENSE)。

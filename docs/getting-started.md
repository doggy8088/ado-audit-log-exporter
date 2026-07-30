# 快速開始

本頁從安裝、設定憑證到匯出第一份 Azure DevOps 稽核記錄。

* * *

## 一、確認 Azure DevOps 條件

使用前確認：

- 組織是 Azure DevOps Services，不是內部部署 Azure DevOps Server
- 組織已連接 Microsoft Entra ID
- Organization settings 的 Auditing 已啟用
- 呼叫身分具有 `View audit log` 權限
- PAT 具有 `vso.auditlog` scope

Azure DevOps 官方文件指出稽核資料保留 90 天。超出保留期限的資料無法由本工具復原。

* * *

## 二、安裝

### 從 npm 安裝

從 npm 安裝：

```sh
npm install --global ado-audit-log-exporter
```

確認執行檔：

```sh
ado-audit-log-exporter --version
```

### 從 Cargo 安裝

發布至 crates.io 後：

```sh
cargo install ado-audit-log-exporter --locked
```

從 GitHub 原始碼安裝：

```sh
git clone https://github.com/doggy8088/ado-audit-log-exporter.git
cd ado-audit-log-exporter
cargo install --path . --locked
```

### 直接建置

```sh
cargo build --release --locked
```

執行檔位於：

- macOS 與 Linux：`target/release/ado-audit-log-exporter`
- Windows：`target/release/ado-audit-log-exporter.exe`

* * *

## 三、設定 PAT

**預設使用 `AZURE_DEVOPS_EXT_PAT`。**

macOS 或 Linux：

```sh
export AZURE_DEVOPS_EXT_PAT='你的 PAT'
```

PowerShell：

```powershell
$env:AZURE_DEVOPS_EXT_PAT = '你的 PAT'
```

不要把 PAT 放在指令參數或可提交檔案中。

* * *

## 四、匯出

CSV：

```sh
ado-audit-log-exporter \
  --organization miniasp \
  --start-time 2026-06-01T00:00:00Z \
  --end-time 2026-07-01T00:00:00Z \
  --format csv \
  --output miniasp-ado-audit-june.csv
```

JSON Lines：

```sh
ado-audit-log-exporter \
  --organization miniasp \
  --format jsonl \
  --output miniasp-ado-audit.jsonl
```

沒有提供時間時，預設查詢最近 30 天。時間必須使用含時區的 RFC 3339，例如：

- `2026-07-01T00:00:00Z`
- `2026-07-01T08:00:00+08:00`

* * *

## 五、使用 Makefile

```sh
make export-csv \
  START_TIME=2026-06-01T00:00:00Z \
  END_TIME=2026-07-01T00:00:00Z \
  OUTPUT=miniasp-ado-audit-june.csv
```

Makefile 預設 `ORGANIZATION=miniasp`，可覆寫：

```sh
make export-jsonl \
  ORGANIZATION=another-org \
  OUTPUT=another-org-audit.jsonl
```

* * *

## 六、確認結果

計算 JSON Lines 筆數：

```sh
wc -l miniasp-ado-audit.jsonl
```

檢查 JSON Lines 前三筆：

```sh
head -n 3 miniasp-ado-audit.jsonl | jq .
```

檢查 CSV 標頭：

```sh
head -n 1 miniasp-ado-audit-june.csv
```

**匯出檔可能含個人識別資訊與管理操作內容，不應提交至 Git。**

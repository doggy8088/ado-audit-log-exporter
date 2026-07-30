# 快速開始

本頁從環境準備開始，說明如何完成第一次稽核記錄匯出。

* * *

## 系統需求

必要項目：

- Python 3.9 以上。
- `make`，僅使用 Python 腳本時可省略。
- 可連線至 `https://auditservice.dev.azure.com`。
- `miniasp` 組織或指定組織的 Azure DevOps 身分。
- `View audit log` 組織層級權限。
- 下列其中一種驗證資料：
  - Microsoft Entra 存取權杖。
  - 具有 `Read Audit Log` 範圍的 Azure DevOps PAT。

選用項目：

- Azure CLI，用於 `make export-entra`。
- Azure DevOps CLI extension；本工具本身不需要它，但 `AZURE_DEVOPS_EXT_PAT` 是 Azure DevOps CLI 採用的標準環境變數。

檢查本機工具：

```sh
python3 --version
make --version
```

**工具只使用 Python 標準函式庫，不需要安裝第三方 Python 套件。**

* * *

## 取得程式碼後的第一次檢查

在專案根目錄執行：

```sh
make check
```

這個目標會：

1. 編譯檢查 `export_ado_audit_logs.py` 與測試檔。
2. 執行所有不需連線至 Azure DevOps 的單元測試。

查看可用目標：

```sh
make help
```

* * *

## 方法一：使用 PAT

在 Azure DevOps 建立僅限目標組織、期限短且只包含 `Read Audit Log` 的 PAT。Microsoft 官方建議 PAT 僅用於個人腳本或臨時工作；可使用 Microsoft Entra 時，應優先採用短效存取權杖。

將 PAT 放入目前 shell 的環境變數：

```sh
export AZURE_DEVOPS_EXT_PAT='replace-with-your-pat'
```

匯出預設組織最近 90 天的 JSON Lines：

```sh
make export
```

預設輸出檔為：

```text
ado-audit.jsonl
```

完成工作後，可從目前 shell 移除 PAT：

```sh
unset AZURE_DEVOPS_EXT_PAT
```

PAT 使用方式與安全規範請參閱：

- [驗證與權限](authentication.md)
- [Microsoft：使用個人存取權杖](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops)

* * *

## 方法二：使用 Microsoft Entra 存取權杖

先登入 Azure CLI：

```sh
az login
```

若登入帳號可存取多個租用戶或訂閱，先切換到與 Azure DevOps 組織連線的 Microsoft Entra 租用戶與適當訂閱。

直接由 Makefile 取得短效權杖並匯出：

```sh
make export-entra
```

此流程會：

1. 使用 Azure DevOps 資源識別碼 `499b84ac-1321-427f-aa17-267ca6975798` 取得存取權杖。
2. 清除該子程序中的 PAT 環境變數。
3. 只在遞迴執行的 `make export` 程序中設定 `ADO_ACCESS_TOKEN`。
4. 不將權杖寫入專案檔案。

也可手動取得權杖：

```sh
export ADO_ACCESS_TOKEN="$(
  az account get-access-token \
    --resource 499b84ac-1321-427f-aa17-267ca6975798 \
    --query accessToken \
    --output tsv
)"
make export
```

Microsoft Entra 操作請參閱 [Microsoft：使用 Azure CLI 發行 Entra 權杖](https://learn.microsoft.com/en-us/azure/devops/cli/entra-tokens?view=azure-devops)。

* * *

## 指定月份

以下命令匯出 2026 年 6 月的 CSV：

```sh
make export-csv \
  START_TIME=2026-06-01T00:00:00Z \
  END_TIME=2026-07-01T00:00:00Z \
  OUTPUT=miniasp-ado-audit-june.csv
```

時間必須符合 RFC 3339，並包含 `Z` 或明確 UTC offset。工具會將時間正規化為 UTC 後送至 Azure DevOps。

若目標檔已存在，工具預設停止。確定要替換時：

```sh
make export-csv \
  START_TIME=2026-06-01T00:00:00Z \
  END_TIME=2026-07-01T00:00:00Z \
  OUTPUT=miniasp-ado-audit-june.csv \
  OVERWRITE=1
```

* * *

## 指定其他組織

使用 Makefile：

```sh
make export ORGANIZATION=contoso OUTPUT=contoso-ado-audit.jsonl
```

直接執行 Python：

```sh
python3 export_ado_audit_logs.py \
  --organization contoso \
  --format jsonl \
  --output contoso-ado-audit.jsonl
```

組織名稱會進行 URL 編碼，再放入下列端點：

```text
https://auditservice.dev.azure.com/{organization}/_apis/audit/auditlog
```

* * *

## 選擇輸出格式

| 需求 | 建議格式 | 命令 |
|---|---|---|
| 大量記錄、串流分析、逐行處理 | JSON Lines | `make export-jsonl` |
| 交換完整 JSON 陣列 | JSON | `make export-json` |
| Excel、試算表、表格篩選 | CSV | `make export-csv` |

完整結構與欄位請參閱 [輸出格式](output-formats.md)。

* * *

## 驗證輸出

檢查檔案是否存在：

```sh
ls -lh ado-audit.jsonl
```

使用 Python 計算 JSON Lines 筆數，不會將內容輸出到終端：

```sh
python3 -c 'print(sum(1 for _ in open("ado-audit.jsonl", encoding="utf-8")))'
```

驗證 CSV 可被解析：

```sh
python3 -c '
import csv
with open("ado-audit.csv", encoding="utf-8", newline="") as stream:
    reader = csv.DictReader(stream)
    print(sum(1 for _ in reader))
'
```

* * *

## 下一步

- 需要完整參數時，閱讀 [命令參考](command-reference.md)。
- 需要理解欄位時，閱讀 [輸出格式](output-formats.md)。
- 需要建立排程或長期保存時，閱讀 [安全與維運](security-and-operations.md)。
- 遇到錯誤時，閱讀 [疑難排解](troubleshooting.md)。

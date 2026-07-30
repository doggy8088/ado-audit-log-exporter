# 驗證與權限

工具只從環境變數讀取憑證，不接受命令列 token 參數。

* * *

## 支援的憑證

| 環境變數 | 類型 | HTTP 驗證 |
|---|---|---|
| `AZURE_DEVOPS_EXT_PAT` | PAT | Basic |
| `ADO_PAT` | PAT | Basic |
| `ADO_ACCESS_TOKEN` | Microsoft Entra access token | Bearer |

PAT 優先順序為 `AZURE_DEVOPS_EXT_PAT`、`ADO_PAT`。**若同時設定 PAT 與 `ADO_ACCESS_TOKEN`，工具會拒絕執行。**

### PAT

```sh
export AZURE_DEVOPS_EXT_PAT='你的 PAT'
ado-audit-log-exporter --organization miniasp
```

PAT 必須包含 `vso.auditlog` scope。Microsoft 的 REST API 文件將此 scope 定義為讀取稽核記錄與 audit streams 的權限。

### Microsoft Entra access token

可讓 Makefile 透過已登入的 Azure CLI 取得短效 token：

```sh
az login
make export-entra \
  ORGANIZATION=miniasp \
  FORMAT=jsonl \
  OUTPUT=miniasp-audit.jsonl
```

Makefile 使用 Azure DevOps resource ID：

```text
499b84ac-1321-427f-aa17-267ca6975798
```

等效的手動操作：

```sh
export ADO_ACCESS_TOKEN="$(
  az account get-access-token \
    --resource 499b84ac-1321-427f-aa17-267ca6975798 \
    --query accessToken \
    --output tsv
)"
ado-audit-log-exporter --organization miniasp
```

Microsoft 建議能使用 Microsoft Entra token 時，優先採用它而不是長效 PAT。

* * *

## Azure DevOps 權限

REST API 驗證成功不代表一定有稽核記錄讀取權限。呼叫身分還需要：

- `View audit log` 設為 Allow，或
- 屬於 Project Collection Administrators

若組織啟用了限制使用者只能看特定專案的預覽功能，Project-Scoped Users 無法存取 Auditing。

* * *

## 安全處理

- 不要把 token 寫入命令列；shell 歷程與程序清單可能暴露參數
- 不要把 token 寫入 `.env` 後提交
- 不要在 CI 使用 echo 印出 token
- 不要把 token 放入 GitHub Actions YAML
- PAT 應採最小 scope、最短有效期限，並定期輪替
- access token 用完後可執行 `unset ADO_ACCESS_TOKEN`
- PAT 用完後可執行 `unset AZURE_DEVOPS_EXT_PAT`

Rust `Authentication` 的 `Debug` 實作會顯示 `[REDACTED]`，不顯示實際 token。HTTP client 也不會在自訂 `Debug` 中列出驗證標頭。

* * *

## 常見狀態碼

| 狀態碼 | 意義 | 檢查方向 |
|---|---|---|
| `401` | 驗證失敗 | token 是否過期、PAT scope 是否包含 `vso.auditlog` |
| `403` | 已驗證但未授權 | `View audit log` 是否為 Allow |
| `429` | 要求過多 | 工具會依 `Retry-After` 或指數退避重試 |

* * *

## 官方資料

- [Audit Log Query REST API](https://learn.microsoft.com/en-us/rest/api/azure/devops/audit/audit-log/query?view=azure-devops-rest-7.1)
- [存取、匯出與篩選 Azure DevOps 稽核記錄](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/azure-devops-auditing?view=azure-devops)
- [Azure DevOps REST API 驗證指引](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/authentication-guidance?view=azure-devops)

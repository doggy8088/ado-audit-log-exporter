# 驗證與權限

工具支援 Microsoft Entra Bearer 權杖與 Azure DevOps PAT。所有機密資料只從環境變數讀取，不接受命令列權杖參數。

**Microsoft 官方建議新整合與長期自動化優先使用 Microsoft Entra，PAT 僅用於個人腳本、臨時工作或缺少適當替代方案的情境。**

* * *

## Azure DevOps 權限

驗證成功不代表具備讀取稽核記錄的授權。權杖代表的使用者或應用程式仍須具備 Azure DevOps 組織層級權限。

必要權限：

| 類型 | 名稱 |
|---|---|
| Azure DevOps UI | `View audit log` |
| 權限識別 | `AuditLog, Read` |
| OAuth 或 PAT 範圍 | `vso.auditlog`，UI 顯示為 `Read Audit Log` |

Project Collection Administrators 已具備組織層級管理權限。其他使用者或群組需要由管理員將 `View audit log` 設為 Allow。

官方說明：

- [存取 Azure DevOps 稽核記錄](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/azure-devops-auditing?view=azure-devops)
- [Azure DevOps 權限參考](https://learn.microsoft.com/en-us/azure/devops/organizations/security/permissions?view=azure-devops)

* * *

## 環境變數

| 環境變數 | 驗證標頭 | 用途 |
|---|---|---|
| `AZURE_DEVOPS_EXT_PAT` | Basic | 預設 PAT 來源，與 Azure DevOps CLI 慣例一致 |
| `ADO_PAT` | Basic | 舊有相容性備援 |
| `ADO_ACCESS_TOKEN` | Bearer | Microsoft Entra 存取權杖 |

PAT 的選擇順序：

1. `AZURE_DEVOPS_EXT_PAT`
2. `ADO_PAT`

若兩個 PAT 變數都存在，工具使用 `AZURE_DEVOPS_EXT_PAT`。若 PAT 與 `ADO_ACCESS_TOKEN` 同時存在，工具會停止並回報驗證類型衝突，避免無法判斷實際使用的身分。

檢查變數是否存在而不顯示內容：

```sh
if [ -n "${AZURE_DEVOPS_EXT_PAT:-}" ]; then
  printf '%s\n' 'AZURE_DEVOPS_EXT_PAT is set'
fi
```

不要執行會印出完整環境的命令，也不要將權杖寫入 `.env` 後提交。

* * *

## PAT 驗證

建立 PAT 時：

1. 將組織限制在實際匯出的 Azure DevOps 組織。
2. 選擇最短可行期限。
3. 只授予 `Read Audit Log`。
4. 將權杖保存在安全的密碼或祕密管理系統。
5. 定期輪替並在不再使用時撤銷。

設定目前 shell：

```sh
export AZURE_DEVOPS_EXT_PAT='replace-with-your-pat'
```

工具會以空白使用者名稱與 PAT 建立 HTTP Basic 驗證值：

```text
Authorization: Basic base64(":PAT")
```

工具不會記錄完整標頭，也不會將 PAT 寫入輸出。

Microsoft 官方資料：

- [使用個人存取權杖](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops)
- [以 `AZURE_DEVOPS_EXT_PAT` 登入 Azure DevOps CLI](https://learn.microsoft.com/en-us/azure/devops/cli/log-in-via-pat?view=azure-devops)

* * *

## Microsoft Entra 驗證

臨時人工匯出可使用 Azure CLI 取得短效存取權杖：

```sh
az login
az account set --subscription '<subscription-id>'
```

取得 Azure DevOps 資源的權杖：

```sh
export ADO_ACCESS_TOKEN="$(
  az account get-access-token \
    --resource 499b84ac-1321-427f-aa17-267ca6975798 \
    --query accessToken \
    --output tsv
)"
```

執行匯出：

```sh
make export
```

或由 Makefile 在單一子程序內完成：

```sh
make export-entra
```

Azure DevOps 資源識別碼固定為：

```text
499b84ac-1321-427f-aa17-267ca6975798
```

權杖應視為 opaque 值。工具只將它放入 Bearer 標頭，不解碼或依賴權杖內部 claim。

官方資料：

- [Azure DevOps 驗證方式指引](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/authentication-guidance?view=azure-devops)
- [使用 Microsoft Entra ID 驗證 Azure DevOps](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/entra?view=azure-devops)
- [使用 Azure CLI 發行 Entra 權杖](https://learn.microsoft.com/en-us/azure/devops/cli/entra-tokens?view=azure-devops)

* * *

## 自動化身分

背景服務、排程工作與 CI/CD 不應長期依賴個人 PAT。可依執行環境選擇：

- Microsoft Entra service principal。
- Azure managed identity。
- Azure DevOps service connection 與 workload identity federation。
- 可安全輪替的短效使用者委派權杖。

本工具只要求最終權杖出現在 `ADO_ACCESS_TOKEN`，不負責取得或重新整理服務身分權杖。正式自動化需要在執行器或祕密管理系統中完成權杖生命週期管理。

* * *

## 常見驗證失敗

| 現象 | 判斷 |
|---|---|
| `missing credentials` | 三個支援的環境變數都沒有值 |
| `set only one credential type` | PAT 與 `ADO_ACCESS_TOKEN` 同時存在 |
| HTTP 401 | 權杖無效、過期、租用戶不符或 PAT 缺少必要範圍 |
| HTTP 403 | 身分已驗證，但沒有 `View audit log` 或組織存取權 |

完整處理方式請參閱 [疑難排解](troubleshooting.md)。

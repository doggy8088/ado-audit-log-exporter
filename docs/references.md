# 參考資料

本頁彙整工具設計與文件採用的原始資料。連結於 2026-07-30 查核。

**Azure DevOps Audit API 仍標示為 preview，使用者應定期重新查閱 API 版本、欄位與限制。**

* * *

## 稽核 REST API

### Audit Log Query

- 連結：[Audit Log - Query - REST API](https://learn.microsoft.com/en-us/rest/api/azure/devops/audit/audit-log/query?view=azure-devops-rest-7.1)
- 用途：
  - 端點 `auditservice.dev.azure.com`。
  - API 版本 `7.1-preview.1`。
  - `startTime`、`endTime`、`batchSize`、`continuationToken`、`skipAggregation`。
  - `AuditLogQueryResult` 與 `DecoratedAuditLogEntry` 欄位。
  - `vso.auditlog` 範圍。

### Microsoft Azure DevOps Go API

- 連結：[AuditLogQueryResult 型別](https://pkg.go.dev/github.com/microsoft/azure-devops-go-api/azuredevops/audit#AuditLogQueryResult)
- 用途：
  - 佐證官方 SDK 將查詢結果建模為直接的 `AuditLogQueryResult`。
  - 對照 `decoratedAuditLogEntries`、`continuationToken` 與 `hasMore`。

* * *

## Azure DevOps 稽核功能

### 存取、匯出與篩選稽核記錄

- 連結：[Access Azure DevOps Audit Logs, Export, and Filter](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/azure-devops-auditing?view=azure-devops)
- 用途：
  - `View audit log` 權限。
  - 90 天事件保留。
  - CSV 與 JSON 匯出。
  - Auditing 限制。
  - Microsoft Entra 登入事件不在 Azure DevOps 稽核記錄內。

### 稽核事件清單

- 連結：[Auditing events list](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/auditing-events?view=azure-devops)
- 用途：
  - 可產生的 `actionId` 與產品區域。
  - 事件種類會持續新增。

### Audit Streaming

- 連結：[Create audit streaming for Azure DevOps](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/auditing-streaming?view=azure-devops)
- 用途：
  - Splunk、Event Grid 與 Azure Monitor Logs。
  - 長期持續收集。
  - stream 權限與操作限制。

### 權限參考

- 連結：[Permissions, security groups, and service accounts reference](https://learn.microsoft.com/en-us/azure/devops/organizations/security/permissions?view=azure-devops)
- 用途：
  - 組織層級權限。
  - `AuditLog, Read`。
  - Project Collection Administrators。

* * *

## 驗證

### 驗證方式指引

- 連結：[Authentication methods for Azure DevOps](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/authentication-guidance?view=azure-devops)
- 用途：
  - 新應用程式優先使用 Microsoft Entra。
  - PAT 僅用於有限情境。
  - service principal 與 managed identity 選擇。
  - 權杖應視為 opaque。

### Microsoft Entra ID

- 連結：[Authenticate to Azure DevOps with Microsoft Entra ID](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/entra?view=azure-devops)
- 用途：
  - 使用者委派與應用程式身分。
  - Entra 與 PAT 的安全性差異。

### Azure CLI 發行 Entra 權杖

- 連結：[Issue Entra tokens with Azure CLI](https://learn.microsoft.com/en-us/azure/devops/cli/entra-tokens?view=azure-devops)
- 用途：
  - Azure DevOps 資源識別碼 `499b84ac-1321-427f-aa17-267ca6975798`。
  - `az account get-access-token`。

### 個人存取權杖

- 連結：[Use personal access tokens](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops)
- 用途：
  - 建立、使用、輪替與撤銷 PAT。
  - `Read Audit Log` 範圍。
  - Basic 驗證格式。
  - PAT 安全實務。

### `AZURE_DEVOPS_EXT_PAT`

- 連結：[Sign in with a personal access token, Azure DevOps CLI](https://learn.microsoft.com/en-us/azure/devops/cli/log-in-via-pat?view=azure-devops)
- 用途：
  - Azure DevOps CLI 對 `AZURE_DEVOPS_EXT_PAT` 的標準支援。
  - 非互動式環境設定方式。

* * *

## 通用 REST API

### Azure DevOps REST API 入門

- 連結：[Get started with the REST APIs for Azure DevOps](https://learn.microsoft.com/en-us/azure/devops/integrate/how-to/call-rest-api?view=azure-devops)
- 用途：
  - Azure DevOps REST API 的 HTTP 方法、驗證與 JSON 回應慣例。

* * *

## 時間格式

### RFC 3339

- 連結：[RFC 3339: Date and Time on the Internet](https://www.rfc-editor.org/rfc/rfc3339)
- 用途：
  - `--start-time` 與 `--end-time` 的日期時間格式。
  - UTC `Z` 與數值 offset。

* * *

## 查核原則

- 以 Microsoft Learn 與 Microsoft 官方 SDK 為主要來源。
- 對 preview API，不將範例回應包裝視為唯一結構。
- 文件敘述與實際工具行為衝突時，以程式與測試為準，並建立議題修正文件。
- 若官方更新 API 版本，應先增加相容性測試，再修改 `API_VERSION`。

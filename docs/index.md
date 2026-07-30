# 文件中心

本文件集說明如何安全地使用 `ado-audit-log-exporter`，透過 Azure DevOps Audit REST API 匯出組織層級的稽核記錄。工具不使用瀏覽器自動化，也不讀取 Azure DevOps 網頁畫面。

**目前預設組織是 `miniasp`，但所有匯出流程都可指定其他 Azure DevOps Services 組織。**

* * *

## 文件導覽

| 文件 | 內容 |
|---|---|
| [快速開始](getting-started.md) | 系統需求、第一次匯出、常見匯出範例 |
| [驗證與權限](authentication.md) | Microsoft Entra、PAT、環境變數優先順序、Azure DevOps 權限 |
| [命令參考](command-reference.md) | Python 命令列選項、Makefile 目標與變數 |
| [輸出格式](output-formats.md) | JSON、JSON Lines、CSV、欄位定義與資料相容性 |
| [API 與實作](api-and-implementation.md) | REST 端點、查詢參數、分頁、重試、回應結構 |
| [疑難排解](troubleshooting.md) | 常見錯誤、診斷流程與處理方式 |
| [安全與維運](security-and-operations.md) | 權杖、檔案保護、90 天保留限制、排程建議 |
| [參考資料](references.md) | Microsoft 與標準規格的原始參考連結 |

* * *

## 工具範圍

工具負責：

- 呼叫 Azure DevOps Audit Log Query REST API。
- 依 `continuationToken` 取得所有分頁。
- 將稽核事件輸出成 JSON、JSON Lines 或 CSV。
- 在可重試的網路或服務錯誤下重試。
- 以暫存檔完成檔案型輸出，成功後再原子替換目標檔。
- 保留 API 未知的新欄位，降低服務結構演進造成的資料遺失。

工具不負責：

- 建立、輪替或撤銷 PAT。
- 修改 Azure DevOps 的稽核權限。
- 建立 Audit Streaming。
- 查詢 Microsoft Entra 登入記錄。
- 將資料自動上傳至 SIEM、物件儲存或資料庫。
- 解密、解析或保存驗證權杖內容。

* * *

## 支援界線

| 項目 | 支援狀態 |
|---|---|
| Azure DevOps Services | 支援 |
| Azure DevOps Server 內部部署版本 | 未宣告支援 |
| Python | 3.9 以上 |
| 作業系統 | 可執行 Python 3.9 與 GNU Make 或相容 Make 的環境 |
| API 版本 | `7.1-preview.1` |
| 預設輸出格式 | JSON Lines |
| 預設查詢期間 | 執行時刻往前 90 天 |

Azure DevOps 官方指出，Auditing 僅適用於以 Microsoft Entra ID 支援的 Azure DevOps Services 組織，且事件保留 90 天。詳細限制請參閱 [安全與維運](security-and-operations.md)。

* * *

## 最短執行路徑

已經有具備適當範圍與權限的 PAT 時：

```sh
export AZURE_DEVOPS_EXT_PAT='replace-with-your-pat'
make export
```

已經登入 Azure CLI 時：

```sh
make export-entra
```

**任何稽核輸出都可能包含使用者識別資訊、IP 位址、User-Agent 與資源名稱，應依敏感紀錄處理。**

* * *

## 官方入口

- [Audit Log Query REST API](https://learn.microsoft.com/en-us/rest/api/azure/devops/audit/audit-log/query?view=azure-devops-rest-7.1)
- [存取、匯出與篩選 Azure DevOps 稽核記錄](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/azure-devops-auditing?view=azure-devops)
- [Azure DevOps 驗證方式指引](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/authentication-guidance?view=azure-devops)

# 安全與維運

稽核資料本身是安全敏感資料。即使工具不將驗證權杖寫入輸出，事件仍可能揭露組織架構、身分、IP 位址、權限變更與資源名稱。

**應同時保護權杖與匯出檔，並為保存、存取、輪替、刪除及事件去重建立明確政策。**

* * *

## Azure DevOps 稽核限制

Microsoft 官方列出的重要限制：

- Auditing 僅適用於以 Microsoft Entra ID 支援的 Azure DevOps Services 組織。
- 稽核事件保存 90 天，之後刪除。
- Azure DevOps 不記錄 Microsoft Entra 登入事件。
- 透過 Microsoft Entra 群組管理的群組內部成員異動，不一定出現在 Azure DevOps `Groups` 區域事件中；應查閱 Microsoft Entra 稽核記錄。
- 間接加入組織的使用者可能顯示為由 Azure DevOps Services 加入，且不一定有直接觸發事件可供關聯。
- Microsoft 持續新增可稽核事件，事件種類不是固定集合。

原始資料：

- [Azure DevOps 稽核記錄與限制](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/azure-devops-auditing?view=azure-devops)
- [Azure DevOps 稽核事件清單](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/auditing-events?view=azure-devops)

* * *

## 權杖安全

### Microsoft Entra

- 優先使用短效權杖。
- 正式背景工作使用 service principal、managed identity 或 workload identity federation。
- 將權杖視為 opaque 值，不解碼 claim。
- 不將權杖放入命令列參數、URL、Git、終端輸出或應用程式記錄。

### PAT

- 僅授予 `Read Audit Log`。
- 限制於單一組織。
- 採用短期限並定期輪替。
- 不同自動化流程使用不同 PAT。
- 使用祕密管理系統，不放在原始碼或 `.env` 提交內容。
- 疑似外洩時立即撤銷。

Microsoft 建議可使用 Entra 時不要長期使用 PAT：

- [Azure DevOps 驗證方式指引](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/authentication-guidance?view=azure-devops)
- [使用個人存取權杖](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops)

* * *

## 本機檔案保護

工具的檔案輸出使用安全暫存檔。在 POSIX 系統，實際產生的檔案通常只有擁有者可讀寫。仍應驗證：

```sh
stat -f '%Sp %N' ado-audit.jsonl
```

Linux 可使用：

```sh
stat -c '%A %n' ado-audit.jsonl
```

建議：

- 將輸出存放於加密磁碟或受控目錄。
- 限制群組與其他使用者權限。
- 傳輸時使用 TLS。
- 備份與封存使用伺服器端加密。
- 不透過一般聊天、公開議題或未受控電子郵件傳送。
- 不提交至 Git。

專案 `.gitignore` 已忽略名稱包含 `ado-audit` 的 JSON、JSON Lines 與 CSV，但忽略規則不是資料外洩防護的替代品。提交前仍須執行：

```sh
git status --short --ignored
git diff --cached --name-only
```

* * *

## 保存期限

Azure DevOps 來源只保留 90 天。若法規、鑑識或內部政策需要更久：

1. 建立固定排程匯出。
2. 將輸出移至受控長期儲存。
3. 記錄查詢時間窗、組織、工具 Git commit 與筆數。
4. 為檔案計算密碼學雜湊。
5. 依資料分類設定保存與刪除期限。
6. 定期測試還原與解析。

計算 SHA-256：

```sh
shasum -a 256 miniasp-ado-audit-june.csv \
  > miniasp-ado-audit-june.csv.sha256
```

雜湊可協助驗證檔案未被修改，但不能證明來源事件在匯出前完整。

* * *

## 排程策略

推薦使用明確 UTC 時間窗，例如每天匯出前一個完整 UTC 日：

```text
startTime = 2026-06-01T00:00:00Z
endTime   = 2026-06-02T00:00:00Z
```

注意事項：

- REST API 文件沒有在參數說明中明確保證界線排除規則。
- 相鄰時間窗可能在邊界產生重複。
- 下游應以事件 `id` 去重。
- 不要只依檔名判斷時間範圍。
- 匯出成功後才更新排程 checkpoint。
- 失敗時保留上一個成功 checkpoint，再重跑相同時間窗。
- 避免多個工作同時匯出相同組織與時間範圍。

每月人工匯出範例：

```sh
make export-csv \
  START_TIME=2026-06-01T00:00:00Z \
  END_TIME=2026-07-01T00:00:00Z \
  OUTPUT=miniasp-ado-audit-june.csv
```

* * *

## 完整性紀錄

每次正式匯出建議另外保存：

| 欄位 | 用途 |
|---|---|
| 組織名稱 | 界定資料來源 |
| 開始與結束時間 | 重現查詢 |
| 輸出格式 | 選擇正確解析器 |
| 事件筆數 | 快速完整性檢查 |
| 分頁數 | 診斷服務行為 |
| 工具 Git commit | 重現程式版本 |
| API 版本 | 追蹤 preview API 變更 |
| SHA-256 | 驗證檔案後續未被修改 |
| 執行時間與執行身分 | 維運稽核 |

不要將權杖、Authorization 標頭或完整 API 回應加入這份中繼資料。

* * *

## Audit Streaming

需要持續輸送或超過 90 天保留時，Microsoft 建議評估 Audit Streaming。Azure DevOps 支援將事件串流至：

- Splunk。
- Azure Event Grid。
- Azure Monitor Logs。

Audit Streaming 與本工具的差異：

| 項目 | 本工具 | Audit Streaming |
|---|---|---|
| 模式 | 主動查詢 | 服務推送 |
| 適合 | 臨時匯出、補抓、人工分析 | 長期持續收集、SIEM |
| 90 天後資料 | 需先匯出保存 | 由目標系統保存政策決定 |
| 設定權限 | `View audit log` | 另需管理 stream 權限 |
| 失敗處理 | 執行端重試 | 依 stream 狀態與目標服務 |

官方說明：[建立 Azure DevOps Audit Streaming](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/auditing-streaming?view=azure-devops)

* * *

## 事件分析注意事項

- `correlationId` 可關聯單一操作引發的多個事件。
- `id` 與 `correlationId` 的關係可協助判斷原始事件與衍生事件，但不是所有事件都有相關事件。
- `details` 是顯示文字，不應作為唯一穩定分析欄位。
- `actionId`、`area`、`category` 與結構化 `data` 更適合規則化分析。
- `data` 依事件種類變化，下游 schema 應允許未知欄位。
- `actorDisplayName` 可能變更，關聯身分時應搭配穩定識別碼。
- IP 位址與 User-Agent 可能為空值。

事件清單會持續演進，規則應採 allow unknown，而不是遇到新 `actionId` 就拒絕整批資料。

* * *

## 事件刪除

刪除匯出檔前確認：

1. 保存期限已到。
2. 沒有法律保全、資安事件或調查需求。
3. 備份與衍生資料也納入刪除範圍。
4. 雜湊與中繼資料的保存政策一致。
5. 刪除方式符合儲存媒體與組織政策。

Git 忽略檔案仍存在於工作目錄，`git clean` 或刪除專案目錄可能移除它。不要將 Git 儲存庫當成稽核資料保存系統。

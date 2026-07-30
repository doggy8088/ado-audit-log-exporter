# 疑難排解

先執行本機檢查：

```sh
make check
```

再確認權杖是否存在，但不要顯示權杖內容：

```sh
if [ -n "${AZURE_DEVOPS_EXT_PAT:-}" ]; then
  printf '%s\n' 'AZURE_DEVOPS_EXT_PAT is set'
fi
if [ -n "${ADO_ACCESS_TOKEN:-}" ]; then
  printf '%s\n' 'ADO_ACCESS_TOKEN is set'
fi
```

* * *

## 快速索引

| 錯誤或現象 | 主要原因 |
|---|---|
| `missing credentials` | 沒有支援的權杖環境變數 |
| `set only one credential type` | PAT 與 Bearer 權杖同時存在 |
| HTTP 401 | 權杖無效、過期、範圍錯誤或租用戶不符 |
| HTTP 403 | 缺少 Azure DevOps `View audit log` 權限 |
| HTTP 429 | 服務節流，工具會依策略重試 |
| HTTP 5xx | Azure DevOps 暫時性服務錯誤 |
| `could not reach Azure DevOps` | DNS、TLS、Proxy、防火牆或網路問題 |
| `invalid RFC 3339 timestamp` | 日期格式錯誤 |
| `timestamp must include Z or a UTC offset` | 日期沒有時區 |
| `start-time must not be later` | 開始時間晚於結束時間 |
| `output file already exists` | 目標存在且未允許覆寫 |
| `unexpected API response shape` | 服務回應不是已知 AuditLogQueryResult 結構 |
| continuation token 錯誤 | 服務宣告還有資料但 token 缺少或重複 |

* * *

## 缺少驗證資料

錯誤：

```text
missing credentials; set AZURE_DEVOPS_EXT_PAT, ADO_ACCESS_TOKEN, or ADO_PAT
```

PAT：

```sh
export AZURE_DEVOPS_EXT_PAT='replace-with-your-pat'
make export
```

Microsoft Entra：

```sh
make export-entra
```

* * *

## 驗證類型衝突

錯誤：

```text
set only one credential type: ADO_ACCESS_TOKEN or a PAT, not both
```

工具不允許 PAT 與 Microsoft Entra 權杖同時存在。

要使用 PAT：

```sh
unset ADO_ACCESS_TOKEN
make export
```

要使用 Entra：

```sh
unset AZURE_DEVOPS_EXT_PAT ADO_PAT
make export
```

`make export-entra` 會在其子程序中自動清除 PAT 變數。

* * *

## HTTP 401

可能原因：

- PAT 已過期、撤銷或輸入錯誤。
- PAT 沒有 `Read Audit Log` 範圍。
- Entra 權杖不是針對 Azure DevOps 資源。
- Azure CLI 使用錯誤的 Microsoft Entra 租用戶。
- 組織限制了 PAT 使用。
- 權杖環境變數含有多餘換行或引號。

處理順序：

1. 確認權杖仍有效，但不要輸出權杖。
2. 確認 PAT 的組織與範圍。
3. 使用 Azure CLI 時執行 `az account show` 確認租用戶。
4. 重新取得 Entra 權杖。
5. 再次執行匯出。

取得正確 Azure DevOps 資源權杖：

```sh
az account get-access-token \
  --resource 499b84ac-1321-427f-aa17-267ca6975798 \
  --query accessToken \
  --output tsv
```

* * *

## HTTP 403

HTTP 403 通常表示權杖已驗證，但身分沒有讀取稽核記錄的權限。

管理員需確認：

- 使用者或應用程式身分已加入目標 Azure DevOps 組織。
- `View audit log` 設為 Allow。
- 沒有更高優先級的 Deny。
- Project-Scoped Users 限制沒有阻止存取 Auditing。
- Auditing 對組織可用且已啟用。

官方指引：[存取 Azure DevOps 稽核記錄](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/azure-devops-auditing?view=azure-devops)

* * *

## HTTP 429 或 5xx

工具預設自動重試四次。可增加重試次數：

```sh
make export RETRIES=8 TIMEOUT=60
```

HTTP 429 若含數字型 `Retry-After`，工具依服務指定秒數等待。沒有該標頭時使用指數退避。

若持續收到 429：

- 降低執行頻率。
- 避免同時啟動多個完整匯出。
- 保持分頁順序。
- 將大範圍拆成不重疊的小時間窗。

若持續收到 5xx，先保留錯誤時間與狀態碼，再查閱 Azure DevOps 服務狀態。

* * *

## DNS 或網路錯誤

錯誤可能包含：

```text
could not reach Azure DevOps
Name or service not known
Temporary failure in name resolution
nodename nor servname provided
```

先檢查：

```sh
nslookup auditservice.dev.azure.com
curl -I https://auditservice.dev.azure.com
```

macOS 可優先清除 DNS 快取：

```sh
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

仍失敗時檢查：

- 公司 Proxy 或 VPN。
- 防火牆 allowlist。
- TLS 攔截憑證。
- `HTTPS_PROXY` 與 `NO_PROXY`。
- Azure DevOps 官方網路範圍與服務狀態。

* * *

## 日期與時間錯誤

錯誤：

```text
invalid RFC 3339 timestamp
```

使用：

```text
2026-06-01T00:00:00Z
2026-06-01T08:00:00+08:00
```

不要使用：

```text
2026-06-01
2026-06-01 00:00:00
2026-06-01T00:00:00
```

錯誤：

```text
--start-time must not be later than --end-time
```

交換或更正時間界線。

Azure DevOps 稽核資料只保留 90 天。要求更早時間即使格式有效，也不會恢復已刪除的事件。

* * *

## 輸出檔已存在

錯誤：

```text
output file already exists
```

選擇新檔名：

```sh
make export-csv OUTPUT=miniasp-ado-audit-june-v2.csv
```

確定要取代時：

```sh
make export-csv OUTPUT=miniasp-ado-audit-june.csv OVERWRITE=1
```

覆寫使用暫存檔與原子替換，不會先清空既有檔案。

* * *

## 非預期 API 回應

錯誤：

```text
unexpected API response shape
```

工具已支援：

- 最上層 `AuditLogQueryResult`。
- `value` 包裝的 `AuditLogQueryResult`。

錯誤訊息只列出頂層鍵與 `value` 類型，避免將可能敏感的 API 回應印到終端。

處理方式：

1. 記錄完整錯誤文字。
2. 確認 API 版本仍為 `7.1-preview.1`。
3. 執行 `make check`。
4. 查閱 Microsoft 的 Audit Log Query API 是否更新。
5. 不要將完整稽核回應貼到公開議題。

* * *

## 分頁錯誤

錯誤：

```text
hasMore is true but no continuationToken was returned
```

或：

```text
Azure DevOps repeated a continuation token
```

這表示服務分頁狀態不一致。工具會停止，避免輸出表面成功但實際不完整的資料，或進入無限迴圈。

可採取：

- 稍後重新執行。
- 縮小時間範圍。
- 保留錯誤時間與要求參數。
- 若可重現，向 Microsoft 支援提供不含權杖與事件內容的診斷資訊。

* * *

## 匯出筆數與入口網站不同

可能原因：

- 工具預設 `skipAggregation=true`，入口網站可能顯示聚合的 `AuditLog.AccessLog`。
- 時間範圍或時區不同。
- 入口網站只先載入部分結果。
- 匯出期間有新事件產生。
- 查詢時間窗重疊。

要允許存取事件聚合：

```sh
make export AGGREGATE_ACCESS_LOG=1
```

比較時應使用完全相同的 UTC 開始與結束時間。

* * *

## Make 顯示狀態 2

範例：

```text
make: *** [export] Error 1
```

Make 最終可能回傳狀態 2，但真正原因是前一行 Python 或配方錯誤。診斷時先閱讀 `make:` 之前的錯誤訊息。

* * *

## 取得最小安全診斷資料

可安全收集：

- 工具版本的 Git commit。
- Python 版本。
- 作業系統。
- 命令列選項，但移除權杖。
- HTTP 狀態碼。
- 錯誤訊息。
- 查詢開始與結束時間。
- 回應頂層鍵名。

不可公開：

- PAT。
- Bearer 權杖。
- 完整 `Authorization` 標頭。
- 完整稽核事件。
- 使用者 UPN、IP 位址或內部資源名稱。

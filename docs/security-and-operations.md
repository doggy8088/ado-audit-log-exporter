# 安全與操作

Azure DevOps 稽核記錄可能包含使用者識別、IP、User-Agent、專案或 repository 名稱、權限變更與其他管理活動。**即使工具原始碼可公開，匯出資料仍應視為敏感資料。**

* * *

## 憑證邊界

工具採取以下限制：

- token 只從環境變數讀取
- CLI 不提供 `--pat` 或 `--token`
- `Authentication` 的 `Debug` 顯示遮罩
- `AuditClient` 的 `Debug` 不顯示 HTTP headers
- 錯誤訊息不重現 request Authorization header

呼叫端責任：

- 使用最小權限與最短效期
- 避免 shell trace，例如含敏感環境時不要開啟 `set -x`
- 不在 CI log 印出環境變數
- 不把本機 `.env`、shell profile 或 credentials 提交
- 人員離職、裝置遺失或疑似外洩時立即撤銷 PAT

* * *

## 匯出檔

建議：

- 存放於存取受控且有靜態加密的空間
- 規範保存期限與刪除程序
- 只授權安全、稽核或法遵必要人員
- 傳輸時使用加密通道
- 匯入 SIEM 前確認欄位映射與個資政策
- 分享前依用途遮罩 UPN、IP 與 resource 名稱

`.gitignore` 排除常見 `ado-audit` JSON、JSON Lines、CSV 與 `audit-logs/`，但檔名若不符合模式仍可能被 Git 納管。提交前必須執行：

```sh
git status --short
git diff --cached --name-only
```

* * *

## 原子寫檔

CLI 在輸出檔同目錄使用暫存檔，完成所有頁面後才持久化。這降低以下風險：

- API 中途失敗卻留下看似完整的輸出
- JSON array 缺少結尾
- 程序中斷後覆寫原本檔案

`--overwrite` 是明確選項；預設不覆寫。

* * *

## npm 供應鏈

- npm tarball 以 `files` allowlist 控制內容
- 原生壓縮檔由 GitHub-hosted runner 建置
- 每個壓縮檔附 SHA-256
- postinstall 強制核對 checksum
- npm 發布前確認所有 Release URL
- trusted publishing 使用短效 OIDC
- 公開 package 與公開 repository 讓 npm 自動產生 provenance

SHA-256 可偵測下載損毀或資產與 checksum 不一致，但如果 GitHub Release 壓縮檔與 checksum 同時遭未授權替換，單純 checksum 無法建立獨立信任根。應同時保護 GitHub 帳號、branch、tag 與 Actions 權限。

* * *

## 儲存庫敏感資料檢查

發布前至少執行：

```sh
git status --short --ignored
git grep -n -I -E \
  'AZURE_DEVOPS_EXT_PAT=|ADO_ACCESS_TOKEN=|ADO_PAT=|NPM_TOKEN=|NODE_AUTH_TOKEN='
git ls-files
```

還應使用 GitHub secret scanning 與 push protection。公開儲存庫若發現已提交的 token，刪除檔案不等於撤銷憑證；必須先撤銷或輪替，再處理 Git 歷史。

* * *

## 稽核記錄保留

Microsoft 文件指出 Azure DevOps Audit Logs 保留 90 天。若有更長期法遵需求：

- 排程本工具定期匯出，或
- 使用 Azure DevOps Audit Streaming 將事件送到 SIEM

本工具是批次匯出器，不是 exactly-once streaming consumer。定期批次匯出時可使用重疊時間窗，再以 `id` 去重。

* * *

## 不在範圍內

- 不管理 Azure DevOps Auditing 開關
- 不授予 `View audit log`
- 不建立或輪替 PAT
- 不處理 SIEM ingestion
- 不匿名化事件
- 不保證超過 Azure DevOps 保留期的資料可取得
- 不支援內部部署 Azure DevOps Server

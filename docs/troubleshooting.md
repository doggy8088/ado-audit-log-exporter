# 疑難排解

* * *

## 缺少憑證

訊息：

```text
缺少憑證；請設定 AZURE_DEVOPS_EXT_PAT、ADO_ACCESS_TOKEN 或 ADO_PAT 環境變數
```

確認環境變數存在，但不要印出內容：

```sh
test -n "${AZURE_DEVOPS_EXT_PAT:-}" &&
  echo AZURE_DEVOPS_EXT_PAT_is_set
```

* * *

## PAT 與 access token 衝突

訊息：

```text
不可同時設定 PAT 與 ADO_ACCESS_TOKEN
```

使用 PAT：

```sh
unset ADO_ACCESS_TOKEN
```

使用 Microsoft Entra access token：

```sh
unset AZURE_DEVOPS_EXT_PAT ADO_PAT
```

* * *

## HTTP 401

檢查：

- token 是否過期或撤銷
- PAT 是否包含 `vso.auditlog`
- PAT 是否屬於正確 Azure DevOps 身分
- `ADO_ACCESS_TOKEN` 是否針對 Azure DevOps resource 取得

* * *

## HTTP 403

驗證已成功，但身分沒有 `View audit log`。請由 Project Collection Administrators 在組織安全設定中授予 Allow。

權限變更可能需要時間傳播。Microsoft 疑難排解文件指出 Microsoft Entra group membership 或權限變更可能最多需要一小時。

* * *

## 查詢沒有資料

檢查：

- 組織名稱是否正確
- Auditing 是否已啟用
- 時間是否落在最近 90 天
- `start-time` 與 `end-time` 是否使用預期時區
- 該時段是否實際有活動

空結果會產生合法空檔：

- JSON：空 array
- JSON Lines：空檔
- CSV：只有標頭

* * *

## 回應結構錯誤

工具支援：

- 最外層直接 `decoratedAuditLogEntries`
- `value` object 包裝
- array 型別 `value`
- `auditLogEntries`

若仍出現回應結構錯誤，記錄錯誤訊息、HTTP 狀態與發生時間。不要貼上 PAT，也不要未經遮罩貼出完整稽核事件。

* * *

## 輸出檔已存在

預設防止覆寫。明確允許：

```sh
ado-audit-log-exporter \
  --output audit.jsonl \
  --overwrite
```

Makefile：

```sh
make export OUTPUT=audit.jsonl OVERWRITE=1
```

* * *

## npm unsupported platform

目前只支援：

- macOS ARM64 與 x64
- Linux ARM64 與 x64
- Windows x64

Windows ARM64、Linux ARMv7、FreeBSD 等平台會明確失敗。可改用 Rust toolchain 自行從原始碼建置，但相依套件與目標平台支援仍需另行驗證。

* * *

## npm 下載 404

常見原因：

- npm 版本已發布，但同版本 GitHub Release 不存在
- Release 資產名稱不符
- Release 尚在建置
- package 版本與 tag 不一致

檢查：

```sh
node npm/prepublish-check.cjs
gh release view "v$(node -p "require('./package.json').version")"
```

* * *

## npm checksum mismatch

**不要略過驗證或手動安裝該下載檔。**

可能原因：

- 下載損毀
- Release 資產更新但 checksum 未同步
- proxy 或快取回傳錯誤內容
- GitHub Release 遭未授權修改

確認 Release workflow 與 GitHub 帳號安全後，使用新的 patch 版本重建與發布。

* * *

## npm trusted publishing 失敗

逐項核對：

- npm package 的 repository URL 是否指向公開 GitHub repo
- npm Trusted Publisher user 是否為 `doggy8088`
- repository 是否為 `ado-audit-log-exporter`
- workflow filename 是否只填 `npm-publish.yml`
- allowed action 是否包含 `npm publish`
- repository variable 是否為 `true`
- workflow 是否有 `id-token: write`
- 是否使用 GitHub-hosted runner
- Node 與 npm 版本是否符合 npm 官方要求

npm 儲存 Trusted Publisher 設定時不會主動驗證欄位；錯誤通常到 publish 才會出現。

* * *

## DNS 解析錯誤

macOS 可先清除 DNS 快取：

```sh
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

接著確認：

```sh
nslookup auditservice.dev.azure.com
nslookup github.com
nslookup registry.npmjs.org
```

公司網路若使用 proxy、TLS inspection 或 allowlist，需確認 Azure DevOps、GitHub Releases、npm registry 都可連線。

# 初版與後續發布

**本文件的初版流程由維護者親自執行；自動化只負責 tag 後的跨平台建置與 GitHub Release。**

目前預定初版為 `0.1.0`。

* * *

## 一、發布前確認

在 `main` 最新提交執行：

```sh
git status --short --branch
make check
cargo package --list
npm pack --dry-run
```

確認版本完全一致：

```sh
node -p "require('./package.json').version"
cargo metadata --no-deps --format-version 1 |
  jq -r '.packages[0].version'
```

兩者都必須是：

```text
0.1.0
```

再次確認 npm 名稱仍可使用：

```sh
npm view ado-audit-log-exporter name
```

`E404` 表示當下尚不存在；名稱可用性可能隨時改變，實際以發布當下 npm 回應為準。

* * *

## 二、建立初版 GitHub Release

建立 annotated tag：

```sh
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

GitHub Actions 的 Release workflow 會：

1. 核對三處版本
2. 建置五個平台
3. 建立五個壓縮檔
4. 建立五個 SHA-256 檔
5. 建立公開 GitHub Release

查看進度：

```sh
gh run list --workflow release.yml --limit 5
gh run watch
```

確認 Release：

```sh
gh release view v0.1.0
node npm/prepublish-check.cjs
```

**十個 Release 資產全部可讀取後，才能發布 npm。**

* * *

## 三、人工發布初版 npm 套件

npm 必須先存在 package 頁面，才能在網頁設定 Trusted Publisher。因此初版使用維護者帳號人工發布：

```sh
npm login
npm whoami
npm publish --access public
```

`prepublishOnly` 會自動執行：

- npm 測試
- npm tarball dry run
- 十個 GitHub Release 資產檢查

若 npm 帳號要求雙因素驗證，依 npm 提示輸入 OTP。**不要建立或提交 `NPM_TOKEN`。**

發布後確認：

```sh
npm view ado-audit-log-exporter version
```

應回傳：

```text
0.1.0
```

* * *

## 四、設定 npm Trusted Publisher

開啟 npmjs.com：

```text
Packages
→ ado-audit-log-exporter
→ Settings
→ Trusted publishing
```

填入：

| 欄位 | 值 |
|---|---|
| Provider | GitHub Actions |
| Organization or user | `doggy8088` |
| Repository | `ado-audit-log-exporter` |
| Workflow filename | `npm-publish.yml` |
| Environment name | 留空 |
| Allowed actions | `npm publish` |

儲存後，在本機啟用 repository variable：

```sh
gh variable set NPM_TRUSTED_PUBLISHING_ENABLED \
  --repo doggy8088/ado-audit-log-exporter \
  --body true
```

確認：

```sh
gh variable list --repo doggy8088/ado-audit-log-exporter
```

接著在 npm package Settings 的 Publishing access 選擇：

```text
Require two-factor authentication and disallow tokens
```

npm 官方文件指出這項設定不會阻擋 Trusted Publisher，因為 OIDC 不使用傳統 token。

**初版已人工發布為 `0.1.0`，不可用 workflow 重複發布同版本來測試。Trusted Publisher 的首次實際驗證會發生在下一個新版本。**

* * *

## 五、人工發布 Rust crate

確認 crate 內容：

```sh
cargo package --list
cargo publish --dry-run --locked
```

登入並發布：

```sh
cargo login
cargo publish --locked
```

確認：

```sh
cargo search ado-audit-log-exporter --limit 1
```

crates.io 與 npm 是不同 registry，版本發布互不替代。

* * *

## 六、測試初版安裝

建立空白暫存目錄後測試 npm：

```sh
npm install --global ado-audit-log-exporter@0.1.0
ado-audit-log-exporter --version
```

測試 Cargo：

```sh
cargo install ado-audit-log-exporter@0.1.0 --locked
ado-audit-log-exporter --version
```

用測試組織與最小時間範圍執行實際匯出，不要把輸出檔提交到儲存庫。

* * *

## 七、後續版本

後續版本流程：

1. 更新 `Cargo.toml` 的 `version`
2. 更新 `package.json` 與 `package-lock.json` 的 `version`
3. 更新 CHANGELOG 或 Release 說明
4. 執行 `make check`
5. 提交並合併至 `main`
6. 建立並 push 相同版本的 `vX.Y.Z` tag
7. Release workflow 建立原生資產
8. GitHub Release published 觸發 npm trusted publishing
9. 視需要人工執行 `cargo publish --locked`

例如 `0.2.0`：

```sh
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

若 npm workflow 未執行，先檢查 repository variable 與 npm Trusted Publisher 的大小寫、儲存庫名稱及 workflow filename。

* * *

## 失敗處理

### Release workflow 失敗

不要移動同一 tag 指向其他提交。修正程式與版本後建立新的 patch 版本，例如 `v0.1.1`。

### GitHub Release 完成但 npm 發布失敗

若版本尚未出現在 npm，可修正 trusted publishing 設定後，在 Actions 手動執行 `npm-publish.yml` 並選擇同一 tag。

若 npm 已存在該版本，不可重複發布；必須增加版本。

### crates.io 發布失敗

若版本尚未上架，可修正 crate metadata 後重試。若已上架，crates.io 不允許覆寫；必須增加版本。

# 發布流程

`0.1.0` 已完成 npm 人工 bootstrap；`0.1.1` 已完成 GitHub Release 與 npm OIDC Trusted Publishing。原始碼已準備 `0.1.2` 修補版本，crates.io crate 尚未發布。

**GitHub Release 與 npm 都由 GitHub Actions 處理；crates.io 仍保留人工發布。**

* * *

## 一、發布前狀態

確認工作目錄、版本與遠端分支：

```sh
git status --short --branch
git fetch --tags origin
test "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)"
```

確認 Cargo、npm manifest 與 lockfile 版本一致：

```sh
node -p "require('./package.json').version"
node -p "require('./package-lock.json').packages[''].version"
cargo metadata --no-deps --format-version 1 |
  jq -r '.packages[0].version'
```

本次三者都必須是：

```text
0.1.2
```

確認 npm 尚未存在同版本：

```sh
npm view ado-audit-log-exporter version
```

發布 `0.1.2` 前應顯示 `0.1.1`。

* * *

## 二、完整品質檢查

```sh
make check
cargo package --list
cargo publish --dry-run --locked
npm pack --dry-run
```

檢查重點：

- Rust fmt、Clippy、測試與文件測試通過
- Rust 1.85 最低版本測試通過
- npm 測試通過
- Cargo crate 只包含 allowlist 檔案
- npm tarball 只包含 JavaScript 薄包裝、README 與 LICENSE
- 不含稽核輸出、PAT、`.env`、Cargo target 或本機快取

`npm/prepublish-check.cjs` 在 `v0.1.2` Release 尚未建立前預期失敗，因為十個公開資產還不存在。

* * *

## 三、啟用 npm Trusted Publisher

npm package Settings 的 Trusted publishing 必須設定：

| 欄位 | 值 |
|---|---|
| Provider | GitHub Actions |
| Organization or user | `doggy8088` |
| Repository | `ado-audit-log-exporter` |
| Workflow filename | `npm-publish.yml` |
| Environment name | 留空 |
| Allowed actions | `npm publish` |

Workflow filename 只能填檔名，不可填完整路徑。

GitHub repository variable 必須啟用：

```sh
gh variable set NPM_TRUSTED_PUBLISHING_ENABLED \
  --repo doggy8088/ado-audit-log-exporter \
  --body true
```

確認：

```sh
gh variable list --repo doggy8088/ado-audit-log-exporter
```

在 npm Publishing access 應選擇：

```text
Require two-factor authentication and disallow tokens
```

**GitHub Actions 不使用 `NPM_TOKEN` 或 `NODE_AUTH_TOKEN`。** `npm-publish.yml` 透過 `id-token: write` 取得 OIDC 短效憑證。

* * *

## 四、提交版本變更

版本與文件變更應使用 Conventional Commits，並以 UTF-8 暫存檔提交：

```sh
commit_msg_file="$(mktemp -t codex-commit-message)"
git commit -F "$commit_msg_file"
```

提交後推送並等待 CI：

```sh
git push origin main
gh run list --workflow ci.yml --limit 5
gh run watch
```

只有 `main` CI 全部成功後才能建立 tag。

* * *

## 五、建立 patch tag

```sh
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin v0.1.2
```

Release workflow 會：

1. 核對 tag、Cargo、`package.json` 與 `package-lock.json` 版本皆為 `0.1.2`
2. 在原生 runner 建置 macOS 與 Windows，並於 Debian 11 容器建置 GNU/Linux
3. 建立五個壓縮檔
4. 建立五個 LF 換行的 SHA-256 檔
5. 驗證 GNU/Linux 資產沒有引用高於 glibc 2.31 的符號
6. 建立公開 GitHub Release，且不覆寫既有同名資產

查看進度：

```sh
gh run list --workflow release.yml --limit 5
gh run watch
```

* * *

## 六、驗證 GitHub Release

```sh
gh release view v0.1.2
node npm/prepublish-check.cjs
```

必須存在十個資產：

- macOS ARM64 壓縮檔與 checksum
- macOS x64 壓縮檔與 checksum
- Linux ARM64 壓縮檔與 checksum
- Linux x64 壓縮檔與 checksum
- Windows x64 壓縮檔與 checksum

**npm 發布必須等十個公開 URL 全部可讀取。**

* * *

## 七、觸發並驗證 npm Trusted Publishing

Release workflow 使用 `GITHUB_TOKEN` 建立 Release。依 GitHub 的遞迴保護規則，這個 `release.published` 事件不會再建立另一個 workflow run，因此 Release 完成後必須手動 dispatch：

```sh
gh workflow run npm-publish.yml \
  --ref main \
  -f tag=v0.1.2
gh run list --workflow npm-publish.yml --limit 5
gh run watch
```

若 Release 是由工作流程外部建立，`release.published` 仍會直接觸發 `npm-publish.yml`。

參考：[GitHub Docs：When `GITHUB_TOKEN` triggers workflow runs](https://docs.github.com/en/actions/concepts/security/github_token#when-github_token-triggers-workflow-runs)

成功後確認：

```sh
npm view ado-audit-log-exporter version
```

應回傳：

```text
0.1.2
```

安裝驗證：

```sh
npm install --global ado-audit-log-exporter@0.1.2
ado-audit-log-exporter --version
```

trusted publishing 會自動產生 npm provenance，不需要在 workflow 使用 `--provenance`。

* * *

## 八、人工發布 Rust crate

crates.io 目前沒有自動發布 workflow。如需發布 `0.1.2`：

```sh
cargo login
cargo publish --dry-run --locked
cargo publish --locked
```

確認：

```sh
cargo search ado-audit-log-exporter --limit 1
```

crates.io 與 npm 是不同 registry，GitHub Release 或 npm 發布不會自動建立 crate。

* * *

## 後續版本

後續版本必須同步更新：

1. `Cargo.toml`
2. `Cargo.lock`
3. `package.json`
4. `package-lock.json`
5. README 與發布狀態文件

版本對應必須固定為：

```text
Cargo version = npm version = Git tag = GitHub Release
```

不要移動已公開的 tag，也不要覆寫已發布至 GitHub Release、npm 或 crates.io 的版本。Release workflow 只會保留既有的壓縮檔與 checksum 配對，或補上兩者皆缺少的完整配對；只存在其中一個時會失敗。

* * *

## 失敗處理

### Release workflow 失敗

若尚未建立 Release，可修正 workflow 後以 `workflow_dispatch` 指定既有 tag 重跑。若 Release 已存在，重跑不會覆寫同名資產。若二進位內容或原始碼需要變更，應建立新的 patch 版本，不可移動已公開 tag。

### GitHub Release 完成但 npm 發布失敗

若 npm 尚未存在該版本，修正 Trusted Publisher、repository variable 或 workflow 後，手動執行 `npm-publish.yml` 並指定同一 Release tag。

若 npm 已存在該版本，不可重複發布；必須增加版本。

### crates.io 發布失敗

若版本尚未上架，可修正 metadata 後重試。若已上架，crates.io 不允許覆寫；必須增加版本。

# CI/CD

本專案使用 GitHub Actions 執行品質檢查、原生執行檔發布與 npm trusted publishing。

* * *

## CI

檔案：

```text
.github/workflows/ci.yml
```

觸發：

- 對 `main` 的 push
- pull request

檢查：

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features --locked`
4. `npm ci --ignore-scripts`
5. `npm test`
6. `npm pack --dry-run`

`npm ci` 使用 `--ignore-scripts`，因此 CI 不會在一般測試階段下載 GitHub Release 執行檔。

* * *

## GitHub Release

檔案：

```text
.github/workflows/release.yml
```

觸發：

- push `v*.*.*` tag
- 維護者手動 workflow dispatch，並指定既有 tag

工作流程先確認：

```text
tag 版本 = Cargo.toml 版本 = package.json 版本
```

接著在原生 GitHub-hosted runner 建置五個 target。每個壓縮檔只有一個原生執行檔，並產生同名 `.sha256`。

最後以 `gh release create --verify-tag --generate-notes` 建立 GitHub Release。若 Release 已存在，會上傳並覆寫同名資產。

* * *

## npm 發布

檔案：

```text
.github/workflows/npm-publish.yml
```

觸發：

- GitHub Release published
- 維護者手動 workflow dispatch，並指定既有 Release tag

Release workflow 以 `GITHUB_TOKEN` 建立 Release 時，GitHub 不會讓該事件再啟動另一個 workflow，以避免遞迴執行。因此標籤觸發的自動 Release 完成後，維護者必須使用 `workflow_dispatch` 啟動 npm 發布；若 Release 是由工作流程外部建立，`release.published` 仍可直接觸發。

參考：[GitHub Docs：When `GITHUB_TOKEN` triggers workflow runs](https://docs.github.com/en/actions/concepts/security/github_token#when-github_token-triggers-workflow-runs)

必要條件：

- npm package 已存在
- npm Trusted Publisher 已綁定正確儲存庫與 workflow 檔名
- repository variable `NPM_TRUSTED_PUBLISHING_ENABLED` 等於 `true`
- GitHub Release 的十個平台資產 URL 可讀取

**工作流程不設定 npm 長效 token。**

* * *

## 權限

| Workflow | GitHub token 權限 |
|---|---|
| CI | `contents: read` |
| Release | `contents: write` |
| npm publish | `contents: read`、`id-token: write` |

OIDC 的 `id-token: write` 只授權 workflow 請求短效 ID token，不會把 npm publish 權限授予其他 workflow。npm 端仍會核對 owner、repository 與 workflow filename。

* * *

## crates.io

crates.io 由維護者人工執行 `cargo publish`。目前沒有把 crates.io token 放入 GitHub Actions，也沒有自動發布 crate。

若日後要自動化 crates.io，應另外評估 crates.io trusted publishing 的當期支援狀態；**不得直接把長效 crates.io token 寫入 workflow YAML。**

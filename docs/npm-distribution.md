# npm 跨平台封裝

npm 套件是 JavaScript 薄包裝，不編譯 Rust，也不把所有平台執行檔塞入 npm tarball。

* * *

## 安裝流程

```text
npm install
    ↓
辨識 process.platform 與 process.arch
    ↓
組合同版本 GitHub Release URL
    ↓
下載壓縮檔與 .sha256
    ↓
強制驗證 SHA-256
    ↓
解壓縮單一原生執行檔
    ↓
npm/cli.cjs 轉交參數與 exit code
```

下載 URL 格式：

```text
https://github.com/doggy8088/ado-audit-log-exporter/releases/download/v{version}/{asset}
```

npm 版本與 GitHub Release tag 必須一致。

* * *

## 支援平台對應

| Node platform | Node arch | Rust target |
|---|---|---|
| `darwin` | `arm64` | `aarch64-apple-darwin` |
| `darwin` | `x64` | `x86_64-apple-darwin` |
| `linux` | `arm64` | `aarch64-unknown-linux-gnu` |
| `linux` | `x64` | `x86_64-unknown-linux-gnu` |
| `win32` | `x64` | `x86_64-pc-windows-msvc` |

其他組合會明確回報 unsupported platform，不會錯誤下載其他架構。

* * *

## npm tarball 內容

`package.json` 的 `files` allowlist 只允許：

- `npm/cli.cjs`
- `npm/postinstall.cjs`
- `npm/prepublish-check.cjs`
- `README.md`
- `LICENSE`

執行：

```sh
npm pack --dry-run
```

可檢查實際封裝清單。**PAT、匯出記錄、Cargo target、Git 設定與本機執行檔不在 npm 套件中。**

* * *

## 發布前資產防護

`npm/prepublish-check.cjs` 會以 HTTP HEAD 檢查五個壓縮檔與五個 checksum，共十個 Release URL。

```sh
node npm/prepublish-check.cjs
```

缺少任何一個資產時，`npm publish` 的 `prepublishOnly` 會失敗。這可避免套件已上架，但使用者安裝時找不到原生執行檔。

* * *

## Trusted publishing

`.github/workflows/npm-publish.yml` 使用：

- GitHub-hosted runner
- Node.js 24
- npm 最新版
- `permissions: id-token: write`
- `npm publish --access public`
- 無 `NPM_TOKEN`
- 無 `NODE_AUTH_TOKEN`

npm CLI 會偵測 GitHub Actions 的 OIDC 環境，交換短效發布憑證。官方文件指出 trusted publishing 需要 npm CLI `11.5.1` 以上與 Node.js `22.14.0` 以上，且會自動產生 provenance，不需要 `--provenance`。

工作流程受 repository variable 控制：

```text
NPM_TRUSTED_PUBLISHING_ENABLED=true
```

`0.1.0` 已完成人工 bootstrap。從 `0.1.1` 起，在 npm 端綁定 Trusted Publisher 後，必須將此變數設為 `true`，GitHub Release 建立時才會觸發 OIDC 發布。

* * *

## npm 端綁定值

在 npm package Settings 的 Trusted publishing 設定：

| 欄位 | 值 |
|---|---|
| Provider | GitHub Actions |
| Organization or user | `doggy8088` |
| Repository | `ado-audit-log-exporter` |
| Workflow filename | `npm-publish.yml` |
| Environment name | 留空 |
| Allowed actions | `npm publish` |

Workflow filename 只能填檔名，不可填 `.github/workflows/npm-publish.yml`。

* * *

## 本機測試

```sh
npm ci --ignore-scripts
npm test
npm pack --dry-run
```

若本機已先執行：

```sh
cargo build --release --locked
```

接著執行一般 `npm install` 時，postinstall 會複製 `target/release` 中的本機執行檔，不會下載 GitHub Release。

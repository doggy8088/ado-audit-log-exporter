# 官方參考資料

本頁整理實作、操作與發布所依據的主要官方文件。**外部服務規格可能變動，發布前應重新查核。**

* * *

## Azure DevOps

- [Audit Log Query REST API 7.1](https://learn.microsoft.com/en-us/rest/api/azure/devops/audit/audit-log/query?view=azure-devops-rest-7.1)
  endpoint、`7.1-preview.1`、query parameters、`vso.auditlog` scope 與回應模型。

- [存取、匯出與篩選 Azure DevOps 稽核記錄](https://learn.microsoft.com/en-us/azure/devops/organizations/audit/azure-devops-auditing?view=azure-devops)
  啟用 Auditing、`View audit log` 權限、90 天保留期與 Azure DevOps Services 限制。

- [Azure DevOps 權限、安全群組與服務帳號參考](https://learn.microsoft.com/en-us/azure/devops/organizations/security/permissions?view=azure-devops)
  `AuditLog, Read` 與其他 Auditing 權限。

- [Azure DevOps REST API 驗證指引](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/authentication-guidance?view=azure-devops)
  Microsoft Entra token、managed identity、service principal 與 PAT 的選擇。

- [Azure DevOps PAT 使用說明](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops)
  PAT 建立、保存、輪替與撤銷。

* * *

## npm

- [Trusted publishing for npm packages](https://docs.npmjs.com/trusted-publishers/)
  OIDC、GitHub Actions 欄位、Node 與 npm 最低版本、provenance 及 token access 限制。

- [About npm provenance](https://docs.npmjs.com/generating-provenance-statements/)
  套件 provenance 的條件與驗證。

- [Creating and publishing unscoped public packages](https://docs.npmjs.com/creating-and-publishing-unscoped-public-packages/)
  初次人工公開發布。

- [package.json](https://docs.npmjs.com/cli/v11/configuring-npm/package-json)
  `bin`、`files`、`engines`、`repository` 與 `publishConfig`。

- [npm CLI publish](https://docs.npmjs.com/cli/v11/commands/npm-publish)
  publish lifecycle 與公開存取設定。

* * *

## GitHub Actions 與 Releases

- [GitHub Actions OIDC](https://docs.github.com/en/actions/concepts/security/openid-connect)
  OIDC 的信任模型。

- [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
  runner 架構與限制。

- [Managing releases in a repository](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
  tag、Release 與資產管理。

- [actions/checkout](https://github.com/actions/checkout)
- [actions/setup-node](https://github.com/actions/setup-node)
- [actions/upload-artifact](https://github.com/actions/upload-artifact)
- [actions/download-artifact](https://github.com/actions/download-artifact)

* * *

## Rust

- [The Cargo Book: Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)
- [The Cargo Book: Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [docs.rs](https://docs.rs/)
- [Serde](https://serde.rs/)
- [reqwest](https://docs.rs/reqwest/)
- [Tokio](https://tokio.rs/)
- [clap](https://docs.rs/clap/)

* * *

## 規格查核日期

本文件最近查核日期：

```text
2026-07-30
```

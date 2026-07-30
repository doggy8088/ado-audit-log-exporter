# 文件索引

本目錄涵蓋 CLI、Rust library、npm 原生執行檔包裝、CI/CD、安全與發布流程。

* * *

## 使用者文件

1. [快速開始](getting-started.md)
2. [驗證與權限](authentication.md)
3. [CLI 與 Makefile 參考](command-reference.md)
4. [輸出格式](output-formats.md)
5. [疑難排解](troubleshooting.md)
6. [安全與操作](security-and-operations.md)

* * *

## 開發者與維護者文件

1. [Rust library 使用說明](rust-library.md)
2. [REST API 與實作架構](api-and-implementation.md)
3. [npm 跨平台封裝](npm-distribution.md)
4. [CI/CD](ci-cd.md)
5. [初版與後續發布](releasing.md)
6. [官方參考資料](references.md)

* * *

## 目前發布狀態

**原始碼版本為 `0.1.0`，但初版 GitHub Release、npm 套件與 crates.io crate 尚未發布。**

維護者應依 [初版發布手冊](releasing.md) 完成人工發布。npm trusted publishing 已在 GitHub Actions 中設定，但必須先建立 npm 套件頁面並完成 npm 端的 Trusted Publisher 綁定後，才能啟用。

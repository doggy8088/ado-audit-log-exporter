use reqwest::{StatusCode, header::InvalidHeaderValue};
use thiserror::Error;

/// Azure DevOps 稽核記錄 client 的錯誤類型。
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("缺少憑證；請設定 AZURE_DEVOPS_EXT_PAT、ADO_ACCESS_TOKEN 或 ADO_PAT 環境變數")]
    MissingAuthentication,

    #[error("不可同時設定 PAT 與 ADO_ACCESS_TOKEN")]
    ConflictingAuthentication,

    #[error("無效的驗證設定：{0}")]
    InvalidAuthentication(String),

    #[error("驗證標頭含有不合法的字元")]
    InvalidAuthorizationHeader(#[source] InvalidHeaderValue),

    #[error("無效的 Azure DevOps 組織名稱：{0}")]
    InvalidOrganization(String),

    #[error("無效的 API endpoint：{0}")]
    InvalidEndpoint(String),

    #[error("無效的查詢條件：{0}")]
    InvalidQuery(String),

    #[error("無法建立 HTTP client")]
    ClientBuild(#[source] reqwest::Error),

    #[error("Azure DevOps REST API 要求失敗")]
    Transport(#[source] reqwest::Error),

    #[error("Azure DevOps REST API 回傳 HTTP {status}：{message}")]
    HttpStatus {
        /// HTTP 狀態碼。
        status: StatusCode,
        /// 經截斷的伺服器錯誤訊息。
        message: String,
    },

    #[error("Azure DevOps REST API 回傳無效 JSON")]
    InvalidJson(#[source] serde_json::Error),

    #[error("無法解析 Azure DevOps REST API 回應：{0}")]
    InvalidResponse(String),

    #[error("Azure DevOps 重複傳回 continuation token，已停止以避免無限迴圈")]
    RepeatedContinuationToken,
}

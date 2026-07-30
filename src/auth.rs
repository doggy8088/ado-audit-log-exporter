use std::{env, fmt};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::header::HeaderValue;

use crate::AuditError;

/// Azure DevOps REST API 驗證方式。
///
/// `Debug` 實作只會顯示已遮罩的內容，不會輸出憑證。
pub enum Authentication {
    /// Azure DevOps Personal Access Token。
    PersonalAccessToken(String),
    /// Microsoft Entra OAuth access token。
    BearerToken(String),
}

impl Authentication {
    /// 建立 PAT 驗證方式。
    pub fn personal_access_token(token: impl Into<String>) -> Result<Self, AuditError> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err(AuditError::InvalidAuthentication(
                "PAT 不可為空白".to_owned(),
            ));
        }
        Ok(Self::PersonalAccessToken(token))
    }

    /// 建立 Bearer token 驗證方式。
    pub fn bearer_token(token: impl Into<String>) -> Result<Self, AuditError> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err(AuditError::InvalidAuthentication(
                "access token 不可為空白".to_owned(),
            ));
        }
        Ok(Self::BearerToken(token))
    }

    /// 從環境變數讀取憑證。
    ///
    /// PAT 的優先順序為 `AZURE_DEVOPS_EXT_PAT`、`ADO_PAT`。若同時設定
    /// PAT 與 `ADO_ACCESS_TOKEN`，會回傳錯誤，避免使用到非預期的身分。
    pub fn from_env() -> Result<Self, AuditError> {
        let extension_pat = non_empty_env("AZURE_DEVOPS_EXT_PAT");
        let fallback_pat = non_empty_env("ADO_PAT");
        let access_token = non_empty_env("ADO_ACCESS_TOKEN");
        let pat = extension_pat.or(fallback_pat);

        match (pat, access_token) {
            (Some(_), Some(_)) => Err(AuditError::ConflictingAuthentication),
            (Some(token), None) => Self::personal_access_token(token),
            (None, Some(token)) => Self::bearer_token(token),
            (None, None) => Err(AuditError::MissingAuthentication),
        }
    }

    pub(crate) fn header_value(&self) -> Result<HeaderValue, AuditError> {
        let value = match self {
            Self::PersonalAccessToken(token) => {
                let encoded = STANDARD.encode(format!(":{token}"));
                format!("Basic {encoded}")
            }
            Self::BearerToken(token) => format!("Bearer {token}"),
        };

        let mut header =
            HeaderValue::from_str(&value).map_err(AuditError::InvalidAuthorizationHeader)?;
        header.set_sensitive(true);
        Ok(header)
    }
}

impl fmt::Debug for Authentication {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PersonalAccessToken(_) => {
                formatter.write_str("Authentication::PersonalAccessToken([REDACTED])")
            }
            Self::BearerToken(_) => formatter.write_str("Authentication::BearerToken([REDACTED])"),
        }
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::Authentication;

    #[test]
    fn debug_output_redacts_pat() {
        let authentication =
            Authentication::personal_access_token("do-not-print-this").expect("valid PAT");

        let rendered = format!("{authentication:?}");

        assert!(rendered.contains("[REDACTED]"));
        assert!(!rendered.contains("do-not-print-this"));
    }

    #[test]
    fn authorization_header_is_marked_sensitive() {
        let authentication =
            Authentication::personal_access_token("do-not-print-this").expect("valid PAT");
        let header = authentication.header_value().expect("valid header");

        assert!(header.is_sensitive());
    }
}

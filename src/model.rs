use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Azure DevOps 稽核記錄項目。
///
/// 已知欄位提供具名屬性；API 新增的欄位會保留在 [`Self::extra_fields`]，
/// 因此升級 Azure DevOps 服務時不會默默遺失資料。
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    #[serde(rename = "actorCUID", skip_serializing_if = "Option::is_none")]
    pub actor_cuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_user_id: Option<String>,
    #[serde(rename = "actorUPN", skip_serializing_if = "Option::is_none")]
    pub actor_upn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_mechanism: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(flatten)]
    pub extra_fields: Map<String, Value>,
}

/// 一頁 Azure DevOps 稽核記錄。
#[derive(Clone, Debug, PartialEq)]
pub struct AuditPage {
    /// 本頁項目。
    pub entries: Vec<AuditLogEntry>,
    /// 下一頁使用的 continuation token。
    pub continuation_token: Option<String>,
    /// 服務端是否表示尚有下一頁。
    pub has_more: bool,
}

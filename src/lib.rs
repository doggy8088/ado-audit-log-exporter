//! Azure DevOps 稽核記錄 REST API 的非同步 Rust client。
//!
//! 這個 crate 同時支援 PAT 與 Microsoft Entra access token，並提供逐頁讀取介面，
//! 讓呼叫端可以串流處理大量稽核記錄。
//!
//! # 範例
//!
//! ```no_run
//! use ado_audit_log_exporter::{AuditClient, AuditQuery, Authentication};
//! use chrono::{Duration, Utc};
//!
//! # async fn run() -> Result<(), ado_audit_log_exporter::AuditError> {
//! let authentication = Authentication::from_env()?;
//! let client = AuditClient::new("miniasp", authentication)?;
//! let query = AuditQuery::new(Utc::now() - Duration::days(1), Utc::now())?;
//! let mut pager = client.pager(query);
//!
//! while let Some(page) = pager.next_page().await? {
//!     for entry in page.entries {
//!         println!("{}", entry.action_id.as_deref().unwrap_or("unknown"));
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod auth;
mod client;
mod error;
mod model;
mod query;

pub use auth::Authentication;
pub use client::{AuditClient, AuditLogPager, RetryPolicy};
pub use error::AuditError;
pub use model::{AuditLogEntry, AuditPage};
pub use query::AuditQuery;

/// Azure DevOps Audit Log REST API 版本。
pub const API_VERSION: &str = "7.1-preview.1";

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DocumentVersion {
    pub id: String,
    pub document_id: String,
    pub version_number: i64,
    pub content: String,
    pub word_count: i64,
    pub char_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DocumentVersionSummary {
    pub id: String,
    pub document_id: String,
    pub version_number: i64,
    pub word_count: i64,
    pub char_count: i64,
    pub created_at: String,
}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for the `messages` table.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub contract_id: Uuid,
    pub sender_id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub is_read: bool,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contracts::Entity",
        from = "Column::ContractId",
        to = "super::contracts::Column::Id"
    )]
    Contract,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::SenderId",
        to = "super::users::Column::Id"
    )]
    Sender,
}

impl Related<super::contracts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contract.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sender.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// ── DTOs ──

/// DTO for creating a new message (used internally by the chat system).
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMessage {
    pub contract_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
}

/// Response DTO for messages sent over WebSocket and REST.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Model> for MessageResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            contract_id: m.contract_id,
            sender_id: m.sender_id,
            content: m.content,
            is_read: m.is_read,
            created_at: m.created_at,
        }
    }
}

/// Query parameters for paginated message history.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

/// Response for the conversations list endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct ConversationSummary {
    pub contract_id: Uuid,
    pub other_user_id: Uuid,
    pub other_user_name: Option<String>,
    pub last_message: Option<String>,
    pub last_message_at: Option<chrono::DateTime<chrono::Utc>>,
    pub unread_count: u64,
}

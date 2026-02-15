use sea_orm::prelude::Expr;
use sea_orm::*;
use uuid::Uuid;

use crate::models::messages::{self, CreateMessage};

/// Insert a new message.
pub async fn insert_message(
    db: &DatabaseConnection,
    input: CreateMessage,
) -> Result<messages::Model, DbErr> {
    let new_message = messages::ActiveModel {
        id: Set(Uuid::new_v4()),
        contract_id: Set(input.contract_id),
        sender_id: Set(input.sender_id),
        content: Set(input.content),
        is_read: Set(false),
        created_at: Set(chrono::Utc::now()),
    };

    new_message.insert(db).await
}

/// Fetch messages for a contract, ordered by created_at descending, with pagination.
pub async fn get_messages_by_contract(
    db: &DatabaseConnection,
    contract_id: Uuid,
    page: u64,
    limit: u64,
) -> Result<Vec<messages::Model>, DbErr> {
    messages::Entity::find()
        .filter(messages::Column::ContractId.eq(contract_id))
        .order_by_desc(messages::Column::CreatedAt)
        .offset((page - 1) * limit)
        .limit(limit)
        .all(db)
        .await
}

/// Fetch a single message by ID.
pub async fn get_message_by_id(
    db: &DatabaseConnection,
    message_id: Uuid,
) -> Result<Option<messages::Model>, DbErr> {
    messages::Entity::find_by_id(message_id).one(db).await
}

/// Mark a single message as read.
pub async fn mark_message_as_read(
    db: &DatabaseConnection,
    message_id: Uuid,
) -> Result<messages::Model, DbErr> {
    let message = messages::Entity::find_by_id(message_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Message not found".to_string()))?;

    let mut active: messages::ActiveModel = message.into();
    active.is_read = Set(true);

    active.update(db).await
}

/// Mark all messages in a contract as read for a specific recipient (i.e., messages NOT sent by them).
pub async fn mark_all_read_for_contract(
    db: &DatabaseConnection,
    contract_id: Uuid,
    reader_id: Uuid,
) -> Result<u64, DbErr> {
    let result = messages::Entity::update_many()
        .col_expr(messages::Column::IsRead, Expr::value(true))
        .filter(messages::Column::ContractId.eq(contract_id))
        .filter(messages::Column::SenderId.ne(reader_id))
        .filter(messages::Column::IsRead.eq(false))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}

/// Count unread messages in a contract for a specific user (messages sent by the other party).
pub async fn count_unread_for_contract(
    db: &DatabaseConnection,
    contract_id: Uuid,
    user_id: Uuid,
) -> Result<u64, DbErr> {
    messages::Entity::find()
        .filter(messages::Column::ContractId.eq(contract_id))
        .filter(messages::Column::SenderId.ne(user_id))
        .filter(messages::Column::IsRead.eq(false))
        .count(db)
        .await
}

/// Get the latest message for a contract.
pub async fn get_latest_message_for_contract(
    db: &DatabaseConnection,
    contract_id: Uuid,
) -> Result<Option<messages::Model>, DbErr> {
    messages::Entity::find()
        .filter(messages::Column::ContractId.eq(contract_id))
        .order_by_desc(messages::Column::CreatedAt)
        .one(db)
        .await
}

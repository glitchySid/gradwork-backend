use sea_orm::prelude::Expr;
use sea_orm::*;
use std::collections::{HashMap, HashSet};
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
    limit: u64,
    cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
    cursor_id: Option<Uuid>,
) -> Result<Vec<messages::Model>, DbErr> {
    let mut query = messages::Entity::find().filter(messages::Column::ContractId.eq(contract_id));

    if let (Some(cursor_created_at), Some(cursor_id)) = (cursor_created_at, cursor_id) {
        query = query.filter(
            Condition::any()
                .add(messages::Column::CreatedAt.lt(cursor_created_at))
                .add(
                    Condition::all()
                        .add(messages::Column::CreatedAt.eq(cursor_created_at))
                        .add(messages::Column::Id.lt(cursor_id)),
                ),
        );
    }

    query
        .order_by_desc(messages::Column::CreatedAt)
        .order_by_desc(messages::Column::Id)
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

/// Count unread messages for many contracts in one query and return a contract_id -> unread_count map.
pub async fn count_unread_for_contracts(
    db: &DatabaseConnection,
    contract_ids: Vec<Uuid>,
    user_id: Uuid,
) -> Result<HashMap<Uuid, u64>, DbErr> {
    if contract_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let unread_messages = messages::Entity::find()
        .filter(messages::Column::ContractId.is_in(contract_ids))
        .filter(messages::Column::SenderId.ne(user_id))
        .filter(messages::Column::IsRead.eq(false))
        .all(db)
        .await?;

    let mut counts: HashMap<Uuid, u64> = HashMap::new();
    for message in unread_messages {
        *counts.entry(message.contract_id).or_insert(0) += 1;
    }

    Ok(counts)
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

/// Get latest messages for many contracts in one query and return a contract_id -> message map.
pub async fn get_latest_messages_for_contracts(
    db: &DatabaseConnection,
    contract_ids: Vec<Uuid>,
) -> Result<HashMap<Uuid, messages::Model>, DbErr> {
    if contract_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = messages::Entity::find()
        .filter(messages::Column::ContractId.is_in(contract_ids))
        .order_by_asc(messages::Column::ContractId)
        .order_by_desc(messages::Column::CreatedAt)
        .order_by_desc(messages::Column::Id)
        .all(db)
        .await?;

    let mut latest: HashMap<Uuid, messages::Model> = HashMap::new();
    let mut seen: HashSet<Uuid> = HashSet::new();

    for row in rows {
        if seen.insert(row.contract_id) {
            latest.insert(row.contract_id, row);
        }
    }

    Ok(latest)
}

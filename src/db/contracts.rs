use sea_orm::*;
use uuid::Uuid;

use crate::models::contracts::{self, CreateContract, Status, UpdateContractStatus};

/// Insert a new contract (defaults to Pending status).
pub async fn insert_contract(
    db: &DatabaseConnection,
    input: CreateContract,
) -> Result<contracts::Model, DbErr> {
    let new_contract = contracts::ActiveModel {
        id: Set(Uuid::new_v4()),
        gig_id: Set(input.gig_id),
        user_id: Set(input.user_id),
        status: Set(Status::Pending),
        created_at: Set(chrono::Utc::now()),
    };

    new_contract.insert(db).await
}

/// Fetch all contracts.
pub async fn get_all_contracts(db: &DatabaseConnection) -> Result<Vec<contracts::Model>, DbErr> {
    contracts::Entity::find().all(db).await
}

/// Fetch a single contract by ID.
pub async fn get_contract_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<contracts::Model>, DbErr> {
    contracts::Entity::find_by_id(id).one(db).await
}

/// Update the status of a contract.
pub async fn update_contract_status(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdateContractStatus,
) -> Result<contracts::Model, DbErr> {
    let contract = contracts::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Contract not found".to_string()))?;

    let mut active: contracts::ActiveModel = contract.into();
    active.status = Set(input.status);

    active.update(db).await
}

/// Delete a contract by ID.
pub async fn delete_contract(db: &DatabaseConnection, id: Uuid) -> Result<DeleteResult, DbErr> {
    contracts::Entity::delete_by_id(id).exec(db).await
}

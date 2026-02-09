use sea_orm::*;
use uuid::Uuid;

use crate::models::gigs::{self, CreateGig, UpdateGig};

/// Insert a new gig into the database.
pub async fn insert_gig(
    db: &DatabaseConnection,
    input: CreateGig,
    user_id: Uuid,
) -> Result<gigs::Model, DbErr> {
    let new_gig = gigs::ActiveModel {
        id: Set(Uuid::new_v4()),
        title: Set(input.title),
        description: Set(input.description),
        price: Set(input.price),
        user_id: Set(user_id),
        created_at: Set(chrono::Utc::now()),
    };

    new_gig.insert(db).await
}

/// Fetch all gigs.
pub async fn get_all_gigs(db: &DatabaseConnection) -> Result<Vec<gigs::Model>, DbErr> {
    gigs::Entity::find().all(db).await
}

/// Fetch a single gig by ID.
pub async fn get_gig_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<gigs::Model>, DbErr> {
    gigs::Entity::find_by_id(id).one(db).await
}

/// Update an existing gig.
pub async fn update_gig(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdateGig,
) -> Result<gigs::Model, DbErr> {
    let gig = gigs::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Gig not found".to_string()))?;

    let mut active: gigs::ActiveModel = gig.into();

    if let Some(title) = input.title {
        active.title = Set(title);
    }
    if let Some(description) = input.description {
        active.description = Set(description);
    }
    if let Some(price) = input.price {
        active.price = Set(price);
    }

    active.update(db).await
}

/// Delete a gig by ID.
pub async fn delete_gig(db: &DatabaseConnection, id: Uuid) -> Result<DeleteResult, DbErr> {
    gigs::Entity::delete_by_id(id).exec(db).await
}

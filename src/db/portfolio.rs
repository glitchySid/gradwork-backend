use sea_orm::*;
use uuid::Uuid;

use crate::models::portfolio::{self, CreatePortfolio, UpdatePortfolio};

/// Insert a new portfolio item.
pub async fn insert_portfolio(
    db: &DatabaseConnection,
    input: CreatePortfolio,
) -> Result<portfolio::Model, DbErr> {
    let new_portfolio = portfolio::ActiveModel {
        id: Set(Uuid::new_v4()),
        title: Set(input.title),
        description: Set(input.description),
        freelancer_id: Set(input.freelancer_id),
        thumbnail_url: Set(input.thumbnail_url),
        price: Set(input.price),
        created_at: Set(chrono::Utc::now()),
    };

    new_portfolio.insert(db).await
}

/// Fetch all portfolio items.
pub async fn get_all_portfolios(db: &DatabaseConnection) -> Result<Vec<portfolio::Model>, DbErr> {
    portfolio::Entity::find().all(db).await
}

/// Fetch a single portfolio item by ID.
pub async fn get_portfolio_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<portfolio::Model>, DbErr> {
    portfolio::Entity::find_by_id(id).one(db).await
}

/// Fetch all portfolio items for a given freelancer.
pub async fn get_portfolios_by_freelancer(
    db: &DatabaseConnection,
    freelancer_id: Uuid,
) -> Result<Vec<portfolio::Model>, DbErr> {
    portfolio::Entity::find()
        .filter(portfolio::Column::FreelancerId.eq(freelancer_id))
        .all(db)
        .await
}

/// Update an existing portfolio item.
pub async fn update_portfolio(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdatePortfolio,
) -> Result<portfolio::Model, DbErr> {
    let item = portfolio::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Portfolio not found".to_string()))?;

    let mut active: portfolio::ActiveModel = item.into();

    if let Some(title) = input.title {
        active.title = Set(title);
    }
    if let Some(description) = input.description {
        active.description = Set(description);
    }
    if let Some(thumbnail_url) = input.thumbnail_url {
        active.thumbnail_url = Set(Some(thumbnail_url));
    }
    if let Some(price) = input.price {
        active.price = Set(price);
    }

    active.update(db).await
}

/// Delete a portfolio item by ID.
pub async fn delete_portfolio(db: &DatabaseConnection, id: Uuid) -> Result<DeleteResult, DbErr> {
    portfolio::Entity::delete_by_id(id).exec(db).await
}

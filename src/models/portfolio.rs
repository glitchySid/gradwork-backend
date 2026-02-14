use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for the `portfolios` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "portfolios")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub freelancer_id: Uuid,
    pub thumbnail_url: Option<String>,
    #[sea_orm(column_type = "Double")]
    pub price: f64,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::FreelancerId",
        to = "super::users::Column::Id"
    )]
    Freelancer,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Freelancer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// ── DTOs ──

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePortfolio {
    pub title: String,
    pub description: String,
    pub freelancer_id: Uuid,
    pub thumbnail_url: Option<String>,
    pub price: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePortfolio {
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub price: Option<f64>,
}

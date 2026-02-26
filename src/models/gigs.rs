use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for the `gigs` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "gigs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    #[sea_orm(column_type = "Double")]
    pub price: f64,
    pub thumbnail_url: Option<String>,
    pub category: Categories,
    pub user_id: Uuid,
    pub created_at: DateTimeUtc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Categories {
    #[sea_orm(string_value = "web_development")]
    WebDevelopment,
    #[sea_orm(string_value = "mobile_development")]
    MobileDevelopment,
    #[sea_orm(string_value = "data_science")]
    DataScience,
    #[sea_orm(string_value = "design")]
    Design,
    #[sea_orm(string_value = "video_editing")]
    VideoEditing,
    #[sea_orm(string_value = "content_writing")]
    ContentWriting,
    #[sea_orm(string_value = "other")]
    Other,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::contracts::Entity")]
    Contracts,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::contracts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contracts.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// ── DTOs ──

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGig {
    pub title: String,
    pub description: String,
    pub price: f64,
    pub thumbnail_url: Option<String>,
    pub category: Option<Categories>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub thumbnail_url: Option<String>,
    pub category: Option<Categories>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GigListQuery {
    pub limit: Option<u64>,
    pub cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor_id: Option<Uuid>,
}

impl GigListQuery {
    pub fn limit(&self) -> u64 {
        self.limit.unwrap_or(20).min(100)
    }
}

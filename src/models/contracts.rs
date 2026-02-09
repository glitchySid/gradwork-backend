use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Contract status stored as a lowercase string in the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Status {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "accepted")]
    Accepted,
    #[sea_orm(string_value = "rejected")]
    Rejected,
}

/// SeaORM entity for the `contracts` table.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contracts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub gig_id: Uuid,
    pub user_id: Uuid,
    pub status: Status,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::gigs::Entity",
        from = "Column::GigId",
        to = "super::gigs::Column::Id"
    )]
    Gig,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::gigs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Gig.def()
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
pub struct CreateContract {
    pub gig_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContractStatus {
    pub status: Status,
}

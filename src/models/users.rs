use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// The `Roles` enum maps to a Postgres TEXT column stored as lowercase strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Roles {
    #[sea_orm(string_value = "client")]
    Client,
    #[sea_orm(string_value = "freelancer")]
    Freelancer,
    #[sea_orm(string_value = "admin")]
    Admin,
}

/// SeaORM entity for the `users` table.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(unique)]
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub auth_provider: String,
    pub role: Roles,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::contracts::Entity")]
    Contracts,
    #[sea_orm(has_many = "super::portfolio::Entity")]
    Portfolios,
}

impl Related<super::contracts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contracts.def()
    }
}

impl Related<super::portfolio::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Portfolios.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// ── DTOs (not stored in DB, used for request bodies) ──

/// Used internally by the auth middleware to create a user from JWT claims.
#[derive(Debug, Clone)]
pub struct CreateUserFromAuth {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub auth_provider: String,
    pub role: Roles,
}

/// Used by the `POST /api/auth/complete-profile` endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CompleteProfile {
    pub username: Option<String>,
    pub role: Option<Roles>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Used for admin-level user updates.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: Option<Roles>,
}

/// A safe user representation for API responses (never leaks internal fields).
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: Roles,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

impl From<Model> for UserResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            username: m.username,
            display_name: m.display_name,
            avatar_url: m.avatar_url,
            role: m.role,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

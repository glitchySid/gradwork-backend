use sea_orm::*;
use uuid::Uuid;

use crate::models::users::{self, CompleteProfile, CreateUserFromAuth, UpdateUser};

/// Create a new user from Supabase Auth JWT claims (called by auth middleware).
pub async fn find_or_create_from_auth(
    db: &DatabaseConnection,
    input: CreateUserFromAuth,
) -> Result<users::Model, DbErr> {
    // Try to find the user first (by Supabase auth UUID).
    if let Some(existing) = users::Entity::find_by_id(input.id).one(db).await? {
        return Ok(existing);
    }

    // User doesn't exist yet — create from JWT claims.
    let new_user = users::ActiveModel {
        id: Set(input.id),
        email: Set(input.email),
        username: Set(None),
        display_name: Set(input.display_name),
        avatar_url: Set(input.avatar_url),
        auth_provider: Set(input.auth_provider),
        role: Set(input.role),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(None),
    };

    new_user.insert(db).await
}

/// Fetch all users.
pub async fn get_all_users(db: &DatabaseConnection) -> Result<Vec<users::Model>, DbErr> {
    users::Entity::find().all(db).await
}

/// Fetch a single user by ID.
pub async fn get_user_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find_by_id(id).one(db).await
}

/// Complete a user's profile (set username, role, display_name after first login).
pub async fn complete_profile(
    db: &DatabaseConnection,
    id: Uuid,
    input: CompleteProfile,
) -> Result<users::Model, DbErr> {
    let user = users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

    let mut active: users::ActiveModel = user.into();

    if let Some(username) = input.username {
        active.username = Set(Some(username));
    }
    if let Some(role) = input.role {
        active.role = Set(role);
    }
    if let Some(display_name) = input.display_name {
        active.display_name = Set(Some(display_name));
    }
    if let Some(avatar_url) = input.avatar_url {
        active.avatar_url = Set(Some(avatar_url));
    }
    active.updated_at = Set(Some(chrono::Utc::now()));

    active.update(db).await
}

/// Update an existing user (admin-level).
pub async fn update_user(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdateUser,
) -> Result<users::Model, DbErr> {
    let user = users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

    let mut active: users::ActiveModel = user.into();

    if let Some(email) = input.email {
        active.email = Set(email);
    }
    if let Some(username) = input.username {
        active.username = Set(Some(username));
    }
    if let Some(display_name) = input.display_name {
        active.display_name = Set(Some(display_name));
    }
    if let Some(avatar_url) = input.avatar_url {
        active.avatar_url = Set(Some(avatar_url));
    }
    if let Some(role) = input.role {
        active.role = Set(role);
    }
    active.updated_at = Set(Some(chrono::Utc::now()));

    active.update(db).await
}

/// Delete a user by ID.
pub async fn delete_user(db: &DatabaseConnection, id: Uuid) -> Result<DeleteResult, DbErr> {
    users::Entity::delete_by_id(id).exec(db).await
}

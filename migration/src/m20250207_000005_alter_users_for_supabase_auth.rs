use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Users {
    Table,
    Password,
    Username,
    DisplayName,
    AvatarUrl,
    AuthProvider,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Drop the `password` column — no longer needed with OAuth.
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Password)
                    .to_owned(),
            )
            .await?;

        // 2. Make `username` nullable (Google users won't pick one at signup).
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(ColumnDef::new(Users::Username).string().null())
                    .to_owned(),
            )
            .await?;

        // 3. Add `display_name` — populated from Google profile.
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::DisplayName).string().null())
                    .to_owned(),
            )
            .await?;

        // 4. Add `avatar_url` — Google profile picture.
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::AvatarUrl).text().null())
                    .to_owned(),
            )
            .await?;

        // 5. Add `auth_provider` — tracks which provider was used.
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::AuthProvider)
                            .string()
                            .not_null()
                            .default("google"),
                    )
                    .to_owned(),
            )
            .await?;

        // 6. Add `updated_at` — nullable timestamp for profile edits.
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse: drop new columns, restore password, make username NOT NULL.

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::AuthProvider)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::AvatarUrl)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::DisplayName)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(ColumnDef::new(Users::Username).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::Password).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

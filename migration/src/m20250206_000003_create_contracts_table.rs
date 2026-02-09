use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Identifiers for the `contracts` table and its columns.
#[derive(DeriveIden)]
enum Contracts {
    Table,
    Id,
    GigId,
    UserId,
    Status,
    CreatedAt,
}

/// Re-declare parent table identifiers for foreign-key references.
#[derive(DeriveIden)]
enum Gigs {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Contracts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Contracts::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Contracts::GigId).uuid().not_null())
                    .col(ColumnDef::new(Contracts::UserId).uuid().not_null())
                    .col(ColumnDef::new(Contracts::Status).string().not_null())
                    .col(
                        ColumnDef::new(Contracts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contracts_gig_id")
                            .from(Contracts::Table, Contracts::GigId)
                            .to(Gigs::Table, Gigs::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contracts_user_id")
                            .from(Contracts::Table, Contracts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Contracts::Table).to_owned())
            .await
    }
}

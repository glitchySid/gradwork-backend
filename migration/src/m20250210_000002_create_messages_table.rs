use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Identifiers for the `messages` table and its columns.
#[derive(DeriveIden)]
enum Messages {
    Table,
    Id,
    ContractId,
    SenderId,
    Content,
    IsRead,
    CreatedAt,
}

/// Re-declare parent table identifiers for foreign-key references.
#[derive(DeriveIden)]
enum Contracts {
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
        // Create the messages table.
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Messages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Messages::ContractId).uuid().not_null())
                    .col(ColumnDef::new(Messages::SenderId).uuid().not_null())
                    .col(ColumnDef::new(Messages::Content).text().not_null())
                    .col(
                        ColumnDef::new(Messages::IsRead)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Messages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_messages_contract_id")
                            .from(Messages::Table, Messages::ContractId)
                            .to(Contracts::Table, Contracts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_messages_sender_id")
                            .from(Messages::Table, Messages::SenderId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create an index on (contract_id, created_at) for efficient history queries.
        manager
            .create_index(
                Index::create()
                    .name("idx_messages_contract_created")
                    .table(Messages::Table)
                    .col(Messages::ContractId)
                    .col(Messages::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await
    }
}

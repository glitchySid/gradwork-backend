use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Messages {
    Table,
    ContractId,
    IsRead,
    SenderId,
    CreatedAt,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_messages_contract_is_read_sender")
                    .table(Messages::Table)
                    .col(Messages::ContractId)
                    .col(Messages::IsRead)
                    .col(Messages::SenderId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_messages_contract_created_id")
                    .table(Messages::Table)
                    .col(Messages::ContractId)
                    .col(Messages::CreatedAt)
                    .col(Messages::Id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_messages_contract_created_id")
                    .table(Messages::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_messages_contract_is_read_sender")
                    .table(Messages::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

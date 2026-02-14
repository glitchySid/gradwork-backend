use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Contracts {
    Table,
    GigId,
    UserId,
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    ContractId,
    SenderId,
}

#[derive(DeriveIden)]
enum Gigs {
    Table,
    UserId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Index on contracts.gig_id for fetching contracts by gig
        manager
            .create_index(
                Index::create()
                    .name("idx_contracts_gig_id")
                    .table(Contracts::Table)
                    .col(Contracts::GigId)
                    .to_owned(),
            )
            .await?;

        // Index on contracts.user_id for fetching contracts by user
        manager
            .create_index(
                Index::create()
                    .name("idx_contracts_user_id")
                    .table(Contracts::Table)
                    .col(Contracts::UserId)
                    .to_owned(),
            )
            .await?;

        // Index on messages.contract_id for fetching messages by contract
        manager
            .create_index(
                Index::create()
                    .name("idx_messages_contract_id")
                    .table(Messages::Table)
                    .col(Messages::ContractId)
                    .to_owned(),
            )
            .await?;

        // Index on messages.sender_id for fetching messages by sender
        manager
            .create_index(
                Index::create()
                    .name("idx_messages_sender_id")
                    .table(Messages::Table)
                    .col(Messages::SenderId)
                    .to_owned(),
            )
            .await?;

        // Index on gigs.user_id for fetching gigs by owner
        manager
            .create_index(
                Index::create()
                    .name("idx_gigs_user_id")
                    .table(Gigs::Table)
                    .col(Gigs::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_contracts_gig_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_contracts_user_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_messages_contract_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_messages_sender_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_gigs_user_id").to_owned())
            .await?;

        Ok(())
    }
}

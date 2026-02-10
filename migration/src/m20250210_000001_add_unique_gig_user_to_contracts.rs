use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Contracts {
    Table,
    GigId,
    UserId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_contracts_gig_user_unique")
                    .table(Contracts::Table)
                    .col(Contracts::GigId)
                    .col(Contracts::UserId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_contracts_gig_user_unique")
                    .table(Contracts::Table)
                    .to_owned(),
            )
            .await
    }
}

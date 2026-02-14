use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Portfolios {
    Table,
    ThumbnailUrl,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Portfolios::Table)
                    .add_column(
                        ColumnDef::new(Portfolios::ThumbnailUrl)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Portfolios::Table)
                    .drop_column(Portfolios::ThumbnailUrl)
                    .to_owned(),
            )
            .await
    }
}

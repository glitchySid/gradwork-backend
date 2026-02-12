use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Gigs {
    Table,
    ThumbnailUrl,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Gigs::Table)
                    .add_column(ColumnDef::new(Gigs::ThumbnailUrl).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Gigs::Table)
                    .drop_column(Gigs::ThumbnailUrl)
                    .to_owned(),
            )
            .await
    }
}

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Identifiers for the `gigs` table and its columns.
#[derive(DeriveIden)]
enum Gigs {
    Table,
    Id,
    Title,
    Description,
    Price,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Gigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Gigs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Gigs::Title).string().not_null())
                    .col(ColumnDef::new(Gigs::Description).text().not_null())
                    .col(ColumnDef::new(Gigs::Price).double().not_null())
                    .col(
                        ColumnDef::new(Gigs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Gigs::Table).to_owned())
            .await
    }
}

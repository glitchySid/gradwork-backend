use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Identifiers for the `portfolios` table and its columns.
#[derive(DeriveIden)]
enum Portfolios {
    Table,
    Id,
    Title,
    Description,
    FreelancerId,
    Price,
    CreatedAt,
}

/// Re-declare parent table identifiers for foreign-key references.
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
                    .table(Portfolios::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Portfolios::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Portfolios::Title).string().not_null())
                    .col(ColumnDef::new(Portfolios::Description).text().not_null())
                    .col(ColumnDef::new(Portfolios::FreelancerId).uuid().not_null())
                    .col(ColumnDef::new(Portfolios::Price).double().not_null())
                    .col(
                        ColumnDef::new(Portfolios::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_portfolios_freelancer_id")
                            .from(Portfolios::Table, Portfolios::FreelancerId)
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
            .drop_table(Table::drop().table(Portfolios::Table).to_owned())
            .await
    }
}

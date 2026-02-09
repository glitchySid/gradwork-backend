use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Gigs {
    Table,
    UserId,
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
            .alter_table(
                Table::alter()
                    .table(Gigs::Table)
                    .add_column(ColumnDef::new(Gigs::UserId).uuid().not_null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_gigs_user_id")
                            .from(Gigs::Table, Gigs::UserId)
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
            .alter_table(
                Table::alter()
                    .table(Gigs::Table)
                    .drop_foreign_key("fk_gigs_user_id")
                    .drop_column(Gigs::UserId)
                    .to_owned(),
            )
            .await
    }
}

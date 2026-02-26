use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Gigs {
    Table,
    Category,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Gigs::Table)
                    .add_column(
                        ColumnDef::new(Gigs::Category)
                            .string()
                            .default("other"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("UPDATE gigs SET category = 'other' WHERE category IS NULL")
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Gigs::Table)
                    .modify_column(
                        ColumnDef::new(Gigs::Category)
                            .string()
                            .not_null()
                            .default("other"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE gigs ADD CONSTRAINT chk_gigs_category_valid CHECK (category IN ('web_development', 'mobile_development', 'data_science', 'design', 'video_editing', 'content_writing', 'other'))",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE gigs DROP CONSTRAINT IF EXISTS chk_gigs_category_valid")
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Gigs::Table)
                    .drop_column(Gigs::Category)
                    .to_owned(),
            )
            .await
    }
}

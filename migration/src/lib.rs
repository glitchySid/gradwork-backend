pub use sea_orm_migration::prelude::*;

mod m20250206_000001_create_users_table;
mod m20250206_000002_create_gigs_table;
mod m20250206_000003_create_contracts_table;
mod m20250206_000004_create_portfolios_table;
mod m20250207_000005_alter_users_for_supabase_auth;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250206_000001_create_users_table::Migration),
            Box::new(m20250206_000002_create_gigs_table::Migration),
            Box::new(m20250206_000003_create_contracts_table::Migration),
            Box::new(m20250206_000004_create_portfolios_table::Migration),
            Box::new(m20250207_000005_alter_users_for_supabase_auth::Migration),
        ]
    }
}

use sea_orm_migration::prelude::*;

mod m20220101_000001_create_mails_table;

#[allow(dead_code)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_mails_table::Migration),
        ]
    }
}

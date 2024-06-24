pub use sea_orm_migration::prelude::*;

mod m20230927_162921_create_users;
mod m20230929_081415_create_subdomains;
mod m20230929_152215_create_file;
mod m20231105_171000_create_origin;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230927_162921_create_users::Migration),
            Box::new(m20230929_081415_create_subdomains::Migration),
            Box::new(m20230929_152215_create_file::Migration),
            Box::new(m20231105_171000_create_origin::Migration),
        ]
    }
}

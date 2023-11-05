use sea_orm_migration::prelude::*;

use crate::m20230929_081415_create_subdomains::Subdomain;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Cors::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Cors::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Cors::SubdomainId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Cors::Table, Cors::SubdomainId)
                            .to(Subdomain::Table, Subdomain::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Cors::Origin).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Cors::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Cors {
    Table,
    Id,
    SubdomainId,
    Origin,
}

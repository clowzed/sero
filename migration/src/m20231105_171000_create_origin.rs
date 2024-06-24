use crate::m20230929_081415_create_subdomains::Subdomain;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Origin::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Origin::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Origin::SubdomainId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Origin::Table, Origin::SubdomainId)
                            .to(Subdomain::Table, Subdomain::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Origin::Value).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Origin::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
pub enum Origin {
    Table,
    Id,
    SubdomainId,
    Value,
}

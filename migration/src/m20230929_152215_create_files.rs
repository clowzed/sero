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
                    .table(File::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(File::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(File::SubdomainId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(File::Table, File::SubdomainId)
                            .to(Subdomain::Table, Subdomain::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(File::UserPath).string().not_null())
                    .col(
                        ColumnDef::new(File::RealPath)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum File {
    Table,
    Id,
    SubdomainId,
    UserPath,
    RealPath,
}

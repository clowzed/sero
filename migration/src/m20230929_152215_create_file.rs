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
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(File::SubdomainId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .from(File::Table, File::SubdomainId)
                            .to(Subdomain::Table, Subdomain::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(ColumnDef::new(File::UserPath).string().not_null())
                    .col(ColumnDef::new(File::RealPath).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(File::Obsolete)
                            .boolean()
                            .not_null()
                            .default(Expr::val(false)),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("file-user-path-idx")
                    .table(File::Table)
                    .col(File::UserPath)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(File::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum File {
    Table,
    Id,
    SubdomainId,
    UserPath,
    RealPath,
    Obsolete,
}

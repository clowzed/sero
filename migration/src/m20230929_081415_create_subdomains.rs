use crate::m20230927_162921_create_users::User;
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subdomain::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Subdomain::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Subdomain::OwnerId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_user_subdomain")
                            .from(Subdomain::Table, Subdomain::OwnerId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Subdomain::Enabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(Subdomain::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Subdomain::ArchivePath).string().unique_key())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("subdomain-name-idx")
                    .table(Subdomain::Table)
                    .col(Subdomain::Name)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subdomain::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Subdomain {
    Table,
    Id,
    Name,
    OwnerId,
    ArchivePath,
    Enabled,
}

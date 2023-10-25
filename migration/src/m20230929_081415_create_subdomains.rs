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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Subdomain::OwnerId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_user_subdomain")
                            .from(Subdomain::Table, Subdomain::OwnerId)
                            .to(User::Table, User::Id),
                    )
                    .col(
                        ColumnDef::new(Subdomain::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Subdomain::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Subdomain::ArchivePath).string().unique_key())
                    .to_owned(),
            )
            .await
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

//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub subdomain_id: i32,
    pub user_path: String,
    #[sea_orm(unique)]
    pub real_path: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subdomain::Entity",
        from = "Column::SubdomainId",
        to = "super::subdomain::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Subdomain,
}

impl Related<super::subdomain::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subdomain.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

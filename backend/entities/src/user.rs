//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub created: DateTime,
    pub last_login: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_source::Entity")]
    UserSource,
}

impl Related<super::user_source::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSource.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

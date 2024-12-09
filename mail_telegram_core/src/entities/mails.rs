//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "mails")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub message_id: String,
    pub timestamp: String,
    pub from: String,
    pub subject: String,
    pub sent_on_telegram: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

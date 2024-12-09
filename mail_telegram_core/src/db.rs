use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveValue, ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use sea_orm_migration::MigratorTrait;

use crate::entities::mails;
use crate::env::EnvVars;
use crate::migrator::Migrator;

#[allow(dead_code)]
pub async fn migrations_up() -> Result<(), Box<dyn std::error::Error>> {
    println!("upgrading migrations...");
    let db = Database::connect(EnvVars::database_url()).await?;
    Migrator::up(&db, None).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn migrations_down() -> Result<(), Box<dyn std::error::Error>> {
    println!("downgrading migrations...");
    let db = Database::connect(EnvVars::database_url()).await?;
    Migrator::down(&db, None).await?;
    Ok(())
}

pub type Mail = mails::Model;

pub struct NewMail {
    pub from: String,
    pub message_id: String,
    pub timestamp: String,
    pub subject: String,
}

pub async fn get_new_connection() -> Result<DatabaseConnection, DbErr> {
    Database::connect(EnvVars::database_url()).await
}

pub struct MailsDB {
    db: DatabaseConnection,
}

impl MailsDB {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_mails_by_message_ids(
        &self,
        message_ids: impl Iterator<Item = String>,
    ) -> Result<Vec<Mail>, Box<dyn std::error::Error>> {
        let mails = mails::Entity::find()
            .filter(mails::Column::MessageId.is_in(message_ids))
            .all(&self.db)
            .await?;
        Ok(mails)
    }

    pub async fn fetch_latest_timestamp(&self) -> Option<String> {
        mails::Entity::find()
            .select_only()
            .column(mails::Column::Timestamp)
            .order_by_desc(mails::Column::Timestamp)
            .one(&self.db)
            .await
            .unwrap_or(None).map(|m| m.timestamp)
    }

    pub async fn fetch_unsent_mails(&self) -> Result<Vec<Mail>, Box<dyn std::error::Error>> {
        let unsent_mails = mails::Entity::find()
            .filter(mails::Column::SentOnTelegram.eq(false))
            .order_by_asc(mails::Column::Timestamp)
            .all(&self.db)
            .await?;
        Ok(unsent_mails)
    }

    pub async fn store_new_mails(
        &self,
        new_mails: impl IntoIterator<Item = NewMail>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        mails::Entity::insert_many(new_mails.into_iter().map(|new_mail| mails::ActiveModel {
            from: ActiveValue::Set(new_mail.from),
            message_id: ActiveValue::Set(new_mail.message_id),
            subject: ActiveValue::Set(new_mail.subject),
            timestamp: ActiveValue::Set(new_mail.timestamp),
            sent_on_telegram: ActiveValue::Set(false),
            ..Default::default()
        }))
        .exec(&self.db)
        .await?;
        Ok(())
    }

    pub async fn update_mails_as_sent(
        &self,
        mails: impl Iterator<Item = Mail>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        mails::Entity::update_many()
            .col_expr(mails::Column::SentOnTelegram, Expr::value(true))
            .filter(mails::Column::Id.is_in(mails.map(|m| m.id)))
            .exec(&self.db)
            .await?;
        Ok(())
    }
}

use hyper::client::connect;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbBackend, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
};
use sea_orm_migration::MigratorTrait;
use warp::redirect::found;

use crate::entities::mails;
use crate::env::EnvVars;
use crate::migrator::Migrator;

pub async fn check_postgres() -> Result<(), DbErr> {
    let db = Database::connect(EnvVars::database_url()).await?;

    let sample_mail = mails::ActiveModel {
        from: ActiveValue::set("abc@gmail.com".to_string()),
        message_id: ActiveValue::set("some-message-123-id".to_string()),
        timestamp: ActiveValue::set("2022-01-01 00:00:00".to_string()),
        subject: ActiveValue::set("sample subject".to_string()),
        ..Default::default()
    };

    let m = sample_mail.insert(&db).await?;

    println!("inserted mail with id {}", m.id);

    Ok(())
}

pub async fn check_existing_mail(mid: &String) -> bool {
    let db = Database::connect(EnvVars::database_url()).await.expect("connection not found");

    let found_mail = mails::Entity::find()
        .filter(mails::Column::MessageId.eq(mid))
        .one(&db)
        .await
        .unwrap_or(None);

    if let Some(_) = found_mail {
        return true;
    }

    false
}

pub async fn store_new_mail(new_mail: NewMail) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::connect(EnvVars::database_url()).await.expect("connection not found");

    let inserting_mail = mails::ActiveModel {
        from: ActiveValue::Set(new_mail.from),
        message_id: ActiveValue::Set(new_mail.message_id),
        timestamp: ActiveValue::Set(new_mail.timestamp),
        subject: ActiveValue::Set(new_mail.subject),
        ..Default::default()
    };

    let _ = inserting_mail.insert(&db).await?;

    Ok(())
}

pub async fn migrations_up() -> Result<(), Box<dyn std::error::Error>> {
    println!("upgrading migrations...");
    let db = Database::connect(EnvVars::database_url()).await?;
    Migrator::up(&db, None).await?;
    Ok(())
}

pub async fn migrations_down() -> Result<(), Box<dyn std::error::Error>> {
    println!("downgrading migrations...");
    let db = Database::connect(EnvVars::database_url()).await?;
    Migrator::down(&db, None).await?;
    Ok(())
}

/// production

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

    pub async fn check_existing_mail(&self, mid: &String) -> Option<Mail> {
        let found_mail = mails::Entity::find()
            .filter(mails::Column::MessageId.eq(mid))
            .one(&self.db)
            .await
            .unwrap_or(None);

        found_mail
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
            .unwrap_or(None)
            .and_then(|m| Some(m.timestamp))
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

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        println!("running up migration now!!!!!");
        
        manager
            .create_table(
                Table::create()
                    .table(Mails::Table)
                    .if_not_exists()
                    .col(pk_auto(Mails::Id))
                    .col(string(Mails::MessageId).not_null())
                    .col(string(Mails::Timestamp).not_null())
                    .col(string(Mails::From).not_null())
                    .col(string(Mails::Subject).not_null())
                    .col(boolean(Mails::SentOnTelegram).default(false))
                    .col(date_time(Mails::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Mails::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Mails {
    Table,
    Id,
    CreatedAt,
    MessageId,
    SentOnTelegram,
    Timestamp,
    From,
    Subject
}

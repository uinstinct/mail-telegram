mod db;
mod migrator;
mod entities;
mod gmail;
mod pdf;
mod telegram;
mod env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::EnvVars::load_all_variables()?;

    let db = db::get_new_connection().await?;
    let gmail_client = gmail::get_gmail_client().await?;

    let messages = gmail::fetch_messages_from_gmail(&db, &gmail_client).await?;
    gmail::extract_and_store_mail_data(&db, &gmail_client, messages).await?;

    telegram::send_mails_in_telegram(&db).await?;
    
    Ok(())
}
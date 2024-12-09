use sea_orm::DatabaseConnection;
use teloxide::{
    prelude::Requester,
    types::{ChatId, InputFile},
    Bot,
};

use crate::{db, env::EnvVars, pdf};

fn get_bot() -> Bot {
    Bot::new(EnvVars::telegram_bot_url())
}

async fn send_message_in_telegram(bot: &Bot, mail: db::Mail) -> Result<db::Mail, Box<dyn std::error::Error>> {
    let path = pdf::get_pdf_path_by_id(&mail.message_id)?;
    let pdf_file = InputFile::file(path).file_name(mail.subject.clone());
    let _ = bot.send_document(ChatId(EnvVars::telegram_my_chat_id()), pdf_file).await?;
    Ok(mail)
}

pub async fn send_mails_in_telegram(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending mails in telegram...");
    
    let bot = get_bot();
    let mails_db = db::MailsDB::new(db.clone());

    let unsent_mails = mails_db
        .fetch_unsent_mails()
        .await?;

    let mut sent_messages  = vec![] as Vec<db::Mail>;
    sent_messages.reserve(unsent_mails.len());

    for unsent_mail in unsent_mails {
        let sent = send_message_in_telegram(&bot, unsent_mail).await?;
        sent_messages.push(sent);
    }
    
    println!("sent {} mails", sent_messages.len());

    mails_db.update_mails_as_sent(sent_messages.into_iter()).await?;

    Ok(())
}

use crate::db::{MailsDB, NewMail};
use crate::pdf::print_pdf;

use google_gmail1::hyper::{self};
use google_gmail1::oauth2::{read_authorized_user_secret, AuthorizedUserAuthenticator};
use google_gmail1::Gmail;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use sea_orm::DatabaseConnection;
use std::collections::BTreeSet;
use std::error::Error;

const GMAIL_MESSAGES_LIMIT: u32 = 30;

pub async fn get_gmail_client() -> Result<Gmail<HttpsConnector<HttpConnector>>, Box<dyn Error>> {
    let authorized_user_secret = read_authorized_user_secret("./token.json")
        .await
        .or_else(|err| Err(format!("Unable to parse gmail token secret ({})", err)))?;

    let auth = AuthorizedUserAuthenticator::builder(authorized_user_secret)
        .build()
        .await?;

    let hub = Gmail::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_only()
                .enable_http1()
                .build(),
        ),
        auth,
    );

    Ok(hub)
}

pub async fn fetch_messages_from_gmail(
    db: &DatabaseConnection,
    gmail_client: &Gmail<HttpsConnector<HttpConnector>>,
) -> Result<Vec<google_gmail1::api::Message>, Box<dyn Error>> {
    println!("fetching messages from gmail...");

    let latest_timestamp = MailsDB::new(db.clone()).fetch_latest_timestamp().await;
    let latest_timestamp_query = latest_timestamp
        .map(|ts| ts[..ts.len() - 3].to_string())
        .map(|ts| format!("is:unread after:{}", ts))
        .unwrap_or("is:unread".to_string());

    let result = gmail_client
        .users()
        .messages_list("me")
        .add_label_ids("CATEGORY_PERSONAL")
        .q(latest_timestamp_query.as_str())
        .include_spam_trash(false)
        .max_results(GMAIL_MESSAGES_LIMIT)
        .doit()
        .await?;

    let messages = result.1.messages.unwrap_or_default();

    println!("fetched {} messages", messages.len());

    Ok(messages)
}

pub struct ExtractedMailData {
    pub new_mail: NewMail,
    pub html: Vec<u8>,
}

fn extract_gmail_message_data(message: google_gmail1::api::Message) -> ExtractedMailData {
    let html = message
        .payload
        .as_ref()
        .and_then(|payload| payload.parts.as_ref())
        .and_then(|parts| parts.get(1))
        .and_then(|body_part| body_part.body.as_ref())
        .and_then(|body| body.data.clone())
        .unwrap();

    let extract_header = |header_name: &str| -> String {
        message
            .payload
            .as_ref()
            .and_then(|payload| payload.headers.as_ref())
            .and_then(|headers| {
                headers
                    .iter()
                    .find(|h| h.name.as_deref() == Some(header_name))
            })
            .and_then(|header| header.value.clone())
            .unwrap_or_default()
    };

    ExtractedMailData {
        html,
        new_mail: NewMail {
            message_id: message.id.as_ref().unwrap().clone(),
            timestamp: message.internal_date.as_ref().unwrap().to_string(),
            from: extract_header("From"),
            subject: extract_header("Subject"),
        },
    }
}

pub async fn extract_and_store_mail_data(
    db: &DatabaseConnection,
    gmail_client: &Gmail<HttpsConnector<HttpConnector>>,
    messages: Vec<google_gmail1::api::Message>,
) -> Result<(), Box<dyn Error>> {
    println!("extracting mail data for {} messages...", messages.len());

    let mails_db = MailsDB::new(db.clone());

    let message_ids = messages.iter().map(|message| message.id.clone().unwrap());

    let found_mails = mails_db.find_mails_by_message_ids(message_ids).await?;
    let found_mail_message_ids = found_mails
        .into_iter()
        .map(|m| m.message_id)
        .collect::<BTreeSet<_>>();

    // println!("found {:#?} mails", found_mail_message_ids);

    let new_messages = messages
        .iter()
        .filter(|message| !found_mail_message_ids.contains(message.id.as_ref().unwrap()));

    println!(
        "found {} new messages",
        new_messages.clone().collect::<Vec<_>>().len()
    );

    let fetched_messages = new_messages.map(|new_message| {
        gmail_client
            .users()
            .messages_get("me", new_message.id.as_ref().unwrap())
            .doit()
    });
    let fetched_messages = futures::future::join_all(fetched_messages).await;
    let fetched_messages = fetched_messages
        .into_iter()
        .filter(|m| m.is_ok())
        .map(|m| m.unwrap().1);

    let extracted_messages_data = fetched_messages
        .map(extract_gmail_message_data)
        .collect::<Vec<_>>();

    if extracted_messages_data.is_empty() {
        println!("no new messages");
        return Ok(());
    }

    println!("extracted {} message(s)", extracted_messages_data.len());

    for extracted_message_data in &extracted_messages_data {
        print_pdf(extracted_message_data).await?;
    }

    MailsDB::new(db.clone())
        .store_new_mails(extracted_messages_data.into_iter().map(|m| m.new_mail))
        .await?;

    Ok(())
}

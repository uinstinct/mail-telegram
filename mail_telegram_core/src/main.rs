use google_gmail1::hyper::{self};
use google_gmail1::oauth2::{
    read_authorized_user_secret, AuthorizedUserAuthenticator
};
use google_gmail1::Gmail;

use std::error::Error;

async fn get_gmail_client() -> Result<Gmail<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>, Box<dyn Error>> {
    let authorized_user_secret = read_authorized_user_secret("./token.json").await?;
    
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let hub = get_gmail_client().await?;

    let result = hub
        .users()
        .messages_list("me")
        .q("is:unread")
        .max_results(500)
        .doit()
        .await?;

    let messages = result.1.messages.unwrap_or_default();

    for message in messages {
        let msg_id = message.id.unwrap();
        let msg = hub.users().messages_get("me", &msg_id).doit().await?;

        if let Some(headers) = msg.1.payload.and_then(|p| p.headers) {
            for header in headers {
                if header.name.unwrap_or_default() == "Subject" {
                    println!("Subject: {}", header.value.unwrap_or_default());
                    break;
                }
            }
        }
    }

    Ok(())
}

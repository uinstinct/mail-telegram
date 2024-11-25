use google_gmail1::hyper::{self};
use google_gmail1::oauth2::{
    self, read_authorized_user_secret, AccessTokenAuthenticator, ApplicationSecret, AuthorizedUserAuthenticator, InstalledFlowAuthenticator
};
use google_gmail1::Gmail;

use std::error::Error;
use std::fs;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load the application secret from the credentials JSON file
    let secret: ApplicationSecret = oauth2::read_application_secret("credentials.json")
        .await
        .expect("expected credentials.json to be present");

    // Set up an OAuth2.0 authenticator
    // let auth = Installedflow::new(
    //     oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    //     secret,
    // );

    // let mut file = fs::File::open("./token.json").expect("expected token.json to be present");

    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;

    // let json: serde_json::Value = serde_json::from_str(&contents)?;

    // let access_token = json
    //     .get("token")
    //     .and_then(|token| token.get("access_token"))
    //     .and_then(|value| Some(value.to_string()))
    //     .expect("access_token key not found");

    // println!("access_token: {}", access_token);

    let authorized_user_secret = read_authorized_user_secret("./token.json").await?;
    
    let auth = AuthorizedUserAuthenticator::builder(authorized_user_secret)
        .build()
        .await?;

    // Create the Gmail API hub
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

    let result = hub
        .users()
        .messages_list("me")
        .q("is:unread")
        .max_results(500)
        .doit()
        .await?;

    let messages = result.1.messages.unwrap_or_default();

    // Process each message
    for message in messages {
        let msg_id = message.id.unwrap();
        let msg = hub.users().messages_get("me", &msg_id).doit().await?;

        // Extract subject from headers
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

use db::{check_existing_mail, store_new_mail, NewMail};
use entities::mails;
use env::EnvVars;
use google_gmail1::hyper::{self};
use google_gmail1::oauth2::{read_authorized_user_secret, AuthorizedUserAuthenticator};
use google_gmail1::{api, Gmail};

use headless_chrome::protocol::cdp::Page;
use headless_chrome::Browser;
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, InputFile};
use std::error::Error;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;


use teloxide::Bot;

use rand::{distributions::Alphanumeric, Rng};

mod db;
mod migrator;
mod entities;
mod gmail;
mod pdf;
mod telegram;
mod env;

fn get_random_string() -> String {
    // Length of the random string
    let string_length = 4;

    // Generate a random string
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric) // Sample random alphanumeric characters
        .take(string_length) // Take the desired number of characters
        .map(char::from) // Convert each byte to a char
        .collect(); // Collect into a String

    random_string
}

async fn get_gmail_client(
) -> Result<Gmail<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>, Box<dyn Error>> {
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

fn get_html_data(msg: api::Message) -> Option<Vec<u8>> {
    msg.payload?.parts?.get(1)?.clone().body?.data
}

async fn extract_message_data(msg: api::Message) -> Result<(), Box<dyn Error>> {
    // if let Some(encoded_html_data) = get_html_data(msg.clone()) {
    //     let html = std::str::from_utf8(&encoded_html_data).expect("invalid utf-8 format html data");
    //     std::fs::write("index.html", html)?;
    //     spin_up_html_page().await?;
    // }

    let extract_header = |header_name: &str| -> String {
        msg.payload
            .as_ref()
            .and_then(|payload| payload.headers.as_ref())
            .and_then(|headers| headers.iter().find(|h| h.name.as_deref() == Some(header_name)))
            .and_then(|header| header.value.clone())
            .unwrap_or_default()
    };

    store_new_mail(NewMail {
        from: extract_header("From"),
        message_id: msg.id.unwrap(),
        timestamp: msg.internal_date.unwrap().to_string(),
        subject: msg.payload.unwrap().headers.unwrap().iter().find(|h| h.name == Some(String::from("Subject"))).unwrap().value.clone().unwrap(),
    }).await?;
    
    Ok(())
}

async fn run_server(port: u16, mut shutdown_rx: oneshot::Receiver<()>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to bind to port");

    println!("Server running on port {}", port);

    let html_content = std::fs::read_to_string("index.html").expect("Failed to read HTML file");

    loop {
        tokio::select! {
            // Accept incoming connections
            Ok((mut socket, addr)) = listener.accept() => {
                println!("New connection from {}", addr);

                // Clone the HTML content for the task
                let html_content = html_content.clone();

                // Handle the connection in a new task
                tokio::spawn(async move {
                    // Simulate handling an HTTP GET request (basic implementation)
                    let mut buffer = [0; 1024];
                    match socket.read(&mut buffer).await {
                        Ok(_) => {
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                                html_content.len(),
                                html_content
                            );
                            socket.write_all(response.as_bytes()).await.unwrap();
                        }
                        Err(e) => println!("Failed to read from socket: {}", e),
                    }
                });
            },

            // Listen for the shutdown signal
            _ = &mut shutdown_rx => {
                println!("Shutdown signal received.");
                break;
            }
        }
    }

    println!("Server shutting down...");
}

async fn spin_up_html_page() -> Result<(), Box<dyn Error>> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Spawn the server in a separate task
    let server_task = tokio::spawn(run_server(3030, shutdown_rx));

    print_pdf()?;

    shutdown_tx
        .send(())
        .expect("Failed to send shutdown signal");

    server_task.await.expect("server task panicked");

    Ok(())
}

fn print_pdf() -> Result<(), Box<dyn Error>> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;

    tab.navigate_to("http://localhost:3030/")?;
    tab.wait_until_navigated()?;

    let pdfdata = tab.print_to_pdf(None)?;

    let pngdata = tab.capture_screenshot(
        Page::CaptureScreenshotFormatOption::Png,
        Some(100),
        None,
        true,
    )?;

    let random_str = get_random_string();
    std::fs::write(format!("screenshot{}.png", random_str), pngdata)?;
    std::fs::write(format!("mail{}.pdf", random_str), pdfdata)?;

    Ok(())
}

async fn send_to_telegram() -> io::Result<()> {
    let current_dir = std::env::current_dir()?;
    println!("Current directory: {:?}", current_dir);

    println!("PDF files in the current directory:");
    let pdf_files: Vec<_> = std::fs::read_dir(&current_dir)?
        .filter_map(Result::ok) // Ignore errors
        .map(|entry| entry.path()) // Extract paths
        .filter(|path| path.extension().map_or(false, |ext| ext == "pdf")) // Filter PDFs
        .collect();
    
    let bot = Bot::new(EnvVars::telegram_bot_url());
    for pdf_file in pdf_files {
        println!("Sending PDF file: {}", pdf_file.display());
        let input_file = InputFile::file(pdf_file);
        bot.send_document(ChatId(EnvVars::telegram_my_chat_id()), input_file).await.expect("could not send the telegram message");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    EnvVars::load_all_variables()?;

    let db = db::get_new_connection().await?;
    let gmail_client = gmail::get_gmail_client().await?;

    let messages = gmail::fetch_messages_from_gmail(&db, &gmail_client).await?;
    gmail::extract_and_store_mail_data(&db, &gmail_client, messages).await?;

    telegram::send_mails_in_telegram(&db).await?;
    
    Ok(())
}
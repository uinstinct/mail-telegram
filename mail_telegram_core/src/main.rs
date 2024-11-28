use google_gmail1::hyper::{self};
use google_gmail1::oauth2::{read_authorized_user_secret, AuthorizedUserAuthenticator};
use google_gmail1::{api, Gmail};

use headless_chrome::protocol::cdp::Page;
use headless_chrome::Browser;
use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, serialize};
use markup5ever_rcdom::{RcDom, SerializableHandle};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use warp::Filter;

use rand::{distributions::Alphanumeric, Rng};

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
    if let Some(encoded_html_data) = get_html_data(msg) {
        let html = std::str::from_utf8(&encoded_html_data).expect("invalid utf-8 format html data");
        std::fs::write("index.html", html)?;
        spin_up_html_page().await?;
    }

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

    for (index, message) in messages.into_iter().enumerate() {
        let msg_id = message.id.unwrap();
        let msg = hub.users().messages_get("me", &msg_id).doit().await?;

        extract_message_data(msg.1).await?;

        break;
    }

    Ok(())
}

use std::sync::Arc;

use headless_chrome::Browser;
use warp::Filter;

use tokio::sync::{oneshot, Notify};

use crate::gmail::ExtractedMailData;

const TEMP_MAILS_DIR: &str = "temp-mails";

struct ServerHandle {
    shutdown_tx: oneshot::Sender<()>,
    notify: Arc<Notify>,
}
impl ServerHandle {
    fn shutdown(self) {
        self.shutdown_tx
            .send(())
            .expect("Failed to shutdown the server!");
    }
}

fn start_server_with_html_content(html_data: Vec<u8>) -> ServerHandle {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    let html_data = String::from_utf8(html_data).expect("invalid utf-8 format html data");
    let route = warp::path::end().map(move || warp::reply::html(html_data.clone()));

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (_, server) =
        warp::serve(route).bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
            shutdown_rx.await.ok();
            notify_clone.notify_one();
        });

    tokio::spawn(server);

    ServerHandle {
        shutdown_tx,
        notify,
    }
}

async fn store_pdf_into_folder(message_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;

    tab.navigate_to("http://127.0.0.1:3030/")?;
    tab.wait_until_navigated()?;

    let pdfdata = tab.print_to_pdf(None)?;

    let mails_dir = std::path::Path::new(TEMP_MAILS_DIR);
    if !mails_dir.exists() {
        std::fs::create_dir(mails_dir)?;
    }
    std::fs::write(mails_dir.join(format!("mail-{}.pdf", message_id)), pdfdata)?;

    Ok(())
}

pub async fn print_pdf(
    extracted_mail_data: &ExtractedMailData,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("printing pdf for {}", extracted_mail_data.new_mail.message_id);
    
    let server_handle = start_server_with_html_content(extracted_mail_data.html.clone());
    store_pdf_into_folder(extracted_mail_data.new_mail.message_id.clone()).await?;
    let notify = server_handle.notify.clone();
    server_handle.shutdown();
    notify.notified().await;
    Ok(())
}

pub fn get_pdf_path_by_id(id: &String) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let mail_path = std::path::PathBuf::from(TEMP_MAILS_DIR).join(format!("{}-{}.pdf", "mail", id).as_str());
    mail_path.try_exists()?;
    Ok(mail_path)
}
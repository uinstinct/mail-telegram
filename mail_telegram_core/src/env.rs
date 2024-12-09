
pub struct EnvVars {}

impl EnvVars {
    fn check_all_variables() -> Result<(), Box<dyn std::error::Error>> {
        std::env::var("DATABASE_URL")?;
        std::env::var("TELEGRAM_BOT_TOKEN")?;
        std::env::var("TELEGRAM_MY_CHAT_ID")?;
        std::env::var("GMAIL_TOKEN_JSON")?;
        Ok(())
    }
    
    pub fn load_all_variables() -> Result<(), Box<dyn std::error::Error>> {
        EnvVars::check_all_variables().unwrap_or_else(|_| {
            dotenvy::dotenv().ok();
        });
        EnvVars::check_all_variables()?;
        EnvVars::write_token_json_into_file();
        Ok(())
    }
    
    pub fn database_url() -> String {
        std::env::var("DATABASE_URL").unwrap()
    }

    pub fn telegram_bot_url() -> String {
        std::env::var("TELEGRAM_BOT_TOKEN").unwrap()
    }

    pub fn telegram_my_chat_id() -> i64 {
        std::env::var("TELEGRAM_MY_CHAT_ID").unwrap().parse::<i64>().unwrap()
    }

    fn write_token_json_into_file() {
        std::fs::write("./token.json", std::env::var("GMAIL_TOKEN_JSON").unwrap_or_else(|_| {
            println!("GMAIL_TOKEN_JSON is not set");
            "".to_string()
        })).unwrap_or_else(|_| {
            println!("Failed to write token.json");
        });
    }
}
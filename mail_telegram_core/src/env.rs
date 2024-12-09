
pub struct EnvVars {}

impl EnvVars {
    pub fn load_all_variables() -> Result<(), Box<dyn std::error::Error>> {
        dotenvy::dotenv()?;
        std::env::var("DATABASE_URL")?;
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
}
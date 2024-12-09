
pub struct EnvVars {}

impl EnvVars {
    fn check_all_variables() -> Result<(), Box<dyn std::error::Error>> {
        std::env::var("DATABASE_URL").or_else(|_| Err("DATABASE_URL is not set"))?;
        std::env::var("TELEGRAM_BOT_TOKEN").or_else(|_| Err("TELEGRAM_BOT_TOKEN is not set"))?;
        std::env::var("TELEGRAM_MY_CHAT_ID").or_else(|_| Err("TELEGRAM_MY_CHAT_ID is not set"))?;
        std::env::var("GMAIL_TOKEN_JSON").or_else(|_| Err("GMAIL_TOKEN_JSON is not set"))?;
        println!("database url length was {}", std::env::var("DATABASE_URL").unwrap().len());
        Ok(())
    }
    
    pub fn load_all_variables() -> Result<(), Box<dyn std::error::Error>> {
        // as to avoid writing all these variables in github workflow file, add a way to extract all these variables from a json string and fallback to extracting them manually if not all the variables are present in that json
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
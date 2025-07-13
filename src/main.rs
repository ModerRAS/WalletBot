mod bot;
mod database;
mod parser;
mod calculator;
mod config;
mod error;
mod retry;
mod utils;

use log::info;
use anyhow::Result;
use dotenv::dotenv;

use config::Settings;
use bot::MessageHandler;
use database::DatabaseOperations;
use utils::Logger;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenv().ok();
    
    // 初始化日志
    env_logger::init();
    
    Logger::log_operation_start("WalletBot", "Initializing application");
    
    // 加载配置
    let settings = match Settings::new() {
        Ok(s) => {
            Logger::log_operation_success("Configuration", "Settings loaded successfully");
            s
        }
        Err(e) => {
            Logger::log_operation_failure("Configuration", &e.to_string());
            return Err(e);
        }
    };
    
    // 验证配置
    if let Err(e) = settings.validate() {
        Logger::log_operation_failure("Configuration validation", &e.to_string());
        return Err(e);
    }
    
    // 初始化数据库
    let db = match DatabaseOperations::new(&settings.database_url).await {
        Ok(db) => {
            Logger::log_operation_success("Database", "Database initialized successfully");
            db
        }
        Err(e) => {
            Logger::log_operation_failure("Database", &e.to_string());
            return Err(e.into());
        }
    };
    
    // 初始化消息处理器
    let _message_handler = MessageHandler::new(db);
    Logger::log_operation_success("MessageHandler", "Handler initialized successfully");
    
    // 创建Bot实例
    let _bot = teloxide::Bot::new(&settings.telegram_bot_token);
    Logger::log_operation_success("TelegramBot", "Bot instance created successfully");
    
    info!("🤖 WalletBot initialized successfully!");
    info!("📊 Configuration:");
    info!("  - Database: {}", settings.database_url);
    info!("  - Bot Name: {}", settings.bot_name);
    info!("  - Max Retry Attempts: {}", settings.max_retry_attempts);
    info!("  - Processing Timeout: {}s", settings.processing_timeout);
    
    // 暂时只进行初始化，不启动消息处理循环
    info!("🔧 Bot initialization completed. To start message processing, add the message handling loop.");
    info!("💡 Next steps:");
    info!("  1. Set TELEGRAM_BOT_TOKEN in .env file");
    info!("  2. Test with actual Telegram messages");
    info!("  3. Monitor logs for transaction processing");
    
    Ok(())
}

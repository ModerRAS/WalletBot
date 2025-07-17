mod bot;
mod calculator;
mod config;
mod database;
mod error;
mod parser;
mod retry;
mod utils;

use anyhow::Result;
use dotenv::dotenv;
use log::info;

use bot::{start_bot, MessageHandler};
use config::Settings;
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
            return Err(e);
        }
    };

    // 初始化消息处理器
    let message_handler = MessageHandler::new(db);
    Logger::log_operation_success("MessageHandler", "Handler initialized successfully");

    info!("🤖 WalletBot initialized successfully!");
    info!("📊 Configuration:");
    info!("  - Database: {}", settings.database_url);
    info!("  - Bot Name: {}", settings.bot_name);
    info!("  - Max Retry Attempts: {}", settings.max_retry_attempts);
    info!("  - Processing Timeout: {}s", settings.processing_timeout);

    // 启动机器人
    info!("🚀 Starting WalletBot...");
    match start_bot(&settings.telegram_bot_token, message_handler).await {
        Ok(()) => {
            Logger::log_operation_success("WalletBot", "Bot stopped gracefully");
        }
        Err(e) => {
            Logger::log_operation_failure("WalletBot", &e.to_string());
            return Err(e);
        }
    }

    Ok(())
}

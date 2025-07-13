mod bot;
mod database;
mod parser;
mod calculator;
mod config;

use log::info;
use anyhow::Result;
use dotenv::dotenv;
use teloxide::prelude::*;

use config::Settings;
use bot::MessageHandler;
use database::DatabaseOperations;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenv().ok();
    
    // 初始化日志
    env_logger::init();
    
    info!("Starting WalletBot...");
    
    // 加载配置
    let settings = Settings::new()?;
    
    // 初始化数据库
    let db = DatabaseOperations::new(&settings.database_url).await?;
    
    // 初始化消息处理器
    let message_handler = MessageHandler::new(db);
    
    // 创建Bot实例
    let bot = Bot::new(&settings.telegram_bot_token);
    
    info!("Bot started successfully!");
    
    // 启动消息处理循环
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let handler = message_handler.clone();
        async move {
            if let Err(e) = handler.handle_message(&bot, &msg).await {
                log::error!("Error handling message: {}", e);
            }
            Ok(())
        }
    })
    .await;
    
    Ok(())
}

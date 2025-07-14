// 公开内部模块以便测试
pub mod bot;
pub mod database;
pub mod parser;
pub mod calculator;
pub mod config;
pub mod error;
pub mod retry;
pub mod utils;

// 重新导出常用的类型和结构体
pub use bot::{MessageHandler, Commands};
pub use database::{DatabaseOperations, models};
pub use parser::MessageParser;
pub use calculator::BalanceCalculator;
pub use config::Settings;
pub use error::WalletBotError; 
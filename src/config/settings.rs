use std::env;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub telegram_bot_token: String,
    pub database_url: String,
    pub bot_name: String,
    pub target_channel_id: Option<i64>,
    pub max_retry_attempts: u32,
    pub processing_timeout: u64,
    pub backup_interval: u64,
    pub backup_retention_days: u32,
    pub log_level: String,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| anyhow!("TELEGRAM_BOT_TOKEN must be set"))?;
        
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "wallet_bot.db".to_string());
        
        let bot_name = env::var("BOT_NAME")
            .unwrap_or_else(|_| "WalletBot".to_string());
        
        let target_channel_id = env::var("TARGET_CHANNEL_ID")
            .ok()
            .and_then(|id| id.parse::<i64>().ok());
        
        let max_retry_attempts = env::var("MAX_RETRY_ATTEMPTS")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u32>()
            .unwrap_or(3);
        
        let processing_timeout = env::var("PROCESSING_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .unwrap_or(30);
        
        let backup_interval = env::var("BACKUP_INTERVAL")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .unwrap_or(3600);
        
        let backup_retention_days = env::var("BACKUP_RETENTION_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse::<u32>()
            .unwrap_or(7);
        
        let log_level = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string());
        
        Ok(Settings {
            telegram_bot_token,
            database_url,
            bot_name,
            target_channel_id,
            max_retry_attempts,
            processing_timeout,
            backup_interval,
            backup_retention_days,
            log_level,
        })
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.telegram_bot_token.is_empty() {
            return Err(anyhow!("Telegram bot token cannot be empty"));
        }
        
        if self.database_url.is_empty() {
            return Err(anyhow!("Database URL cannot be empty"));
        }
        
        if self.max_retry_attempts == 0 {
            return Err(anyhow!("Max retry attempts must be greater than 0"));
        }
        
        if self.processing_timeout == 0 {
            return Err(anyhow!("Processing timeout must be greater than 0"));
        }
        
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            telegram_bot_token: String::new(),
            database_url: "wallet_bot.db".to_string(),
            bot_name: "WalletBot".to_string(),
            target_channel_id: None,
            max_retry_attempts: 3,
            processing_timeout: 30,
            backup_interval: 3600,
            backup_retention_days: 7,
            log_level: "info".to_string(),
        }
    }
} 
use log::{info, warn, error};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use crate::error::Result;

/// 日志记录工具
pub struct Logger;

impl Logger {
    pub fn log_operation_start(operation: &str, details: &str) {
        info!("🚀 Starting {}: {}", operation, details);
    }
    
    pub fn log_operation_success(operation: &str, details: &str) {
        info!("✅ {} completed successfully: {}", operation, details);
    }
    
    pub fn log_operation_failure(operation: &str, error: &str) {
        error!("❌ {} failed: {}", operation, error);
    }
    
    pub fn log_wallet_transaction(
        wallet_name: &str,
        transaction_type: &str,
        amount: f64,
        old_balance: f64,
        new_balance: f64,
    ) {
        info!(
            "💰 Wallet Transaction: {} | {} {:.2}元 | {} → {:.2}元",
            wallet_name, transaction_type, amount, old_balance, new_balance
        );
    }
    
    pub fn log_balance_update(
        wallet_name: &str,
        old_balance: f64,
        new_balance: f64,
        source: &str,
    ) {
        info!(
            "🔄 Balance Update: {} | {:.2}元 → {:.2}元 ({})",
            wallet_name, old_balance, new_balance, source
        );
    }
    
    pub fn log_message_processed(message_id: i64, chat_id: i64, wallet_name: &str) {
        info!(
            "📝 Message Processed: ID={} Chat={} Wallet={}",
            message_id, chat_id, wallet_name
        );
    }
}

/// 格式化工具
pub struct Formatter;

impl Formatter {
    /// 格式化金额显示
    pub fn format_amount(amount: f64) -> String {
        format!("{:.2}元", amount)
    }
    
    /// 格式化百分比变化
    pub fn format_balance_change(old_balance: f64, new_balance: f64) -> String {
        if old_balance == 0.0 {
            return "初始设置".to_string();
        }
        
        let change = new_balance - old_balance;
        let percentage = (change / old_balance.abs()) * 100.0;
        
        if change > 0.0 {
            format!("+{:.2}元 (+{:.1}%)", change, percentage)
        } else {
            format!("{:.2}元 ({:.1}%)", change, percentage)
        }
    }
    
    /// 格式化时间戳
    pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
        timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }
}

/// 验证工具
pub struct Validator;

impl Validator {
    /// 验证钱包名称
    pub fn is_valid_wallet_name(name: &str) -> bool {
        !name.is_empty() && name.len() <= 100 && !name.contains('\n')
    }
    
    /// 验证金额
    pub fn is_valid_amount(amount: f64) -> bool {
        amount >= 0.0 && amount <= 999_999_999.99 && !amount.is_nan() && !amount.is_infinite()
    }
    
    /// 验证月份
    pub fn is_valid_month(month: &str) -> bool {
        if let Ok(m) = month.parse::<u32>() {
            m >= 1 && m <= 12
        } else {
            false
        }
    }
    
    /// 验证年份
    pub fn is_valid_year(year: &str) -> bool {
        if let Ok(y) = year.parse::<u32>() {
            y >= 2000 && y <= 2100
        } else {
            false
        }
    }
}

/// 文件工具
pub struct FileUtils;

impl FileUtils {
    /// 确保目录存在
    pub fn ensure_dir_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
            info!("Created directory: {}", path.display());
        }
        Ok(())
    }
    
    /// 备份文件
    pub fn backup_file(source: &Path, backup_dir: &Path) -> Result<()> {
        if !source.exists() {
            warn!("Source file does not exist: {}", source.display());
            return Ok(());
        }
        
        Self::ensure_dir_exists(backup_dir)?;
        
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = source.file_name()
            .ok_or_else(|| crate::error::WalletBotError::parser_error("Invalid source filename"))?;
        
        let backup_path = backup_dir.join(format!("{}_{}", timestamp, filename.to_string_lossy()));
        
        fs::copy(source, &backup_path)?;
        info!("Backed up {} to {}", source.display(), backup_path.display());
        
        Ok(())
    }
    
    /// 清理旧备份文件
    pub fn cleanup_old_backups(backup_dir: &Path, retention_days: u32) -> Result<()> {
        if !backup_dir.exists() {
            return Ok(());
        }
        
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut deleted_count = 0;
        
        for entry in fs::read_dir(backup_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            if let Ok(created) = metadata.created() {
                let created_dt: DateTime<Utc> = created.into();
                
                if created_dt < cutoff {
                    if let Err(e) = fs::remove_file(entry.path()) {
                        warn!("Failed to delete old backup {}: {}", entry.path().display(), e);
                    } else {
                        deleted_count += 1;
                    }
                }
            }
        }
        
        if deleted_count > 0 {
            info!("Cleaned up {} old backup files", deleted_count);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_amount() {
        assert_eq!(Formatter::format_amount(1000.0), "1000.00元");
        assert_eq!(Formatter::format_amount(1000.5), "1000.50元");
    }

    #[test]
    fn test_format_balance_change() {
        assert_eq!(Formatter::format_balance_change(0.0, 1000.0), "初始设置");
        assert_eq!(Formatter::format_balance_change(1000.0, 1100.0), "+100.00元 (+10.0%)");
        assert_eq!(Formatter::format_balance_change(1000.0, 900.0), "-100.00元 (-10.0%)");
    }

    #[test]
    fn test_validators() {
        // 钱包名称验证
        assert!(Validator::is_valid_wallet_name("测试钱包"));
        assert!(!Validator::is_valid_wallet_name(""));
        assert!(!Validator::is_valid_wallet_name("钱包\n名称"));
        
        // 金额验证
        assert!(Validator::is_valid_amount(1000.0));
        assert!(Validator::is_valid_amount(0.0));
        assert!(!Validator::is_valid_amount(-100.0));
        assert!(!Validator::is_valid_amount(f64::NAN));
        
        // 月份验证
        assert!(Validator::is_valid_month("7"));
        assert!(Validator::is_valid_month("12"));
        assert!(!Validator::is_valid_month("0"));
        assert!(!Validator::is_valid_month("13"));
        assert!(!Validator::is_valid_month("abc"));
        
        // 年份验证
        assert!(Validator::is_valid_year("2025"));
        assert!(!Validator::is_valid_year("1999"));
        assert!(!Validator::is_valid_year("2101"));
        assert!(!Validator::is_valid_year("abc"));
    }
} 
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Option<i64>,
    pub name: String,
    pub current_balance: f64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Option<i64>,
    pub wallet_id: i64,
    pub transaction_type: String, // "出账" 或 "入账"
    pub amount: f64,
    pub month: String,
    pub year: String,
    pub message_id: Option<i64>,
    pub chat_id: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<i64>,
    pub message_id: i64,
    pub chat_id: i64,
    pub wallet_id: i64,
    pub has_total: bool,
    pub processed: bool,
    pub original_balance: Option<f64>, // 消息编辑前的余额
    pub new_balance: Option<f64>,      // 消息编辑后的余额
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub wallet_name: String,
    pub transaction_type: String,
    pub amount: f64,
    pub month: String,
    pub year: String,
    pub total_amount: Option<f64>, // 解析出的总额（如果有）
    pub original_text: String,
}

#[derive(Debug, Clone)]
pub struct BalanceUpdate {
    pub wallet_name: String,
    pub old_balance: f64,
    pub new_balance: f64,
    pub source: BalanceUpdateSource,
    pub message_id: Option<i64>,
    pub chat_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum BalanceUpdateSource {
    Transaction,    // 从交易计算
    ManualEdit,     // 从手动编辑的总额
    Initial,        // 初始设置
} 
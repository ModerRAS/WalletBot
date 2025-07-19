use crate::database::models::{BalanceUpdate, BalanceUpdateSource};
use crate::database::operations::DatabaseOperations;
use anyhow::Result;
use log::{debug, info, warn};

#[derive(Clone, Debug)]
pub struct BalanceCalculator {
    db: DatabaseOperations,
}

impl BalanceCalculator {
    pub fn new(db: DatabaseOperations) -> Self {
        Self { db }
    }

    /// 计算基于交易的新余额
    pub async fn calculate_transaction_balance(
        &self,
        chat_id: i64,
        wallet_name: &str,
        transaction_type: &str,
        amount: f64,
        _month: &str,
        _year: &str,
    ) -> Result<f64> {
        debug!("💰 Starting transaction balance calculation");
        debug!("   ├─ Chat ID: {}", chat_id);
        debug!("   ├─ Wallet: {}", wallet_name);
        debug!("   ├─ Transaction type: {}", transaction_type);
        debug!("   ├─ Amount: {}", amount);

        // 获取或创建钱包
        debug!("🗄️ Getting or creating wallet for chat {}: {}", chat_id, wallet_name);
        let wallet = self.db.get_or_create_wallet(chat_id, wallet_name).await?;

        // 获取当前余额
        let current_balance = wallet.current_balance;
        debug!("💵 Current balance for {}: {}", wallet_name, current_balance);

        // 计算新余额
        let new_balance = match transaction_type {
            "出账" => {
                debug!("➖ Calculating outgoing transaction: {} - {} = {}", current_balance, amount, current_balance - amount);
                current_balance - amount
            },
            "入账" => {
                debug!("➕ Calculating incoming transaction: {} + {} = {}", current_balance, amount, current_balance + amount);
                current_balance + amount
            },
            _ => {
                warn!("⚠️ Unknown transaction type: {}", transaction_type);
                current_balance
            }
        };

        info!("✅ Transaction balance calculated: {} {} → {}", wallet_name, current_balance, new_balance);
        Ok(new_balance)
    }

    /// 从手动编辑的总额更新余额
    pub async fn update_from_manual_total(
        &self,
        chat_id: i64,
        wallet_name: &str,
        total_amount: f64,
        _message_id: Option<i64>,
    ) -> Result<BalanceUpdate> {
        debug!("📝 Starting manual total update");
        debug!("   ├─ Wallet: {}", wallet_name);
        debug!("   ├─ Total amount: {}", total_amount);

        // 获取或创建钱包
        debug!("🗄️ Getting wallet for manual update: {} in chat {}", wallet_name, chat_id);
        let wallet = self.db.get_or_create_wallet(chat_id, wallet_name).await?;
        let old_balance = wallet.current_balance;
        debug!("💵 Current balance: {} -> {}", old_balance, total_amount);

        // 更新钱包余额
        debug!("🔄 Updating wallet balance...");
        self.db
            .update_wallet_balance(chat_id, wallet_name, total_amount)
            .await?;

        info!("✅ Manual balance update completed: {} {} → {}", wallet_name, old_balance, total_amount);

        Ok(BalanceUpdate {
            wallet_name: wallet_name.to_string(),
            old_balance,
            new_balance: total_amount,
            source: BalanceUpdateSource::ManualEdit,
            message_id: _message_id,
            chat_id: Some(chat_id),
        })
    }

    /// 智能余额计算：优先使用总额，否则计算交易余额
    #[allow(clippy::too_many_arguments)]
    pub async fn smart_calculate_balance(
        &self,
        chat_id: i64,
        wallet_name: &str,
        transaction_type: &str,
        amount: f64,
        month: &str,
        year: &str,
        total_amount: Option<f64>,
        message_id: Option<i64>,
    ) -> Result<BalanceUpdate> {
        debug!("🧮 Starting smart balance calculation");
        debug!("   ├─ Chat ID: {}", chat_id);
        debug!("   ├─ Wallet: {}", wallet_name);
        debug!("   ├─ Transaction type: {}", transaction_type);
        debug!("   ├─ Amount: {}", amount);
        debug!("   ├─ Month: {}", month);
        debug!("   ├─ Year: {}", year);
        debug!("   ├─ Total amount: {:?}", total_amount);
        debug!("   ├─ Message ID: {:?}", message_id);

        match total_amount {
            Some(total) => {
                debug!("📊 Using manual total for calculation: {}", total);
                self.update_from_manual_total(chat_id, wallet_name, total, message_id)
                    .await
            }
            None => {
                debug!("💰 Using transaction-based calculation");
                // 如果没有总额，基于交易计算
                debug!("🗄️ Getting wallet for transaction calculation: {} in chat {}", wallet_name, chat_id);
                let wallet = self.db.get_or_create_wallet(chat_id, wallet_name).await?;
                let old_balance = wallet.current_balance;
                debug!("💵 Current balance: {}", old_balance);

                let new_balance = self
                    .calculate_transaction_balance(
                        chat_id,
                        wallet_name,
                        transaction_type,
                        amount,
                        month,
                        year,
                    )
                    .await?;

                debug!("🔄 Updating wallet balance from {} to {}", old_balance, new_balance);
                // 更新钱包余额
                self.db
                    .update_wallet_balance(chat_id, wallet_name, new_balance)
                    .await?;

                debug!("✅ Transaction-based calculation completed");
                Ok(BalanceUpdate {
                    wallet_name: wallet_name.to_string(),
                    old_balance,
                    new_balance,
                    source: BalanceUpdateSource::Transaction,
                    message_id,
                    chat_id: Some(chat_id),
                })
            }
        }
    }

    /// 获取最新的余额信息
    #[allow(dead_code)]
    pub async fn get_latest_balance(
        &self,
        chat_id: i64,
        wallet_name: &str,
        month: &str,
        year: &str,
    ) -> Result<f64> {
        self.db
            .get_latest_balance(chat_id, wallet_name, month, year)
            .await
    }

    /// 检查余额是否需要调整
    #[allow(dead_code)]
    pub async fn should_adjust_balance(
        &self,
        _wallet_name: &str,
        current_total: f64,
        calculated_total: f64,
    ) -> bool {
        let tolerance = 0.01; // 1分的容差
        (current_total - calculated_total).abs() > tolerance
    }

    /// 生成余额调整记录
    #[allow(dead_code)]
    pub async fn create_balance_adjustment(
        &self,
        wallet_name: &str,
        old_balance: f64,
        new_balance: f64,
        reason: &str,
        _message_id: Option<i64>,
        _chat_id: Option<i64>,
    ) -> Result<()> {
        info!(
            "Creating balance adjustment for {wallet_name}: {old_balance} -> {new_balance} ({reason})"
        );

        // 这里可以添加审计日志逻辑
        // 比如记录到专门的 balance_adjustments 表

        Ok(())
    }
}

// Tests will be added later

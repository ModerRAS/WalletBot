use teloxide::{Bot, types::Message, RequestError, requests::Requester};
use crate::parser::message::MessageParser;
use crate::calculator::balance::BalanceCalculator;
use crate::database::operations::DatabaseOperations;
use crate::database::models::BalanceUpdateSource;
use log::{debug, info, warn, error};
use anyhow::Result;

#[derive(Clone)]
pub struct MessageHandler {
    parser: MessageParser,
    calculator: BalanceCalculator,
    db: DatabaseOperations,
}

impl MessageHandler {
    pub fn new(db: DatabaseOperations) -> Self {
        let calculator = BalanceCalculator::new(db.clone());
        Self {
            parser: MessageParser::new(),
            calculator,
            db,
        }
    }

    pub async fn handle_message(
        &self,
        bot: &Bot,
        message: &Message,
    ) -> Result<(), RequestError> {
        // 只处理文本消息
        if let Some(text) = message.text() {
            debug!("Processing message: {}", text);
            
            // 检查是否是钱包相关消息
            if !self.parser.is_wallet_message(text) {
                debug!("Not a wallet message, skipping");
                return Ok(());
            }
            
            // 检查消息是否已经处理过
            if self.db.is_message_processed(message.id.0 as i64, message.chat.id.0).await
                .unwrap_or(false) {
                debug!("Message already processed, skipping");
                return Ok(());
            }
            
            // 检查是否已经包含总额
            if self.parser.has_total(text) {
                debug!("Message already has total, updating balance from manual edit");
                return self.handle_message_with_total(bot, message, text).await;
            }

            // 解析消息
            if let Some(parsed) = self.parser.parse(text) {
                info!("Parsed message: wallet={}, type={}, amount={}", 
                      parsed.wallet_name, parsed.transaction_type, parsed.amount);
                
                // 智能计算余额
                match self.calculator.smart_calculate_balance(
                    &parsed.wallet_name,
                    &parsed.transaction_type,
                    parsed.amount,
                    &parsed.month,
                    &parsed.year,
                    parsed.total_amount,
                    Some(message.id.0 as i64),
                    Some(message.chat.id.0),
                ).await {
                    Ok(balance_update) => {
                        // 构建新消息文本
                        let new_text = format!(
                            "{}\n#总额 {:.2}元",
                            text,
                            balance_update.new_balance
                        );

                        // 编辑消息
                        bot.edit_message_text(
                            message.chat.id,
                            message.id,
                            new_text,
                        ).await?;

                        // 记录交易
                        if let Err(e) = self.db.record_transaction(
                            &parsed.wallet_name,
                            &parsed.transaction_type,
                            parsed.amount,
                            &parsed.month,
                            &parsed.year,
                            Some(message.id.0 as i64),
                            Some(message.chat.id.0),
                        ).await {
                            error!("Failed to record transaction: {}", e);
                        }

                        // 记录消息处理状态
                        if let Err(e) = self.db.record_message(
                            message.id.0 as i64,
                            message.chat.id.0,
                            &parsed.wallet_name,
                            true,
                            Some(balance_update.old_balance),
                            Some(balance_update.new_balance),
                        ).await {
                            error!("Failed to record message: {}", e);
                        }

                        match balance_update.source {
                            BalanceUpdateSource::Transaction => {
                                info!("Successfully processed transaction: {} {} -> {}", 
                                      parsed.wallet_name, balance_update.old_balance, balance_update.new_balance);
                            }
                            BalanceUpdateSource::ManualEdit => {
                                info!("Successfully updated balance from manual edit: {} {} -> {}", 
                                      parsed.wallet_name, balance_update.old_balance, balance_update.new_balance);
                            }
                            BalanceUpdateSource::Initial => {
                                info!("Successfully set initial balance: {} -> {}", 
                                      parsed.wallet_name, balance_update.new_balance);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to calculate balance: {}", e);
                    }
                }
            } else {
                warn!("Failed to parse wallet message: {}", text);
            }
        }

        Ok(())
    }

    async fn handle_message_with_total(
        &self,
        bot: &Bot,
        message: &Message,
        text: &str,
    ) -> Result<(), RequestError> {
        debug!("Handling message with existing total");
        
        // 解析消息
        if let Some(parsed) = self.parser.parse(text) {
            // 如果有总额，使用总额更新余额
            if let Some(total_amount) = parsed.total_amount {
                match self.calculator.update_from_manual_total(
                    &parsed.wallet_name,
                    total_amount,
                    Some(message.id.0 as i64),
                    Some(message.chat.id.0),
                ).await {
                    Ok(balance_update) => {
                        // 记录交易（即使是从总额更新，也需要记录这个交易）
                        if let Err(e) = self.db.record_transaction(
                            &parsed.wallet_name,
                            &parsed.transaction_type,
                            parsed.amount,
                            &parsed.month,
                            &parsed.year,
                            Some(message.id.0 as i64),
                            Some(message.chat.id.0),
                        ).await {
                            error!("Failed to record transaction: {}", e);
                        }

                        // 记录消息处理状态
                        if let Err(e) = self.db.record_message(
                            message.id.0 as i64,
                            message.chat.id.0,
                            &parsed.wallet_name,
                            true,
                            Some(balance_update.old_balance),
                            Some(balance_update.new_balance),
                        ).await {
                            error!("Failed to record message: {}", e);
                        }

                        info!("Successfully processed message with manual total: {} {} -> {}", 
                              parsed.wallet_name, balance_update.old_balance, balance_update.new_balance);
                    }
                    Err(e) => {
                        error!("Failed to update balance from manual total: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// 重新处理消息（管理员命令）
    pub async fn reprocess_message(
        &self,
        bot: &Bot,
        message: &Message,
    ) -> Result<(), RequestError> {
        info!("Reprocessing message: {}", message.id);
        
        // 重置处理状态
        // 这里可以添加重置逻辑
        
        // 重新处理
        self.handle_message(bot, message).await
    }
}

// Tests will be added later 
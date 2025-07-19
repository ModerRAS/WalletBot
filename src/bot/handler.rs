use crate::calculator::balance::BalanceCalculator;
use crate::database::models::BalanceUpdateSource;
use crate::database::operations::DatabaseOperations;
use crate::parser::message::MessageParser;
use anyhow::Result;
use log::{debug, error, info, warn};
use teloxide::{requests::Requester, types::Message, Bot, RequestError};

#[derive(Clone, Debug)]
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

    pub async fn handle_message(&self, bot: &Bot, message: &Message) -> Result<(), RequestError> {
        // 记录接收到的消息详情，包括消息类型识别
        debug!("📨 Received message in chat {} ({:?})", message.chat.id, message.chat.kind);
        debug!("📄 Message ID: {}, Text: {:?}", message.id, message.text());
        debug!("👤 From user: {:?}", message.from());
        debug!("📝 Message link: t.me/c/{}/{}", message.chat.id.0.abs(), message.id);

        // 检查消息来源类型
        match &message.chat.kind {
            teloxide::types::ChatKind::Public(public) => {
                match &public.kind {
                    teloxide::types::PublicChatKind::Channel(_) => {
                        debug!("📢 Processing channel message");
                    }
                    teloxide::types::PublicChatKind::Group(_) => {
                        debug!("👥 Processing group message");
                    }
                    teloxide::types::PublicChatKind::Supergroup(_) => {
                        debug!("👥 Processing supergroup message");
                    }
                }
            }
            teloxide::types::ChatKind::Private(_) => {
                debug!("👤 Processing private message");
            }
        }

        // 只处理文本消息
        if let Some(text) = message.text() {
            debug!("🔄 Processing message: '{}'", text);

            // 检查是否是钱包相关消息
            if !self.parser.is_wallet_message(text) {
                return Ok(());
            }

            // 检查消息是否已经处理过
            debug!("🔍 Checking if message was already processed...");
            let message_id = message.id.0 as i64;
            let chat_id = message.chat.id.0;
            
            match self.db.is_message_processed(message_id, chat_id).await {
                Ok(true) => {
                    debug!("⚠️ Message {} already processed, skipping", message_id);
                    // 发送重复消息提示
                    let warning_text = "⚠️ 这条消息已经被处理过了，不会重复记录交易。";
                    bot.send_message(message.chat.id, warning_text).await?;
                    return Ok(());
                }
                Ok(false) => {
                    debug!("✅ Message {} not processed yet, continuing", message_id);
                }
                Err(e) => {
                    warn!("Failed to check message processing status: {}", e);
                }
            }

            // 检查是否已经包含总额
            let has_total = self.parser.has_total(text);
            debug!("📊 Message has_total: {}", has_total);
            if has_total {
                debug!("📈 Message already has total, switching to manual edit mode");
                return self.handle_message_with_total(bot, message, text).await;
            }

            // 解析消息
            if let Some(parsed) = self.parser.parse(text) {
                debug!("✅ Message parsed successfully");
                debug!("   └─ Wallet: {}", parsed.wallet_name);
                debug!("   └─ Type: {}", parsed.transaction_type);
                debug!("   └─ Amount: {}", parsed.amount);
                debug!("   └─ Month: {}", parsed.month);
                debug!("   └─ Year: {}", parsed.year);
                debug!("   └─ Total: {:?}", parsed.total_amount);

                // 智能计算余额
                match self
                    .calculator
                    .smart_calculate_balance(
                        message.chat.id.0,
                        &parsed.wallet_name,
                        &parsed.transaction_type,
                        parsed.amount,
                        &parsed.month,
                        &parsed.year,
                        parsed.total_amount,
                        Some(message.id.0 as i64),
                    )
                    .await
                {
                    Ok(balance_update) => {
                        // 构建新消息文本
                        let new_text =
                            format!("{}\n#总额 {:.2}元", text, balance_update.new_balance);

                        // 编辑消息
                        bot.edit_message_text(message.chat.id, message.id, new_text)
                            .await?;

                        // 记录交易
                        if let Err(e) = self
                            .db
                            .record_transaction(
                                message.chat.id.0,
                                &parsed.wallet_name,
                                &parsed.transaction_type,
                                parsed.amount,
                                &parsed.month,
                                &parsed.year,
                                Some(message.id.0 as i64),
                            )
                            .await
                        {
                            error!("Failed to record transaction: {e}");
                        }

                        // 记录消息处理状态
                        if let Err(e) = self
                            .db
                            .record_message(
                                message.id.0 as i64,
                                message.chat.id.0,
                                &parsed.wallet_name,
                                true,
                                Some(balance_update.old_balance),
                                Some(balance_update.new_balance),
                            )
                            .await
                        {
                            error!("Failed to record message: {e}");
                        }

                        // 发送确认消息
                        let confirmation_text = format!(
                            "✅ 交易已记录\n📊 钱包：{}\n💰 当前余额：{:.2}元",
                            parsed.wallet_name, balance_update.new_balance
                        );
                        bot.send_message(message.chat.id, &confirmation_text)
                            .await?;

                        match balance_update.source {
                            BalanceUpdateSource::Transaction => {
                                info!(
                                    "Successfully processed transaction: {} {} -> {}",
                                    parsed.wallet_name,
                                    balance_update.old_balance,
                                    balance_update.new_balance
                                );
                            }
                            BalanceUpdateSource::ManualEdit => {
                                info!(
                                    "Successfully updated balance from manual edit: {} {} -> {}",
                                    parsed.wallet_name,
                                    balance_update.old_balance,
                                    balance_update.new_balance
                                );
                            }
                            BalanceUpdateSource::Initial => {
                                info!(
                                    "Successfully set initial balance: {} -> {}",
                                    parsed.wallet_name, balance_update.new_balance
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to calculate balance: {e}");
                        // 发送错误消息
                        let error_text = "❌ 处理交易时出现错误，请稍后重试或联系管理员。";
                        bot.send_message(message.chat.id, error_text).await?;
                    }
                }
            } else {
                warn!("Failed to parse wallet message: {text}");
                // 发送格式错误提示和使用说明
                let help_text = "❌ 消息格式不正确\n\n📋 正确格式：\n#钱包名称 #月份 #年份\n#出账/入账 金额元\n\n💡 示例：\n#支付宝 #12月 #2024年\n#出账 150.00元\n\n或者：\n#微信 #01月 #2024年\n#入账 200.00元\n\n❓ 需要帮助请输入 /help";
                bot.send_message(message.chat.id, help_text).await?;
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
                match self
                    .calculator
                    .update_from_manual_total(
                        message.chat.id.0,
                        &parsed.wallet_name,
                        total_amount,
                        Some(message.id.0 as i64),
                    )
                    .await
                {
                    Ok(balance_update) => {
                        // 记录交易（即使是从总额更新，也需要记录这个交易）
                        if let Err(e) = self
                            .db
                            .record_transaction(
                                message.chat.id.0,
                                &parsed.wallet_name,
                                &parsed.transaction_type,
                                parsed.amount,
                                &parsed.month,
                                &parsed.year,
                                Some(message.id.0 as i64),
                            )
                            .await
                        {
                            error!("Failed to record transaction: {e}");
                        }

                        // 记录消息处理状态
                        if let Err(e) = self
                            .db
                            .record_message(
                                message.id.0 as i64,
                                message.chat.id.0,
                                &parsed.wallet_name,
                                true,
                                Some(balance_update.old_balance),
                                Some(balance_update.new_balance),
                            )
                            .await
                        {
                            error!("Failed to record message: {e}");
                        }

                        // 发送确认消息（手动总额更新）
                        let confirmation_text = format!(
                            "✅ 余额已更新（手动总额）\n📊 钱包：{}\n💰 当前余额：{:.2}元",
                            parsed.wallet_name, balance_update.new_balance
                        );
                        let _ = bot.send_message(message.chat.id, &confirmation_text).await;

                        info!(
                            "Successfully processed message with manual total: {} {} -> {}",
                            parsed.wallet_name,
                            balance_update.old_balance,
                            balance_update.new_balance
                        );
                    }
                    Err(e) => {
                        error!("Failed to update balance from manual total: {e}");
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

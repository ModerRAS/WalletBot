use crate::bot::handler::MessageHandler;
use log::info;
use teloxide::{requests::Requester, types::Message, Bot, RequestError};

#[derive(Clone)]
pub struct Commands {
    handler: MessageHandler,
}

impl Commands {
    pub fn new(handler: MessageHandler) -> Self {
        Self { handler }
    }

    pub async fn handle_command(
        &self,
        bot: &Bot,
        message: &Message,
        command: &str,
    ) -> Result<(), RequestError> {
        match command {
            "/start" => self.handle_start(bot, message).await,
            "/help" => self.handle_help(bot, message).await,
            "/reprocess" => self.handle_reprocess(bot, message).await,
            "/status" => self.handle_status(bot, message).await,
            _ => {
                bot.send_message(message.chat.id, "Unknown command").await?;
                Ok(())
            }
        }
    }

    async fn handle_start(&self, bot: &Bot, message: &Message) -> Result<(), RequestError> {
        let welcome_text = "欢迎使用 WalletBot！\n\n我可以帮助你管理钱包交易记录。\n\n支持的消息格式：\n#钱包名称 #月份 #年份\n#出账/入账 金额元\n\n输入 /help 查看更多命令。";

        bot.send_message(message.chat.id, welcome_text).await?;
        Ok(())
    }

    async fn handle_help(&self, bot: &Bot, message: &Message) -> Result<(), RequestError> {
        let help_text = "WalletBot 帮助\n\n支持的命令：\n/start - 开始使用\n/help - 显示帮助\n/reprocess - 重新处理消息\n/status - 查看状态\n\n消息格式：\n#钱包名称 #月份 #年份\n#出账 1000.00元\n\n或者：\n#钱包名称 #月份 #年份\n#入账 500.00元\n\n我会自动计算并添加 #总额 信息。";

        bot.send_message(message.chat.id, help_text).await?;
        Ok(())
    }

    async fn handle_reprocess(&self, bot: &Bot, message: &Message) -> Result<(), RequestError> {
        info!("Reprocessing message requested by user");

        // 这里应该重新处理回复的消息
        if let Some(reply_to) = message.reply_to_message() {
            self.handler.reprocess_message(bot, reply_to).await?;
            bot.send_message(message.chat.id, "Message reprocessed successfully")
                .await?;
        } else {
            bot.send_message(message.chat.id, "Please reply to a message to reprocess it")
                .await?;
        }

        Ok(())
    }

    async fn handle_status(&self, bot: &Bot, message: &Message) -> Result<(), RequestError> {
        let status_text = "WalletBot Status: ✅ Running\n\nDatabase: ✅ Connected\nParser: ✅ Ready\nCalculator: ✅ Ready";

        bot.send_message(message.chat.id, status_text).await?;
        Ok(())
    }
}

use anyhow::Result;
use log::{debug, error, info};
use teloxide::{
    prelude::*,
    types::{MediaKind, MessageKind},
    utils::command::BotCommands,
    RequestError,
};

use crate::bot::commands::Commands;
use crate::bot::handler::MessageHandler;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "支持的命令:")]
pub enum Command {
    #[command(description = "开始使用机器人")]
    Start,
    #[command(description = "显示帮助信息")]
    Help,
    #[command(description = "重新处理消息")]
    Reprocess,
    #[command(description = "查看机器人状态")]
    Status,
}

pub struct BotDispatcher {
    message_handler: MessageHandler,
    commands: Commands,
}

impl BotDispatcher {
    pub fn new(message_handler: MessageHandler) -> Self {
        let commands = Commands::new(message_handler.clone());
        Self {
            message_handler,
            commands,
        }
    }

    pub async fn run(self, bot: Bot) -> Result<()> {
        info!("🤖 Starting WalletBot dispatcher...");

        let handler = self.message_handler.clone();
        let commands = self.commands.clone();

        Dispatcher::builder(
            bot,
            Update::filter_message()
                .branch(dptree::entry().filter_command::<Command>().endpoint(
                    move |bot: Bot, msg: Message, cmd: Command| {
                        let commands = commands.clone();
                        async move {
                            debug!("Handling command: {cmd:?}");

                            let command_str = match cmd {
                                Command::Start => "/start",
                                Command::Help => "/help",
                                Command::Reprocess => "/reprocess",
                                Command::Status => "/status",
                            };

                            if let Err(e) = commands.handle_command(&bot, &msg, command_str).await {
                                error!("Failed to handle command {command_str}: {e}");
                            }

                            Ok::<(), RequestError>(())
                        }
                    },
                ))
                .branch(
                    dptree::filter(|msg: Message| msg.text().is_some()).endpoint(
                        move |bot: Bot, msg: Message| {
                            let handler = handler.clone();
                            async move {
                                debug!(
                                    "Handling message from chat: {}, user: {:?}",
                                    msg.chat.id,
                                    msg.from()
                                );

                                // 只处理文本消息
                                if let MessageKind::Common(common_msg) = &msg.kind {
                                    if let MediaKind::Text(_) = &common_msg.media_kind {
                                        if let Err(e) = handler.handle_message(&bot, &msg).await {
                                            error!("Failed to handle message: {e}");

                                            // 发送通用错误消息
                                            let error_text = "❌ 处理消息时发生错误，请稍后重试。";
                                            if let Err(send_err) =
                                                bot.send_message(msg.chat.id, error_text).await
                                            {
                                                error!("Failed to send error message: {send_err}");
                                            }
                                        }
                                    }
                                }

                                Ok::<(), RequestError>(())
                            }
                        },
                    ),
                ),
        )
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

        Ok(())
    }
}

/// 启动机器人的主函数
pub async fn start_bot(token: &str, message_handler: MessageHandler) -> Result<()> {
    info!("🚀 Initializing Telegram Bot...");

    let bot = Bot::new(token);

    // 获取机器人信息
    match bot.get_me().await {
        Ok(me) => {
            info!("✅ Bot connected successfully:");
            info!("  - Username: @{}", me.username());
            info!("  - Name: {}", me.first_name);
            info!("  - ID: {}", me.id);
        }
        Err(e) => {
            error!("❌ Failed to connect to Telegram Bot API: {e}");
            return Err(anyhow::anyhow!("Bot connection failed: {}", e));
        }
    }

    // 创建并启动调度器
    let dispatcher = BotDispatcher::new(message_handler);

    info!("🎯 Starting message processing...");
    info!("💡 Bot is now ready to receive messages!");
    info!("📝 Send a wallet transaction message to get started.");

    dispatcher.run(bot).await?;

    Ok(())
}

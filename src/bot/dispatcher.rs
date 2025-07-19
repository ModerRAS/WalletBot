use anyhow::Result;
use log::{debug, error, info};
use teloxide::{
    prelude::*,
    types::Update,
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

        let message_handler = self.message_handler.clone();
        let edited_message_handler = self.message_handler.clone();
        let channel_post_handler = self.message_handler.clone();
        let edited_channel_post_handler = self.message_handler.clone();
        let commands = self.commands.clone();

        Dispatcher::builder(
            bot,
            dptree::entry()
                // 处理常规消息
                .branch(Update::filter_message()
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
                        }
                    ))
                    .branch(
                        dptree::filter(|msg: Message| msg.text().is_some())
                            .endpoint(move |bot: Bot, msg: Message| {
                                let handler = message_handler.clone();
                                async move {
                                    debug!(
                                        "📨 Processing message from chat: {}, type: {:?}, user: {:?}",
                                        msg.chat.id,
                                        msg.chat.kind,
                                        msg.from()
                                    );

                                    if let Some(text) = msg.text() {
                                        debug!("📄 Message text: {}", text);
                                        
                                        // 处理消息
                                        if let Err(e) = handler.handle_message(&bot, &msg).await {
                                            error!("❌ Failed to handle message: {e}");
                                            
                                            // 只在可以发送消息的聊天中发送错误
                                            if !matches!(msg.chat.kind, teloxide::types::ChatKind::Public(_)) {
                                                let error_text = "❌ 处理消息时出现错误，请稍后重试。";
                                                let _ = bot.send_message(msg.chat.id, error_text).await;
                                            }
                                        }
                                    }

                                    Ok::<(), RequestError>(())
                                }
                            }),
                    ))
                // 处理编辑的消息
                .branch(Update::filter_edited_message().branch(
                    dptree::filter(|msg: Message| msg.text().is_some())
                        .endpoint(move |bot: Bot, msg: Message| {
                            let handler = edited_message_handler.clone();
                            async move {
                                debug!("📝 Processing edited message from chat: {}", msg.chat.id);
                                if let Some(text) = msg.text() {
                                    debug!("📄 Edited message text: {}", text);
                                    
                                    if let Err(e) = handler.handle_message(&bot, &msg).await {
                                        error!("❌ Failed to handle edited message: {e}");
                                        
                                        if !matches!(msg.chat.kind, teloxide::types::ChatKind::Public(_)) {
                                            let error_text = "❌ 处理编辑消息时出现错误。";
                                            let _ = bot.send_message(msg.chat.id, error_text).await;
                                        }
                                    }
                                }
                                Ok::<(), RequestError>(())
                            }
                        }),
                ))
                // 处理频道帖子
                .branch(Update::filter_channel_post().branch(
                    dptree::filter(|post: Message| post.text().is_some())
                        .endpoint(move |bot: Bot, post: Message| {
                            let handler = channel_post_handler.clone();
                            async move {
                                debug!(
                                    "📢 Processing channel post from channel: {}, title: {:?}",
                                    post.chat.id,
                                    post.chat.title()
                                );

                                if let Some(text) = post.text() {
                                    debug!("📄 Channel post text: {}", text);
                                    
                                    // 处理频道帖子
                                    if let Err(e) = handler.handle_message(&bot, &post).await {
                                        error!("❌ Failed to handle channel post: {e}");
                                        // 频道消息通常无法回复，所以不发送错误消息
                                    }
                                }

                                Ok::<(), RequestError>(())
                            }
                        }),
                ))
                // 处理编辑的频道帖子
                .branch(Update::filter_edited_channel_post().branch(
                    dptree::filter(|post: Message| post.text().is_some())
                        .endpoint(move |bot: Bot, post: Message| {
                            let handler = edited_channel_post_handler.clone();
                            async move {
                                debug!("📝 Processing edited channel post from channel: {}", post.chat.id);
                                if let Some(text) = post.text() {
                                    debug!("📄 Edited channel post text: {}", text);
                                    
                                    if let Err(e) = handler.handle_message(&bot, &post).await {
                                        error!("❌ Failed to handle edited channel post: {e}");
                                    }
                                }
                                Ok::<(), RequestError>(())
                            }
                        }),
                )),
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

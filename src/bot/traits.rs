use async_trait::async_trait;
use teloxide::{
    types::{ChatId, Message, MessageId},
    RequestError,
};

/// 抽象Bot API操作的trait，用于测试时mock
#[async_trait]
#[allow(dead_code)]
pub trait BotApi {
    /// 发送消息
    async fn send_message(&self, chat_id: ChatId, text: &str) -> Result<Message, RequestError>;

    /// 编辑消息文本
    async fn edit_message_text(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
        text: &str,
    ) -> Result<Message, RequestError>;

    /// 删除消息
    async fn delete_message(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
    ) -> Result<(), RequestError>;

    /// 发送回复消息
    async fn reply_to_message(
        &self,
        message: &Message,
        text: &str,
    ) -> Result<Message, RequestError>;
}

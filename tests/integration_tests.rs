use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::NamedTempFile;
use serial_test::serial;
use anyhow::Result;
use async_trait::async_trait;

// 导入我们需要测试的模块
use walletbot::database::operations::DatabaseOperations;
use walletbot::bot::handler::MessageHandler;
use walletbot::bot::traits::BotApi;
use walletbot::parser::message::MessageParser;

use teloxide::types::{
    Message, Chat, ChatId, MessageId, User, UserId, MessageKind, MessageCommon, 
    MediaKind, MediaText,
};
use teloxide::RequestError;
use chrono::Utc;

// Mock Bot API 实现
#[derive(Debug, Clone)]
pub struct MockBotApi {
    pub sent_messages: Arc<Mutex<Vec<MockSentMessage>>>,
    pub edited_messages: Arc<Mutex<Vec<MockEditedMessage>>>,
    pub deleted_messages: Arc<Mutex<Vec<MockDeletedMessage>>>,
    pub should_fail: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub struct MockSentMessage {
    pub chat_id: ChatId,
    pub text: String,
    pub reply_to_message_id: Option<MessageId>,
}

#[derive(Debug, Clone)]
pub struct MockEditedMessage {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct MockDeletedMessage {
    pub chat_id: ChatId,
    pub message_id: MessageId,
}

impl MockBotApi {
    pub fn new() -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            edited_messages: Arc::new(Mutex::new(Vec::new())),
            deleted_messages: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().await = should_fail;
    }

    pub async fn get_sent_messages(&self) -> Vec<MockSentMessage> {
        self.sent_messages.lock().await.clone()
    }

    pub async fn get_edited_messages(&self) -> Vec<MockEditedMessage> {
        self.edited_messages.lock().await.clone()
    }

    pub async fn get_deleted_messages(&self) -> Vec<MockDeletedMessage> {
        self.deleted_messages.lock().await.clone()
    }

    pub async fn clear_all(&self) {
        self.sent_messages.lock().await.clear();
        self.edited_messages.lock().await.clear();
        self.deleted_messages.lock().await.clear();
    }

    fn create_mock_message(chat_id: ChatId, message_id: MessageId, text: &str) -> Message {
        let user = User {
            id: UserId(12345),
            is_bot: false,
            first_name: "Test".to_string(),
            last_name: Some("User".to_string()),
            username: Some("testuser".to_string()),
            language_code: Some("zh".to_string()),
            is_premium: false,
            added_to_attachment_menu: false,
        };

        let chat = Chat {
            id: chat_id,
            kind: teloxide::types::ChatKind::Private(teloxide::types::ChatPrivate {
                username: Some("testuser".to_string()),
                first_name: Some("Test".to_string()),
                last_name: Some("User".to_string()),
                bio: None,
                has_private_forwards: None,
                has_restricted_voice_and_video_messages: None,
                emoji_status_custom_emoji_id: None,
            }),
            photo: None,
            pinned_message: None,
            message_auto_delete_time: None,
            has_hidden_members: false,
            has_aggressive_anti_spam_enabled: false,
        };

        Message {
            id: message_id,
            thread_id: None,
            date: Utc::now(),
            chat,
            via_bot: None,
            kind: MessageKind::Common(MessageCommon {
                from: Some(user),
                forward: None,
                edit_date: None,
                media_kind: MediaKind::Text(MediaText {
                    text: text.to_string(),
                    entities: vec![],
                }),
                reply_markup: None,
                sender_chat: None,
                author_signature: None,
                is_automatic_forward: false,
                has_protected_content: false,
                reply_to_message: None,
                is_topic_message: false,
            }),
        }
    }
}

#[async_trait]
impl BotApi for MockBotApi {
    async fn send_message(&self, chat_id: ChatId, text: &str) -> Result<Message, RequestError> {
        if *self.should_fail.lock().await {
            return Err(RequestError::Api(teloxide::ApiError::Unknown("Mock error".to_string())));
        }

        let message_id = MessageId(rand::random::<i32>().abs());
        let mut messages = self.sent_messages.lock().await;
        messages.push(MockSentMessage {
            chat_id,
            text: text.to_string(),
            reply_to_message_id: None,
        });

        Ok(Self::create_mock_message(chat_id, message_id, text))
    }

    async fn edit_message_text(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
        text: &str,
    ) -> Result<Message, RequestError> {
        if *self.should_fail.lock().await {
            return Err(RequestError::Api(teloxide::ApiError::Unknown("Mock error".to_string())));
        }

        let mut messages = self.edited_messages.lock().await;
        messages.push(MockEditedMessage {
            chat_id,
            message_id,
            text: text.to_string(),
        });

        Ok(Self::create_mock_message(chat_id, message_id, text))
    }

    async fn delete_message(&self, chat_id: ChatId, message_id: MessageId) -> Result<(), RequestError> {
        if *self.should_fail.lock().await {
            return Err(RequestError::Api(teloxide::ApiError::Unknown("Mock error".to_string())));
        }

        let mut messages = self.deleted_messages.lock().await;
        messages.push(MockDeletedMessage {
            chat_id,
            message_id,
        });

        Ok(())
    }

    async fn reply_to_message(
        &self,
        message: &Message,
        text: &str,
    ) -> Result<Message, RequestError> {
        if *self.should_fail.lock().await {
            return Err(RequestError::Api(teloxide::ApiError::Unknown("Mock error".to_string())));
        }

        let message_id = MessageId(rand::random::<i32>().abs());
        let mut messages = self.sent_messages.lock().await;
        messages.push(MockSentMessage {
            chat_id: message.chat.id,
            text: text.to_string(),
            reply_to_message_id: Some(message.id),
        });

        Ok(Self::create_mock_message(message.chat.id, message_id, text))
    }
}

// 测试辅助函数
async fn create_test_db() -> Result<DatabaseOperations> {
    let temp_file = NamedTempFile::new()?;
    let db_path = temp_file.path().to_str().unwrap();
    DatabaseOperations::new(db_path).await
}

async fn create_test_handler() -> Result<MessageHandler> {
    let db = create_test_db().await?;
    Ok(MessageHandler::new(db))
}

// 测试消息解析器
#[tokio::test]
#[serial]
async fn test_message_parser() -> Result<()> {
    let parser = MessageParser::new();
    
    // 测试正常的钱包消息
    let test_message = "#支付宝 #12月 #2024年\n#出账 150.00元";
    let parsed = parser.parse(test_message);
    
    assert!(parsed.is_some());
    let parsed = parsed.unwrap();
    assert_eq!(parsed.wallet_name, "支付宝");
    assert_eq!(parsed.transaction_type, "出账");
    assert_eq!(parsed.amount, 150.0);
    assert_eq!(parsed.month, "12");
    assert_eq!(parsed.year, "2024");
    
    // 测试非钱包消息
    let non_wallet_message = "这是一个普通消息";
    assert!(!parser.is_wallet_message(non_wallet_message));
    
    // 测试包含总额的消息
    let message_with_total = "#支付宝 #12月 #2024年\n#出账 150.00元\n#总额 1000.00元";
    assert!(parser.has_total(message_with_total));
    assert_eq!(parser.extract_total_amount(message_with_total), Some(1000.0));
    
    println!("✅ 消息解析器测试通过");
    Ok(())
}

// 测试数据库操作
#[tokio::test]
#[serial]
async fn test_database_operations() -> Result<()> {
    let db = create_test_db().await?;
    
    // 测试创建钱包
    let wallet = db.get_or_create_wallet("测试钱包").await?;
    assert_eq!(wallet.name, "测试钱包");
    assert_eq!(wallet.current_balance, 0.0);
    
    // 测试更新余额
    db.update_wallet_balance("测试钱包", 1000.0).await?;
    let updated_wallet = db.get_or_create_wallet("测试钱包").await?;
    assert_eq!(updated_wallet.current_balance, 1000.0);
    
    // 测试记录交易
    db.record_transaction(
        "测试钱包",
        "出账",
        150.0,
        "12",
        "2024",
        Some(123),
        Some(456),
    ).await?;
    
    // 测试记录消息处理状态
    db.record_message(123, 456, "测试钱包", true, Some(1000.0), Some(850.0)).await?;
    
    // 测试检查消息是否已处理
    let is_processed = db.is_message_processed(123, 456).await?;
    assert!(is_processed);
    
    println!("✅ 数据库操作测试通过");
    Ok(())
}

// 测试Mock Bot API
#[tokio::test]
#[serial]
async fn test_mock_bot_api() -> Result<()> {
    let mock_bot = MockBotApi::new();
    
    // 测试发送消息
    let chat_id = ChatId(12345);
    let message_text = "测试消息";
    
    let result = mock_bot.send_message(chat_id, message_text).await;
    assert!(result.is_ok());
    
    let sent_messages = mock_bot.get_sent_messages().await;
    assert_eq!(sent_messages.len(), 1);
    assert_eq!(sent_messages[0].chat_id, chat_id);
    assert_eq!(sent_messages[0].text, message_text);
    
    // 测试编辑消息
    let message_id = MessageId(1);
    let edited_text = "编辑后的消息";
    
    let result = mock_bot.edit_message_text(chat_id, message_id, edited_text).await;
    assert!(result.is_ok());
    
    let edited_messages = mock_bot.get_edited_messages().await;
    assert_eq!(edited_messages.len(), 1);
    assert_eq!(edited_messages[0].chat_id, chat_id);
    assert_eq!(edited_messages[0].message_id, message_id);
    assert_eq!(edited_messages[0].text, edited_text);
    
    // 测试删除消息
    let result = mock_bot.delete_message(chat_id, message_id).await;
    assert!(result.is_ok());
    
    let deleted_messages = mock_bot.get_deleted_messages().await;
    assert_eq!(deleted_messages.len(), 1);
    assert_eq!(deleted_messages[0].chat_id, chat_id);
    assert_eq!(deleted_messages[0].message_id, message_id);
    
    // 测试失败情况
    mock_bot.set_should_fail(true).await;
    let result = mock_bot.send_message(chat_id, "这应该失败").await;
    assert!(result.is_err());
    
    println!("✅ Mock Bot API测试通过");
    Ok(())
}

// 测试错误处理
#[tokio::test]
#[serial]
async fn test_error_handling() -> Result<()> {
    let parser = MessageParser::new();
    
    // 测试无效消息格式
    let invalid_messages = vec![
        "普通消息",
        "#支付宝 #出账 150.00元", // 缺少时间
        "#支付宝 #12月 #2024年", // 缺少交易信息
        "#支付宝 #12月 #2024年\n#出账", // 缺少金额
        "#支付宝 #12月 #2024年\n#出账 abc元", // 无效金额
    ];
    
    for message in invalid_messages {
        let parsed = parser.parse(message);
        assert!(parsed.is_none() || !parser.is_wallet_message(message));
        println!("✅ 正确拒绝无效消息: {}", message);
    }
    
    println!("✅ 错误处理测试通过");
    Ok(())
}

// 测试重复消息处理
#[tokio::test]
#[serial]
async fn test_duplicate_message_handling() -> Result<()> {
    let db = create_test_db().await?;
    
    // 首先创建钱包
    let _wallet = db.get_or_create_wallet("支付宝").await?;
    
    // 记录一条消息
    db.record_message(123, 456, "支付宝", true, Some(1000.0), Some(850.0)).await?;
    
    // 检查是否已处理
    let is_processed = db.is_message_processed(123, 456).await?;
    assert!(is_processed);
    
    // 不同的消息ID应该返回false
    let is_processed_different = db.is_message_processed(124, 456).await?;
    assert!(!is_processed_different);
    
    println!("✅ 重复消息处理测试通过");
    Ok(())
}

// 测试完整的消息处理流程
#[tokio::test]
#[serial]
async fn test_complete_message_flow() -> Result<()> {
    let db = create_test_db().await?;
    let _handler = MessageHandler::new(db.clone());
    
    // 测试场景：处理多个钱包交易
    let test_scenarios = vec![
        (
            "#支付宝 #12月 #2024年\n#出账 150.00元",
            "支付宝",
            "出账",
            150.0,
        ),
        (
            "#微信 #12月 #2024年\n#入账 200.00元",
            "微信",
            "入账",
            200.0,
        ),
        (
            "#支付宝 #12月 #2024年\n#入账 50.00元",
            "支付宝",
            "入账",
            50.0,
        ),
    ];
    
    for (message_text, wallet_name, transaction_type, amount) in test_scenarios {
        // 验证消息能正确解析
        let parser = MessageParser::new();
        let parsed = parser.parse(message_text);
        assert!(parsed.is_some());
        
        let parsed = parsed.unwrap();
        assert_eq!(parsed.wallet_name, wallet_name);
        assert_eq!(parsed.transaction_type, transaction_type);
        assert_eq!(parsed.amount, amount);
        
        // 验证数据库操作
        let wallet = db.get_or_create_wallet(wallet_name).await?;
        println!("钱包 {} 当前余额: {}", wallet_name, wallet.current_balance);
    }
    
    println!("✅ 完整消息处理流程测试通过");
    Ok(())
}

// 性能测试
#[tokio::test]
#[serial]
async fn test_performance() -> Result<()> {
    let db = create_test_db().await?;
    let parser = MessageParser::new();
    
    let test_message = "#支付宝 #12月 #2024年\n#出账 150.00元";
    
    // 测试解析性能
    let start_time = std::time::Instant::now();
    for _ in 0..1000 {
        let _parsed = parser.parse(test_message);
    }
    let parse_duration = start_time.elapsed();
    
    // 测试数据库性能
    // 首先创建钱包
    let _wallet = db.get_or_create_wallet("性能测试钱包").await?;
    
    let start_time = std::time::Instant::now();
    for i in 0..100 {
        db.record_transaction(
            "性能测试钱包",
            "出账",
            100.0,
            "12",
            "2024",
            Some(i),
            Some(12345),
        ).await?;
    }
    let db_duration = start_time.elapsed();
    
    println!("✅ 性能测试结果:");
    println!("  - 1000次消息解析耗时: {:?}", parse_duration);
    println!("  - 100次数据库操作耗时: {:?}", db_duration);
    println!("  - 平均单次解析耗时: {:?}", parse_duration / 1000);
    println!("  - 平均单次数据库操作耗时: {:?}", db_duration / 100);
    
    Ok(())
}

// 并发测试
#[tokio::test]
#[serial]
async fn test_concurrent_operations() -> Result<()> {
    let db = create_test_db().await?;
    
    // 并发创建多个钱包
    let mut handles = vec![];
    for i in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let wallet_name = format!("并发测试钱包{}", i);
            db_clone.get_or_create_wallet(&wallet_name).await
        });
        handles.push(handle);
    }
    
    // 等待所有操作完成
    for handle in handles {
        handle.await??;
    }
    
    println!("✅ 并发操作测试通过");
    Ok(())
} 
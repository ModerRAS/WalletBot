use teloxide::RequestError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletBotError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Telegram API error: {0}")]
    Telegram(#[from] RequestError),

    #[error("Parser error: {message}")]
    Parser { message: String },

    #[error("Balance calculation error: {message}")]
    BalanceCalculation { message: String },

    #[error("Wallet not found: {name}")]
    WalletNotFound { name: String },

    #[error("Invalid message format: {message}")]
    InvalidMessageFormat { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),
}

pub type Result<T> = std::result::Result<T, WalletBotError>;

impl WalletBotError {
    pub fn parser_error(message: impl Into<String>) -> Self {
        Self::Parser {
            message: message.into(),
        }
    }

    pub fn balance_calculation_error(message: impl Into<String>) -> Self {
        Self::BalanceCalculation {
            message: message.into(),
        }
    }

    pub fn wallet_not_found(name: impl Into<String>) -> Self {
        Self::WalletNotFound { name: name.into() }
    }

    pub fn invalid_message_format(message: impl Into<String>) -> Self {
        Self::InvalidMessageFormat {
            message: message.into(),
        }
    }

    /// 检查错误是否为可重试的类型
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            WalletBotError::Database(_) | WalletBotError::Telegram(_) | WalletBotError::Io(_)
        )
    }

    /// 获取错误的严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            WalletBotError::Config(_) => ErrorSeverity::Critical,
            WalletBotError::Database(_) => ErrorSeverity::High,
            WalletBotError::Telegram(_) => ErrorSeverity::Medium,
            WalletBotError::Parser { .. } => ErrorSeverity::Low,
            WalletBotError::BalanceCalculation { .. } => ErrorSeverity::High,
            WalletBotError::WalletNotFound { .. } => ErrorSeverity::Medium,
            WalletBotError::InvalidMessageFormat { .. } => ErrorSeverity::Low,
            WalletBotError::Io(_) => ErrorSeverity::Medium,
            WalletBotError::Env(_) => ErrorSeverity::Critical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

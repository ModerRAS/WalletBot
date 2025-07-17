use crate::error::Result;
use log::{debug, warn};
use std::time::Duration;
use tokio::time::sleep;

#[cfg(test)]
use crate::error::WalletBotError;

#[allow(dead_code)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

#[allow(dead_code)]
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;
    let mut delay = config.base_delay;

    for attempt in 1..=config.max_attempts {
        debug!(
            "Attempting operation '{}' (attempt {}/{})",
            operation_name, attempt, config.max_attempts
        );

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("Operation '{operation_name}' succeeded on attempt {attempt}");
                }
                return Ok(result);
            }
            Err(error) => {
                warn!("Operation '{operation_name}' failed on attempt {attempt}: {error}");

                // 检查错误是否可重试
                if !error.is_retryable() {
                    warn!("Error is not retryable, stopping attempts");
                    return Err(error);
                }

                last_error = Some(error);

                // 如果还有重试机会，等待后重试
                if attempt < config.max_attempts {
                    debug!("Waiting {delay:?} before next attempt");
                    sleep(delay).await;

                    // 指数退避
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * config.backoff_multiplier) as u64,
                        ),
                        config.max_delay,
                    );
                }
            }
        }
    }

    // 所有重试都失败了
    let final_error = last_error.unwrap();
    warn!(
        "Operation '{}' failed after {} attempts: {}",
        operation_name, config.max_attempts, final_error
    );
    Err(final_error)
}

/// 重试装饰器宏
#[macro_export]
macro_rules! retry_operation {
    ($operation:expr, $name:expr) => {
        $crate::retry::retry_with_backoff(
            || async { $operation },
            $crate::retry::RetryConfig::default(),
            $name,
        )
        .await
    };

    ($operation:expr, $name:expr, $config:expr) => {
        $crate::retry::retry_with_backoff(|| async { $operation }, $config, $name).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let operation = || {
            let counter = counter_clone.clone();
            async move {
                let mut count = counter.lock().unwrap();
                *count += 1;

                if *count == 1 {
                    // 使用可重试的IO错误而不是不可重试的解析错误
                    Err(WalletBotError::Io(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Temporary error",
                    )))
                } else {
                    Ok("success")
                }
            }
        };

        let result = retry_with_backoff(
            operation,
            RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(10),
                backoff_multiplier: 2.0,
            },
            "test_operation",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        let operation = || async { Err(WalletBotError::parser_error("Non-retryable error")) };

        let result: Result<()> =
            retry_with_backoff(operation, RetryConfig::default(), "test_operation").await;

        assert!(result.is_err());
        // 应该立即失败，不重试
    }
}

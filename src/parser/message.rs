use crate::database::models::ParsedMessage;
use crate::parser::regex::RegexPatterns;
use log::debug;

#[derive(Clone, Debug)]
pub struct MessageParser {
    patterns: &'static RegexPatterns,
}

impl MessageParser {
    pub fn new() -> Self {
        Self {
            patterns: RegexPatterns::get_instance(),
        }
    }

    pub fn parse(&self, text: &str) -> Option<ParsedMessage> {
        debug!("Parsing message: {text}");

        // 解析钱包名称
        let wallet_name = self
            .patterns
            .wallet_regex
            .captures(text)?
            .get(1)?
            .as_str()
            .to_string();
        debug!("Wallet name: {wallet_name}");

        // 解析交易类型
        let transaction_type = self
            .patterns
            .transaction_regex
            .captures(text)?
            .get(1)?
            .as_str()
            .to_string();
        debug!("Transaction type: {transaction_type}");

        // 解析金额 - 需要找到交易金额，而不是总额
        let amount = self.parse_transaction_amount(text)?;
        debug!("Transaction amount: {amount}");

        // 解析时间
        let time_captures = self.patterns.time_regex.captures(text)?;
        let month = time_captures.get(1)?.as_str().to_string();
        let year = time_captures.get(2)?.as_str().to_string();
        debug!("Time: {month}月 {year}年");

        // 解析总额（如果存在）
        let total_amount = self.parse_total_amount(text);
        if let Some(total) = total_amount {
            debug!("Total amount found: {total}");
        }

        Some(ParsedMessage {
            wallet_name,
            transaction_type,
            amount,
            month,
            year,
            total_amount,
            original_text: text.to_string(),
        })
    }

    fn parse_transaction_amount(&self, text: &str) -> Option<f64> {
        // 找到所有金额，排除总额
        let mut amounts = Vec::new();
        for cap in self.patterns.amount_regex.captures_iter(text) {
            if let Some(amount_match) = cap.get(0) {
                let amount_str = amount_match.as_str();
                // 检查这个金额是否是总额
                if !self.is_total_amount(text, amount_match.start()) {
                    if let Ok(amount) = amount_str.trim_end_matches("元").parse::<f64>() {
                        amounts.push(amount);
                    }
                }
            }
        }

        // 返回第一个非总额的金额
        amounts.into_iter().next()
    }

    fn is_total_amount(&self, text: &str, amount_pos: usize) -> bool {
        // 检查金额前面是否有 #总额
        let prefix = &text[..amount_pos];
        prefix.contains("#总额")
    }

    fn parse_total_amount(&self, text: &str) -> Option<f64> {
        self.patterns
            .total_regex
            .captures(text)?
            .get(1)?
            .as_str()
            .parse::<f64>()
            .ok()
    }

    pub fn has_total(&self, text: &str) -> bool {
        self.patterns.total_regex.is_match(text)
    }

    #[allow(dead_code)]
    pub fn extract_total_amount(&self, text: &str) -> Option<f64> {
        self.parse_total_amount(text)
    }

    /// 检查消息是否符合钱包操作格式
    pub fn is_wallet_message(&self, text: &str) -> bool {
        self.patterns.wallet_regex.is_match(text)
            && self.patterns.transaction_regex.is_match(text)
            && self.patterns.amount_regex.is_match(text)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Transaction {
    pub transaction_type: String,
    pub amount: f64,
    pub description: String,
}

impl MessageParser {
    #[allow(dead_code)]
    pub fn parse_transaction(&self, text: &str) -> Result<Transaction, anyhow::Error> {
        // 简化的交易解析，适用于"收入 100 工作收入"这样的格式
        let parts: Vec<&str> = text.split_whitespace().collect();

        if parts.len() < 3 {
            return Err(anyhow::Error::msg("Invalid transaction format"));
        }

        let transaction_type = parts[0].to_string();
        let amount = parts[1]
            .parse::<f64>()
            .map_err(|_| anyhow::Error::msg("Invalid amount"))?;
        let description = parts[2..].join(" ");

        // 验证交易类型
        if transaction_type != "收入" && transaction_type != "支出" {
            return Err(anyhow::Error::msg("Invalid transaction type"));
        }

        Ok(Transaction {
            transaction_type,
            amount,
            description,
        })
    }
}

impl Default for MessageParser {
    fn default() -> Self {
        Self::new()
    }
}

// Tests will be added later

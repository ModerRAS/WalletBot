use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct RegexPatterns {
    pub wallet_regex: Regex,
    pub transaction_regex: Regex,
    pub amount_regex: Regex,
    pub time_regex: Regex,
    pub total_regex: Regex,
}

impl RegexPatterns {
    pub fn new() -> Self {
        Self {
            // 匹配钱包名称 #钱包名称 #月份 #年份
            wallet_regex: Regex::new(r"#([^#\s]+)\s+#\d+月").unwrap(),
            // 匹配交易类型 #出账 或 #入账 或 #收入 或 #支出
            transaction_regex: Regex::new(r"#(出账|入账|收入|支出)").unwrap(),
            // 匹配金额 数字.数字元
            amount_regex: Regex::new(r"(\d+(?:\.\d+)?)元").unwrap(),
            // 匹配时间 #数字月 #数字年 - 捕获完整的月份和年份
            time_regex: Regex::new(r"#(\d+月)\s+#(\d+年)").unwrap(),
            // 匹配总额 #总额 数字元
            total_regex: Regex::new(r"#总额\s+(\d+(?:\.\d+)?)元").unwrap(),
        }
    }

    pub fn get_instance() -> &'static Self {
        static INSTANCE: OnceLock<RegexPatterns> = OnceLock::new();
        INSTANCE.get_or_init(RegexPatterns::new)
    }
}

impl Default for RegexPatterns {
    fn default() -> Self {
        Self::new()
    }
}

use crate::database::models::{Transaction, Wallet};
use anyhow::Result;
use chrono::{Datelike, Utc};
use log::{debug, info};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct DatabaseOperations {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseOperations {
    pub async fn new(database_url: &str) -> Result<Self> {
        let conn = Connection::open(database_url)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().await;

        // 创建钱包表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wallets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                current_balance REAL NOT NULL DEFAULT 0.0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(chat_id, name)
            )",
            [],
        )?;

        // 创建交易表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_id INTEGER NOT NULL,
                transaction_type TEXT NOT NULL,
                amount REAL NOT NULL,
                month TEXT NOT NULL,
                year TEXT NOT NULL,
                message_id INTEGER,
                chat_id INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (wallet_id) REFERENCES wallets(id)
            )",
            [],
        )?;

        // 创建消息表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL,
                chat_id INTEGER NOT NULL,
                wallet_id INTEGER NOT NULL,
                has_total BOOLEAN DEFAULT FALSE,
                processed BOOLEAN DEFAULT FALSE,
                original_balance REAL,
                new_balance REAL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (wallet_id) REFERENCES wallets(id),
                UNIQUE(message_id, chat_id)
            )",
            [],
        )?;

        info!("Database schema initialized successfully");
        Ok(())
    }

    pub async fn get_or_create_wallet(&self, chat_id: i64, name: &str) -> Result<Wallet> {
        let conn = self.conn.lock().await;

        // 尝试获取现有钱包
        let mut stmt = conn.prepare("SELECT id, chat_id, name, current_balance, created_at, updated_at FROM wallets WHERE chat_id = ?1 AND name = ?2")?;
        let mut wallet_iter = stmt.query_map(params![chat_id, name], |row| {
            Ok(Wallet {
                id: Some(row.get(0)?),
                chat_id: row.get(1)?,
                name: row.get(2)?,
                current_balance: row.get(3)?,
                created_at: row.get(4).ok(),
                updated_at: row.get(5).ok(),
            })
        })?;

        if let Some(wallet) = wallet_iter.next() {
            return Ok(wallet?);
        }

        // 如果不存在，创建新钱包
        let now = Utc::now();
        conn.execute(
            "INSERT INTO wallets (chat_id, name, current_balance, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![chat_id, name, 0.0, now, now],
        )?;

        let wallet_id = conn.last_insert_rowid();
        debug!(
            "Created new wallet: {} in chat {} with ID: {}",
            name, chat_id, wallet_id
        );

        Ok(Wallet {
            id: Some(wallet_id),
            chat_id,
            name: name.to_string(),
            current_balance: 0.0,
            created_at: Some(now),
            updated_at: Some(now),
        })
    }

    pub async fn update_wallet_balance(
        &self,
        chat_id: i64,
        name: &str,
        balance: f64,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now();

        conn.execute(
            "UPDATE wallets SET current_balance = ?1, updated_at = ?2 WHERE chat_id = ?3 AND name = ?4",
            params![balance, now, chat_id, name],
        )?;

        info!(
            "Updated wallet balance: {} in chat {} -> {}",
            name, chat_id, balance
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn record_transaction(
        &self,
        chat_id: i64,
        wallet_name: &str,
        transaction_type: &str,
        amount: f64,
        month: &str,
        year: &str,
        message_id: Option<i64>,
    ) -> Result<()> {
        let conn = self.conn.lock().await;

        // 获取钱包ID
        let wallet = self.get_wallet_by_name_sync(&conn, chat_id, wallet_name)?;
        let wallet_id = wallet.id.unwrap();

        let now = Utc::now();
        conn.execute(
            "INSERT INTO transactions (wallet_id, transaction_type, amount, month, year, message_id, chat_id, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![wallet_id, transaction_type, amount, month, year, message_id, Some(chat_id), now],
        )?;

        debug!(
            "Recorded transaction: {} {} {}",
            wallet_name, transaction_type, amount
        );
        Ok(())
    }

    pub async fn record_message(
        &self,
        message_id: i64,
        chat_id: i64,
        wallet_name: &str,
        has_total: bool,
        original_balance: Option<f64>,
        new_balance: Option<f64>,
    ) -> Result<()> {
        let conn = self.conn.lock().await;

        // 获取钱包ID
        let wallet = self.get_wallet_by_name_sync(&conn, chat_id, wallet_name)?;
        let wallet_id = wallet.id.unwrap();

        let now = Utc::now();
        conn.execute(
            "INSERT OR REPLACE INTO messages (message_id, chat_id, wallet_id, has_total, processed, original_balance, new_balance, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![message_id, chat_id, wallet_id, has_total, true, original_balance, new_balance, now],
        )?;

        debug!("Recorded message: {} in chat {}", message_id, chat_id);
        Ok(())
    }

    pub async fn get_latest_balance(
        &self,
        chat_id: i64,
        wallet_name: &str,
        _month: &str,
        _year: &str,
    ) -> Result<f64> {
        let conn = self.conn.lock().await;

        // 获取钱包
        let wallet = self.get_wallet_by_name_sync(&conn, chat_id, wallet_name)?;

        // 返回当前余额
        Ok(wallet.current_balance)
    }

    pub async fn is_message_processed(&self, message_id: i64, chat_id: i64) -> Result<bool> {
        let conn = self.conn.lock().await;
        let mut stmt =
            conn.prepare("SELECT id FROM messages WHERE message_id = ? AND chat_id = ?")?;
        let rows: Vec<i64> = stmt
            .query_map(params![message_id, chat_id], |row| row.get(0))?
            .collect::<SqliteResult<Vec<i64>>>()?;

        Ok(!rows.is_empty())
    }

    pub async fn get_transactions(
        &self,
        chat_id: i64,
        wallet_name: &str,
    ) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().await;
        let wallet = self.get_wallet_by_name_sync(&conn, chat_id, wallet_name)?;

        let mut stmt = conn.prepare(
            "SELECT id, wallet_id, transaction_type, amount, month, year, message_id, chat_id, created_at 
             FROM transactions 
             WHERE wallet_id = ? 
             ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map(params![wallet.id], |row| {
            Ok(Transaction {
                id: Some(row.get(0)?),
                wallet_id: row.get(1)?,
                transaction_type: row.get(2)?,
                amount: row.get(3)?,
                month: row.get(4)?,
                year: row.get(5)?,
                message_id: row.get(6)?,
                chat_id: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    pub async fn get_balance(&self, chat_id: i64, wallet_name: &str) -> Result<f64> {
        let conn = self.conn.lock().await;
        let wallet = self.get_wallet_by_name_sync(&conn, chat_id, wallet_name)?;
        Ok(wallet.current_balance)
    }

    pub async fn create_wallet(&self, chat_id: i64, name: &str) -> Result<Wallet> {
        self.get_or_create_wallet(chat_id, name).await
    }

    pub async fn wallet_exists(&self, chat_id: i64, name: &str) -> Result<bool> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT 1 FROM wallets WHERE chat_id = ?1 AND name = ?2")?;
        let exists = stmt.exists(params![chat_id, name])?;
        Ok(exists)
    }

    pub async fn add_transaction(
        &self,
        chat_id: i64,
        wallet_name: &str,
        transaction_type: &str,
        amount: f64,
        _description: &str,
        _transaction_id: &str,
    ) -> Result<()> {
        // 确保钱包存在
        let _ = self.get_or_create_wallet(chat_id, wallet_name).await?;

        // 对于简化的API，我们使用当前时间
        let now = Utc::now();
        let month = format!("{:02}", now.month());
        let year = now.year().to_string();

        self.record_transaction(
            chat_id,
            wallet_name,
            transaction_type,
            amount,
            &month,
            &year,
            None,
        )
        .await?;

        // 更新钱包余额
        let current_balance = self.get_balance(chat_id, wallet_name).await?;
        let new_balance = match transaction_type {
            "收入" | "入账" => current_balance + amount,
            "支出" | "出账" => current_balance - amount,
            _ => current_balance - amount, // 默认为支出类型
        };

        self.update_wallet_balance(chat_id, wallet_name, new_balance)
            .await?;

        Ok(())
    }

    fn get_wallet_by_name_sync(
        &self,
        conn: &Connection,
        chat_id: i64,
        name: &str,
    ) -> Result<Wallet> {
        let mut stmt = conn.prepare("SELECT id, chat_id, name, current_balance, created_at, updated_at FROM wallets WHERE chat_id = ?1 AND name = ?2")?;
        let mut wallet_iter = stmt.query_map(params![chat_id, name], |row| {
            Ok(Wallet {
                id: Some(row.get(0)?),
                chat_id: row.get(1)?,
                name: row.get(2)?,
                current_balance: row.get(3)?,
                created_at: row.get(4).ok(),
                updated_at: row.get(5).ok(),
            })
        })?;

        if let Some(wallet) = wallet_iter.next() {
            return Ok(wallet?);
        }

        Err(anyhow::anyhow!(
            "Wallet not found: {} in chat {}",
            name,
            chat_id
        ))
    }
}

// Tests will be added later

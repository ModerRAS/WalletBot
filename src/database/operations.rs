use crate::database::models::{Wallet, Transaction, Message};
use rusqlite::{Connection, Result as SqliteResult, params};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use log::{debug, info, error};
use chrono::Utc;

#[derive(Clone)]
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
                name TEXT NOT NULL UNIQUE,
                current_balance REAL NOT NULL DEFAULT 0.0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
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

    pub async fn get_or_create_wallet(&self, name: &str) -> Result<Wallet> {
        let conn = self.conn.lock().await;
        
        // 尝试获取现有钱包
        let mut stmt = conn.prepare("SELECT id, name, current_balance, created_at, updated_at FROM wallets WHERE name = ?1")?;
        let wallet_iter = stmt.query_map(params![name], |row| {
            Ok(Wallet {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                current_balance: row.get(2)?,
                created_at: row.get(3).ok(),
                updated_at: row.get(4).ok(),
            })
        })?;

        for wallet in wallet_iter {
            return Ok(wallet?);
        }

        // 如果不存在，创建新钱包
        let now = Utc::now();
        conn.execute(
            "INSERT INTO wallets (name, current_balance, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![name, 0.0, now, now],
        )?;

        let wallet_id = conn.last_insert_rowid();
        debug!("Created new wallet: {} with ID: {}", name, wallet_id);

        Ok(Wallet {
            id: Some(wallet_id),
            name: name.to_string(),
            current_balance: 0.0,
            created_at: Some(now),
            updated_at: Some(now),
        })
    }

    pub async fn update_wallet_balance(&self, name: &str, balance: f64) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now();
        
        conn.execute(
            "UPDATE wallets SET current_balance = ?1, updated_at = ?2 WHERE name = ?3",
            params![balance, now, name],
        )?;

        info!("Updated wallet balance: {} -> {}", name, balance);
        Ok(())
    }

    pub async fn record_transaction(
        &self,
        wallet_name: &str,
        transaction_type: &str,
        amount: f64,
        month: &str,
        year: &str,
        message_id: Option<i64>,
        chat_id: Option<i64>,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        
        // 获取钱包ID
        let wallet = self.get_wallet_by_name_sync(&conn, wallet_name)?;
        let wallet_id = wallet.id.unwrap();
        
        let now = Utc::now();
        conn.execute(
            "INSERT INTO transactions (wallet_id, transaction_type, amount, month, year, message_id, chat_id, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![wallet_id, transaction_type, amount, month, year, message_id, chat_id, now],
        )?;

        debug!("Recorded transaction: {} {} {}", wallet_name, transaction_type, amount);
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
        let wallet = self.get_wallet_by_name_sync(&conn, wallet_name)?;
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

    pub async fn get_latest_balance(&self, wallet_name: &str, month: &str, year: &str) -> Result<f64> {
        let conn = self.conn.lock().await;
        
        // 获取钱包
        let wallet = self.get_wallet_by_name_sync(&conn, wallet_name)?;
        
        // 返回当前余额
        Ok(wallet.current_balance)
    }

    pub async fn is_message_processed(&self, message_id: i64, chat_id: i64) -> Result<bool> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare("SELECT processed FROM messages WHERE message_id = ?1 AND chat_id = ?2")?;
        let processed_iter = stmt.query_map(params![message_id, chat_id], |row| {
            Ok(row.get::<_, bool>(0)?)
        })?;

        for processed in processed_iter {
            return Ok(processed?);
        }

        Ok(false)
    }

    fn get_wallet_by_name_sync(&self, conn: &Connection, name: &str) -> Result<Wallet> {
        let mut stmt = conn.prepare("SELECT id, name, current_balance, created_at, updated_at FROM wallets WHERE name = ?1")?;
        let wallet_iter = stmt.query_map(params![name], |row| {
            Ok(Wallet {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                current_balance: row.get(2)?,
                created_at: row.get(3).ok(),
                updated_at: row.get(4).ok(),
            })
        })?;

        for wallet in wallet_iter {
            return Ok(wallet?);
        }

        Err(anyhow::anyhow!("Wallet not found: {}", name))
    }
}

// Tests will be added later 
use crate::error::Result;
use rusqlite::{Connection, OptionalExtension};
use std::{
    default,
    sync::{LazyLock, Mutex},
};

const DB_PATH: &str = ".db";

pub static CONN: LazyLock<Mutex<Connection>> = LazyLock::new(|| {
    let conn = create_new_conn().unwrap();

    Mutex::new(conn)
});

fn create_new_conn() -> Result<Connection> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute_batch(
        r#"BEGIN;
CREATE TABLE IF NOT EXISTS product_ai_summary (
	id TEXT PRIMARY KEY,
	ai_summary TEXT NOT NULL,
	created_at INTEGER NOT NULL
);
COMMIT;"#,
    )?;

    Ok(conn)
}

#[derive(Debug)]
pub struct ProductAiSummaryRow {
    pub id: String,
    pub ai_summary: String,
    pub created_at: u64,
}

pub fn insert_or_replace_product_ai_summary(id: &str, ai_summary: &str) -> Result<()> {
    let conn = CONN.lock().unwrap();

    const SQL: &str = "INSERT OR REPLACE INTO product_ai_summary (id, ai_summary, created_at) VALUES (?1, ?2, strftime('%s','now'))";

    conn.execute(SQL, &[id, ai_summary])?;

    Ok(())
}

pub fn select_product_ai_summary(id: &str) -> Result<Option<ProductAiSummaryRow>> {
    let conn = CONN.lock().unwrap();

    const SQL: &str = "SELECT * FROM product_ai_summary WHERE id = ?1";

    let row = conn
        .query_one(SQL, &[id], |row| {
            Ok(ProductAiSummaryRow {
                id: row.get(0)?,
                ai_summary: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .optional()?;

    Ok(row)
}

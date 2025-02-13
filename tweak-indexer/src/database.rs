
use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct Block {
    pub id: i32,
    pub block_hash: String,
    pub has_tweaks: bool,
}

#[derive(Debug)]
pub struct Tweaks {
    pub id: i32,
    pub block_hash: String,
    pub tx_id: String,
    pub tweak: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                block_hash TEXT NOT NULL,
                has_tweaks BOOLEAN NOT NULL
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tweaks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                block_hash TEXT NOT NULL,
                tx_id TEXT NOT NULL,
                tweak TEXT NOT NULL,
                FOREIGN KEY(block_hash) REFERENCES blocks(block_hash)
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn insert_block(&self, block_hash: &str, has_tweaks: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO blocks (block_hash, has_tweaks) VALUES (?1, ?2)",
            params![block_hash, has_tweaks],
        )?;
        Ok(())
    }

    pub fn insert_tweak(&self, block_hash: &str, tweak: &str, tx_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tweaks (block_hash, tx_id, tweak) VALUES (?1, ?2, ?3)",
            params![block_hash, tx_id, tweak],
        )?;
        Ok(())
    }

    pub fn get_block(&self, block_hash: &str) -> Result<Vec<Block>> {
        let mut stmt = self.conn.prepare("SELECT id, block_hash, has_tweaks FROM blocks WHERE block_hash = ?1")?;
        let blocks_iter = stmt.query_map(params![block_hash], |row| {
            Ok(Block {
                id: row.get(0)?,
                block_hash: row.get(1)?,
                has_tweaks: row.get(2)?,
            })
        })?;

        Ok(blocks_iter.filter_map(Result::ok).collect())
    }

    pub fn get_tweaks(&self, block_hash: &str) -> Result<Vec<Tweaks>> {
        let mut stmt = self.conn.prepare("SELECT id, block_hash, tx_id, tweak FROM tweaks WHERE block_hash = ?1")?;
        let tweaks_iter = stmt.query_map(params![block_hash], |row| {
            Ok(Tweaks {
                id: row.get(0)?,
                block_hash: row.get(1)?,
                tx_id: row.get(2)?,
                tweak: row.get(3)?,
            })
        })?;

        Ok(tweaks_iter.filter_map(Result::ok).collect())
    }

    pub fn has_tweak(&self, tweak: &str) -> Result<Vec<Tweaks>> {
        let query = "SELECT id, block_hash, tx_id, tweak FROM tweaks WHERE tweak = ?1".to_string();
        let params: Vec<&dyn rusqlite::ToSql> = vec![&tweak];

        let mut stmt = self.conn.prepare(&query)?;
        let tweaks_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(Tweaks {
                id: row.get(0)?,
                block_hash: row.get(1)?,
                tx_id: row.get(2)?,
                tweak: row.get(3)?,
            })
        })?;

        Ok(tweaks_iter.filter_map(Result::ok).collect())
    }

    pub fn close(self) { 
        let _ = self.conn.close();
    }
}


use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct Block {
    pub height: u32,
    pub hash: String,
    pub has_tweaks: bool,
}

#[derive(Debug)]
pub struct Tweak {
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
                height INTEGER PRIMARY KEY,
                hash TEXT NOT NULL,
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
                FOREIGN KEY(block_hash) REFERENCES blocks(hash)
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn insert_block(&self, block: &Block) -> Result<()> {
        self.conn.execute(
            "INSERT INTO blocks (height, hash, has_tweaks) VALUES (?1, ?2, ?3)",
            params![block.height, block.hash, block.has_tweaks],
        )?;
        Ok(())
    }

    pub fn insert_tweak(&self, tweak: &Tweak) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tweaks (block_hash, tx_id, tweak) VALUES (?1, ?2, ?3)",
            params![tweak.block_hash, tweak.tx_id, tweak.tweak],
        )?;
        Ok(())
    }

    pub fn get_block(&self, block_hash: &str) -> Result<Vec<Block>> {
        let mut stmt = self.conn.prepare("SELECT height, hash, has_tweaks FROM blocks WHERE hash = ?1")?;
        let blocks_iter = stmt.query_map(params![block_hash], |row| {
            Ok(Block {
                height: row.get(0)?,
                hash: row.get(1)?,
                has_tweaks: row.get(2)?,
            })
        })?;

        Ok(blocks_iter.filter_map(Result::ok).collect())
    }

    pub fn get_highest_block(&self) -> Result<u32> {
        let mut stmt = self.conn.prepare("SELECT max(height) FROM blocks")?;
        let highest_block: Option<u32> = stmt.query_row([], |row| row.get(0)).ok();

        Ok(highest_block.unwrap_or(0))
    }

    pub fn get_tweaks(&self, block_hash: &str) -> Result<Vec<Tweak>> {
        let mut stmt = self.conn.prepare("SELECT block_hash, tx_id, tweak FROM tweaks WHERE block_hash = ?1")?;
        let tweaks_iter = stmt.query_map(params![block_hash], |row| {
            Ok(Tweak {
                block_hash: row.get(0)?,
                tx_id: row.get(1)?,
                tweak: row.get(2)?,
            })
        })?;

        Ok(tweaks_iter.filter_map(Result::ok).collect())
    }

    pub fn has_tweak(&self, tweak: &str) -> Result<Vec<Tweak>> {
        let query = "SELECT block_hash, tx_id, tweak FROM tweaks WHERE tweak = ?1".to_string();
        let params: Vec<&dyn rusqlite::ToSql> = vec![&tweak];

        let mut stmt = self.conn.prepare(&query)?;
        let tweaks_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(Tweak {
                block_hash: row.get(0)?,
                tx_id: row.get(1)?,
                tweak: row.get(2)?,
            })
        })?;

        Ok(tweaks_iter.filter_map(Result::ok).collect())
    }

    pub fn close(self) { 
        let _ = self.conn.close();
    }
}

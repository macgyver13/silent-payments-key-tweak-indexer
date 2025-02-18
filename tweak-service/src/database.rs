
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Tweak {
    pub block_hash: String,
    pub tx_id: String,
    pub tweak: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TweakMetrics {
    pub block_hash: String,
    pub tweak_count: u32,
}

// Function to fetch tweaks from SQLite
pub fn fetch_tweaks(block_hash: String, db_path: &String) -> Result<Vec<Tweak>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT block_hash, tx_id, tweak FROM tweaks WHERE block_hash = ?1")?;
    let tweaks_iter = stmt.query_map(params![block_hash], |row| {
        Ok(Tweak {
            block_hash: row.get(0)?,
            tx_id: row.get(1)?,
            tweak: row.get(2)?,
        })
    })?;
    
    let tweaks = tweaks_iter.filter_map(Result::ok).collect();
    Ok(tweaks)
}

pub fn get_tweak_metrics(db_path: &String) -> Result<Vec<TweakMetrics>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT block_hash, count(tweak) FROM tweaks GROUP BY block_hash order by count(tweak) desc")?;
    let tweaks_iter = stmt.query_map(params![], |row| {
        Ok(TweakMetrics {
            block_hash: row.get(0)?,
            tweak_count: row.get(1)?,
        })
    })?;
    
    let tweaks = tweaks_iter.filter_map(Result::ok).collect();
    Ok(tweaks)
}

pub fn get_highest_block(db_path: &String) -> Result<u32> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT max(height) FROM blocks")?;
    let highest_block: Option<u32> = stmt.query_row([], |row| row.get(0)).ok();

    Ok(highest_block.unwrap_or(0))
}
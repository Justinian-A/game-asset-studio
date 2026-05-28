use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetRecord {
    pub id: i64,
    pub source_id: String,
    pub title: String,
    pub author: String,
    pub url: String,
    pub preview_url: Option<String>,
    pub local_path: Option<String>,
    pub price: Option<String>,
    pub license: String,
    pub short_text: String,
    pub tags: String,
    pub asset_type: String,
    pub source: String,
    pub downloaded: bool,
    pub favorite: bool,
    pub created_at: String,
}

pub fn init_database(db_path: &Path) -> Result<()> {
    let conn = Connection::open(db_path)?;
    
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            author TEXT DEFAULT '',
            url TEXT NOT NULL,
            preview_url TEXT,
            local_path TEXT,
            price TEXT DEFAULT 'Free',
            license TEXT DEFAULT '',
            short_text TEXT DEFAULT '',
            tags TEXT DEFAULT '',
            asset_type TEXT DEFAULT '2d',
            source TEXT NOT NULL,
            downloaded INTEGER DEFAULT 0,
            favorite INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now'))
        );
        
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        
        CREATE TABLE IF NOT EXISTS asset_tags (
            asset_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (asset_id, tag_id),
            FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_assets_source ON assets(source);
        CREATE INDEX IF NOT EXISTS idx_assets_downloaded ON assets(downloaded);
        CREATE INDEX IF NOT EXISTS idx_assets_favorite ON assets(favorite);
        CREATE INDEX IF NOT EXISTS idx_assets_title ON assets(title);
        "
    )?;
    
    log::info!("Database initialized successfully");
    Ok(())
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn insert_asset(&self, asset: &AssetRecord) -> Result<i64> {
        // First check if asset already exists
        let existing_id: Option<i64> = self.conn.query_row(
            "SELECT id FROM assets WHERE source_id = ?1",
            params![asset.source_id],
            |row| row.get(0),
        ).ok();
        
        if let Some(id) = existing_id {
            // Asset exists, return its id
            return Ok(id);
        }
        
        // Insert new asset
        self.conn.execute(
            "INSERT INTO assets (source_id, title, author, url, preview_url, local_path, price, license, short_text, tags, asset_type, source, downloaded, favorite)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                asset.source_id,
                asset.title,
                asset.author,
                asset.url,
                asset.preview_url,
                asset.local_path,
                asset.price,
                asset.license,
                asset.short_text,
                asset.tags,
                asset.asset_type,
                asset.source,
                asset.downloaded,
                asset.favorite,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_asset(&self, id: i64) -> Result<Option<AssetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_id, title, author, url, preview_url, local_path, price, license, short_text, tags, asset_type, source, downloaded, favorite, created_at FROM assets WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(AssetRecord {
                id: row.get(0)?,
                source_id: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                url: row.get(4)?,
                preview_url: row.get(5)?,
                local_path: row.get(6)?,
                price: row.get(7)?,
                license: row.get(8)?,
                short_text: row.get(9)?,
                tags: row.get(10)?,
                asset_type: row.get(11)?,
                source: row.get(12)?,
                downloaded: row.get(13)?,
                favorite: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;
        
        rows.next().transpose()
    }

    pub fn get_assets(&self, offset: i64, limit: i64) -> Result<Vec<AssetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_id, title, author, url, preview_url, local_path, price, license, short_text, tags, asset_type, source, downloaded, favorite, created_at FROM assets ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        )?;
        
        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok(AssetRecord {
                id: row.get(0)?,
                source_id: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                url: row.get(4)?,
                preview_url: row.get(5)?,
                local_path: row.get(6)?,
                price: row.get(7)?,
                license: row.get(8)?,
                short_text: row.get(9)?,
                tags: row.get(10)?,
                asset_type: row.get(11)?,
                source: row.get(12)?,
                downloaded: row.get(13)?,
                favorite: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;
        
        let mut assets = Vec::new();
        for row in rows {
            assets.push(row?);
        }
        Ok(assets)
    }

    pub fn search_assets(&self, query: &str) -> Result<Vec<AssetRecord>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, source_id, title, author, url, preview_url, local_path, price, license, short_text, tags, asset_type, source, downloaded, favorite, created_at FROM assets WHERE title LIKE ?1 OR author LIKE ?1 OR tags LIKE ?1 ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map(params![search_pattern], |row| {
            Ok(AssetRecord {
                id: row.get(0)?,
                source_id: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                url: row.get(4)?,
                preview_url: row.get(5)?,
                local_path: row.get(6)?,
                price: row.get(7)?,
                license: row.get(8)?,
                short_text: row.get(9)?,
                tags: row.get(10)?,
                asset_type: row.get(11)?,
                source: row.get(12)?,
                downloaded: row.get(13)?,
                favorite: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;
        
        let mut assets = Vec::new();
        for row in rows {
            assets.push(row?);
        }
        Ok(assets)
    }

    pub fn get_favorites(&self) -> Result<Vec<AssetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_id, title, author, url, preview_url, local_path, price, license, short_text, tags, asset_type, source, downloaded, favorite, created_at FROM assets WHERE favorite = 1 ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(AssetRecord {
                id: row.get(0)?,
                source_id: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                url: row.get(4)?,
                preview_url: row.get(5)?,
                local_path: row.get(6)?,
                price: row.get(7)?,
                license: row.get(8)?,
                short_text: row.get(9)?,
                tags: row.get(10)?,
                asset_type: row.get(11)?,
                source: row.get(12)?,
                downloaded: row.get(13)?,
                favorite: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;
        
        let mut assets = Vec::new();
        for row in rows {
            assets.push(row?);
        }
        Ok(assets)
    }

    pub fn get_downloaded(&self) -> Result<Vec<AssetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_id, title, author, url, preview_url, local_path, price, license, short_text, tags, asset_type, source, downloaded, favorite, created_at FROM assets WHERE downloaded = 1 ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(AssetRecord {
                id: row.get(0)?,
                source_id: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                url: row.get(4)?,
                preview_url: row.get(5)?,
                local_path: row.get(6)?,
                price: row.get(7)?,
                license: row.get(8)?,
                short_text: row.get(9)?,
                tags: row.get(10)?,
                asset_type: row.get(11)?,
                source: row.get(12)?,
                downloaded: row.get(13)?,
                favorite: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;
        
        let mut assets = Vec::new();
        for row in rows {
            assets.push(row?);
        }
        Ok(assets)
    }

    pub fn update_favorite(&self, id: i64, favorite: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE assets SET favorite = ?1 WHERE id = ?2",
            params![favorite, id],
        )?;
        Ok(())
    }

    pub fn update_downloaded(&self, id: i64, downloaded: bool, local_path: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE assets SET downloaded = ?1, local_path = ?2 WHERE id = ?3",
            params![downloaded, local_path, id],
        )?;
        Ok(())
    }

    pub fn delete_asset(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM assets WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_asset_count(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM assets",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_stats(&self) -> Result<(i64, i64, i64)> {
        let total: i64 = self.conn.query_row("SELECT COUNT(*) FROM assets", [], |row| row.get(0))?;
        let downloaded: i64 = self.conn.query_row("SELECT COUNT(*) FROM assets WHERE downloaded = 1", [], |row| row.get(0))?;
        let favorites: i64 = self.conn.query_row("SELECT COUNT(*) FROM assets WHERE favorite = 1", [], |row| row.get(0))?;
        Ok((total, downloaded, favorites))
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}

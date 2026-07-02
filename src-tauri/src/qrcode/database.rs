use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

const MAX_GENERATE: i64 = 30;
const MAX_DECODE: i64 = 50;

fn time_filter_clause(time_filter: Option<&str>) -> &'static str {
    match time_filter {
        Some("today") => " AND date(created_at) = date('now', 'localtime')",
        Some("yesterday") => " AND date(created_at) = date('now', 'localtime', '-1 day')",
        Some("week") => " AND created_at >= datetime('now', 'localtime', '-7 days')",
        Some("month") => " AND created_at >= datetime('now', 'localtime', '-30 days')",
        _ => "",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrGeneratePreview {
    pub id: i64,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrGenerateDetail {
    pub id: i64,
    pub text: String,
    pub png_data: Vec<u8>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrDecodeItem {
    pub id: i64,
    pub text: String,
    pub file_name: String,
    pub created_at: String,
}

pub struct QrcodeDatabase {
    conn: Connection,
}

impl QrcodeDatabase {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS qr_generate_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_text TEXT NOT NULL,
                png_image BLOB NOT NULL,
                created_at DATETIME DEFAULT (datetime('now', 'localtime'))
            );
            CREATE TABLE IF NOT EXISTS qr_decode_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_text TEXT NOT NULL,
                file_name TEXT NOT NULL,
                created_at DATETIME DEFAULT (datetime('now', 'localtime'))
            );
            CREATE INDEX IF NOT EXISTS idx_qr_generate_created ON qr_generate_items(created_at);
            CREATE INDEX IF NOT EXISTS idx_qr_decode_created ON qr_decode_items(created_at);",
        )?;
        Ok(())
    }

    pub fn add_generate(&self, text: &str, png: &[u8]) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO qr_generate_items (content_text, png_image) VALUES (?1, ?2)",
            params![text, png],
        )?;
        let id = self.conn.last_insert_rowid();
        self.trim_generate()?;
        Ok(id)
    }

    pub fn list_generate(
        &self,
        time_filter: Option<&str>,
        search_query: Option<&str>,
    ) -> Result<Vec<QrGeneratePreview>> {
        let mut query = String::from(
            "SELECT id, content_text, created_at FROM qr_generate_items WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(sq) = search_query {
            if !sq.is_empty() {
                let idx = param_values.len() + 1;
                query.push_str(&format!(" AND content_text LIKE ?{idx}"));
                param_values.push(Box::new(format!("%{sq}%")));
            }
        }

        if time_filter.is_some_and(|tf| tf != "all") {
            query.push_str(time_filter_clause(time_filter));
        }

        let idx = param_values.len() + 1;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ?{idx}"));
        param_values.push(Box::new(MAX_GENERATE));

        let mut stmt = self.conn.prepare(&query)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let items = stmt
            .query_map(params_ref.as_slice(), |row| {
                Ok(QrGeneratePreview {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn count_generate(
        &self,
        time_filter: Option<&str>,
        search_query: Option<&str>,
    ) -> Result<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM qr_generate_items WHERE 1=1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(sq) = search_query {
            if !sq.is_empty() {
                let idx = param_values.len() + 1;
                query.push_str(&format!(" AND content_text LIKE ?{idx}"));
                param_values.push(Box::new(format!("%{sq}%")));
            }
        }

        if time_filter.is_some_and(|tf| tf != "all") {
            query.push_str(time_filter_clause(time_filter));
        }

        let mut stmt = self.conn.prepare(&query)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        stmt.query_row(params_ref.as_slice(), |row| row.get(0))
    }

    pub fn get_generate(&self, id: i64) -> Result<Option<QrGenerateDetail>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_text, png_image, created_at FROM qr_generate_items WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(QrGenerateDetail {
                id: row.get(0)?,
                text: row.get(1)?,
                png_data: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn delete_generate(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM qr_generate_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_generate(&self) -> Result<()> {
        self.conn.execute("DELETE FROM qr_generate_items", [])?;
        Ok(())
    }

    pub fn add_decode(&self, text: &str, file_name: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO qr_decode_items (content_text, file_name) VALUES (?1, ?2)",
            params![text, file_name],
        )?;
        let id = self.conn.last_insert_rowid();
        self.trim_decode()?;
        Ok(id)
    }

    pub fn list_decode(
        &self,
        time_filter: Option<&str>,
        search_query: Option<&str>,
    ) -> Result<Vec<QrDecodeItem>> {
        let mut query = String::from(
            "SELECT id, content_text, file_name, created_at FROM qr_decode_items WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(sq) = search_query {
            if !sq.is_empty() {
                let idx = param_values.len() + 1;
                query.push_str(&format!(
                    " AND (content_text LIKE ?{idx} OR file_name LIKE ?{})",
                    idx + 1
                ));
                param_values.push(Box::new(format!("%{sq}%")));
                param_values.push(Box::new(format!("%{sq}%")));
            }
        }

        if time_filter.is_some_and(|tf| tf != "all") {
            query.push_str(time_filter_clause(time_filter));
        }

        let idx = param_values.len() + 1;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ?{idx}"));
        param_values.push(Box::new(MAX_DECODE));

        let mut stmt = self.conn.prepare(&query)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let items = stmt
            .query_map(params_ref.as_slice(), |row| {
                Ok(QrDecodeItem {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    file_name: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn count_decode(
        &self,
        time_filter: Option<&str>,
        search_query: Option<&str>,
    ) -> Result<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM qr_decode_items WHERE 1=1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(sq) = search_query {
            if !sq.is_empty() {
                let idx = param_values.len() + 1;
                query.push_str(&format!(
                    " AND (content_text LIKE ?{idx} OR file_name LIKE ?{})",
                    idx + 1
                ));
                param_values.push(Box::new(format!("%{sq}%")));
                param_values.push(Box::new(format!("%{sq}%")));
            }
        }

        if time_filter.is_some_and(|tf| tf != "all") {
            query.push_str(time_filter_clause(time_filter));
        }

        let mut stmt = self.conn.prepare(&query)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        stmt.query_row(params_ref.as_slice(), |row| row.get(0))
    }

    pub fn get_decode(&self, id: i64) -> Result<Option<QrDecodeItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_text, file_name, created_at FROM qr_decode_items WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(QrDecodeItem {
                id: row.get(0)?,
                text: row.get(1)?,
                file_name: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn delete_decode(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM qr_decode_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_decode(&self) -> Result<()> {
        self.conn.execute("DELETE FROM qr_decode_items", [])?;
        Ok(())
    }

    fn trim_generate(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM qr_generate_items", [], |row| row.get(0))?;
        if count <= MAX_GENERATE {
            return Ok(());
        }
        let excess = count - MAX_GENERATE;
        self.conn.execute(
            "DELETE FROM qr_generate_items WHERE id IN (
                SELECT id FROM qr_generate_items ORDER BY created_at ASC LIMIT ?1
            )",
            params![excess],
        )?;
        Ok(())
    }

    fn trim_decode(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM qr_decode_items", [], |row| row.get(0))?;
        if count <= MAX_DECODE {
            return Ok(());
        }
        let excess = count - MAX_DECODE;
        self.conn.execute(
            "DELETE FROM qr_decode_items WHERE id IN (
                SELECT id FROM qr_decode_items ORDER BY created_at ASC LIMIT ?1
            )",
            params![excess],
        )?;
        Ok(())
    }
}

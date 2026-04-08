use mysql::prelude::*;
use mysql::{Pool, OptsBuilder, Result as MysqlResult};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub path: String,
    pub hash: String,
    pub last_checked: String,
}

pub struct Database {
    pool: Pool,
}

impl Database {
    /// Creates a new database connection pool
    pub fn new(
        host: &str,
        user: &str,
        password: &str,
        database: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(host))
            .user(Some(user))
            .pass(Some(password))
            .db_name(Some(database));
        
        let pool = Pool::new(opts)?;

        // Verify connection and initialize schema
        let mut conn = pool.get_conn()?;
        conn.exec_drop(
            "CREATE TABLE IF NOT EXISTS files (
                path VARCHAR(4096) PRIMARY KEY,
                hash VARCHAR(64) NOT NULL,
                last_checked TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )",
            (),
        )?;

        Ok(Database { pool })
    }

    /// Fetches all file records from the database
    pub fn get_all_files(&self) -> MysqlResult<Vec<FileRecord>> {
        let mut conn = self.pool.get_conn()?;
        conn.query_map(
            "SELECT path, hash, last_checked FROM files",
            |(path, hash, last_checked)| FileRecord {
                path,
                hash,
                last_checked,
            },
        )
    }

    /// Fetches a single file record by path
    pub fn get_file(&self, path: &str) -> MysqlResult<Option<FileRecord>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_map(
            "SELECT path, hash, last_checked FROM files WHERE path = ?",
            (path,),
            |(path, hash, last_checked)| FileRecord {
                path,
                hash,
                last_checked,
            },
        )
        .map(|mut result| result.pop())
    }

    /// Inserts a new file record
    pub fn insert_file(&self, path: &str, hash: &str) -> MysqlResult<()> {
        let mut conn = self.pool.get_conn()?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.exec_drop(
            "INSERT INTO files (path, hash, last_checked) VALUES (?, ?, ?)",
            (path, hash, now),
        )?;
        Ok(())
    }

    /// Updates an existing file record
    pub fn update_file(&self, path: &str, hash: &str) -> MysqlResult<()> {
        let mut conn = self.pool.get_conn()?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.exec_drop(
            "UPDATE files SET hash = ?, last_checked = ? WHERE path = ?",
            (hash, now, path),
        )?;
        Ok(())
    }

    /// Deletes a file record
    pub fn delete_file(&self, path: &str) -> MysqlResult<()> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop("DELETE FROM files WHERE path = ?", (path,))?;
        Ok(())
    }

    /// Checks if a file exists in the database
    pub fn file_exists(&self, path: &str) -> MysqlResult<bool> {
        let mut conn = self.pool.get_conn()?;
        let count: i64 = conn.exec_first(
            "SELECT COUNT(*) FROM files WHERE path = ?",
            (path,),
        )?
        .unwrap_or(0);
        Ok(count > 0)
    }

    /// Gets all paths that are in the database
    pub fn get_all_paths(&self) -> MysqlResult<Vec<String>> {
        let mut conn = self.pool.get_conn()?;
        conn.query("SELECT path FROM files")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running MySQL server.
    // They are commented out by default.
    // To run tests, ensure MySQL is running and accessible with:
    // RUST_LOG=debug cargo test -- --test-threads=1

    // #[test]
    // fn test_database_connection() {
    //     let db = Database::new("localhost", "root", "password", "test_fim");
    //     // Would require actual database setup
    // }
}

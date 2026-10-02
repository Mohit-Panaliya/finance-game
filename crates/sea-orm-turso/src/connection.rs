use async_trait::async_trait;
use sea_orm::{
    AccessMode, ConnectionTrait, DbBackend, ExecResult, IsolationLevel, ProxyExecResult, ProxyRow,
    QueryResult, Statement, TransactionError, TransactionOptions, TransactionTrait,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use turso::{Builder, Connection, Database};

use crate::error::TursoError;
use crate::row::{row_to_proxy_row, statement_to_sql};

/// SQLite column type from PRAGMA table_info
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    Boolean,
    Integer,
    Real,
    Text,
    Blob,
    DateTime,
    Unknown(String),
}

impl ColumnType {
    pub fn from_sqlite_decl(decl: &str) -> Self {
        let upper = decl.to_uppercase();
        if upper.contains("BOOL") {
            ColumnType::Boolean
        } else if upper.contains("INT") {
            ColumnType::Integer
        } else if upper.contains("REAL")
            || upper.contains("FLOAT")
            || upper.contains("DOUBLE")
            || upper.contains("NUMERIC")
        {
            ColumnType::Real
        } else if upper.contains("TEXT") || upper.contains("CLOB") || upper.contains("CHAR") {
            ColumnType::Text
        } else if upper.contains("BLOB") {
            ColumnType::Blob
        } else if upper.contains("DATE") || upper.contains("TIME") {
            ColumnType::DateTime
        } else {
            ColumnType::Unknown(decl.to_string())
        }
    }
}

/// A SeaORM-compatible connection backed by Turso Database
///
/// This struct implements SeaORM's [`ConnectionTrait`] and [`TransactionTrait`],
/// allowing you to use Turso as a drop-in replacement for any SeaORM-supported
/// database.
///
/// # Features
///
/// - Full MVCC support via Turso engine
/// - In-memory performance with WAL persistence
/// - Async/await native support
/// - Compatible with all SeaORM entities and queries
/// - Schema-aware type conversion (booleans, datetimes, etc.)
#[derive(Clone)]
pub struct TursoConnection {
    db_path: String,
    db: Arc<Database>,
    conn: Arc<Mutex<Connection>>,
    schema_cache: Arc<Mutex<HashMap<String, HashMap<String, ColumnType>>>>,
}

impl TursoConnection {
    /// Connect to a Turso database file
    ///
    /// Creates the file if it doesn't exist. The database uses MVCC
    /// with WAL mode for concurrent access and persistence.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database file (e.g., "app.db")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use sea_orm_turso::TursoConnection;
    ///
    /// let db = TursoConnection::connect("my_database.db").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(path: &str) -> Result<Self, TursoError> {
        let db = Builder::new_local(path)
            .build()
            .await
            .map_err(|e| TursoError::Connection(e.to_string()))?;

        let conn = db
            .connect()
            .map_err(|e| TursoError::Connection(e.to_string()))?;

        Ok(Self {
            db_path: path.to_string(),
            db: Arc::new(db),
            conn: Arc::new(Mutex::new(conn)),
            schema_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Connect to an in-memory Turso database (no persistence)
    ///
    /// Useful for testing or ephemeral data.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use sea_orm_turso::TursoConnection;
    ///
    /// let db = TursoConnection::connect_memory().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_memory() -> Result<Self, TursoError> {
        Self::connect(":memory:").await
    }

    /// Create a TursoConnection from an existing Turso database and connection
    ///
    /// This is useful when you need to share a Turso database instance
    /// or create multiple connections to the same database.
    pub fn from_existing(db: Database, conn: Connection) -> Self {
        Self {
            db_path: String::new(),
            db: Arc::new(db),
            conn: Arc::new(Mutex::new(conn)),
            schema_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a reference to the underlying Turso database
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Flush cached data to disk (WAL checkpoint)
    ///
    /// Call this after write operations to ensure data persists to disk.
    /// The turso engine buffers writes in memory; cacheflush forces them to the WAL file.
    pub async fn cacheflush(&self) -> Result<(), TursoError> {
        let conn = self.conn.lock().await;
        conn.cacheflush()
            .map_err(|e| TursoError::Query(format!("cacheflush failed: {}", e)))?;
        Ok(())
    }

    /// Persist data to disk by creating a temporary Database on the same path and dropping it.
    ///
    /// Turso's local engine only syncs the WAL to disk when a Database is dropped.
    /// This creates a second Database instance (which reads existing data), then drops it
    /// to trigger the WAL sync. The main Database continues to work normally.
    pub async fn persist(&self) -> Result<(), TursoError> {
        if self.db_path.is_empty() || self.db_path == ":memory:" {
            return Ok(());
        }

        // Create a temporary Database on the same path — this opens the existing WAL
        let tmp_db = Builder::new_local(&self.db_path)
            .build()
            .await
            .map_err(|e| TursoError::Connection(format!("persist failed: {}", e)))?;

        let tmp_conn = tmp_db
            .connect()
            .map_err(|e| TursoError::Connection(format!("persist failed: {}", e)))?;

        // Run cacheflush on the temp connection to ensure all dirty pages are written
        tmp_conn
            .cacheflush()
            .map_err(|e| TursoError::Query(format!("persist cacheflush failed: {}", e)))?;

        // Dropping tmp_db triggers WAL sync to disk
        drop(tmp_conn);
        drop(tmp_db);

        Ok(())
    }

    /// Load schema info for a table via PRAGMA table_info
    async fn load_table_schema(&self, table_name: &str) -> Result<(), TursoError> {
        let mut cache = self.schema_cache.lock().await;
        if cache.contains_key(table_name) {
            return Ok(());
        }

        let conn = self.conn.lock().await;
        let mut prepared = conn
            .prepare(&format!("PRAGMA table_info({})", table_name))
            .await
            .map_err(|e| TursoError::Query(e.to_string()))?;
        let mut rows = prepared
            .query(())
            .await
            .map_err(|e| TursoError::Query(e.to_string()))?;

        let mut columns = HashMap::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| TursoError::Query(e.to_string()))?
        {
            let name = row
                .get_value(1)
                .map_err(|e| TursoError::Query(e.to_string()))?;
            let type_str = row
                .get_value(2)
                .map_err(|e| TursoError::Query(e.to_string()))?;

            if let turso::Value::Text(col_name) = name {
                if let turso::Value::Text(col_type) = type_str {
                    let upper = col_type.to_uppercase();
                    // SQLite stores booleans as INTEGER. Check common patterns:
                    // 1. Explicit BOOLEAN/BOOL type declaration
                    // 2. We'll also check column name patterns in the row converter
                    let ct = if upper.contains("BOOL") {
                        ColumnType::Boolean
                    } else {
                        ColumnType::from_sqlite_decl(&col_type)
                    };
                    columns.insert(col_name, ct);
                }
            }
        }

        cache.insert(table_name.to_string(), columns);
        Ok(())
    }

    /// Extract table names from a SQL query (simple parser)
    fn extract_table_names(sql: &str) -> Vec<String> {
        let mut tables = Vec::new();
        let upper = sql.to_uppercase();

        // Look for FROM and JOIN clauses
        for keyword in &["FROM", "JOIN"] {
            if let Some(pos) = upper.find(keyword) {
                let rest = &sql[pos + keyword.len()..].trim_start();
                // Extract the table name (before any space, comma, or WHERE)
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '"')
                    .collect();
                let name = name.trim_matches('"').to_string();
                if !name.is_empty() && !name.eq_ignore_ascii_case("DUAL") {
                    tables.push(name);
                }
            }
        }
        tables
    }

    /// Execute a SeaORM Statement and return rows as ProxyRow
    async fn execute_statement(&self, stmt: &Statement) -> Result<Vec<ProxyRow>, TursoError> {
        let (sql, params) = statement_to_sql(stmt);

        // Load schema for referenced tables
        for table in Self::extract_table_names(&sql) {
            self.load_table_schema(&table).await?;
        }

        let conn = self.conn.lock().await;

        // Use prepared statement to get column names
        let mut prepared = conn
            .prepare(&sql)
            .await
            .map_err(|e| TursoError::Query(e.to_string()))?;

        let column_names: Vec<String> = prepared
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        // Execute query
        let mut rows = if params.is_empty() {
            prepared
                .query(())
                .await
                .map_err(|e| TursoError::Query(e.to_string()))?
        } else {
            prepared
                .query(params)
                .await
                .map_err(|e| TursoError::Query(e.to_string()))?
        };

        // Collect results
        let schema_cache = self.schema_cache.lock().await;
        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| TursoError::Query(e.to_string()))?
        {
            let proxy_row = row_to_proxy_row(&row, &column_names, &schema_cache)
                .map_err(TursoError::Conversion)?;
            results.push(proxy_row);
        }

        Ok(results)
    }
}

#[async_trait]
impl ConnectionTrait for TursoConnection {
    fn get_database_backend(&self) -> DbBackend {
        DbBackend::Sqlite
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, sea_orm::DbErr> {
        let (sql, params) = statement_to_sql(&stmt);

        let (last_insert_id, rows_affected) = {
            let conn = self.conn.lock().await;

            let rows_affected = if params.is_empty() {
                conn.execute(&sql, ())
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?
            } else {
                conn.execute(&sql, params)
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?
            };

            let last_insert_id = conn.last_insert_rowid() as u64;
            (last_insert_id, rows_affected)
        };

        // Auto-flush + persist to disk after write operations
        let _ = self.cacheflush().await;
        let _ = self.persist().await;

        Ok(ExecResult::from(ProxyExecResult::new(
            last_insert_id,
            rows_affected,
        )))
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, sea_orm::DbErr> {
        let (last_insert_id, rows_affected) = {
            let conn = self.conn.lock().await;

            let rows_affected = conn
                .execute(sql, ())
                .await
                .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

            let last_insert_id = conn.last_insert_rowid() as u64;
            (last_insert_id, rows_affected)
        };

        // Auto-flush + persist to disk after write operations
        let _ = self.cacheflush().await;
        let _ = self.persist().await;

        Ok(ExecResult::from(ProxyExecResult::new(
            last_insert_id,
            rows_affected,
        )))
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, sea_orm::DbErr> {
        let mut results = self
            .execute_statement(&stmt)
            .await
            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

        Ok(results.pop().map(|r| r.into()))
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, sea_orm::DbErr> {
        let results = self
            .execute_statement(&stmt)
            .await
            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

        Ok(results.into_iter().map(|r| r.into()).collect())
    }
}

/// Transaction wrapper for Turso
pub struct TursoTransaction {
    conn: TursoConnection,
}

#[async_trait]
impl ConnectionTrait for TursoTransaction {
    fn get_database_backend(&self) -> DbBackend {
        DbBackend::Sqlite
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, sea_orm::DbErr> {
        self.conn.execute_raw(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, sea_orm::DbErr> {
        self.conn.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, sea_orm::DbErr> {
        self.conn.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, sea_orm::DbErr> {
        self.conn.query_all_raw(stmt).await
    }
}

#[async_trait]
impl sea_orm::TransactionSession for TursoTransaction {
    async fn commit(self) -> Result<(), sea_orm::DbErr> {
        {
            let conn = self.conn.conn.lock().await;
            conn.execute("COMMIT", ())
                .await
                .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to commit: {}", e)))?;
        }
        // Flush committed transaction to disk
        let _ = self.conn.cacheflush().await;
        let _ = self.conn.persist().await;
        Ok(())
    }

    async fn rollback(self) -> Result<(), sea_orm::DbErr> {
        let conn = self.conn.conn.lock().await;
        conn.execute("ROLLBACK", ())
            .await
            .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to rollback: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl TransactionTrait for TursoConnection {
    type Transaction = TursoTransaction;

    async fn begin(&self) -> Result<Self::Transaction, sea_orm::DbErr> {
        let conn = self.conn.lock().await;
        conn.unchecked_transaction()
            .await
            .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to begin transaction: {}", e)))?;
        drop(conn);

        Ok(TursoTransaction { conn: self.clone() })
    }

    async fn begin_with_config(
        &self,
        _isolation_level: Option<IsolationLevel>,
        _access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, sea_orm::DbErr> {
        self.begin().await
    }

    async fn begin_with_options(
        &self,
        _options: TransactionOptions,
    ) -> Result<Self::Transaction, sea_orm::DbErr> {
        self.begin().await
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'c>,
            > + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let txn = self.begin().await.map_err(TransactionError::Connection)?;
        let result = callback(&txn)
            .await
            .map_err(TransactionError::Transaction)?;
        Ok(result)
    }

    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        _isolation_level: Option<IsolationLevel>,
        _access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'c>,
            > + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let txn = self.begin().await.map_err(TransactionError::Connection)?;
        let result = callback(&txn)
            .await
            .map_err(TransactionError::Transaction)?;
        Ok(result)
    }
}

#[async_trait]
impl TransactionTrait for TursoTransaction {
    type Transaction = TursoTransaction;

    async fn begin(&self) -> Result<Self::Transaction, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Custom(
            "Nested transactions not supported".to_string(),
        ))
    }

    async fn begin_with_config(
        &self,
        _isolation_level: Option<IsolationLevel>,
        _access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, sea_orm::DbErr> {
        self.begin().await
    }

    async fn begin_with_options(
        &self,
        _options: TransactionOptions,
    ) -> Result<Self::Transaction, sea_orm::DbErr> {
        self.begin().await
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'c>,
            > + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let result = callback(self)
            .await
            .map_err(TransactionError::Transaction)?;
        Ok(result)
    }

    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        _isolation_level: Option<IsolationLevel>,
        _access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'c>,
            > + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let result = callback(self)
            .await
            .map_err(TransactionError::Transaction)?;
        Ok(result)
    }
}

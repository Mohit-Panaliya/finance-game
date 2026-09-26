use std::fmt;

/// Error type for Turso database operations
#[derive(Debug)]
pub enum TursoError {
    /// Database connection or initialization error
    Connection(String),
    /// SQL query execution error
    Query(String),
    /// Parameter binding error
    Parameters(String),
    /// Transaction error
    Transaction(String),
    /// Row conversion error
    Conversion(String),
    /// SeaORM error
    SeaOrm(sea_orm::DbErr),
}

impl fmt::Display for TursoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TursoError::Connection(msg) => write!(f, "Turso connection error: {}", msg),
            TursoError::Query(msg) => write!(f, "Turso query error: {}", msg),
            TursoError::Parameters(msg) => write!(f, "Turso parameter error: {}", msg),
            TursoError::Transaction(msg) => write!(f, "Turso transaction error: {}", msg),
            TursoError::Conversion(msg) => write!(f, "Turso conversion error: {}", msg),
            TursoError::SeaOrm(err) => write!(f, "SeaORM error: {}", err),
        }
    }
}

impl std::error::Error for TursoError {}

impl From<turso::Error> for TursoError {
    fn from(err: turso::Error) -> Self {
        TursoError::Query(err.to_string())
    }
}

impl From<sea_orm::DbErr> for TursoError {
    fn from(err: sea_orm::DbErr) -> Self {
        TursoError::SeaOrm(err)
    }
}

impl From<TursoError> for sea_orm::DbErr {
    fn from(err: TursoError) -> Self {
        sea_orm::DbErr::Custom(err.to_string())
    }
}

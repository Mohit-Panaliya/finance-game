# sea-orm-turso

A universal [SeaORM](https://docs.rs/sea-orm) adapter for [Turso Database](https://turso.tech), providing MVCC, in-process, async database access.

Implements SeaORM's [`ConnectionTrait`] and [`TransactionTrait`], so it works as a drop-in replacement for any SeaORM-supported database.

## Features

- **MVCC**: Full multi-version concurrency control via Turso engine
- **In-process**: No network overhead, runs directly in your application
- **Async**: Native async/await support via Tokio
- **WAL persistence**: In-memory speed with file-based durability
- **Universal**: Works with any SeaORM-based framework (Loco, Axum, etc.)

## Installation

```toml
[dependencies]
sea-orm-turso = "0.1"
sea-orm = "2.0"
```

## Quick Start

```rust
use sea_orm_turso::TursoConnection;
use sea_orm::{EntityTrait, QueryFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a local Turso database
    let db = TursoConnection::connect("app.db").await?;

    // Use with SeaORM entities
    // let users = user::Entity::find().all(&db).await?;

    Ok(())
}
```

## Usage with SeaORM

The `TursoConnection` implements SeaORM's `ConnectionTrait` and `TransactionTrait`, so it works seamlessly with all SeaORM APIs:

```rust
use sea_orm::{EntityTrait, QueryFilter, ActiveModelTrait, Set};

// Query
let users = user::Entity::find()
    .filter(user::Column::Name.contains("Bob"))
    .all(&db)
    .await?;

// Insert
let new_user = user::ActiveModel {
    name: Set("Alice".to_string()),
    email: Set("alice@example.com".to_string()),
    ..Default::default()
};
new_user.insert(&db).await?;

// Update
let mut active: user::ActiveModel = existing_model.into();
active.name = Set("Bob".to_string());
active.update(&db).await?;

// Delete
existing_model.delete(&db).await?;

// Transaction
db.transaction(|txn| {
    Box::pin(async move {
        model1.insert(txn).await?;
        model2.insert(txn).await?;
        Ok(())
    })
}).await?;
```

## Integration with Loco

```rust
use sea_orm_turso::TursoConnection;

// In your app.rs after_context hook:
async fn after_context(mut ctx: AppContext) -> Result<AppContext> {
    let turso = TursoConnection::connect("app.db").await.unwrap();
    ctx.shared_store.insert(turso);
    Ok(ctx)
}
```

## Persistence

Turso's local engine uses WAL (Write-Ahead Logging) for persistence. Data is stored
in-memory for fast reads and periodically synced to disk via WAL. The adapter calls
`cacheflush()` and `persist()` after every write operation to ensure data durability.

**Note**: Turso's local engine creates its own WAL format when it first creates the database
file. If you have an existing SQLite file created by another library, use a separate file
path for Turso to ensure proper WAL support.

## Architecture

```
┌─────────────────────────────┐
│     Your SeaORM Code        │
├─────────────────────────────┤
│   TursoConnection           │  ← Implements ConnectionTrait + TransactionTrait
├─────────────────────────────┤
│   Turso Engine (MVCC)       │  ← In-process, async, concurrent writes
├─────────────────────────────┤
│   WAL → File Persistence    │  ← Survives restarts
└─────────────────────────────┘
```

## API

### Connection

```rust
use sea_orm_turso::TursoConnection;

// Connect to a file
let db = TursoConnection::connect("app.db").await?;

// Connect to memory (no persistence)
let db = TursoConnection::connect_memory().await?;

// From existing Turso database
let db = TursoConnection::from_existing(database, connection);
```

### Convenience Functions

```rust
use sea_orm_turso;

// Shorthand for TursoConnection::connect
let db = sea_orm_turso::connect("app.db").await?;

// Shorthand for TursoConnection::connect_memory
let db = sea_orm_turso::connect_memory().await?;
```

## Related

- [SeaORM](https://github.com/SeaQL/sea-orm) - The ORM this adapter supports
- [Turso](https://turso.tech) - The database engine powering this adapter
- [SeaQL/sea-orm#2763](https://github.com/SeaQL/sea-orm/issues/2763) - Original feature request

## License

MIT OR Apache-2.0

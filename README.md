# Rustyway

A minimalistic database migration and versioning crate for applications using [DuckDB](https://duckdb.org/), written in Rust.

Rustyway reads a list of *change sets* (a JSON file) and the SQL scripts they refer to, applies the ones that have not been run yet, and records the result of every run in a history table inside your database.

## Adding Rustyway to your project

Rustyway is not published on crates.io yet, so add it as a git or path dependency. Because Rustyway uses the `duckdb` crate's `Connection`, your project must depend on the **same `duckdb` version** (see this crate's [Cargo.toml](Cargo.toml)).

```toml
[dependencies]
rustyway = { git = "https://github.com/slaytanic87/rustyway" }
# or, for a local checkout:
# rustyway = { path = "../rustyway" }

duckdb = { version = "1.20000.1", features = ["bundled"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter"] }  # optional, to see migration logs
```

## Usage

Open a DuckDB connection, wrap it in `RustywayDuckDB` and call `initialize()`:

```rust
use duckdb::Connection;
use rustyway::{MigrationRepository, RustywayDuckDB};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional: show what Rustyway is doing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .init();

    let connection = Connection::open("my_app.duckdb")?;

    // Creates the history table if needed and applies all pending migrations
    let migrations = RustywayDuckDB::new(connection);
    migrations.initialize()?;

    Ok(())
}
```

Call this once at application start-up, before the rest of your code touches the database. Running it repeatedly is safe: already applied change sets are skipped.

### Methods of `MigrationRepository`

| Method           | Description                                                                              |
|------------------|------------------------------------------------------------------------------------------|
| `initialize()`   | Creates the history table (if missing) and runs the migration scripts. This is what you normally call. |

## Excamples: Change sets and scripts

Migrations live in the `migrations` folder:

```text
migrations/
├── changesets.json          # list of change sets
└── V1_initial_setup.sql     # one SQL script per change set
```

### `migrations/changesets.json`

```json
[
    {
        "id": 1,
        "name": "Initial tables",
        "description": "Initial setup of tables",
        "script": "V1_initial_setup.sql",
        "author": "test_user"
    }
]
```

| Field         | Type    | Description                                                        |
|---------------|---------|--------------------------------------------------------------------|
| `id`          | integer | Unique, positive number. Change sets are applied in ascending order. |
| `name`        | string  | Short name of the change set.                                      |
| `description` | string  | What the change set does. **Required.**                            |
| `script`      | string  | File name of the SQL script inside `migrations/`.                  |
| `author`      | string  | Who wrote the change set.                                          |

### SQL scripts

Each script is plain DuckDB SQL and may contain multiple statements (it is run as a batch):

```sql
-- migrations/V1_initial_setup.sql
CREATE TABLE IF NOT EXISTS investment (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);
```

### Adding a new migration

1. Create a new script, e.g. `migrations/V2_add_notes.sql`.
2. Append an entry to `changesets.json` with the next `id`:

   ```json
   {
       "id": 2,
       "name": "Add notes",
       "description": "Adds a notes column",
       "script": "V2_add_notes.sql",
       "author": "test_user"
   }
   ```
3. Rebuild and start your application; the new change set is applied on `initialize()`.

Do not edit or renumber change sets that have already been applied. A change set is considered applied if its `id` **or** its `script` name is already in the history table.

## Migration history

Rustyway stores every executed change set in the table `migration_history_table`:

| Column            | Description                              |
|-------------------|------------------------------------------|
| `id`              | Change set id                            |
| `change_set_name` | Change set name                          |
| `description`     | Change set description                   |
| `script`          | Script file name                         |
| `migration_type`  | Currently always `SQL`                   |
| `author`          | Change set author                        |
| `applied_at`      | Timestamp of execution                   |
| `state`           | `APPLIED` or `FAILED`                    |

## Failure behaviour

- Change sets run one after another. If a script fails, it is recorded with state `FAILED`, the error is logged, and no further change sets are run in that pass.
- On the next `initialize()` / `migrate()`, the failed script is re-executed first. If it succeeds it is marked `APPLIED`, and the remaining change sets continue.
- `initialize()` returns an `Err` if a script or `changesets.json` is missing or invalid.
- Scripts are not wrapped in a transaction by Rustyway. Write idempotent SQL (`CREATE TABLE IF NOT EXISTS`, ...) so a retry after a partial failure is safe.

## Logging

Rustyway logs through [`tracing`](https://crates.io/crates/tracing): `info` for executed scripts, `debug` for skipped scripts and script contents, `error` for failures. Install any `tracing` subscriber in your application to see them.

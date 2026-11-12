
use duckdb::{Connection, params};
use tracing::{debug, error, info};

use crate::{MigrationRepository, model::{MigrationSetType, MigrationState, ChangeSet}, persistence::{get_migration_script, read_change_sets}};

static MIGRATION_HISTORY_TABLE: &str = "migration_history_table";


pub struct RustywayDuckDB {
    connection: Connection,
}

impl RustywayDuckDB {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }
}

impl MigrationRepository for RustywayDuckDB {
    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connection.execute_batch(
            format!(
                r#"
           CREATE TABLE IF NOT EXISTS {} (
               id INTEGER PRIMARY KEY NOT NULL,
               change_set_name TEXT NOT NULL,
               description TEXT NOT NULL,
               script TEXT NOT NULL,
               migration_type TEXT NOT NULL,
               author TEXT NOT NULL,
               applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
               state TEXT NOT NULL
           );
        "#,
                MIGRATION_HISTORY_TABLE
            )
            .as_str(),
        )?;

        self.migrate()?;
        Ok(())
    }

    fn execute_failed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let last_failed_changeset: Option<(u64, String)> = self
            .connection
            .prepare(
                format!(
                    r#"
                SELECT id, script FROM {} WHERE state = ? ORDER BY applied_at DESC LIMIT 1
                "#,
                    MIGRATION_HISTORY_TABLE
                )
                .as_str(),
            )?
            .query_map(params![MigrationState::Failed.to_string()], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .next()
            .transpose()?;

        if let Some((id, script_name)) = last_failed_changeset.clone() {

            let script: String = get_migration_script(&script_name)
                .ok_or_else(|| format!("Migration script not found: {}", script_name))?;

            info!("Re-executing failed migration script: {}", script_name);

            self.connection.execute_batch(&script)?;
            self.connection.execute(
                format!(
                    r#"
                UPDATE {} SET state = ? WHERE id = ?"#,
                    MIGRATION_HISTORY_TABLE
                )
                .as_str(),
                params![MigrationState::Applied.to_string(), id],
            )?;
        }
        Ok(())
    }

    fn execute_new(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut change_sets: Vec<ChangeSet> = read_change_sets()?;

        change_sets.sort_by(|a, b| a.id.cmp(&b.id));

        let applied_scripts: Vec<(u64, String)> = self
            .connection
            .prepare(
                format!(
                    r#"
                SELECT id, script FROM {} WHERE state = ? ORDER BY id, applied_at ASC
                "#,
                    MIGRATION_HISTORY_TABLE
                )
                .as_str(),
            )?
            .query_map(params![MigrationState::Applied.to_string()], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<(u64, String)>, _>>()?;

        for change_set in change_sets {
            if applied_scripts
                .iter()
                .any(|(id, script)| id == &change_set.id || script == &change_set.script)
            {
                debug!("Skipping already applied migration script {} with id {}", change_set.script, change_set.id);
                continue;
            }

            let script: String = get_migration_script(&change_set.script)
                .ok_or_else(|| format!("Migration script not found: {}", change_set.script))?;

            info!("Executing migration script: {}", change_set.script);
            debug!("{}", script);

            let execution_result = self.connection.execute_batch(&script);

            let state: MigrationState = if let Err(e) = &execution_result {
                error!("Failed to execute migration script {}: {}", change_set.script, e);
                MigrationState::Failed
            } else {
                MigrationState::Applied
            };

            self.connection.execute(
                format!(
                    r#"
                INSERT INTO {} (id, change_set_name, description, script, migration_type, author, state)
                VALUES (?, ?, ?, ?, ?, ?, ?)"#,
                    MIGRATION_HISTORY_TABLE
                )
                .as_str(),
                params![
                    change_set.id,
                    change_set.name,
                    change_set.description,
                    change_set.script,
                    MigrationSetType::SQL.to_string(),
                    change_set.author,
                    state.to_string()
                ],
            )?;

            if state == MigrationState::Failed {
                break;
            }
        }

        Ok(())
    }

    fn migrate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.execute_failed()?;

        self.execute_new()?;

        Ok(())
    }
}

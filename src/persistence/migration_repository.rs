use crate::model::ChangeSet;

static CHANGESETS_FILE: &str = "changesets.json";

/// Supplies the change set manifest and migration scripts to Rustyway.
///
/// Rustyway has no migrations of its own: implement this in the consuming
/// application, typically by embedding its own `migrations/` folder with
/// `rust_embed` and returning file contents by name.
pub trait MigrationSource {
    fn get(&self, file_path: &str) -> Option<String>;
}

pub trait MigrationRepository {
    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn execute_failed(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn execute_new(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn migrate(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn read_change_sets(
    source: &dyn MigrationSource,
) -> Result<Vec<ChangeSet>, Box<dyn std::error::Error>> {
    let file_content = source
        .get(CHANGESETS_FILE)
        .ok_or_else(|| format!("Migration script not found: {}", CHANGESETS_FILE))?;
    let change_sets: Vec<ChangeSet> = serde_json::from_str(&file_content)?;
    Ok(change_sets)
}

use crate::model::ChangeSet;

static CHANGESETS_FILE: &str = "changesets.json";

pub trait MigrationRepository {
    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn execute_failed(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn execute_new(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn migrate(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn read_change_sets() -> Result<Vec<ChangeSet>, Box<dyn std::error::Error>> {
    let file_content = get_migration_script(CHANGESETS_FILE)
        .ok_or_else(|| format!("Migration script not found: {}", CHANGESETS_FILE))?;
    let change_sets: Vec<ChangeSet> = serde_json::from_str(&file_content)?;
    Ok(change_sets)
}

#[derive(rust_embed::RustEmbed)]
#[folder = "migrations"]
struct MigrationScriptFolder;

pub fn get_migration_script(file_path: &str) -> Option<String> {
    MigrationScriptFolder::get(file_path)
    .map(|content| String::from_utf8_lossy(&content.data.as_ref()).into_owned())
}
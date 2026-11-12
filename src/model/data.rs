use core::fmt;
use serde::Deserialize;

#[derive(PartialEq)]
pub enum MigrationState {
    Applied,
    Failed,
}

impl fmt::Display for MigrationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationState::Applied => write!(f, "APPLIED"),
            MigrationState::Failed => write!(f, "FAILED"),
        }
    }
}

pub enum MigrationSetType {
    SQL,
}

impl fmt::Display for MigrationSetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationSetType::SQL => write!(f, "SQL"),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct ChangeSet {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub script: String,
    pub author: String,
}
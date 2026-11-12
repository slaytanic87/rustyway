use duckdb::Connection;
use rustyway::{MigrationRepository, RustywayDuckDB};

fn init_logging(log_level: String, log_format: String) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    if log_format.eq_ignore_ascii_case("json") {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .compact()
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    init_logging("DEBUG".to_string(), "json".to_string());
    let connection = Connection::open("test.duckdb").unwrap();
    let duckdb = RustywayDuckDB::new(connection);
    duckdb.initialize()?;
    Ok(())
}

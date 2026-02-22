use crate::error::DsmError;
use std::path::Path;

/// Build arguments for `mysql -e "CREATE DATABASE IF NOT EXISTS ..."`.
/// Uses a defaults-file for credential passing.
pub fn mysql_create_db_args(defaults_file: &str, database: &str) -> Vec<String> {
    vec![
        format!("--defaults-extra-file={}", defaults_file),
        "-e".to_string(),
        format!("CREATE DATABASE IF NOT EXISTS `{}`", database),
    ]
}

/// Build arguments to check if a PostgreSQL database exists via `psql`.
/// Connects to the `postgres` default database to run the existence check.
pub fn pg_check_db_args(host: &str, port: u16, username: &str, database: &str) -> Vec<String> {
    vec![
        "--host".to_string(),
        host.to_string(),
        "--port".to_string(),
        port.to_string(),
        "--username".to_string(),
        username.to_string(),
        "postgres".to_string(),
        "-tAc".to_string(),
        format!("SELECT 1 FROM pg_database WHERE datname = '{}'", database),
    ]
}

/// Build arguments to create a PostgreSQL database via `psql`.
/// Connects to the `postgres` default database to run CREATE DATABASE.
pub fn pg_create_db_args(host: &str, port: u16, username: &str, database: &str) -> Vec<String> {
    vec![
        "--host".to_string(),
        host.to_string(),
        "--port".to_string(),
        port.to_string(),
        "--username".to_string(),
        username.to_string(),
        "postgres".to_string(),
        "-c".to_string(),
        format!("CREATE DATABASE \"{}\"", database),
    ]
}

/// Ensure the parent directory exists for a SQLite database file path.
pub fn ensure_sqlite_parent_dir(database_path: &str) -> Result<(), DsmError> {
    let path = Path::new(database_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Validate that the snapshot's database type matches the target profile's database type.
/// Cross-type restores (e.g. MySQL snapshot -> PostgreSQL profile) are not supported.
pub fn validate_db_type_match(snapshot_type: &str, target_type: &str) -> Result<(), DsmError> {
    if snapshot_type != target_type {
        return Err(DsmError::DbTypeMismatch {
            snapshot_type: snapshot_type.to_string(),
            profile_type: target_type.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mysql_create_db_args_builds_correct_command() {
        let args = mysql_create_db_args("/tmp/defaults.cnf", "my_database");
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "--defaults-extra-file=/tmp/defaults.cnf");
        assert_eq!(args[1], "-e");
        assert_eq!(args[2], "CREATE DATABASE IF NOT EXISTS `my_database`");
    }

    #[test]
    fn pg_check_db_args_connects_to_postgres_database() {
        let args = pg_check_db_args("localhost", 5432, "postgres", "my_db");
        assert!(args.contains(&"postgres".to_string()));
        assert!(args.contains(&"-tAc".to_string()));
        assert!(args.iter().any(|a| a.contains("pg_database")));
    }

    #[test]
    fn pg_create_db_args_builds_correct_command() {
        let args = pg_create_db_args("localhost", 5432, "postgres", "my_db");
        assert!(args.contains(&"postgres".to_string()));
        assert!(args.contains(&"-c".to_string()));
        assert!(args.iter().any(|a| a.contains("CREATE DATABASE")));
    }

    #[test]
    fn ensure_sqlite_parent_dir_succeeds_for_existing_parent() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("subdir").join("test.db");
        ensure_sqlite_parent_dir(db_path.to_str().unwrap()).unwrap();
        assert!(tmp.path().join("subdir").exists());
    }

    #[test]
    fn validate_db_type_match_accepts_same_types() {
        assert!(validate_db_type_match("mysql", "mysql").is_ok());
        assert!(validate_db_type_match("postgresql", "postgresql").is_ok());
        assert!(validate_db_type_match("sqlite", "sqlite").is_ok());
    }

    #[test]
    fn validate_db_type_match_rejects_different_types() {
        let result = validate_db_type_match("mysql", "postgresql");
        assert!(result.is_err());
    }
}

use crate::credentials::{self, ProfileCredentials};
use crate::db::{AppPaths, DbState};
use crate::error::DsmError;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;

const LOCAL_HOST: &str = "127.0.0.1";
const MYSQL_CREDENTIAL_ID: &str = "restore-test-mysql";
const POSTGRESQL_CREDENTIAL_ID: &str = "restore-test-postgresql";
const ADMIN_TIMEOUT_SECS: u64 = 60;
const RESTORE_TIMEOUT_SECS: u64 = 60 * 60;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreTestEngineSettings {
    pub enabled: bool,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreTestSettings {
    pub mysql: RestoreTestEngineSettings,
    pub postgresql: RestoreTestEngineSettings,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreTestSettingsInput {
    pub mysql_enabled: bool,
    pub mysql_port: u16,
    pub mysql_username: String,
    pub mysql_password: Option<String>,
    pub postgresql_enabled: bool,
    pub postgresql_port: u16,
    pub postgresql_username: String,
    pub postgresql_password: Option<String>,
}

#[derive(Debug, Clone)]
struct LocalConnection {
    port: u16,
    username: String,
    password: String,
}

#[derive(Debug, Clone)]
pub struct RestoreTestResult {
    pub status: String,
    pub message: Option<String>,
    pub tested_at: Option<String>,
}

impl RestoreTestResult {
    fn passed(message: impl Into<String>) -> Self {
        Self {
            status: "passed".to_string(),
            message: Some(message.into()),
            tested_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    fn failed(message: impl Into<String>) -> Self {
        Self {
            status: "failed".to_string(),
            message: Some(message.into()),
            tested_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    fn not_configured(database_type: &str) -> Self {
        Self {
            status: "not_configured".to_string(),
            message: Some(format!(
                "Local {database_type} restore-test credentials are not configured."
            )),
            tested_at: None,
        }
    }
}

pub fn get_settings(db: &DbState) -> Result<RestoreTestSettings, DsmError> {
    let conn = db.0.lock().map_err(|error| DsmError::ConnectionError {
        message: format!("Failed to acquire database lock: {error}"),
    })?;

    Ok(RestoreTestSettings {
        mysql: RestoreTestEngineSettings {
            enabled: setting_bool(&conn, "restore_test_mysql_enabled")?,
            port: setting_u16(&conn, "restore_test_mysql_port", 3306)?,
            username: setting_string(&conn, "restore_test_mysql_username", "root")?,
        },
        postgresql: RestoreTestEngineSettings {
            enabled: setting_bool(&conn, "restore_test_postgresql_enabled")?,
            port: setting_u16(&conn, "restore_test_postgresql_port", 5432)?,
            username: setting_string(&conn, "restore_test_postgresql_username", "postgres")?,
        },
    })
}

pub fn set_settings(db: &DbState, input: &RestoreTestSettingsInput) -> Result<(), DsmError> {
    validate_engine(
        "MySQL",
        input.mysql_enabled,
        input.mysql_port,
        &input.mysql_username,
    )?;
    validate_engine(
        "PostgreSQL",
        input.postgresql_enabled,
        input.postgresql_port,
        &input.postgresql_username,
    )?;
    validate_mysql_option_value("username", &input.mysql_username)?;
    if let Some(password) = input.mysql_password.as_deref() {
        validate_mysql_option_value("password", password)?;
    }

    if let Some(password) = &input.mysql_password {
        credentials::store_credentials(
            MYSQL_CREDENTIAL_ID,
            &ProfileCredentials {
                password: Some(password.clone()),
                ..ProfileCredentials::default()
            },
        )?;
    }
    if let Some(password) = &input.postgresql_password {
        credentials::store_credentials(
            POSTGRESQL_CREDENTIAL_ID,
            &ProfileCredentials {
                password: Some(password.clone()),
                ..ProfileCredentials::default()
            },
        )?;
    }

    {
        let mut conn = db.0.lock().map_err(|error| DsmError::ConnectionError {
            message: format!("Failed to acquire database lock: {error}"),
        })?;
        let transaction = conn.transaction()?;
        let values = [
            (
                "restore_test_mysql_enabled",
                input.mysql_enabled.to_string(),
            ),
            ("restore_test_mysql_port", input.mysql_port.to_string()),
            (
                "restore_test_mysql_username",
                input.mysql_username.trim().to_string(),
            ),
            (
                "restore_test_postgresql_enabled",
                input.postgresql_enabled.to_string(),
            ),
            (
                "restore_test_postgresql_port",
                input.postgresql_port.to_string(),
            ),
            (
                "restore_test_postgresql_username",
                input.postgresql_username.trim().to_string(),
            ),
        ];
        for (key, value) in values {
            transaction.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                rusqlite::params![key, value],
            )?;
        }
        transaction.commit()?;
    }

    Ok(())
}

pub async fn verify_snapshot(
    db: &DbState,
    paths: &AppPaths,
    database_type: &str,
    snapshot_path: &Path,
) -> RestoreTestResult {
    match database_type {
        "mysql" => match local_connection(db, "mysql") {
            Ok(Some(connection)) => verify_mysql(paths, snapshot_path, &connection).await,
            Ok(None) => RestoreTestResult::not_configured("MySQL"),
            Err(error) => RestoreTestResult::failed(format!(
                "Could not load local MySQL restore-test settings: {error}"
            )),
        },
        "postgresql" => match local_connection(db, "postgresql") {
            Ok(Some(connection)) => verify_postgresql(paths, snapshot_path, &connection).await,
            Ok(None) => RestoreTestResult::not_configured("PostgreSQL"),
            Err(error) => RestoreTestResult::failed(format!(
                "Could not load local PostgreSQL restore-test settings: {error}"
            )),
        },
        "sqlite" => verify_sqlite(paths, snapshot_path).await,
        other => RestoreTestResult::failed(format!("Unsupported database type: {other}")),
    }
}

fn validate_engine(label: &str, enabled: bool, port: u16, username: &str) -> Result<(), DsmError> {
    if enabled && username.trim().is_empty() {
        return Err(DsmError::ValidationError {
            message: format!("A local {label} username is required."),
        });
    }
    if enabled && port == 0 {
        return Err(DsmError::ValidationError {
            message: format!("A valid local {label} port is required."),
        });
    }
    Ok(())
}

fn validate_mysql_option_value(label: &str, value: &str) -> Result<(), DsmError> {
    if value.contains(['\r', '\n']) {
        return Err(DsmError::ValidationError {
            message: format!("The local MySQL {label} cannot contain a line break."),
        });
    }
    Ok(())
}

fn setting_value(conn: &rusqlite::Connection, key: &str) -> Result<Option<String>, DsmError> {
    Ok(conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |row| row.get(0),
        )
        .optional()?)
}

fn setting_bool(conn: &rusqlite::Connection, key: &str) -> Result<bool, DsmError> {
    Ok(setting_value(conn, key)?.is_some_and(|value| value == "true"))
}

fn setting_u16(conn: &rusqlite::Connection, key: &str, default: u16) -> Result<u16, DsmError> {
    Ok(setting_value(conn, key)?
        .and_then(|value| value.parse().ok())
        .unwrap_or(default))
}

fn setting_string(
    conn: &rusqlite::Connection,
    key: &str,
    default: &str,
) -> Result<String, DsmError> {
    Ok(setting_value(conn, key)?.unwrap_or_else(|| default.to_string()))
}

fn local_connection(
    db: &DbState,
    database_type: &str,
) -> Result<Option<LocalConnection>, DsmError> {
    let settings = get_settings(db)?;
    let (engine, credential_id) = match database_type {
        "mysql" => (settings.mysql, MYSQL_CREDENTIAL_ID),
        "postgresql" => (settings.postgresql, POSTGRESQL_CREDENTIAL_ID),
        _ => return Ok(None),
    };
    if !engine.enabled {
        return Ok(None);
    }

    let creds = credentials::get_credentials(credential_id)?;
    if database_type == "mysql" {
        validate_mysql_option_value("username", &engine.username)?;
        validate_mysql_option_value("password", creds.password.as_deref().unwrap_or(""))?;
    }
    Ok(Some(LocalConnection {
        port: engine.port,
        username: engine.username,
        password: creds.password.unwrap_or_default(),
    }))
}

fn temporary_database_name() -> String {
    format!("amber_verify_{}", uuid::Uuid::new_v4().simple())
}

async fn verify_mysql(
    paths: &AppPaths,
    snapshot_path: &Path,
    connection: &LocalConnection,
) -> RestoreTestResult {
    let mysql = match crate::dump::find_tool("mysql") {
        Ok(tool) => tool,
        Err(error) => return RestoreTestResult::failed(error.to_string()),
    };
    let database_name = temporary_database_name();
    let defaults_file = match crate::dump::write_mysql_defaults_file(
        &paths.tmp_dir,
        LOCAL_HOST,
        connection.port,
        &connection.username,
        &connection.password,
    ) {
        Ok(path) => path,
        Err(error) => {
            return RestoreTestResult::failed(format!(
                "Could not prepare local MySQL credentials: {error}"
            ));
        }
    };

    let create = run_mysql_admin(
        &mysql,
        &defaults_file,
        connection.port,
        &format!("CREATE DATABASE `{database_name}`"),
    )
    .await;
    let verification = match create {
        Ok(()) => {
            restore_with_mysql(
                &mysql,
                &defaults_file,
                connection.port,
                &database_name,
                snapshot_path,
                paths,
            )
            .await
        }
        Err(message) => Err(format!("Could not create local test database: {message}")),
    };

    let cleanup = run_mysql_admin(
        &mysql,
        &defaults_file,
        connection.port,
        &format!("DROP DATABASE IF EXISTS `{database_name}`"),
    )
    .await
    .map_err(|message| format!("{message}. Temporary database: {database_name}"));
    let defaults_cleanup = remove_temporary_file(&defaults_file, "MySQL credentials file");
    let cleanup = combine_results(cleanup, defaults_cleanup);

    finish_verification("MySQL", verification, cleanup)
}

async fn run_mysql_admin(
    mysql: &Path,
    defaults_file: &Path,
    port: u16,
    statement: &str,
) -> Result<(), String> {
    let mut command = tokio::process::Command::new(mysql);
    command
        .args(mysql_connection_args(defaults_file, port))
        .arg("--execute")
        .arg(statement);
    let output = run_command(command, ADMIN_TIMEOUT_SECS, "mysql admin command").await?;
    output_result(&output)
}

async fn restore_with_mysql(
    mysql: &Path,
    defaults_file: &Path,
    port: u16,
    database_name: &str,
    snapshot_path: &Path,
    paths: &AppPaths,
) -> Result<(), String> {
    let sql_path = decompress_for_restore(paths, snapshot_path).await?;
    let output = match std::fs::File::open(&sql_path) {
        Ok(input) => {
            let mut command = tokio::process::Command::new(mysql);
            command
                .args(mysql_connection_args(defaults_file, port))
                .arg(database_name)
                .stdin(Stdio::from(input));
            run_command(command, RESTORE_TIMEOUT_SECS, "mysql restore").await
        }
        Err(error) => Err(format!("Could not open decompressed snapshot: {error}")),
    };
    let restore = output.and_then(|result| output_result(&result));
    let cleanup = remove_temporary_file(&sql_path, "decompressed snapshot");
    combine_results(restore, cleanup)
}

fn mysql_connection_args(defaults_file: &Path, port: u16) -> Vec<String> {
    vec![
        format!("--defaults-extra-file={}", defaults_file.display()),
        "--protocol=TCP".to_string(),
        format!("--host={LOCAL_HOST}"),
        format!("--port={port}"),
    ]
}

async fn verify_postgresql(
    paths: &AppPaths,
    snapshot_path: &Path,
    connection: &LocalConnection,
) -> RestoreTestResult {
    let psql = match crate::dump::find_tool("psql") {
        Ok(tool) => tool,
        Err(error) => return RestoreTestResult::failed(error.to_string()),
    };
    let database_name = temporary_database_name();
    let create = run_psql_admin(
        &psql,
        connection,
        &format!("CREATE DATABASE \"{database_name}\""),
    )
    .await;
    let verification = match create {
        Ok(()) => restore_with_psql(&psql, connection, &database_name, snapshot_path, paths).await,
        Err(message) => Err(format!("Could not create local test database: {message}")),
    };

    let cleanup = run_psql_admin(
        &psql,
        connection,
        &format!("DROP DATABASE IF EXISTS \"{database_name}\""),
    )
    .await
    .map_err(|message| format!("{message}. Temporary database: {database_name}"));

    finish_verification("PostgreSQL", verification, cleanup)
}

async fn run_psql_admin(
    psql: &Path,
    connection: &LocalConnection,
    statement: &str,
) -> Result<(), String> {
    let mut command = tokio::process::Command::new(psql);
    command
        .env("PGPASSWORD", &connection.password)
        .args(pg_connection_args(connection, "postgres"))
        .arg("--set=ON_ERROR_STOP=1")
        .arg("--command")
        .arg(statement);
    let output = run_command(command, ADMIN_TIMEOUT_SECS, "psql admin command").await?;
    output_result(&output)
}

async fn restore_with_psql(
    psql: &Path,
    connection: &LocalConnection,
    database_name: &str,
    snapshot_path: &Path,
    paths: &AppPaths,
) -> Result<(), String> {
    let sql_path = decompress_for_restore(paths, snapshot_path).await?;
    let output = match std::fs::File::open(&sql_path) {
        Ok(input) => {
            let mut command = tokio::process::Command::new(psql);
            command
                .env("PGPASSWORD", &connection.password)
                .args(pg_connection_args(connection, database_name))
                .arg("--set=ON_ERROR_STOP=1")
                .stdin(Stdio::from(input));
            run_command(command, RESTORE_TIMEOUT_SECS, "psql restore").await
        }
        Err(error) => Err(format!("Could not open decompressed snapshot: {error}")),
    };
    let restore = output.and_then(|result| output_result(&result));
    let cleanup = remove_temporary_file(&sql_path, "decompressed snapshot");
    combine_results(restore, cleanup)
}

fn pg_connection_args(connection: &LocalConnection, database_name: &str) -> Vec<String> {
    vec![
        "--host".to_string(),
        LOCAL_HOST.to_string(),
        "--port".to_string(),
        connection.port.to_string(),
        "--username".to_string(),
        connection.username.clone(),
        "--dbname".to_string(),
        database_name.to_string(),
    ]
}

async fn verify_sqlite(paths: &AppPaths, snapshot_path: &Path) -> RestoreTestResult {
    let database_path = paths
        .tmp_dir
        .join(format!("{}.db", temporary_database_name()));
    let gz = snapshot_path.to_path_buf();
    let restored = database_path.clone();
    let verification = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut output = create_private_file(&restored)
            .map_err(|error| format!("Could not create local test database: {error}"))?;
        crate::compress::decompress_to_writer(&gz, &mut output)
            .map_err(|error| format!("Could not import snapshot: {error}"))?;
        drop(output);

        let conn = rusqlite::Connection::open_with_flags(
            &restored,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|error| format!("Could not open restored database: {error}"))?;
        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|error| format!("Integrity check failed: {error}"))?;
        if integrity == "ok" {
            Ok(())
        } else {
            Err(format!("SQLite integrity check returned: {integrity}"))
        }
    })
    .await
    .map_err(|error| format!("Local restore-test task failed: {error}"))
    .and_then(|result| result);

    let cleanup = std::fs::remove_file(&database_path)
        .or_else(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(error)
            }
        })
        .map_err(|error| {
            format!(
                "Could not delete local test database {}: {error}",
                database_path.display()
            )
        });

    finish_verification("SQLite", verification, cleanup)
}

async fn decompress_for_restore(paths: &AppPaths, snapshot_path: &Path) -> Result<PathBuf, String> {
    let sql_path = paths
        .tmp_dir
        .join(format!("restore-test-{}.sql", uuid::Uuid::new_v4()));
    let gz = snapshot_path.to_path_buf();
    let output = sql_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut file = create_private_file(&output)?;
        crate::compress::decompress_to_writer(&gz, &mut file)
    })
    .await
    .map_err(|error| format!("Decompression task failed: {error}"))?
    .map_err(|error| format!("Could not decompress snapshot: {error}"));
    if let Err(error) = result {
        let cleanup = remove_temporary_file(&sql_path, "partial decompressed snapshot");
        return Err(match cleanup {
            Ok(()) => error,
            Err(cleanup_error) => format!("{error}. {cleanup_error}"),
        });
    }
    Ok(sql_path)
}

fn create_private_file(path: &Path) -> Result<std::fs::File, std::io::Error> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn remove_temporary_file(path: &Path, label: &str) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Could not delete temporary {label} {}: {error}",
            path.display()
        )),
    }
}

fn combine_results(primary: Result<(), String>, cleanup: Result<(), String>) -> Result<(), String> {
    match (primary, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(format!("{error}. {cleanup_error}")),
    }
}

fn output_result(output: &std::process::Output) -> Result<(), String> {
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("process exited with status {}", output.status))
        } else {
            Err(stderr)
        }
    }
}

async fn run_command(
    mut command: tokio::process::Command,
    timeout_secs: u64,
    label: &str,
) -> Result<std::process::Output, String> {
    command.kill_on_drop(true);
    match tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        command.output(),
    )
    .await
    {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(error)) => Err(format!("Failed to run {label}: {error}")),
        Err(_) => Err(format!("{label} timed out after {timeout_secs} seconds")),
    }
}

fn finish_verification(
    database_type: &str,
    verification: Result<(), String>,
    cleanup: Result<(), String>,
) -> RestoreTestResult {
    match (verification, cleanup) {
        (Ok(()), Ok(())) => RestoreTestResult::passed(format!(
            "{database_type} snapshot restored successfully and the local test database was deleted."
        )),
        (Err(verification), Ok(())) => RestoreTestResult::failed(format!(
            "Local {database_type} restore test failed: {verification}. The test database was deleted."
        )),
        (Ok(()), Err(cleanup)) => RestoreTestResult::failed(format!(
            "Local {database_type} restore succeeded, but cleanup failed: {cleanup}"
        )),
        (Err(verification), Err(cleanup)) => RestoreTestResult::failed(format!(
            "Local {database_type} restore test failed: {verification}. Cleanup also failed: {cleanup}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_database_names_are_safe_and_namespaced() {
        let name = temporary_database_name();
        assert!(name.starts_with("amber_verify_"));
        assert!(name.chars().all(|character| character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '_'));
    }

    #[test]
    fn postgresql_arguments_are_for_loopback_only() {
        let connection = LocalConnection {
            port: 5432,
            username: "amber".to_string(),
            password: String::new(),
        };
        let args = pg_connection_args(&connection, "postgres");
        let host_index = args.iter().position(|arg| arg == "--host").unwrap();
        assert_eq!(args[host_index + 1], "127.0.0.1");
        assert!(!args.iter().any(|arg| arg.contains("ssh")));
    }

    #[test]
    fn mysql_defaults_file_is_for_loopback_only() {
        let tmp = tempfile::tempdir().unwrap();
        let path = crate::dump::write_mysql_defaults_file(tmp.path(), LOCAL_HOST, 3306, "root", "")
            .unwrap();
        let contents = std::fs::read_to_string(path).unwrap();
        assert!(contents.contains("host=127.0.0.1"));
        assert!(!contents.contains("localhost"));
    }

    #[test]
    fn mysql_command_arguments_force_tcp_loopback_after_the_options_file() {
        let args = mysql_connection_args(Path::new("/tmp/restore.cnf"), 3307);
        assert_eq!(args[0], "--defaults-extra-file=/tmp/restore.cnf");
        assert_eq!(args[1], "--protocol=TCP");
        assert_eq!(args[2], "--host=127.0.0.1");
        assert_eq!(args[3], "--port=3307");
        assert!(!args.iter().any(|arg| arg.contains("ssh")));
    }

    #[test]
    fn mysql_option_values_reject_line_break_injection() {
        let error =
            validate_mysql_option_value("password", "secret\nhost=remote.example.com").unwrap_err();
        assert!(error.to_string().contains("cannot contain a line break"));
    }

    #[test]
    fn local_engine_settings_round_trip_without_storing_hosts() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations_for_test()
            .to_latest(&mut conn)
            .unwrap();
        let db = DbState(std::sync::Mutex::new(conn));
        let input = RestoreTestSettingsInput {
            mysql_enabled: true,
            mysql_port: 3307,
            mysql_username: "amber_mysql".to_string(),
            mysql_password: None,
            postgresql_enabled: true,
            postgresql_port: 5433,
            postgresql_username: "amber_pg".to_string(),
            postgresql_password: None,
        };

        set_settings(&db, &input).unwrap();
        let settings = get_settings(&db).unwrap();

        assert!(settings.mysql.enabled);
        assert_eq!(settings.mysql.port, 3307);
        assert_eq!(settings.mysql.username, "amber_mysql");
        assert!(settings.postgresql.enabled);
        assert_eq!(settings.postgresql.port, 5433);
        assert_eq!(settings.postgresql.username, "amber_pg");
        let conn = db.0.lock().unwrap();
        let stored_host_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM settings WHERE key LIKE 'restore_test_%host%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored_host_count, 0);
    }

    #[cfg(unix)]
    #[test]
    fn restored_temporary_files_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("private.sql");
        drop(create_private_file(&path).unwrap());
        let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn cleanup_failure_makes_a_successful_restore_fail() {
        let result = finish_verification("MySQL", Ok(()), Err("permission denied".to_string()));
        assert_eq!(result.status, "failed");
        assert!(result.message.unwrap().contains("cleanup failed"));
    }

    #[tokio::test]
    async fn sqlite_snapshot_is_restored_checked_and_deleted() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source.db");
        let snapshot = tmp.path().join("snapshot.db.gz");
        let conn = rusqlite::Connection::open(&source).unwrap();
        conn.execute("CREATE TABLE checks (value TEXT NOT NULL)", [])
            .unwrap();
        conn.execute("INSERT INTO checks (value) VALUES ('restorable')", [])
            .unwrap();
        drop(conn);
        crate::compress::compress_file(&source, &snapshot).unwrap();

        let paths = AppPaths {
            data_dir: tmp.path().to_path_buf(),
            snapshots_dir: tmp.path().to_path_buf(),
            tmp_dir: tmp.path().to_path_buf(),
        };
        let result = verify_sqlite(&paths, &snapshot).await;

        assert_eq!(result.status, "passed");
        let leftovers = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("amber_verify_")
            })
            .count();
        assert_eq!(leftovers, 0);
    }
}

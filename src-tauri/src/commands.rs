use crate::credentials::{self, ProfileCredentials};
use crate::db::{AppPaths, DbState};
use crate::error::DsmError;
use crate::progress::SnapshotProgress;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::State;

// -- Data Types ---------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub project: String,
    pub name: String,
    pub db_type: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database_name: String,
    pub username: Option<String>,
    pub ssh_enabled: bool,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    pub environment: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileInput {
    pub project: String,
    pub name: String,
    pub db_type: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database_name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssh_enabled: bool,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    pub ssh_key_path: Option<String>,
    pub ssh_password: Option<String>,
    pub environment: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileInput {
    pub project: Option<String>,
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssh_enabled: Option<bool>,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    pub ssh_key_path: Option<String>,
    pub ssh_password: Option<String>,
    pub environment: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: String,
    pub profile_id: String,
    pub name: String,
    pub note: Option<String>,
    pub file_path: String,
    pub size_bytes: u64,
    pub db_version: Option<String>,
    pub dump_tool_version: Option<String>,
    pub checksum: Option<String>,
    pub created_at: String,
    pub restored_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub db_version: Option<String>,
    pub latency_ms: u64,
    pub error_kind: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOptions {
    pub target_profile_id: Option<String>,
    pub target_database_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecord {
    pub id: String,
    pub snapshot_id: String,
    pub snapshot_name: String,
    pub target_profile_id: String,
    pub target_db_name: String,
    pub duration_secs: f64,
    pub restored_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub total_bytes: u64,
    pub projects: Vec<ProjectStorage>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStorage {
    pub project: String,
    pub size_bytes: u64,
    pub snapshot_count: u32,
}

// -- Helpers ------------------------------------------------------------------

/// Map a `rusqlite` row to a `Profile` struct.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn row_to_profile(row: &rusqlite::Row) -> Result<Profile, rusqlite::Error> {
    Ok(Profile {
        id: row.get("id")?,
        project: row.get("project")?,
        name: row.get("name")?,
        db_type: row.get("db_type")?,
        host: row.get("host")?,
        port: row.get::<_, Option<i32>>("port")?.map(|v| v as u16),
        database_name: row.get("database_name")?,
        username: row.get("username")?,
        ssh_enabled: row.get::<_, i32>("ssh_enabled")? != 0,
        ssh_host: row.get("ssh_host")?,
        ssh_port: row.get::<_, Option<i32>>("ssh_port")?.map(|v| v as u16),
        ssh_user: row.get("ssh_user")?,
        environment: row.get("environment")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Lock the database mutex, mapping a poison error to `DsmError`.
fn lock_db(db: &DbState) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, DsmError> {
    db.0.lock().map_err(|e| DsmError::ConnectionError {
        message: format!("Failed to acquire database lock: {e}"),
    })
}

/// Map a `rusqlite` row to a `RestoreRecord` struct.
fn row_to_restore_record(row: &rusqlite::Row) -> Result<RestoreRecord, rusqlite::Error> {
    Ok(RestoreRecord {
        id: row.get("id")?,
        snapshot_id: row.get("snapshot_id")?,
        snapshot_name: row.get("snapshot_name")?,
        target_profile_id: row.get("target_profile_id")?,
        target_db_name: row.get("target_db_name")?,
        duration_secs: row.get("duration_secs")?,
        restored_at: row.get("restored_at")?,
    })
}

// -- Profile Commands ---------------------------------------------------------

#[tauri::command]
pub async fn profile_create(
    db: State<'_, DbState>,
    _paths: State<'_, AppPaths>,
    input: CreateProfileInput,
) -> Result<Profile, DsmError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = lock_db(&db)?;
    conn.execute(
        "INSERT INTO profiles (id, project, name, db_type, host, port, database_name, username, ssh_enabled, ssh_host, ssh_port, ssh_user, environment, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            id,
            input.project,
            input.name,
            input.db_type,
            input.host,
            input.port.map(i32::from),
            input.database_name,
            input.username,
            i32::from(input.ssh_enabled),
            input.ssh_host,
            input.ssh_port.map(i32::from),
            input.ssh_user,
            input.environment,
            input.notes,
            now,
            now,
        ],
    )?;

    // Store credentials in Keychain if any are provided
    let has_creds =
        input.password.is_some() || input.ssh_key_path.is_some() || input.ssh_password.is_some();
    if has_creds {
        let creds = ProfileCredentials {
            password: input.password,
            ssh_key_path: input.ssh_key_path,
            ssh_password: input.ssh_password,
        };
        credentials::store_credentials(&id, &creds)?;
    }

    // Read back the inserted profile
    let profile = conn.query_row(
        "SELECT * FROM profiles WHERE id = ?1",
        params![id],
        row_to_profile,
    )?;

    Ok(profile)
}

#[tauri::command]
pub async fn profile_update(
    db: State<'_, DbState>,
    id: String,
    input: UpdateProfileInput,
) -> Result<Profile, DsmError> {
    let conn = lock_db(&db)?;

    // Verify profile exists
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM profiles WHERE id = ?1",
        params![id],
        |row| row.get::<_, i32>(0).map(|c| c > 0),
    )?;
    if !exists {
        return Err(DsmError::ProfileNotFound(id));
    }

    // Build dynamic UPDATE -- only set fields that are Some
    let mut sets: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    macro_rules! push_field {
        ($field:expr, $col:expr) => {
            if let Some(val) = $field {
                sets.push(format!("{} = ?", $col));
                values.push(Box::new(val));
            }
        };
    }

    push_field!(input.project, "project");
    push_field!(input.name, "name");
    push_field!(input.host, "host");
    push_field!(input.database_name, "database_name");
    push_field!(input.username, "username");
    push_field!(input.notes, "notes");
    push_field!(input.ssh_host, "ssh_host");
    push_field!(input.ssh_user, "ssh_user");
    push_field!(input.environment, "environment");

    if let Some(port) = input.port {
        sets.push("port = ?".to_string());
        values.push(Box::new(i32::from(port)));
    }
    if let Some(ssh_port) = input.ssh_port {
        sets.push("ssh_port = ?".to_string());
        values.push(Box::new(i32::from(ssh_port)));
    }
    if let Some(ssh_enabled) = input.ssh_enabled {
        sets.push("ssh_enabled = ?".to_string());
        values.push(Box::new(i32::from(ssh_enabled)));
    }

    // Always update updated_at
    let now = chrono::Utc::now().to_rfc3339();
    sets.push("updated_at = ?".to_string());
    values.push(Box::new(now));

    // Add the WHERE id = ? parameter
    values.push(Box::new(id.clone()));

    let sql = format!("UPDATE profiles SET {} WHERE id = ?", sets.join(", "));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(AsRef::as_ref).collect();
    conn.execute(&sql, param_refs.as_slice())?;

    // Update credentials in Keychain if any credential fields are provided
    let has_cred_changes =
        input.password.is_some() || input.ssh_key_path.is_some() || input.ssh_password.is_some();
    if has_cred_changes {
        // Get existing credentials and merge
        let mut creds = credentials::get_credentials(&id)?;
        if let Some(pw) = input.password {
            creds.password = Some(pw);
        }
        if let Some(key) = input.ssh_key_path {
            creds.ssh_key_path = Some(key);
        }
        if let Some(ssh_pw) = input.ssh_password {
            creds.ssh_password = Some(ssh_pw);
        }
        credentials::store_credentials(&id, &creds)?;
    }

    // Read back the updated profile
    let profile = conn.query_row(
        "SELECT * FROM profiles WHERE id = ?1",
        params![id],
        row_to_profile,
    )?;

    Ok(profile)
}

#[tauri::command]
pub async fn profile_delete(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    id: String,
) -> Result<(), DsmError> {
    let conn = lock_db(&db)?;

    // Delete snapshot files from disk
    let mut stmt = conn.prepare("SELECT file_path FROM snapshots WHERE profile_id = ?1")?;
    let file_paths: Vec<String> = stmt
        .query_map(params![id], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();

    for file_path in &file_paths {
        let full_path = paths.snapshots_dir.join(file_path);
        if full_path.exists() {
            let _ = std::fs::remove_file(&full_path);
        }
    }

    // Delete snapshot metadata
    conn.execute("DELETE FROM snapshots WHERE profile_id = ?1", params![id])?;

    // Delete credentials from Keychain
    credentials::delete_credentials(&id)?;

    // Delete profile from database
    conn.execute("DELETE FROM profiles WHERE id = ?1", params![id])?;

    Ok(())
}

#[tauri::command]
pub async fn profile_list(db: State<'_, DbState>) -> Result<Vec<Profile>, DsmError> {
    let conn = lock_db(&db)?;
    let mut stmt = conn.prepare("SELECT * FROM profiles ORDER BY project, name")?;
    let profiles = stmt
        .query_map([], row_to_profile)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(profiles)
}

#[tauri::command]
pub async fn profile_test_connection(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    id: String,
) -> Result<ConnectionTestResult, DsmError> {
    // Load profile (scoped to drop MutexGuard before any .await)
    let profile = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT * FROM profiles WHERE id = ?1",
            params![id],
            row_to_profile,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DsmError::ProfileNotFound(id.clone()),
            other => DsmError::DatabaseError(other),
        })?
    };

    // Get credentials
    let creds = credentials::get_credentials(&id)?;

    // Open SSH tunnel if needed
    let mut tunnel: Option<crate::ssh::SshTunnel> = None;
    let (effective_host, effective_port) = if profile.ssh_enabled {
        let ssh_host = profile.ssh_host.as_deref().unwrap_or("localhost");
        let ssh_port = profile.ssh_port.unwrap_or(22);
        let ssh_user = profile.ssh_user.as_deref().unwrap_or("root");
        let db_host = profile.host.as_deref().unwrap_or("127.0.0.1");
        let db_port = profile.port.unwrap_or(default_port_for(&profile.db_type));

        let t = crate::ssh::SshTunnel::open(
            ssh_host,
            ssh_port,
            ssh_user,
            creds.ssh_key_path.as_deref(),
            db_host,
            db_port,
        )
        .await?;

        let local_port = t.local_port();
        tunnel = Some(t);
        ("127.0.0.1".to_string(), local_port)
    } else {
        (
            profile
                .host
                .clone()
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            profile.port.unwrap_or(default_port_for(&profile.db_type)),
        )
    };

    let start = std::time::Instant::now();
    let result = match profile.db_type.as_str() {
        "mysql" => {
            test_mysql(
                &paths,
                &effective_host,
                effective_port,
                profile.username.as_deref().unwrap_or("root"),
                creds.password.as_deref().unwrap_or(""),
                &profile.database_name,
            )
            .await
        }
        "postgresql" => {
            test_postgresql(
                &effective_host,
                effective_port,
                profile.username.as_deref().unwrap_or("postgres"),
                creds.password.as_deref().unwrap_or(""),
                &profile.database_name,
            )
            .await
        }
        "sqlite" => test_sqlite(&profile.database_name),
        _ => Err(DsmError::ConnectionError {
            message: format!("Unsupported database type: {}", profile.db_type),
        }),
    };
    #[allow(clippy::cast_possible_truncation)]
    let latency_ms = start.elapsed().as_millis() as u64;

    // Clean up SSH tunnel
    if let Some(t) = tunnel {
        let _ = t.close().await;
    }

    match result {
        Ok(db_version) => Ok(ConnectionTestResult {
            success: true,
            message: "Connection successful".to_string(),
            db_version: Some(db_version),
            latency_ms,
            error_kind: None,
            remediation: None,
        }),
        Err(e) => {
            let classified =
                crate::classify::classify_connection_error(&profile.db_type, &e.to_string());
            Ok(ConnectionTestResult {
                success: false,
                message: classified.message,
                db_version: None,
                latency_ms,
                error_kind: Some(classified.kind.to_string()),
                remediation: Some(classified.remediation),
            })
        }
    }
}

/// Default port for a given database type.
fn default_port_for(db_type: &str) -> u16 {
    match db_type {
        "mysql" => 3306,
        "postgresql" => 5432,
        _ => 0,
    }
}

/// Test `MySQL` connection by running `mysql -e "SELECT VERSION()"`.
async fn test_mysql(
    paths: &AppPaths,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    database: &str,
) -> Result<String, DsmError> {
    crate::dump::find_tool("mysql")?;

    let defaults_file =
        crate::dump::write_mysql_defaults_file(&paths.tmp_dir, host, port, username, password)?;

    let output = tokio::process::Command::new("mysql")
        .arg(format!("--defaults-extra-file={}", defaults_file.display()))
        .arg(database)
        .arg("-e")
        .arg("SELECT VERSION()")
        .arg("--skip-column-names")
        .output()
        .await
        .map_err(|e| DsmError::ConnectionError {
            message: format!("Failed to execute mysql: {e}"),
        })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&defaults_file);

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(DsmError::ConnectionError {
            message: format!("MySQL connection failed: {stderr}"),
        })
    }
}

/// Test `PostgreSQL` connection by running `psql -c "SELECT version()"`.
async fn test_postgresql(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    database: &str,
) -> Result<String, DsmError> {
    crate::dump::find_tool("psql")?;

    let output = tokio::process::Command::new("psql")
        .env("PGPASSWORD", password)
        .arg("--host")
        .arg(host)
        .arg("--port")
        .arg(port.to_string())
        .arg("--username")
        .arg(username)
        .arg(database)
        .arg("-t")
        .arg("-c")
        .arg("SELECT version()")
        .output()
        .await
        .map_err(|e| DsmError::ConnectionError {
            message: format!("Failed to execute psql: {e}"),
        })?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(DsmError::ConnectionError {
            message: format!("PostgreSQL connection failed: {stderr}"),
        })
    }
}

/// Test `SQLite` connection by checking the file exists.
fn test_sqlite(database_path: &str) -> Result<String, DsmError> {
    let path = std::path::Path::new(database_path);
    if path.exists() {
        // Try opening with rusqlite to verify it is a valid database
        let conn = rusqlite::Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        let version: String = conn.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;
        Ok(version)
    } else {
        Err(DsmError::ConnectionError {
            message: format!("SQLite database file not found: {database_path}"),
        })
    }
}

// -- Snapshot Commands --------------------------------------------------------

/// Map a `rusqlite` row to a `Snapshot` struct.
#[allow(clippy::cast_sign_loss)]
fn row_to_snapshot(row: &rusqlite::Row) -> Result<Snapshot, rusqlite::Error> {
    Ok(Snapshot {
        id: row.get("id")?,
        profile_id: row.get("profile_id")?,
        name: row.get("name")?,
        note: row.get("note")?,
        file_path: row.get("file_path")?,
        size_bytes: row.get::<_, i64>("size_bytes")? as u64,
        db_version: row.get("db_version")?,
        dump_tool_version: row.get("dump_tool_version")?,
        checksum: row.get("checksum")?,
        created_at: row.get("created_at")?,
        restored_at: row.get("restored_at")?,
    })
}

#[tauri::command]
#[allow(clippy::too_many_lines)]
pub async fn snapshot_create(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    profile_id: String,
    name: String,
    note: Option<String>,
    on_progress: Channel<SnapshotProgress>,
) -> Result<Snapshot, DsmError> {
    let start = std::time::Instant::now();
    let snapshot_id = uuid::Uuid::new_v4().to_string();

    // Load profile (scoped to drop MutexGuard before any .await)
    let profile = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT * FROM profiles WHERE id = ?1",
            params![profile_id],
            row_to_profile,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DsmError::ProfileNotFound(profile_id.clone()),
            other => DsmError::DatabaseError(other),
        })?
    };

    // Get credentials
    let creds = credentials::get_credentials(&profile_id)?;

    // Send Started progress
    let _ = on_progress.send(SnapshotProgress::Started {
        operation: "dump".to_string(),
        profile_name: profile.name.clone(),
    });

    // Open SSH tunnel if needed
    let mut tunnel: Option<crate::ssh::SshTunnel> = None;
    let (effective_host, effective_port) = if profile.ssh_enabled {
        let _ = on_progress.send(SnapshotProgress::Phase {
            phase: "ssh_tunnel".to_string(),
            message: "Opening SSH tunnel...".to_string(),
        });

        let ssh_host = profile.ssh_host.as_deref().unwrap_or("localhost");
        let ssh_port = profile.ssh_port.unwrap_or(22);
        let ssh_user = profile.ssh_user.as_deref().unwrap_or("root");
        let db_host = profile.host.as_deref().unwrap_or("127.0.0.1");
        let db_port = profile.port.unwrap_or(default_port_for(&profile.db_type));

        let t = crate::ssh::SshTunnel::open(
            ssh_host,
            ssh_port,
            ssh_user,
            creds.ssh_key_path.as_deref(),
            db_host,
            db_port,
        )
        .await?;

        let local_port = t.local_port();
        tunnel = Some(t);
        ("127.0.0.1".to_string(), local_port)
    } else {
        (
            profile
                .host
                .clone()
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            profile.port.unwrap_or(default_port_for(&profile.db_type)),
        )
    };

    // Ensure project subdirectory exists under snapshots_dir
    let project_dir = paths.snapshots_dir.join(&profile.project);
    std::fs::create_dir_all(&project_dir)?;

    let relative_path = format!("{}/{snapshot_id}.sql.gz", profile.project);
    let output_path = paths.snapshots_dir.join(&relative_path);

    // Track temp files to clean up
    let mut defaults_file: Option<std::path::PathBuf> = None;

    let _ = on_progress.send(SnapshotProgress::Phase {
        phase: "dumping".to_string(),
        message: format!("Dumping {}...", profile.database_name),
    });

    // Build and run the dump command
    let size_bytes = match profile.db_type.as_str() {
        "mysql" => {
            crate::dump::find_tool("mysqldump")?;
            let df = crate::dump::write_mysql_defaults_file(
                &paths.tmp_dir,
                &effective_host,
                effective_port,
                profile.username.as_deref().unwrap_or("root"),
                creds.password.as_deref().unwrap_or(""),
            )?;
            let mut cmd = crate::dump::build_mysqldump_command(&df, &profile.database_name);
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let child_output = cmd.output().await.map_err(|e| DsmError::DumpError {
                message: format!("Failed to run mysqldump: {e}"),
                output: String::new(),
            })?;

            if !child_output.status.success() {
                let stderr = String::from_utf8_lossy(&child_output.stderr)
                    .trim()
                    .to_string();
                let _ = std::fs::remove_file(&output_path);
                return Err(DsmError::DumpError {
                    message: format!("mysqldump exited with status {}", child_output.status),
                    output: stderr,
                });
            }

            // Compress captured stdout to .sql.gz
            let stdout_data = child_output.stdout;
            let out = output_path.clone();
            let compressed_size = tokio::task::spawn_blocking(move || {
                crate::compress::compress_from_reader(std::io::Cursor::new(stdout_data), &out)
            })
            .await
            .map_err(|e| DsmError::DumpError {
                message: format!("Compression task failed: {e}"),
                output: String::new(),
            })??;

            defaults_file = Some(df);
            compressed_size
        }
        "postgresql" => {
            crate::dump::find_tool("pg_dump")?;
            let mut cmd = crate::dump::build_pg_dump_command(
                &effective_host,
                effective_port,
                profile.username.as_deref().unwrap_or("postgres"),
                creds.password.as_deref().unwrap_or(""),
                &profile.database_name,
            );
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let child_output = cmd.output().await.map_err(|e| DsmError::DumpError {
                message: format!("Failed to run pg_dump: {e}"),
                output: String::new(),
            })?;

            if !child_output.status.success() {
                let stderr = String::from_utf8_lossy(&child_output.stderr)
                    .trim()
                    .to_string();
                let _ = std::fs::remove_file(&output_path);
                return Err(DsmError::DumpError {
                    message: format!("pg_dump exited with status {}", child_output.status),
                    output: stderr,
                });
            }

            let stdout_data = child_output.stdout;
            let out = output_path.clone();
            let compressed_size = tokio::task::spawn_blocking(move || {
                crate::compress::compress_from_reader(std::io::Cursor::new(stdout_data), &out)
            })
            .await
            .map_err(|e| DsmError::DumpError {
                message: format!("Compression task failed: {e}"),
                output: String::new(),
            })??;

            compressed_size
        }
        "sqlite" => {
            // SQLite uses VACUUM INTO which writes a .db file, then we compress it
            let temp_db_path = paths.tmp_dir.join(format!("{snapshot_id}.db"));

            let mut cmd = crate::dump::build_sqlite_dump_command(
                &profile.database_name,
                &temp_db_path.to_string_lossy(),
            );

            let child_output = cmd.output().await.map_err(|e| DsmError::DumpError {
                message: format!("Failed to run sqlite3: {e}"),
                output: String::new(),
            })?;

            if !child_output.status.success() {
                let stderr = String::from_utf8_lossy(&child_output.stderr)
                    .trim()
                    .to_string();
                return Err(DsmError::DumpError {
                    message: format!("sqlite3 VACUUM INTO failed: {stderr}"),
                    output: stderr,
                });
            }

            // Compress the temp .db file to .sql.gz
            let temp = temp_db_path.clone();
            let out = output_path.clone();
            let compressed_size =
                tokio::task::spawn_blocking(move || crate::compress::compress_file(&temp, &out))
                    .await
                    .map_err(|e| DsmError::DumpError {
                        message: format!("Compression task failed: {e}"),
                        output: String::new(),
                    })??;

            // Clean up temp .db file
            let _ = std::fs::remove_file(&temp_db_path);

            compressed_size
        }
        _ => {
            return Err(DsmError::DumpError {
                message: format!("Unsupported database type: {}", profile.db_type),
                output: String::new(),
            });
        }
    };

    // Clean up temp files
    if let Some(df) = defaults_file {
        let _ = std::fs::remove_file(&df);
    }

    // Close SSH tunnel
    if let Some(t) = tunnel {
        let _ = t.close().await;
    }

    // Compute SHA-256 checksum of the compressed snapshot file
    let _ = on_progress.send(SnapshotProgress::Phase {
        phase: "checksum".to_string(),
        message: "Computing integrity checksum...".to_string(),
    });

    let checksum_path = output_path.clone();
    let checksum =
        tokio::task::spawn_blocking(move || crate::checksum::compute_sha256(&checksum_path))
            .await
            .map_err(|e| DsmError::DumpError {
                message: format!("Checksum task failed: {e}"),
                output: String::new(),
            })??;

    // Save metadata
    let _ = on_progress.send(SnapshotProgress::Phase {
        phase: "saving".to_string(),
        message: "Saving snapshot metadata...".to_string(),
    });

    let now = chrono::Utc::now().to_rfc3339();
    {
        let conn = lock_db(&db)?;
        conn.execute(
            "INSERT INTO snapshots (id, profile_id, name, note, file_path, size_bytes, checksum, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                snapshot_id,
                profile_id,
                name,
                note,
                relative_path,
                size_bytes.cast_signed(),
                checksum,
                now,
            ],
        )?;
    }

    let duration_secs = start.elapsed().as_secs_f64();
    let _ = on_progress.send(SnapshotProgress::Completed {
        message: "Snapshot created successfully".to_string(),
        size_bytes: Some(size_bytes),
        duration_secs,
    });

    Ok(Snapshot {
        id: snapshot_id,
        profile_id,
        name,
        note,
        file_path: relative_path,
        size_bytes,
        db_version: None,
        dump_tool_version: None,
        checksum: Some(checksum),
        created_at: now,
        restored_at: None,
    })
}

#[tauri::command]
pub async fn snapshot_list(
    db: State<'_, DbState>,
    profile_id: Option<String>,
) -> Result<Vec<Snapshot>, DsmError> {
    let conn = lock_db(&db)?;

    let snapshots = if let Some(pid) = profile_id {
        let mut stmt =
            conn.prepare("SELECT * FROM snapshots WHERE profile_id = ?1 ORDER BY created_at DESC")?;
        let rows = stmt
            .query_map(params![pid], row_to_snapshot)?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    } else {
        let mut stmt = conn.prepare("SELECT * FROM snapshots ORDER BY created_at DESC")?;
        let rows = stmt
            .query_map([], row_to_snapshot)?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    Ok(snapshots)
}

#[tauri::command]
#[allow(clippy::too_many_lines)]
pub async fn snapshot_restore(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    snapshot_id: String,
    options: Option<RestoreOptions>,
    on_progress: Channel<SnapshotProgress>,
) -> Result<RestoreRecord, DsmError> {
    let start = std::time::Instant::now();
    let options = options.unwrap_or(RestoreOptions {
        target_profile_id: None,
        target_database_name: None,
    });

    // Load snapshot + original profile (scoped to drop MutexGuard before async)
    let (snapshot, original_profile) = {
        let conn = lock_db(&db)?;
        let snap = conn
            .query_row(
                "SELECT * FROM snapshots WHERE id = ?1",
                params![snapshot_id],
                row_to_snapshot,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DsmError::SnapshotNotFound(snapshot_id.clone())
                }
                other => DsmError::DatabaseError(other),
            })?;

        let prof = conn
            .query_row(
                "SELECT * FROM profiles WHERE id = ?1",
                params![snap.profile_id],
                row_to_profile,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DsmError::ProfileNotFound(snap.profile_id.clone())
                }
                other => DsmError::DatabaseError(other),
            })?;

        (snap, prof)
    };

    // Verify snapshot integrity before proceeding with restore
    if let Some(ref expected_checksum) = snapshot.checksum {
        let gz_path = paths.snapshots_dir.join(&snapshot.file_path);
        if !gz_path.exists() {
            return Err(DsmError::RestoreError {
                message: "Snapshot file is missing from disk.".to_string(),
                output: String::new(),
            });
        }

        let path = gz_path.clone();
        let actual = tokio::task::spawn_blocking(move || crate::checksum::compute_sha256(&path))
            .await
            .map_err(|e| DsmError::RestoreError {
                message: format!("Integrity check failed: {e}"),
                output: String::new(),
            })??;

        if actual != *expected_checksum {
            return Err(DsmError::RestoreError {
                message: format!(
                    "Integrity check FAILED — the snapshot file has been modified or corrupted.\nExpected: {expected_checksum}\nActual:   {actual}"
                ),
                output: String::new(),
            });
        }
    }

    // Resolve effective profile — use target override if provided
    let profile = if let Some(ref target_id) = options.target_profile_id {
        if *target_id == snapshot.profile_id {
            original_profile
        } else {
            let conn = lock_db(&db)?;
            let target_profile = conn
                .query_row(
                    "SELECT * FROM profiles WHERE id = ?1",
                    params![target_id],
                    row_to_profile,
                )
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => {
                        DsmError::ProfileNotFound(target_id.clone())
                    }
                    other => DsmError::DatabaseError(other),
                })?;

            // Validate db_type match
            crate::ensure_db::validate_db_type_match(
                &original_profile.db_type,
                &target_profile.db_type,
            )?;

            target_profile
        }
    } else {
        original_profile
    };

    // Resolve effective database name
    let effective_db_name = options
        .target_database_name
        .clone()
        .unwrap_or_else(|| profile.database_name.clone());

    // Get credentials for the effective profile
    let creds = credentials::get_credentials(&profile.id)?;

    // Send Started progress
    let _ = on_progress.send(SnapshotProgress::Started {
        operation: "restore".to_string(),
        profile_name: profile.name.clone(),
    });

    // Open SSH tunnel if needed
    let mut tunnel: Option<crate::ssh::SshTunnel> = None;
    let (effective_host, effective_port) = if profile.ssh_enabled {
        let _ = on_progress.send(SnapshotProgress::Phase {
            phase: "ssh_tunnel".to_string(),
            message: "Opening SSH tunnel...".to_string(),
        });

        let ssh_host = profile.ssh_host.as_deref().unwrap_or("localhost");
        let ssh_port = profile.ssh_port.unwrap_or(22);
        let ssh_user = profile.ssh_user.as_deref().unwrap_or("root");
        let db_host = profile.host.as_deref().unwrap_or("127.0.0.1");
        let db_port = profile.port.unwrap_or(default_port_for(&profile.db_type));

        let t = crate::ssh::SshTunnel::open(
            ssh_host,
            ssh_port,
            ssh_user,
            creds.ssh_key_path.as_deref(),
            db_host,
            db_port,
        )
        .await?;

        let local_port = t.local_port();
        tunnel = Some(t);
        ("127.0.0.1".to_string(), local_port)
    } else {
        (
            profile
                .host
                .clone()
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            profile.port.unwrap_or(default_port_for(&profile.db_type)),
        )
    };

    // Ensure target database exists (only when overriding database name)
    if options.target_database_name.is_some() {
        let _ = on_progress.send(SnapshotProgress::Phase {
            phase: "creating_database".to_string(),
            message: format!("Ensuring database '{effective_db_name}' exists..."),
        });

        match profile.db_type.as_str() {
            "mysql" => {
                crate::dump::find_tool("mysql")?;
                let df = crate::dump::write_mysql_defaults_file(
                    &paths.tmp_dir,
                    &effective_host,
                    effective_port,
                    profile.username.as_deref().unwrap_or("root"),
                    creds.password.as_deref().unwrap_or(""),
                )?;
                let args = crate::ensure_db::mysql_create_db_args(
                    &df.to_string_lossy(),
                    &effective_db_name,
                );
                let output = tokio::process::Command::new("mysql")
                    .args(&args)
                    .output()
                    .await
                    .map_err(|e| DsmError::RestoreError {
                        message: format!("Failed to create database: {e}"),
                        output: String::new(),
                    })?;
                let _ = std::fs::remove_file(&df);
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    return Err(DsmError::RestoreError {
                        message: format!(
                            "Failed to create MySQL database '{effective_db_name}': {stderr}"
                        ),
                        output: stderr,
                    });
                }
            }
            "postgresql" => {
                crate::dump::find_tool("psql")?;
                let password = creds.password.as_deref().unwrap_or("");
                let username = profile.username.as_deref().unwrap_or("postgres");

                // Check if database exists
                let check_args = crate::ensure_db::pg_check_db_args(
                    &effective_host,
                    effective_port,
                    username,
                    &effective_db_name,
                );
                let check_output = tokio::process::Command::new("psql")
                    .env("PGPASSWORD", password)
                    .args(&check_args)
                    .output()
                    .await
                    .map_err(|e| DsmError::RestoreError {
                        message: format!("Failed to check database existence: {e}"),
                        output: String::new(),
                    })?;

                let stdout = String::from_utf8_lossy(&check_output.stdout)
                    .trim()
                    .to_string();
                if stdout != "1" {
                    // Database doesn't exist, create it
                    let create_args = crate::ensure_db::pg_create_db_args(
                        &effective_host,
                        effective_port,
                        username,
                        &effective_db_name,
                    );
                    let create_output = tokio::process::Command::new("psql")
                        .env("PGPASSWORD", password)
                        .args(&create_args)
                        .output()
                        .await
                        .map_err(|e| DsmError::RestoreError {
                            message: format!("Failed to create database: {e}"),
                            output: String::new(),
                        })?;
                    if !create_output.status.success() {
                        let stderr = String::from_utf8_lossy(&create_output.stderr)
                            .trim()
                            .to_string();
                        return Err(DsmError::RestoreError {
                            message: format!("Failed to create PostgreSQL database '{effective_db_name}': {stderr}"),
                            output: stderr,
                        });
                    }
                }
            }
            "sqlite" => {
                crate::ensure_db::ensure_sqlite_parent_dir(&effective_db_name)?;
            }
            _ => {}
        }
    }

    let gz_path = paths.snapshots_dir.join(&snapshot.file_path);
    let mut defaults_file: Option<std::path::PathBuf> = None;

    let _ = on_progress.send(SnapshotProgress::Phase {
        phase: "restoring".to_string(),
        message: format!("Restoring to {effective_db_name}..."),
    });

    match profile.db_type.as_str() {
        "mysql" => {
            let tool_path = crate::dump::find_tool("mysql")?;
            let df = crate::dump::write_mysql_defaults_file(
                &paths.tmp_dir,
                &effective_host,
                effective_port,
                profile.username.as_deref().unwrap_or("root"),
                creds.password.as_deref().unwrap_or(""),
            )?;

            let temp_sql = paths
                .tmp_dir
                .join(format!("restore-{}.sql", uuid::Uuid::new_v4()));

            let gz = gz_path.clone();
            let sql_path = temp_sql.clone();
            tokio::task::spawn_blocking(move || {
                let mut out_file = std::fs::File::create(&sql_path)?;
                crate::compress::decompress_to_writer(&gz, &mut out_file)
            })
            .await
            .map_err(|e| DsmError::RestoreError {
                message: format!("Decompression task failed: {e}"),
                output: String::new(),
            })??;

            let df_clone = df.clone();
            let db_name = effective_db_name.clone();
            let sql_input = temp_sql.clone();
            let output = tokio::task::spawn_blocking(move || {
                let input_file = std::fs::File::open(&sql_input)?;
                std::process::Command::new(tool_path)
                    .arg(format!("--defaults-extra-file={}", df_clone.display()))
                    .arg(&db_name)
                    .stdin(input_file)
                    .stderr(std::process::Stdio::piped())
                    .output()
            })
            .await
            .map_err(|e| DsmError::RestoreError {
                message: format!("Restore task failed: {e}"),
                output: String::new(),
            })?
            .map_err(|e| DsmError::RestoreError {
                message: format!("Failed to run mysql: {e}"),
                output: String::new(),
            })?;

            let _ = std::fs::remove_file(&temp_sql);

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(DsmError::RestoreError {
                    message: format!("mysql exited with status {}", output.status),
                    output: stderr,
                });
            }

            defaults_file = Some(df);
        }
        "postgresql" => {
            let tool_path = crate::dump::find_tool("psql")?;

            let temp_sql = paths
                .tmp_dir
                .join(format!("restore-{}.sql", uuid::Uuid::new_v4()));

            let gz = gz_path.clone();
            let sql_path = temp_sql.clone();
            tokio::task::spawn_blocking(move || {
                let mut out_file = std::fs::File::create(&sql_path)?;
                crate::compress::decompress_to_writer(&gz, &mut out_file)
            })
            .await
            .map_err(|e| DsmError::RestoreError {
                message: format!("Decompression task failed: {e}"),
                output: String::new(),
            })??;

            let host = effective_host.clone();
            let port = effective_port;
            let user = profile
                .username
                .clone()
                .unwrap_or_else(|| "postgres".to_string());
            let pass = creds.password.clone().unwrap_or_default();
            let db_name = effective_db_name.clone();
            let sql_input = temp_sql.clone();
            let output = tokio::task::spawn_blocking(move || {
                let input_file = std::fs::File::open(&sql_input)?;
                std::process::Command::new(tool_path)
                    .env("PGPASSWORD", &pass)
                    .arg("--host")
                    .arg(&host)
                    .arg("--port")
                    .arg(port.to_string())
                    .arg("--username")
                    .arg(&user)
                    .arg(&db_name)
                    .stdin(input_file)
                    .stderr(std::process::Stdio::piped())
                    .output()
            })
            .await
            .map_err(|e| DsmError::RestoreError {
                message: format!("Restore task failed: {e}"),
                output: String::new(),
            })?
            .map_err(|e| DsmError::RestoreError {
                message: format!("Failed to run psql: {e}"),
                output: String::new(),
            })?;

            let _ = std::fs::remove_file(&temp_sql);

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(DsmError::RestoreError {
                    message: format!("psql exited with status {}", output.status),
                    output: stderr,
                });
            }
        }
        "sqlite" => {
            let temp_db_path = paths
                .tmp_dir
                .join(format!("restore-{}.db", uuid::Uuid::new_v4()));

            let temp = temp_db_path.clone();
            let gz = gz_path.clone();
            tokio::task::spawn_blocking(move || {
                let mut out_file = std::fs::File::create(&temp)?;
                crate::compress::decompress_to_writer(&gz, &mut out_file)
            })
            .await
            .map_err(|e| DsmError::RestoreError {
                message: format!("Decompression task failed: {e}"),
                output: String::new(),
            })??;

            std::fs::copy(&temp_db_path, &effective_db_name).map_err(|e| {
                DsmError::RestoreError {
                    message: format!("Failed to copy restored database: {e}"),
                    output: String::new(),
                }
            })?;

            let _ = std::fs::remove_file(&temp_db_path);
        }
        _ => {
            return Err(DsmError::RestoreError {
                message: format!("Unsupported database type: {}", profile.db_type),
                output: String::new(),
            });
        }
    }

    // Clean up temp files
    if let Some(df) = defaults_file {
        let _ = std::fs::remove_file(&df);
    }

    // Close SSH tunnel
    if let Some(t) = tunnel {
        let _ = t.close().await;
    }

    let duration_secs = start.elapsed().as_secs_f64();

    // Update restored_at timestamp and insert restore history record
    let record = {
        let conn = lock_db(&db)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE snapshots SET restored_at = ?1 WHERE id = ?2",
            params![now, snapshot_id],
        )?;

        let record_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO restore_history (id, snapshot_id, snapshot_name, target_profile_id, target_db_name, duration_secs, restored_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record_id,
                snapshot_id,
                snapshot.name,
                profile.id,
                effective_db_name,
                duration_secs,
                now,
            ],
        )?;

        RestoreRecord {
            id: record_id,
            snapshot_id,
            snapshot_name: snapshot.name,
            target_profile_id: profile.id,
            target_db_name: effective_db_name,
            duration_secs,
            restored_at: now,
        }
    };

    let _ = on_progress.send(SnapshotProgress::Completed {
        message: "Restore completed successfully".to_string(),
        size_bytes: None,
        duration_secs,
    });

    Ok(record)
}

#[tauri::command]
pub async fn snapshot_delete(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    snapshot_id: String,
) -> Result<(), DsmError> {
    let conn = lock_db(&db)?;

    // Load snapshot metadata
    let file_path: String = conn
        .query_row(
            "SELECT file_path FROM snapshots WHERE id = ?1",
            params![snapshot_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DsmError::SnapshotNotFound(snapshot_id.clone()),
            other => DsmError::DatabaseError(other),
        })?;

    // Delete .sql.gz file from disk
    let full_path = paths.snapshots_dir.join(&file_path);
    if full_path.exists() {
        let _ = std::fs::remove_file(&full_path);
    }

    // Delete record from snapshots table
    conn.execute("DELETE FROM snapshots WHERE id = ?1", params![snapshot_id])?;

    Ok(())
}

#[tauri::command]
pub async fn snapshot_delete_by_project(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    project: String,
) -> Result<(), DsmError> {
    let conn = lock_db(&db)?;

    // Get all snapshot file paths for this project
    let mut stmt = conn.prepare(
        "SELECT s.file_path FROM snapshots s
         JOIN profiles p ON s.profile_id = p.id
         WHERE p.project = ?1",
    )?;
    let file_paths: Vec<String> = stmt
        .query_map(params![project], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();

    // Delete files from disk
    for file_path in &file_paths {
        let full_path = paths.snapshots_dir.join(file_path);
        if full_path.exists() {
            let _ = std::fs::remove_file(&full_path);
        }
    }

    // Delete snapshot records for all profiles in this project
    conn.execute(
        "DELETE FROM snapshots WHERE profile_id IN (SELECT id FROM profiles WHERE project = ?1)",
        params![project],
    )?;

    // Remove the project subdirectory if empty
    let project_dir = paths.snapshots_dir.join(&project);
    if project_dir.exists() {
        let _ = std::fs::remove_dir(&project_dir);
    }

    Ok(())
}

// -- Integrity Commands -------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityResult {
    pub snapshot_id: String,
    pub valid: bool,
    pub expected_checksum: Option<String>,
    pub actual_checksum: Option<String>,
    pub message: String,
}

#[tauri::command]
pub async fn snapshot_verify_integrity(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    snapshot_id: String,
) -> Result<IntegrityResult, DsmError> {
    let snapshot = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT * FROM snapshots WHERE id = ?1",
            params![snapshot_id],
            row_to_snapshot,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DsmError::SnapshotNotFound(snapshot_id.clone()),
            other => DsmError::DatabaseError(other),
        })?
    };

    let Some(expected) = &snapshot.checksum else {
        return Ok(IntegrityResult {
            snapshot_id,
            valid: false,
            expected_checksum: None,
            actual_checksum: None,
            message:
                "No checksum recorded — snapshot was created before integrity tracking was enabled."
                    .to_string(),
        });
    };

    let gz_path = paths.snapshots_dir.join(&snapshot.file_path);
    if !gz_path.exists() {
        return Ok(IntegrityResult {
            snapshot_id,
            valid: false,
            expected_checksum: Some(expected.clone()),
            actual_checksum: None,
            message: "Snapshot file is missing from disk.".to_string(),
        });
    }

    let path = gz_path.clone();
    let actual = tokio::task::spawn_blocking(move || crate::checksum::compute_sha256(&path))
        .await
        .map_err(|e| {
            DsmError::FileSystemError(std::io::Error::other(format!("Checksum task failed: {e}")))
        })??;

    let valid = actual == *expected;
    let message = if valid {
        "Integrity verified — checksum matches.".to_string()
    } else {
        "Integrity check FAILED — the snapshot file has been modified or corrupted.".to_string()
    };

    Ok(IntegrityResult {
        snapshot_id,
        valid,
        expected_checksum: Some(expected.clone()),
        actual_checksum: Some(actual),
        message,
    })
}

// -- Restore History Commands -------------------------------------------------

#[tauri::command]
pub async fn restore_history_list(
    db: State<'_, DbState>,
    limit: Option<u32>,
) -> Result<Vec<RestoreRecord>, DsmError> {
    let conn = lock_db(&db)?;
    let effective_limit = limit.unwrap_or(100);
    let mut stmt =
        conn.prepare("SELECT * FROM restore_history ORDER BY restored_at DESC LIMIT ?1")?;
    let records = stmt
        .query_map(params![effective_limit], row_to_restore_record)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(records)
}

// -- Storage Commands ---------------------------------------------------------

#[tauri::command]
pub async fn storage_usage(
    _db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
) -> Result<StorageInfo, DsmError> {
    let mut projects: std::collections::HashMap<String, (u64, u32)> =
        std::collections::HashMap::new();
    let mut total_bytes: u64 = 0;

    // Walk the snapshots directory: each subdirectory is a project
    if let Ok(entries) = std::fs::read_dir(&paths.snapshots_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let project_name = entry.file_name().to_string_lossy().to_string();

            if let Ok(files) = std::fs::read_dir(&path) {
                for file_entry in files.flatten() {
                    let file_path = file_entry.path();
                    if file_path.extension().is_some_and(|ext| ext == "gz") {
                        if let Ok(metadata) = std::fs::metadata(&file_path) {
                            let size = metadata.len();
                            total_bytes += size;
                            let entry = projects.entry(project_name.clone()).or_insert((0, 0));
                            entry.0 += size;
                            entry.1 += 1;
                        }
                    }
                }
            }
        }
    }

    let project_list = projects
        .into_iter()
        .map(|(project, (size_bytes, snapshot_count))| ProjectStorage {
            project,
            size_bytes,
            snapshot_count,
        })
        .collect();

    Ok(StorageInfo {
        total_bytes,
        projects: project_list,
    })
}

// -- Filesystem Commands ------------------------------------------------------

#[tauri::command]
pub async fn get_snapshots_dir(paths: State<'_, AppPaths>) -> Result<String, DsmError> {
    Ok(paths.snapshots_dir.to_string_lossy().to_string())
}

// -- Settings Commands --------------------------------------------------------

#[tauri::command]
pub async fn settings_list(
    db: State<'_, DbState>,
) -> Result<std::collections::HashMap<String, String>, DsmError> {
    let conn = lock_db(&db)?;
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let settings = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(Result::ok)
        .collect();
    Ok(settings)
}

#[tauri::command]
pub async fn settings_set(
    db: State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), DsmError> {
    let conn = lock_db(&db)?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Create an in-memory database with the app schema applied.
    fn test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        crate::db::migrations_for_test()
            .to_latest(&mut conn)
            .unwrap();
        conn
    }

    /// Insert a minimal profile and return its id.
    fn insert_profile(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO profiles (id, project, name, db_type, database_name, created_at, updated_at)
             VALUES (?1, 'proj', 'name', 'mysql', 'mydb', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![id],
        )
        .unwrap();
    }

    /// Insert a minimal snapshot and return its id.
    fn insert_snapshot(conn: &Connection, id: &str, profile_id: &str) {
        conn.execute(
            "INSERT INTO snapshots (id, profile_id, name, file_path, size_bytes, created_at)
             VALUES (?1, ?2, 'snap-name', 'test/snap.sql.gz', 1024, '2026-01-01T00:00:00Z')",
            params![id, profile_id],
        )
        .unwrap();
    }

    #[test]
    fn test_row_to_restore_record_maps_correctly() {
        let conn = test_db();
        insert_profile(&conn, "prof-1");
        insert_snapshot(&conn, "snap-1", "prof-1");

        conn.execute(
            "INSERT INTO restore_history (id, snapshot_id, snapshot_name, target_profile_id, target_db_name, duration_secs, restored_at)
             VALUES ('rec-1', 'snap-1', 'My Snapshot', 'prof-1', 'target_db', 1.23, '2026-02-01T12:00:00Z')",
            [],
        )
        .unwrap();

        let record = conn
            .query_row(
                "SELECT * FROM restore_history WHERE id = 'rec-1'",
                [],
                row_to_restore_record,
            )
            .unwrap();

        assert_eq!(record.id, "rec-1");
        assert_eq!(record.snapshot_id, "snap-1");
        assert_eq!(record.snapshot_name, "My Snapshot");
        assert_eq!(record.target_profile_id, "prof-1");
        assert_eq!(record.target_db_name, "target_db");
        assert!((record.duration_secs - 1.23).abs() < f64::EPSILON);
        assert_eq!(record.restored_at, "2026-02-01T12:00:00Z");
    }

    #[test]
    fn test_restore_history_ordered_by_most_recent() {
        let conn = test_db();
        insert_profile(&conn, "prof-1");
        insert_snapshot(&conn, "snap-1", "prof-1");

        // Insert in non-chronological order
        conn.execute(
            "INSERT INTO restore_history (id, snapshot_id, snapshot_name, target_profile_id, target_db_name, duration_secs, restored_at)
             VALUES ('rec-old', 'snap-1', 'Old', 'prof-1', 'db1', 0.5, '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO restore_history (id, snapshot_id, snapshot_name, target_profile_id, target_db_name, duration_secs, restored_at)
             VALUES ('rec-new', 'snap-1', 'New', 'prof-1', 'db2', 1.0, '2026-02-01T00:00:00Z')",
            [],
        )
        .unwrap();

        let mut stmt = conn
            .prepare("SELECT * FROM restore_history ORDER BY restored_at DESC LIMIT 10")
            .unwrap();
        let records: Vec<RestoreRecord> = stmt
            .query_map([], row_to_restore_record)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, "rec-new");
        assert_eq!(records[1].id, "rec-old");
    }
}

use crate::credentials::{self, ProfileCredentials};
use crate::db::{AppPaths, DbState};
use crate::error::DsmError;
use crate::progress::SnapshotProgress;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub pinned: bool,
    pub tags: Vec<String>,
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
    pub pre_restore_snapshot_id: Option<String>,
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
        pre_restore_snapshot_id: None, // Not stored in restore_history — only returned from live restores
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

/// Format bytes as a short human-readable string.
#[allow(clippy::cast_precision_loss)]
fn format_bytes_short(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
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

/// Map a `rusqlite` row to a `Snapshot` struct (tags populated separately).
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
        pinned: row.get::<_, i32>("pinned").unwrap_or(0) != 0,
        tags: Vec::new(), // Populated by load_tags_for_snapshots
    })
}

/// Load tags for a batch of snapshots from the database.
fn load_tags_for_snapshots(
    conn: &rusqlite::Connection,
    snapshots: &mut [Snapshot],
) -> Result<(), rusqlite::Error> {
    if snapshots.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = snapshots.iter().map(|s| s.id.as_str()).collect();
    let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT snapshot_id, tag FROM snapshot_tags WHERE snapshot_id IN ({}) ORDER BY tag",
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (snapshot_id, tag) = row?;
        tag_map.entry(snapshot_id).or_default().push(tag);
    }
    for snapshot in snapshots.iter_mut() {
        if let Some(tags) = tag_map.remove(&snapshot.id) {
            snapshot.tags = tags;
        }
    }
    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub async fn snapshot_create(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    locks: State<'_, crate::db::OperationLocks>,
    profile_id: String,
    name: String,
    note: Option<String>,
    tags: Option<Vec<String>>,
    on_progress: Channel<SnapshotProgress>,
) -> Result<Snapshot, DsmError> {
    // Acquire per-profile operation lock — prevents concurrent snapshot/restore
    let _op_guard = acquire_profile_lock(&locks, &profile_id, "snapshot")?;

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

    // Capture tool and database versions for compatibility tracking
    let dump_tool_version = detect_tool_version(&profile.db_type).await;

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

            // Compress captured stdout to .sql.gz with streaming progress
            let stdout_data = child_output.stdout;
            let out = output_path.clone();
            let progress_channel = on_progress.clone();
            let compressed_size = tokio::task::spawn_blocking(move || {
                crate::compress::compress_from_reader_with_progress(
                    std::io::Cursor::new(stdout_data),
                    &out,
                    |bytes, table| {
                        let msg = if let Some(t) = table {
                            format!(
                                "Compressing... table: {t} ({} processed)",
                                format_bytes_short(bytes)
                            )
                        } else {
                            format!("Compressing... ({} processed)", format_bytes_short(bytes))
                        };
                        let _ = progress_channel.send(SnapshotProgress::Phase {
                            phase: "compressing".to_string(),
                            message: msg,
                        });
                        if let Some(t) = table {
                            let _ = progress_channel.send(SnapshotProgress::TableProgress {
                                current_table: t.to_string(),
                                tables_completed: 0,
                                total_tables: 0,
                                bytes_processed: bytes,
                            });
                        }
                    },
                )
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
            let progress_channel = on_progress.clone();
            let compressed_size = tokio::task::spawn_blocking(move || {
                crate::compress::compress_from_reader_with_progress(
                    std::io::Cursor::new(stdout_data),
                    &out,
                    |bytes, table| {
                        let msg = if let Some(t) = table {
                            format!(
                                "Compressing... table: {t} ({} processed)",
                                format_bytes_short(bytes)
                            )
                        } else {
                            format!("Compressing... ({} processed)", format_bytes_short(bytes))
                        };
                        let _ = progress_channel.send(SnapshotProgress::Phase {
                            phase: "compressing".to_string(),
                            message: msg,
                        });
                    },
                )
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
    let snapshot_tags = tags.unwrap_or_default();
    {
        let conn = lock_db(&db)?;
        conn.execute(
            "INSERT INTO snapshots (id, profile_id, name, note, file_path, size_bytes, db_version, dump_tool_version, checksum, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                snapshot_id,
                profile_id,
                name,
                note,
                relative_path,
                size_bytes.cast_signed(),
                dump_tool_version.as_ref().map(|v| v.1.as_str()),
                dump_tool_version.as_ref().map(|v| v.0.as_str()),
                checksum,
                now,
            ],
        )?;

        // Insert tags
        for tag in &snapshot_tags {
            conn.execute(
                "INSERT OR IGNORE INTO snapshot_tags (snapshot_id, tag) VALUES (?1, ?2)",
                params![snapshot_id, tag],
            )?;
        }
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
        db_version: dump_tool_version.as_ref().map(|v| v.1.clone()),
        dump_tool_version: dump_tool_version.map(|v| v.0),
        checksum: Some(checksum),
        created_at: now,
        restored_at: None,
        pinned: false,
        tags: snapshot_tags,
    })
}

#[tauri::command]
pub async fn snapshot_list(
    db: State<'_, DbState>,
    profile_id: Option<String>,
) -> Result<Vec<Snapshot>, DsmError> {
    let conn = lock_db(&db)?;

    let mut snapshots = if let Some(pid) = profile_id {
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

    load_tags_for_snapshots(&conn, &mut snapshots)?;

    Ok(snapshots)
}

/// Internal helper: create a pre-restore safety snapshot of the current database state.
/// This runs the dump pipeline without progress streaming and tags the result as "[auto] pre-restore".
/// Returns the snapshot ID on success, or an error if the capture fails.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
async fn create_pre_restore_snapshot(
    db: &DbState,
    paths: &AppPaths,
    profile: &Profile,
    creds: &ProfileCredentials,
    effective_host: &str,
    effective_port: u16,
    effective_db_name: &str,
    restoring_snapshot_name: &str,
) -> Result<String, DsmError> {
    let snapshot_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    let name = format!("[auto] pre-restore {}", now.format("%Y-%m-%d %H:%M"));
    let note = Some(format!(
        "Automatic safety snapshot before restoring '{restoring_snapshot_name}'"
    ));

    // Ensure project subdirectory exists
    let project_dir = paths.snapshots_dir.join(&profile.project);
    std::fs::create_dir_all(&project_dir)?;

    let relative_path = format!("{}/{snapshot_id}.sql.gz", profile.project);
    let output_path = paths.snapshots_dir.join(&relative_path);

    let size_bytes = match profile.db_type.as_str() {
        "mysql" => {
            crate::dump::find_tool("mysqldump")?;
            let df = crate::dump::write_mysql_defaults_file(
                &paths.tmp_dir,
                effective_host,
                effective_port,
                profile.username.as_deref().unwrap_or("root"),
                creds.password.as_deref().unwrap_or(""),
            )?;
            let mut cmd = crate::dump::build_mysqldump_command(&df, effective_db_name);
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let child_output = cmd.output().await.map_err(|e| DsmError::DumpError {
                message: format!("Pre-restore snapshot failed (mysqldump): {e}"),
                output: String::new(),
            })?;

            if !child_output.status.success() {
                let stderr = String::from_utf8_lossy(&child_output.stderr)
                    .trim()
                    .to_string();
                let _ = std::fs::remove_file(&df);
                return Err(DsmError::DumpError {
                    message: format!(
                        "Pre-restore snapshot failed: mysqldump exited with status {}",
                        child_output.status
                    ),
                    output: stderr,
                });
            }

            let stdout_data = child_output.stdout;
            let out = output_path.clone();
            let compressed_size = tokio::task::spawn_blocking(move || {
                crate::compress::compress_from_reader_with_progress(
                    std::io::Cursor::new(stdout_data),
                    &out,
                    |_, _| {},
                )
            })
            .await
            .map_err(|e| DsmError::DumpError {
                message: format!("Pre-restore compression failed: {e}"),
                output: String::new(),
            })??;

            let _ = std::fs::remove_file(&df);
            compressed_size
        }
        "postgresql" => {
            crate::dump::find_tool("pg_dump")?;
            let mut cmd = crate::dump::build_pg_dump_command(
                effective_host,
                effective_port,
                profile.username.as_deref().unwrap_or("postgres"),
                creds.password.as_deref().unwrap_or(""),
                effective_db_name,
            );
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let child_output = cmd.output().await.map_err(|e| DsmError::DumpError {
                message: format!("Pre-restore snapshot failed (pg_dump): {e}"),
                output: String::new(),
            })?;

            if !child_output.status.success() {
                let stderr = String::from_utf8_lossy(&child_output.stderr)
                    .trim()
                    .to_string();
                return Err(DsmError::DumpError {
                    message: format!(
                        "Pre-restore snapshot failed: pg_dump exited with status {}",
                        child_output.status
                    ),
                    output: stderr,
                });
            }

            let stdout_data = child_output.stdout;
            let out = output_path.clone();
            let compressed_size = tokio::task::spawn_blocking(move || {
                crate::compress::compress_from_reader_with_progress(
                    std::io::Cursor::new(stdout_data),
                    &out,
                    |_, _| {},
                )
            })
            .await
            .map_err(|e| DsmError::DumpError {
                message: format!("Pre-restore compression failed: {e}"),
                output: String::new(),
            })??;

            compressed_size
        }
        "sqlite" => {
            let temp_db_path = paths.tmp_dir.join(format!("{snapshot_id}.db"));
            let mut cmd = crate::dump::build_sqlite_dump_command(
                effective_db_name,
                &temp_db_path.to_string_lossy(),
            );

            let child_output = cmd.output().await.map_err(|e| DsmError::DumpError {
                message: format!("Pre-restore snapshot failed (sqlite3): {e}"),
                output: String::new(),
            })?;

            if !child_output.status.success() {
                let stderr = String::from_utf8_lossy(&child_output.stderr)
                    .trim()
                    .to_string();
                return Err(DsmError::DumpError {
                    message: format!(
                        "Pre-restore snapshot failed: sqlite3 VACUUM INTO failed: {stderr}"
                    ),
                    output: stderr,
                });
            }

            let temp = temp_db_path.clone();
            let out = output_path.clone();
            let compressed_size =
                tokio::task::spawn_blocking(move || crate::compress::compress_file(&temp, &out))
                    .await
                    .map_err(|e| DsmError::DumpError {
                        message: format!("Pre-restore compression failed: {e}"),
                        output: String::new(),
                    })??;

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

    // Compute checksum
    let checksum_path = output_path.clone();
    let checksum =
        tokio::task::spawn_blocking(move || crate::checksum::compute_sha256(&checksum_path))
            .await
            .map_err(|e| DsmError::DumpError {
                message: format!("Pre-restore checksum failed: {e}"),
                output: String::new(),
            })??;

    // Save metadata
    let now_str = now.to_rfc3339();
    {
        let conn = lock_db(db)?;
        conn.execute(
            "INSERT INTO snapshots (id, profile_id, name, note, file_path, size_bytes, checksum, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                snapshot_id,
                profile.id,
                name,
                note,
                relative_path,
                size_bytes.cast_signed(),
                checksum,
                now_str,
            ],
        )?;

        // Tag as auto pre-restore
        conn.execute(
            "INSERT OR IGNORE INTO snapshot_tags (snapshot_id, tag) VALUES (?1, ?2)",
            params![snapshot_id, "[auto] pre-restore"],
        )?;
    }

    Ok(snapshot_id)
}

#[tauri::command]
#[allow(clippy::too_many_lines)]
pub async fn snapshot_restore(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    locks: State<'_, crate::db::OperationLocks>,
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

    // Acquire per-profile operation lock — prevents concurrent snapshot/restore
    let effective_profile_id = options
        .target_profile_id
        .as_deref()
        .unwrap_or(&snapshot.profile_id);
    let _op_guard = acquire_profile_lock(&locks, effective_profile_id, "restore")?;

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

    // Create a pre-restore safety snapshot of the current database state.
    // If this fails, abort the restore — safety-first: don't proceed without a rollback path.
    let _ = on_progress.send(SnapshotProgress::Phase {
        phase: "pre_restore_snapshot".to_string(),
        message: "Capturing pre-restore safety snapshot...".to_string(),
    });

    let pre_restore_snapshot_id = create_pre_restore_snapshot(
        &db,
        &paths,
        &profile,
        &creds,
        &effective_host,
        effective_port,
        &effective_db_name,
        &snapshot.name,
    )
    .await
    .map_err(|e| DsmError::RestoreError {
        message: format!("Pre-restore safety snapshot failed — restore aborted. {e}"),
        output: String::new(),
    })?;

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
            pre_restore_snapshot_id: Some(pre_restore_snapshot_id.clone()),
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

// -- Tagging Commands ---------------------------------------------------------

#[tauri::command]
pub async fn snapshot_add_tags(
    db: State<'_, DbState>,
    snapshot_id: String,
    tags: Vec<String>,
) -> Result<Vec<String>, DsmError> {
    let conn = lock_db(&db)?;
    for tag in &tags {
        let normalised = tag.trim().to_lowercase();
        if !normalised.is_empty() {
            conn.execute(
                "INSERT OR IGNORE INTO snapshot_tags (snapshot_id, tag) VALUES (?1, ?2)",
                params![snapshot_id, normalised],
            )?;
        }
    }
    // Return all tags for this snapshot
    let mut stmt =
        conn.prepare("SELECT tag FROM snapshot_tags WHERE snapshot_id = ?1 ORDER BY tag")?;
    let result = stmt
        .query_map(params![snapshot_id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(result)
}

#[tauri::command]
pub async fn snapshot_remove_tag(
    db: State<'_, DbState>,
    snapshot_id: String,
    tag: String,
) -> Result<Vec<String>, DsmError> {
    let conn = lock_db(&db)?;
    conn.execute(
        "DELETE FROM snapshot_tags WHERE snapshot_id = ?1 AND tag = ?2",
        params![snapshot_id, tag],
    )?;
    let mut stmt =
        conn.prepare("SELECT tag FROM snapshot_tags WHERE snapshot_id = ?1 ORDER BY tag")?;
    let result = stmt
        .query_map(params![snapshot_id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(result)
}

#[tauri::command]
pub async fn snapshot_list_all_tags(db: State<'_, DbState>) -> Result<Vec<String>, DsmError> {
    let conn = lock_db(&db)?;
    let mut stmt = conn.prepare("SELECT DISTINCT tag FROM snapshot_tags ORDER BY tag")?;
    let result = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(result)
}

// -- Pinning Commands ---------------------------------------------------------

#[tauri::command]
pub async fn snapshot_set_pinned(
    db: State<'_, DbState>,
    snapshot_id: String,
    pinned: bool,
) -> Result<(), DsmError> {
    let conn = lock_db(&db)?;
    conn.execute(
        "UPDATE snapshots SET pinned = ?1 WHERE id = ?2",
        params![i32::from(pinned), snapshot_id],
    )?;
    Ok(())
}

// -- Size Estimation Command --------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeEstimation {
    pub estimated_raw_bytes: u64,
    pub estimated_compressed_bytes: u64,
    pub compression_ratio: f64,
    pub estimation_method: String,
}

#[tauri::command]
#[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
pub async fn snapshot_estimate_size(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    profile_id: String,
) -> Result<SizeEstimation, DsmError> {
    // Load profile
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

    let creds = credentials::get_credentials(&profile_id)?;

    // Get the raw database size
    let raw_bytes = match profile.db_type.as_str() {
        "mysql" => {
            let df = crate::dump::write_mysql_defaults_file(
                &paths.tmp_dir,
                profile.host.as_deref().unwrap_or("127.0.0.1"),
                profile.port.unwrap_or(3306),
                profile.username.as_deref().unwrap_or("root"),
                creds.password.as_deref().unwrap_or(""),
            )?;
            let output = tokio::process::Command::new("mysql")
                .arg(format!("--defaults-extra-file={}", df.display()))
                .arg(&profile.database_name)
                .arg("-e")
                .arg("SELECT SUM(data_length + index_length) FROM information_schema.TABLES WHERE table_schema = DATABASE()")
                .arg("--skip-column-names")
                .output()
                .await
                .map_err(|e| DsmError::ConnectionError {
                    message: format!("Failed to estimate size: {e}"),
                })?;
            let _ = std::fs::remove_file(&df);
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                stdout.parse::<u64>().unwrap_or(0)
            } else {
                0
            }
        }
        "postgresql" => {
            let output = tokio::process::Command::new("psql")
                .env("PGPASSWORD", creds.password.as_deref().unwrap_or(""))
                .arg("--host")
                .arg(profile.host.as_deref().unwrap_or("127.0.0.1"))
                .arg("--port")
                .arg(profile.port.unwrap_or(5432).to_string())
                .arg("--username")
                .arg(profile.username.as_deref().unwrap_or("postgres"))
                .arg(&profile.database_name)
                .arg("-t")
                .arg("-c")
                .arg("SELECT pg_database_size(current_database())")
                .output()
                .await
                .map_err(|e| DsmError::ConnectionError {
                    message: format!("Failed to estimate size: {e}"),
                })?;
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                stdout.parse::<u64>().unwrap_or(0)
            } else {
                0
            }
        }
        "sqlite" => {
            let path = std::path::Path::new(&profile.database_name);
            if path.exists() {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            }
        }
        _ => 0,
    };

    // Look up historical compression ratio from previous snapshots of this profile
    let historical_ratio = {
        let conn = lock_db(&db)?;
        let result: Option<f64> = conn
            .query_row(
                "SELECT AVG(CAST(size_bytes AS REAL) / NULLIF(
                    (SELECT SUM(data_length) FROM (SELECT 1 as data_length)), 0))
                 FROM snapshots WHERE profile_id = ?1 AND size_bytes > 0",
                params![profile_id],
                |row| row.get(0),
            )
            .ok()
            .flatten();
        result
    };

    // Default compression ratio: SQL text typically compresses ~5:1 with gzip
    let compression_ratio = historical_ratio.unwrap_or(0.2);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let estimated_compressed = (raw_bytes as f64 * compression_ratio) as u64;

    let estimation_method = if historical_ratio.is_some() {
        "historical".to_string()
    } else {
        "default".to_string()
    };

    Ok(SizeEstimation {
        estimated_raw_bytes: raw_bytes,
        estimated_compressed_bytes: if estimated_compressed > 0 {
            estimated_compressed
        } else {
            raw_bytes / 5
        },
        compression_ratio,
        estimation_method,
    })
}

// -- Tool Discovery Commands --------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredTool {
    pub name: String,
    pub path: Option<String>,
    pub version: Option<String>,
    pub found: bool,
    pub install_hint: String,
    pub min_version: String,
}

#[tauri::command]
pub async fn discover_tools() -> Result<Vec<DiscoveredTool>, DsmError> {
    let tools = vec![
        ("mysqldump", "brew install mysql-client", "8.0"),
        ("mysql", "brew install mysql-client", "8.0"),
        ("pg_dump", "brew install libpq", "14"),
        ("psql", "brew install libpq", "14"),
        ("sqlite3", "Bundled with macOS", "3.0"),
    ];

    let mut results = Vec::new();
    for (tool_name, install_hint, min_version) in tools {
        let tool_result = discover_single_tool(tool_name, install_hint, min_version).await;
        results.push(tool_result);
    }
    Ok(results)
}

async fn discover_single_tool(name: &str, install_hint: &str, min_version: &str) -> DiscoveredTool {
    match which::which(name) {
        Ok(path) => {
            let version = get_tool_version(name, &path).await;
            DiscoveredTool {
                name: name.to_string(),
                path: Some(path.to_string_lossy().to_string()),
                version,
                found: true,
                install_hint: install_hint.to_string(),
                min_version: min_version.to_string(),
            }
        }
        Err(_) => DiscoveredTool {
            name: name.to_string(),
            path: None,
            version: None,
            found: false,
            install_hint: install_hint.to_string(),
            min_version: min_version.to_string(),
        },
    }
}

async fn get_tool_version(name: &str, path: &std::path::Path) -> Option<String> {
    let version_flag = match name {
        "mysqldump" | "mysql" | "pg_dump" | "psql" => "--version",
        "sqlite3" => "-version",
        _ => return None,
    };
    let output = tokio::process::Command::new(path)
        .arg(version_flag)
        .output()
        .await
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Extract just the version number
        extract_version_number(&stdout)
    } else {
        None
    }
}

fn extract_version_number(output: &str) -> Option<String> {
    // Common patterns: "mysqldump  Ver 8.0.33", "pg_dump (PostgreSQL) 14.9", "3.39.5 2022-..."
    for word in output.split_whitespace() {
        if word.chars().next().is_some_and(|c| c.is_ascii_digit()) && word.contains('.') {
            // Trim trailing commas or parentheses
            let clean = word.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.');
            return Some(clean.to_string());
        }
    }
    None
}

/// Detect dump tool version for the current database type.
/// Returns `(tool_version, db_version_from_tool)` or `None`.
async fn detect_tool_version(db_type: &str) -> Option<(String, String)> {
    let (tool_name, version_flag) = match db_type {
        "mysql" => ("mysqldump", "--version"),
        "postgresql" => ("pg_dump", "--version"),
        "sqlite" => ("sqlite3", "-version"),
        _ => return None,
    };

    let path = which::which(tool_name).ok()?;
    let output = tokio::process::Command::new(&path)
        .arg(version_flag)
        .output()
        .await
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let version = extract_version_number(&stdout).unwrap_or_else(|| stdout.clone());
        Some((version.clone(), version))
    } else {
        None
    }
}

#[tauri::command]
pub async fn save_tool_path(
    db: State<'_, DbState>,
    tool_name: String,
    path: String,
) -> Result<(), DsmError> {
    let version = get_tool_version(&tool_name, std::path::Path::new(&path)).await;
    let now = chrono::Utc::now().to_rfc3339();
    let conn = lock_db(&db)?;
    conn.execute(
        "INSERT OR REPLACE INTO tool_paths (tool_name, path, version, discovered_at) VALUES (?1, ?2, ?3, ?4)",
        params![tool_name, path, version, now],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn get_setup_complete(db: State<'_, DbState>) -> Result<bool, DsmError> {
    let conn = lock_db(&db)?;
    let result: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'setup_complete'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "false".to_string());
    Ok(result == "true")
}

// -- Retention Policy Commands ------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub profile_id: String,
    pub max_count: Option<u32>,
    pub max_age_days: Option<u32>,
    pub max_size_bytes: Option<u64>,
}

#[tauri::command]
#[allow(clippy::cast_sign_loss)]
pub async fn retention_policy_get(
    db: State<'_, DbState>,
    profile_id: String,
) -> Result<Option<RetentionPolicy>, DsmError> {
    let conn = lock_db(&db)?;
    let result = conn.query_row(
        "SELECT profile_id, max_count, max_age_days, max_size_bytes FROM retention_policies WHERE profile_id = ?1",
        params![profile_id],
        |row| {
            Ok(RetentionPolicy {
                profile_id: row.get(0)?,
                max_count: row.get::<_, Option<i32>>(1)?.map(|v| v as u32),
                max_age_days: row.get::<_, Option<i32>>(2)?.map(|v| v as u32),
                max_size_bytes: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
            })
        },
    );
    match result {
        Ok(policy) => Ok(Some(policy)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DsmError::DatabaseError(e)),
    }
}

#[tauri::command]
#[allow(clippy::cast_possible_wrap)]
pub async fn retention_policy_set(
    db: State<'_, DbState>,
    policy: RetentionPolicy,
) -> Result<(), DsmError> {
    let conn = lock_db(&db)?;
    conn.execute(
        "INSERT OR REPLACE INTO retention_policies (profile_id, max_count, max_age_days, max_size_bytes)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            policy.profile_id,
            policy.max_count.map(|v| v as i32),
            policy.max_age_days.map(|v| v as i32),
            policy.max_size_bytes.map(|v| v as i64),
        ],
    )?;
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionEnforcementResult {
    pub deleted_count: u32,
    pub freed_bytes: u64,
    pub reasons: Vec<String>,
}

#[tauri::command]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub async fn retention_enforce(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    profile_id: String,
) -> Result<RetentionEnforcementResult, DsmError> {
    let policy = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT profile_id, max_count, max_age_days, max_size_bytes FROM retention_policies WHERE profile_id = ?1",
            params![profile_id],
            |row| {
                Ok(RetentionPolicy {
                    profile_id: row.get(0)?,
                    max_count: row.get::<_, Option<i32>>(1)?.map(|v| v as u32),
                    max_age_days: row.get::<_, Option<i32>>(2)?.map(|v| v as u32),
                    max_size_bytes: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                })
            },
        )
    };

    let policy = match policy {
        Ok(p) => p,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Ok(RetentionEnforcementResult {
                deleted_count: 0,
                freed_bytes: 0,
                reasons: vec![],
            });
        }
        Err(e) => return Err(DsmError::DatabaseError(e)),
    };

    let mut to_delete: Vec<(String, String, u64)> = Vec::new(); // (id, file_path, size_bytes)
    let mut reasons: Vec<String> = Vec::new();

    let conn = lock_db(&db)?;

    // Get all unpinned snapshots for this profile, ordered by creation date (oldest first)
    let mut stmt = conn.prepare(
        "SELECT id, file_path, size_bytes, created_at FROM snapshots
         WHERE profile_id = ?1 AND pinned = 0
         ORDER BY created_at ASC",
    )?;
    let rows: Vec<(String, String, i64, String)> = stmt
        .query_map(params![profile_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut to_keep: Vec<&(String, String, i64, String)> = rows.iter().collect();

    // Enforce max_count: keep only the newest N
    if let Some(max_count) = policy.max_count {
        let max = max_count as usize;
        if to_keep.len() > max {
            let excess = to_keep.len() - max;
            for row in to_keep.iter().take(excess) {
                to_delete.push((row.0.clone(), row.1.clone(), row.2 as u64));
            }
            reasons.push(format!(
                "Removed {excess} snapshot(s) exceeding max count of {max_count}"
            ));
            to_keep = to_keep[excess..].to_vec();
        }
    }

    // Enforce max_age_days
    if let Some(max_age) = policy.max_age_days {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(max_age));
        let cutoff_str = cutoff.to_rfc3339();
        let mut aged_count = 0u32;
        to_keep.retain(|row| {
            if row.3 < cutoff_str {
                to_delete.push((row.0.clone(), row.1.clone(), row.2 as u64));
                aged_count += 1;
                false
            } else {
                true
            }
        });
        if aged_count > 0 {
            reasons.push(format!(
                "Removed {aged_count} snapshot(s) older than {max_age} days"
            ));
        }
    }

    // Enforce max_size_bytes: delete oldest until total size is under limit
    if let Some(max_size) = policy.max_size_bytes {
        let mut total: u64 = to_keep.iter().map(|r| r.2 as u64).sum();
        let mut size_count = 0u32;
        while total > max_size && !to_keep.is_empty() {
            let oldest = to_keep.remove(0);
            total -= oldest.2 as u64;
            to_delete.push((oldest.0.clone(), oldest.1.clone(), oldest.2 as u64));
            size_count += 1;
        }
        if size_count > 0 {
            reasons.push(format!(
                "Removed {size_count} snapshot(s) to stay under size limit"
            ));
        }
    }

    // Deduplicate to_delete by id
    to_delete.sort_by(|a, b| a.0.cmp(&b.0));
    to_delete.dedup_by(|a, b| a.0 == b.0);

    #[allow(clippy::cast_possible_truncation)]
    let deleted_count = to_delete.len() as u32;
    let freed_bytes: u64 = to_delete.iter().map(|d| d.2).sum();

    // Delete snapshot files and records
    for (id, file_path, _) in &to_delete {
        let full_path = paths.snapshots_dir.join(file_path);
        if full_path.exists() {
            let _ = std::fs::remove_file(&full_path);
        }
        conn.execute("DELETE FROM snapshots WHERE id = ?1", params![id])?;
    }

    Ok(RetentionEnforcementResult {
        deleted_count,
        freed_bytes,
        reasons,
    })
}

// -- Schema Diff Commands -----------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDiff {
    pub snapshot_a_name: String,
    pub snapshot_b_name: String,
    pub tables_added: Vec<String>,
    pub tables_removed: Vec<String>,
    pub tables_modified: Vec<TableDiff>,
    pub summary: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableDiff {
    pub table_name: String,
    pub columns_added: Vec<String>,
    pub columns_removed: Vec<String>,
    pub columns_modified: Vec<String>,
}

#[tauri::command]
#[allow(clippy::similar_names)]
pub async fn snapshot_compare_schema(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    snapshot_a_id: String,
    snapshot_b_id: String,
) -> Result<SchemaDiff, DsmError> {
    let (snap_a, snap_b) = {
        let conn = lock_db(&db)?;
        let a = conn
            .query_row(
                "SELECT * FROM snapshots WHERE id = ?1",
                params![snapshot_a_id],
                row_to_snapshot,
            )
            .map_err(|_| DsmError::SnapshotNotFound(snapshot_a_id.clone()))?;
        let b = conn
            .query_row(
                "SELECT * FROM snapshots WHERE id = ?1",
                params![snapshot_b_id],
                row_to_snapshot,
            )
            .map_err(|_| DsmError::SnapshotNotFound(snapshot_b_id.clone()))?;
        (a, b)
    };

    // Extract schema from both snapshots by decompressing and parsing CREATE TABLE statements
    let schema_a =
        extract_schema_from_snapshot(&paths.snapshots_dir.join(&snap_a.file_path)).await?;
    let schema_b =
        extract_schema_from_snapshot(&paths.snapshots_dir.join(&snap_b.file_path)).await?;

    let tables_a: std::collections::HashSet<&str> = schema_a.keys().map(String::as_str).collect();
    let tables_b: std::collections::HashSet<&str> = schema_b.keys().map(String::as_str).collect();

    let tables_added: Vec<String> = tables_b
        .difference(&tables_a)
        .map(|s| (*s).to_string())
        .collect();
    let tables_removed: Vec<String> = tables_a
        .difference(&tables_b)
        .map(|s| (*s).to_string())
        .collect();

    let mut tables_modified = Vec::new();
    for table in tables_a.intersection(&tables_b) {
        let cols_a = &schema_a[*table];
        let cols_b = &schema_b[*table];
        let set_a: std::collections::HashSet<&str> = cols_a.iter().map(String::as_str).collect();
        let set_b: std::collections::HashSet<&str> = cols_b.iter().map(String::as_str).collect();

        let added: Vec<String> = set_b.difference(&set_a).map(|s| (*s).to_string()).collect();
        let removed: Vec<String> = set_a.difference(&set_b).map(|s| (*s).to_string()).collect();

        if !added.is_empty() || !removed.is_empty() {
            tables_modified.push(TableDiff {
                table_name: (*table).to_string(),
                columns_added: added,
                columns_removed: removed,
                columns_modified: Vec::new(),
            });
        }
    }

    let summary = format!(
        "{} table(s) added, {} removed, {} modified",
        tables_added.len(),
        tables_removed.len(),
        tables_modified.len()
    );

    Ok(SchemaDiff {
        snapshot_a_name: snap_a.name,
        snapshot_b_name: snap_b.name,
        tables_added,
        tables_removed,
        tables_modified,
        summary,
    })
}

/// Extract table-to-columns map from a compressed SQL snapshot by parsing CREATE TABLE statements.
async fn extract_schema_from_snapshot(
    gz_path: &std::path::Path,
) -> Result<HashMap<String, Vec<String>>, DsmError> {
    let gz = gz_path.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut tables: HashMap<String, Vec<String>> = HashMap::new();
        let mut current_table: Option<String> = None;
        let mut current_columns: Vec<String> = Vec::new();

        for line in crate::compress::decompressed_lines(
            &gz,
            MAX_SNAPSHOT_PARSE_BYTES,
            MAX_SNAPSHOT_LINE_BYTES,
        )? {
            let line = line?;
            let trimmed = line.trim();
            // Match CREATE TABLE patterns
            if let Some(name) = parse_create_table_line(trimmed) {
                current_table = Some(name);
                current_columns = Vec::new();
            } else if current_table.is_some() {
                if trimmed.starts_with(')') {
                    // End of CREATE TABLE
                    if let Some(table_name) = current_table.take() {
                        tables.insert(table_name, current_columns.clone());
                    }
                    current_columns.clear();
                } else if trimmed.starts_with('`')
                    || trimmed.starts_with('"')
                    || trimmed
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic())
                {
                    // Column definition line
                    let col = trimmed.trim_end_matches(',').to_string();
                    if !col.to_uppercase().starts_with("PRIMARY KEY")
                        && !col.to_uppercase().starts_with("KEY ")
                        && !col.to_uppercase().starts_with("INDEX ")
                        && !col.to_uppercase().starts_with("UNIQUE ")
                        && !col.to_uppercase().starts_with("CONSTRAINT ")
                        && !col.to_uppercase().starts_with("FOREIGN KEY")
                    {
                        current_columns.push(col);
                    }
                }
            }
        }

        Ok(tables)
    })
    .await
    .map_err(|e| {
        DsmError::FileSystemError(std::io::Error::other(format!(
            "Schema extraction failed: {e}"
        )))
    })?
}

/// Parse a CREATE TABLE line and extract the table name.
fn parse_create_table_line(line: &str) -> Option<String> {
    let upper = line.to_uppercase();
    if !upper.starts_with("CREATE TABLE") {
        return None;
    }
    // Skip "CREATE TABLE IF NOT EXISTS" or "CREATE TABLE"
    let rest = if upper.contains("IF NOT EXISTS") {
        line.splitn(6, ' ').nth(5)?
    } else {
        line.splitn(3, ' ').nth(2)?
    };
    let name = rest
        .trim_start_matches('`')
        .trim_start_matches('"')
        .split(['`', '"', '(', ' '])
        .next()?
        .trim_end_matches('.')
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

// -- Dry-Run Preview Command --------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreview {
    pub snapshot_name: String,
    pub snapshot_tables: Vec<String>,
    pub current_tables: Vec<String>,
    pub tables_to_add: Vec<String>,
    pub tables_to_remove: Vec<String>,
    pub tables_in_common: Vec<String>,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub async fn snapshot_restore_preview(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    snapshot_id: String,
) -> Result<RestorePreview, DsmError> {
    // Load snapshot and profile
    let (snapshot, profile) = {
        let conn = lock_db(&db)?;
        let snap = conn
            .query_row(
                "SELECT * FROM snapshots WHERE id = ?1",
                params![snapshot_id],
                row_to_snapshot,
            )
            .map_err(|_| DsmError::SnapshotNotFound(snapshot_id.clone()))?;
        let prof = conn
            .query_row(
                "SELECT * FROM profiles WHERE id = ?1",
                params![snap.profile_id],
                row_to_profile,
            )
            .map_err(|_| DsmError::ProfileNotFound(snap.profile_id.clone()))?;
        (snap, prof)
    };

    let creds = credentials::get_credentials(&profile.id)?;

    // Extract tables from snapshot
    let schema =
        extract_schema_from_snapshot(&paths.snapshots_dir.join(&snapshot.file_path)).await?;
    let snapshot_tables: Vec<String> = schema.keys().cloned().collect();

    // Get current database tables
    let current_tables = get_current_tables(&profile, &creds, &paths)
        .await
        .unwrap_or_default();

    let snap_set: std::collections::HashSet<&str> =
        snapshot_tables.iter().map(String::as_str).collect();
    let curr_set: std::collections::HashSet<&str> =
        current_tables.iter().map(String::as_str).collect();

    let tables_to_add: Vec<String> = snap_set
        .difference(&curr_set)
        .map(|s| (*s).to_string())
        .collect();
    let tables_to_remove: Vec<String> = curr_set
        .difference(&snap_set)
        .map(|s| (*s).to_string())
        .collect();
    let tables_in_common: Vec<String> = snap_set
        .intersection(&curr_set)
        .map(|s| (*s).to_string())
        .collect();

    let mut warnings = Vec::new();
    if !tables_to_remove.is_empty() {
        warnings.push(format!(
            "Restoring this snapshot will drop {} table(s) that exist in the current database: {}",
            tables_to_remove.len(),
            tables_to_remove.join(", ")
        ));
    }

    Ok(RestorePreview {
        snapshot_name: snapshot.name,
        snapshot_tables,
        current_tables,
        tables_to_add,
        tables_to_remove,
        tables_in_common,
        warnings,
    })
}

/// Get current table list from a live database.
async fn get_current_tables(
    profile: &Profile,
    creds: &ProfileCredentials,
    paths: &AppPaths,
) -> Result<Vec<String>, DsmError> {
    match profile.db_type.as_str() {
        "mysql" => {
            let df = crate::dump::write_mysql_defaults_file(
                &paths.tmp_dir,
                profile.host.as_deref().unwrap_or("127.0.0.1"),
                profile.port.unwrap_or(3306),
                profile.username.as_deref().unwrap_or("root"),
                creds.password.as_deref().unwrap_or(""),
            )?;
            let output = tokio::process::Command::new("mysql")
                .arg(format!("--defaults-extra-file={}", df.display()))
                .arg(&profile.database_name)
                .arg("-e")
                .arg("SHOW TABLES")
                .arg("--skip-column-names")
                .output()
                .await
                .map_err(|e| DsmError::ConnectionError {
                    message: format!("Failed to list tables: {e}"),
                })?;
            let _ = std::fs::remove_file(&df);
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect())
            } else {
                Ok(Vec::new())
            }
        }
        "postgresql" => {
            let output = tokio::process::Command::new("psql")
                .env("PGPASSWORD", creds.password.as_deref().unwrap_or(""))
                .arg("--host")
                .arg(profile.host.as_deref().unwrap_or("127.0.0.1"))
                .arg("--port")
                .arg(profile.port.unwrap_or(5432).to_string())
                .arg("--username")
                .arg(profile.username.as_deref().unwrap_or("postgres"))
                .arg(&profile.database_name)
                .arg("-t")
                .arg("-c")
                .arg("SELECT tablename FROM pg_tables WHERE schemaname = 'public'")
                .output()
                .await
                .map_err(|e| DsmError::ConnectionError {
                    message: format!("Failed to list tables: {e}"),
                })?;
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect())
            } else {
                Ok(Vec::new())
            }
        }
        "sqlite" => {
            let path = std::path::Path::new(&profile.database_name);
            if path.exists() {
                let conn = rusqlite::Connection::open_with_flags(
                    path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                )?;
                let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")?;
                let tables = stmt
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<Vec<String>, _>>()?;
                Ok(tables)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

// -- Version Compatibility Command --------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionCompatibility {
    pub compatible: bool,
    pub snapshot_tool_version: Option<String>,
    pub snapshot_db_version: Option<String>,
    pub current_tool_version: Option<String>,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub async fn snapshot_check_version_compatibility(
    db: State<'_, DbState>,
    snapshot_id: String,
) -> Result<VersionCompatibility, DsmError> {
    let snapshot = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT * FROM snapshots WHERE id = ?1",
            params![snapshot_id],
            row_to_snapshot,
        )
        .map_err(|_| DsmError::SnapshotNotFound(snapshot_id.clone()))?
    };

    // Get profile to determine db type
    let profile = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT * FROM profiles WHERE id = ?1",
            params![snapshot.profile_id],
            row_to_profile,
        )
        .map_err(|_| DsmError::ProfileNotFound(snapshot.profile_id.clone()))?
    };

    // Get current tool version
    let current_version = detect_tool_version(&profile.db_type).await;

    let mut warnings = Vec::new();
    let mut compatible = true;

    if let (Some(ref snap_ver), Some(ref curr_ver)) =
        (&snapshot.dump_tool_version, &current_version)
    {
        let snap_major = snap_ver.split('.').next().unwrap_or("0");
        let curr_major = curr_ver.0.split('.').next().unwrap_or("0");
        if snap_major != curr_major {
            compatible = false;
            warnings.push(format!(
                "Major version mismatch: snapshot was created with version {}, current tool is version {}. Cross-major-version restores may produce errors or data loss.",
                snap_ver, curr_ver.0
            ));
        }
    }

    if snapshot.dump_tool_version.is_none() {
        warnings.push("No tool version was recorded when this snapshot was created. Version compatibility cannot be verified.".to_string());
    }

    Ok(VersionCompatibility {
        compatible,
        snapshot_tool_version: snapshot.dump_tool_version,
        snapshot_db_version: snapshot.db_version,
        current_tool_version: current_version.map(|v| v.0),
        warnings,
    })
}

// -- Export SQL Command -------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub output_path: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub async fn snapshot_export_sql(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    snapshot_id: String,
    output_dir: String,
) -> Result<ExportResult, DsmError> {
    let snapshot = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT * FROM snapshots WHERE id = ?1",
            params![snapshot_id],
            row_to_snapshot,
        )
        .map_err(|_| DsmError::SnapshotNotFound(snapshot_id.clone()))?
    };

    // Get profile for metadata header
    let profile = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT * FROM profiles WHERE id = ?1",
            params![snapshot.profile_id],
            row_to_profile,
        )
        .ok()
    };

    let gz_path = paths.snapshots_dir.join(&snapshot.file_path);
    if !gz_path.exists() {
        return Err(DsmError::RestoreError {
            message: "Snapshot file is missing from disk.".to_string(),
            output: String::new(),
        });
    }

    // Build output filename: dbname-date-snapshotname.sql
    let date_str = snapshot
        .created_at
        .split('T')
        .next()
        .unwrap_or("unknown-date");
    let safe_name = snapshot
        .name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    let db_name = profile
        .as_ref()
        .map_or("database", |p| p.database_name.as_str());
    let output_filename = format!("{db_name}-{date_str}-{safe_name}.sql");
    let output_path = std::path::Path::new(&output_dir).join(&output_filename);

    let gz = gz_path.clone();
    let out = output_path.clone();
    let snap_meta = format!(
        "-- Amber Snapshot Export\n-- Database: {}\n-- Captured: {}\n-- Snapshot: {} ({})\n-- Tool version: {}\n\n",
        db_name,
        snapshot.created_at,
        snapshot.name,
        snapshot.id,
        snapshot.dump_tool_version.as_deref().unwrap_or("unknown"),
    );

    let size = tokio::task::spawn_blocking(move || -> Result<u64, std::io::Error> {
        let mut out_file = std::fs::File::create(&out)?;
        // Write metadata header
        std::io::Write::write_all(&mut out_file, snap_meta.as_bytes())?;
        // Decompress and write SQL content
        crate::compress::decompress_to_writer(&gz, &mut out_file)?;
        Ok(std::fs::metadata(&out)?.len())
    })
    .await
    .map_err(|e| {
        DsmError::FileSystemError(std::io::Error::other(format!("Export task failed: {e}")))
    })?
    .map_err(DsmError::FileSystemError)?;

    Ok(ExportResult {
        output_path: output_path.to_string_lossy().to_string(),
        size_bytes: size,
    })
}

// -- Streaming Progress Commands (added for progress tracking) ----------------

/// Update the progress phase for streaming dump operations.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingDumpProgress {
    pub current_table: String,
    pub tables_completed: u32,
    pub total_tables: u32,
    pub bytes_processed: u64,
}

// -- Health Monitoring ---------------------------------------------------------

/// Result of a periodic health check — lightweight version of `ConnectionTestResult`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResult {
    pub profile_id: String,
    pub status: String, // "connected", "unreachable", "unchecked"
    pub message: String,
    pub latency_ms: u64,
    pub checked_at: String,
}

/// Lightweight health check for a profile. Reuses the same connection test
/// logic as `profile_test_connection` but returns a simpler result suitable
/// for periodic polling.
#[tauri::command]
pub async fn profile_health_check(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    id: String,
) -> Result<HealthCheckResult, DsmError> {
    let result = profile_test_connection(db, paths, id.clone()).await?;
    let now = chrono::Utc::now().to_rfc3339();
    Ok(HealthCheckResult {
        profile_id: id,
        status: if result.success {
            "connected".to_string()
        } else {
            "unreachable".to_string()
        },
        message: result.message,
        latency_ms: result.latency_ms,
        checked_at: now,
    })
}

// -- Disk Space Pre-flight Check ----------------------------------------------

/// Information about available disk space on the snapshots volume.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpaceInfo {
    pub available_bytes: u64,
    pub estimated_bytes: u64,
    pub sufficient: bool,
    pub safety_margin: f64,
    pub message: String,
}

/// Check whether there is sufficient disk space for a new snapshot.
///
/// Compares the estimated snapshot size (with a configurable safety margin,
/// default 2x) against available space on the volume hosting the snapshots
/// directory.
#[tauri::command]
pub async fn check_disk_space(
    paths: State<'_, AppPaths>,
    estimated_bytes: u64,
    safety_margin: Option<f64>,
) -> Result<DiskSpaceInfo, DsmError> {
    let margin = safety_margin.unwrap_or(2.0);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let required = (estimated_bytes as f64 * margin) as u64;
    let snapshots_dir = paths.snapshots_dir.clone();

    let available = tokio::task::spawn_blocking(move || -> Result<u64, std::io::Error> {
        get_available_disk_space(&snapshots_dir)
    })
    .await
    .map_err(|e| {
        DsmError::FileSystemError(std::io::Error::other(format!("Disk check failed: {e}")))
    })?
    .map_err(DsmError::FileSystemError)?;

    let sufficient = available >= required;
    let message = if sufficient {
        format!(
            "{} available ({} required with {}x safety margin)",
            format_bytes_short(available),
            format_bytes_short(required),
            margin,
        )
    } else {
        format!(
            "Only {} available but {} required ({}x safety margin). Free up {} to proceed.",
            format_bytes_short(available),
            format_bytes_short(required),
            margin,
            format_bytes_short(required.saturating_sub(available)),
        )
    };

    Ok(DiskSpaceInfo {
        available_bytes: available,
        estimated_bytes,
        sufficient,
        safety_margin: margin,
        message,
    })
}

/// Get available disk space on the volume containing the given path.
#[cfg(target_os = "macos")]
fn get_available_disk_space(path: &std::path::Path) -> Result<u64, std::io::Error> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path = CString::new(path.to_string_lossy().as_bytes())
        .map_err(|e| std::io::Error::other(format!("Invalid path: {e}")))?;

    unsafe {
        let mut stat = MaybeUninit::<libc::statfs>::uninit();
        if libc::statfs(c_path.as_ptr(), stat.as_mut_ptr()) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let stat = stat.assume_init();
        #[allow(clippy::cast_sign_loss)]
        Ok(stat.f_bavail * u64::from(stat.f_bsize))
    }
}

#[cfg(not(target_os = "macos"))]
fn get_available_disk_space(_path: &std::path::Path) -> Result<u64, std::io::Error> {
    // Fallback: report a large value so the check never blocks
    Ok(u64::MAX)
}

// -- Concurrent Operation Guard -----------------------------------------------

/// Status of any in-progress operation on a profile.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationStatusResult {
    pub profile_id: String,
    pub busy: bool,
    pub operation: Option<String>,
}

/// Check whether a profile has an operation in progress.
#[tauri::command]
pub async fn operation_status(
    locks: State<'_, crate::db::OperationLocks>,
    profile_id: String,
) -> Result<OperationStatusResult, DsmError> {
    let map = locks.0.lock().map_err(|e| DsmError::ConnectionError {
        message: format!("Failed to check operation status: {e}"),
    })?;

    if let Some(slot) = map.get(&profile_id) {
        let busy = slot.busy.load(std::sync::atomic::Ordering::Acquire);
        let operation = if busy {
            slot.operation.lock().ok().map(|op| op.clone())
        } else {
            None
        };
        Ok(OperationStatusResult {
            profile_id,
            busy,
            operation,
        })
    } else {
        Ok(OperationStatusResult {
            profile_id,
            busy: false,
            operation: None,
        })
    }
}

// -- Orphaned Snapshot Cleanup ------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrphanedFile {
    pub path: String,
    pub size_bytes: u64,
    pub created_at: Option<String>,
    pub inferred_db_type: Option<String>,
}

/// Scan the snapshots directory for files not referenced by any snapshot
/// metadata record. Returns a list of orphaned files with size and metadata.
#[tauri::command]
pub async fn scan_orphaned_snapshots(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
) -> Result<Vec<OrphanedFile>, DsmError> {
    // Collect all known file_path values from the database
    let known_paths: std::collections::HashSet<String> = {
        let conn = lock_db(&db)?;
        let mut stmt = conn.prepare("SELECT file_path FROM snapshots")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().collect()
    };

    let snapshots_dir = paths.snapshots_dir.clone();
    tokio::task::spawn_blocking(move || -> Result<Vec<OrphanedFile>, DsmError> {
        let mut orphans = Vec::new();

        // Walk all project subdirectories
        let Ok(entries) = std::fs::read_dir(&snapshots_dir) else {
            return Ok(orphans);
        };

        for project_entry in entries.flatten() {
            let project_path = project_entry.path();
            if !project_path.is_dir() {
                continue;
            }

            let project_name = project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let Ok(files) = std::fs::read_dir(&project_path) else {
                continue;
            };

            for file_entry in files.flatten() {
                let file_path = file_entry.path();
                if !file_path.is_file() {
                    continue;
                }

                // Only consider .sql.gz and .db.gz files (snapshot formats)
                let file_name = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if !file_name.ends_with(".sql.gz") && !file_name.ends_with(".db.gz") {
                    continue;
                }

                // Build the relative path as stored in the database
                let relative_path = format!("{project_name}/{file_name}");

                if !known_paths.contains(&relative_path) {
                    let metadata = std::fs::metadata(&file_path).ok();
                    let size_bytes = metadata.as_ref().map_or(0, std::fs::Metadata::len);
                    let created_at = metadata
                        .as_ref()
                        .and_then(|m| m.created().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

                    // Infer database type from file extension
                    let inferred_db_type = if file_name.ends_with(".db.gz") {
                        Some("sqlite".to_string())
                    } else {
                        // Could be mysql or postgresql — can't determine without parsing
                        None
                    };

                    orphans.push(OrphanedFile {
                        path: relative_path,
                        size_bytes,
                        created_at,
                        inferred_db_type,
                    });
                }
            }
        }

        Ok(orphans)
    })
    .await
    .map_err(|e| DsmError::ConnectionError {
        message: format!("Orphan scan task failed: {e}"),
    })?
}

/// Delete specific orphaned snapshot files from disk.
#[tauri::command]
pub async fn delete_orphaned_snapshots(
    paths: State<'_, AppPaths>,
    file_paths: Vec<String>,
) -> Result<u32, DsmError> {
    let snapshots_dir = paths.snapshots_dir.clone();
    let mut deleted = 0u32;

    for relative_path in &file_paths {
        let full_path = snapshots_dir.join(relative_path);

        // Safety: only delete files within the snapshots directory
        if !full_path.starts_with(&snapshots_dir) {
            continue;
        }

        if full_path.is_file() {
            let _ = std::fs::remove_file(&full_path);
            deleted += 1;
        }
    }

    // Clean up empty project directories
    if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(mut contents) = std::fs::read_dir(&path) {
                    if contents.next().is_none() {
                        let _ = std::fs::remove_dir(&path);
                    }
                }
            }
        }
    }

    Ok(deleted)
}

/// Acquire a per-profile operation lock. Returns a guard that clears the busy
/// flag when dropped.
fn acquire_profile_lock(
    locks: &crate::db::OperationLocks,
    profile_id: &str,
    operation: &str,
) -> Result<crate::db::ProfileOpGuard, DsmError> {
    use std::sync::atomic::Ordering;

    let mut map = locks.0.lock().map_err(|e| DsmError::ConnectionError {
        message: format!("Failed to acquire operation registry: {e}"),
    })?;

    let slot = map
        .entry(profile_id.to_string())
        .or_insert_with(|| {
            std::sync::Arc::new(crate::db::OperationSlot {
                busy: std::sync::atomic::AtomicBool::new(false),
                operation: std::sync::Mutex::new(String::new()),
            })
        })
        .clone();

    // Drop the map lock — we only hold the slot Arc now
    drop(map);

    // Try to set busy from false → true atomically
    if slot
        .busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        if let Ok(mut op) = slot.operation.lock() {
            *op = operation.to_string();
        }
        Ok(crate::db::ProfileOpGuard { slot })
    } else {
        Err(DsmError::OperationInProgress {
            operation: "Another operation is already running on this profile".to_string(),
        })
    }
}

// -- Schedule Config Commands -------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub profile_id: String,
    pub interval: String,
    pub last_snapshot_at: Option<String>,
    pub next_due_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Compute the next due timestamp from last snapshot time and interval.
pub fn compute_next_due(last_at: &str, interval: &str) -> Option<String> {
    let last = chrono::DateTime::parse_from_rfc3339(last_at).ok()?;
    let duration = match interval {
        "hourly" => chrono::Duration::hours(1),
        "every_6h" => chrono::Duration::hours(6),
        "daily" => chrono::Duration::days(1),
        "weekly" => chrono::Duration::weeks(1),
        _ => return None,
    };
    Some((last + duration).to_rfc3339())
}

#[tauri::command]
pub async fn schedule_config_get(
    db: State<'_, DbState>,
    profile_id: String,
) -> Result<Option<ScheduleConfig>, DsmError> {
    let conn = lock_db(&db)?;
    let result = conn.query_row(
        "SELECT profile_id, interval, last_snapshot_at, next_due_at, created_at, updated_at
         FROM schedule_configs WHERE profile_id = ?1",
        params![profile_id],
        |row| {
            Ok(ScheduleConfig {
                profile_id: row.get(0)?,
                interval: row.get(1)?,
                last_snapshot_at: row.get(2)?,
                next_due_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    );
    match result {
        Ok(config) => Ok(Some(config)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DsmError::DatabaseError(e)),
    }
}

#[tauri::command]
pub async fn schedule_config_set(
    db: State<'_, DbState>,
    profile_id: String,
    interval: String,
) -> Result<ScheduleConfig, DsmError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = lock_db(&db)?;

    // Check if config already exists to preserve last_snapshot_at
    let existing_last: Option<String> = conn
        .query_row(
            "SELECT last_snapshot_at FROM schedule_configs WHERE profile_id = ?1",
            params![profile_id],
            |row| row.get(0),
        )
        .ok();

    let next_due = existing_last
        .as_ref()
        .and_then(|last| compute_next_due(last, &interval));

    conn.execute(
        "INSERT INTO schedule_configs (profile_id, interval, last_snapshot_at, next_due_at, created_at, updated_at)
         VALUES (?1, ?2, (SELECT last_snapshot_at FROM schedule_configs WHERE profile_id = ?1), ?3, ?4, ?5)
         ON CONFLICT(profile_id) DO UPDATE SET
            interval = excluded.interval,
            next_due_at = excluded.next_due_at,
            updated_at = excluded.updated_at",
        params![profile_id, interval, next_due, now, now],
    )?;

    Ok(ScheduleConfig {
        profile_id,
        interval,
        last_snapshot_at: existing_last,
        next_due_at: next_due,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn schedule_config_list(db: State<'_, DbState>) -> Result<Vec<ScheduleConfig>, DsmError> {
    let conn = lock_db(&db)?;
    let mut stmt = conn.prepare(
        "SELECT profile_id, interval, last_snapshot_at, next_due_at, created_at, updated_at
         FROM schedule_configs",
    )?;
    let configs = stmt
        .query_map([], |row| {
            Ok(ScheduleConfig {
                profile_id: row.get(0)?,
                interval: row.get(1)?,
                last_snapshot_at: row.get(2)?,
                next_due_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(configs)
}

// -- Snapshot Content Browser --------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotContentTable {
    pub table_name: String,
    pub columns: Vec<String>,
    pub row_count: usize,
    pub sample_rows: Vec<Vec<Option<String>>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotContent {
    pub snapshot_name: String,
    pub db_type: String,
    pub tables: Vec<SnapshotContentTable>,
    pub total_rows: usize,
}

const MAX_SAMPLE_ROWS: usize = 50;
const MAX_CELL_LENGTH: usize = 200;
const MAX_SNAPSHOT_PARSE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_SNAPSHOT_LINE_BYTES: usize = 16 * 1024 * 1024;

/// Truncate a string value for display, appending an ellipsis if truncated.
fn truncate_cell(value: &str) -> String {
    match value.char_indices().nth(MAX_CELL_LENGTH) {
        Some((end, _)) => format!("{}…", &value[..end]),
        None => value.to_string(),
    }
}

/// Parse `MySQL` INSERT VALUES rows from a single INSERT statement line.
/// Handles quoted strings (with escaped quotes), NULL, and numeric values.
fn parse_mysql_insert_values(values_part: &str) -> Vec<Vec<Option<String>>> {
    let mut rows = Vec::new();
    let mut current_row: Vec<Option<String>> = Vec::new();
    let mut current_value = String::new();
    let mut current_value_quoted = false;
    let mut in_string = false;
    let mut escape_next = false;
    let mut depth = 0;
    let chars: Vec<char> = values_part.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if escape_next {
            current_value.push(ch);
            escape_next = false;
            i += 1;
            continue;
        }

        if in_string {
            if ch == '\\' {
                current_value.push(ch);
                escape_next = true;
            } else if ch == '\'' {
                // Check for escaped quote ''
                if i + 1 < len && chars[i + 1] == '\'' {
                    current_value.push('\'');
                    i += 1;
                } else {
                    in_string = false;
                }
            } else {
                current_value.push(ch);
            }
        } else {
            match ch {
                '(' => {
                    depth += 1;
                    if depth == 1 {
                        current_row = Vec::new();
                        current_value = String::new();
                        current_value_quoted = false;
                    } else {
                        current_value.push(ch);
                    }
                }
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        let trimmed = current_value.trim();
                        if current_value_quoted || !trimmed.is_empty() || !current_row.is_empty() {
                            current_row.push(parse_mysql_cell(trimmed, current_value_quoted));
                        }
                        rows.push(std::mem::take(&mut current_row));
                        current_value = String::new();
                        current_value_quoted = false;
                    } else {
                        current_value.push(ch);
                    }
                }
                ',' if depth == 1 => {
                    current_row.push(parse_mysql_cell(current_value.trim(), current_value_quoted));
                    current_value = String::new();
                    current_value_quoted = false;
                }
                '\'' if depth >= 1 => {
                    in_string = true;
                    current_value_quoted = true;
                }
                _ => {
                    if depth >= 1 {
                        current_value.push(ch);
                    }
                }
            }
        }

        i += 1;
    }

    rows
}

fn parse_mysql_cell(value: &str, quoted: bool) -> Option<String> {
    if !quoted && value.eq_ignore_ascii_case("NULL") {
        None
    } else {
        Some(truncate_cell(value))
    }
}

/// Extract the table name from an INSERT INTO statement.
fn parse_insert_table_name(line: &str) -> Option<String> {
    // INSERT INTO `table` or INSERT INTO "table" or INSERT INTO table
    let upper = line.to_uppercase();
    if !upper.starts_with("INSERT INTO") {
        return None;
    }
    let rest = line.get(12..)?.trim();
    let name = rest
        .trim_start_matches('`')
        .trim_start_matches('"')
        .split(['`', '"', ' ', '('])
        .next()?
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Extract table name and column list from a `PostgreSQL` COPY ... FROM stdin statement.
fn parse_pg_copy_line(line: &str) -> Option<(String, Vec<String>)> {
    let upper = line.to_uppercase();
    if !upper.starts_with("COPY ") || !upper.contains("FROM STDIN") {
        return None;
    }
    // COPY table_name (col1, col2, ...) FROM stdin;
    let rest = line.get(5..)?.trim();
    let table_end = rest.find([' ', '('])?;
    let table_name = rest[..table_end].trim().to_string();

    let mut columns = Vec::new();
    if let Some(paren_start) = rest.find('(') {
        if let Some(paren_end) = rest.find(')') {
            let cols_str = &rest[paren_start + 1..paren_end];
            columns = cols_str
                .split(',')
                .map(|c| c.trim().trim_matches('"').to_string())
                .collect();
        }
    }

    Some((table_name, columns))
}

#[tauri::command]
#[allow(clippy::too_many_lines)]
pub async fn snapshot_browse_content(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    snapshot_id: String,
) -> Result<SnapshotContent, DsmError> {
    let (snap, profile_db_type) = {
        let conn = lock_db(&db)?;
        let snapshot = conn
            .query_row(
                "SELECT * FROM snapshots WHERE id = ?1",
                params![snapshot_id],
                row_to_snapshot,
            )
            .map_err(|_| DsmError::SnapshotNotFound(snapshot_id.clone()))?;
        let db_type: String = conn
            .query_row(
                "SELECT db_type FROM profiles WHERE id = ?1",
                params![snapshot.profile_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "mysql".to_string());
        (snapshot, db_type)
    };

    let gz_path = paths.snapshots_dir.join(&snap.file_path);
    let snapshot_name = snap.name.clone();
    let db_type = profile_db_type.clone();

    let tables = tokio::task::spawn_blocking(move || {
        // Phase 1: Extract schema (table → columns) using existing pattern
        let mut schema: HashMap<String, Vec<String>> = HashMap::new();
        let mut current_table: Option<String> = None;
        let mut current_columns: Vec<String> = Vec::new();

        // Phase 2: Track row data per table
        let mut row_counts: HashMap<String, usize> = HashMap::new();
        let mut sample_rows: HashMap<String, Vec<Vec<Option<String>>>> = HashMap::new();

        // PostgreSQL COPY state
        let mut in_pg_copy = false;
        let mut pg_copy_table = String::new();

        for line in crate::compress::decompressed_lines(
            &gz_path,
            MAX_SNAPSHOT_PARSE_BYTES,
            MAX_SNAPSHOT_LINE_BYTES,
        )? {
            let line = line?;
            let trimmed = line.trim();

            // Skip empty lines and comments (except structured markers)
            if trimmed.is_empty() {
                continue;
            }

            // --- Schema extraction (CREATE TABLE) ---
            if let Some(name) = parse_create_table_line(trimmed) {
                current_table = Some(name);
                current_columns = Vec::new();
                continue;
            }

            if current_table.is_some() {
                if trimmed.starts_with(')') {
                    if let Some(table_name) = current_table.take() {
                        let col_names: Vec<String> = current_columns
                            .iter()
                            .filter_map(|col_def| {
                                let name = col_def
                                    .trim_start_matches('`')
                                    .trim_start_matches('"')
                                    .split(['`', '"', ' '])
                                    .next()?;
                                if name.is_empty() {
                                    None
                                } else {
                                    Some(name.to_string())
                                }
                            })
                            .collect();
                        schema.insert(table_name, col_names);
                    }
                    current_columns.clear();
                    continue;
                }

                if trimmed.starts_with('`')
                    || trimmed.starts_with('"')
                    || trimmed
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic())
                {
                    let col = trimmed.trim_end_matches(',').to_string();
                    let upper = col.to_uppercase();
                    if !upper.starts_with("PRIMARY KEY")
                        && !upper.starts_with("KEY ")
                        && !upper.starts_with("INDEX ")
                        && !upper.starts_with("UNIQUE ")
                        && !upper.starts_with("CONSTRAINT ")
                        && !upper.starts_with("FOREIGN KEY")
                    {
                        current_columns.push(col);
                    }
                }
                continue;
            }

            // --- PostgreSQL COPY data ---
            if in_pg_copy {
                if trimmed == "\\." {
                    in_pg_copy = false;
                    continue;
                }
                let count = row_counts.entry(pg_copy_table.clone()).or_insert(0);
                *count += 1;

                let table_samples = sample_rows.entry(pg_copy_table.clone()).or_default();
                if table_samples.len() < MAX_SAMPLE_ROWS {
                    let row: Vec<Option<String>> = line
                        .split('\t')
                        .map(|val| (val != "\\N").then(|| truncate_cell(val)))
                        .collect();
                    table_samples.push(row);
                }
                continue;
            }

            // --- PostgreSQL COPY ... FROM stdin ---
            if db_type == "postgresql" {
                if let Some((table, columns)) = parse_pg_copy_line(trimmed) {
                    in_pg_copy = true;
                    pg_copy_table.clone_from(&table);
                    // If schema doesn't have columns from CREATE TABLE, use COPY columns
                    if !columns.is_empty() {
                        schema.entry(table).or_insert(columns);
                    }
                    continue;
                }
            }

            // --- MySQL INSERT INTO ---
            if let Some(table_name) = parse_insert_table_name(trimmed) {
                // Find VALUES keyword
                let upper = trimmed.to_uppercase();
                if let Some(values_pos) = upper.find("VALUES") {
                    let values_part = &trimmed[values_pos + 6..];
                    let parsed = parse_mysql_insert_values(values_part);
                    let count = row_counts.entry(table_name.clone()).or_insert(0);
                    let table_samples = sample_rows.entry(table_name.clone()).or_default();

                    for row in parsed {
                        *count += 1;
                        if table_samples.len() < MAX_SAMPLE_ROWS {
                            table_samples.push(row);
                        }
                    }
                }
            }
        }

        // Build the result: one entry per table found in schema
        let mut tables: Vec<SnapshotContentTable> = schema
            .into_iter()
            .map(|(table_name, columns)| {
                let row_count = row_counts.get(&table_name).copied().unwrap_or(0);
                let rows = sample_rows.remove(&table_name).unwrap_or_default();
                SnapshotContentTable {
                    table_name,
                    columns,
                    row_count,
                    sample_rows: rows,
                }
            })
            .collect();

        // Sort tables alphabetically
        tables.sort_by(|a, b| a.table_name.cmp(&b.table_name));

        Ok::<Vec<SnapshotContentTable>, DsmError>(tables)
    })
    .await
    .map_err(|e| {
        DsmError::FileSystemError(std::io::Error::other(format!(
            "Content extraction failed: {e}"
        )))
    })??;

    let total_rows = tables.iter().map(|t| t.row_count).sum();

    Ok(SnapshotContent {
        snapshot_name,
        db_type: profile_db_type,
        tables,
        total_rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn snapshot_content_parsers_handle_supported_dump_formats_and_unicode() {
        let rows = parse_mysql_insert_values(&format!("(1,'{}',NULL,'NULL','')", "é".repeat(201)));
        assert_eq!(rows[0][0].as_deref(), Some("1"));
        assert_eq!(
            rows[0][1].as_deref(),
            Some(format!("{}…", "é".repeat(200)).as_str())
        );
        assert_eq!(rows[0][2], None);
        assert_eq!(rows[0][3].as_deref(), Some("NULL"));
        assert_eq!(rows[0][4].as_deref(), Some(""));

        let (table, columns) =
            parse_pg_copy_line("COPY public.users (id, name) FROM stdin;").unwrap();
        assert_eq!(table, "public.users");
        assert_eq!(columns, ["id", "name"]);
    }

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

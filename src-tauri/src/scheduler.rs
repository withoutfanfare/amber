use crate::commands::{compute_next_due, ScheduleConfig};
use crate::db::{AppPaths, DbState, OperationLocks};
use crate::error::DsmError;
use rusqlite::params;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::Manager;

/// Check interval for the scheduler (60 seconds).
const CHECK_INTERVAL_SECS: u64 = 60;

/// Start the background scheduler that creates snapshots on configured intervals.
pub fn start(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Wait 30 seconds after app launch before first check, to let the UI settle
        tokio::time::sleep(Duration::from_secs(30)).await;

        let mut interval = tokio::time::interval(Duration::from_secs(CHECK_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = check_and_run(&app).await {
                eprintln!("[scheduler] Error: {e}");
            }
        }
    });
}

/// Check all enabled schedule configs and create snapshots for any that are due.
async fn check_and_run(app: &tauri::AppHandle) -> Result<(), String> {
    let db = app.state::<DbState>();
    let configs = {
        let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
        let mut stmt = conn
            .prepare(
                "SELECT profile_id, interval, last_snapshot_at, next_due_at, created_at, updated_at
                 FROM schedule_configs WHERE interval != 'disabled'",
            )
            .map_err(|e| format!("Query failed: {e}"))?;
        let result = stmt
            .query_map([], |row| {
                Ok(ScheduleConfig {
                    profile_id: row.get(0)?,
                    interval: row.get(1)?,
                    last_snapshot_at: row.get(2)?,
                    next_due_at: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Query map failed: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Collect failed: {e}"))?;
        result
    };

    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();

    for config in configs {
        if is_due(&config, &now_str) {
            match create_scheduled_snapshot(app, &config).await {
                Ok(()) => {
                    update_schedule_after_success(app, &config.profile_id, &config.interval)?;
                }
                Err(e) => {
                    eprintln!(
                        "[scheduler] Failed to create snapshot for profile {}: {e}",
                        config.profile_id
                    );
                }
            }
        }
    }

    Ok(())
}

/// Determine whether a schedule config is due for a snapshot.
fn is_due(config: &ScheduleConfig, now_str: &str) -> bool {
    // If never run before, it's due immediately
    let Some(ref next_due) = config.next_due_at else {
        return config.last_snapshot_at.is_none();
    };
    // Due if current time is past the next_due_at
    now_str >= next_due.as_str()
}

/// Create a snapshot for a scheduled profile.
#[allow(clippy::too_many_lines)]
async fn create_scheduled_snapshot(
    app: &tauri::AppHandle,
    config: &ScheduleConfig,
) -> Result<(), DsmError> {
    let db = app.state::<DbState>();
    let paths = app.state::<AppPaths>();
    let locks = app.state::<OperationLocks>();

    // Try to acquire the operation lock — if busy, skip this cycle
    let _op_guard = {
        let mut map = locks.0.lock().map_err(|e| DsmError::ConnectionError {
            message: format!("Failed to acquire operation registry: {e}"),
        })?;

        let slot = map
            .entry(config.profile_id.clone())
            .or_insert_with(|| {
                std::sync::Arc::new(crate::db::OperationSlot {
                    busy: std::sync::atomic::AtomicBool::new(false),
                    operation: std::sync::Mutex::new(String::new()),
                })
            })
            .clone();

        drop(map);

        if slot
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            if let Ok(mut op) = slot.operation.lock() {
                *op = "scheduled_snapshot".to_string();
            }
            crate::db::ProfileOpGuard { slot }
        } else {
            // Another operation in progress — skip this cycle
            return Ok(());
        }
    };

    // Load profile
    let profile = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT id, project, database_name, db_type FROM profiles WHERE id = ?1",
            params![config.profile_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DsmError::ProfileNotFound(config.profile_id.clone())
            }
            other => DsmError::DatabaseError(other),
        })?
    };

    let (profile_id, project, database_name, db_type) = profile;

    // Get credentials
    let creds = crate::credentials::get_credentials(&profile_id)?;

    // Load full profile for SSH/host/port details
    let full_profile = {
        let conn = lock_db(&db)?;
        conn.query_row(
            "SELECT host, port, username, ssh_enabled, ssh_host, ssh_port, ssh_user FROM profiles WHERE id = ?1",
            params![profile_id],
            |row| {
                Ok(ProfileInfo {
                    host: row.get(0)?,
                    port: row.get(1)?,
                    username: row.get(2)?,
                    ssh_enabled: row.get::<_, bool>(3)?,
                    ssh_host: row.get(4)?,
                    ssh_port: row.get(5)?,
                    ssh_user: row.get(6)?,
                })
            },
        )
        .map_err(DsmError::DatabaseError)?
    };

    // Open SSH tunnel if needed
    let mut tunnel: Option<crate::ssh::SshTunnel> = None;
    let (effective_host, effective_port) = if full_profile.ssh_enabled {
        let ssh_host = full_profile.ssh_host.as_deref().unwrap_or("localhost");
        let ssh_port = full_profile.ssh_port.unwrap_or(22);
        let ssh_user = full_profile.ssh_user.as_deref().unwrap_or("root");
        let db_host = full_profile.host.as_deref().unwrap_or("127.0.0.1");
        let db_port = full_profile.port.unwrap_or(default_port(&db_type));

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
            full_profile
                .host
                .clone()
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            full_profile.port.unwrap_or(default_port(&db_type)),
        )
    };

    // Generate snapshot identifiers
    let snapshot_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    let name = format!("{}-scheduled-{}", database_name, now.format("%Y%m%d-%H%M"));

    // Ensure project subdirectory exists
    let project_dir = paths.snapshots_dir.join(&project);
    std::fs::create_dir_all(&project_dir)?;

    let relative_path = format!("{project}/{snapshot_id}.sql.gz");
    let output_path = paths.snapshots_dir.join(&relative_path);

    let mut defaults_file: Option<std::path::PathBuf> = None;

    // Run the dump
    let size_bytes = match db_type.as_str() {
        "mysql" => {
            crate::dump::find_tool("mysqldump")?;
            let df = crate::dump::write_mysql_defaults_file(
                &paths.tmp_dir,
                &effective_host,
                effective_port,
                full_profile.username.as_deref().unwrap_or("root"),
                creds.password.as_deref().unwrap_or(""),
            )?;
            let mut cmd = crate::dump::build_mysqldump_command(&df, &database_name);
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
                return Err(DsmError::DumpError {
                    message: format!("mysqldump exited with status {}", child_output.status),
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
                full_profile.username.as_deref().unwrap_or("postgres"),
                creds.password.as_deref().unwrap_or(""),
                &database_name,
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
                return Err(DsmError::DumpError {
                    message: format!("pg_dump exited with status {}", child_output.status),
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
                message: format!("Compression task failed: {e}"),
                output: String::new(),
            })??;

            compressed_size
        }
        "sqlite" => {
            let temp_db_path = paths.tmp_dir.join(format!("{snapshot_id}.db"));

            let mut cmd = crate::dump::build_sqlite_dump_command(
                &database_name,
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

            let temp = temp_db_path.clone();
            let out = output_path.clone();
            let compressed_size =
                tokio::task::spawn_blocking(move || crate::compress::compress_file(&temp, &out))
                    .await
                    .map_err(|e| DsmError::DumpError {
                        message: format!("Compression task failed: {e}"),
                        output: String::new(),
                    })??;

            let _ = std::fs::remove_file(&temp_db_path);
            compressed_size
        }
        _ => {
            return Err(DsmError::DumpError {
                message: format!("Unsupported database type: {db_type}"),
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

    // Compute checksum
    let checksum_path = output_path.clone();
    let checksum =
        tokio::task::spawn_blocking(move || crate::checksum::compute_sha256(&checksum_path))
            .await
            .map_err(|e| DsmError::DumpError {
                message: format!("Checksum task failed: {e}"),
                output: String::new(),
            })??;

    // Save snapshot metadata with scheduled tags
    let now_str = now.to_rfc3339();
    let tags = vec![
        "[auto] scheduled".to_string(),
        format!("[interval] {}", config.interval),
    ];
    {
        let conn = lock_db(&db)?;
        conn.execute(
            "INSERT INTO snapshots (id, profile_id, name, note, file_path, size_bytes, checksum, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                snapshot_id,
                profile_id,
                name,
                format!("Automatic {} snapshot", config.interval),
                relative_path,
                size_bytes.cast_signed(),
                checksum,
                now_str,
            ],
        )?;

        for tag in &tags {
            conn.execute(
                "INSERT OR IGNORE INTO snapshot_tags (snapshot_id, tag) VALUES (?1, ?2)",
                params![snapshot_id, tag],
            )?;
        }
    }

    eprintln!(
        "[scheduler] Created scheduled snapshot for profile {} ({})",
        profile_id, config.interval
    );

    Ok(())
}

/// Update the schedule config after a successful snapshot.
fn update_schedule_after_success(
    app: &tauri::AppHandle,
    profile_id: &str,
    interval: &str,
) -> Result<(), String> {
    let db = app.state::<DbState>();
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();
    let next_due = compute_next_due(&now, interval);

    conn.execute(
        "UPDATE schedule_configs SET last_snapshot_at = ?1, next_due_at = ?2, updated_at = ?3
         WHERE profile_id = ?4",
        params![now, next_due, now, profile_id],
    )
    .map_err(|e| format!("Failed to update schedule config: {e}"))?;

    Ok(())
}

fn lock_db(db: &DbState) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, DsmError> {
    db.0.lock().map_err(|e| DsmError::ConnectionError {
        message: format!("Failed to acquire database lock: {e}"),
    })
}

fn default_port(db_type: &str) -> u16 {
    match db_type {
        "mysql" => 3306,
        "postgresql" => 5432,
        _ => 0,
    }
}

/// Minimal profile info for the scheduler (avoids pulling the full Profile struct).
struct ProfileInfo {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    ssh_enabled: bool,
    ssh_host: Option<String>,
    ssh_port: Option<u16>,
    ssh_user: Option<String>,
}

# Rust Backend Implementation Plan

> Phase 1 — Database Snapshot Manager
>
> **Date:** 19 February 2026
> **Scope:** All Rust backend components for real database snapshot/restore operations

---

## 1. Rust Dependencies

### Cargo.toml Additions

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# New dependencies for Phase 1
rusqlite = { version = "0.32", features = ["bundled"] }
rusqlite_migration = { version = "1", features = [] }
flate2 = "1.1"
uuid = { version = "1", features = ["v4"] }
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
keyring = { version = "3", features = ["apple-native"] }
fix-path-env = { git = "https://github.com/tauri-apps/fix-path-env-rs" }
tokio = { version = "1", features = ["process", "io-util", "time", "fs"] }
```

### Dependency Rationale

| Crate | Version | Purpose |
|---|---|---|
| `rusqlite` | 0.32 | SQLite metadata store; `bundled` feature embeds SQLite so no system dependency |
| `rusqlite_migration` | 1.x | Lightweight embedded SQL migrations using `user_version` pragma |
| `flate2` | 1.1 | gzip compression/decompression for `.sql.gz` snapshot files |
| `uuid` | 1.x | UUID v4 generation for profile and snapshot IDs |
| `thiserror` | 2.x | Derive macro for structured error enums |
| `chrono` | 0.4 | ISO 8601 timestamps for `created_at`, `updated_at`, `restored_at` |
| `keyring` | 3.x | Direct macOS Keychain access; `apple-native` feature required for macOS |
| `fix-path-env` | git | Fixes PATH inheritance for macOS GUI apps (finds `mysqldump`, `pg_dump`, `sqlite3`) |
| `tokio` | 1.x | Async subprocess spawning, I/O, and timeouts for dump/restore/SSH |

**Note on `rusqlite` version:** We pin to 0.32 rather than the latest 0.38 because Tauri 2's bundled SQLite and `rusqlite_migration 1.x` are tested against the 0.32 line. Upgrading can be done after verifying compatibility.

**Note on `keyring` vs `tauri-plugin-keyring`:** We use the `keyring` crate directly (v3.x) rather than `tauri-plugin-keyring` (v0.1.0). The plugin is a thin wrapper and using the crate directly gives us full control, avoids an immature plugin dependency, and the Rust-side API is straightforward.

**Note on `fix-path-env`:** This crate is not published on crates.io; it must be pulled from the Tauri GitHub repository. It is the officially recommended solution from the Tauri team.

---

## 2. SQLite Setup

### Database Initialisation

File: `src-tauri/src/db.rs`

```rust
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::PathBuf;
use std::sync::Mutex;

/// Embedded migrations — add new M::up() entries for schema changes.
fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "CREATE TABLE profiles (
                id              TEXT PRIMARY KEY,
                project         TEXT NOT NULL,
                name            TEXT NOT NULL,
                db_type         TEXT NOT NULL CHECK(db_type IN ('mysql','postgresql','sqlite')),
                host            TEXT,
                port            INTEGER,
                database_name   TEXT NOT NULL,
                username        TEXT,
                ssh_enabled     INTEGER DEFAULT 0,
                ssh_host        TEXT,
                ssh_port        INTEGER DEFAULT 22,
                ssh_user        TEXT,
                notes           TEXT,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );

            CREATE TABLE snapshots (
                id                TEXT PRIMARY KEY,
                profile_id        TEXT NOT NULL REFERENCES profiles(id),
                name              TEXT NOT NULL,
                note              TEXT,
                file_path         TEXT NOT NULL,
                size_bytes        INTEGER NOT NULL,
                db_version        TEXT,
                dump_tool_version TEXT,
                created_at        TEXT NOT NULL,
                restored_at       TEXT
            );

            CREATE TABLE settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        ),
    ])
}

/// Open (or create) the metadata database and run pending migrations.
pub fn init_db(app_data_dir: &PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(app_data_dir)?;
    let db_path = app_data_dir.join("metadata.db");
    let mut conn = Connection::open(&db_path)?;

    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // Run migrations
    migrations().to_latest(&mut conn)?;

    Ok(conn)
}
```

### Connection Management

The database connection is wrapped in a `Mutex` and registered as Tauri managed state:

```rust
pub struct DbState(pub Mutex<Connection>);
```

In `lib.rs`:

```rust
use crate::db::{init_db, DbState};

pub fn run() {
    let _ = fix_path_env::fix();

    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let conn = init_db(&app_data_dir)
                .expect("failed to initialise database");

            app.manage(DbState(Mutex::new(conn)));
            app.manage(AppPaths {
                data_dir: app_data_dir.clone(),
                snapshots_dir: app_data_dir.join("snapshots"),
                tmp_dir: app_data_dir.join("tmp"),
            });

            // Ensure directories exist
            std::fs::create_dir_all(app_data_dir.join("snapshots"))?;
            std::fs::create_dir_all(app_data_dir.join("tmp"))?;

            // Clean tmp directory on startup
            if let Ok(entries) = std::fs::read_dir(app_data_dir.join("tmp")) {
                for entry in entries.flatten() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profile_create,
            profile_update,
            profile_delete,
            profile_list,
            profile_test_connection,
            snapshot_create,
            snapshot_list,
            snapshot_restore,
            snapshot_delete,
            storage_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Migrations Approach

- **Embedded SQL** via `rusqlite_migration` — no external migration CLI or files needed.
- Migrations are stored as `M::up()` entries in Rust code.
- The crate tracks state via SQLite's `user_version` pragma (lightweight integer, no migration table).
- To add a new migration, append a new `M::up()` entry to the vector. Order matters and entries must never be modified after release.

---

## 3. Keychain Integration

### Direct `keyring` Crate Usage

File: `src-tauri/src/credentials.rs`

```rust
use serde::{Deserialize, Serialize};

const SERVICE_PREFIX: &str = "com.dsm.profile";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProfileCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_password: Option<String>,
}

fn entry_for(profile_id: &str) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(&format!("{}.{}", SERVICE_PREFIX, profile_id), "credentials")
}

/// Store credentials as JSON in macOS Keychain.
pub fn store_credentials(
    profile_id: &str,
    creds: &ProfileCredentials,
) -> Result<(), crate::error::DsmError> {
    let entry = entry_for(profile_id)?;
    let json = serde_json::to_string(creds)?;
    entry.set_password(&json)?;
    Ok(())
}

/// Retrieve credentials from macOS Keychain.
pub fn get_credentials(
    profile_id: &str,
) -> Result<ProfileCredentials, crate::error::DsmError> {
    let entry = entry_for(profile_id)?;
    match entry.get_password() {
        Ok(json) => Ok(serde_json::from_str(&json)?),
        Err(keyring::Error::NoEntry) => Ok(ProfileCredentials::default()),
        Err(e) => Err(e.into()),
    }
}

/// Delete credentials from macOS Keychain.
pub fn delete_credentials(profile_id: &str) -> Result<(), crate::error::DsmError> {
    let entry = entry_for(profile_id)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already gone, not an error
        Err(e) => Err(e.into()),
    }
}
```

### Keychain Entry Format

- **Service:** `com.dsm.profile.{profile_id}` (e.g. `com.dsm.profile.a1b2c3d4-...`)
- **User:** `credentials` (constant — one entry per profile)
- **Value:** JSON string: `{"password":"secret","ssh_key_path":"/Users/x/.ssh/id_rsa","ssh_password":"passphrase"}`

### macOS-Specific Notes

- The `apple-native` feature on the `keyring` crate must be enabled. Without it, no credential store is compiled in.
- On first access, macOS may prompt the user to allow Keychain access. This is expected behaviour.
- The app's code signing identity determines Keychain access — in development, the prompt appears each launch. In production (signed .app bundle), the grant persists.

---

## 4. Subprocess Handling

### File: `src-tauri/src/dump.rs`

### MySQL — `mysqldump` with `--defaults-extra-file`

```rust
use std::io::Write;
use tokio::process::Command;

/// Create a temporary MySQL options file for safe credential passing.
/// Returns the temp file path. The file is created in the app's tmp directory.
fn write_mysql_defaults_file(
    tmp_dir: &std::path::Path,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<std::path::PathBuf, std::io::Error> {
    let path = tmp_dir.join(format!("mysql-defaults-{}.cnf", uuid::Uuid::new_v4()));
    let mut file = std::fs::File::create(&path)?;

    // Restrictive permissions — owner read/write only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }

    writeln!(file, "[client]")?;
    writeln!(file, "host={}", host)?;
    writeln!(file, "port={}", port)?;
    writeln!(file, "user={}", username)?;
    writeln!(file, "password={}", password)?;

    Ok(path)
}

/// Build the mysqldump command.
pub fn build_mysqldump_command(
    defaults_file: &std::path::Path,
    database: &str,
) -> Command {
    let mut cmd = Command::new("mysqldump");
    cmd.arg(format!("--defaults-extra-file={}", defaults_file.display()));
    cmd.arg("--single-transaction");
    cmd.arg("--routines");
    cmd.arg("--triggers");
    cmd.arg("--set-gtid-purged=OFF");
    cmd.arg(database);
    cmd
}
```

**Why `--defaults-extra-file`:** Passing passwords via command-line arguments exposes them in `ps` output. The `--defaults-extra-file` option reads credentials from a temporary file with 0600 permissions, which is the MySQL-recommended secure approach. The temp file is deleted immediately after the dump completes.

### PostgreSQL — `pg_dump` with `PGPASSWORD` Environment Variable

```rust
pub fn build_pg_dump_command(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    database: &str,
) -> Command {
    let mut cmd = Command::new("pg_dump");
    cmd.env("PGPASSWORD", password);
    cmd.arg("--host").arg(host);
    cmd.arg("--port").arg(port.to_string());
    cmd.arg("--username").arg(username);
    cmd.arg("--format=plain");
    cmd.arg("--no-owner");
    cmd.arg("--no-acl");
    cmd.arg(database);
    cmd
}
```

**Note on PGPASSWORD vs .pgpass:** While `.pgpass` is considered more secure, `PGPASSWORD` set via `Command::env()` is scoped to the child process only and does not appear in `ps` output on modern macOS. For a desktop app where the user owns the process tree, this is acceptable. A `.pgpass` approach can be added later if needed.

### SQLite — `sqlite3 VACUUM INTO`

```rust
pub fn build_sqlite_dump_command(
    source_db_path: &str,
    output_path: &str,
) -> Command {
    let mut cmd = Command::new("sqlite3");
    cmd.arg(source_db_path);
    cmd.arg(format!("VACUUM INTO '{}';", output_path));
    cmd
}
```

**Note:** `VACUUM INTO` creates a clean, compact copy of the database. It requires SQLite 3.27.0+ (macOS ships with a recent enough version). The output is a raw `.db` file, which we then gzip compress.

### Restore Commands

```rust
/// Build mysql restore command (pipes SQL into mysql client).
pub fn build_mysql_restore_command(
    defaults_file: &std::path::Path,
    database: &str,
) -> Command {
    let mut cmd = Command::new("mysql");
    cmd.arg(format!("--defaults-extra-file={}", defaults_file.display()));
    cmd.arg(database);
    cmd.stdin(std::process::Stdio::piped());
    cmd
}

/// Build pg_restore command (pipes SQL into psql).
pub fn build_psql_restore_command(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    database: &str,
) -> Command {
    let mut cmd = Command::new("psql");
    cmd.env("PGPASSWORD", password);
    cmd.arg("--host").arg(host);
    cmd.arg("--port").arg(port.to_string());
    cmd.arg("--username").arg(username);
    cmd.arg(database);
    cmd.stdin(std::process::Stdio::piped());
    cmd
}
```

### Tool Detection on PATH

```rust
use std::path::PathBuf;

/// Check if a CLI tool is available on PATH.
/// Returns the full path if found, or an error with Homebrew install instructions.
pub fn find_tool(tool_name: &str) -> Result<PathBuf, crate::error::DsmError> {
    which::which(tool_name).map_err(|_| {
        let install_hint = match tool_name {
            "mysqldump" | "mysql" => "brew install mysql-client",
            "pg_dump" | "psql" => "brew install libpq",
            "sqlite3" => "sqlite3 is bundled with macOS",
            _ => "install the required tool",
        };
        crate::error::DsmError::ToolNotFound {
            tool: tool_name.to_string(),
            install_hint: install_hint.to_string(),
        }
    })
}
```

**Note:** Add `which = "7"` to `Cargo.toml` for the `which` crate (cross-platform tool-on-PATH detection). Combined with `fix-path-env`, this ensures tools installed via Homebrew (`/opt/homebrew/bin`) are discoverable even when the app is launched from Finder.

**Updated Cargo.toml addition:**
```toml
which = "7"
```

---

## 5. SSH Tunnel Management

### File: `src-tauri/src/ssh.rs`

```rust
use tokio::process::{Child, Command};
use tokio::time::{timeout, Duration};
use std::net::TcpStream;

pub struct SshTunnel {
    child: Child,
    local_port: u16,
}

impl SshTunnel {
    /// Spawn an SSH tunnel: ssh -L local_port:db_host:db_port user@ssh_host -p ssh_port -N
    pub async fn open(
        ssh_host: &str,
        ssh_port: u16,
        ssh_user: &str,
        ssh_key_path: Option<&str>,
        db_host: &str,
        db_port: u16,
    ) -> Result<Self, crate::error::DsmError> {
        // Pick an ephemeral local port
        let local_port = find_available_port()?;

        let mut cmd = Command::new("ssh");
        cmd.arg("-L")
            .arg(format!("{}:{}:{}", local_port, db_host, db_port));
        cmd.arg(format!("{}@{}", ssh_user, ssh_host));
        cmd.arg("-p").arg(ssh_port.to_string());
        cmd.arg("-N");  // No remote command — tunnel only
        cmd.arg("-o").arg("StrictHostKeyChecking=accept-new");
        cmd.arg("-o").arg("ConnectTimeout=10");
        cmd.arg("-o").arg("ServerAliveInterval=15");
        cmd.arg("-o").arg("ServerAliveCountMax=3");
        cmd.arg("-o").arg("ExitOnForwardFailure=yes");

        if let Some(key_path) = ssh_key_path {
            cmd.arg("-i").arg(key_path);
        }

        // Detach from stdin to prevent blocking
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::piped());

        let child = cmd.spawn().map_err(|e| crate::error::DsmError::SshError(
            format!("Failed to spawn ssh: {}", e)
        ))?;

        let tunnel = SshTunnel { child, local_port };

        // Wait for tunnel to become ready (poll TCP connection to local_port)
        tunnel.wait_for_ready().await?;

        Ok(tunnel)
    }

    /// Poll the local port until it accepts connections (tunnel is ready).
    async fn wait_for_ready(&self) -> Result<(), crate::error::DsmError> {
        let addr = format!("127.0.0.1:{}", self.local_port);
        let result = timeout(Duration::from_secs(10), async {
            loop {
                if TcpStream::connect(&addr).is_ok() {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            _ => Err(crate::error::DsmError::SshError(
                "SSH tunnel failed to become ready within 10 seconds".to_string(),
            )),
        }
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Kill the SSH tunnel process.
    pub async fn close(mut self) -> Result<(), crate::error::DsmError> {
        self.child.kill().await.map_err(|e| {
            crate::error::DsmError::SshError(format!("Failed to kill SSH tunnel: {}", e))
        })
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        // Best-effort kill on drop — fire and forget
        let _ = self.child.start_kill();
    }
}

/// Find an available TCP port by binding to port 0.
fn find_available_port() -> Result<u16, crate::error::DsmError> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| crate::error::DsmError::SshError(
            format!("Failed to find available port: {}", e)
        ))?;
    Ok(listener.local_addr().unwrap().port())
}
```

### SSH Tunnel Lifecycle

1. **Open:** `SshTunnel::open()` spawns `ssh -L` as a child process with a random local port.
2. **Ready detection:** Polls `TcpStream::connect("127.0.0.1:{local_port}")` every 200ms with a 10-second timeout. `ExitOnForwardFailure=yes` ensures SSH exits if the port forward fails.
3. **Usage:** The caller uses `tunnel.local_port()` as the `host:port` for the dump/restore command (connecting to `127.0.0.1:{local_port}` instead of the remote DB host).
4. **Cleanup:** `SshTunnel::close()` kills the SSH process. The `Drop` impl provides a safety net if the struct is dropped without explicit close.

---

## 6. Gzip Compression

### File: `src-tauri/src/compress.rs`

```rust
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Compress a raw SQL file to .sql.gz using streaming gzip.
/// Returns the compressed file size in bytes.
pub fn compress_file(input: &Path, output: &Path) -> Result<u64, io::Error> {
    let input_file = File::open(input)?;
    let reader = BufReader::new(input_file);

    let output_file = File::create(output)?;
    let writer = BufWriter::new(output_file);
    let mut encoder = GzEncoder::new(writer, Compression::default());

    // Stream in 64KB chunks to keep memory usage low
    let mut buf_reader = BufReader::with_capacity(64 * 1024, reader);
    io::copy(&mut buf_reader, &mut encoder)?;

    encoder.finish()?;

    Ok(std::fs::metadata(output)?.len())
}

/// Decompress a .sql.gz file to raw SQL, streaming to a writer.
/// Used for restore: decompress -> pipe into mysql/psql stdin.
pub fn decompress_to_writer<W: Write>(
    gz_path: &Path,
    writer: &mut W,
) -> Result<(), io::Error> {
    let file = File::open(gz_path)?;
    let reader = BufReader::new(file);
    let mut decoder = GzDecoder::new(reader);

    // Stream in 64KB chunks
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let bytes_read = decoder.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
}

/// Compress subprocess stdout directly to a .sql.gz file (zero intermediate file).
/// Reads from the child process stdout and streams through gzip to disk.
pub fn compress_from_reader<R: Read>(
    reader: R,
    output: &Path,
) -> Result<u64, io::Error> {
    let output_file = File::create(output)?;
    let writer = BufWriter::new(output_file);
    let mut encoder = GzEncoder::new(writer, Compression::default());

    let mut buf_reader = BufReader::with_capacity(64 * 1024, reader);
    io::copy(&mut buf_reader, &mut encoder)?;

    encoder.finish()?;

    Ok(std::fs::metadata(output)?.len())
}
```

### Streaming Pipeline

For snapshot creation, the pipeline is:

```text
mysqldump stdout -> GzEncoder -> BufWriter -> .sql.gz file on disk
```

This avoids writing an uncompressed intermediate file. The `compress_from_reader` function takes the child process's stdout handle directly.

For restore, the pipeline is:

```text
.sql.gz file -> GzDecoder -> BufReader -> mysql/psql stdin
```

---

## 7. Progress Reporting

### Tauri 2 Channel-Based Progress

We use Tauri's `Channel` type for ordered, high-performance progress streaming. This is preferred over the event system for streaming data.

File: `src-tauri/src/progress.rs`

```rust
use serde::Serialize;
use tauri::ipc::Channel;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum SnapshotProgress {
    #[serde(rename_all = "camelCase")]
    Started {
        operation: String,    // "dump" or "restore"
        profile_name: String,
    },
    #[serde(rename_all = "camelCase")]
    Phase {
        phase: String,        // "connecting", "ssh_tunnel", "dumping", "compressing", "restoring", "decompressing"
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        percentage: u8,       // 0-100
        bytes_processed: u64,
    },
    #[serde(rename_all = "camelCase")]
    Completed {
        message: String,
        size_bytes: Option<u64>,
        duration_secs: f64,
    },
    #[serde(rename_all = "camelCase")]
    Failed {
        error: String,
        phase: String,
    },
}
```

### Frontend Channel Setup (TypeScript)

```typescript
import { invoke, Channel } from '@tauri-apps/api/core';

type SnapshotProgress =
  | { event: 'started'; data: { operation: string; profileName: string } }
  | { event: 'phase'; data: { phase: string; message: string } }
  | { event: 'progress'; data: { percentage: number; bytesProcessed: number } }
  | { event: 'completed'; data: { message: string; sizeBytes: number | null; durationSecs: number } }
  | { event: 'failed'; data: { error: string; phase: string } };

const onProgress = new Channel<SnapshotProgress>();
onProgress.onmessage = (msg) => {
  switch (msg.event) {
    case 'started':
      // Show progress bar
      break;
    case 'phase':
      // Update status text
      break;
    case 'progress':
      // Update progress bar percentage
      break;
    case 'completed':
      // Show success toast
      break;
    case 'failed':
      // Show error toast
      break;
  }
};

await invoke('snapshot_create', {
  profileId: '...',
  name: 'My Snapshot',
  note: 'Before migration',
  onProgress,
});
```

### Progress Estimation Strategy

Since `mysqldump` and `pg_dump` do not report progress, we use phase-based progress:

| Phase | Percentage Range |
|---|---|
| Connecting / SSH tunnel | 0-10% |
| Dumping (running subprocess) | 10-70% |
| Compressing | 70-90% |
| Saving metadata | 90-100% |

Within the dumping phase, we can estimate progress by monitoring bytes written to the gzip encoder vs. expected database size (from a prior `information_schema` query or previous snapshot size).

---

## 8. Error Handling

### File: `src-tauri/src/error.rs`

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum DsmError {
    #[error("Connection failed: {message}")]
    ConnectionError { message: String },

    #[error("Dump failed: {message}\n\nTool output:\n{output}")]
    DumpError { message: String, output: String },

    #[error("Restore failed: {message}\n\nTool output:\n{output}")]
    RestoreError { message: String, output: String },

    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),

    #[error("Keychain error: {0}")]
    KeychainError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("SSH tunnel error: {0}")]
    SshError(String),

    #[error("{tool} not found on PATH. Install with: {install_hint}")]
    ToolNotFound { tool: String, install_hint: String },

    #[error("Serialisation error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),
}

// Serialise errors as structured JSON for the frontend.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorPayload {
    kind: String,
    message: String,
}

impl Serialize for DsmError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let kind = match self {
            DsmError::ConnectionError { .. } => "connectionError",
            DsmError::DumpError { .. } => "dumpError",
            DsmError::RestoreError { .. } => "restoreError",
            DsmError::FileSystemError(_) => "fileSystemError",
            DsmError::KeychainError(_) => "keychainError",
            DsmError::DatabaseError(_) => "databaseError",
            DsmError::SshError(_) => "sshError",
            DsmError::ToolNotFound { .. } => "toolNotFound",
            DsmError::SerdeError(_) => "serdeError",
            DsmError::ProfileNotFound(_) => "profileNotFound",
            DsmError::SnapshotNotFound(_) => "snapshotNotFound",
        };

        ErrorPayload {
            kind: kind.to_string(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

// Convert keyring errors
impl From<keyring::Error> for DsmError {
    fn from(e: keyring::Error) -> Self {
        DsmError::KeychainError(e.to_string())
    }
}
```

### Frontend Error Shape

All errors arrive as JSON:

```json
{
  "kind": "toolNotFound",
  "message": "mysqldump not found on PATH. Install with: brew install mysql-client"
}
```

The frontend can switch on `kind` for specific error handling (e.g. showing an install prompt for `toolNotFound`) whilst displaying `message` to the user.

---

## 9. Tauri Command Signatures

### File: `src-tauri/src/commands.rs`

All commands return `Result<T, DsmError>` where `DsmError` implements `Serialize`.

```rust
use crate::db::DbState;
use crate::error::DsmError;
use crate::progress::SnapshotProgress;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::State;

// ── Data Types ────────────────────────────────────────────────────

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

// ── Profile Commands ──────────────────────────────────────────────

#[tauri::command]
pub async fn profile_create(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    input: CreateProfileInput,
) -> Result<Profile, DsmError> {
    // 1. Generate UUID
    // 2. Insert into profiles table
    // 3. Store credentials in Keychain
    // 4. Return the created profile
    todo!()
}

#[tauri::command]
pub async fn profile_update(
    db: State<'_, DbState>,
    id: String,
    input: UpdateProfileInput,
) -> Result<Profile, DsmError> {
    // 1. Verify profile exists
    // 2. Update changed fields in profiles table
    // 3. Update credentials in Keychain if password/ssh fields changed
    // 4. Return the updated profile
    todo!()
}

#[tauri::command]
pub async fn profile_delete(
    db: State<'_, DbState>,
    id: String,
) -> Result<(), DsmError> {
    // 1. Delete all snapshots (files + metadata) for this profile
    // 2. Delete credentials from Keychain
    // 3. Delete profile from profiles table
    todo!()
}

#[tauri::command]
pub async fn profile_list(
    db: State<'_, DbState>,
) -> Result<Vec<Profile>, DsmError> {
    // 1. SELECT * FROM profiles ORDER BY project, name
    // 2. Return as Vec<Profile>
    todo!()
}

#[tauri::command]
pub async fn profile_test_connection(
    db: State<'_, DbState>,
    id: String,
) -> Result<ConnectionTestResult, DsmError> {
    // 1. Load profile + credentials
    // 2. Open SSH tunnel if ssh_enabled
    // 3. Attempt database connection (mysql --execute="SELECT VERSION()" / psql -c "SELECT version()")
    // 4. Measure latency
    // 5. Close SSH tunnel
    // 6. Return result
    todo!()
}

// ── Snapshot Commands ─────────────────────────────────────────────

#[tauri::command]
pub async fn snapshot_create(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    profile_id: String,
    name: String,
    note: Option<String>,
    on_progress: Channel<SnapshotProgress>,
) -> Result<Snapshot, DsmError> {
    // 1. Load profile + credentials
    // 2. Send Started progress
    // 3. Open SSH tunnel if needed (Phase: "ssh_tunnel")
    // 4. Find dump tool on PATH
    // 5. Build dump command
    // 6. Spawn subprocess, pipe stdout through GzEncoder to .sql.gz (Phase: "dumping")
    // 7. Wait for completion, check exit code
    // 8. Record metadata in snapshots table (Phase: "saving")
    // 9. Clean up temp files, close SSH tunnel
    // 10. Send Completed progress
    // 11. Return Snapshot
    todo!()
}

#[tauri::command]
pub async fn snapshot_list(
    db: State<'_, DbState>,
    profile_id: String,
) -> Result<Vec<Snapshot>, DsmError> {
    // 1. SELECT * FROM snapshots WHERE profile_id = ? ORDER BY created_at DESC
    // 2. Return as Vec<Snapshot>
    todo!()
}

#[tauri::command]
pub async fn snapshot_restore(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
    snapshot_id: String,
    on_progress: Channel<SnapshotProgress>,
) -> Result<(), DsmError> {
    // 1. Load snapshot metadata + profile + credentials
    // 2. Send Started progress
    // 3. Open SSH tunnel if needed
    // 4. Find restore tool on PATH
    // 5. Build restore command
    // 6. Decompress .sql.gz and pipe into restore command stdin (Phase: "restoring")
    // 7. Wait for completion, check exit code
    // 8. Update restored_at timestamp in snapshots table
    // 9. Clean up SSH tunnel
    // 10. Send Completed progress
    todo!()
}

#[tauri::command]
pub async fn snapshot_delete(
    db: State<'_, DbState>,
    snapshot_id: String,
) -> Result<(), DsmError> {
    // 1. Load snapshot metadata
    // 2. Delete .sql.gz file from disk
    // 3. Delete record from snapshots table
    todo!()
}

// ── Storage Commands ──────────────────────────────────────────────

#[tauri::command]
pub async fn storage_usage(
    db: State<'_, DbState>,
    paths: State<'_, AppPaths>,
) -> Result<StorageInfo, DsmError> {
    // 1. Walk snapshots directory
    // 2. Group by project slug
    // 3. Sum file sizes per project
    // 4. Return StorageInfo
    todo!()
}
```

### `AppPaths` State

```rust
#[derive(Clone)]
pub struct AppPaths {
    pub data_dir: std::path::PathBuf,
    pub snapshots_dir: std::path::PathBuf,
    pub tmp_dir: std::path::PathBuf,
}
```

---

## 10. File Structure

```bash
src-tauri/src/
    lib.rs              # Tauri app builder, plugin registration, state setup
    commands.rs         # All #[tauri::command] functions and IPC data types
    db.rs               # SQLite connection init, migrations, DbState
    credentials.rs      # Keychain store/get/delete via keyring crate
    dump.rs             # Subprocess command builders for dump/restore
    ssh.rs              # SshTunnel struct with open/close/drop
    compress.rs         # GzEncoder/GzDecoder streaming helpers
    progress.rs         # SnapshotProgress enum for channel events
    error.rs            # DsmError enum with thiserror + serde::Serialize
```

---

## 11. Final Cargo.toml

```toml
[package]
name = "database-snapshot-manager"
version = "0.1.0"
description = "Database Snapshot Manager desktop app"
authors = ["you"]
edition = "2021"

[lib]
name = "database_snapshot_manager_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
rusqlite_migration = "1"
flate2 = "1.1"
uuid = { version = "1", features = ["v4"] }
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
keyring = { version = "3", features = ["apple-native"] }
fix-path-env = { git = "https://github.com/tauri-apps/fix-path-env-rs" }
tokio = { version = "1", features = ["process", "io-util", "time", "fs"] }
which = "7"
```

---

*Plan authored 19 February 2026*

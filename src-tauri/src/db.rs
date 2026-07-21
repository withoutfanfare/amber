use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Managed Tauri state wrapping the `SQLite` connection.
pub struct DbState(pub Mutex<Connection>);

/// Application directory paths, registered as managed Tauri state.
#[derive(Clone)]
#[allow(clippy::struct_field_names)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub snapshots_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

/// Tracks whether an operation is in progress for a given profile.
pub struct OperationSlot {
    pub busy: std::sync::atomic::AtomicBool,
    pub operation: Mutex<String>,
}

/// Per-profile operation lock preventing concurrent snapshot/restore operations.
pub struct OperationLocks(pub Mutex<HashMap<String, Arc<OperationSlot>>>);

/// RAII guard that clears the busy flag when dropped.
pub struct ProfileOpGuard {
    pub(crate) slot: Arc<OperationSlot>,
}

impl Drop for ProfileOpGuard {
    fn drop(&mut self) {
        self.slot
            .busy
            .store(false, std::sync::atomic::Ordering::Release);
        if let Ok(mut op) = self.slot.operation.lock() {
            op.clear();
        }
    }
}

/// Embedded migrations -- add new `M::up()` entries for schema changes.
#[allow(clippy::too_many_lines)]
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
            );",
        ),
        M::up(
            "CREATE TABLE restore_history (
                id                TEXT PRIMARY KEY,
                snapshot_id       TEXT NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
                snapshot_name     TEXT NOT NULL,
                target_profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
                target_db_name    TEXT NOT NULL,
                duration_secs     REAL NOT NULL,
                restored_at       TEXT NOT NULL
            );

            CREATE INDEX idx_restore_history_restored_at
                ON restore_history(restored_at DESC);",
        ),
        M::up(
            "ALTER TABLE profiles ADD COLUMN environment TEXT CHECK(environment IN ('local','staging','live'));",
        ),
        M::up(
            "ALTER TABLE snapshots ADD COLUMN checksum TEXT;",
        ),
        // Migration 5: Tags table for snapshot tagging
        M::up(
            "CREATE TABLE snapshot_tags (
                snapshot_id TEXT NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
                tag         TEXT NOT NULL,
                PRIMARY KEY (snapshot_id, tag)
            );

            CREATE INDEX idx_snapshot_tags_tag ON snapshot_tags(tag);",
        ),
        // Migration 6: Pinned flag for retention policy protection
        M::up(
            "ALTER TABLE snapshots ADD COLUMN pinned INTEGER DEFAULT 0;",
        ),
        // Migration 7: Retention policies per profile
        M::up(
            "CREATE TABLE retention_policies (
                profile_id      TEXT PRIMARY KEY REFERENCES profiles(id) ON DELETE CASCADE,
                max_count       INTEGER,
                max_age_days    INTEGER,
                max_size_bytes  INTEGER
            );",
        ),
        // Migration 8: Tool paths for setup wizard
        M::up(
            "CREATE TABLE tool_paths (
                tool_name   TEXT PRIMARY KEY,
                path        TEXT NOT NULL,
                version     TEXT,
                discovered_at TEXT NOT NULL
            );",
        ),
        // Migration 9: Scheduled snapshot configuration per profile
        M::up(
            "CREATE TABLE schedule_configs (
                profile_id        TEXT PRIMARY KEY REFERENCES profiles(id) ON DELETE CASCADE,
                interval          TEXT NOT NULL CHECK(interval IN ('disabled','hourly','every_6h','daily','weekly')),
                last_snapshot_at  TEXT,
                next_due_at       TEXT,
                created_at        TEXT NOT NULL,
                updated_at        TEXT NOT NULL
            );",
        ),
        // Migration 10: one profile can snapshot multiple databases on the same connection
        M::up(
            "ALTER TABLE profiles ADD COLUMN database_names TEXT NOT NULL DEFAULT '[]';
             ALTER TABLE snapshots ADD COLUMN database_name TEXT;",
        ),
        // Migration 11: record whether each snapshot can be restored locally
        M::up(
            "ALTER TABLE snapshots ADD COLUMN restore_test_status TEXT NOT NULL DEFAULT 'not_configured'
                 CHECK(restore_test_status IN ('passed','failed','not_configured'));
             ALTER TABLE snapshots ADD COLUMN restore_test_message TEXT;
             ALTER TABLE snapshots ADD COLUMN restore_tested_at TEXT;",
        ),
    ])
}

/// Expose migrations for unit tests (in-memory databases).
#[cfg(test)]
pub fn migrations_for_test() -> Migrations<'static> {
    migrations()
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

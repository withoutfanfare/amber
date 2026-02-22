use std::io::Write;
use std::path::PathBuf;
use tokio::process::Command;

/// Create a temporary `MySQL` options file for safe credential passing.
/// The file is created with 0600 permissions in the app's tmp directory.
pub fn write_mysql_defaults_file(
    tmp_dir: &std::path::Path,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<PathBuf, std::io::Error> {
    let path = tmp_dir.join(format!("mysql-defaults-{}.cnf", uuid::Uuid::new_v4()));
    let mut file = std::fs::File::create(&path)?;

    // Restrictive permissions -- owner read/write only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }

    writeln!(file, "[client]")?;
    writeln!(file, "host={host}")?;
    writeln!(file, "port={port}")?;
    writeln!(file, "user={username}")?;
    writeln!(file, "password={password}")?;

    Ok(path)
}

/// Build the `mysqldump` command with secure defaults-file credential passing.
pub fn build_mysqldump_command(defaults_file: &std::path::Path, database: &str) -> Command {
    let mut cmd = Command::new("mysqldump");
    cmd.arg(format!("--defaults-extra-file={}", defaults_file.display()));
    cmd.arg("--single-transaction");
    cmd.arg("--routines");
    cmd.arg("--triggers");
    cmd.arg("--set-gtid-purged=OFF");
    cmd.arg(database);
    cmd
}

/// Build the `pg_dump` command with PGPASSWORD environment variable.
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

/// Build the `sqlite3 VACUUM INTO` command for a clean database copy.
pub fn build_sqlite_dump_command(source_db_path: &str, output_path: &str) -> Command {
    let mut cmd = Command::new("sqlite3");
    cmd.arg(source_db_path);
    cmd.arg(format!("VACUUM INTO '{output_path}';"));
    cmd
}

/// Build the `mysql` restore command (pipes SQL into the mysql client).
pub fn build_mysql_restore_command(defaults_file: &std::path::Path, database: &str) -> Command {
    let mut cmd = Command::new("mysql");
    cmd.arg(format!("--defaults-extra-file={}", defaults_file.display()));
    cmd.arg(database);
    cmd.stdin(std::process::Stdio::piped());
    cmd
}

/// Build the `psql` restore command (pipes SQL into psql).
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

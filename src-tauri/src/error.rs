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

    #[error("Cannot restore {snapshot_type} snapshot to {profile_type} profile")]
    DbTypeMismatch { snapshot_type: String, profile_type: String },
}

/// Structured JSON payload sent to the frontend for all errors.
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
            Self::ConnectionError { .. } => "connectionError",
            Self::DumpError { .. } => "dumpError",
            Self::RestoreError { .. } => "restoreError",
            Self::FileSystemError(_) => "fileSystemError",
            Self::KeychainError(_) => "keychainError",
            Self::DatabaseError(_) => "databaseError",
            Self::SshError(_) => "sshError",
            Self::ToolNotFound { .. } => "toolNotFound",
            Self::SerdeError(_) => "serdeError",
            Self::ProfileNotFound(_) => "profileNotFound",
            Self::SnapshotNotFound(_) => "snapshotNotFound",
            Self::DbTypeMismatch { .. } => "dbTypeMismatch",
        };

        ErrorPayload {
            kind: kind.to_string(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

impl From<keyring::Error> for DsmError {
    fn from(e: keyring::Error) -> Self {
        Self::KeychainError(e.to_string())
    }
}

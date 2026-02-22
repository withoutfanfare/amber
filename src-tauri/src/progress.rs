use serde::Serialize;

/// Progress events streamed to the frontend via Tauri's `Channel` type.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum SnapshotProgress {
    #[serde(rename_all = "camelCase")]
    Started {
        operation: String,
        profile_name: String,
    },
    #[serde(rename_all = "camelCase")]
    Phase { phase: String, message: String },
    #[serde(rename_all = "camelCase")]
    Progress {
        percentage: u8,
        bytes_processed: u64,
    },
    #[serde(rename_all = "camelCase")]
    Completed {
        message: String,
        size_bytes: Option<u64>,
        duration_secs: f64,
    },
    #[serde(rename_all = "camelCase")]
    Failed { error: String, phase: String },
}

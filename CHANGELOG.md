# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Flexible snapshot restore** — restore a snapshot to a different profile and/or a different database name, with automatic database creation if the target does not exist. Cross-type restores (e.g. MySQL snapshot to PostgreSQL profile) are rejected with a `dbTypeMismatch` error.
- `RestoreOptions` struct on the Rust backend accepting optional `targetProfileId` and `targetDatabaseName` overrides.
- `SnapshotRestoreOptions` TypeScript interface on the frontend.
- `ensure_db` Rust module with pure helper functions for building database creation commands (`mysql_create_db_args`, `pg_check_db_args`, `pg_create_db_args`, `ensure_sqlite_parent_dir`, `validate_db_type_match`).
- `DbTypeMismatch` error variant in `DsmError` with serialised kind `"dbTypeMismatch"`.
- Profile selector in `SnapshotRestoreDialog` filtered to compatible database types.
- Database name override checkbox and input in the restore dialog.
- Warning banner in the restore dialog showing the exact target that will be overwritten.
- Vitest test suite for the snapshot store `restore()` action (7 tests).
- Vitest test suite for `SnapshotRestoreDialog` component (5 tests).
- 6 Rust unit tests for the `ensure_db` module.

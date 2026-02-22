# Testing Strategy & Dev Tooling Plan

> Database Snapshot Manager — testing, linting, formatting, and CI setup

**Date:** 19 February 2026
**Status:** Draft
**Scope:** Phase 1 — Rust backend tests, Vue 3 frontend tests, linting/formatting, CI pipeline

---

## 1. Rust Unit Testing

### 1.1 Test Structure in `src-tauri/`

Organise tests alongside source code using inline test modules plus a dedicated `tests/` directory for integration tests:

```text
src-tauri/
    src/
        lib.rs              # re-exports modules, Tauri builder
        db/
            mod.rs           # SQLite metadata operations
            mod_test.rs      # unit tests for db module (conditional)
        commands/
            mod.rs           # Tauri command handlers
            profiles.rs      # profile CRUD commands
            snapshots.rs     # snapshot commands
            storage.rs       # storage commands
        dump/
            mod.rs           # subprocess execution (mysqldump, pg_dump, sqlite3)
            command_builder.rs
        ssh/
            mod.rs           # SSH tunnel management
        compression/
            mod.rs           # gzip via flate2
    tests/
        integration/
            mod.rs
            dump_restore.rs  # real dump/restore against local DBs
    Cargo.toml
```

Each module includes inline `#[cfg(test)] mod tests { ... }` blocks for unit tests.

### 1.2 Cargo.toml Test Configuration

Add the `test` feature to the Tauri dependency and include `tempfile` for test fixtures:

```toml
[dependencies]
tauri = { version = "2", features = [] }
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
flate2 = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### 1.3 Testing SQLite with In-Memory Databases

Use `rusqlite::Connection::open_in_memory()` to create isolated, throwaway databases for each test. This avoids file system state and is fast.

```rust
// src-tauri/src/db/mod.rs

use rusqlite::{Connection, Result};

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS profiles (
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

        CREATE TABLE IF NOT EXISTS snapshots (
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

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_schema_creation() {
        let conn = setup_db();
        // Verify tables exist
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='profiles'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_profile() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO profiles (id, project, name, db_type, database_name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params!["p1", "myproject", "dev-db", "postgresql", "mydb", "2026-01-01T00:00:00Z", "2026-01-01T00:00:00Z"],
        ).unwrap();

        let name: String = conn
            .query_row("SELECT name FROM profiles WHERE id = ?1", ["p1"], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "dev-db");
    }

    #[test]
    fn test_db_type_check_constraint() {
        let conn = setup_db();
        let result = conn.execute(
            "INSERT INTO profiles (id, project, name, db_type, database_name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params!["p1", "proj", "bad", "mongodb", "db", "2026-01-01T00:00:00Z", "2026-01-01T00:00:00Z"],
        );
        assert!(result.is_err());
    }
}
```

### 1.4 Testing Tauri Commands

For simple Tauri commands that are pure functions (taking arguments, returning results), test the underlying function directly without the Tauri runtime. Extract business logic from `#[tauri::command]` handlers into testable functions:

```rust
// src-tauri/src/commands/profiles.rs

use crate::db;
use rusqlite::Connection;

/// Pure logic — testable without Tauri runtime
pub fn create_profile_impl(conn: &Connection, profile: &Profile) -> Result<(), String> {
    db::insert_profile(conn, profile).map_err(|e| e.to_string())
}

/// Tauri command — thin wrapper
#[tauri::command]
pub fn profile_create(state: tauri::State<'_, AppState>, profile: Profile) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    create_profile_impl(&conn, &profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_schema;
    use rusqlite::Connection;

    #[test]
    fn test_create_profile() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        let profile = Profile {
            id: "test-1".into(),
            project: "myproject".into(),
            name: "dev".into(),
            db_type: DbType::PostgreSql,
            // ...
        };

        let result = create_profile_impl(&conn, &profile);
        assert!(result.is_ok());
    }
}
```

### 1.5 Mocking Subprocess Execution

Wrap subprocess calls behind a trait so they can be mocked in tests:

```rust
// src-tauri/src/dump/mod.rs

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<std::process::Output, std::io::Error>;
}

/// Real implementation using std::process::Command
pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<std::process::Output, std::io::Error> {
        std::process::Command::new(program).args(args).output()
    }
}

/// Build the mysqldump command arguments
pub fn build_mysqldump_args(host: &str, port: u16, database: &str, username: &str) -> Vec<String> {
    vec![
        format!("--host={}", host),
        format!("--port={}", port),
        format!("--user={}", username),
        "--single-transaction".to_string(),
        "--routines".to_string(),
        "--triggers".to_string(),
        database.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCommandRunner {
        pub output: std::process::Output,
    }

    impl CommandRunner for MockCommandRunner {
        fn run(&self, _program: &str, _args: &[&str]) -> Result<std::process::Output, std::io::Error> {
            Ok(self.output.clone())
        }
    }

    #[test]
    fn test_build_mysqldump_args() {
        let args = build_mysqldump_args("localhost", 3306, "mydb", "root");
        assert!(args.contains(&"--host=localhost".to_string()));
        assert!(args.contains(&"--port=3306".to_string()));
        assert!(args.contains(&"mydb".to_string()));
    }

    #[test]
    fn test_build_pg_dump_args() {
        // Similar test for pg_dump command construction
    }

    #[test]
    fn test_ssh_tunnel_command() {
        // Verify SSH tunnel command is constructed correctly
    }
}
```

### 1.6 Running Rust Tests

```bash
# Run all unit tests
cd src-tauri && cargo test

# Run tests for a specific module
cd src-tauri && cargo test db::tests

# Run with output
cd src-tauri && cargo test -- --nocapture
```

---

## 2. Rust Integration Testing

### 2.1 Integration Test Structure

Place integration tests in `src-tauri/tests/` which compile as separate crates:

```text
src-tauri/tests/
    integration/
        mod.rs
        dump_restore.rs
```

### 2.2 Test Database Setup/Teardown

Create a helper script for setting up local test databases:

```bash
#!/usr/bin/env bash
# scripts/test-db-setup.sh — create test databases for integration tests

set -euo pipefail

echo "Setting up PostgreSQL test database..."
createdb dsm_test_db 2>/dev/null || echo "dsm_test_db already exists"
psql dsm_test_db -c "
    CREATE TABLE IF NOT EXISTS test_table (
        id SERIAL PRIMARY KEY,
        name TEXT NOT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW()
    );
    INSERT INTO test_table (name) VALUES ('seed_row_1'), ('seed_row_2')
    ON CONFLICT DO NOTHING;
"

echo "Setting up MySQL test database..."
mysql -u root -e "CREATE DATABASE IF NOT EXISTS dsm_test_db;"
mysql -u root dsm_test_db -e "
    CREATE TABLE IF NOT EXISTS test_table (
        id INT AUTO_INCREMENT PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    INSERT IGNORE INTO test_table (name) VALUES ('seed_row_1'), ('seed_row_2');
"

echo "Test databases ready."
```

```bash
#!/usr/bin/env bash
# scripts/test-db-teardown.sh — remove test databases

set -euo pipefail

echo "Dropping PostgreSQL test database..."
dropdb dsm_test_db 2>/dev/null || true

echo "Dropping MySQL test database..."
mysql -u root -e "DROP DATABASE IF EXISTS dsm_test_db;"

echo "Test databases cleaned up."
```

### 2.3 Integration Test Examples

```rust
// src-tauri/tests/integration/dump_restore.rs

use std::process::Command;
use tempfile::TempDir;

/// Skip integration tests unless DSM_INTEGRATION env var is set
fn skip_unless_integration() {
    if std::env::var("DSM_INTEGRATION").is_err() {
        eprintln!("Skipping integration test (set DSM_INTEGRATION=1 to run)");
        return;
    }
}

#[test]
fn test_pg_dump_and_restore() {
    skip_unless_integration();

    let tmp = TempDir::new().unwrap();
    let dump_path = tmp.path().join("test.sql");

    // Dump
    let output = Command::new("pg_dump")
        .args(["--format=plain", "--file", dump_path.to_str().unwrap(), "dsm_test_db"])
        .output()
        .expect("pg_dump failed to execute");

    assert!(output.status.success(), "pg_dump failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dump_path.exists());
    assert!(std::fs::metadata(&dump_path).unwrap().len() > 0);

    // Restore to a fresh database
    let _ = Command::new("createdb").arg("dsm_test_restore").output();

    let restore_output = Command::new("psql")
        .args(["-d", "dsm_test_restore", "-f", dump_path.to_str().unwrap()])
        .output()
        .expect("psql restore failed");

    assert!(restore_output.status.success());

    // Cleanup
    let _ = Command::new("dropdb").arg("dsm_test_restore").output();
}

#[test]
fn test_gzip_compression_roundtrip() {
    // This can run without external databases
    use flate2::write::GzEncoder;
    use flate2::read::GzDecoder;
    use flate2::Compression;
    use std::io::{Read, Write};

    let original = b"CREATE TABLE test (id INT); INSERT INTO test VALUES (1);";

    // Compress
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(original).unwrap();
    let compressed = encoder.finish().unwrap();

    assert!(compressed.len() < original.len());

    // Decompress
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).unwrap();

    assert_eq!(decompressed.as_bytes(), original);
}
```

### 2.4 CI vs Manual Testing

| Test Category | CI | Manual | Notes |
|---|---|---|---|
| Unit tests (SQLite in-memory, command building, compression) | Yes | -- | Fast, no external deps |
| `cargo clippy`, `cargo fmt --check` | Yes | -- | Lint + format checks |
| Integration: dump/restore against real DB | No | Yes | Requires local MySQL/PostgreSQL |
| SSH tunnel tests | No | Yes | Requires SSH server access |
| Keychain integration | No | Yes | Requires macOS Keychain access |
| Full E2E (UI + backend) | No | Yes | Phase 2 consideration |

---

## 3. Frontend Testing with Vitest

### 3.1 Installation

```bash
npm install -D vitest @vue/test-utils happy-dom @pinia/testing
```

### 3.2 Vitest Configuration

Update `vite.config.ts` to include Vitest configuration:

```typescript
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    globals: true,
    environment: "happy-dom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,vue}"],
      exclude: ["src/test/**", "src/**/*.d.ts"],
    },
  },
}));
```

### 3.3 Test Setup File

```typescript
// src/test/setup.ts

import { vi, afterEach } from "vitest";
import { config } from "@vue/test-utils";

// Mock Tauri IPC globally
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Provide default stubs for router-link
config.global.stubs = {
  RouterLink: {
    template: '<a><slot /></a>',
  },
};

afterEach(() => {
  vi.restoreAllMocks();
});
```

### 3.4 Mocking Tauri Invoke Calls

Two approaches — use whichever fits the test:

**Approach A: vi.mock (module-level, defined in setup.ts)**

```typescript
// In individual test files
import { vi, describe, it, expect } from "vitest";
import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

describe("profile operations", () => {
  it("fetches profiles from backend", async () => {
    const mockProfiles = [
      { id: "p1", name: "dev-db", project: "myapp", db_type: "postgresql" },
    ];
    mockInvoke.mockResolvedValueOnce(mockProfiles);

    // Call your store action or composable
    const result = await invoke("profile_list");
    expect(result).toEqual(mockProfiles);
    expect(mockInvoke).toHaveBeenCalledWith("profile_list");
  });
});
```

**Approach B: mockIPC from @tauri-apps/api/mocks (more realistic)**

```typescript
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeAll, describe, it, expect } from "vitest";
import { randomFillSync } from "crypto";

// WebCrypto polyfill for happy-dom
beforeAll(() => {
  Object.defineProperty(window, "crypto", {
    value: {
      getRandomValues: (buffer: ArrayBufferView) => randomFillSync(buffer),
    },
  });
});

afterEach(() => {
  clearMocks();
});

describe("snapshot commands", () => {
  it("creates a snapshot", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "snapshot_create") {
        return {
          id: "snap-1",
          name: (args as Record<string, unknown>).name,
          size_bytes: 1024,
        };
      }
    });

    const result = await invoke("snapshot_create", {
      profile_id: "p1",
      name: "before-migration",
    });

    expect(result).toMatchObject({ id: "snap-1", name: "before-migration" });
  });
});
```

### 3.5 Testing Pinia Stores

```typescript
// src/stores/__tests__/profileStore.spec.ts

import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useProfileStore } from "../profileStore";
import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

describe("useProfileStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("loads profiles from backend", async () => {
    const mockProfiles = [
      { id: "p1", name: "dev-db", project: "myapp", db_type: "postgresql", database_name: "mydb" },
    ];
    mockInvoke.mockResolvedValueOnce(mockProfiles);

    const store = useProfileStore();
    await store.loadProfiles();

    expect(store.profiles).toEqual(mockProfiles);
    expect(mockInvoke).toHaveBeenCalledWith("profile_list");
  });

  it("creates a profile", async () => {
    mockInvoke.mockResolvedValueOnce(undefined); // profile_create returns void
    mockInvoke.mockResolvedValueOnce([]); // profile_list after create

    const store = useProfileStore();
    await store.createProfile({
      project: "myapp",
      name: "staging",
      db_type: "mysql",
      database_name: "staging_db",
    });

    expect(mockInvoke).toHaveBeenCalledWith("profile_create", expect.objectContaining({
      name: "staging",
    }));
  });

  it("handles backend errors gracefully", async () => {
    mockInvoke.mockRejectedValueOnce("Connection refused");

    const store = useProfileStore();
    await store.loadProfiles();

    expect(store.error).toBe("Connection refused");
    expect(store.profiles).toEqual([]);
  });
});
```

### 3.6 Testing Vue Components with Vue Test Utils

```typescript
// src/components/__tests__/ProfileCard.spec.ts

import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import { createTestingPinia } from "@pinia/testing";
import ProfileCard from "../ProfileCard.vue";

describe("ProfileCard", () => {
  it("renders profile name and database type", () => {
    const wrapper = mount(ProfileCard, {
      props: {
        profile: {
          id: "p1",
          name: "dev-db",
          project: "myapp",
          db_type: "postgresql",
          database_name: "mydb",
          host: "localhost",
          port: 5432,
        },
      },
      global: {
        plugins: [
          createTestingPinia({
            createSpy: vi.fn,
          }),
        ],
      },
    });

    expect(wrapper.text()).toContain("dev-db");
    expect(wrapper.text()).toContain("postgresql");
  });

  it("emits delete event on button click", async () => {
    const wrapper = mount(ProfileCard, {
      props: {
        profile: { id: "p1", name: "test", project: "p", db_type: "mysql", database_name: "db" },
      },
      global: {
        plugins: [createTestingPinia({ createSpy: vi.fn })],
      },
    });

    await wrapper.find("[data-testid='delete-btn']").trigger("click");
    expect(wrapper.emitted("delete")).toBeTruthy();
  });
});
```

### 3.7 Test File Structure

```text
src/
    test/
        setup.ts                         # global test setup
    stores/
        profileStore.ts
        __tests__/
            profileStore.spec.ts
        snapshotStore.ts
        __tests__/
            snapshotStore.spec.ts
    components/
        ProfileCard.vue
        __tests__/
            ProfileCard.spec.ts
        SnapshotTable.vue
        __tests__/
            SnapshotTable.spec.ts
    composables/
        useConnectionTest.ts
        __tests__/
            useConnectionTest.spec.ts
```

### 3.8 npm Scripts

Add to `package.json`:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "lint": "eslint .",
    "lint:fix": "eslint . --fix",
    "format": "prettier --write \"src/**/*.{ts,vue,css,html}\"",
    "format:check": "prettier --check \"src/**/*.{ts,vue,css,html}\"",
    "typecheck": "vue-tsc --noEmit"
  }
}
```

---

## 4. Linting and Formatting

### 4.1 ESLint v9 Flat Config

Install dependencies:

```bash
npm install -D eslint @eslint/js eslint-plugin-vue typescript-eslint globals eslint-config-prettier
```

Create `eslint.config.js`:

```javascript
import js from "@eslint/js";
import eslintPluginVue from "eslint-plugin-vue";
import globals from "globals";
import typescriptEslint from "typescript-eslint";
import eslintConfigPrettier from "eslint-config-prettier";

export default typescriptEslint.config(
  {
    ignores: [
      "dist/**",
      "src-tauri/**",
      "*.d.ts",
      "node_modules/**",
    ],
  },
  {
    extends: [
      js.configs.recommended,
      ...typescriptEslint.configs.recommended,
      ...eslintPluginVue.configs["flat/recommended"],
    ],
    files: ["**/*.{ts,vue}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: globals.browser,
      parserOptions: {
        parser: typescriptEslint.parser,
      },
    },
    rules: {
      // Vue 3 specific
      "vue/multi-word-component-names": "off",
      "vue/no-unused-vars": "error",
      "vue/component-tags-order": [
        "error",
        { order: ["script", "template", "style"] },
      ],
      "vue/define-macros-order": [
        "error",
        { order: ["defineProps", "defineEmits"] },
      ],
      "vue/block-lang": [
        "error",
        { script: { lang: "ts" } },
      ],

      // TypeScript
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/no-explicit-any": "warn",
    },
  },
  eslintConfigPrettier,
);
```

### 4.2 Prettier Config

Install dependencies:

```bash
npm install -D prettier prettier-plugin-tailwindcss
```

Create `.prettierrc`:

```json
{
  "semi": true,
  "singleQuote": false,
  "tabWidth": 2,
  "trailingComma": "all",
  "printWidth": 100,
  "bracketSameLine": false,
  "arrowParens": "always",
  "endOfLine": "lf",
  "vueIndentScriptAndStyle": true,
  "plugins": ["prettier-plugin-tailwindcss"],
  "tailwindStylesheet": "./src/styles/main.css"
}
```

Create `.prettierignore`:

```text
dist/
src-tauri/target/
node_modules/
package-lock.json
*.md
```

### 4.3 rustfmt Configuration

Create `src-tauri/rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
```

### 4.4 Clippy Configuration

Create `src-tauri/clippy.toml` (or configure via `Cargo.toml`):

In `src-tauri/Cargo.toml`, add a `[lints]` section:

```toml
[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
# Allow common patterns
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
```

Run clippy locally:

```bash
cd src-tauri && cargo clippy -- -D warnings
```

Run rustfmt locally:

```bash
cd src-tauri && cargo fmt --check
```

---

## 5. VS Code Recommended Extensions

Update `.vscode/extensions.json`:

```json
{
  "recommendations": [
    "tauri-apps.tauri-vscode",
    "rust-lang.rust-analyzer",
    "Vue.volar",
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode",
    "bradlc.vscode-tailwindcss",
    "formulahendry.auto-rename-tag",
    "EditorConfig.EditorConfig"
  ]
}
```

| Extension | Purpose |
|---|---|
| `tauri-apps.tauri-vscode` | Tauri command palette, config schema |
| `rust-lang.rust-analyzer` | Rust LSP, inline hints, code actions |
| `Vue.volar` | Vue 3 language support (replaces Vetur) |
| `dbaeumer.vscode-eslint` | ESLint integration with flat config support |
| `esbenp.prettier-vscode` | Prettier formatting on save |
| `bradlc.vscode-tailwindcss` | Tailwind CSS IntelliSense and class sorting |
| `formulahendry.auto-rename-tag` | Auto-rename paired HTML/Vue tags |
| `EditorConfig.EditorConfig` | Consistent editor settings across IDEs |

Add `.vscode/settings.json` recommendations:

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },
  "eslint.useFlatConfig": true,
  "typescript.tsdk": "node_modules/typescript/lib",
  "tailwindCSS.experimental.classRegex": []
}
```

---

## 6. Git Hooks with Lefthook

### 6.1 Installation

```bash
npm install -D lefthook
npx lefthook install
```

### 6.2 Configuration

Create `lefthook.yml` at the project root:

```yaml
pre-commit:
  parallel: true
  jobs:
    - name: eslint
      glob: "*.{ts,vue}"
      run: npx eslint {staged_files}

    - name: prettier
      glob: "*.{ts,vue,css,html,json}"
      run: npx prettier --check {staged_files}

    - name: typecheck
      run: npx vue-tsc --noEmit

    - name: rustfmt
      root: "src-tauri/"
      glob: "*.rs"
      run: cargo fmt --check

    - name: clippy
      root: "src-tauri/"
      glob: "*.rs"
      run: cargo clippy -- -D warnings

pre-push:
  parallel: true
  jobs:
    - name: frontend-tests
      run: npm run test

    - name: rust-tests
      root: "src-tauri/"
      run: cargo test
```

### 6.3 Developer Workflow

After cloning the repo:

```bash
npm install        # installs lefthook as a dev dependency
npx lefthook install  # sets up git hooks (runs automatically via postinstall)
```

Add `postinstall` to `package.json` for automatic setup:

```json
{
  "scripts": {
    "postinstall": "lefthook install"
  }
}
```

---

## 7. GitHub Actions CI

### 7.1 CI Workflow

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  rust:
    name: Rust checks
    runs-on: macos-latest
    defaults:
      run:
        working-directory: src-tauri
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo registry and build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            src-tauri/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('src-tauri/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: Install Tauri system dependencies
        run: |
          # macOS includes most deps already; Xcode CLI tools provide what's needed
          rustup target add aarch64-apple-darwin

      - name: Check formatting
        run: cargo fmt --check

      - name: Clippy lint
        run: cargo clippy -- -D warnings

      - name: Compile check
        run: cargo check

      - name: Run tests
        run: cargo test

  frontend:
    name: Frontend checks
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: "npm"

      - name: Install dependencies
        run: npm ci

      - name: ESLint
        run: npm run lint

      - name: Prettier check
        run: npm run format:check

      - name: TypeScript type check
        run: npm run typecheck

      - name: Run tests
        run: npm run test
```

### 7.2 Workflow Explanation

The CI runs two parallel jobs:

**Rust job:**
1. `cargo fmt --check` — ensures consistent Rust formatting
2. `cargo clippy -- -D warnings` — catches common mistakes, treats warnings as errors
3. `cargo check` — fast compilation check without producing binaries
4. `cargo test` — runs all unit tests (integration tests excluded unless `DSM_INTEGRATION` is set)

**Frontend job:**
1. `npm run lint` — ESLint with Vue 3 + TypeScript rules
2. `npm run format:check` — Prettier formatting verification
3. `npm run typecheck` — `vue-tsc --noEmit` for full type checking
4. `npm run test` — Vitest test suite

### 7.3 Optional: Branch Protection

Configure GitHub branch protection for `main`:
- Require status checks to pass (both `Rust checks` and `Frontend checks`)
- Require branches to be up to date before merging
- Require pull request reviews (optional for solo development)

---

## Summary of All Dependencies

### npm devDependencies to Add

```bash
npm install -D \
  vitest @vue/test-utils happy-dom @pinia/testing \
  eslint @eslint/js eslint-plugin-vue typescript-eslint globals eslint-config-prettier \
  prettier prettier-plugin-tailwindcss \
  lefthook \
  vue-tsc
```

### Cargo dev-dependencies to Add

```toml
[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### New Config Files to Create

| File | Purpose |
|---|---|
| `eslint.config.js` | ESLint v9 flat config |
| `.prettierrc` | Prettier configuration |
| `.prettierignore` | Files to skip formatting |
| `lefthook.yml` | Git hook definitions |
| `src-tauri/rustfmt.toml` | Rust formatting rules |
| `.vscode/settings.json` | Editor settings |
| `.github/workflows/ci.yml` | CI pipeline |
| `src/test/setup.ts` | Vitest global test setup |
| `scripts/test-db-setup.sh` | Integration test database setup |
| `scripts/test-db-teardown.sh` | Integration test database teardown |

---

*Plan drafted 19 February 2026*

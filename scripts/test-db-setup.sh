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

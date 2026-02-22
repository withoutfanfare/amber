#!/usr/bin/env bash
# scripts/test-db-teardown.sh — remove test databases

set -euo pipefail

echo "Dropping PostgreSQL test database..."
dropdb dsm_test_db 2>/dev/null || true

echo "Dropping MySQL test database..."
mysql -u root -e "DROP DATABASE IF EXISTS dsm_test_db;"

echo "Test databases cleaned up."

#!/bin/bash
# Weekly backup restoration verification

set -euo pipefail

BACKUP_BUCKET="s3://scavenger-backups"
TEST_DB="scavenger_restore_test"
DB_HOST="${DB_HOST:-localhost}"

echo "Starting backup restoration verification..."

# Get latest backup
LATEST_BACKUP=$(aws s3 ls "$BACKUP_BUCKET/database/" --recursive | sort | tail -n 1 | awk '{print $4}')

if [ -z "$LATEST_BACKUP" ]; then
    echo "✗ No backups found"
    exit 1
fi

echo "Testing restore from: $LATEST_BACKUP"

# Download backup
aws s3 cp "$BACKUP_BUCKET/$LATEST_BACKUP" /tmp/restore_test.sql.gz

# Create test database
psql -h "$DB_HOST" -U postgres -c "DROP DATABASE IF EXISTS $TEST_DB;" || true
psql -h "$DB_HOST" -U postgres -c "CREATE DATABASE $TEST_DB;"

# Restore backup
gunzip -c /tmp/restore_test.sql.gz | psql -h "$DB_HOST" -U postgres -d "$TEST_DB"

# Verify data integrity
RECORD_COUNT=$(psql -h "$DB_HOST" -U postgres -d "$TEST_DB" -t -c "SELECT COUNT(*) FROM participants;")

if [ "$RECORD_COUNT" -gt 0 ]; then
    echo "✓ Restore verification passed (found $RECORD_COUNT records)"
    psql -h "$DB_HOST" -U postgres -c "DROP DATABASE $TEST_DB;"
    rm /tmp/restore_test.sql.gz
    exit 0
else
    echo "✗ Restore verification failed (no records found)"
    exit 1
fi

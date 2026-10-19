#!/bin/bash
# Database backup script with verification

set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/backups}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
DB_HOST="${DB_HOST:-localhost}"
DB_NAME="${DB_NAME:-scavenger}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/db_backup_$TIMESTAMP.sql.gz"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Perform backup
echo "Starting database backup..."
pg_dump -h "$DB_HOST" -U postgres "$DB_NAME" | gzip > "$BACKUP_FILE"

# Verify backup
echo "Verifying backup integrity..."
if gunzip -t "$BACKUP_FILE" > /dev/null 2>&1; then
    echo "✓ Backup verified successfully"
    ls -lh "$BACKUP_FILE"
else
    echo "✗ Backup verification failed"
    rm "$BACKUP_FILE"
    exit 1
fi

# Upload to S3
echo "Uploading to S3..."
aws s3 cp "$BACKUP_FILE" "s3://scavenger-backups/database/$TIMESTAMP/" \
    --region us-east-1 \
    --sse AES256

# Cleanup old backups
echo "Cleaning up old backups..."
find "$BACKUP_DIR" -name "db_backup_*.sql.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed successfully"

#!/bin/bash
# Data backup script

BACKUP_DIR="backup/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

echo "💾 Creating backup in $BACKUP_DIR..."

# Backup database
docker exec postgres pg_dump -U postgres cerberus > "$BACKUP_DIR/database.sql"

# Backup configuration
cp infrastructure/.env "$BACKUP_DIR/config.env"

# Backup Qdrant data
docker exec qdrant tar -czf /tmp/qdrant_backup.tar.gz /qdrant/storage
docker cp qdrant:/tmp/qdrant_backup.tar.gz "$BACKUP_DIR/"

echo "✅ Backup completed: $BACKUP_DIR"

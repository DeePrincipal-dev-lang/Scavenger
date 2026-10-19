resource "aws_s3_bucket" "backups" {
  bucket = "scavenger-backups-${var.environment}-${var.account_id}"

  tags = {
    Name = "scavenger-backups-${var.environment}"
  }
}

resource "aws_s3_bucket_versioning" "backups" {
  bucket = aws_s3_bucket.backups.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "backups" {
  bucket = aws_s3_bucket.backups.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "backups" {
  bucket = aws_s3_bucket.backups.id

  rule {
    id     = "backup-retention"
    status = "Enabled"

    expiration {
      days = var.retention_days
    }

    noncurrent_version_expiration {
      noncurrent_days = 7
    }
  }
}

resource "aws_s3_bucket_public_access_block" "backups" {
  bucket                  = aws_s3_bucket.backups.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_cloudwatch_metric_alarm" "backup_failure" {
  alarm_name          = "scavenger-backup-failure-${var.environment}"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = "1"
  metric_name         = "BackupSuccess"
  namespace           = "Scavenger/Backups"
  period              = "86400"
  statistic           = "Sum"
  threshold           = "1"
  treat_missing_data  = "breaching"
  alarm_description   = "Triggers if no successful backup in 24 hours"
  alarm_actions       = var.alarm_sns_arns

  tags = {
    Name = "scavenger-backup-failure-${var.environment}"
  }
}

resource "aws_cloudwatch_metric_alarm" "backup_verification_failure" {
  alarm_name          = "scavenger-backup-verify-failure-${var.environment}"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = "1"
  metric_name         = "BackupVerificationSuccess"
  namespace           = "Scavenger/Backups"
  period              = "86400"
  statistic           = "Sum"
  threshold           = "1"
  treat_missing_data  = "breaching"
  alarm_description   = "Triggers if backup verification failed in last 24 hours"
  alarm_actions       = var.alarm_sns_arns

  tags = {
    Name = "scavenger-backup-verify-failure-${var.environment}"
  }
}

resource "aws_cloudwatch_dashboard" "backups" {
  dashboard_name = "scavenger-backups-${var.environment}"

  dashboard_body = jsonencode({
    widgets = [
      {
        type   = "metric"
        width  = 12
        height = 6
        properties = {
          title  = "Backup Success Rate"
          period = 86400
          stat   = "Sum"
          metrics = [
            ["Scavenger/Backups", "BackupSuccess"],
            ["Scavenger/Backups", "BackupVerificationSuccess"]
          ]
        }
      },
      {
        type   = "alarm"
        width  = 12
        height = 6
        properties = {
          title  = "Backup Alarms"
          alarms = [
            aws_cloudwatch_metric_alarm.backup_failure.arn,
            aws_cloudwatch_metric_alarm.backup_verification_failure.arn
          ]
        }
      }
    ]
  })
}

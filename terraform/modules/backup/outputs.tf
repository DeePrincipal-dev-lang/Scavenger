output "bucket_name" {
  value = aws_s3_bucket.backups.bucket
}

output "bucket_arn" {
  value = aws_s3_bucket.backups.arn
}

output "backup_failure_alarm_arn" {
  value = aws_cloudwatch_metric_alarm.backup_failure.arn
}

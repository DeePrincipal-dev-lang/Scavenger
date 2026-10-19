variable "environment" {
  type = string
}

variable "account_id" {
  type        = string
  description = "AWS account ID used to create a globally unique S3 bucket name"
}

variable "retention_days" {
  type        = number
  default     = 30
  description = "Days to retain backups in S3"
}

variable "alarm_sns_arns" {
  type        = list(string)
  default     = []
  description = "SNS topic ARNs to notify on backup alarm"
}

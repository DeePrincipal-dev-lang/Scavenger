environment = "staging"
region      = "us-east-1"

# VPC
vpc_cidr = "10.1.0.0/16"

# Database
db_instance_class   = "db.t3.small"
db_storage_gb       = 100
db_backup_retention = 14

# ECS
container_memory = 1024
container_cpu    = 512
desired_count    = 2

# Load Balancer
enable_load_balancer = true

# Monitoring
enable_detailed_monitoring = true
log_retention_days        = 14

# Tags
tags = {
  Environment = "staging"
  ManagedBy   = "Terraform"
  Project     = "Scavenger"
}

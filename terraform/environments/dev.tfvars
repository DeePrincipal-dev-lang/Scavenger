environment = "dev"
region      = "us-east-1"

# VPC
vpc_cidr = "10.0.0.0/16"

# Database
db_instance_class   = "db.t3.micro"
db_storage_gb       = 20
db_backup_retention = 7

# ECS
container_memory = 512
container_cpu    = 256
desired_count    = 1

# Load Balancer
enable_load_balancer = false

# Monitoring
enable_detailed_monitoring = true
log_retention_days        = 7

# Tags
tags = {
  Environment = "dev"
  ManagedBy   = "Terraform"
  Project     = "Scavenger"
}

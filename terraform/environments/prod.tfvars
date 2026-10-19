environment = "prod"
region      = "us-east-1"

# VPC
vpc_cidr = "10.2.0.0/16"

# Database
db_instance_class    = "db.r6i.large"
db_storage_gb        = 500
db_backup_retention  = 90
db_multi_az          = true
db_enhanced_backup   = true

# ECS
container_memory = 2048
container_cpu    = 1024
desired_count    = 3

# Load Balancer
enable_load_balancer = true

# Monitoring
enable_detailed_monitoring = true
log_retention_days        = 90

# Auto Scaling
enable_autoscaling = true
autoscaling_min_capacity = 2
autoscaling_max_capacity = 10
autoscaling_target_cpu   = 70

# Tags
tags = {
  Environment = "prod"
  ManagedBy   = "Terraform"
  Project     = "Scavenger"
  CostCenter  = "Platform"
}

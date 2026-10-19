variable "primary_region" {
  type        = string
  description = "Primary AWS region"
}

variable "secondary_region" {
  type        = string
  description = "Secondary (failover) AWS region"
}

variable "environment" {
  type        = string
  description = "Environment name"
}

variable "vpc_cidr_primary" {
  type        = string
  default     = "10.0.0.0/16"
  description = "VPC CIDR for primary region"
}

variable "vpc_cidr_secondary" {
  type        = string
  default     = "10.1.0.0/16"
  description = "VPC CIDR for secondary region"
}

variable "private_subnet_ids_primary" {
  type        = list(string)
  description = "Private subnet IDs in primary region"
}

variable "private_subnet_ids_secondary" {
  type        = list(string)
  description = "Private subnet IDs in secondary region"
}

variable "public_subnet_ids_primary" {
  type        = list(string)
  description = "Public subnet IDs in primary region"
}

variable "public_subnet_ids_secondary" {
  type        = list(string)
  description = "Public subnet IDs in secondary region"
}

variable "db_instance_class" {
  type        = string
  default     = "db.r6i.large"
  description = "RDS instance class"
}

variable "db_name" {
  type        = string
  default     = "scavenger"
  description = "Database name"
}

variable "container_image" {
  type        = string
  description = "Docker image URI"
}

variable "container_port" {
  type        = number
  default     = 8080
}

variable "ecs_min_capacity" {
  type        = number
  default     = 2
}

variable "ecs_max_capacity" {
  type        = number
  default     = 8
}

variable "certificate_arn_primary" {
  type        = string
  description = "ACM certificate ARN for primary region"
}

variable "certificate_arn_secondary" {
  type        = string
  description = "ACM certificate ARN for secondary region"
}

variable "domain_name" {
  type        = string
  description = "Route53 domain name"
}

variable "health_check_interval" {
  type        = number
  default     = 30
  description = "Health check interval in seconds"
}

variable "health_check_failure_threshold" {
  type        = number
  default     = 3
  description = "Health check failure threshold"
}

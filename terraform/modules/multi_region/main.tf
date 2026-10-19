terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  alias  = "primary"
  region = var.primary_region
}

provider "aws" {
  alias  = "secondary"
  region = var.secondary_region
}

# VPC Peering
resource "aws_vpc_peering_connection" "cross_region" {
  provider      = aws.primary
  vpc_id        = var.vpc_id_primary
  peer_vpc_id   = var.vpc_id_secondary
  peer_region   = var.secondary_region
  peer_owner_id = data.aws_caller_identity.current.account_id
  auto_accept   = false

  tags = {
    Name        = "scavenger-cross-region-${var.environment}"
    Environment = var.environment
    Project     = "scavenger"
  }
}

resource "aws_vpc_peering_connection_accepter" "cross_region" {
  provider                  = aws.secondary
  vpc_peering_connection_id = aws_vpc_peering_connection.cross_region.id
  auto_accept               = true

  tags = {
    Name        = "scavenger-cross-region-${var.environment}"
    Environment = var.environment
    Project     = "scavenger"
  }
}

# Route tables for cross-region traffic
resource "aws_route" "primary_to_secondary" {
  provider                  = aws.primary
  count                     = length(var.private_subnet_ids_primary)
  route_table_id            = element(var.private_route_table_ids_primary, count.index)
  destination_cidr_block    = var.vpc_cidr_secondary
  vpc_peering_connection_id = aws_vpc_peering_connection.cross_region.id
}

resource "aws_route" "secondary_to_primary" {
  provider                  = aws.secondary
  count                     = length(var.private_subnet_ids_secondary)
  route_table_id            = element(var.private_route_table_ids_secondary, count.index)
  destination_cidr_block    = var.vpc_cidr_primary
  vpc_peering_connection_id = aws_vpc_peering_connection.cross_region.id
}

# Primary region ALB
resource "aws_lb" "primary" {
  provider           = aws.primary
  name               = "scavenger-primary-${var.environment}"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb_primary.id]
  subnets            = var.public_subnet_ids_primary

  enable_deletion_protection = var.environment == "prod"

  tags = {
    Name        = "scavenger-primary-alb-${var.environment}"
    Environment = var.environment
    Project     = "scavenger"
  }
}

resource "aws_lb_target_group" "primary" {
  provider     = aws.primary
  name         = "scavenger-primary-${var.environment}"
  port         = var.container_port
  protocol     = "HTTP"
  vpc_id       = var.vpc_id_primary
  target_type  = "ip"

  health_check {
    enabled             = true
    path                = "/health"
    interval            = var.health_check_interval
    timeout             = 5
    healthy_threshold   = 2
    unhealthy_threshold = var.health_check_failure_threshold
    matcher             = "200"
  }

  tags = {
    Environment = var.environment
    Project     = "scavenger"
  }
}

resource "aws_lb_listener" "primary_https" {
  provider          = aws.primary
  load_balancer_arn = aws_lb.primary.arn
  port              = 443
  protocol          = "HTTPS"
  certificate_arn   = var.certificate_arn_primary
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.primary.arn
  }
}

# Secondary region ALB
resource "aws_lb" "secondary" {
  provider           = aws.secondary
  name               = "scavenger-secondary-${var.environment}"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb_secondary.id]
  subnets            = var.public_subnet_ids_secondary

  enable_deletion_protection = var.environment == "prod"

  tags = {
    Name        = "scavenger-secondary-alb-${var.environment}"
    Environment = var.environment
    Project     = "scavenger"
  }
}

resource "aws_lb_target_group" "secondary" {
  provider     = aws.secondary
  name         = "scavenger-secondary-${var.environment}"
  port         = var.container_port
  protocol     = "HTTP"
  vpc_id       = var.vpc_id_secondary
  target_type  = "ip"

  health_check {
    enabled             = true
    path                = "/health"
    interval            = var.health_check_interval
    timeout             = 5
    healthy_threshold   = 2
    unhealthy_threshold = var.health_check_failure_threshold
    matcher             = "200"
  }

  tags = {
    Environment = var.environment
    Project     = "scavenger"
  }
}

resource "aws_lb_listener" "secondary_https" {
  provider          = aws.secondary
  load_balancer_arn = aws_lb.secondary.arn
  port              = 443
  protocol          = "HTTPS"
  certificate_arn   = var.certificate_arn_secondary
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.secondary.arn
  }
}

# Route53 DNS failover
resource "aws_route53_health_check" "primary" {
  provider                  = aws.primary
  fqdn                      = aws_lb.primary.dns_name
  port                      = 443
  type                      = "HTTPS"
  resource_path             = "/health"
  failure_threshold         = var.health_check_failure_threshold
  request_interval          = var.health_check_interval
  measure_latency           = true
  child_health_threshold    = 2

  tags = {
    Name        = "scavenger-primary-health-${var.environment}"
    Environment = var.environment
    Project     = "scavenger"
  }
}

resource "aws_route53_health_check" "secondary" {
  provider                  = aws.secondary
  fqdn                      = aws_lb.secondary.dns_name
  port                      = 443
  type                      = "HTTPS"
  resource_path             = "/health"
  failure_threshold         = var.health_check_failure_threshold
  request_interval          = var.health_check_interval
  measure_latency           = true
  child_health_threshold    = 2

  tags = {
    Name        = "scavenger-secondary-health-${var.environment}"
    Environment = var.environment
    Project     = "scavenger"
  }
}

# Security groups
resource "aws_security_group" "alb_primary" {
  provider    = aws.primary
  name        = "scavenger-alb-primary-${var.environment}"
  description = "Security group for primary ALB"
  vpc_id      = var.vpc_id_primary

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Environment = var.environment
    Project     = "scavenger"
  }
}

resource "aws_security_group" "alb_secondary" {
  provider    = aws.secondary
  name        = "scavenger-alb-secondary-${var.environment}"
  description = "Security group for secondary ALB"
  vpc_id      = var.vpc_id_secondary

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Environment = var.environment
    Project     = "scavenger"
  }
}

# Route53 failover record set
resource "aws_route53_record" "app" {
  zone_id = var.route53_zone_id
  name    = var.domain_name
  type    = "A"

  set_identifier = "primary"
  failover_routing_policy {
    type = "PRIMARY"
  }

  alias {
    name                   = aws_lb.primary.dns_name
    zone_id                = aws_lb.primary.zone_id
    evaluate_target_health = true
  }

  health_check_id = aws_route53_health_check.primary.id
}

resource "aws_route53_record" "app_secondary" {
  zone_id = var.route53_zone_id
  name    = var.domain_name
  type    = "A"

  set_identifier = "secondary"
  failover_routing_policy {
    type = "SECONDARY"
  }

  alias {
    name                   = aws_lb.secondary.dns_name
    zone_id                = aws_lb.secondary.zone_id
    evaluate_target_health = true
  }

  health_check_id = aws_route53_health_check.secondary.id
}

# Cross-region data replication - S3 bucket
resource "aws_s3_bucket" "replication" {
  provider   = aws.primary
  bucket     = "scavenger-replication-${var.environment}-primary"
  force_destroy = var.environment != "prod"
}

resource "aws_s3_bucket_versioning" "replication" {
  provider = aws.primary
  bucket   = aws_s3_bucket.replication.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_replication_configuration" "cross_region" {
  provider   = aws.primary
  bucket     = aws_s3_bucket.replication.id
  role       = aws_iam_role.replication.arn

  rule {
    status = "Enabled"
    destination {
      bucket        = aws_s3_bucket.replication_secondary.arn
      storage_class = "STANDARD_IA"
    }
  }
}

resource "aws_s3_bucket" "replication_secondary" {
  provider   = aws.secondary
  bucket     = "scavenger-replication-${var.environment}-secondary"
  force_destroy = var.environment != "prod"
}

resource "aws_s3_bucket_versioning" "replication_secondary" {
  provider = aws.secondary
  bucket   = aws_s3_bucket.replication_secondary.id
  versioning_configuration {
    status = "Enabled"
  }
}

# IAM role for replication
resource "aws_iam_role" "replication" {
  provider = aws.primary
  name     = "scavenger-replication-${var.environment}"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "s3.amazonaws.com"
        }
      }
    ]
  })
}

# Outputs
output "primary_alb_dns" {
  value = aws_lb.primary.dns_name
}

output "secondary_alb_dns" {
  value = aws_lb.secondary.dns_name
}

output "primary_alb_zone_id" {
  value = aws_lb.primary.zone_id
}

output "secondary_alb_zone_id" {
  value = aws_lb.secondary.zone_id
}

output "primary_target_group_arn" {
  value = aws_lb_target_group.primary.arn
}

output "secondary_target_group_arn" {
  value = aws_lb_target_group.secondary.arn
}

output "replication_bucket_primary" {
  value = aws_s3_bucket.replication.id
}

output "replication_bucket_secondary" {
  value = aws_s3_bucket.replication_secondary.id
}

output "vpc_peering_id" {
  value = aws_vpc_peering_connection.cross_region.id
}

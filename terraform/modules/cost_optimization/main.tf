# ── Auto-scaling for ECS ──────────────────────────────────────────────────────

resource "aws_appautoscaling_target" "ecs" {
  max_capacity       = var.ecs_max_capacity
  min_capacity       = var.ecs_min_capacity
  resource_id        = "service/${var.ecs_cluster_name}/${var.ecs_service_name}"
  scalable_dimension = "ecs:service:DesiredCount"
  service_namespace  = "ecs"
}

resource "aws_appautoscaling_policy" "cpu" {
  name               = "scavenger-cpu-scaling-${var.environment}"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.ecs.resource_id
  scalable_dimension = aws_appautoscaling_target.ecs.scalable_dimension
  service_namespace  = aws_appautoscaling_target.ecs.service_namespace

  target_tracking_scaling_policy_configuration {
    target_value       = 70
    scale_in_cooldown  = 300
    scale_out_cooldown = 60

    predefined_metric_specification {
      predefined_metric_type = "ECSServiceAverageCPUUtilization"
    }
  }
}

resource "aws_appautoscaling_policy" "memory" {
  name               = "scavenger-memory-scaling-${var.environment}"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.ecs.resource_id
  scalable_dimension = aws_appautoscaling_target.ecs.scalable_dimension
  service_namespace  = aws_appautoscaling_target.ecs.service_namespace

  target_tracking_scaling_policy_configuration {
    target_value       = 80
    scale_in_cooldown  = 300
    scale_out_cooldown = 60

    predefined_metric_specification {
      predefined_metric_type = "ECSServiceAverageMemoryUtilization"
    }
  }
}

# ── Spot capacity provider ────────────────────────────────────────────────────

resource "aws_ecs_capacity_provider" "spot" {
  name = "scavenger-spot-${var.environment}"

  auto_scaling_group_provider {
    auto_scaling_group_arn         = var.spot_asg_arn
    managed_termination_protection = "DISABLED"

    managed_scaling {
      status                    = "ENABLED"
      target_capacity           = 80
      minimum_scaling_step_size = 1
      maximum_scaling_step_size = 10
    }
  }
}

resource "aws_ecs_cluster_capacity_providers" "this" {
  cluster_name = var.ecs_cluster_name

  capacity_providers = ["FARGATE", "FARGATE_SPOT", aws_ecs_capacity_provider.spot.name]

  default_capacity_provider_strategy {
    capacity_provider = "FARGATE_SPOT"
    weight            = 3
    base              = 0
  }

  default_capacity_provider_strategy {
    capacity_provider = "FARGATE"
    weight            = 1
    base              = 1
  }
}

# ── Budget alerts ─────────────────────────────────────────────────────────────

resource "aws_budgets_budget" "monthly" {
  name         = "scavenger-monthly-${var.environment}"
  budget_type  = "COST"
  limit_amount = var.monthly_budget_usd
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  cost_filter {
    name   = "TagKeyValue"
    values = ["user:Project$scavenger"]
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 80
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = var.budget_alert_emails
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 100
    threshold_type             = "PERCENTAGE"
    notification_type          = "FORECASTED"
    subscriber_email_addresses = var.budget_alert_emails
  }
}

# ── Resource tagging policy ───────────────────────────────────────────────────

resource "aws_organizations_policy" "tagging" {
  count       = var.enable_org_tagging_policy ? 1 : 0
  name        = "scavenger-required-tags"
  type        = "TAG_POLICY"
  description = "Enforce required tags on Scavenger resources"

  content = jsonencode({
    tags = {
      Environment = {
        tag_key = { "@@assign" = "Environment" }
        tag_value = {
          "@@assign" = ["dev", "staging", "prod"]
        }
        enforced_for = {
          "@@assign" = ["ec2:instance", "rds:db", "ecs:cluster", "ecs:service"]
        }
      }
      Project = {
        tag_key = { "@@assign" = "Project" }
        tag_value = {
          "@@assign" = ["scavenger"]
        }
        enforced_for = {
          "@@assign" = ["ec2:instance", "rds:db", "ecs:cluster", "ecs:service"]
        }
      }
    }
  })
}

# ── RDS auto-stop for non-prod ────────────────────────────────────────────────

resource "aws_cloudwatch_event_rule" "rds_stop" {
  count               = var.environment != "prod" ? 1 : 0
  name                = "scavenger-rds-stop-${var.environment}"
  description         = "Stop RDS at night to save costs"
  schedule_expression = "cron(0 20 * * ? *)" # 8 PM UTC
}

resource "aws_cloudwatch_event_target" "rds_stop" {
  count    = var.environment != "prod" ? 1 : 0
  rule     = aws_cloudwatch_event_rule.rds_stop[0].name
  arn      = "arn:aws:ssm:${var.aws_region}::automation-definition/AWS-StopRdsInstance"
  role_arn = var.scheduler_role_arn

  input = jsonencode({
    InstanceId = [var.rds_instance_id]
  })
}

resource "aws_cloudwatch_event_rule" "rds_start" {
  count               = var.environment != "prod" ? 1 : 0
  name                = "scavenger-rds-start-${var.environment}"
  description         = "Start RDS in the morning"
  schedule_expression = "cron(0 7 * * ? *)" # 7 AM UTC
}

resource "aws_cloudwatch_event_target" "rds_start" {
  count    = var.environment != "prod" ? 1 : 0
  rule     = aws_cloudwatch_event_rule.rds_start[0].name
  arn      = "arn:aws:ssm:${var.aws_region}::automation-definition/AWS-StartRdsInstance"
  role_arn = var.scheduler_role_arn

  input = jsonencode({
    InstanceId = [var.rds_instance_id]
  })
}

# ── Cost anomaly detection ────────────────────────────────────────────────────

resource "aws_ce_anomaly_monitor" "scavenger" {
  name              = "scavenger-anomaly-monitor"
  monitor_type      = "DIMENSIONAL"
  monitor_dimension = "SERVICE"
}

resource "aws_ce_anomaly_subscription" "scavenger" {
  name      = "scavenger-anomaly-alerts"
  frequency = "DAILY"

  monitor_arn_list = [aws_ce_anomaly_monitor.scavenger.arn]

  subscriber {
    type    = "EMAIL"
    address = var.budget_alert_emails[0]
  }

  threshold_expression {
    dimension {
      key           = "ANOMALY_TOTAL_IMPACT_PERCENTAGE"
      values        = ["10"]
      match_options = ["GREATER_THAN_OR_EQUAL"]
    }
  }
}

# ── Cost monitoring dashboard ─────────────────────────────────────────────────

resource "aws_cloudwatch_dashboard" "cost" {
  dashboard_name = "scavenger-cost-${var.environment}"

  dashboard_body = jsonencode({
    widgets = [
      {
        type   = "metric"
        width  = 12
        height = 6
        properties = {
          title  = "ECS CPU Utilization"
          period = 300
          stat   = "Average"
          metrics = [
            ["AWS/ECS", "CPUUtilization", "ClusterName", var.ecs_cluster_name]
          ]
        }
      },
      {
        type   = "metric"
        width  = 12
        height = 6
        properties = {
          title  = "ECS Memory Utilization"
          period = 300
          stat   = "Average"
          metrics = [
            ["AWS/ECS", "MemoryUtilization", "ClusterName", var.ecs_cluster_name]
          ]
        }
      }
    ]
  })
}

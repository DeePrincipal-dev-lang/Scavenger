output "spot_capacity_provider_name" {
  value = aws_ecs_capacity_provider.spot.name
}

output "budget_name" {
  value = aws_budgets_budget.monthly.name
}

output "anomaly_monitor_arn" {
  value = aws_ce_anomaly_monitor.scavenger.arn
}

output "cost_dashboard_name" {
  value = aws_cloudwatch_dashboard.cost.dashboard_name
}

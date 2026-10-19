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

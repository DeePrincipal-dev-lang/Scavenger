# Cost Optimization Guide

## Overview
This guide provides strategies and best practices for optimizing costs across the Scavenger platform infrastructure.

## Cost Monitoring

### Dashboards
The Grafana cost monitoring dashboard provides real-time visibility into AWS spending:
- Monthly spend overview
- Cost breakdown by service
- Budget status and alerts
- Cost anomaly events
- Optimization recommendations

### Reports
Regular cost reports help track spending trends:
```bash
# Generate monthly cost report
./scripts/cost-report.sh

# Generate report for specific period
DAYS=90 ENVIRONMENT=prod ./scripts/cost-report.sh
```

## Cost Allocation

### Tagging Strategy
Resources are tagged for cost allocation:
- `Project=scavenger` - Identify project costs
- `Environment={dev,staging,prod}` - Track environment costs
- `CostCenter={Platform,Engineering}` - Allocate to cost centers
- `Team={backend,frontend,devops}` - Track team spending

### Apply Tags
```bash
# Update resource tags for cost allocation
./scripts/cost-allocation.sh
```

## Optimization Strategies

### Compute Optimization

#### ECS Right-sizing
- Review task CPU/memory utilization weekly
- Downsize consistently under-utilized tasks (<20% CPU)
- Use Fargate Spot for fault-tolerant workloads
- Implement scheduled scaling for predictable patterns

#### Spot Instance Usage
- Configure spot percentage: 70% for non-critical workloads
- Maintain baseline of on-demand instances
- Use spot instance diversification across instance types
- Implement graceful shutdown handling

### Storage Optimization

#### RDS
- Right-size instance class based on utilization
- Enable storage auto-scaling with 80% threshold
- Use Provisioned IOPS only when needed (>10K IOPS)
- Delete unused snapshots older than 30 days

#### S3
- Implement lifecycle policies for data tiering
- Transition infrequently accessed data to Glacier
- Enable Intelligent-Tiering for unpredictable access patterns
- Delete incomplete multipart uploads after 7 days

## Budget Alerts

### Alert Thresholds
| Threshold | Action |
|-----------|--------|
| 50% of budget | Email notification |
| 80% of budget | Email + Slack notification |
| 100% forecasted | Email + Slack + PagerDuty |

### Cost Anomaly Detection
- Monitors daily spending patterns
- Triggers on >10% unexplained cost increase
- Alerts sent to DevOps team

## Best Practices
1. Tag all resources for cost allocation
2. Review cost reports weekly
3. Set budget alerts for all environments
4. Clean up unused resources (EIPs, snapshots, load balancers)
5. Use managed services where cost-effective
6. Monitor data transfer costs between regions
7. Enable detailed billing reports
8. Review and delete unused resources monthly

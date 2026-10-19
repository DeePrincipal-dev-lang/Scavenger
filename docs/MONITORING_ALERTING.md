# Monitoring and Alerting

## Overview
Comprehensive monitoring using Prometheus for metrics collection, Grafana for visualization, AlertManager for alert routing, and the ELK stack for log aggregation.

## Components

### Prometheus
- Scrapes metrics from all services every 15s
- Stores time-series data with 30-day retention
- Evaluates alert rules from `config/prometheus-rules.yml`
- Metrics endpoints: 8080 (contract), 8081 (indexer), 9100 (node-exporter)

### Grafana
- Visualizes metrics via auto-provisioned dashboards
- Default credentials: `admin` / `admin` (set `GRAFANA_ADMIN_PASSWORD` env var for production)
- Access: http://localhost:3000
- Provisioned data source: Prometheus (`config/grafana/provisioning/datasources/`)
- Provisioned dashboards: `config/grafana/provisioning/dashboards/`

### AlertManager
- Routes and manages alerts to Slack channels
- Critical alerts → `#critical-alerts`, warnings → `#warnings`
- Groups related alerts, deduplicates notifications

### Node Exporter
- Collects host-level system metrics (CPU, memory, disk, network)
- Access: http://localhost:9100/metrics

### Log Aggregation (ELK Stack)
- **Elasticsearch** – stores and indexes log data (port 9200)
- **Logstash** – ingests logs via UDP (port 5000) or file (`/var/log/app/*.log`)
- **Kibana** – log search and visualization (port 5601)

## Key Metrics

### Application Metrics
- `contract_errors_total` - Total contract errors
- `contract_transaction_duration_seconds` - Transaction latency histogram
- `participants_registered_total` - Participant registrations
- `waste_submitted_total` - Waste submissions
- `incentives_distributed_total` - Incentive distributions

### Infrastructure Metrics
- `node_memory_MemAvailable_bytes` - Available memory
- `node_cpu_seconds_total` - CPU time by mode
- `node_filesystem_avail_bytes` - Disk space per mount

## Alert Rules

### Critical Alerts
- `HighErrorRate` – error rate >5% over 5 minutes
- `DiskSpaceLow` – disk space <10%
- `ServiceDown` – any target unreachable for 2 minutes
- `ContractServiceDown` – contract service down for 2 minutes
- `IndexerDown` – indexer service down for 2 minutes

### Warning Alerts
- `SlowTransactions` – P95 latency >5s for 10 minutes
- `HighMemoryUsage` – memory usage >85% for 5 minutes
- `HighCPUUsage` – CPU usage >80% for 5 minutes
- `LowParticipantActivity` – fewer than 1 registration/hour for 30 minutes
- `PrometheusTargetMissing` – scrape target missing for 5 minutes

## Grafana Dashboards

Dashboards are auto-provisioned on startup from `config/grafana/provisioning/dashboards/`.

| Dashboard | UID | Description |
|-----------|-----|-------------|
| Scavenger Overview | `scavenger-overview` | Transactions, error rate, latency, resource usage |

### Add a New Dashboard
1. Create or export a dashboard JSON from the Grafana UI
2. Place the `.json` file in `config/grafana/provisioning/dashboards/`
3. Restart Grafana or wait for the 30s reload interval

## Setup

### Start Monitoring Stack
```bash
docker-compose -f docker-compose.monitoring.yml up -d
```

### Start Log Aggregation Stack
```bash
docker-compose -f docker-compose.logging.yml up -d
```

### Configure Slack Notifications
```bash
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
docker-compose -f docker-compose.monitoring.yml up -d alertmanager
```

### Reload Prometheus Config (without restart)
```bash
curl -X POST http://localhost:9090/-/reload
```

## SLA Monitoring

- Transaction success rate: >99.5%
- P95 latency: <5 seconds
- Error rate: <0.1%
- Uptime: >99.9%

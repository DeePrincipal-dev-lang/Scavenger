# CDN Configuration

Scavenger uses AWS CloudFront for global content delivery, optimized cache policies, and monitored invalidation workflows.

## Architecture

- **Origin**: S3 bucket for static assets and frontend deployment
- **Distribution**: CloudFront CDN with edge locations worldwide
- **Cache Layers**: Path-based caching for static content, API traffic, and SPA fallback
- **SSL/TLS**: ACM certificate and HTTPS-only viewer policy

## Cache Policies

The CloudFront distribution is configured with path-based cache behavior:

- **`/static/*`**
  - Long-term caching for hashed assets
  - AWS managed caching optimized policy
  - Compression enabled
- **`/api/*`**
  - No caching for API traffic
  - All query strings and cookies forwarded to the origin
  - AWS managed caching disabled policy
- **Default behavior**
  - HTML/JS/CSS served with a shorter TTL and HTTPS redirect
  - Supports SPA routing via `index.html` fallback

## Setup

### Prerequisites

- AWS account with CloudFront and S3 permissions
- S3 bucket for asset hosting
- ACM certificate in `us-east-1` for the CDN custom domain

### Deployment

1. **Create S3 Origin**

```bash
aws s3 mb s3://scavenger-app-cdn
aws s3api put-bucket-versioning \
  --bucket scavenger-app-cdn \
  --versioning-configuration Status=Enabled
```

2. **Deploy CloudFront Distribution**

```bash
aws cloudfront create-distribution-with-tags \
  --distribution-config-with-tags file://config/cloudfront-distribution.yaml
```

3. **Update DNS**

```bash
# Point CNAME to CloudFront domain
# scavenger.example.com -> d111111abcdef8.cloudfront.net
```

## Cache Invalidation

Use the CDN invalidation helper script after frontend releases:

```bash
./scripts/cdn-invalidation.sh E1234567890ABC "/*"
```

Invalidate only changed assets:

```bash
./scripts/cdn-invalidation.sh E1234567890ABC "/index.html /app.js"
```

## Monitoring

Use CloudWatch and the CDN monitor script to track distribution health.

```bash
./scripts/cdn-monitor.sh E1234567890ABC
```

Key CloudFront metrics:

- `Requests`
- `BytesDownloaded`
- `4xxErrorRate`
- `5xxErrorRate`
- `CacheHitRate`

### Quick CloudWatch query

```bash
aws cloudwatch get-metric-statistics \
  --namespace AWS/CloudFront \
  --metric-name BytesDownloaded \
  --dimensions Name=DistributionId,Value=E1234567890ABC \
  --start-time 2026-05-28T00:00:00Z \
  --end-time 2026-05-29T00:00:00Z \
  --period 3600 \
  --statistics Sum
```

## Notes

- The distribution config uses path-based caching rules for static assets and API endpoints.
- Custom error responses route `403` and `404` to `index.html` for SPA fallback.
- Monitoring should be enabled for cache hit rate and HTTPS error spikes.

## Cost Optimization

- **Price Class**: PriceClass_100 (reduced edge locations)
- **Compression**: Reduces bandwidth by 60-80%
- **Cache Hit Ratio**: Target > 90%

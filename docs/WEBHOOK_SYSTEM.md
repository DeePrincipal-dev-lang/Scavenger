# Webhook System

## Overview

Webhooks enable real-time event notifications for important platform events.

## Supported Events

- `WasteRegistered` - When waste is submitted
- `WasteTransferred` - When waste is transferred between participants
- `WasteVerified` - When waste is verified
- `IncentiveCreated` - When a new incentive is created
- `IncentiveUpdated` - When an incentive is updated
- `RewardDistributed` - When rewards are distributed
- `ParticipantRegistered` - When a new participant registers

## API Endpoints

### Create Webhook
```
POST /api/webhooks
Content-Type: application/json

{
  "url": "https://example.com/webhook",
  "events": ["WasteRegistered", "WasteTransferred"]
}
```

Response:
```json
{
  "id": "webhook_123",
  "url": "https://example.com/webhook",
  "events": ["WasteRegistered", "WasteTransferred"],
  "active": true,
  "secret": "whsec_abc123",
  "created_at": "2024-05-30T10:00:00Z",
  "updated_at": "2024-05-30T10:00:00Z"
}
```

### List Webhooks
```
GET /api/webhooks
```

### Get Webhook
```
GET /api/webhooks/{id}
```

### Update Webhook
```
PUT /api/webhooks/{id}
Content-Type: application/json

{
  "url": "https://example.com/webhook-updated",
  "events": ["WasteRegistered"],
  "active": true
}
```

### Delete Webhook
```
DELETE /api/webhooks/{id}
```

## Webhook Payload

All webhook payloads follow this format:

```json
{
  "id": "evt_123",
  "event": "WasteRegistered",
  "timestamp": "2024-05-30T10:00:00Z",
  "data": {
    "waste_id": "waste_456",
    "submitter": "participant_789",
    "waste_type": "Plastic",
    "weight": 5.5
  }
}
```

## Security

Each webhook includes a secret token in the `X-Webhook-Secret` header. Verify this token to ensure the webhook is authentic.

## Retry Policy

Failed webhook deliveries are retried with exponential backoff:
- 1st retry: 5 seconds
- 2nd retry: 30 seconds
- 3rd retry: 5 minutes
- 4th retry: 30 minutes

After 4 failed attempts, the webhook is marked as failed.

## Testing

Test your webhook endpoint:
```bash
curl -X POST https://example.com/webhook \
  -H "Content-Type: application/json" \
  -H "X-Webhook-Secret: whsec_abc123" \
  -d '{
    "id": "evt_test",
    "event": "WasteRegistered",
    "timestamp": "2024-05-30T10:00:00Z",
    "data": {"waste_id": "test_123"}
  }'
```

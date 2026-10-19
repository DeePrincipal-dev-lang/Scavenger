#!/bin/bash
set -e

VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
VAULT_TOKEN="${VAULT_TOKEN}"

if [ -z "$VAULT_TOKEN" ]; then
  echo "Error: VAULT_TOKEN not set"
  exit 1
fi

echo "Initializing Vault secrets for Scavenger..."

# Enable KV v2 secrets engine
vault secrets enable -version=2 -path=secret kv || true

# Create API secrets
vault kv put secret/scavenger/api \
  contract_id="${CONTRACT_ID}" \
  network="${NETWORK:-TESTNET}" \
  rpc_url="${RPC_URL}" \
  firebase_api_key="${FIREBASE_API_KEY}" \
  firebase_auth_domain="${FIREBASE_AUTH_DOMAIN}" \
  firebase_project_id="${FIREBASE_PROJECT_ID}" \
  firebase_storage_bucket="${FIREBASE_STORAGE_BUCKET}" \
  firebase_messaging_sender_id="${FIREBASE_MESSAGING_SENDER_ID}" \
  firebase_app_id="${FIREBASE_APP_ID}" \
  firebase_measurement_id="${FIREBASE_MEASUREMENT_ID}"

# Create Stellar secrets
vault kv put secret/scavenger/stellar \
  secret_key="${STELLAR_SECRET_KEY}" \
  network="${NETWORK:-TESTNET}"

# Enable audit logging
vault audit enable file file_path=/vault/logs/audit.log || true

# Create policy for app access
vault policy write scavenger-policy - <<EOF
path "secret/data/scavenger/*" {
  capabilities = ["read", "list"]
}

path "secret/metadata/scavenger/*" {
  capabilities = ["list"]
}

path "auth/token/renew-self" {
  capabilities = ["update"]
}
EOF

# Create Kubernetes auth role
vault write auth/kubernetes/role/scavenger-app \
  bound_service_account_names=scavenger \
  bound_service_account_namespaces=default \
  policies=scavenger-policy \
  ttl=24h

echo "Vault initialization complete!"
